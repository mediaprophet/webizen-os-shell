//! Dual-SKU Binary Delivery & Synth-Style Ed25519 Updater (`C6`, `C7`, `A7`, `D5`, `F4`).
//!
//! 1. **SKU A — Guest Shell Binary Installer & Self-Updater (`C6`, `C7`, `KD-16`)**:
//!    - Delivers `webizen-os-shell`, `qualia-cli` sidecar (`:4242`), and optional
//!      Ring C `www-engine-worker` + `worker-abi.json` onto existing Windows, macOS,
//!      and Linux Device OS environments without altering the host OS.
//!    - Implements `webizen-os-shell update check|upgrade|verify|apply` with a
//!      host-provisioned vendor trust key and atomic `.new` executable staging
//!      (never overwriting a locked running binary).
//! 2. **SKU B — Linux-Backed User-Agent OS Appliance Overlay (`D5`, `F4`, `KD-3`, `KD-19`)**:
//!    - Emits the Debian 13 Trixie `/opt/webizen` overlay closure (`qualiadb.service`,
//!      `webizen-hostd.service`, `webizen-os-shell.service` Wayland `cage` DRM/KMS
//!      session, `webizen.target`, three-ring `LICENSE-BOUNDARY.md`, and zero-touch
//!      `provision-debian-trixie-overlay.sh`) for `amd64` and `arm64` targets
//!      (VPS, Desktop Workstation, Raspberry Pi, Android 16 AVF `pKVM`, and WSL2).

use crate::host_stewardship::{
    validate_human_centric_host_nomenclature, HostUpdateProposal, NaturalHostStewardshipFabric,
};
use crate::native_gpu::NativeEtalonStatus;
use crate::session::{LoopbackHost, Sku};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashSet;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Legacy descriptor value preserved for source compatibility. It encodes an off-curve
/// Ed25519 point. Guest updates require a real host-provisioned vendor public key.
pub const FIRST_PARTY_UPDATER_PUBKEY_HEX: &str =
    "7f18b3209dad11d180b400c04fd430c84210d00306917504aa55ee1120261122";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateAction {
    Check,
    Verify,
    Upgrade,
    Apply,
}

impl UpdateAction {
    pub fn parse(arg: &str) -> Option<Self> {
        match arg.trim().to_ascii_lowercase().as_str() {
            "check" => Some(Self::Check),
            "verify" => Some(Self::Verify),
            "upgrade" => Some(Self::Upgrade),
            "apply" => Some(Self::Apply),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    pub sku: String,
    pub artifact_name: String,
    pub sha3_256_hex: String,
    #[serde(default)]
    pub signature_hex: String,
    #[serde(default)]
    pub pubkey_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quorum: Option<QuorumUpdateProof>,
}

/// Detached signatures for one suspended host update proposal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuorumUpdateProof {
    pub proposal_id: String,
    pub approvals: Vec<QuorumApproval>,
}

/// An Ed25519 approval from an enrolled steward key.
/// Both hex fields encode raw bytes, in RFC 8032 format.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuorumApproval {
    pub steward_did: String,
    pub pubkey_hex: String,
    pub signature_hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateReceipt {
    pub action: String,
    pub ok: bool,
    pub current_version: String,
    pub staged_path: Option<PathBuf>,
    pub detail: String,
}

/// Summary of generated Dual-SKU delivery artefacts (`C6` Guest bundle + `D5`/`F4` Debian Trixie overlay).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DualSkuPackagingReceipt {
    pub output_dir: PathBuf,
    pub license_boundary_path: PathBuf,
    pub guest_manifest_path: PathBuf,
    pub debian_overlay_dir: PathBuf,
    pub systemd_units: Vec<String>,
    pub hardware_attested_vault_advertised: bool,
}

/// Verify signatures and governance against the caller's trusted local fabric.
/// The installer must also compare the candidate file to the returned digest.
pub fn verify_quorum_update_manifest(
    manifest: &UpdateManifest,
    fabric: &NaturalHostStewardshipFabric,
) -> Result<String, String> {
    verify_quorum_with(manifest, fabric, verify_detached_ed25519)
}

/// Build the exact message an enrolled steward signs for a queued proposal.
/// The length-prefixed fields bind the signer, host, proposal, artifact, and threshold.
pub fn quorum_update_signing_payload(
    manifest: &UpdateManifest,
    fabric: &NaturalHostStewardshipFabric,
    steward_did: &str,
) -> Result<Vec<u8>, String> {
    let quorum = quorum_context(manifest, fabric)?;
    quorum.approval_payload(manifest, steward_did)
}

