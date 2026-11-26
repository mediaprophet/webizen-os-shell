// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Local workspace preferences and the renderer's latest observed health.

use crate::browser_app::BrowserExecutionMode;
use crate::identifiers::IdentifierFabricPlane;
use crate::semantic_fs::SemanticToolboxId;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const WORKSPACE_PATH: &str = "wellfair/shell_workspace.json";
const MAX_SNAPSHOT_BYTES: usize = 64 * 1024;
const MAX_ADDRESS_BYTES: usize = 4096;
const MAX_HARMONIC_ORDER: u8 = 4;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedWorkspaceSnapshot {
    pub active_stage_slug: String,
    pub active_plane_code: String,
    pub stage_chrome_mode: String,
    pub theme_preset: String,
    pub reduced_motion: bool,
    pub ui_scale_percent: u16,
    pub animate_icons: bool,
    pub inspect_continuity_open: bool,
    pub active_holo_specimen_id: String,
    pub holo_scale_k: u8,
    pub enabled_toolbox_slugs: Vec<String>,
    pub browser_url: String,
    pub browser_mode_code: String,
}

impl Default for PersistedWorkspaceSnapshot {
    fn default() -> Self {
        Self {
            active_stage_slug: "none".into(),
            active_plane_code: IdentifierFabricPlane::NaturalAgent.code().into(),
            stage_chrome_mode: "default-depth".into(),
            theme_preset: "theme-holosonic".into(),
            reduced_motion: false,
            ui_scale_percent: 115,
            animate_icons: true,
            inspect_continuity_open: false,
            active_holo_specimen_id: String::new(),
            holo_scale_k: 2,
            enabled_toolbox_slugs: SemanticToolboxId::ALL
                .into_iter()
                .map(SemanticToolboxId::spec)
                .filter(|toolbox| toolbox.default_enabled)
                .map(|toolbox| toolbox.slug.into())
                .collect(),
            browser_url: "qualia://chora/universe".into(),
            browser_mode_code: BrowserExecutionMode::QualiaNative.code().into(),
        }
    }
}

impl PersistedWorkspaceSnapshot {
    fn validate(&self) -> Result<(), &'static str> {
        if !matches!(
            self.active_stage_slug.as_str(),
            "none"
                | "talk"
                | "mail"
                | "directory"
                | "browser"
                | "keep"
                | "library"
                | "instruments"
                | "poet"
                | "wellfair"
                | "wallet"
                | "projects"
                | "anatomy"
                | "settings"
                | "admin"
        ) {
            return Err("Unknown workspace volume");
        }
        if !IdentifierFabricPlane::ALL
            .into_iter()
            .any(|plane| plane.code() == self.active_plane_code.as_str())
        {
            return Err("Unknown workspace identifier plane");
        }
        if !matches!(
            self.stage_chrome_mode.as_str(),
            "default-depth" | "full-stage" | "float-layer"
        ) {
            return Err("Unknown workspace chrome mode");
        }
        let theme = self.theme_preset.as_str();
        if !(matches!(
            theme,
            "theme-holosonic"
                | "theme-obsidian"
                | "theme-commons"
                | "theme-holoslate"
                | "theme-cyber-grid"
                | "theme-spatial-commons"
        ) || theme.starts_with("custom-theme-"))
        {
            return Err("Unknown workspace theme");
        }
        if !(85..=175).contains(&self.ui_scale_percent) {
            return Err("Invalid workspace UI scale percentage");
        }
        if self.holo_scale_k > MAX_HARMONIC_ORDER || !bounded_address(&self.active_holo_specimen_id)
        {
            return Err("Invalid workspace specimen or harmonic order");
        }
        if self.enabled_toolbox_slugs.len() > SemanticToolboxId::ALL.len()
            || self
                .enabled_toolbox_slugs
                .iter()
                .enumerate()
                .any(|(index, slug)| {
                    !SemanticToolboxId::ALL
                        .into_iter()
                        .any(|toolbox| toolbox.spec().slug == slug.as_str())
                        || self.enabled_toolbox_slugs[..index].contains(slug)
                })
        {
            return Err("Invalid workspace toolbox selection");
        }
        if !BrowserExecutionMode::ALL
            .into_iter()
            .any(|mode| mode.code() == self.browser_mode_code.as_str())
            || !valid_browser_address(&self.browser_url)
        {
            return Err("Invalid workspace browser address or mode");
        }
        Ok(())
    }
}

