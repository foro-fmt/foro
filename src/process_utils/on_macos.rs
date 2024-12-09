use anyhow::{anyhow, Result};
use libc::{proc_bsdinfo, proc_pidinfo, PROC_PIDTBSDINFO};
use std::mem;

pub fn is_alive(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as i32, 0) };
    result == 0
}

pub fn get_start_time(pid: u32) -> Result<u64> {
    let mut bsd_info: proc_bsdinfo = unsafe { mem::zeroed() };

    let size = size_of::<proc_bsdinfo>() as i32;
    let ret = unsafe {
        proc_pidinfo(
            pid as i32,
            PROC_PIDTBSDINFO,
            0,
            &mut bsd_info as *mut _ as *mut libc::c_void,
            size,
        )
    };

    if ret <= 0 {
        return Err(anyhow!("Failed to retrieve process info"));
    }

    let start_time = bsd_info.pbi_start_tvsec;
    Ok(start_time)
}
