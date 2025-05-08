use nix::libc;
use std::fs::OpenOptions;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::io::RawFd;

const PCITOOL_VERSION: u32 = 1;
const PCITOOL_DEVICE_SET_INTR: i32 = 0x40087002;
const PCITOOL_SYSTEM_INTR_INFO: i32 = 0x40087003;
const PCITOOL_INTR_FLAG_SET_GROUP: u32 = 1;
const PCITOOL_CTLR_TYPE_PCPLUSMP: u32 = 1;
const PCITOOL_CTLR_TYPE_APIX: u32 = 2;

#[repr(C)]
struct PciToolIntrSet {
    old_cpu: i32,
    ino: i32,
    cpu_id: i32,
    flags: u32,
    user_version: u32,
}

#[repr(C)]
struct PciToolIntrInfo {
    user_version: u32,
    ctlr_type: u32,
}

fn open_dev(path: &str) -> io::Result<RawFd> {
    let dev_path = format!("/devices{}:intr", path);
    let file = OpenOptions::new().read(true).write(true).open(dev_path)?;
    Ok(file.as_raw_fd())
}

pub fn intrmove(path: &str, oldcpu: i32, ino: i32, cpu: i32, num_ino: i32) -> io::Result<()> {
    let fd = open_dev(path)?;

    let flags = if num_ino > 1 {
        PCITOOL_INTR_FLAG_SET_GROUP
    } else {
        0
    };

    let iset = PciToolIntrSet {
        cpu_id: cpu,
        flags,
        ino,
        old_cpu: oldcpu,
        user_version: PCITOOL_VERSION,
    };

    let ret = unsafe { libc::ioctl(fd, PCITOOL_DEVICE_SET_INTR, &iset) };

    if ret == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn is_apic(path: &str) -> io::Result<bool> {
    let fd = open_dev(path)?;

    let mut intr_info = PciToolIntrInfo {
        ctlr_type: 0,
        user_version: PCITOOL_VERSION,
    };

    let ret = unsafe { libc::ioctl(fd, PCITOOL_SYSTEM_INTR_INFO, &mut intr_info) };

    if ret == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(intr_info.ctlr_type == PCITOOL_CTLR_TYPE_PCPLUSMP
            || intr_info.ctlr_type == PCITOOL_CTLR_TYPE_APIX)
    }
}
