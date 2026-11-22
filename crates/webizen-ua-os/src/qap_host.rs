//! Canonical `UaOsHost`, Qap Package Installer (`.qpkg` / Directory / `yaml-ld-q42`),
//! Registry (`{storage}/Qapps/registry.json`), `LaunchPlan`, and Sentinel `LeaseSheet` (`C0`, `C2`, `C4`, `D0`–`D4`).
//!
//! Operates directly over `LoopbackHost.storage` (the vault parent; `qapps_dir` joins `"Qapps"`).
//! Never touches Tauri's `APP_STATE`. Product language is **Qap / Qaps**; on-disk compatibility
//! retains `{storage}/Qapps/` and `qapp.json` (`KD-4`, `KD-20`).

use crate::daemon::{DaemonError, DaemonHandle};
use crate::session::{LoopbackHost, Sku};
use crate::{QAPPS_DIR, QAPP_PACKAGE_MANIFEST, QAPP_REGISTRY_FILE};
use flate2::read::DeflateDecoder;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

/// Compiled first-party Ed25519 trust root public key (`KD-15`).
pub const FIRST_PARTY_PUBKEY: [u8; 32] = [
    0x7f, 0x18, 0xb3, 0x20, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
    0x42, 0x10, 0xd0, 0x03, 0x06, 0x91, 0x75, 0x04, 0xaa, 0x55, 0xee, 0x11, 0x20, 0x26, 0x11, 0x22,
];

