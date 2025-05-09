use crate::util::constants::{NORMAL_SLEEP_TIME, SYSLOG_PROCESS_NAME, USING_SCENGEN};
use kstat_rs::NamedData::Char as KChar;
use kstat_rs::NamedData::String as KString;
use kstat_rs::NamedData::UInt64;

use crate::util::helpers;
use kstat_rs::{Ctl, Data, Kstat, Named, NamedData, NamedType};
use log::{debug, info};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
mod util;
use anyhow::anyhow;
use anyhow::Context;

// This will end up being something more sophisticated, I'm sure
type Delta = usize;
type Deltas = Vec<Delta>;
type DeltasTotalTime = usize;

struct Ivecs {
    //     ->{<cookie#>}     iterates over pci_intrs::<nexus>:cookie
    time: u64,       // pci_intrs:<ivec#>:<nexus>:time (in nsec)
    pil: u64,        // pci_intrs:<ivec#>:<nexus>:pil
    crtime: u64,     // pci_intrs:<ivec#>:<nexus>:crtime
    ino: u64,        // pci_intrs:<ivec#>:<nexus>:ino
    num_ino: u64, // num inos of single device instance sharing this entry. Will be > 1 on pcplusmp X86 systems for devices with multiple MSI interrupts.
    buspath: String, // pci_intrs:<ivec#>:<nexus>:buspath
    name: String, // pci_intrs:<ivec#>:<nexus>:name
    ihs: u64,     // pci_intrs:<ivec#>:<nexus>:ihs
}

struct CpuStat {
    tot: u64,    // cpu:<cpuid>:sys:cpu_nsec_{user + kernel + idle}
    crtime: u64, // cpu:<cpuid>:sys:crtime
    ivecs: Ivecs,
}

struct GotStat {
    snaptime: u64,                 // kstat's snaptime
    cpus: HashMap<CpuId, CpuStat>, // one hash reference per online cpu
}

// # getstat() is responsible for reading the kstats and generating a "stat" hash.
// #
// # generate_delta() is responsible for taking two "stat" hashes and creating
// # a new "delta" hash that represents what has changed over time.
// #
// # compress_deltas() is responsible for taking a list of deltas and generating
// # a single delta hash that encompasses all the time periods described by the
// # deltas.
// # getstat() is handed a reference to a kstat and generates a hash, returned
// # by reference, containing all the fields from the kstats which we need.
// # If it returns the scalar 0, it failed to gather the kstats, and the caller
// # should react accordingly.
// #
// # getstat() is also responsible for maintaining a reasonable $sleeptime.

fn get_string_value(stat: &Named) -> anyhow::Result<String> {
    if let NamedData::Char(bytes) = stat.value {
        Ok(std::str::from_utf8(bytes)?.trim_matches('\0').to_string())
    } else {
        Err(anyhow!("not valid UTF8"))
    }
}

type CpuId = i32;

