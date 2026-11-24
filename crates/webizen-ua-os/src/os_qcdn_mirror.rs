// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Mounted-volume publication for an explicitly authorized public Q-CDN artifact.
//!
//! `WEBIZEN_QCDN_MIRROR_CONFIG` must name a protected absolute JSON file outside
//! session storage. The launcher trusts that operator-supplied policy. Its
//! `publications` array binds each project and registered mirror to an external
//! public artifact, exact byte length, SHA-384 digest, and mounted directory.
//! Each row sets `project_id`, `mirror_id`, `public_disclosure_authorized`,
//! `artifact_path`, `artifact_bytes`, `artifact_sha384_hex`, and `mirror_directory`.
//! Protect the configured path, its parents, and the selected mirror directory
//! from untrusted writers. Publication requires atomic hard links on that volume.

use crate::{validate_human_centric_host_nomenclature, HOST_STEWARDSHIP_STORE_REL_PATH};
use qualia_core_db::{
    crypto::network::{digest::sha384, fill_os},
    net::{
        peer::replication::manifest::{
            disclose_raw_artifact, ByteRange, ContentManifest, MAX_DECODED_BYTES,
        },
        qdnf::types::StrongDigest,
    },
};
use serde::Deserialize;
use std::{
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const MAX_CONFIG_BYTES: u64 = 64 * 1024;
const MAX_LEDGER_BYTES: u64 = 4 * 1024 * 1024;
const LEAF_BYTES: usize = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MirrorConfig {
    publications: Vec<Publication>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Publication {
    project_id: String,
    mirror_id: String,
    public_disclosure_authorized: bool,
    artifact_path: PathBuf,
    artifact_bytes: u32,
    artifact_sha384_hex: String,
    mirror_directory: PathBuf,
}

#[derive(Deserialize)]
struct MirrorLedger {
    #[serde(default)]
    hosted_projects: Vec<ProjectMirrors>,
}

#[derive(Deserialize)]
struct ProjectMirrors {
    project_id: String,
    #[serde(default)]
    qcdn_swarm_mirror_dids: Vec<String>,
}

pub(crate) fn sync_local_qcdn_mirror(
    storage: &Path,
    project_id: &str,
    cancelled: &AtomicBool,
) -> Result<String, String> {
    let config = std::env::var_os("WEBIZEN_QCDN_MIRROR_CONFIG")
        .map(PathBuf::from)
        .ok_or("An operator must provision an authorized local Q-CDN mirror configuration.")?;
    sync_with_config(storage, &config, project_id, cancelled)
}

fn sync_with_config(
    storage: &Path,
    config: &Path,
    project_id: &str,
    cancelled: &AtomicBool,
) -> Result<String, String> {
    require_running(cancelled)?;
    validate_identifier(project_id)?;
    let storage = storage
        .canonicalize()
        .map_err(|error| format!("Cannot locate Q-CDN session storage: {error}"))?;
    let config = external_path(&storage, config, false)?;
    let publication = load_publication(&config, project_id)?;
    require_current_membership(&storage, &publication)?;
    let source = external_path(&storage, &publication.artifact_path, false)?;
    let target = external_path(&storage, &publication.mirror_directory, true)?;
    if source == config || config.starts_with(&target) || source.starts_with(&target) {
        return Err("The public artifact and trust file must be separate from the mirror.".into());
    }

    disclose_raw_artifact(publication.public_disclosure_authorized)
        .map_err(|error| format!("Public Q-CDN disclosure declined: {error}"))?;
    let manifest = checked_manifest(&publication)?;
    let bytes = read_verified(&source, &manifest)?;
    let root = merkle_root(&bytes);
    require_running(cancelled)?;

    let output = target.join(format!("sha384-{}.artifact", hex(&manifest.digest.0)));
    let staged = StagedArtifact::write_to(&target, &bytes)?;
    let readback = read_verified(&staged.0, &manifest)?;
    if merkle_root(&readback) != root {
        return Err("The staged mirror data failed Merkle readback verification.".into());
    }

    // Do not publish a long-running copy after the operator withdraws its policy.
    require_running(cancelled)?;
    require_current_membership(&storage, &publication)?;
    if load_publication(&config, project_id)? != publication
        || external_path(&storage, &publication.mirror_directory, true)? != target
    {
        return Err("The authorized Q-CDN publication changed during synchronization.".into());
    }
    require_running(cancelled)?;

    let action = match fs::hard_link(&staged.0, &output) {
        Ok(()) => "Published",
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let existing = read_verified(&output, &manifest)?;
            if merkle_root(&existing) != root {
                return Err(
                    "The existing mirror artifact failed Merkle readback verification.".into(),
                );
            }
            "Already present"
        }
        Err(error) => {
            return Err(format!(
                "The Q-CDN mount cannot atomically publish this artifact: {error}"
            ));
        }
    };
    Ok(format!(
        "{action} · verified local Q-CDN mirror · {} bytes · SHA-384 Merkle {}",
        manifest.decoded_len,
        hex(&root.0)
    ))
}

fn validate_identifier(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 256
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err("The Q-CDN project or mirror identifier is invalid.".into());
    }
    validate_human_centric_host_nomenclature(value)
}

