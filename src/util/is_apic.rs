use anyhow::anyhow;
use libc::ioctl;
use std::fs::OpenOptions;
use std::io;
use std::os::fd::AsRawFd;

const PCITOOL_CTLR_TYPE_APIX: u8 = 4;
const PCITOOL_CTLR_TYPE_PCPLUSMP: u8 = 3;
const PCITOOL_SYSTEM_INTR_INFO: u32 =
    (('P' as u32) << 24) | (('C' as u32) << 16) | (('T' as u32) << 8) | 8;
const PCITOOL_VERSION: u16 = 2;

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