/// Verify an RFC 8032 Ed25519 signature with an already trusted raw public key.
/// OpenSSL 3 is invoked directly, without a shell or a PATH search. Hosts may set
/// WEBIZEN_OPENSSL_BIN to an absolute path to their trusted OpenSSL executable.
pub fn verify_detached_ed25519(
    public_key_hex: &str,
    signature_hex: &str,
    payload: &[u8],
) -> Result<(), String> {
    let public_key = decode_hex::<32>(public_key_hex)?;
    let signature = decode_hex::<64>(signature_hex)?;
    if payload.is_empty() || payload.len() > 256 * 1024 {
        return Err("Ed25519 verification payload is empty or too large".into());
    }
    // RFC 8410 SubjectPublicKeyInfo for a raw Ed25519 public key.
    let mut spki = Vec::from([
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
    ]);
    spki.extend_from_slice(&public_key);
    let verifier = openssl_verifier()?;
    let scratch = ScratchDirectory::create(&std::env::temp_dir(), "signature")?;
    let public_path = scratch.path.join("public.der");
    let signature_path = scratch.path.join("signature.bin");
    let message_path = scratch.path.join("message.bin");
    write_new_file(&public_path, &spki)?;
    write_new_file(&signature_path, &signature)?;
    write_new_file(&message_path, payload)?;

    let mut command = Command::new(&verifier);
    command.env_clear().current_dir(&scratch.path);
    #[cfg(windows)]
    {
        if let Some(sys_root) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", sys_root);
        }
        if let Some(bin_dir) = verifier.parent() {
            command.env("PATH", bin_dir);
        }
    }
    command
        .args([
            "pkeyutl",
            "-verify",
            "-pubin",
            "-keyform",
            "DER",
            "-rawin",
            "-inkey",
            "public.der",
            "-sigfile",
            "signature.bin",
            "-in",
            "message.bin",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let child = command
        .spawn()
        .map_err(|error| format!("Cannot start the OpenSSL Ed25519 verifier: {error}"))?;
    let result = wait_for_verifier(child, Duration::from_secs(10))
        .map_err(|error| format!("Cannot complete OpenSSL Ed25519 verification: {error}"))?;
    if !result.success() {
        return Err("OpenSSL rejected the Ed25519 signature or could not verify it".into());
    }
    Ok(())
}

/// Wait for a directly launched verifier while keeping native UI calls bounded.
pub(crate) fn wait_for_verifier(mut child: Child, timeout: Duration) -> io::Result<ExitStatus> {
    let deadline = Instant::now() + timeout;
    let error = loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(15));
            }
            Ok(None) => {
                break io::Error::new(io::ErrorKind::TimedOut, "verifier deadline exceeded")
            }
            Err(error) => break error,
        }
    };
    let _ = child.kill();
    let _ = child.wait();
    Err(error)
}

struct QuorumContext<'a> {
    proof: &'a QuorumUpdateProof,
    proposal: &'a HostUpdateProposal,
    required: u8,
}

impl QuorumContext<'_> {
    fn approval_payload(
        &self,
        manifest: &UpdateManifest,
        steward_did: &str,
    ) -> Result<Vec<u8>, String> {
        validate_steward_did(steward_did)?;
        let digest = manifest.sha3_256_hex.to_ascii_lowercase();
        encode_signing_message(
            b"webizen.host.quorum-approval/v1\0",
            &[
                self.proposal.target_host_domain.as_str(),
                self.proof.proposal_id.as_str(),
                steward_did,
                manifest.sku.as_str(),
                manifest.version.as_str(),
                manifest.artifact_name.as_str(),
                digest.as_str(),
            ],
            &[self.required],
        )
    }
}

fn quorum_context<'a>(
    manifest: &'a UpdateManifest,
    fabric: &'a NaturalHostStewardshipFabric,
) -> Result<QuorumContext<'a>, String> {
    validate_manifest_fields(manifest)?;
    let proof = manifest
        .quorum
        .as_ref()
        .ok_or("No host quorum proof was supplied")?;
    let mut matching_proposals = fabric
        .update_queue
        .iter()
        .filter(|proposal| proposal.proposal_id == proof.proposal_id);
    let proposal = matching_proposals
        .next()
        .ok_or("The host update proposal is absent")?;
    if matching_proposals.next().is_some()
        || proposal.target_host_domain != fabric.primary_host_domain
        || proposal.target_deb_package != manifest.artifact_name
        || proposal.target_version != manifest.version
        || proposal.applied_to_host
    {
        return Err("The proposal does not match this pending host update".into());
    }
    if !proposal.artifact_sha3_256.is_empty()
        && !proposal
            .artifact_sha3_256
            .eq_ignore_ascii_case(&manifest.sha3_256_hex)
    {
        return Err("The manifest digest differs from the queued artifact digest".into());
    }
    if fabric.update_queue.iter().any(|item| {
        item.target_deb_package == manifest.artifact_name
            && item.target_version == manifest.version
            && item.target_host_domain == fabric.primary_host_domain
            && (item.paraconsistent_review_flagged || item.votes.iter().any(|vote| !vote.approve))
    }) {
        return Err("An unresolved steward dissent requires human review (0x32)".into());
    }
    let required = fabric.threshold_m.max(proposal.required_m);
    if fabric.threshold_m == 0
        || proposal.required_m == 0
        || usize::from(required) > fabric.stewards.len()
    {
        return Err("The steward quorum threshold is invalid".into());
    }
    Ok(QuorumContext {
        proof,
        proposal,
        required,
    })
}

fn verify_quorum_with(
    manifest: &UpdateManifest,
    fabric: &NaturalHostStewardshipFabric,
    verify: impl Fn(&str, &str, &[u8]) -> Result<(), String>,
) -> Result<String, String> {
    let quorum = quorum_context(manifest, fabric)?;
    let approvals = &quorum.proof.approvals;
    if approvals.len() < usize::from(quorum.required) || approvals.len() > fabric.stewards.len() {
        return Err(format!(
            "Host update requires {} distinct steward approvals",
            quorum.required
        ));
    }
    let mut seen_stewards = HashSet::new();
    let mut seen_keys = HashSet::new();
    for approval in approvals {
        validate_steward_did(&approval.steward_did)?;
        let mut matching_stewards = fabric
            .stewards
            .iter()
            .filter(|steward| steward.steward_did == approval.steward_did);
        let steward = matching_stewards
            .next()
            .ok_or("A signer is not an enrolled steward")?;
        if matching_stewards.next().is_some() || !steward.has_full_legal_capacity {
            return Err("A signer is not an unambiguous, independently eligible steward".into());
        }
        let public_key = decode_hex::<32>(&approval.pubkey_hex)?;
        let _signature = decode_hex::<64>(&approval.signature_hex)?;
        if !seen_stewards.insert(approval.steward_did.as_str()) || !seen_keys.insert(public_key) {
            return Err("Host update signatures repeat a steward or public key".into());
        }
        let expected_pin = format!("ed25519:sha3-256:{}", sha3_256_hex(&public_key));
        if steward.dual_pubkey_fingerprint != expected_pin {
            return Err("The Ed25519 key is not pinned by the trusted steward roster".into());
        }
        let message = quorum.approval_payload(manifest, &approval.steward_did)?;
        verify(&approval.pubkey_hex, &approval.signature_hex, &message)?;
    }
    Ok(manifest.sha3_256_hex.to_ascii_lowercase())
}

