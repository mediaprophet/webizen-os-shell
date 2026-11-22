//! Device OS host environment summary (Ring A substrate facts).
//!
//! Records only current session host facts (hostname, OS, architecture,
//! active Device OS user, total physical RAM, and logical CPU threads).
//! Never scrapes other user home directories or invents shadow auth keys.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OsEnvironment {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub username: String,
    pub ram_bytes: Option<u64>,
    pub cpu_count: Option<u32>,
    pub cpu_brand: Option<String>,
}

impl OsEnvironment {
    pub fn probe(username: &str) -> Self {
        let hostname = hostname();
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();
        let ram_bytes = total_ram_bytes();
        let cpu_count = std::thread::available_parallelism()
            .ok()
            .map(|n| n.get() as u32);
        let cpu_brand = std::env::var("PROCESSOR_IDENTIFIER").ok().or_else(|| {
            std::fs::read_to_string("/proc/cpuinfo")
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.to_ascii_lowercase().starts_with("model name"))
                        .map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string())
                })
        });
        Self {
            hostname,
            os,
            arch,
            username: username.to_string(),
            ram_bytes,
            cpu_count,
            cpu_brand,
        }
    }
}

fn hostname() -> String {
    for key in ["COMPUTERNAME", "HOSTNAME"] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "localhost".into())
}

fn total_ram_bytes() -> Option<u64> {
    #[cfg(windows)]
    {
        windows_total_ram()
    }
    #[cfg(not(windows))]
    {
        unix_total_ram()
    }
}

#[cfg(windows)]
fn windows_total_ram() -> Option<u64> {
    #[repr(C)]
    struct MemoryStatusEx {
        dw_length: u32,
        dw_memory_load: u32,
        ull_total_phys: u64,
        ull_avail_phys: u64,
        ull_total_page_file: u64,
        ull_avail_page_file: u64,
        ull_total_virtual: u64,
        ull_avail_virtual: u64,
        ull_avail_extended_virtual: u64,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalMemoryStatusEx(lp: *mut MemoryStatusEx) -> i32;
    }
    let mut st = MemoryStatusEx {
        dw_length: std::mem::size_of::<MemoryStatusEx>() as u32,
        dw_memory_load: 0,
        ull_total_phys: 0,
        ull_avail_phys: 0,
        ull_total_page_file: 0,
        ull_avail_page_file: 0,
        ull_total_virtual: 0,
        ull_avail_virtual: 0,
        ull_avail_extended_virtual: 0,
    };
    let ok = unsafe { GlobalMemoryStatusEx(&mut st) };
    if ok != 0 && st.ull_total_phys > 0 {
        Some(st.ull_total_phys)
    } else {
        None
    }
}

#[cfg(not(windows))]
fn unix_total_ram() -> Option<u64> {
    if let Ok(s) = std::fs::read_to_string("/proc/meminfo") {
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                let kb: u64 = rest
                    .split_whitespace()
                    .next()
                    .and_then(|n| n.parse().ok())?;
                return Some(kb.saturating_mul(1024));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_has_username_and_hostname() {
        let e = OsEnvironment::probe("timothy");
        assert_eq!(e.username, "timothy");
        assert!(!e.hostname.is_empty());
    }
}