fn load_publication(config: &Path, project_id: &str) -> Result<Publication, String> {
    let file = open_existing(config)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = file
            .metadata()
            .map_err(|error| error.to_string())?
            .permissions();
        if permissions.mode() & 0o022 != 0 {
            return Err("The Q-CDN trust configuration permits group or public writes.".into());
        }
    }
    let config: MirrorConfig = serde_json::from_slice(&read_bounded(file, MAX_CONFIG_BYTES)?)
        .map_err(|_| "The protected Q-CDN mirror configuration is invalid.".to_string())?;
    let mut matches = config
        .publications
        .into_iter()
        .filter(|entry| entry.project_id == project_id);
    let publication = matches
        .next()
        .ok_or("The operator has not authorized a Q-CDN publication for this project.")?;
    if matches.next().is_some() {
        return Err("The Q-CDN configuration must identify one mounted mirror per project.".into());
    }
    validate_identifier(&publication.mirror_id)?;
    Ok(publication)
}

fn require_current_membership(storage: &Path, publication: &Publication) -> Result<(), String> {
    let path = storage.join(HOST_STEWARDSHIP_STORE_REL_PATH);
    require_plain_path(&path, false)?;
    let ledger: MirrorLedger =
        serde_json::from_slice(&read_bounded(open_existing(&path)?, MAX_LEDGER_BYTES)?)
            .map_err(|_| "The persisted Q-CDN project mirror roster is invalid.".to_string())?;
    let mut matches = ledger
        .hosted_projects
        .iter()
        .filter(|project| project.project_id == publication.project_id);
    if matches.next().is_some_and(|project| {
        project
            .qcdn_swarm_mirror_dids
            .iter()
            .any(|mirror| mirror == &publication.mirror_id)
    }) && matches.next().is_none()
    {
        return Ok(());
    }
    Err("The configured Q-CDN mirror is absent from the project's current roster.".into())
}

fn checked_manifest(publication: &Publication) -> Result<ContentManifest, String> {
    let expected = publication.artifact_sha384_hex.as_bytes();
    let mut digest = [0u8; 48];
    if expected.len() != digest.len() * 2 || !expected.iter().all(u8::is_ascii_hexdigit) {
        return Err("The Q-CDN configuration must pin the artifact's full SHA-384 digest.".into());
    }
    for (out, pair) in digest.iter_mut().zip(expected.chunks_exact(2)) {
        let value = std::str::from_utf8(pair).map_err(|error| error.to_string())?;
        *out = u8::from_str_radix(value, 16).map_err(|error| error.to_string())?;
    }
    let len = publication.artifact_bytes;
    ContentManifest::bind(
        StrongDigest(digest),
        0,
        len,
        &[ByteRange { offset: 0, len }],
    )
    .map_err(|error| {
        format!("Q-CDN content manifest rejected (maximum {MAX_DECODED_BYTES} bytes): {error}")
    })
}

fn require_running(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Acquire) {
        return Err("Q-CDN synchronization cancelled before publication.".into());
    }
    Ok(())
}

fn read_verified(path: &Path, manifest: &ContentManifest) -> Result<Vec<u8>, String> {
    let bytes = read_bounded(open_existing(path)?, u64::from(manifest.decoded_len))?;
    if bytes.len() != manifest.decoded_len as usize || sha384(&bytes) != manifest.digest {
        return Err("The Q-CDN artifact differs from the authorized content manifest.".into());
    }
    Ok(bytes)
}

fn merkle_root(bytes: &[u8]) -> StrongDigest {
    let mut layer: Vec<_> = bytes
        .chunks(LEAF_BYTES)
        .map(|chunk| {
            let mut leaf = [0; 49];
            leaf[1..].copy_from_slice(&sha384(chunk).0);
            sha384(&leaf)
        })
        .collect();
    while layer.len() > 1 {
        layer = layer
            .chunks(2)
            .map(|pair| {
                let [left, right] = pair else {
                    return pair[0];
                };
                let mut branch = [0; 97];
                branch[0] = 1;
                branch[1..49].copy_from_slice(&left.0);
                branch[49..].copy_from_slice(&right.0);
                sha384(&branch)
            })
            .collect();
    }
    // ContentManifest rejects zero-length content before Merkle verification.
    layer[0]
}

fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

fn external_path(storage: &Path, path: &Path, directory: bool) -> Result<PathBuf, String> {
    require_plain_path(path, directory)?;
    let path = path
        .canonicalize()
        .map_err(|error| format!("Cannot resolve the approved Q-CDN path: {error}"))?;
    if path.starts_with(storage) || directory && storage.starts_with(&path) {
        return Err("Provision public Q-CDN paths outside session storage.".into());
    }
    Ok(path)
}