fn bounded_address(address: &str) -> bool {
    address.len() <= MAX_ADDRESS_BYTES
        && address.trim() == address
        && !address.chars().any(char::is_control)
}

fn valid_browser_address(address: &str) -> bool {
    if !bounded_address(address) {
        return false;
    }
    if address == "about:blank" {
        return true;
    }
    if let Some((scheme, target)) = address.split_once("://") {
        let authority = target.split(['/', '?', '#']).next().unwrap_or_default();
        return ["https", "http", "qualia", "webizen"]
            .iter()
            .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
            && !authority.is_empty()
            && !authority.contains('@')
            && !address.contains('\\')
            && !address.chars().any(char::is_whitespace);
    }

    // Local Qualia assets can use a Windows drive prefix. Other explicit schemes
    // and device or network paths stay out of startup preferences.
    let local_path = match address.split_once(':') {
        Some((drive, path)) if drive.len() == 1 && drive.as_bytes()[0].is_ascii_alphabetic() => {
            path.starts_with(['/', '\\']) && !path.contains(':')
        }
        Some(_) => false,
        None => !address.starts_with("//") && !address.starts_with("\\\\"),
    };
    let normalized = address.to_ascii_lowercase();
    local_path
        && [".10d", ".hmc", ".q42", ".hcf"]
            .iter()
            .any(|&extension| normalized.ends_with(extension))
}

/// Persist UI preferences using a same-directory atomic replacement.
pub fn save_workspace_snapshot(
    storage: &Path,
    snapshot: &PersistedWorkspaceSnapshot,
) -> Result<PathBuf, String> {
    snapshot.validate()?;
    let encoded = serde_json::to_vec_pretty(snapshot).map_err(|error| error.to_string())?;
    if encoded.len() > MAX_SNAPSHOT_BYTES {
        return Err("Workspace snapshot exceeds its size limit".into());
    }
    let path = storage.join(WORKSPACE_PATH);
    let parent = path
        .parent()
        .ok_or("Workspace snapshot has no parent directory")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let (temporary_path, mut file) = create_temporary_snapshot(&path)
        .map_err(|error| format!("Cannot stage workspace snapshot: {error}"))?;
    let written = file.write_all(&encoded).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = written.and_then(|()| fs::rename(&temporary_path, &path)) {
        let _ = fs::remove_file(temporary_path);
        return Err(format!("Cannot save workspace snapshot: {error}"));
    }
    Ok(path)
}