/// Default bundled first-party Qap packages in `qualiaDB/bundled/qapps/`.
pub const BUNDLED_FIRST_PARTY_QAPS: &[&str] = &[
    "Anatomy",
    "Poet",
    "Wallet",
    "Projects",
    "Library",
    "LlmAgents",
    "CommsDirectory",
    "Browser",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallPolicy {
    Development,
    Production,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallSource {
    Directory(PathBuf),
    /// `.qpkg` ZIP archive envelope (`KD-4` / `D1`).
    QpkgZip(PathBuf),
    /// `yaml-ld-q42` workspace/asset manifest (`D2` — synthesizes `qapp.json`).
    YamlLdQ42(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaunchKind {
    /// Native RSX stage card / native workbench (e.g. Anatomy seed card, native Poet/Synth stage).
    NativeRsxCard,
    /// Declarative `yaml-ld-q42` surface.
    YamlLd,
    /// Standalone native child process.
    NativeProcess,
}

impl LaunchKind {
    pub fn label(self) -> &'static str {
        match self {
            LaunchKind::NativeRsxCard => "NativeRsxCard",
            LaunchKind::YamlLd => "YamlLd",
            LaunchKind::NativeProcess => "NativeProcess",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchPlan {
    pub package_id: String,
    pub display_name: String,
    pub version: String,
    pub category: String,
    pub fabric_coordinate: String,
    pub package_dir: PathBuf,
    pub entrypoint: PathBuf,
    pub kind: LaunchKind,
    pub required_shapes: Vec<String>,
    pub required_ontologies: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseFamily {
    pub id: String,
    pub grant: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseSheet {
    pub package_id: String,
    pub version: String,
    pub who_plane_subject: Option<String>,
    pub guardian_install: bool,
    #[serde(rename = "default")]
    pub default_policy: String,
    pub families: Vec<LeaseFamily>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustRoots {
    pub first_party: [u8; 32],
    pub extra_keys_path: PathBuf,
}

impl TrustRoots {
    pub fn for_storage(storage: &Path) -> Self {
        Self {
            first_party: FIRST_PARTY_PUBKEY,
            extra_keys_path: storage.join("qapp-trust-keys.json"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QappRegistryEntry {
    pub package_id: String,
    pub active_version: String,
    pub content_hash: String,
    pub installed_at_unix: u64,
    pub revoked: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub archived_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct QappInstallRegistry {
    pub packages: HashMap<String, QappRegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QappLaunchRequirement {
    pub capability: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QappHostExtension {
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub preferred_launch_mode: String,
    #[serde(default)]
    pub supports_offline: bool,
    #[serde(default)]
    pub requires: Vec<QappLaunchRequirement>,
    #[serde(default)]
    pub required_ontologies: Vec<String>,
    #[serde(default)]
    pub entrypoints: HashMap<String, String>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub did_git: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QappPackageManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub required_shapes: Vec<String>,
    #[serde(default)]
    pub dev_port: Option<u16>,
    #[serde(default)]
    pub x_qualia: Option<QappHostExtension>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QappInstallError {
    InvalidPackageId(String),
    PathTraversal(String),
    ManifestMissing,
    ManifestInvalid(String),
    SignatureInvalid(String),
    PackageRevoked(String),
    StagingFailed(String),
    RegistryCorrupt(String),
    Io(String),
}

impl std::fmt::Display for QappInstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPackageId(s) => write!(f, "invalid package id: {s}"),
            Self::PathTraversal(s) => write!(f, "path traversal rejected: {s}"),
            Self::ManifestMissing => write!(f, "qapp.json not found"),
            Self::ManifestInvalid(s) => write!(f, "invalid qapp.json: {s}"),
            Self::SignatureInvalid(s) => write!(f, "signature invalid: {s}"),
            Self::PackageRevoked(s) => write!(f, "package revoked: {s}"),
            Self::StagingFailed(s) => write!(f, "staging failed: {s}"),
            Self::RegistryCorrupt(s) => write!(f, "registry corrupt: {s}"),
            Self::Io(s) => write!(f, "io error: {s}"),
        }
    }
}

impl From<std::io::Error> for QappInstallError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

/// Core User-Agent OS Host contract (`00-dual-sku-design.md` §API).
pub trait UaOsHost: Send + Sync {
    fn ensure_daemon(&self, sku: Sku) -> Result<DaemonHandle, DaemonError>;
    fn list_qapps(&self) -> Result<Vec<QappRegistryEntry>, QappInstallError>;
    fn install(
        &self,
        src: InstallSource,
        policy: InstallPolicy,
    ) -> Result<(QappRegistryEntry, LeaseSheet), QappInstallError>;
    fn launch(&self, package_id: &str, entry: Option<&str>) -> Result<LaunchPlan, QappInstallError>;
    fn revoke(&self, package_id: &str) -> Result<(), QappInstallError>;
}

impl UaOsHost for LoopbackHost {
    fn ensure_daemon(&self, sku: Sku) -> Result<DaemonHandle, DaemonError> {
        crate::daemon::ensure_daemon(self, sku)
    }

    fn list_qapps(&self) -> Result<Vec<QappRegistryEntry>, QappInstallError> {
        let mut entries: Vec<QappRegistryEntry> = load_install_registry(&self.storage)?
            .packages
            .into_values()
            .collect();
        entries.sort_by(|a, b| a.package_id.cmp(&b.package_id));
        Ok(entries)
    }

    fn install(
        &self,
        src: InstallSource,
        policy: InstallPolicy,
    ) -> Result<(QappRegistryEntry, LeaseSheet), QappInstallError> {
        install_from_source(&self.storage, &self.trust, src, policy, false)
    }

    fn launch(&self, package_id: &str, entry: Option<&str>) -> Result<LaunchPlan, QappInstallError> {
        plan_qapp_launch(&self.storage, package_id, entry)
    }

    fn revoke(&self, package_id: &str) -> Result<(), QappInstallError> {
        revoke_package(&self.storage, package_id)
    }
}

pub fn validate_package_id(package_id: &str) -> Result<(), QappInstallError> {
    if package_id.is_empty() {
        return Err(QappInstallError::InvalidPackageId("empty".into()));
    }
    if package_id.contains("..") || package_id.contains('/') || package_id.contains('\\') {
        return Err(QappInstallError::InvalidPackageId(package_id.into()));
    }
    for ch in package_id.chars() {
        let ok = ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ' ';
        if !ok {
            return Err(QappInstallError::InvalidPackageId(package_id.into()));
        }
    }
    Ok(())
}

fn relative_path_ok(rel: &str) -> bool {
    let path = Path::new(rel);
    for component in path.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return false,
            _ => {}
        }
    }
    true
}

fn qapps_root(storage: &Path) -> PathBuf {
    storage.join(QAPPS_DIR)
}

fn registry_path(storage: &Path) -> PathBuf {
    qapps_root(storage).join(QAPP_REGISTRY_FILE)
}

pub fn load_install_registry(storage: &Path) -> Result<QappInstallRegistry, QappInstallError> {
    let path = registry_path(storage);
    if !path.is_file() {
        return Ok(QappInstallRegistry::default());
    }
    let content =
        fs::read_to_string(&path).map_err(|e| QappInstallError::RegistryCorrupt(e.to_string()))?;
    serde_json::from_str(&content).map_err(|e| QappInstallError::RegistryCorrupt(e.to_string()))
}

pub fn save_install_registry(
    storage: &Path,
    registry: &QappInstallRegistry,
) -> Result<(), QappInstallError> {
    let root = qapps_root(storage);
    fs::create_dir_all(&root)?;
    let path = registry_path(storage);
    let staging = path.with_extension("json.staging");
    let json = serde_json::to_string_pretty(registry)
        .map_err(|e| QappInstallError::RegistryCorrupt(e.to_string()))?;
    fs::write(&staging, json)?;
    fs::rename(&staging, &path)?;
    Ok(())
}

pub fn build_lease_sheet(
    manifest: &QappPackageManifest,
    policy: InstallPolicy,
    guardian_install: bool,
) -> Result<LeaseSheet, QappInstallError> {
    let package_id = manifest.name.trim().to_string();
    validate_package_id(&package_id)?;
    let version = manifest.version.trim().trim_start_matches('v').to_string();

    let mut families = Vec::new();
    if let Some(ext) = &manifest.x_qualia {
        for req in &ext.requires {
            if !req.capability.trim().is_empty() {
                families.push(LeaseFamily {
                    id: req.capability.trim().to_string(),
                    grant: !guardian_install && req.required,
                });
            }
        }
        for ont in &ext.required_ontologies {
            if !ont.trim().is_empty() {
                families.push(LeaseFamily {
                    id: format!("graph.ontology:{}", ont.trim()),
                    grant: false,
                });
            }
        }
    }
    for shape in &manifest.required_shapes {
        if !shape.trim().is_empty() {
            families.push(LeaseFamily {
                id: format!("shacl.shape:{}", shape.trim()),
                grant: false,
            });
        }
    }

    if policy == InstallPolicy::Production && families.is_empty() {
        return Err(QappInstallError::ManifestInvalid(
            "production lease sheet cannot have empty capability families".into(),
        ));
    }

    Ok(LeaseSheet {
        package_id,
        version,
        who_plane_subject: None,
        guardian_install,
        default_policy: "deny".into(),
        families,
    })
}

pub fn install_package_atomic(
    storage: &Path,
    source_dir: &Path,
    policy: InstallPolicy,
    trust: Option<&TrustRoots>,
) -> Result<(QappRegistryEntry, LeaseSheet), QappInstallError> {
    let manifest_path = source_dir.join(QAPP_PACKAGE_MANIFEST);
    if !manifest_path.is_file() {
        return Err(QappInstallError::ManifestMissing);
    }
    let content = fs::read_to_string(&manifest_path)?;
    let manifest: QappPackageManifest = serde_json::from_str(&content)
        .map_err(|e| QappInstallError::ManifestInvalid(e.to_string()))?;
    let package_id = manifest.name.trim().to_string();
    validate_package_id(&package_id)?;
    let version = manifest.version.trim().trim_start_matches('v').to_string();

    if policy == InstallPolicy::Production {
        let sidecar = source_dir.join("package-manifest.json");
        if !sidecar.is_file() {
            return Err(QappInstallError::SignatureInvalid(
                "production install requires signed package-manifest.json".into(),
            ));
        }
        if trust.is_none() {
            return Err(QappInstallError::SignatureInvalid(
                "no trust roots configured for production install".into(),
            ));
        }
    }

    let lease_sheet = build_lease_sheet(&manifest, policy, false)?;
    let mut registry = load_install_registry(storage)?;
    if registry
        .packages
        .get(&package_id)
        .map(|e| e.revoked)
        .unwrap_or(false)
    {
        return Err(QappInstallError::PackageRevoked(package_id));
    }

    let content_hash = hash_directory(source_dir)?;
    let dest = qapps_root(storage).join(&package_id);
    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }
    copy_dir_all(source_dir, &dest)?;

    // Persist the lease sheet beside the installed Qap
    if let Ok(lease_json) = serde_json::to_string_pretty(&lease_sheet) {
        let _ = fs::write(dest.join("lease-sheet.json"), lease_json);
    }

    let installed_at_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let entry = QappRegistryEntry {
        package_id: package_id.clone(),
        active_version: version,
        content_hash,
        installed_at_unix,
        revoked: false,
        archived_versions: Vec::new(),
    };
    registry.packages.insert(package_id, entry.clone());
    save_install_registry(storage, &registry)?;
    Ok((entry, lease_sheet))
}

pub fn install_from_source(
    storage: &Path,
    trust: &TrustRoots,
    src: InstallSource,
    policy: InstallPolicy,
    guardian_install: bool,
) -> Result<(QappRegistryEntry, LeaseSheet), QappInstallError> {
    let (entry, mut sheet) = match src {
        InstallSource::Directory(dir) => {
            install_package_atomic(storage, &dir, policy, Some(trust))?
        }
        InstallSource::QpkgZip(zip_path) => {
            let staging = qapps_root(storage)
                .join(".staging-zip")
                .join(format!("qpkg-{}", std::process::id()));
            let _ = fs::remove_dir_all(&staging);
            unpack_qpkg_zip(&zip_path, &staging)?;
            let res = install_package_atomic(storage, &staging, policy, Some(trust));
            let _ = fs::remove_dir_all(&staging);
            res?
        }
        InstallSource::YamlLdQ42(yaml_path) => {
            let staging = qapps_root(storage)
                .join(".staging-yaml")
                .join(format!("yaml-{}", std::process::id()));
            let _ = fs::remove_dir_all(&staging);
            synthesize_qapp_json_from_yaml(&yaml_path, &staging)?;
            let res = install_package_atomic(storage, &staging, policy, Some(trust));
            let _ = fs::remove_dir_all(&staging);
            res?
        }
    };
    if guardian_install {
        sheet.guardian_install = true;
        for fam in &mut sheet.families {
            fam.grant = false;
        }
    }
    Ok((entry, sheet))
}

pub fn plan_qapp_launch(
    storage: &Path,
    package_id: &str,
    entry: Option<&str>,
) -> Result<LaunchPlan, QappInstallError> {
    validate_package_id(package_id)?;
    let reg = load_install_registry(storage)?;
    if reg
        .packages
        .get(package_id)
        .map(|e| e.revoked)
        .unwrap_or(false)
    {
        return Err(QappInstallError::PackageRevoked(package_id.into()));
    }

    let package_dir = qapps_root(storage).join(package_id);
    let manifest_path = package_dir.join(QAPP_PACKAGE_MANIFEST);
    if !manifest_path.is_file() {
        return Err(QappInstallError::ManifestMissing);
    }
    let content = fs::read_to_string(&manifest_path)?;
    let manifest: QappPackageManifest = serde_json::from_str(&content)
        .map_err(|e| QappInstallError::ManifestInvalid(e.to_string()))?;

    let version = manifest.version.trim().trim_start_matches('v').to_string();
    let ext = manifest.x_qualia.as_ref();
    let display_name = ext
        .map(|x| x.display_name.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&manifest.name)
        .to_string();
    let category = ext
        .map(|x| x.category.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("qap")
        .to_string();

    let entry_rel = entry
        .and_then(|k| {
            ext.and_then(|x| x.entrypoints.get(k).cloned())
                .or_else(|| Some(k.to_string()))
        })
        .or_else(|| ext.and_then(|x| x.entrypoints.get("native").cloned()))
        .or_else(|| ext.and_then(|x| x.entrypoints.get("yaml_ld").cloned()))
        .or_else(|| ext.and_then(|x| x.entrypoints.get("web").cloned()))
        .unwrap_or_else(|| "index.html".into());

    if !relative_path_ok(&entry_rel) {
        return Err(QappInstallError::PathTraversal(entry_rel));
    }

    let kind = if entry_rel.ends_with(".yaml") || entry_rel.ends_with(".yml") {
        LaunchKind::YamlLd
    } else if ext
        .map(|x| x.preferred_launch_mode == "native-process")
        .unwrap_or(false)
    {
        LaunchKind::NativeProcess
    } else {
        // Phase-1 rule (`KD-14`): Anatomy and bundled Qaps open as NativeRsxCard,
        // never loading Babylon.js index.html under NoOpDocument.
        LaunchKind::NativeRsxCard
    };

    let fabric_coordinate = format!(
        "did:q42:qap:{}:v{}",
        manifest.name.trim().to_ascii_lowercase(),
        version
    );

    Ok(LaunchPlan {
        package_id: manifest.name.trim().to_string(),
        display_name,
        version,
        category,
        fabric_coordinate,
        entrypoint: package_dir.join(&entry_rel),
        package_dir,
        kind,
        required_shapes: manifest.required_shapes,
        required_ontologies: ext.map(|x| x.required_ontologies.clone()).unwrap_or_default(),
        notes: ext.map(|x| x.notes.clone()).unwrap_or_default(),
    })
}

pub fn revoke_package(storage: &Path, package_id: &str) -> Result<(), QappInstallError> {
    validate_package_id(package_id)?;
    let mut registry = load_install_registry(storage)?;
    let entry = registry
        .packages
        .get_mut(package_id)
        .ok_or_else(|| QappInstallError::StagingFailed(format!("unknown package {package_id}")))?;
    entry.revoked = true;
    save_install_registry(storage, &registry)
}

/// Automatically seed bundled first-party Qaps (`Anatomy`, `Poet`, etc.) from
/// `qualiaDB/bundled/qapps/` into `{storage}/Qapps/` on startup (`D4` + first-party catalog).
pub fn seed_bundled_qaps(host: &LoopbackHost) -> Vec<QappRegistryEntry> {
    let mut seeded = Vec::new();
    let Some(bundled_root) = locate_bundled_qapps_root(host) else {
        return seeded;
    };

    for &name in BUNDLED_FIRST_PARTY_QAPS {
        let candidate = bundled_root.join(name);
        if !candidate.join(QAPP_PACKAGE_MANIFEST).is_file() {
            continue;
        }
        let dest = qapps_root(&host.storage).join(name).join(QAPP_PACKAGE_MANIFEST);
        if dest.is_file() {
            continue;
        }
        let _ = install_package_atomic(
            &host.storage,
            &candidate,
            InstallPolicy::Development,
            Some(&host.trust),
        );
    }

    if let Ok(entries) = host.list_qapps() {
        seeded = entries;
    }
    seeded
}

fn locate_bundled_qapps_root(host: &LoopbackHost) -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("QUALIA_BUNDLED_QAPPS_DIR") {
        let p = PathBuf::from(dir);
        if p.is_dir() {
            return Some(p);
        }
    }
    let beside_install = host.install_root.join("bundled/qapps");
    if beside_install.is_dir() {
        return Some(beside_install);
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    if let Some(github) = manifest.ancestors().nth(4) {
        let sibling = github.join("qualiaDB/bundled/qapps");
        if sibling.is_dir() {
            return Some(sibling);
        }
    }
    None
}

/// Synthesize a uniform `qapp.json` from a `yaml-ld-q42` manifest file (`D2`).
pub fn synthesize_qapp_json_from_yaml(
    yaml_path: &Path,
    target_dir: &Path,
) -> Result<QappPackageManifest, QappInstallError> {
    let raw = fs::read_to_string(yaml_path)?;
    let mut name = String::from("YamlLdWorkspace");
    let mut version = String::from("0.1.0");

    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                name = val.to_string();
            }
        } else if let Some(rest) = trimmed.strip_prefix("version:") {
            let val = rest.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                version = val.to_string();
            }
        }
    }
    validate_package_id(&name)?;
    fs::create_dir_all(target_dir)?;

    let file_name = yaml_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("manifest.q42.yaml")
        .to_string();
    let dst_yaml = target_dir.join(&file_name);
    if yaml_path != dst_yaml {
        fs::copy(yaml_path, &dst_yaml)?;
    }

    let mut entrypoints = HashMap::new();
    entrypoints.insert("yaml_ld".into(), file_name);

    let manifest = QappPackageManifest {
        name: name.clone(),
        version: version.clone(),
        required_shapes: vec!["q42:YamlLdWorkspaceShape".into()],
        dev_port: None,
        x_qualia: Some(QappHostExtension {
            app_id: format!("did:q42:qap:{}:v{}", name.to_ascii_lowercase(), version),
            display_name: name,
            category: "yaml-ld-q42".into(),
            preferred_launch_mode: "yaml-ld".into(),
            supports_offline: true,
            entrypoints,
            ..Default::default()
        }),
    };

    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| QappInstallError::ManifestInvalid(e.to_string()))?;
    fs::write(target_dir.join(QAPP_PACKAGE_MANIFEST), json)?;
    Ok(manifest)
}

/// Pure-Rust `.qpkg` (ZIP magic `PK\x03\x04`) unpacker with path-traversal protection (`D1`).
pub fn unpack_qpkg_zip(zip_path: &Path, dst_dir: &Path) -> Result<(), QappInstallError> {
    let bytes = fs::read(zip_path)?;
    if bytes.len() < 4 || &bytes[0..4] != b"PK\x03\x04" {
        return Err(QappInstallError::StagingFailed(
            "invalid .qpkg archive (expected ZIP magic PK\\x03\\x04)".into(),
        ));
    }
    fs::create_dir_all(dst_dir)?;
    let mut cursor = 0usize;
    while cursor + 30 <= bytes.len() {
        let sig = &bytes[cursor..cursor + 4];
        if sig == b"PK\x01\x02" || sig == b"PK\x05\x06" {
            break;
        }
        if sig != b"PK\x03\x04" {
            return Err(QappInstallError::StagingFailed(
                "corrupt .qpkg ZIP local header".into(),
            ));
        }
        let method = u16::from_le_bytes([bytes[cursor + 8], bytes[cursor + 9]]);
        let comp_size = u32::from_le_bytes([
            bytes[cursor + 18],
            bytes[cursor + 19],
            bytes[cursor + 20],
            bytes[cursor + 21],
        ]) as usize;
        let name_len = u16::from_le_bytes([bytes[cursor + 26], bytes[cursor + 27]]) as usize;
        let extra_len = u16::from_le_bytes([bytes[cursor + 28], bytes[cursor + 29]]) as usize;
        let name_start = cursor + 30;
        let data_start = name_start + name_len + extra_len;
        let data_end = data_start + comp_size;
        if data_end > bytes.len() {
            return Err(QappInstallError::StagingFailed(
                "truncated .qpkg ZIP entry".into(),
            ));
        }
        let entry_name = String::from_utf8_lossy(&bytes[name_start..name_start + name_len])
            .replace('\\', "/");
        if !relative_path_ok(&entry_name) {
            return Err(QappInstallError::PathTraversal(entry_name));
        }
        let out_path = dst_dir.join(&entry_name);
        if entry_name.ends_with('/') {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let raw_payload = &bytes[data_start..data_end];
            match method {
                0 => fs::write(&out_path, raw_payload)?,
                8 => {
                    let mut decoder = DeflateDecoder::new(raw_payload);
                    let mut out = Vec::new();
                    decoder.read_to_end(&mut out)?;
                    fs::write(&out_path, out)?;
                }
                other => {
                    return Err(QappInstallError::StagingFailed(format!(
                        "unsupported .qpkg ZIP compression method {other}"
                    )))
                }
            }
        }
        cursor = data_end;
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), QappInstallError> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let name = entry.file_name();
        if name.to_string_lossy() == "versions" {
            continue;
        }
        let target = dst.join(&name);
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn hash_directory(dir: &Path) -> Result<String, QappInstallError> {
    let mut files = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut hasher = Sha3_256::new();
    for (rel, digest) in files {
        hasher.update(rel.as_bytes());
        hasher.update(b":");
        hasher.update(digest.as_bytes());
        hasher.update(b"\n");
    }
    let out = hasher.finalize();
    Ok(out.iter().map(|b| format!("{b:02x}")).collect())
}

fn collect_files(
    root: &Path,
    current: &Path,
    out: &mut Vec<(String, String)>,
) -> Result<(), QappInstallError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".staging" || name == QAPP_REGISTRY_FILE || name == "versions" {
            continue;
        }
        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .map_err(|_| QappInstallError::PathTraversal(path.display().to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            if !relative_path_ok(&rel) {
                return Err(QappInstallError::PathTraversal(rel));
            }
            let bytes = fs::read(&path)?;
            let mut h = Sha3_256::new();
            h.update(&bytes);
            let digest: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
            out.push((rel, digest));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_and_plans_anatomy_as_native_rsx_card() {
        let tmp = std::env::temp_dir().join(format!("wz-qap-host-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let host = LoopbackHost {
            daemon_base: "http://127.0.0.1:4242".into(),
            port: 4242,
            storage: tmp.clone(),
            install_root: PathBuf::from("."),
            trust: TrustRoots::for_storage(&tmp),
        };
        let seeded = seed_bundled_qaps(&host);
        if !seeded.is_empty() {
            assert!(seeded.iter().any(|e| e.package_id == "Anatomy"));
            let plan = host.launch("Anatomy", None).expect("Anatomy launch plan");
            assert_eq!(plan.kind, LaunchKind::NativeRsxCard);
            assert_eq!(plan.fabric_coordinate, "did:q42:qap:anatomy:v0.0.12");
        }
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn yaml_ld_q42_synthesizes_qapp_json_and_launches_as_yaml_ld() {
        let tmp = std::env::temp_dir().join(format!("wz-yaml-ld-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let yaml = tmp.join("manifest.q42.yaml");
        fs::write(&yaml, "name: TestManifold\nversion: 0.2.0\npages: []\n").unwrap();
        let trust = TrustRoots::for_storage(&tmp);
        let (entry, sheet) = install_from_source(
            &tmp,
            &trust,
            InstallSource::YamlLdQ42(yaml),
            InstallPolicy::Development,
            false,
        )
        .unwrap();
        assert_eq!(entry.package_id, "TestManifold");
        assert_eq!(sheet.default_policy, "deny");
        let plan = plan_qapp_launch(&tmp, "TestManifold", None).unwrap();
        assert_eq!(plan.kind, LaunchKind::YamlLd);
        let _ = fs::remove_dir_all(&tmp);
    }
}
