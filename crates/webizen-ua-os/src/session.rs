use crate::browser_app::BrowserSocialGraph;
use crate::daemon::{self, DaemonError, DaemonHandle};
use crate::host_stewardship::NaturalHostStewardshipFabric;
use crate::identifiers::{self, IdentifierSet};
use crate::llm_management::LlmAndAgentManagementRegistry;
use crate::native_gpu::{self, NativeEtalonStatus};
use crate::os_env::OsEnvironment;
use crate::poet_synth::{self, LocalLlmModelEntry};
use crate::qap_host::{self, QappRegistryEntry, TrustRoots, UaOsHost};
use crate::semantic_fs::SemanticFileSystem;
use crate::{DEFAULT_DAEMON_PORT, PID_FILE_NAME, QAPPS_DIR};
use std::path::{Path, PathBuf};

/// Which artefact family this process belongs to (`KD-2`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sku {
    /// Application on an unmodified Device OS (Windows / macOS / Linux).
    Guest,
    /// Overlay on unmodified Debian 13 Trixie (`/opt/webizen`, vault `/var/lib/qualiadb`).
    Appliance,
}

impl Sku {
    pub fn as_str(self) -> &'static str {
        match self {
            Sku::Guest => "guest",
            Sku::Appliance => "appliance",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Sku::Guest => "Guest — app on the Device OS",
            Sku::Appliance => "Appliance — overlay on Debian 13 Trixie",
        }
    }

    pub fn resolve() -> Self {
        match std::env::var("WEBIZEN_SKU") {
            Ok(v) if v.eq_ignore_ascii_case("appliance") => return Sku::Appliance,
            Ok(v) if v.eq_ignore_ascii_case("guest") => return Sku::Guest,
            _ => {}
        }
        let marker = Path::new("/etc/webizen/sku");
        if let Ok(raw) = std::fs::read_to_string(marker) {
            if raw.trim().eq_ignore_ascii_case("appliance") {
                return Sku::Appliance;
            }
        }
        Sku::Guest
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceOs {
    Windows,
    MacOs,
    Linux,
    Other,
}

impl DeviceOs {
    pub fn detect() -> Self {
        if cfg!(target_os = "windows") {
            DeviceOs::Windows
        } else if cfg!(target_os = "macos") {
            DeviceOs::MacOs
        } else if cfg!(target_os = "linux") {
            DeviceOs::Linux
        } else {
            DeviceOs::Other
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            DeviceOs::Windows => "windows",
            DeviceOs::MacOs => "macos",
            DeviceOs::Linux => "linux",
            DeviceOs::Other => "other",
        }
    }
}

/// Hosting account / VPS personality. Guest pairs later; Appliance is local.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostLinkStatus {
    Unpaired,
    Paired,
    ApplianceLocal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostLink {
    pub status: HostLinkStatus,
    pub base_url: Option<String>,
}

impl HostLink {
    pub fn for_sku(sku: Sku) -> Self {
        match sku {
            Sku::Guest => {
                let base_url = std::env::var("WEBIZEN_HOST_URL")
                    .ok()
                    .filter(|s| !s.is_empty());
                let status = if base_url.is_some() {
                    HostLinkStatus::Paired
                } else {
                    HostLinkStatus::Unpaired
                };
                HostLink { status, base_url }
            }
            Sku::Appliance => HostLink {
                status: HostLinkStatus::ApplianceLocal,
                base_url: Some("http://127.0.0.1:8080".into()),
            },
        }
    }

    pub fn label(&self) -> &'static str {
        match self.status {
            HostLinkStatus::Unpaired => "Unpaired — Personal Cloud / webizen-host pairing ready",
            HostLinkStatus::Paired => "Paired with webizen-host Personal Cloud",
            HostLinkStatus::ApplianceLocal => "Local Debian Trixie Appliance hostd (:8080)",
        }
    }
}