fn validate_steward_did(steward_did: &str) -> Result<(), String> {
    let lower = steward_did.to_ascii_lowercase();
    let valid_did = steward_did
        .strip_prefix("did:")
        .and_then(|value| value.split_once(':'))
        .is_some_and(|(method, identifier)| {
            !identifier.is_empty()
                && !method.is_empty()
                && method
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        });
    if !valid_did
        || steward_did.chars().any(char::is_whitespace)
        || [":agent:", ":instrument:", ":bot"]
            .iter()
            .any(|marker| lower.contains(*marker))
    {
        return Err("An independent natural-person steward DID is required".into());
    }
    validate_human_centric_host_nomenclature(steward_did)
}

/// Build the signed message for a first-party Guest update.
pub fn vendor_update_signing_payload(manifest: &UpdateManifest) -> Result<Vec<u8>, String> {
    validate_manifest_fields(manifest)?;
    let digest = manifest.sha3_256_hex.to_ascii_lowercase();
    let mut fields = vec![
        manifest.sku.as_str(),
        manifest.version.as_str(),
        manifest.artifact_name.as_str(),
        digest.as_str(),
    ];
    if let Some(proof) = &manifest.quorum {
        fields.push(&proof.proposal_id);
    }
    encode_signing_message(
        b"webizen.vendor-update/v1\0",
        &fields,
        &[u8::from(manifest.quorum.is_some())],
    )
}

fn validate_manifest_fields(manifest: &UpdateManifest) -> Result<(), String> {
    decode_hex::<32>(&manifest.sha3_256_hex)?;
    if !matches!(manifest.sku.as_str(), "guest" | "appliance")
        || manifest.version.is_empty()
        || manifest.version.trim() != manifest.version
        || manifest.artifact_name.is_empty()
        || matches!(manifest.artifact_name.as_str(), "." | "..")
        || manifest.artifact_name.trim() != manifest.artifact_name
        || manifest.artifact_name.contains(['/', '\\', ':'])
    {
        return Err("The update manifest has an invalid SKU, version, or artifact filename".into());
    }
    Ok(())
}

fn encode_signing_message(
    domain: &[u8],
    fields: &[&str],
    policy: &[u8],
) -> Result<Vec<u8>, String> {
    let mut payload = domain.to_vec();
    for field in fields {
        if field.is_empty() || field.len() > 4096 || field.chars().any(char::is_control) {
            return Err(
                "An update signing field is empty, too large, or contains a control character"
                    .into(),
            );
        }
        validate_human_centric_host_nomenclature(field)?;
        payload.extend_from_slice(&(field.len() as u32).to_be_bytes());
        payload.extend_from_slice(field.as_bytes());
    }
    payload.extend_from_slice(policy);
    Ok(payload)
}

fn decode_hex<const N: usize>(hex: &str) -> Result<[u8; N], String> {
    if hex.len() != 2 * N {
        return Err(format!("Expected {N} bytes encoded as hexadecimal"));
    }
    let mut decoded = [0u8; N];
    for (byte, pair) in decoded.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        let high = char::from(pair[0])
            .to_digit(16)
            .ok_or("Invalid hexadecimal")?;
        let low = char::from(pair[1])
            .to_digit(16)
            .ok_or("Invalid hexadecimal")?;
        *byte = (high * 16 + low) as u8;
    }
    Ok(decoded)
}

fn openssl_verifier() -> Result<PathBuf, String> {
    let supplied = std::env::var_os("WEBIZEN_OPENSSL_BIN").map(PathBuf::from);
    #[cfg(windows)]
    let default = std::env::var_os("ProgramFiles").and_then(|base| {
        let base = PathBuf::from(base);
        ["Git/ucrt64/bin/openssl.exe", "Git/mingw64/bin/openssl.exe"]
            .into_iter()
            .map(|relative| base.join(relative))
            .find(|path| path.is_file())
    });
    #[cfg(not(windows))]
    let default = Some(PathBuf::from("/usr/bin/openssl"));
    supplied
        .or(default)
        .filter(|path| path.is_absolute() && path.is_file())
        .ok_or_else(|| {
            "Set WEBIZEN_OPENSSL_BIN to the absolute path of a trusted OpenSSL 3 executable".into()
        })
}

// Both artifact and signature workspaces hold only files created by this process.
struct ScratchDirectory {
    path: PathBuf,
}

impl ScratchDirectory {
    fn create(root: &Path, purpose: &str) -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = root
            .canonicalize()
            .map_err(|error| format!("Cannot locate update workspace: {error}"))?;
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = DirBuilder::new();
            builder.mode(0o700);
            builder
        };
        #[cfg(not(unix))]
        let builder = DirBuilder::new();
        for _ in 0..128 {
            let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!(
                ".webizen-{purpose}-{}-{sequence}",
                std::process::id()
            ));
            match builder.create(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("Cannot create update workspace: {error}")),
            }
        }
        Err("Could not reserve a private update workspace".into())
    }
}

