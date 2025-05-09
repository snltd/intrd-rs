use crate::util::constants::{
    PCITOOL_CTLR_TYPE_APIX, PCITOOL_CTLR_TYPE_PCPLUSMP, PCITOOL_SYSTEM_INTR_INFO, PCITOOL_VERSION,
};
use anyhow::anyhow;
use libc::ioctl;
use std::fs::OpenOptions;
use std::io;
use std::os::fd::AsRawFd;

#[repr(C)]
pub struct PciToolIntrInfo {
    user_version: u16,
    drvr_version: u16,
    flags: u32,
    num_intr: u32,
    num_cpu: u32,
    ctlr_version: u32,
    ctlr_type: u8,
}

pub fn is_apic(buspath: &str) -> anyhow::Result<bool> {
    let path = format!("/devices{}:intr", buspath);
    let file = OpenOptions::new().read(true).open(path)?;
    let fd = file.as_raw_fd();

    let mut iinfo = PciToolIntrInfo {
        user_version: PCITOOL_VERSION,
        drvr_version: 0,
        flags: 0,
        num_intr: 0,
        num_cpu: 0,
        ctlr_version: 0,
        ctlr_type: 0,
    };

    let ret = unsafe { ioctl(fd, PCITOOL_SYSTEM_INTR_INFO.try_into().unwrap(), &mut iinfo) };

    if ret == -1 {
        return Err(anyhow!(io::Error::last_os_error()));
    }

    Ok(iinfo.ctlr_type == PCITOOL_CTLR_TYPE_PCPLUSMP || iinfo.ctlr_type == PCITOOL_CTLR_TYPE_APIX)
}

// pub fn intrmove(path: &str, oldcpu: i32, ino: i32, cpu: i32, num_ino: i32) -> io::Result<()> {
//     let fd = open_dev(path)?;

//     let flags = if num_ino > 1 {
//         PCITOOL_INTR_FLAG_SET_GROUP
//     } else {
//         0
//     };

//     let iset = PciToolIntrSet {
//         cpu_id: cpu,
//         flags,
//         ino,
//         old_cpu: oldcpu,
//         user_version: PCITOOL_VERSION,
//     };

//     let ret = unsafe { libc::ioctl(fd, PCITOOL_DEVICE_SET_INTR, &iset) };

//     if ret == -1 {
//         Err(io::Error::last_os_error())
//     } else {
//         Ok(())
//     }
// }
