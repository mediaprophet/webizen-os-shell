//! Device OS host environment summary (Ring A substrate facts).
//!
//! Records only current session host facts (hostname, OS, architecture,
//! active Device OS user, total physical RAM, and logical CPU threads).
//! Never scrapes other user home directories or invents shadow auth keys.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wsl2CompatibilityMode {
    /// Running directly on Debian 13 (`trixie`) / Linux as the user's primary OS (`SKU B Appliance`).
    NativeLinuxPrimaryOs,
    /// Running inside a WSL2 Linux guest (`microsoft-standard-WSL2` kernel).
    InsideWsl2LinuxGuest,
    /// Running on a Windows host that has an active WSL2 Linux subsystem available to bridge
    /// kernel-level `eBPF`, `nftables`, `ext4` `mmap`, and `webizen-hostd.service`.
    WindowsHostWithWsl2KernelBridgeAvailable,
    /// Running natively on Windows without an active WSL2 subsystem (falls back to user-space
    /// `webizen-hostd` enforcement without Linux kernel `eBPF` / `nftables` offload).
    WindowsNativeUserSpaceOnly,
    /// Running on macOS or another POSIX host.
    OtherPosixHost,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wsl2BridgeAndPrimaryOsPortabilityPlan {
    pub detected_mode: Wsl2CompatibilityMode,
    pub wsl2_kernel_ebpf_and_nftables_offload_ready: bool,
    pub ext4_vhdx_mmap_persistence_path: String,
    pub primary_os_migration_deb_package: String,
    pub phone_pocket_cosigner_reanchor_required: bool,
    pub portable_container_formats: Vec<String>,
    pub wsl2_daemon_bridge_command: String,
    pub primary_os_cutover_command: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OsEnvironment {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub username: String,
    pub ram_bytes: Option<u64>,
    pub cpu_count: Option<u32>,
    pub cpu_brand: Option<String>,
    pub wsl2_mode: Wsl2CompatibilityMode,
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
        let wsl2_mode = probe_wsl2_compatibility_mode(&os);
        Self {
            hostname,
            os,
            arch,
            username: username.to_string(),
            ram_bytes,
            cpu_count,
            cpu_brand,
            wsl2_mode,
        }
    }

    /// Evaluates the WSL2 Linux kernel bridge benefits on Windows and produces a deterministic
    /// Windows-to-Webizen Primary OS (Debian 13 `trixie` `webizen-desktop-appliance`) portability plan.
    pub fn build_wsl2_and_primary_os_portability_plan(
        &self,
    ) -> Wsl2BridgeAndPrimaryOsPortabilityPlan {
        let ebpf_ready = matches!(
            self.wsl2_mode,
            Wsl2CompatibilityMode::NativeLinuxPrimaryOs
                | Wsl2CompatibilityMode::InsideWsl2LinuxGuest
                | Wsl2CompatibilityMode::WindowsHostWithWsl2KernelBridgeAvailable
        );
        Wsl2BridgeAndPrimaryOsPortabilityPlan {
            detected_mode: self.wsl2_mode,
            wsl2_kernel_ebpf_and_nftables_offload_ready: ebpf_ready,
            ext4_vhdx_mmap_persistence_path: format!(
                "/var/lib/webizen-host/sanctuary/{}/vault",
                self.username
            ),
            primary_os_migration_deb_package:
                "webizen-desktop-appliance_0.1.0_amd64.deb + webizen-hostd_0.1.0_amd64.deb"
                    .to_string(),
            phone_pocket_cosigner_reanchor_required: true,
            portable_container_formats: vec![
                ".q42 (Q42\\0 v4 40KiB Page-Aligned NQuin48 Volume)".to_string(),
                ".10d (10d\\0 v2 HOLO 0x0306 Holographic Asset)".to_string(),
                ".hmc (QBDL v1 Hypermedia Bundle)".to_string(),
                ".qap (QAP\\0 v1 64B-Aligned Live .vibe Application Package)".to_string(),
                ".qar (QAR\\0 v1 Cold Evidentiary Archive with 4 Evidence Act Timestamps)"
                    .to_string(),
                "VaultContainerV2 (sanctuary_vault.cbor 4-Slot Argon2id Anti-Coercion Vault)"
                    .to_string(),
                ".q42j (WZQ42J\\x01\\0 GF(256) M-of-N Shamir Encrypted Journal)".to_string(),
            ],
            wsl2_daemon_bridge_command:
                "wsl.exe -d Debian -u root -- systemctl start webizen-hostd.service && nft -f /etc/webizen-host/nftables-qpolicy.conf"
                    .to_string(),
            primary_os_cutover_command:
                "webizen-os-shell portability export-primary-os-capsule --verify-qar-evidence --require-phone-cosign && dpkg -i webizen-hostd_0.1.0_amd64.deb webizen-desktop-appliance_0.1.0_amd64.deb"
                    .to_string(),
        }
    }
}

pub fn probe_wsl2_compatibility_mode(os: &str) -> Wsl2CompatibilityMode {
    if os == "linux" {
        if let Ok(rel) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
            let lower = rel.to_ascii_lowercase();
            if lower.contains("microsoft") || lower.contains("wsl") {
                return Wsl2CompatibilityMode::InsideWsl2LinuxGuest;
            }
        }
        if std::env::var_os("WSL_DISTRO_NAME").is_some()
            || std::env::var_os("WSL_INTEROP").is_some()
        {
            return Wsl2CompatibilityMode::InsideWsl2LinuxGuest;
        }
        return Wsl2CompatibilityMode::NativeLinuxPrimaryOs;
    }
    if os == "windows" {
        let windir = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
        let wsl_exe = std::path::Path::new(&windir)
            .join("System32")
            .join("wsl.exe");
        if wsl_exe.exists() || std::env::var_os("WSLENV").is_some() {
            return Wsl2CompatibilityMode::WindowsHostWithWsl2KernelBridgeAvailable;
        }
        return Wsl2CompatibilityMode::WindowsNativeUserSpaceOnly;
    }
    Wsl2CompatibilityMode::OtherPosixHost
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
        let plan = e.build_wsl2_and_primary_os_portability_plan();
        assert_eq!(plan.portable_container_formats.len(), 7);
        assert!(plan.phone_pocket_cosigner_reanchor_required);
    }
}
