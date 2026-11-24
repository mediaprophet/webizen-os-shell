//! Canonical `UaOsHost`, Qap Package Installer (`.qpkg` / Directory / `yaml-ld-q42`),
//! Registry (`{storage}/Qapps/registry.json`), `LaunchPlan`, and Sentinel `LeaseSheet` (`C0`, `C2`, `C4`, `D0`–`D4`).
//!
//! Operates directly over `LoopbackHost.storage` (the vault parent; `qapps_dir` joins `"Qapps"`).
//! Never touches Tauri's `APP_STATE`. Product language is **Qap / Qaps**; on-disk compatibility
//! retains `{storage}/Qapps/` and `qapp.json` (`KD-4`, `KD-20`).

use crate::daemon::{DaemonError, DaemonHandle};
use crate::os_jobs_leases::{CapabilityLeaseScope, OsControlCenterState};
use crate::qualia_host_foundation::AdoptedQualiaFoundation;
use crate::session::{LoopbackHost, Sku};
use crate::{QAPPS_DIR, QAPP_PACKAGE_MANIFEST, QAPP_REGISTRY_FILE};
use flate2::read::DeflateDecoder;
use qualia_cooperative_core::authoring::{authorize, AuthoringAction, AuthoringRequest};
use qualia_core_db::vibe_host::native_authority::DocumentTransaction;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use vibe::extension_registry::CallExecutionContext;

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
    fn launch(&self, package_id: &str, entry: Option<&str>)
        -> Result<LaunchPlan, QappInstallError>;
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

    fn launch(
        &self,
        package_id: &str,
        entry: Option<&str>,
    ) -> Result<LaunchPlan, QappInstallError> {
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

fn read_bounded_json<T: serde::de::DeserializeOwned>(path: &Path, limit: u64) -> Result<T, String> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|file| file.take(limit + 1).read_to_end(&mut bytes))
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("Qap metadata exceeds the read limit".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

fn normalize_version(version: &str) -> &str {
    version.trim().trim_start_matches('v')
}

#[cfg(windows)]
fn reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn reparse_point(_: &fs::Metadata) -> bool {
    false
}

fn read_qap_manifest(
    package_dir: &Path,
    package_id: &str,
) -> Result<QappPackageManifest, QappInstallError> {
    let qaps_root = package_dir
        .parent()
        .ok_or(QappInstallError::ManifestMissing)?;
    for directory in [qaps_root, package_dir] {
        match fs::symlink_metadata(directory) {
            Ok(meta) if meta.is_dir() && !reparse_point(&meta) => {}
            Ok(_) => {
                return Err(QappInstallError::ManifestInvalid(
                    "Qap directory must be a local unlinked directory".into(),
                ))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(QappInstallError::ManifestMissing)
            }
            Err(error) => return Err(error.into()),
        }
    }
    let manifest_path = package_dir.join(QAPP_PACKAGE_MANIFEST);
    match fs::symlink_metadata(&manifest_path) {
        Ok(meta) if meta.is_file() && !reparse_point(&meta) => {}
        Ok(_) => {
            return Err(QappInstallError::ManifestInvalid(
                "Qap manifest must be a regular file".into(),
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(QappInstallError::ManifestMissing)
        }
        Err(error) => return Err(error.into()),
    }
    let manifest: QappPackageManifest =
        read_bounded_json(&manifest_path, 256 * 1024).map_err(QappInstallError::ManifestInvalid)?;
    // A differently cased registry or filesystem alias must not evade a revocation.
    if manifest.name.trim() != package_id {
        return Err(QappInstallError::ManifestInvalid(
            "Qap manifest name differs from its installed identifier".into(),
        ));
    }
    Ok(manifest)
}

fn reject_package_revocation(
    registry: &QappInstallRegistry,
    package_id: &str,
) -> Result<(), QappInstallError> {
    if registry
        .packages
        .get(package_id)
        .is_some_and(|entry| entry.revoked)
    {
        return Err(QappInstallError::PackageRevoked(package_id.into()));
    }
    Ok(())
}

fn check_qap_policy(
    policy: &OsControlCenterState,
    package_id: &str,
    scope: CapabilityLeaseScope,
) -> Result<(), QappInstallError> {
    if policy.qap_lease_revoked(package_id, scope) {
        return Err(QappInstallError::ManifestInvalid(format!(
            "{} is revoked for Qap {package_id} by persisted local policy",
            scope.code()
        )));
    }
    Ok(())
}

/// Re-read package revocation and the persisted local restriction for an
/// existing caller. Success is only permission to continue canonical checks.
pub fn require_qap_capability(
    storage: &Path,
    package_id: &str,
    scope: CapabilityLeaseScope,
) -> Result<(), QappInstallError> {
    validate_package_id(package_id)?;
    let registry = load_install_registry(storage)?;
    reject_package_revocation(&registry, package_id)?;
    let policy = OsControlCenterState::read_qap_policy(storage)
        .map_err(QappInstallError::RegistryCorrupt)?;
    check_qap_policy(&policy, package_id, scope)?;
    let entry = registry
        .packages
        .get(package_id)
        .ok_or_else(|| QappInstallError::ManifestInvalid("Qap is not installed".into()))?;
    require_active_qap_surface(storage, package_id, scope, entry)
}

fn require_active_qap_surface(
    storage: &Path,
    package_id: &str,
    scope: CapabilityLeaseScope,
    entry: &QappRegistryEntry,
) -> Result<(), QappInstallError> {
    let directory = qapps_root(storage).join(package_id);
    let manifest = read_qap_manifest(&directory, package_id)?;
    let ext = manifest.x_qualia.as_ref();
    let compiled_app_id = match (package_id, scope) {
        (
            "Browser",
            CapabilityLeaseScope::QualiaQ42Storage
            | CapabilityLeaseScope::GovernedVibeScriptEval
            | CapabilityLeaseScope::EtalonGpuSurface,
        ) => Some("did:qualia:qap:browser"),
        (
            "Poet",
            CapabilityLeaseScope::QualiaQ42Storage
            | CapabilityLeaseScope::GovernedVibeScriptEval
            | CapabilityLeaseScope::EtalonGpuSurface,
        ) => Some("did:qualia:qap:poet"),
        ("Library", CapabilityLeaseScope::EtalonGpuSurface) => Some("did:qualia:qap:library"),
        ("Anatomy", CapabilityLeaseScope::EtalonGpuSurface) => Some("did:qualia:qapp:anatomy"),
        _ => None,
    };
    let declares_scope = ext.is_some_and(|extension| match compiled_app_id {
        Some(app_id) => extension.app_id == app_id,
        None => extension.requires.iter().any(|requirement| {
            CapabilityLeaseScope::from_family_id(requirement.capability.trim()) == Some(scope)
        }),
    });
    if entry.package_id != package_id
        || entry.active_version != normalize_version(&manifest.version)
        || !declares_scope
    {
        return Err(QappInstallError::ManifestInvalid(
            "Installed Qap does not declare this scope or match its compiled host surface".into(),
        ));
    }
    let sheet_path = directory.join("lease-sheet.json");
    // Local install metadata can make an operation more restrictive. Its grant
    // bits and subject fields never supply verified operation authority.
    match fs::symlink_metadata(&sheet_path) {
        Ok(meta) if meta.is_file() && !reparse_point(&meta) => {}
        Ok(_) => {
            return Err(QappInstallError::ManifestInvalid(
                "Qap install lease sheet must be a regular file".into(),
            ))
        }
        Err(error) => return Err(error.into()),
    }
    let sheet: LeaseSheet =
        read_bounded_json(&sheet_path, 256 * 1024).map_err(QappInstallError::ManifestInvalid)?;
    if sheet.package_id != package_id
        || sheet.version != entry.active_version
        || sheet.default_policy != "deny"
        || sheet.guardian_install
    {
        return Err(QappInstallError::ManifestInvalid(
            "The Qap install lease sheet does not admit this host surface".into(),
        ));
    }
    Ok(())
}

/// Exact compiled graph surfaces still need the authenticated project owner's
/// signed read grant. The leaf operation uses that same grant and native session.
pub fn authorize_qap_graph_read(
    storage: &Path,
    package_id: &str,
    authority: &AdoptedQualiaFoundation,
) -> Result<(), String> {
    require_qap_capability(storage, package_id, CapabilityLeaseScope::QualiaQ42Storage)
        .map_err(|error| error.to_string())?;
    if !matches!(package_id, "Browser" | "Poet") {
        return Err("This Qap has no compiled native graph surface.".into());
    }
    let grant = authority
        .graph_grant()
        .ok_or("A signed graph read grant is required.")?;
    if !grant.grant.rights.read {
        return Err("The signed graph scope does not grant read access.".into());
    }
    authority
        .authority()
        .bind_graph_scope(&authority.session(), grant)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// Only the compiled Poet route can execute adopted source. The native check
/// binds the exact source bytes, signed graph scope, current session and actor.
pub fn authorize_qap_vibe_source(
    storage: &Path,
    authority: &AdoptedQualiaFoundation,
    source: &str,
) -> Result<(CallExecutionContext, u64), String> {
    require_qap_capability(
        storage,
        "Poet",
        CapabilityLeaseScope::GovernedVibeScriptEval,
    )
    .map_err(|error| error.to_string())?;
    require_qap_capability(storage, "Poet", CapabilityLeaseScope::QualiaQ42Storage)
        .map_err(|error| error.to_string())?;
    authority.authorize_source(source)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QapLeaseStatus {
    LocallyDenied,
    AwaitingAuthority,
    HostVerified,
    BuiltInSurface,
    Unavailable,
}

impl QapLeaseStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::LocallyDenied => "Revoked by local policy",
            Self::AwaitingAuthority => "Awaiting host authority",
            Self::HostVerified => "Current host scope verified",
            Self::BuiltInSurface => "Restricted built-in host surface available",
            Self::Unavailable => "Qap or policy unavailable",
        }
    }
}

/// Report current checks without granting access or evaluating user source.
pub fn qap_lease_status(
    storage: &Path,
    package_id: &str,
    scope: CapabilityLeaseScope,
    authority: Option<&AdoptedQualiaFoundation>,
) -> QapLeaseStatus {
    let status = || -> Result<QapLeaseStatus, String> {
        validate_package_id(package_id).map_err(|error| error.to_string())?;
        let registry = load_install_registry(storage).map_err(|error| error.to_string())?;
        reject_package_revocation(&registry, package_id).map_err(|error| error.to_string())?;
        let policy = OsControlCenterState::read_qap_policy(storage)?;
        if policy.qap_lease_revoked(package_id, scope) {
            return Ok(QapLeaseStatus::LocallyDenied);
        }
        let Some(entry) = registry.packages.get(package_id) else {
            return Ok(QapLeaseStatus::Unavailable);
        };
        if require_active_qap_surface(storage, package_id, scope, entry).is_err() {
            return Ok(QapLeaseStatus::AwaitingAuthority);
        }
        if matches!(
            (package_id, scope),
            ("Browser", CapabilityLeaseScope::GovernedVibeScriptEval)
                | (
                    "Browser" | "Poet" | "Library" | "Anatomy",
                    CapabilityLeaseScope::EtalonGpuSurface
                )
        ) {
            return Ok(QapLeaseStatus::BuiltInSurface);
        }
        let Some(authority) = authority else {
            return Ok(QapLeaseStatus::AwaitingAuthority);
        };
        let verified = match (package_id, scope) {
            ("Poet" | "Browser", CapabilityLeaseScope::QualiaQ42Storage) => {
                authorize_qap_graph_read(storage, package_id, authority).is_ok()
            }
            ("Poet", CapabilityLeaseScope::GovernedVibeScriptEval) => {
                require_qap_capability(storage, package_id, CapabilityLeaseScope::QualiaQ42Storage)
                    .is_ok()
                    && current_source_scope(authority).is_ok()
            }
            _ => false,
        };
        Ok(if verified {
            QapLeaseStatus::HostVerified
        } else {
            QapLeaseStatus::AwaitingAuthority
        })
    };
    status().unwrap_or(QapLeaseStatus::Unavailable)
}

fn current_source_scope(authority: &AdoptedQualiaFoundation) -> Result<(), String> {
    let grant = authority
        .graph_grant()
        .ok_or("A signed graph scope is required.")?;
    let owner = authority.authority();
    let session = authority.session();
    owner
        .bind_graph_scope(&session, grant)
        .map_err(|error| error.to_string())?;
    owner.transact(&session, |view| DocumentTransaction {
        result: authorize(
            view,
            &AuthoringRequest {
                action: AuthoringAction::ExecuteSource,
                target: view.current_document,
                expected_authority_generation: view.generation,
            },
        )
        .map(|_| ())
        .map_err(|error| error.to_string()),
        successor: None,
        reconciliation_required: false,
    })
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
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() && !reparse_point(&metadata) => {}
        Ok(_) => {
            return Err(QappInstallError::RegistryCorrupt(
                "Qap registry must be a regular file".into(),
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(QappInstallRegistry::default())
        }
        Err(error) => return Err(QappInstallError::RegistryCorrupt(error.to_string())),
    }
    read_bounded_json(&path, 2 * 1024 * 1024).map_err(QappInstallError::RegistryCorrupt)
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
                    // A declaration records demand. Only the executing host can grant it.
                    grant: false,
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
    install_package_with_guardian_policy(storage, source_dir, policy, trust, false)
}

fn install_package_with_guardian_policy(
    storage: &Path,
    source_dir: &Path,
    policy: InstallPolicy,
    trust: Option<&TrustRoots>,
    guardian_install: bool,
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

    let lease_sheet = build_lease_sheet(&manifest, policy, guardian_install)?;
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

    // Persist the default-deny sheet and guardian restriction with the install.
    let lease_json = serde_json::to_vec_pretty(&lease_sheet)
        .map_err(|error| QappInstallError::ManifestInvalid(error.to_string()))?;
    fs::write(dest.join("lease-sheet.json"), lease_json)?;

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
    let result = match src {
        InstallSource::Directory(dir) => install_package_with_guardian_policy(
            storage,
            &dir,
            policy,
            Some(trust),
            guardian_install,
        )?,
        InstallSource::QpkgZip(zip_path) => {
            let staging = qapps_root(storage)
                .join(".staging-zip")
                .join(format!("qpkg-{}", std::process::id()));
            let _ = fs::remove_dir_all(&staging);
            unpack_qpkg_zip(&zip_path, &staging)?;
            let res = install_package_with_guardian_policy(
                storage,
                &staging,
                policy,
                Some(trust),
                guardian_install,
            );
            let _ = fs::remove_dir_all(&staging);
            res?
        }
        InstallSource::YamlLdQ42(yaml_path) => {
            let staging = qapps_root(storage)
                .join(".staging-yaml")
                .join(format!("yaml-{}", std::process::id()));
            let _ = fs::remove_dir_all(&staging);
            synthesize_qapp_json_from_yaml(&yaml_path, &staging)?;
            let res = install_package_with_guardian_policy(
                storage,
                &staging,
                policy,
                Some(trust),
                guardian_install,
            );
            let _ = fs::remove_dir_all(&staging);
            res?
        }
    };
    Ok(result)
}

pub fn plan_qapp_launch(
    storage: &Path,
    package_id: &str,
    entry: Option<&str>,
) -> Result<LaunchPlan, QappInstallError> {
    validate_package_id(package_id)?;
    let registry = load_install_registry(storage)?;
    reject_package_revocation(&registry, package_id)?;
    let package_dir = qapps_root(storage).join(package_id);
    let manifest = read_qap_manifest(&package_dir, package_id)?;
    let required = manifest
        .x_qualia
        .iter()
        .flat_map(|extension| &extension.requires)
        .filter(|requirement| requirement.required)
        .filter_map(|requirement| {
            CapabilityLeaseScope::from_family_id(requirement.capability.trim())
        });
    let mut required = required.peekable();
    if required.peek().is_some() {
        let policy = OsControlCenterState::read_qap_policy(storage)
            .map_err(QappInstallError::RegistryCorrupt)?;
        for scope in required {
            check_qap_policy(&policy, package_id, scope)?;
        }
    }

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
        required_ontologies: ext
            .map(|x| x.required_ontologies.clone())
            .unwrap_or_default(),
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
        let dest = qapps_root(&host.storage)
            .join(name)
            .join(QAPP_PACKAGE_MANIFEST);
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
        let entry_name =
            String::from_utf8_lossy(&bytes[name_start..name_start + name_len]).replace('\\', "/");
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
    use qualia_cooperative_core::{
        authoring::{DocumentRevision, IdentifierAssociationEvidence, PrincipalKind},
        projects::{Project, ProjectMembership, ProjectRole},
    };
    use qualia_core_db::{
        crypto::network::ed25519,
        vibe_host::native_authority::{
            admission_digest,
            graph::{
                graph_grant_digest, GraphGrant, GraphRights, SignedGraphGrant, GRAPH_GRANT_CONTEXT,
            },
            AdmissionSnapshot, NativeAuthority, SignedAdmission, SubjectAdmission, TrustedProject,
            ADMISSION_CONTEXT, SESSION_CONTEXT,
        },
    };
    use std::{
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc,
        },
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestVault(PathBuf);

    impl TestVault {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let vault = std::env::temp_dir().join(format!(
                "webizen-qap-leases-{}-{now}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&vault).unwrap();
            Self(vault)
        }

        fn install(&self, name: &str, app_id: &str, required: &str, guardian: bool) {
            let source = self.0.join("fixture").join(name);
            fs::create_dir_all(&source).unwrap();
            fs::write(
                source.join(QAPP_PACKAGE_MANIFEST),
                serde_json::to_vec(&serde_json::json!({
                    "name": name,
                    "version": "1.0",
                    "x_qualia": {
                        "app_id": app_id,
                        "requires": [{ "capability": required, "required": true }]
                    }
                }))
                .unwrap(),
            )
            .unwrap();
            let (_, sheet) = install_from_source(
                &self.0,
                &TrustRoots::for_storage(&self.0),
                InstallSource::Directory(source),
                InstallPolicy::Development,
                guardian,
            )
            .unwrap();
            assert_eq!(sheet.guardian_install, guardian);
            assert!(sheet.families.iter().all(|family| !family.grant));
        }
    }

    impl Drop for TestVault {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn signed_foundation(vault: &Path, source: &str) -> AdoptedQualiaFoundation {
        // This is an isolated signed technical fixture, with no claim of human enrollment.
        let issuer_key = [13_u8; 32];
        let subject_key = [23_u8; 32];
        let project = "lease-test-project";
        let issuer = "lease-test-issuer";
        let subject = "test-operator";
        let did = "did:test:operator";
        let expires = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 900;
        let snapshot = AdmissionSnapshot {
            version: 1,
            sequence: 1,
            issuer_identifier: issuer.into(),
            project: Project {
                id: project.into(),
                name: "Qap authority fixture".into(),
                description: String::new(),
                licensing_ontologies: Vec::new(),
                created_at_unix: 1,
                created_at_instant: None,
            },
            valid_from_unix: 1,
            valid_until_unix: expires,
            revoked: false,
            subjects: vec![SubjectAdmission {
                subject: subject.into(),
                kind: PrincipalKind::SoftwareAgent,
                controller_public_key: ed25519::public_from_seed(&subject_key),
                membership: ProjectMembership {
                    id: "lease-test-membership".into(),
                    project_id: project.into(),
                    member_did: did.into(),
                    role: ProjectRole::Contributor,
                    agreed_at_unix: 1,
                    agreed_at_instant: None,
                },
                identifiers: vec![IdentifierAssociationEvidence {
                    value: did.into(),
                    scheme: "did:test".into(),
                    issuer: issuer.into(),
                    project_id: project.into(),
                    evidence_ref: "fixture-association".into(),
                    valid_from_unix: 1,
                    valid_until_unix: expires,
                }],
                reviews: Vec::new(),
                admission_ref: "fixture-admission".into(),
                valid_from_unix: 1,
                valid_until_unix: expires,
                revoked: false,
            }],
            package_reviews: Vec::new(),
        };
        let digest = admission_digest(&snapshot, 16 * 1024).unwrap();
        let mut admission_signature = [0; 64];
        ed25519::sign_with_context(
            &issuer_key,
            &digest,
            ADMISSION_CONTEXT,
            &mut admission_signature,
        )
        .unwrap();
        let journal = vault.join("authority");
        fs::create_dir(&journal).unwrap();
        let authority = Arc::new(
            NativeAuthority::open(
                &journal,
                TrustedProject {
                    project_id: project.into(),
                    issuer_identifier: issuer.into(),
                    public_key: ed25519::public_from_seed(&issuer_key),
                },
                SignedAdmission {
                    snapshot,
                    signature: admission_signature.to_vec(),
                },
                DocumentRevision {
                    project_id: project.into(),
                    document_id: "fixture-document".into(),
                    source_id: "fixture-source".into(),
                    revision: 1,
                    source_digest: vibe::crypto::sha256_bytes(source.as_bytes()),
                    author_subject: Some(subject.into()),
                },
                1024 * 1024,
                16,
            )
            .unwrap(),
        );
        let challenge = authority.begin_session(subject, 300).unwrap();
        let mut possession_proof = [0; 64];
        ed25519::sign_with_context(
            &subject_key,
            challenge.message_digest(),
            SESSION_CONTEXT,
            &mut possession_proof,
        )
        .unwrap();
        let session = authority
            .complete_session(challenge, &possession_proof)
            .unwrap();
        let grant = GraphGrant {
            version: 1,
            project_id: project.into(),
            subject: subject.into(),
            admission_digest: digest,
            context: 41,
            rights: GraphRights {
                read: true,
                replace: false,
                subscribe: false,
            },
            max_quins: 4,
            valid_from_unix: 1,
            valid_until_unix: expires,
        };
        let mut graph_signature = [0; 64];
        ed25519::sign_with_context(
            &issuer_key,
            &graph_grant_digest(&grant).unwrap(),
            GRAPH_GRANT_CONTEXT,
            &mut graph_signature,
        )
        .unwrap();
        AdoptedQualiaFoundation::connect(
            authority,
            session,
            Some(SignedGraphGrant {
                grant,
                signature: graph_signature.to_vec(),
            }),
        )
        .unwrap()
    }

    #[test]
    fn local_revoke_narrows_launch_and_live_admission_without_minting_a_grant() {
        use CapabilityLeaseScope::{GovernedVibeScriptEval, QualiaQ42Storage};
        let vault = TestVault::new();
        vault.install(
            "Browser",
            "did:qualia:qap:browser",
            "qualia.localDaemon.query",
            false,
        );
        vault.install(
            "Poet",
            "did:qualia:qap:poet",
            "qualia.service.scheduler",
            false,
        );
        let fabric = crate::host_stewardship::NaturalHostStewardshipFabric::default();
        let mut policy = OsControlCenterState::load_or_seed(&vault.0, &fabric).unwrap();
        policy
            .set_qap_lease(&vault.0, "Browser", QualiaQ42Storage, true)
            .unwrap();
        assert_eq!(
            qap_lease_status(&vault.0, "Browser", QualiaQ42Storage, None),
            QapLeaseStatus::AwaitingAuthority
        );
        assert!(plan_qapp_launch(&vault.0, "Browser", None).is_ok());
        assert!(plan_qapp_launch(&vault.0, "browser", None).is_err());

        let source = "= 21 + 21";
        let signed = signed_foundation(&vault.0, source);
        assert_eq!(
            qap_lease_status(&vault.0, "Browser", QualiaQ42Storage, Some(&signed)),
            QapLeaseStatus::HostVerified
        );
        assert!(authorize_qap_graph_read(&vault.0, "Browser", &signed).is_ok());
        assert!(authorize_qap_graph_read(&vault.0, "Library", &signed).is_err());
        assert!(authorize_qap_vibe_source(&vault.0, &signed, source).is_ok());
        assert!(authorize_qap_vibe_source(&vault.0, &signed, "= 41 + 2").is_err());

        policy
            .set_qap_lease(&vault.0, "Browser", QualiaQ42Storage, false)
            .unwrap();
        assert_eq!(
            qap_lease_status(&vault.0, "Browser", QualiaQ42Storage, Some(&signed)),
            QapLeaseStatus::LocallyDenied
        );
        assert!(plan_qapp_launch(&vault.0, "Browser", None).is_err());
        assert!(authorize_qap_graph_read(&vault.0, "Browser", &signed).is_err());
        policy
            .set_qap_lease(&vault.0, "Poet", QualiaQ42Storage, false)
            .unwrap();
        assert!(authorize_qap_vibe_source(&vault.0, &signed, source).is_err());
        policy
            .set_qap_lease(&vault.0, "Poet", QualiaQ42Storage, true)
            .unwrap();
        policy
            .set_qap_lease(&vault.0, "Poet", GovernedVibeScriptEval, true)
            .unwrap();
        assert!(authorize_qap_vibe_source(&vault.0, &signed, source).is_ok());
        signed.authority().disconnect(&signed.session()).unwrap();
        assert!(authorize_qap_vibe_source(&vault.0, &signed, source).is_err());
    }

    #[test]
    fn denials_do_not_turn_broad_services_into_capabilities_and_guardian_policy_persists() {
        let vault = TestVault::new();
        vault.install(
            "Browser",
            "did:qualia:qap:browser",
            "qualia.service.networking",
            true,
        );
        assert!(plan_qapp_launch(&vault.0, "Browser", None).is_ok());
        assert!(require_qap_capability(
            &vault.0,
            "Browser",
            CapabilityLeaseScope::QualiaQ42Storage
        )
        .is_err());
        assert!(CapabilityLeaseScope::from_family_id("qualia.service.networking").is_none());
        let signed = signed_foundation(&vault.0, "= 1");
        assert!(authorize_qap_graph_read(&vault.0, "Browser", &signed).is_err());
    }

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