/// Qualia vault parent + sidecar binary location + trust roots (`KD-13`, `KD-15`).
#[derive(Clone, Debug)]
pub struct LoopbackHost {
    pub daemon_base: String,
    pub port: u16,
    /// Vault parent. Guest: platform user-data. Appliance: `/var/lib/qualiadb`.
    pub storage: PathBuf,
    /// Artefact dir; sidecar is `{install_root}/qualia-cli` (`.exe` on Windows).
    pub install_root: PathBuf,
    /// First-party compiled Ed25519 key + extra-keys file.
    pub trust: TrustRoots,
}

impl LoopbackHost {
    pub fn qapps_dir(&self) -> PathBuf {
        self.storage.join(QAPPS_DIR)
    }

    pub fn pid_path(&self) -> PathBuf {
        self.storage.join("run").join(PID_FILE_NAME)
    }

    pub fn export_vault_env(&self) {
        let s = self.storage.display().to_string();
        std::env::set_var("QUALIA_STORAGE_PATH", &s);
        std::env::set_var("QUALIA_DATA_DIR", &s);
    }

    pub fn ensure_runtime_dir(&self) -> Result<(), DaemonError> {
        std::fs::create_dir_all(self.storage.join("run"))
            .map_err(|e| DaemonError::Io(format!("create {}: {e}", self.storage.display())))
    }

    /// Probe, then start `{install_root}/qualia-cli daemon start` if needed.
    /// Never execs the OS-shell binary.
    pub fn ensure_daemon(&self, sku: Sku) -> Result<DaemonHandle, DaemonError> {
        daemon::ensure_daemon(self, sku)
    }
}

/// Resolved User-Agent OS session for this process. Built once at launch.
#[derive(Clone, Debug)]
pub struct DeviceSession {
    pub sku: Sku,
    pub device_os: DeviceOs,
    pub os_username: String,
    pub os_env: OsEnvironment,
    pub host: LoopbackHost,
    pub gpu: NativeEtalonStatus,
    pub identifiers: IdentifierSet,
    pub installed_qaps: Vec<QappRegistryEntry>,
    pub llm_models: Vec<LocalLlmModelEntry>,
    pub semantic_fs: SemanticFileSystem,
    pub browser_social: BrowserSocialGraph,
    pub host_stewardship: NaturalHostStewardshipFabric,
    pub llm_management: LlmAndAgentManagementRegistry,
    pub host_link: HostLink,
}

impl DeviceSession {
    pub fn resolve() -> Self {
        let sku = Sku::resolve();
        let device_os = DeviceOs::detect();
        let os_username = device_os_username();
        let os_env = OsEnvironment::probe(&os_username);
        let storage = resolve_storage(sku);
        let install_root = resolve_install_root();
        let trust = TrustRoots::for_storage(&storage);
        let host = LoopbackHost {
            daemon_base: format!("http://127.0.0.1:{DEFAULT_DAEMON_PORT}"),
            port: DEFAULT_DAEMON_PORT,
            storage: storage.clone(),
            install_root,
            trust,
        };
        let _ = host.ensure_runtime_dir();
        host.export_vault_env();

        // 1. Probe native Etalon GPU adapters, Anvil NP64, and native VibeScript providers
        let gpu = native_gpu::probe_native_etalon(&storage);

        // 2. Enumerate QualiaDB-compliant identity bindings (noisy-OR < 1.0, NQuin48 parity)
        let identifiers = identifiers::enumerate_session_identifiers(&host, sku, &os_env, &gpu);

        // 3. Seed bundled first-party Qaps (`Anatomy`, `Poet`, etc.) into `{storage}/Qapps/`
        let installed_qaps = {
            let seeded = qap_host::seed_bundled_qaps(&host);
            if seeded.is_empty() {
                host.list_qapps().unwrap_or_default()
            } else {
                seeded
            }
        };

        // 4. Discover local SLD/LLM models across unified vault roots (`S1`/`S2`)
        let llm_models = poet_synth::discover_unified_llm_vault(&storage);

        // 5. Load or seed the persistent Semantic File-System (`{storage}/wellfair/hypermedia_library.json`)
        let semantic_fs = SemanticFileSystem::load_or_seed(&storage, &llm_models);

        // 6. Load or seed the Browser Social-Web Graph (`{storage}/wellfair/browser_social_graph.json`)
        let browser_social = BrowserSocialGraph::load_or_seed(&storage);

        // 7. Load or seed Natural-Person & M-of-N Cooperative Quorum Host Stewardship (`{storage}/wellfair/host_stewardship_quorum.json`)
        let host_stewardship = NaturalHostStewardshipFabric::load_or_seed(&storage, &identifiers);

        // 8. Load or seed Unified LLM (.gguf -> .p64 Conversion & Purge), Governed Agent Roster & External MCP Accounts (`{storage}/wellfair/llm_agent_management.json`)
        let llm_management = LlmAndAgentManagementRegistry::load_or_seed(
            &storage,
            &identifiers.fabric_did,
            &llm_models,
        );

        let host_link = HostLink::for_sku(sku);
        DeviceSession {
            sku,
            device_os,
            os_username,
            os_env,
            host,
            gpu,
            identifiers,
            installed_qaps,
            llm_models,
            semantic_fs,
            browser_social,
            host_stewardship,
            llm_management,
            host_link,
        }
    }

