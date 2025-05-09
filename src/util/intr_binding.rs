// use nix::libc;
// use std::fs::OpenOptions;
// use std::io;
// use std::os::fd::AsRawFd;
// use std::os::unix::io::RawFd;

// #[repr(C)]
// struct PciToolIntrSet {
//     old_cpu: i32,
//     ino: i32,
//     cpu_id: i32,
//     flags: u32,
//     user_version: u32,
// }

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
