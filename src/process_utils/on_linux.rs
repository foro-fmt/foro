use anyhow::{anyhow, Context, Result};
use nix::unistd::{sysconf, Pid, SysconfVar};
use std::fs;
use std::path::PathBuf;

pub fn is_alive(pid: u32) -> bool {
    let pid = Pid::from_raw(pid as i32);
    PathBuf::from(format!("/proc/{}", pid.as_raw())).exists()
}

fn get_boot_time() -> Result<u64> {
    let proc_stat = fs::read_to_string(&"/proc/stat")?;

    for line in proc_stat.lines() {
        if line.starts_with("btime") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 2 {
                if let Ok(btime) = parts[1].parse::<u64>() {
                    return Ok(btime);
                }
            } else {
                return Err(anyhow!("Failed to parse btime"));
            }
        }
    }

    Err(anyhow!("Failed to find btime"))
}

fn get_start_time_clock_tick(pid: u32) -> Result<u64> {
    let pid = Pid::from_raw(pid as i32);

    let stat_path = format!("/proc/{}/stat", pid.as_raw());
    let stat_content = fs::read_to_string(&stat_path)
        .with_context(|| format!("Failed to read stat file: {}", stat_path))?;

    let start_time = stat_content
        .split_whitespace()
        .nth(21)
        .context("Failed to parse start time")?
        .parse::<u64>()
        .context("Failed to convert start time to u64")?;

    Ok(start_time)
}

pub fn get_start_time(pid: u32) -> Result<u64> {
    let clock_ticks_per_second = sysconf(SysconfVar::CLK_TCK)
        .context("Failed to get clock ticks per second")?
        .ok_or_else(|| anyhow!("Failed to get clock ticks per second"))?
        as u64;

    let start_time_clock_tick = get_start_time_clock_tick(pid)?;
    let start_time_sec = start_time_clock_tick / clock_ticks_per_second;

    let boot_time = get_boot_time()?;
    Ok(boot_time + start_time_sec)
}
