use crate::util::is_apic;
use kstat_rs::{Ctl, Kstat, NamedData};
use log::{debug, info};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;
mod util;
use anyhow::anyhow;
use anyhow::Context;
use std::env;

const SYSLOG_PROCESS_NAME: &str = "intrd-rs";

// my $normal_sleeptime = 10;		# time to sleep between samples
const NORMAL_SLEEP_TIME: usize = 10;
// my $idle_sleeptime = 45;		# time to sleep when idle
const IDLE_SLEEP_TIME: usize = 45;
// my $onecpu_sleeptime = (60 * 15);	# used if only 1 CPU on system
const SINGLE_CPU_SLEEP_TIME: usize = (60 * 15);
//
// my $sleeptime = $normal_sleeptime;	# either normal_ or idle_ or onecpu_

// my $idle_intrload = .1;			# idle if interrupt load < 10%
const IDLE_INTR_LOAD: f32 = 0.1;

// my $timerange_toohi    = .01;
const TIME_RANGE_TOO_HIGH: f32 = 0.01;
// my $statslen = 60;	# time period (in secs) to keep in @deltas
const STATS_LEN: usize = 60;

fn setup_signal_handler() -> Arc<AtomicBool> {
    let gotsig = Arc::new(AtomicBool::new(false));
    let mut signals = Signals::new([SIGINT, SIGHUP, SIGTERM]).expect("Failed to register signals");

    let gotsig_clone = Arc::clone(&gotsig);

    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal: {}", sig);
            gotsig_clone.store(true, Ordering::SeqCst);
        }
    });

    gotsig
}

fn get_pci_intr_kstats() {
    todo!()
}

fn getstat() {
    // 2 args
    todo!()
}
fn generate_delta() {
    // 2 args
    todo!()
}
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

fn in_debug_mode() -> anyhow::Result<bool> {
    let args: Vec<String> = env::args().collect();

    // Parse arguments. intrd does not accept any public arguments; the two
    // arguments below are meant for testing purposes. -D generates a significant
    // amount of syslog output. -S <filename> loads the filename as a perl
    // script. That file is expected to implement a kstat "simulator" which
    // can be used to feed information to intrd and verify intrd's responses.

    if args.len() > 1 {
        return Err(anyhow!("The only valid argument is '-D' or '--debug'."));
    }

    Ok(args.len() == 2 && (args[1] == "-D" || args[1] == "--debug"))
}

fn setup_logger() -> std::result::Result<(), log::SetLoggerError> {
    // TODO intrd logs to syslog. Logging to syslog from Rust on illumos is a pain, because none
    // of the syslog crates I can find will write STREAMS. So we'll just use simplelog for now. In
    // the unlikely event of this thing ever being completed and proven good, I'll write a syslog
    // interface.
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
}

fn is_apic_system(ctl: &Ctl, first_stat: &mut Kstat) -> anyhow::Result<bool> {
    let bus_path = match ctl.read(first_stat) {
        Ok(kstat_rs::Data::Named(stat)) => {
            stat.iter()
                .find(|s| s.name == "buspath")
                .and_then(|buspath_stat| {
                    if let NamedData::String(val) = buspath_stat.value {
                        Some(val)
                    } else {
                        None
                    }
                })
        }
        Ok(_) => None,
        Err(_) => None,
    };

    match bus_path {
        Some(path) => is_apic::is_apic(path),
        None => Err(anyhow!("Could not find buspath kstat")),
    }
}

fn main() -> anyhow::Result<()> {
    setup_logger().context("Failed to instantiate logger")?;
    let debug = in_debug_mode()?;

    if debug {
        debug!("{} is starting (debug)", SYSLOG_PROCESS_NAME);
    } else {
        info!("{} is starting", SYSLOG_PROCESS_NAME);
    }

    let gotsig = setup_signal_handler();

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
    let ctl = Ctl::new().context("Cannot get kstat handle")?;

    let mut intr_stats: Vec<_> = ctl.filter(Some("pci_intrs"), None, None).collect();

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

    let is_apic = is_apic_system(&ctl, first_stat);

    debug!("APIC system: {:?}", is_apic);

    // This will end up being something more sophisticated, I'm sure
    // type Delta = usize;

    // type Deltas = Vec<Delta>;

    // type DeltasTotalTime = usize;

    while !gotsig.load(Ordering::SeqCst) {
        // clear deltas
        // let mut deltas: Deltas = Vec::new();
        // let mut deltas_total_time: DeltasTotalTime = 0;
        // let mut stat = 0;

        // # 1. Sleep, update the kstats, and save the new stats in $newstat.
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
    // #
    // # {"snaptime"}          kstat's snaptime
    // # {<cpuid>}             one hash reference per online cpu
    // #  ->{"tot"}            == cpu:<cpuid>:sys:cpu_nsec_{user + kernel + idle}
    // #  ->{"crtime"}         == cpu:<cpuid>:sys:crtime
    // #  ->{"ivecs"}
    // #     ->{<cookie#>}     iterates over pci_intrs::<nexus>:cookie
    // #        ->{"time"}     == pci_intrs:<ivec#>:<nexus>:time (in nsec)
    // #        ->{"pil"}      == pci_intrs:<ivec#>:<nexus>:pil
    // #        ->{"crtime"}   == pci_intrs:<ivec#>:<nexus>:crtime
    // #        ->{"ino"}      == pci_intrs:<ivec#>:<nexus>:ino
    // #        ->{"num_ino"}  == num inos of single device instance sharing this entry
    // #				Will be > 1 on pcplusmp X86 systems for devices
    // #				with multiple MSI interrupts.
    // #        ->{"buspath"}  == pci_intrs:<ivec#>:<nexus>:buspath
    // #        ->{"name"}     == pci_intrs:<ivec#>:<nexus>:name
    // #        ->{"ihs"}      == pci_intrs:<ivec#>:<nexus>:ihs
    // #
    //
    //

    Ok(())
}