impl Drop for ScratchDirectory {
    fn drop(&mut self) {
        for name in [
            "public.der",
            "signature.bin",
            "message.bin",
            "candidate.bin",
        ] {
            let _ = fs::remove_file(self.path.join(name));
        }
        let _ = fs::remove_dir(&self.path);
    }
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(bytes))
        .map_err(|error| format!("Cannot write verification workspace: {error}"))
}

/// Execute the shell updater. Guest commits a verified candidate beside the running executable.
/// Appliance verifies admission evidence and leaves package installation to APT.
pub fn run_updater(
    host: &LoopbackHost,
    sku: Sku,
    action: UpdateAction,
    candidate_binary: Option<&Path>,
) -> UpdateReceipt {
    let staged_new = host.install_root.join(if cfg!(windows) {
        "webizen-os-shell.exe.new"
    } else {
        "webizen-os-shell.new"
    });
    let manifest_path = host.storage.join("update-manifest.json");
    let result = match action {
        UpdateAction::Check => match read_update_json::<UpdateManifest>(&manifest_path) {
            Ok(manifest) => Ok((
                (sku == Sku::Guest && staged_new.is_file()).then_some(staged_new),
                format!(
                    "Update manifest available for v{} ({})",
                    manifest.version, manifest.artifact_name
                ),
            )),
            Err(_) if !manifest_path.exists() => Ok((
                (sku == Sku::Guest && staged_new.is_file()).then_some(staged_new),
                "No update manifest is available".into(),
            )),
            Err(error) => Err(error),
        },
        _ => verify_or_stage_update(
            host,
            sku,
            action,
            candidate_binary,
            &manifest_path,
            &staged_new,
        )
        .map(|(path, detail)| (Some(path), detail)),
    };
    let current_version = env!("CARGO_PKG_VERSION").into();
    let action = format!("{action:?}").to_ascii_lowercase();
    match result {
        Ok((staged_path, detail)) => UpdateReceipt {
            action,
            ok: true,
            current_version,
            staged_path,
            detail,
        },
        Err(detail) => UpdateReceipt {
            action,
            ok: false,
            current_version,
            staged_path: None,
            detail,
        },
    }
}

fn verify_or_stage_update(
    host: &LoopbackHost,
    sku: Sku,
    action: UpdateAction,
    candidate_binary: Option<&Path>,
    manifest_path: &Path,
    staged_new: &Path,
) -> Result<(PathBuf, String), String> {
    let manifest: UpdateManifest = read_update_json(manifest_path)?;
    validate_manifest_fields(&manifest)?;
    if manifest.sku != sku.as_str() {
        return Err("The update targets a different SKU".into());
    }
    let appliance = sku == Sku::Appliance;
    if appliance && !manifest.artifact_name.ends_with(".deb") {
        return Err("An Appliance update must identify a Debian package".into());
    }
    let appliance_candidate = host.storage.join("updates").join(&manifest.artifact_name);
    let candidate = candidate_binary.unwrap_or(if appliance {
        &appliance_candidate
    } else {
        staged_new
    });
    if !candidate.is_file() {
        return Err(format!("No update artifact at {}", candidate.display()));
    }
    let mut input =
        File::open(candidate).map_err(|error| format!("Cannot read update artifact: {error}"))?;

    if appliance || action == UpdateAction::Verify {
        let digest = digest_and_copy(&mut input, &mut io::sink())?;
        authorize_update(sku, &manifest, &digest)?;
        let message = if appliance {
            "Authenticated host quorum and package digest for admission. APT handles package installation."
        } else {
            "Verified the update signature and artifact digest."
        };
        return Ok((
            candidate.to_path_buf(),
            format!("{message} SHA3-256 {digest}"),
        ));
    }

    let exe_name = if cfg!(windows) {
        "webizen-os-shell.exe"
    } else {
        "webizen-os-shell"
    };
    let target_exe = host.install_root.join(exe_name);
    if target_exe
        .canonicalize()
        .ok()
        .is_some_and(|target| candidate.canonicalize().ok() == Some(target))
    {
        return Err("The running executable cannot serve as the staged update candidate".into());
    }
    let scratch = ScratchDirectory::create(&host.install_root, "update")?;
    let temp_candidate = scratch.path.join("candidate.bin");
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_candidate)
        .map_err(|error| format!("Cannot stage update artifact: {error}"))?;
    let digest = digest_and_copy(&mut input, &mut output)?;
    drop(input);
    authorize_update(sku, &manifest, &digest)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        output
            .set_permissions(fs::Permissions::from_mode(0o755))
            .map_err(|error| format!("Cannot set staged executable permissions: {error}"))?;
    }
    output
        .sync_all()
        .map_err(|error| format!("Cannot sync staged update: {error}"))?;
    drop(output);
    fs::rename(&temp_candidate, staged_new)
        .map_err(|error| format!("Cannot publish verified .new update: {error}"))?;
    Ok((
        staged_new.to_path_buf(),
        format!("Staged verified update with SHA3-256 {digest}"),
    ))
}