fn require_plain_path(path: &Path, directory: bool) -> Result<(), String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err("Q-CDN paths must be absolute and contain no parent traversal.".into());
    }
    #[cfg(windows)]
    if !matches!(path.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)))
    {
        return Err("Select a locally mounted drive for Q-CDN publication.".into());
    }
    for parent in path.ancestors().skip(1) {
        require_plain_item(parent, true)?;
    }
    require_plain_item(path, directory)
}

fn require_plain_item(path: &Path, directory: bool) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Required Q-CDN path is unavailable: {error}"))?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & REPARSE_POINT != 0 {
            return Err("Q-CDN paths cannot contain reparse points.".into());
        }
    }
    if (directory && metadata.is_dir()) || (!directory && metadata.is_file()) {
        Ok(())
    } else {
        Err("Q-CDN paths must be plain files or directories without symbolic links.".into())
    }
}

fn open_existing(path: &Path) -> Result<File, String> {
    require_plain_item(path, false)?;
    File::open(path).map_err(|error| format!("Cannot open the approved Q-CDN file: {error}"))
}

fn read_bounded(file: File, capacity: u64) -> Result<Vec<u8>, String> {
    let info = file.metadata().map_err(|error| error.to_string())?;
    if !info.is_file() || info.len() > capacity {
        return Err("The Q-CDN input exceeds its authorized size or is not a regular file.".into());
    }
    let mut bytes = Vec::new();
    file.take(capacity + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > capacity {
        return Err("The Q-CDN input exceeds its authorized size.".into());
    }
    Ok(bytes)
}

struct StagedArtifact(PathBuf);

impl StagedArtifact {
    fn write_to(directory: &Path, bytes: &[u8]) -> Result<Self, String> {
        let mut nonce = [0; 16];
        fill_os(&mut nonce).map_err(|_| "Cannot generate a unique Q-CDN staging filename.")?;
        let path = directory.join(format!(".qcdn-{}.pending", hex(&nonce)));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| format!("Cannot stage the authorized Q-CDN artifact: {error}"))?;
        let pending = Self(path);
        let written = file.write_all(bytes).and_then(|()| file.sync_all());
        drop(file);
        written.map_err(|error| format!("Cannot write the authorized Q-CDN artifact: {error}"))?;
        Ok(pending)
    }
}

impl Drop for StagedArtifact {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let mut nonce = [0; 16];
            fill_os(&mut nonce).unwrap();
            let root = std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("webizen-qcdn-test-{}", hex(&nonce)));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn synchronizes_an_authorized_mounted_mirror_without_overwriting() {
        let temp = TestDirectory::new();
        let storage = temp.0.join("storage");
        let source = temp.0.join("public.q42");
        let target = temp.0.join("mirror");
        let config = temp.0.join("mirror.json");
        fs::create_dir_all(storage.join("wellfair")).unwrap();
        fs::create_dir(&target).unwrap();
        fs::write(storage.join(HOST_STEWARDSHIP_STORE_REL_PATH), r#"{"hosted_projects":[{"project_id":"project:test","qcdn_swarm_mirror_dids":["node:test"]}]}"#).unwrap();
        let bytes = b"explicitly published project artifact";
        fs::write(&source, bytes).unwrap();
        let digest = hex(&sha384(bytes).0);
        fs::write(&config, serde_json::to_vec(&serde_json::json!({ "publications": [{
            "project_id": "project:test", "mirror_id": "node:test", "public_disclosure_authorized": true,
            "artifact_path": source, "artifact_bytes": bytes.len(), "artifact_sha384_hex": digest,
            "mirror_directory": target
        }]})).unwrap()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let cancelled = AtomicBool::new(false);
        let attempt = || sync_with_config(&storage, &config, "project:test", &cancelled);
        let receipt = attempt().unwrap();
        assert!(receipt.contains("Published · verified local"));
        let output = target.join(format!("sha384-{digest}.artifact"));
        assert_eq!(fs::read(&output).unwrap(), bytes);
        assert!(attempt().unwrap().starts_with("Already present"));

        fs::write(&source, b"tampered").unwrap();
        assert!(attempt().is_err());
        assert_eq!(fs::read(&output).unwrap(), bytes);
        fs::write(&source, bytes).unwrap();
        fs::write(&output, b"other owner's existing copy").unwrap();
        assert!(attempt().is_err());
        assert_eq!(fs::read(&output).unwrap(), b"other owner's existing copy");
        assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
    }

    #[test]
    fn local_mirror_merkle_root_binds_the_order_and_exact_tail() {
        let mut first = vec![0xA5; LEAF_BYTES];
        first.extend_from_slice(&[0xB6; LEAF_BYTES]);
        let mut swapped = vec![0xB6; LEAF_BYTES];
        swapped.extend_from_slice(&[0xA5; LEAF_BYTES]);
        assert_ne!(merkle_root(&first), merkle_root(&swapped));
        assert_ne!(merkle_root(&first), merkle_root(&first[..first.len() - 1]));
    }
}