    pub fn fabric_identifier(&self) -> &str {
        self.identifiers.fabric_did.as_str()
    }
}

pub fn device_os_username() -> String {
    for key in ["WEBIZEN_OS_USER", "USERNAME", "USER", "LOGNAME"] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    "unknown".into()
}

pub fn guest_storage_dir(device_os: DeviceOs) -> PathBuf {
    if let Ok(p) = std::env::var("WEBIZEN_STORAGE") {
        let t = p.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    match device_os {
        DeviceOs::Windows => {
            let base = std::env::var("LOCALAPPDATA")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".into());
            PathBuf::from(base).join("Webizen").join("storage")
        }
        DeviceOs::MacOs => home_dir().join("Library/Application Support/Webizen/storage"),
        DeviceOs::Linux | DeviceOs::Other => {
            if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
                if !xdg.trim().is_empty() {
                    return PathBuf::from(xdg).join("webizen").join("storage");
                }
            }
            home_dir().join(".local/share/webizen/storage")
        }
    }
}

fn appliance_storage_dir() -> PathBuf {
    if let Ok(p) = std::env::var("WEBIZEN_STORAGE") {
        let t = p.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    PathBuf::from("/var/lib/qualiadb")
}

fn resolve_storage(sku: Sku) -> PathBuf {
    match sku {
        Sku::Guest => guest_storage_dir(DeviceOs::detect()),
        Sku::Appliance => appliance_storage_dir(),
    }
}

fn resolve_install_root() -> PathBuf {
    if let Ok(p) = std::env::var("WEBIZEN_INSTALL_ROOT") {
        let t = p.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV: Mutex<()> = Mutex::new(());

    #[test]
    fn appliance_default_storage_is_var_lib() {
        let _g = ENV.lock().unwrap();
        std::env::remove_var("WEBIZEN_STORAGE");
        assert_eq!(appliance_storage_dir(), PathBuf::from("/var/lib/qualiadb"));
    }

    #[test]
    fn loopback_exports_storage_not_parent() {
        let _g = ENV.lock().unwrap();
        let storage = PathBuf::from("/tmp/wz-vault-env");
        let host = LoopbackHost {
            daemon_base: "http://127.0.0.1:4242".into(),
            port: 4242,
            storage: storage.clone(),
            install_root: PathBuf::from("/opt/webizen"),
            trust: TrustRoots::for_storage(&storage),
        };
        host.export_vault_env();
        assert_eq!(
            std::env::var("QUALIA_STORAGE_PATH").unwrap(),
            "/tmp/wz-vault-env"
        );
        assert_eq!(
            std::env::var("QUALIA_DATA_DIR").unwrap(),
            "/tmp/wz-vault-env"
        );
        assert_eq!(host.qapps_dir(), PathBuf::from("/tmp/wz-vault-env/Qapps"));
        assert_eq!(
            host.pid_path(),
            PathBuf::from("/tmp/wz-vault-env/run/daemon-service.json")
        );
    }
}