fn authorize_update(
    sku: Sku,
    manifest: &UpdateManifest,
    computed_digest: &str,
) -> Result<(), String> {
    if !manifest.sha3_256_hex.eq_ignore_ascii_case(computed_digest) {
        return Err("The candidate SHA3-256 digest does not match its update manifest".into());
    }
    if sku == Sku::Guest {
        decode_hex::<64>(&manifest.signature_hex)?;
        let trusted_key_path = provisioned_trust_file("WEBIZEN_VENDOR_UPDATE_PUBKEY_FILE")?;
        let file = File::open(trusted_key_path)
            .map_err(|error| format!("Cannot open the provisioned Guest vendor key: {error}"))?;
        let mut key = String::new();
        file.take(129)
            .read_to_string(&mut key)
            .map_err(|error| format!("Cannot read the provisioned Guest vendor key: {error}"))?;
        let trusted_key = key.trim();
        if key.len() > 128
            || decode_hex::<32>(trusted_key)? != decode_hex::<32>(&manifest.pubkey_hex)?
        {
            return Err(
                "The Guest update key does not match the provisioned vendor trust root".into(),
            );
        }
        verify_detached_ed25519(
            trusted_key,
            &manifest.signature_hex,
            &vendor_update_signing_payload(manifest)?,
        )?;
    }
    if sku == Sku::Appliance || manifest.quorum.is_some() {
        let trusted_path = provisioned_trust_file("WEBIZEN_UPDATE_TRUST_FABRIC")?;
        let fabric = read_update_json(&trusted_path)?;
        verify_quorum_update_manifest(manifest, &fabric)?;
    }
    Ok(())
}

fn provisioned_trust_file(variable: &str) -> Result<PathBuf, String> {
    let path = std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| format!("Set {variable} to an absolute host-provisioned trust file"))?;
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("Cannot inspect provisioned trust file {variable}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!(
            "Provisioned trust file {variable} must be a regular file"
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o022 != 0 {
            return Err(format!(
                "Provisioned trust file {variable} allows group or public writes"
            ));
        }
    }
    Ok(path)
}

fn digest_and_copy(reader: &mut impl Read, writer: &mut impl Write) -> Result<String, String> {
    let mut hasher = Sha3_256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("Cannot read update artifact: {error}"))?;
        if count == 0 {
            return Ok(hasher
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect());
        }
        let bytes = &buffer[..count];
        hasher.update(bytes);
        writer
            .write_all(bytes)
            .map_err(|error| format!("Cannot stage artifact: {error}"))?;
    }
}

fn read_update_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    const MAX_BYTES: u64 = 4 * 1024 * 1024;
    if !path.is_file() {
        return Err(format!(
            "Required update evidence is missing at {}",
            path.display()
        ));
    }
    let file = File::open(path).map_err(|error| format!("Cannot open update evidence: {error}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Cannot read update evidence: {error}"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Update evidence exceeds the local size limit".into());
    }
    serde_json::from_slice(&bytes).map_err(|error| format!("Invalid update evidence: {error}"))
}

