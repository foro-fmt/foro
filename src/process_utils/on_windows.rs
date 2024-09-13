use anyhow::{Context, Result};
use log::trace;
use std::mem::zeroed;
use std::ptr::null_mut;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use winapi::shared::minwindef::FILETIME;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetProcessTimes, OpenProcess};
use winapi::um::winnt::{HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

pub fn is_alive(pid: u32) -> bool {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() {
            return false;
        }
        CloseHandle(handle);
        true
    }
}

pub fn get_start_time(pid: u32) -> Result<u64> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() {
            return Err(anyhow::anyhow!("Failed to open process: {}", pid));
        }

        let mut creation_time: FILETIME = zeroed();
        let mut exit_time: FILETIME = zeroed();
        let mut kernel_time: FILETIME = zeroed();
        let mut user_time: FILETIME = zeroed();

        if GetProcessTimes(
            handle,
            &mut creation_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        ) == 0
        {
            CloseHandle(handle);
            return Err(anyhow::anyhow!("Failed to get process times"));
        }
        trace!("1");

        let creation_time_u64 =
            ((creation_time.dwHighDateTime as u64) << 32) | (creation_time.dwLowDateTime as u64);
        let start_system_time = UNIX_EPOCH + Duration::from_nanos(creation_time_u64 * 100);
        let start_time = start_system_time
            .duration_since(UNIX_EPOCH)
            .context("Failed to calculate process start time")?
            .as_secs();

        CloseHandle(handle);
        Ok(start_time)
    }
}
