// from is_apic.rs
pub const PCITOOL_CTLR_TYPE_APIX: u8 = 4;
pub const PCITOOL_CTLR_TYPE_PCPLUSMP: u8 = 3;
pub const PCITOOL_SYSTEM_INTR_INFO: u32 =
    (('P' as u32) << 24) | (('C' as u32) << 16) | (('T' as u32) << 8) | 8;
pub const PCITOOL_VERSION: u16 = 2;

// from intr_binding.rs
//
// const PCITOOL_VERSION: u32 = 1;
// const PCITOOL_DEVICE_SET_INTR: i32 = 0x40087002;
// const PCITOOL_SYSTEM_INTR_INFO: i32 = 0x40087003;
// const PCITOOL_INTR_FLAG_SET_GROUP: u32 = 1;
// const PCITOOL_CTLR_TYPE_PCPLUSMP: u32 = 1;
// const PCITOOL_CTLR_TYPE_APIX: u32 = 2;

// from main.rs
//
pub const SYSLOG_PROCESS_NAME: &str = "intrd-rs"; // my name
pub const USING_SCENGEN: bool = false; // I probably won't implement this.
pub const NORMAL_SLEEP_TIME: u64 = 10; // time to sleep between samples

// pub const IDLE_SLEEP_TIME: u64 = 45; // time to sleep when idle
// pub const SINGLE_CPU_SLEEP_TIME: u64 = (60 * 15); // used only on single CPU systems
// pub const IDLE_INTR_LOAD: f32 = 0.1; // idle if interrupt load < 10%
// pub const TIME_RANGE_TOO_HIGH: f32 = 0.01;
// pub const STATS_LEN: usize = 60; // time period (in secs) to keep in deltas