/// Generate the Dual-SKU delivery package descriptors (`C6` Guest Binary Installer +
/// `D5`/`F4` Debian 13 Trixie Linux User-Agent OS Appliance Overlay + `A7` Three-Ring
/// `LICENSE-BOUNDARY.md`).
pub fn generate_dual_sku_delivery_artifacts(
    host: &LoopbackHost,
    gpu: &NativeEtalonStatus,
) -> Result<DualSkuPackagingReceipt, String> {
    let out_dir = host.storage.join("delivery");
    let guest_dir = out_dir.join("sku-a-guest-installer");
    let debian_dir = out_dir.join("sku-b-debian-trixie-overlay");
    let systemd_dir = debian_dir.join("etc/systemd/system");

    fs::create_dir_all(&guest_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&systemd_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(debian_dir.join("opt/webizen/bin")).map_err(|e| e.to_string())?;

    // 1. A7 Three-Ring LICENSE-BOUNDARY.md
    let license_boundary = render_three_ring_license_boundary();
    let license_boundary_path = out_dir.join("LICENSE-BOUNDARY.md");
    fs::write(&license_boundary_path, &license_boundary).map_err(|e| e.to_string())?;
    fs::write(guest_dir.join("LICENSE-BOUNDARY.md"), &license_boundary)
        .map_err(|e| e.to_string())?;
    fs::write(
        debian_dir.join("opt/webizen/LICENSE-BOUNDARY.md"),
        &license_boundary,
    )
    .map_err(|e| e.to_string())?;

    // 2. SKU A — Guest Shell Binary Installer Manifest (`C6`)
    // N4 gate: only advertise hardware-attested vault if live Anvil NP64 supports it.
    let attestation_claim = if gpu.may_advertise_hardware_attested_vault {
        "hardware-attested vault"
    } else {
        "OS-user vault isolation (Anvil NP64 hardware-attestation gate closed on this host)"
    };

    let guest_manifest = serde_json::json!({
        "schema": "webizen.guest-installer-bundle/v1",
        "sku": "guest",
        "version": env!("CARGO_PKG_VERSION"),
        "license_ring_b": "CC BY-NC-ND 4.0",
        "license_ring_c_worker": "MPL-2.0 (sibling process only; never linked into Ring B)",
        "targets": ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu", "aarch64-apple-darwin"],
        "binaries": [
            {
                "name": "webizen-os-shell",
                "role": "Aether-native Etalon GPU spatial shell + integrated Poet & Hypermedia workbench"
            },
            {
                "name": "qualia-cli",
                "role": "QualiaDB loopback sidecar daemon (127.0.0.1:4242) owning .q42 volumes, 42 MiB SlgArena Sentinel, and local SLD/LLM inference"
            },
            {
                "name": "www-engine-worker",
                "role": "Optional Ring C MPL-2.0 old-web engine sibling worker (copied RGBA frames over IPC; worker-abi.json)"
            }
        ],
        "updater_vendor_trust_key_file_env": "WEBIZEN_VENDOR_UPDATE_PUBKEY_FILE",
        "vault_isolation": attestation_claim,
    });
    let guest_manifest_path = guest_dir.join("guest-bundle-manifest.json");
    fs::write(
        &guest_manifest_path,
        serde_json::to_string_pretty(&guest_manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    // 3. SKU B — Debian 13 Trixie Linux User-Agent OS Appliance Overlay (`D5` + `F4`)
    let qualiadb_service = r#"[Unit]
Description=QualiaDB Semantic Graph, 42 MiB SlgArena Sentinel & Local SLD/LLM Daemon (Ring B)
After=network-online.target
Wants=network-online.target
PartOf=webizen.target

[Service]
Type=simple
User=qualiadb
Group=qualiadb
Environment=QUALIA_STORAGE_PATH=/var/lib/qualiadb
Environment=QUALIA_DATA_DIR=/var/lib/qualiadb
Environment=ANVIL_NODE_PROFILE=/var/lib/qualiadb/anvil_node_profile.np64
ExecStart=/opt/webizen/bin/qualia-cli daemon start --port 4242
Restart=on-failure
RestartSec=3
MemoryMax=512M
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/qualiadb

[Install]
WantedBy=webizen.target
"#;

    let hostd_service = r#"[Unit]
Description=Webizen Host VPS & Personal Cloud Supervisor (StrictRetail Auth, Q42\0 Volumes)
After=qualiadb.service
Requires=qualiadb.service
PartOf=webizen.target

[Service]
Type=simple
User=qualiadb
Group=qualiadb
Environment=QUALIA_STORAGE_PATH=/var/lib/qualiadb
Environment=QUALIA_DATA_DIR=/var/lib/qualiadb
Environment=WEBIZEN_AUTH_MODE=StrictRetail
ExecStart=/opt/webizen/bin/webizen-hostd --bind 127.0.0.1:8080
Restart=on-failure
RestartSec=3
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/qualiadb /var/lib/webizen-host

[Install]
WantedBy=webizen.target
"#;

    let os_shell_service = r#"[Unit]
Description=Webizen User-Agent OS Graphical Appliance Session (Wayland Cage + Etalon Native Vulkan DRM/KMS)
After=qualiadb.service systemd-user-sessions.service
Requires=qualiadb.service
Conflicts=getty@tty1.service
PartOf=webizen.target

[Service]
Type=simple
User=webizen
Group=webizen
SupplementaryGroups=video render input audio
TTYPath=/dev/tty1
Environment=WEBIZEN_SKU=appliance
Environment=QUALIA_STORAGE_PATH=/var/lib/qualiadb
Environment=QUALIA_DATA_DIR=/var/lib/qualiadb
Environment=XDG_SESSION_TYPE=wayland
ExecStart=/usr/bin/cage -s -- /opt/webizen/bin/webizen-os-shell
Restart=always
RestartSec=2

[Install]
WantedBy=webizen.target
"#;

    let webizen_target = r#"[Unit]
Description=Webizen Human-Centric User-Agent OS Target (Overlay on Unmodified Debian 13 Trixie)
Requires=multi-user.target qualiadb.service
Wants=webizen-hostd.service
AllowIsolate=yes
"#;

    let provision_script = r#"#!/usr/bin/env bash
# Webizen User-Agent OS Overlay Provisioner for Unmodified Debian 13 (Trixie)
# Targets: amd64 / arm64 (VPS, Desktop Workstation Wayland Appliance, RPi, Android 16 AVF pKVM, WSL2)
# Ring A (Debian Trixie Device OS) remains unmodified upstream; Ring B installs under /opt/webizen.
set -euo pipefail

PROFILE="${1:-vps}" # vps | desktop-appliance | avf-sanctuary
install -d -m 0755 /opt/webizen/bin /etc/webizen /var/lib/qualiadb /var/lib/webizen-host
echo "appliance" > /etc/webizen/sku

if ! id -u qualiadb >/dev/null 2>&1; then
    useradd --system --home-dir /var/lib/qualiadb --shell /usr/sbin/nologin qualiadb
fi
chown -R qualiadb:qualiadb /var/lib/qualiadb /var/lib/webizen-host

cp -f etc/systemd/system/qualiadb.service /etc/systemd/system/qualiadb.service
cp -f etc/systemd/system/webizen-hostd.service /etc/systemd/system/webizen-hostd.service
cp -f etc/systemd/system/webizen-os-shell.service /etc/systemd/system/webizen-os-shell.service
cp -f etc/systemd/system/webizen.target /etc/systemd/system/webizen.target

systemctl daemon-reload
systemctl enable qualiadb.service webizen.target
if [ "$PROFILE" = "desktop-appliance" ]; then
    systemctl enable webizen-os-shell.service
    systemctl set-default webizen.target
else
    systemctl enable webizen-hostd.service
fi
"#;

    fs::write(systemd_dir.join("qualiadb.service"), qualiadb_service).map_err(|e| e.to_string())?;
    fs::write(systemd_dir.join("webizen-hostd.service"), hostd_service)
        .map_err(|e| e.to_string())?;
    fs::write(
        systemd_dir.join("webizen-os-shell.service"),
        os_shell_service,
    )
    .map_err(|e| e.to_string())?;
    fs::write(systemd_dir.join("webizen.target"), webizen_target).map_err(|e| e.to_string())?;
    fs::write(
        debian_dir.join("provision-debian-trixie-overlay.sh"),
        provision_script,
    )
    .map_err(|e| e.to_string())?;

    Ok(DualSkuPackagingReceipt {
        output_dir: out_dir,
        license_boundary_path,
        guest_manifest_path,
        debian_overlay_dir: debian_dir,
        systemd_units: vec![
            "qualiadb.service".into(),
            "webizen-hostd.service".into(),
            "webizen-os-shell.service".into(),
            "webizen.target".into(),
        ],
        hardware_attested_vault_advertised: gpu.may_advertise_hardware_attested_vault,
    })
}

fn render_three_ring_license_boundary() -> String {
    String::from(
        "# Webizen / QualiaDB Three-Ring Licence Boundary (`A7` / `KD-19`)\n\n\
         Copyright (c) 2026 Timothy Charles Holborn <timothy.holborn@gmail.com>\n\n\
         | Ring | Scope | Licence & Isolation Rule |\n\
         | :--- | :--- | :--- |\n\
         | **Ring A — Device OS** | Unmodified Debian 13 Trixie (Appliance) or Windows / macOS / Linux (Guest) | Retains upstream Device OS licences. Never copied or mixed into Ring B repositories. Overlay packages `Depend:` on Debian packages without vendoring kernel, glibc, systemd, or Mesa. |\n\
         | **Ring B — User-Agent OS Overlay** | QualiaDB, Webizen OS-Shell, Aether, Etalon, Anvil, Synth, Hypermedia Production, Poet, `webizen-host` | **CC BY-NC-ND 4.0** (protecting the moral and material interests of the author under UDHR Art. 27(2) / ICESCR Art. 15(1)(c); future transition to `mediaprophet/license` applies to Ring B only). Installed under `/opt/webizen` (Appliance) or user-data/install dir (Guest). |\n\
         | **Ring C — `webizen-WWW-engine`** | Independent Servo-derived old-web browser engine | **MPL-2.0**. Executed strictly as a separate sibling worker binary (`www-engine-worker` / `chora-worker`) communicating over IPC and copied RGBA frames (`worker-abi.json`). Never statically linked into Ring B crates. |\n",
    )
}

fn sha3_256_hex(bytes: &[u8]) -> String {
    let mut h = Sha3_256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_stewardship::StewardQuorumVote;
    use crate::native_gpu::probe_native_etalon;
    use crate::qap_host::TrustRoots;

    fn quorum_fixture() -> (UpdateManifest, NaturalHostStewardshipFabric) {
        let artifact_name = "webizen-hostd_0.1.1_amd64.deb";
        let version = "0.1.1";
        let proposal_id = "tx:fixture:package";
        let digest = sha3_256_hex(b"verified package fixture\n");
        let mut fabric = NaturalHostStewardshipFabric::default();
        fabric.primary_host_domain = "fixture.example".into();
        fabric.threshold_m = 2;
        fabric.update_queue = vec![HostUpdateProposal {
            slot_index: 0,
            proposal_id: proposal_id.into(),
            proposal_hash: 0,
            target_host_domain: fabric.primary_host_domain.clone(),
            target_deb_package: artifact_name.into(),
            target_version: version.into(),
            artifact_sha3_256: digest.clone(),
            proposed_by_did: "did:qi:steward:desktop-test".into(),
            proposer_is_instrument_tool: false,
            required_m: 2,
            total_n: 2,
            votes: Vec::new(),
            paraconsistent_review_flagged: false,
            ratified_for_host_apply: false,
            applied_to_host: false,
            cbor_ld_envelope_hex: String::new(),
            sealed_nquin_parity: 0,
        }];
        let approvals = vec![
            QuorumApproval {
                steward_did: "did:qi:steward:desktop-test".into(),
                pubkey_hex: "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a"
                    .into(),
                signature_hex: concat!(
                    "c9478028dbcebca134e794227bc8817e4d0808d448e380623675c3fd4a967a8a19",
                    "97289630fd6bee4e887b42fa2285e6b1ca63001e48a180b38c1af7a5521804",
                )
                .into(),
            },
            QuorumApproval {
                steward_did: "did:qi:steward:mobile-test".into(),
                pubkey_hex: "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c"
                    .into(),
                signature_hex: concat!(
                    "6774c637d0045c65869d1bcdafecc4ece2c336c3511c91187971531bc291a7d61f",
                    "9fb36782aeb6c799fde5674b7fee35aa38f52b6aac160db3b17b9473a6d107",
                )
                .into(),
            },
        ];
        for (steward, approval) in fabric.stewards.iter_mut().zip(&approvals) {
            steward.steward_did = approval.steward_did.clone();
            steward.steward_did_hash = crate::identifiers::q_hash_60(&approval.steward_did);
            steward.dual_pubkey_fingerprint = format!(
                "ed25519:sha3-256:{}",
                sha3_256_hex(&decode_hex::<32>(&approval.pubkey_hex).unwrap()),
            );
        }
        fabric.stewards.truncate(approvals.len());
        let manifest = UpdateManifest {
            version: version.into(),
            sku: "appliance".into(),
            artifact_name: artifact_name.into(),
            sha3_256_hex: digest,
            signature_hex: String::new(),
            pubkey_hex: String::new(),
            quorum: Some(QuorumUpdateProof {
                proposal_id: proposal_id.into(),
                approvals,
            }),
        };
        (manifest, fabric)
    }

    #[test]
    fn requires_threshold_distinct_enrolled_natural_stewards_and_no_dissent() {
        let (manifest, mut fabric) = quorum_fixture();
        let verify = |_: &str, _: &str, _: &[u8]| Ok(());
        assert_eq!(
            verify_quorum_with(&manifest, &fabric, verify).unwrap(),
            manifest.sha3_256_hex
        );

        let mut tampered = manifest.clone();
        let approvals = &mut tampered.quorum.as_mut().unwrap().approvals;
        approvals.pop();
        assert!(verify_quorum_with(&tampered, &fabric, verify)
            .unwrap_err()
            .contains("distinct"));
        let mut tampered = manifest.clone();
        let approvals = &mut tampered.quorum.as_mut().unwrap().approvals;
        approvals[1] = approvals[0].clone();
        assert!(verify_quorum_with(&tampered, &fabric, verify)
            .unwrap_err()
            .contains("repeat"));
        let mut tampered = manifest.clone();
        let approvals = &mut tampered.quorum.as_mut().unwrap().approvals;
        approvals[1].steward_did = "did:qi:agent:mobile-test".into();
        assert!(verify_quorum_with(&tampered, &fabric, verify)
            .unwrap_err()
            .contains("natural-person"));
        let mut tampered = manifest.clone();
        tampered.quorum.as_mut().unwrap().approvals[1]
            .signature_hex
            .clear();
        assert!(verify_quorum_with(&tampered, &fabric, verify).is_err());
        let mut tampered = manifest.clone();
        tampered.sha3_256_hex = "ab".repeat(32);
        assert!(verify_quorum_with(&tampered, &fabric, verify)
            .unwrap_err()
            .contains("queued artifact"));

        fabric.update_queue[0].votes.push(StewardQuorumVote {
            steward_did: fabric.stewards[0].steward_did.clone(),
            apparatus_profile: String::new(),
            approve: false,
            comment: "Review requested".into(),
            dual_signature_hex: String::new(),
            sealed_nquin_parity: 0,
        });
        assert!(verify_quorum_with(&manifest, &fabric, verify)
            .unwrap_err()
            .contains("0x32"));
        fabric.update_queue[0].votes.clear();
        fabric.update_queue[0].paraconsistent_review_flagged = true;
        assert!(verify_quorum_with(&manifest, &fabric, verify).is_err());

        fabric.update_queue[0].paraconsistent_review_flagged = false;
        fabric.stewards[0].dual_pubkey_fingerprint = "ed25519+mldsa65:sample".into();
        assert!(verify_quorum_with(&manifest, &fabric, verify)
            .unwrap_err()
            .contains("pinned"));
    }

    #[test]
    #[ignore = "requires provisioned OpenSSL 3 via WEBIZEN_OPENSSL_BIN or a trusted system installation"]
    fn verifies_real_quorum_signatures_and_rejects_bound_artifact_changes() {
        let (mut manifest, mut fabric) = quorum_fixture();
        assert_eq!(
            verify_quorum_update_manifest(&manifest, &fabric).unwrap(),
            manifest.sha3_256_hex
        );
        fabric.update_queue[0].target_version = "0.1.2".into();
        manifest.version = "0.1.2".into();
        assert!(verify_quorum_update_manifest(&manifest, &fabric).is_err());
    }

    #[test]
    fn digest_matching_alone_never_stages_a_guest_update() {
        let tmp = ScratchDirectory::create(&std::env::temp_dir(), "test-update").unwrap();
        let host = LoopbackHost {
            daemon_base: "http://127.0.0.1:4242".into(),
            port: 4242,
            storage: tmp.path.clone(),
            install_root: tmp.path.clone(),
            trust: TrustRoots::for_storage(&tmp.path),
        };
        let candidate = tmp.path.join("candidate.bin");
        let message = b"fixture, never staged";
        write_new_file(&candidate, message).unwrap();
        let mut manifest = serde_json::json!({
            "version": "0.1.1",
            "sku": "guest",
            "artifact_name": "webizen-os-shell",
            "sha3_256_hex": sha3_256_hex(message),
            "pubkey_hex": FIRST_PARTY_UPDATER_PUBKEY_HEX,
        });
        let path = tmp.path.join("update-manifest.json");
        write_new_file(&path, &serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(!run_updater(&host, Sku::Guest, UpdateAction::Verify, Some(&candidate)).ok);
        assert!(!run_updater(&host, Sku::Guest, UpdateAction::Apply, Some(&candidate)).ok);
        // The untrusted manifest may request quorum, but cannot switch off Guest vendor evidence.
        manifest["quorum"] =
            serde_json::json!({"proposal_id": "tx:fixture:package", "approvals": []});
        fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let apply = run_updater(&host, Sku::Guest, UpdateAction::Apply, Some(&candidate));
        assert!(!apply.ok);
        assert!(apply.detail.contains("Expected 64 bytes"));
        assert!(!tmp.path.join("webizen-os-shell.new").is_file());
        assert!(!tmp.path.join("webizen-os-shell.exe.new").is_file());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn generates_dual_sku_delivery_and_enforces_three_ring_boundary() {
        let tmp = std::env::temp_dir().join(format!("wz-delivery-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let host = LoopbackHost {
            daemon_base: "http://127.0.0.1:4242".into(),
            port: 4242,
            storage: tmp.clone(),
            install_root: tmp.clone(),
            trust: TrustRoots::for_storage(&tmp),
        };
        let gpu = probe_native_etalon(&tmp);
        let receipt =
            generate_dual_sku_delivery_artifacts(&host, &gpu).expect("delivery artifacts");
        assert!(receipt.license_boundary_path.is_file());
        assert!(receipt.guest_manifest_path.is_file());
        assert_eq!(receipt.systemd_units.len(), 4);

        let check = run_updater(&host, Sku::Guest, UpdateAction::Check, None);
        assert!(check.ok);
        let _ = fs::remove_dir_all(&tmp);
    }
}
