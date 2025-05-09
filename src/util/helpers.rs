use crate::util::is_apic;
use anyhow::anyhow;
use kstat_rs::{Ctl, Kstat, NamedData};
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub fn setup_signal_handler() -> Arc<AtomicBool> {
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

pub fn in_debug_mode() -> anyhow::Result<bool> {
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

pub fn setup_logger() -> std::result::Result<(), log::SetLoggerError> {
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

// I'm not convinced we need this check. Feels like vestigial SPARC support to me.
pub fn is_apic_system(ctl: &Ctl, first_stat: &mut Kstat) -> anyhow::Result<bool> {
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

// sub getstat($$);
// sub generate_delta($$);
// sub compress_deltas($);
// sub dumpdelta($);

// sub goodness($);
// sub imbalanced($$);
// sub do_reconfig($);

// sub goodness_cpu($$);		# private function
// sub move_intr($$$$);		# private function
// sub ivecs_to_string(@);		# private function
// sub do_find_goal($$$$);		# private function
// sub find_goal($$);		# private function
// sub do_reconfig_cpu2cpu($$$$);	# private function
// sub do_reconfig_cpu($$$);	# private function