fn online_cpus(ctl: &Ctl) -> anyhow::Result<Vec<CpuId>> {
    Ok(ctl
        .filter(Some("cpu_info"), None, None)
        .filter_map(|mut cpu| {
            if let Ok(Data::Named(val)) = ctl.read(&mut cpu) {
                val.iter().find(|x| x.name == "state").and_then(|c| {
                    if get_string_value(c).ok()? == "on-line" {
                        Some(cpu.ks_instance)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .collect())
}

fn getstat(ctl: &Ctl, is_apic: bool) -> anyhow::Result<Option<GotStat>> {
    // kstats are not generated atomically. Each kstat hierarchy will
    // have been generated within the kernel at a different time. On a
    // thrashing system, we may not run quickly enough in order to get
    // coherent kstat timing information across all the kstats. To
    // determine if this is occurring, $minsnap/$maxsnap are used to
    // find the breadth between the first and last snaptime of all the
    // kstats we access. $maxsnap - $minsnap roughly represents the
    // total time taken up in getstat(). If this time approaches the
    // time between snapshots, our results may not be useful.

    // let mut minsnap = -1;
    // let mut maxsnap = -1;

    // Hash of hash which matches (MSI device, ino) combos to kstats.
    // my %msidevs = ();

    // Record cpu:<cpuid>:sys:snaptime, and check $minsnap/$maxsnap.

    let online_cpus = online_cpus(&ctl)?;

    if online_cpus.len() <= 1 {
        return Ok(None);
    }

    let mut snaptimes = Vec::new();

    for cpu_id in &online_cpus {
        let mut cpu_sys: Vec<_> = ctl
            .filter(Some("cpu"), Some(*cpu_id), Some("sys"))
            .collect();
        // This is bound to be a single element vector
        let mut this_cpu = cpu_sys
            .first_mut()
            .context(format!("failed to find cpu:{}:sys", cpu_id))?;

        snaptimes.push(this_cpu.ks_snaptime);

        if let Ok(Data::Named(val)) = ctl.read(this_cpu) {
            let vals = numeric_value_map(&val);
            // I'm surprised we count idle time, but that's what the original does
            let cpu_time: u64 = numeric_value_map(&val)
                .iter()
                .filter_map(|(k, v)| k.starts_with("cpu_nsec").then_some(*v))
                .sum();
            let cpu_crtime = vals.get("crtime").context("could not get crtime");
        }
    }

    let mut pci_intrs: Vec<_> = ctl.filter(Some("pci_intrs"), None, None).collect();

    for mut pci_intr in pci_intrs {
        // println!("{:?}", pci_intr);
        // let vals = numeric_value_map(&val);
        // let mut this_cpu = pci_intr
        //     .first_mut()
        //     .context("failed to find pci_intr kstat")?;
        // println!("----------------------");
        // println!("{:#?}", this_cpu);
        // for mut x in pci_intrs {
        snaptimes.push(pci_intr.ks_snaptime);

        if let Ok(Data::Named(val)) = ctl.read(&mut pci_intr) {
            let nvals = numeric_value_map(&val);
            let svals = string_value_map(&val);
            let cpu_id = nvals.get("cpu").context("failed to get pci_intr cpu")?;

            if !&online_cpus.contains(&(*cpu_id as i32)) {
                continue;
            }

            if *svals.get("type").context("failed to get pci_intr type")? == "disabled" {
                continue;
            }
        }
        // }
    }

    todo!()
}
type KstatNumericValueMap<'a> = HashMap<&'a str, u64>;
type KstatStrinValueMap<'a> = HashMap<&'a str, &'a str>;
// use kstat::Data::Named;
// use kstat::NamedVal::UInt64;

fn numeric_value_map<'a>(named: &'a [Named]) -> KstatNumericValueMap<'a> {
    named
        .iter()
        .filter_map(|n| match n.value {
            UInt64(v) => Some((n.name, v)),
            _ => None,
        })
        .collect()
}

fn string_value_map<'a>(named: &'a [Named]) -> KstatStrinValueMap<'a> {
    named
        .iter()
        .filter_map(|n| match n.value {
            KString(v) => Some((n.name, v)),
            KChar(v) => Some((n.name, std::str::from_utf8(v).unwrap().trim_matches('\0'))),
            _ => None,
        })
        .collect()
}

// generate_delta() is responsible for taking two "stat" hashes and creating a new "delta" hash
// that represents what has changed over time.
fn generate_delta() {
    // 2 args
    todo!()
}

// compress_deltas() is responsible for taking a list of deltas and generating a single delta hash that encompasses all the time periods described by the deltas.
fn compress_deltas() {
    // 1 arg
    todo!()
}
fn dumpdelta() {
    // 1 arg
    todo!()
}
fn goodness() {
    // 1 arg
    todo!()
}
fn imbalanced() {
    // 2 args
    todo!()
}
fn do_reconfig() {
    // 1 arg
    todo!()
}
fn goodness_cpu() {
    // 2 args
    todo!()
}
fn move_intr() {
    // 4 args
    todo!()
}
fn ivecs_to_string() {
    // 1 vec arg
    todo!()
}
fn do_find_goal() {
    // 4 args
    todo!()
}
fn find_goal() {
    // 2 args
    todo!()
}
fn do_reconfig_cpu2cpu() {
    // 4 args
    todo!()
}
fn do_reconfig_cpu() {
    // 3 args
    todo!()
}

fn main() -> anyhow::Result<()> {
    helpers::setup_logger().context("Failed to instantiate logger")?;
    let debug = helpers::in_debug_mode()?;

    if debug {
        debug!("{} is starting (debug)", SYSLOG_PROCESS_NAME);
    } else {
        info!("{} is starting", SYSLOG_PROCESS_NAME);
    }

    let gotsig = helpers::setup_signal_handler();

    // let mut deltas = Vec::new();
    // my @deltas = ();
    // let mut deltas_tottime = 0;
    // my $deltas_tottime = 0;		# sum of maxsnap-minsnap across @deltas
    // let avggoodness;
    // my $avggoodness;
    // let mut baseline_goodness = 0;
    // my $baseline_goodness = 0;
    // let compdelta;
    // my $compdelta;
    // let do_reconfig;
    // my $do_reconfig;
    // d
    // // # temp variables
    // let goodness;
    // // my $goodness;
    // let deltatime;
    // // my $deltatime;
    // let olddelta;
    // // my $olddelta;
    // let olddeltatime;
    // // my $olddeltatime;
    // let delta;
    // // my $delta;
    // let newstat;
    // // my $newstat;
    // let below_statslen;
    // // my $below_statslen;
    // let newtime;
    // // my $newtime;
    // let ret;
    // my $ret;
    //
    let mut ctl = Ctl::new().context("Cannot get kstat handle")?;
    let mut intr_stats: Vec<_> = ctl.filter(Some("pci_intrs"), None, None).collect();

    // From inside loop
    let stats = getstat(&ctl, true);

    // # If no pci_intrs kstats were found, we need to exit, but we can't because
    // # SMF will restart us and/or report an error to the administrator. But
    // # there's nothing an administrator can do. So print out a message for SMF
    // # logs and silently pause forever.

    let first_stat = match intr_stats.first_mut() {
        Some(stat) => stat,
        None => {
            eprintln!(
                "{}: no interrupts were found; your PCI bus may not yet be supported",
                SYSLOG_PROCESS_NAME,
            );

            while !gotsig.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(100));
            }

            std::process::exit(0);
        }
    };

    // # See if this is a system with a pcplusmp APIC.
    // # Such systems will get special handling.
    // # Assume that if one bus has a pcplusmp APIC that they all do.

    let is_apic = helpers::is_apic_system(&ctl, first_stat)?;

    debug!("APIC system: {:?}", is_apic);
    let mut sleep_time = NORMAL_SLEEP_TIME;

    while !gotsig.load(Ordering::SeqCst) {
        // clear deltas
        // let mut deltas: Deltas = Vec::new();
        // let mut deltas_total_time: DeltasTotalTime = 0;

        debug!("loop!");
        // let mut stat = 0;

        // 1. Sleep, update the kstats, and save the new stats in $newstat.
        //
        if USING_SCENGEN {
            debug!("scenario generator is not implemented");
        } else {
            thread::sleep(Duration::from_secs(sleep_time));
            ctl = ctl.update()?;
        }

        let stats = getstat(&ctl, is_apic);

        // # 2. Compare $newstat with the prior set of values, result in %$delta.

        // # 3. If $delta->{missing}, then there has been a reconfiguration of
        // # either cpus or interrupts (probably both). We need to toss out our
        // # old set of statistics and start from scratch.
        // #
        // # Also, if the delta covers a very long range of time, then we've
        // # been experiencing a system overload that has resulted in intrd
        // # not being allowed to run effectively for a while now. As above,
        // # toss our old statistics and start from scratch.

        // # 4. Incorporate new delta into the list of deltas, and associated
        // # statistics. If we've just now received $statslen deltas, then it's
        // # time to evaluate a reconfiguration.
        //
        //
        //
        // # 5. Remove old deltas if total time is more than $statslen. We use
        // # @deltas as a moving average of the last $statslen seconds. Shift
        // # off the olders deltas, but only if that doesn't cause us to fall
        // # below $statslen seconds.
        //
        //
        // # 6. The brains of the operation are here. First, check if we're
        // # imbalanced, and if so set $do_reconfig. If $do_reconfig is set,
        // # either because of imbalance or above in step 4, we evaluate a
        // # new configuration.
        // #
        // # First, take @deltas and generate a single "compressed" delta
        // # which summarizes them all. Pass that to do_reconfig and see
        // # what it does with it:
        // #
        // # $ret == -1 : failure
        // # $ret ==  0 : current config is optimal (or close enough)
        // # $ret ==  1 : reconfiguration has occurred
        // #
        // # If $ret is -1 or 1, dump all our deltas and start from scratch.
        // # Step 4 above will set do_reconfig soon thereafter.
        // #
        // # If $ret is 0, then nothing has happened because we're already
        // # good enough. Set baseline_goodness to current goodness.
    }

    Ok(())
}