fn create_temporary_snapshot(path: &Path) -> io::Result<(PathBuf, File)> {
    for _ in 0..16 {
        let temporary_path = path.with_extension(format!(
            "json.{}.{}.tmp",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => return Ok((temporary_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "No unique workspace staging filename is available",
    ))
}

/// Read a bounded, validated snapshot. Missing or invalid data leaves startup defaults intact.
pub fn load_workspace_snapshot(storage: &Path) -> Option<PersistedWorkspaceSnapshot> {
    let path = storage.join(WORKSPACE_PATH);
    if !fs::symlink_metadata(&path).ok()?.is_file() {
        return None;
    }
    let file = File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() > MAX_SNAPSHOT_BYTES as u64 {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_SNAPSHOT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > MAX_SNAPSHOT_BYTES {
        return None;
    }
    let snapshot: PersistedWorkspaceSnapshot = serde_json::from_slice(&bytes).ok()?;
    snapshot.validate().ok()?;
    Some(snapshot)
}

/// Observed renderer status. Workspace preferences never serialize this telemetry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellRendererHealthSummary {
    pub backend_label: String,
    pub fallback_policy_label: String,
    pub fallback_engaged: bool,
    pub active_holo_surface_status: String,
    pub viewport_dimensions: (u32, u32),
}

impl Default for ShellRendererHealthSummary {
    fn default() -> Self {
        Self {
            backend_label: "Awaiting native renderer".into(),
            fallback_policy_label: "Awaiting renderer policy".into(),
            fallback_engaged: false,
            active_holo_surface_status: "No HOLO surface observation".into(),
            viewport_dimensions: (0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TemporaryWorkspace(PathBuf);

    impl TemporaryWorkspace {
        fn new() -> Self {
            let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let dir = std::env::temp_dir().join(format!(
                "webizen-workspace-test-{}-{}-{}",
                std::process::id(),
                stamp.as_nanos(),
                TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed),
            ));
            fs::create_dir(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TemporaryWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn persists_and_restores_workspace_snapshot_atomically() {
        let scratch = TemporaryWorkspace::new();
        let storage = &scratch.0;
        assert_eq!(load_workspace_snapshot(storage), None);
        let browser = PersistedWorkspaceSnapshot {
            active_stage_slug: "browser".into(),
            active_plane_code: "SpatiotemporalHandle".into(),
            theme_preset: "theme-obsidian".into(),
            enabled_toolbox_slugs: vec!["assembly-10d-strata".into()],
            browser_mode_code: "hybrid_augmented".into(),
            browser_url: "https://example.test/preview".into(),
            ..Default::default()
        };
        let path = save_workspace_snapshot(storage, &browser).unwrap();
        assert_eq!(path, storage.join(WORKSPACE_PATH));
        assert_eq!(load_workspace_snapshot(storage).as_ref(), Some(&browser));

        let library = PersistedWorkspaceSnapshot {
            active_stage_slug: "library".into(),
            stage_chrome_mode: "float-layer".into(),
            active_holo_specimen_id: "qualia://specimens/test.10d".into(),
            reduced_motion: true,
            inspect_continuity_open: true,
            holo_scale_k: 4,
            enabled_toolbox_slugs: Vec::new(),
            ..browser.clone()
        };
        std::thread::scope(|scope| {
            let first = scope.spawn(|| save_workspace_snapshot(storage, &browser));
            let second = scope.spawn(|| save_workspace_snapshot(storage, &library));
            first.join().unwrap().unwrap();
            second.join().unwrap().unwrap();
        });
        let loaded = load_workspace_snapshot(storage).unwrap();
        assert!(loaded == browser || loaded == library);
        assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);

        let invalid = PersistedWorkspaceSnapshot {
            browser_url: "https://account:password@example.test/preview".into(),
            ..library
        };
        assert!(save_workspace_snapshot(storage, &invalid).is_err());
        assert_eq!(load_workspace_snapshot(storage), Some(loaded));
        fs::write(path, vec![b' '; MAX_SNAPSHOT_BYTES + 1]).unwrap();
        assert_eq!(load_workspace_snapshot(storage), None);
    }

    #[test]
    fn loads_older_snapshots_with_omitted_preferences() {
        let scratch = TemporaryWorkspace::new();
        let path = scratch.0.join(WORKSPACE_PATH);
        fs::create_dir(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            br#"{"reduced_motion":true,"active_stage_slug":"settings"}"#,
        )
        .unwrap();
        let restored = load_workspace_snapshot(&scratch.0).unwrap();
        assert_eq!(restored.active_stage_slug, "settings");
        assert!(restored.reduced_motion);
        assert_eq!(
            restored,
            PersistedWorkspaceSnapshot {
                active_stage_slug: "settings".into(),
                reduced_motion: true,
                ..Default::default()
            }
        );
    }
}
