# Codex Implementation Plan: Webizen OS-Shell, Mobile AVF, Desktop Appliance & `webizen-hostd` Reconstruction (`WP-OS1`–`WP-OS6`)
**Date:** 2026-11-22  
**Copyright (c) 2026 Timothy Charles Holborn — CC BY-NC-ND 4.0**  
**Target Workspace:** `C:\github\webizenai\webizen-os-shell` (consuming `C:\github\Aether` `WP1`–`WP6`)  
**Execution Role:** External Coding Agent (**Codex**) — **SOURCE-ONLY IMPLEMENTATION (NO CARGO COMMANDS)**  
**Verification & Build Owner:** **Antigravity Coordinator** (runs all `cargo test` and `cargo build` passes after Codex reports completion)

---

## 0. Mandatory Rules & Constraints for Codex

1. **STRICT NO-CARGO RULE**:
   - **Do NOT run `cargo check`, `cargo test`, `cargo build`, `cargo clippy`, `cargo update`, or `cargo metadata`.**
   - Your task is strictly to author complete, type-correct Rust source files, unit test modules (`#[cfg(test)]`), and the specified `Cargo.toml` entries across `WP-OS1` through `WP-OS6`.
   - When all 6 Work-Packages are saved to disk, stop and report the list of created/modified files so the Coordinator can review, compile, and test.

2. **LICENSING & HUMAN-RIGHTS VOCABULARY INVARIANTS**:
   - Start all new files with:
     ```rust
     // Copyright (c) 2026 Timothy Charles Holborn.
     // SPDX-License-Identifier: CC-BY-NC-ND-4.0
     ```
   - **Never** use the words `"sovereign"`, `"sovereignty"`, or `"self-sovereign"` anywhere in code, comments, identifiers, test strings, `.deb` control descriptions, or UI labels (`validate_human_centric_host_nomenclature` enforces this fail-closed!). Always use human-rights agreement vocabulary (**Human-Centric**, **Human Rights**, **Human Dignity**, **Human Agency**, **Natural Person**, **Sanctuary**, **Guardianship**, **Permissive Commons**).
   - Always use **"8-Sector Command Orb"** / **"Command Orb (`◉`)"** (never "Command Wheel" in user-facing labels).

3. **LOCKFILE & OFFLINE DEPENDENCY INVARIANTS**:
   - Do **NOT** add any external crates that are absent from `C:\github\webizenai\webizen-os-shell\Cargo.lock` (do **not** add `tar`, `ar`, `sha2`, `md5`, `md-5`, or `clap`).
   - `flate2 = "1.0"` (`flate2::GzBuilder`, `Compression`) and `sha3 = "0.10"` (`sha3::Sha3_256`, `Digest`) are **already** in `crates/webizen-ua-os/Cargo.toml` and `Cargo.lock`.
   - `anyrender = "0.14.0"`, `peniko = "0.6.1"`, `kurbo = "0.13.1"`, `blitz-dom = { version = "=0.3.0-beta.4", default-features = false }`, and `blitz-traits = "=0.3.0-beta.4"` are **already** in `Cargo.lock` and can be added to root `C:\github\webizenai\webizen-os-shell\Cargo.toml`.
   - `refract` (`../../Aether/refract`) is a zero-dependency local path crate; enabling `"refract"` on `aether` in root `Cargo.toml` requires zero external crates.

4. **AETHER `rsx!` MACRO IDENTIFIER RULE**:
   - Inside `rsx! { ... }`, **never** name a local variable or format interpolation `{fragment}` (it collides with Aether's built-in `Fragment` element). Use `fragment_sel` or `ann_frag` instead.

---

## Work-Package Overview & File Ownership

| Work-Package | Title | Owned Files |
| --- | --- | --- |
| **WP-OS1** | Pure-Rust Deterministic `.deb` Archive Builder (`deb_builder.rs`) & Mobile AVF / Desktop Appliance Runtime Profiles (`mobile_avf.rs`, `desktop_appliance.rs`) | `crates/webizen-ua-os/src/deb_builder.rs` (NEW), `crates/webizen-ua-os/src/mobile_avf.rs` (NEW), `crates/webizen-ua-os/src/desktop_appliance.rs` (NEW), `crates/webizen-ua-os/src/host_stewardship.rs`, `crates/webizen-ua-os/src/lib.rs` |
| **WP-OS2** | Standalone `webizen-hostd` Host Extension Daemon Binary & Runtime Engine | `crates/webizen-ua-os/src/host_daemon_runtime.rs` (NEW), `src/bin/webizen-hostd.rs` (NEW), `Cargo.toml`, `src/main.rs` |
| **WP-OS3** | `ShellState` + `WebizenRefractModel` (`RefractStore` `QuinWindow<64>`) + Multi-Window Native Overlays (`auxiliary_windows.rs`) | `src/state/shell_state.rs`, `src/components/auxiliary_windows.rs` (NEW), `src/app.rs`, `src/tray.rs` |
| **WP-OS4** | Live `EtalonSurfaceWidget` Holographic 10D v2 (`HOLO` `0x0306`) Viewport & Interactive `ChoraFrameWidget` Browser Surface (`viewports.rs`) | `src/components/viewports.rs` (NEW) |
| **WP-OS5** | Modularizing the 4,522-Line `src/components/stage_host.rs` into Per-Volume Modules (`src/components/volumes/*`) | `src/components/mod.rs`, `src/components/stage_host.rs`, `src/components/volumes/{mod.rs, communications.rs, browser.rs, llm_agents.rs, instruments.rs, projects.rs, admin.rs, poet_synth.rs, settings.rs, wallet.rs, library.rs, sanctuary.rs, wellfair.rs}` (NEW) |
| **WP-OS6** | Wiring `aether::refract_bridge` Semantic UI RSX Cards & the 15 New Native DOM Events Across Volumes + Unit Tests | Integrated across `src/components/volumes/*`, `src/app.rs`, and `crates/webizen-ua-os/src/lib.rs` tests |

---

## WP-OS1: Pure-Rust Deterministic `.deb` Archive Builder & Mobile AVF / Desktop Appliance Profiles

### Files Owned
- `crates/webizen-ua-os/src/deb_builder.rs` (NEW)
- `crates/webizen-ua-os/src/mobile_avf.rs` (NEW)
- `crates/webizen-ua-os/src/desktop_appliance.rs` (NEW)
- `crates/webizen-ua-os/src/host_stewardship.rs`
- `crates/webizen-ua-os/src/lib.rs`

### 1.1 Pure-Rust Deterministic Debian 2.0 `.deb` Builder (`crates/webizen-ua-os/src/deb_builder.rs`)
- Build real, spec-compliant Debian 2.0 `.deb` binary packages directly from the staged package directories produced by `generate_tri_target_debian_suite` (`desktop_deb_tree`, `mobile_avf_deb_tree`, `host_deb_tree`) using **only** `std`, `flate2::{Compression, GzBuilder}`, and `sha3::{Digest, Sha3_256}`:
  1. **Global `ar` magic (8 bytes)**: `b"!<arch>\n"`.
  2. **Three `ar` members in strict order**:
     - Member 1: `debian-binary` -> payload `b"2.0\n"` (4 bytes).
     - Member 2: `control.tar.gz` -> deterministic gzipped POSIX `ustar` archive of all files inside `<staging_root>/DEBIAN/` stored at `./control`, `./postinst`, `./prerm`, `./conffiles` (do **not** emit a `./DEBIAN` directory entry in `control.tar.gz`).
     - Member 3: `data.tar.gz` -> deterministic gzipped POSIX `ustar` archive of all directories and files outside `DEBIAN/`, prefixed with `./` (e.g., `./opt/webizen/bin/...`, `./etc/webizen-host/...`, `./lib/systemd/system/...`, `./usr/local/bin/...`).
  3. **60-byte `ar` Member Header Format**:
     - `name[0..16]`: ASCII space-padded (`"debian-binary   "`, `"control.tar.gz  "`, `"data.tar.gz     "`).
     - `mtime[16..28]`: `"0           "` (deterministic epoch 0).
     - `uid[28..34]`: `"0     "`.
     - `gid[34..40]`: `"0     "`.
     - `mode[40..48]`: `"100644  "`.
     - `size[48..58]`: left-aligned decimal byte length, space-padded to 10 bytes.
     - `fmag[58..60]`: ``b"`\n"``.
     - If member payload byte length is odd (`len % 2 == 1`), append a single `b'\n'` padding byte before the next member header.
  4. **512-byte POSIX `ustar` Header Format**:
     - Sort entries lexicographically for determinism.
     - `name[0..100]` (and `prefix[345..500]` if path > 100 bytes, split on `/`).
     - `mode[100..108]`: `"0000755\0"` for directories (`typeflag = b'5'`), `postinst`, `prerm`, `*.sh`, `/opt/webizen/bin/*`, `/usr/local/bin/*`, or files starting with `#!`; `"0000644\0"` for regular config/metadata files (`typeflag = b'0'`).
     - `uid[108..116]` & `gid[116..124]`: `"0000000\0"`.
     - `size[124..136]`: `format!("{:011o}\0", file_len)` (`0` for directories).
     - `mtime[136..148]`: `"00000000000\0"`.
     - `chksum[148..156]`: initialize with 8 spaces `b' '`, sum all 512 header bytes as `u32`, write `format!("{:06o}\0 ", sum)`.
     - `typeflag[156]`: `b'5'` for directory (ensure trailing `/` on directory name), `b'0'` for regular file.
     - `magic[257..263]`: `b"ustar\0"`.
     - `version[263..265]`: `b"00"`.
     - `uname[265..297]` & `gname[297..329]`: `"root\0"` padded with zeros.
     - Pad each file's data to a 512-byte multiple with zero bytes; finish the tar archive with `1024` zero bytes (`2 × 512` EOF blocks).
  5. **Deterministic Gzip**:
     - `GzBuilder::new().mtime(0).operating_system(3).write(Vec::new(), Compression::default())`.
- **Public Types & Functions in `deb_builder.rs`**:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
  pub struct DebBuildReceipt {
      pub package_name: String,
      pub version: String,
      pub architecture: String,
      pub output_deb_path: PathBuf,
      pub archive_bytes: u64,
      pub control_tar_gz_bytes: u64,
      pub data_tar_gz_bytes: u64,
      pub member_file_count: usize,
      pub sha3_256_hex: String,
  }

  pub fn build_deb_from_tree(
      staging_root: &Path,
      output_deb_path: &Path,
  ) -> Result<DebBuildReceipt, String>;

  pub fn build_tri_target_debs_from_receipt(
      suite: &TriTargetDebianSuiteReceipt,
  ) -> Result<Vec<DebBuildReceipt>, String>;

  pub fn verify_deb_ar_container(bytes: &[u8]) -> Result<Vec<(String, usize)>, String>;
  ```
- Also add `pub built_deb_packages: Vec<DebBuildReceipt>` to `TriTargetDebianSuiteReceipt` in `host_stewardship.rs` and invoke `build_deb_from_tree` inside `generate_tri_target_debian_suite` so `package-tri-target-deb` produces the 3 real `.deb` files (`webizen-desktop-appliance_0.1.0_amd64.deb`, `webizen-mobile-avf_0.1.0_arm64.deb`, `webizen-hostd_0.1.0_amd64.deb`) on disk automatically!

### 1.2 Mobile AVF Sanctuary & Pocket Co-Signer Profile (`crates/webizen-ua-os/src/mobile_avf.rs`)
- Implement:
  - `AvfIsolationState { ProtectedPkvmEl2Active, GuestDevSimulation, UnsupportedHost }`
  - `VsockTierIsolation { Tier1El2SanctuaryLocal, Tier2BlindedPublicNpuRpc }`
  - `VsockChannelDescriptor { cid: u32, port: u32, tier: VsockTierIsolation, transport_label: String, max_sensitivity_allowed: u8, zero_copy_ahb_gfxstream: bool }`
  - `AvfSanctuaryProbe { isolation_state: AvfIsolationState, pkvm_el2_detected: bool, pvmfw_dice_present: bool, anvil_np64_attested: bool, virtio_gpu_venus_active: bool, volatile_scrub_on_sleep: bool, vsock_channels: Vec<VsockChannelDescriptor>, summary: String }`
    - **Hardware Attestation Gate**: `anvil_np64_attested` must be `true` **only** when `gpu.may_advertise_hardware_attested_vault` and real EL2 markers are present (never falsified on dev hosts).
  - `ThermalPressureLevel { Nominal, Fair, Serious, Critical }`
  - `P64ThermalBatteryGovernor { thermal_level: ThermalPressureLevel, battery_percent: u8, on_external_power: bool, active_p64_kv_cache_mib: u32, max_kv_cache_cap_mib: u32, recommended_residency: ManagedModelResidency, allow_new_streaming_vram: bool, scrub_evicted_on_critical: bool }`
    - `pub fn evaluate_governor(thermal: ThermalPressureLevel, battery_percent: u8, on_external_power: bool) -> Self`
  - `ThumbArcOrbSector { index: u8, label: &'static str, glyph: &'static str, angle_deg_tenths: i16 }` & `ThumbArcOrbGeometry { right_handed: bool, arc_start_deg: i16, arc_sweep_deg: i16, radius_dp: u16, target_hit_dp: u16, sectors: [ThumbArcOrbSector; 8] }` (`pub fn compute_thumb_arc(right_handed: bool, viewport_width_dp: u32) -> Self` for the 8-Sector Command Orb `◉`).
  - `MobileOneTapCheckInReceipt { circle_id: String, alert_id: String, principal_did: String, escalation_reversed: bool, duty_of_inquiry_status: String, sealed_nquin_parity: u64 }`
  - `MobileAvfSanctuaryProfile { probe: AvfSanctuaryProbe, governor: P64ThermalBatteryGovernor, orb_geometry: ThumbArcOrbGeometry, last_ok_checkin: Option<MobileOneTapCheckInReceipt> }`
  - `pub fn probe_mobile_avf_sanctuary(gpu: &NativeEtalonStatus) -> MobileAvfSanctuaryProfile`
  - `pub fn trigger_one_tap_principal_ok_checkin(fabric: &mut NaturalHostStewardshipFabric, storage: &Path, alert_id: &str, principal_did: &str, note: Option<&str>) -> Result<MobileOneTapCheckInReceipt, String>`

### 1.3 Desktop Appliance & Multi-Locus Fleet Profile (`crates/webizen-ua-os/src/desktop_appliance.rs`)
- Implement:
  - `DesktopWorkspacePreset { SpatialOrbit, AuthoringPoetSynth, DualWebBrowser, CarerStewardConsole, QuietSanctuary }` (`ALL`, `.code()`, `.title()`, `.default_stage_slug()`)
  - `LocusFleetNodeSummary`, `MultiLocusFleetControllerState`, `TrayMenuBindingSpec`, `DesktopApplianceProfile`
  - `pub fn build_desktop_appliance_profile(sku: Sku, env: &OsEnvironment, fabric: &NaturalHostStewardshipFabric) -> DesktopApplianceProfile`
  - `pub fn sync_fleet_from_fabric(profile: &mut DesktopApplianceProfile, fabric: &NaturalHostStewardshipFabric)`
- Also add `pub fn register_hosted_project(...) -> Result<HostedProjectWorkload, String>` to `NaturalHostStewardshipFabric` in `crates/webizen-ua-os/src/host_stewardship.rs` (validating nomenclature, requiring `incorporated_entity_id` to exist when stage is `IncorporatedCommercialTenantProject`, sealing `NQuin48` parity, and saving to disk).

---

## WP-OS2: Standalone `webizen-hostd` Host Extension Daemon Binary & Runtime Engine

### Files Owned
- `crates/webizen-ua-os/src/host_daemon_runtime.rs` (NEW)
- `src/bin/webizen-hostd.rs` (NEW)
- `Cargo.toml`
- `src/main.rs`

### 2.1 Root `Cargo.toml` Binary & Dependency Wiring
- In `C:\github\webizenai\webizen-os-shell\Cargo.toml`:
  - Add `"refract"` to `aether` features:
    ```toml
    aether = { path = "../../Aether/packages/aether", default-features = false, features = ["macro", "html", "signals", "hooks", "launch", "native", "refract"] }
    ```
  - Add optional features:
    ```toml
    [features]
    default = ["native"]
    native = []
    chora = ["aether-native/chora"]
    gpu-cpu-fallback = ["aether-native/gpu-cpu-fallback"]
    ```
  - Add already-locked crates to `[dependencies]`:
    ```toml
    anyrender = "0.14.0"
    peniko = "0.6.1"
    kurbo = "0.13.1"
    blitz-dom = { version = "=0.3.0-beta.4", default-features = false }
    blitz-traits = "=0.3.0-beta.4"
    ```
  - Explicitly declare both binary targets:
    ```toml
    [[bin]]
    name = "webizen-os-shell"
    path = "src/main.rs"

    [[bin]]
    name = "webizen-hostd"
    path = "src/bin/webizen-hostd.rs"
    ```

### 2.2 Headless Host Extension Runtime (`crates/webizen-ua-os/src/host_daemon_runtime.rs` & `src/bin/webizen-hostd.rs`)
- Implement `HostDaemonConfig`, `HostDaemonStatusSnapshot`, `HostDaemonTickReport`, `HostDaemonCommand`, `parse_hostd_args`, `execute_hostd_command`, `run_daemon_tick`, `handle_peer_p64_mcp_json_rpc_line`, `serve_peer_p64_mcp_tcp`, and `serve_hostd_http_control`.
- Support CLI subcommands on `webizen-hostd` (and parse `--storage <dir>`, `--bind <addr>`, `--mcp-bind <addr>`, `--auth-mode <mode>`, `--json`):
  - `status`
  - `run-tick`
  - `propose-update <deb_package> <version> <proposer_did> [--instrument]`
  - `vote-quorum <proposal_id> <steward_did> <approve|dissent> <comment> [--guardian <did>]`
  - `register-project <project_id> <title> <domain> <personal|group|mutual-aid|commercial> <steward_did> [--entity <inc_id>]`
  - `transition-qcdn <project_id> <personal|group|mutual-aid|commercial> <threshold_m> [--add-steward <did>] [--add-mirror <node_id>]`
  - `audit-jural-license <natural|agent-of-entity|incorporated> [--entity <inc_id>]`
  - `poll-care-heartbeats [--circle <id>] [--dispatch-lapse] [--ack <alert_id> <responder_did> <principal-ok|carer-ack> <note>]`
  - `serve-peer-p64-mcp [--once]` (JSON-RPC 2.0 `initialize`, `tools/list`, `tools/call` for `"llm_infer"`, strictly capped at `SENSITIVITY_PUBLIC_ONLY = 0` so `Sanctuary100` data is never exfiltrated over peer MCP).
- **Important**: `src/bin/webizen-hostd.rs` must only import `webizen_ua_os::*`, `serde_json`, and `std` (zero GUI/windowing dependencies).

---

## WP-OS3: `ShellState` + `WebizenRefractModel` (`RefractStore` `QuinWindow<64>`) + Multi-Window Native Overlays (`auxiliary_windows.rs`)

### Files Owned
- `src/state/shell_state.rs`
- `src/components/auxiliary_windows.rs` (NEW)
- `src/app.rs`
- `src/tray.rs`

### 3.1 `WebizenRefractModel` (`RefractStore` + `QuinWindow<64>`) & `ShellState` Extensions (`src/state/shell_state.rs`)
- Define `WebizenRefractModel` in `src/state/shell_state.rs` (exported via `src/state/mod.rs`):
  ```rust
  #[derive(Clone, Debug)]
  pub struct WebizenRefractModel {
      pub quin_window: aether::refract::QuinWindow<64>,
      pub instrument_readiness: [aether::refract::InstrumentReadiness; 6],
      pub wellfair_review: aether::refract::FairnessReviewView,
      pub assertion: aether::refract::ContextualAssertionView,
      pub assertion_evidence: Option<aether::refract::AssertionEvidenceLevel>,
      pub contribution: aether::refract::ContributionStageView,
  }
  ```
- Implement `WebizenRefractModel::from_session(sess: &DeviceSession) -> Self` and `WebizenRefractModel::placeholder() -> Self`:
  - Seed `quin_window` slots `0..4` with the 4 Identifier Fabric planes, slots `4..10` with active Host Stewardship / Project / Care Circle `NQuin48` records, and slots `10..16` with the 6 Core Semantic Instruments (`SI-01`..`SI-06`), calling `slot.recompute_parity()` on every seeded slot.
  - Provide helper `pub fn update_refract_quin_word(store: &aether::refract_bridge::RefractStore<WebizenRefractModel>, slot_idx: usize, field_idx: u8, value: u64)` that writes the requested word (`0..5`) and automatically updates field `5` (`parity`) so 6-column `NQuin` parity remains valid.
  - Remember: `InstrumentReadiness::Held { why }`, `FairnessReviewView::remedy_notes`, and `ContextualAssertionView::summary` require `&'static str`. Use a `const fn` / static lookup table for those `&'static str` fields.
- Extend `ShellState` with:
  - `pub mobile_avf: MobileAvfSanctuaryProfile`
  - `pub desktop_appliance: DesktopApplianceProfile`
  - `pub host_daemon_snapshot: HostDaemonStatusSnapshot`
  - `pub aux_orb_window_open: bool`, `pub aux_alert_modal_open: bool`, `pub aux_palette_window_open: bool`
  - `pub viewport_widget_id: usize` (starts at `1`)
  - `pub active_holo_specimen_id: String`, `pub holo_wireframe_animating: bool`, `pub holo_scale_k: u8`
  - `pub shell_viewport_size: (f64, f64)`
  - `pub beforeinput_guard_notice: String`, `pub clipboard_toast: String`, `pub selection_range_label: String`

### 3.2 Multi-Window Native Overlays (`src/components/auxiliary_windows.rs`, `src/app.rs`, `src/tray.rs`)
- Because each auxiliary window opened via `aether_native::NativeAppHandle::open_window` runs its own `VirtualDom` and `open_window` returns `()`, implement a thread-safe `#[derive(Clone, Default)] pub struct AuxWindowBridge(Arc<Mutex<AuxBridgeInner>>)` in `src/components/auxiliary_windows.rs`:
  - Holds `AuxSnapshot` (synced from `ShellState`), `pending_commands: VecDeque<AuxShellCommand>`, `main_window: Option<Arc<dyn aether_native::winit::window::Window>>`, and `AuxWindowSlot { id: WindowId, handle: Arc<dyn aether_native::winit::window::Window> }` for each of the 3 auxiliary window roles:
    1. **`WindowRole::OrbOverlay` (`open_or_toggle_orb_overlay`)**:
       - Uses `WindowRole::OrbOverlay`, `WindowClosePolicy::HideOnClose`, `RendererFallbackPolicy::FailClosed` (because software fallback does not support transparent alpha), size `420x340`.
       - Renders the floating 8-Sector Command Orb (`◉`) (`Close`, `Soften`, `Full`, `Float`, `Keep`, `Share`, `Inspect`, `More`) + stage quick-jump buttons, dispatching `AuxShellCommand` back to the primary window.
    2. **`WindowRole::UrgentAlertModal` (`open_urgent_care_alert_modal`)**:
       - Uses `WindowRole::UrgentAlertModal`, `WindowClosePolicy::CloseAndDrop`, `RendererFallbackPolicy::CpuOnFailure`, size `520x400`.
       - Displays active `SharedGuardianshipAndCareCircle` alerts (`CareAlertSeverity::UrgentActionRequired` / `CriticalEmergency`), with one-click **`"I am OK (Principal Reversible Check-In)"`** (`AuxShellCommand::AcknowledgeCareAlert { alert_id, principal_ok: true }`) and **`"Remote Carer Duty-of-Inquiry Verify"`** (`principal_ok: false`).
    3. **`WindowRole::AuxiliaryToolPalette` (`open_or_toggle_tool_palette`)**:
       - Uses `WindowRole::AuxiliaryToolPalette`, `WindowClosePolicy::HideOnClose`, `RendererFallbackPolicy::CpuOnFailure`, size `380x560`.
       - Displays the floating **Opt-In Semantic ToolChest** toggles and **`@agent` (`.p64` / MCP) Quick-Invoke** bar.
- In `src/app.rs`:
  - Initialize `let refract_store = use_refract_store(|| WebizenRefractModel::from_session(...));` and `let aux_bridge = use_context_provider(AuxWindowBridge::new);`.
  - In `use_window_event`, drain `aux_bridge.drain_commands()` alongside `crate::tray::take_stage()`, applying commands to `state.write()` and syncing `aux_bridge.sync_from_state(&state.read())`.
  - Attach `onresize` on the top-level `<div class: "{root_class}">` in `App` (since Blitz dispatches viewport `Resize` events to top-level root elements under `ElementId(0)`) to update `state.write().shell_viewport_size`.

---

## WP-OS4: Live `EtalonSurfaceWidget` Holographic 10D v2 (`HOLO` `0x0306`) Viewport & `ChoraFrameWidget` Browser Surface

### Files Owned
- `src/components/viewports.rs` (NEW)

### 4.1 `WebizenHolo10dSurfaceDelegate` & `Holo10dViewport` Component (`src/components/viewports.rs`)
- Import `aether_native::viewport::{EtalonFrameContext, EtalonSurfaceDelegate, EtalonSurfaceError, EtalonSurfaceWidget, EtalonViewportState, RenderOutcome}`, `aether_native::CustomWidgetAttr`, `anyrender::{PaintScene, Scene}`, `peniko::{Color, Fill}`, `kurbo::{Affine, Circle, Line, Point, Rect, Stroke}`, `blitz_dom::IntrinsicSizes`, and `blitz_traits::events::UiEvent`.
- Implement `pub struct WebizenHolo10dSurfaceDelegate` (`impl EtalonSurfaceDelegate for WebizenHolo10dSurfaceDelegate`):
  - Shared `control: Arc<Mutex<HoloViewportControl>>` (`specimen_id`, `harmonic_k: u8`, `animating: bool`, `yaw: f32`, `pitch: f32`, `zoom: f32`, `raymarch_steps: u32` capped at `<= 32`).
  - **GPU Path (`prepare` & `render_frame`)**:
    - Target texture format is `etalon::TextureFormat::Rgba8Unorm`.
    - **Do NOT use `Features::IMMEDIATES` / `set_immediates`** (not enabled by `vello-hybrid`). Use a 32-byte `etalon::Buffer` (`BufferUsages::UNIFORM | BufferUsages::COPY_DST`) + `@group(0) @binding(0) var<uniform>` bind group and a WGSL shader rendering the Holographic 10D v2 (`HOLO` `0x0306`) phase-space wavefield interference rings ($k=0..4$) and 12-triangle proxy lens bounding box.
    - Return `RenderOutcome::ContinuousRedraw` only when `control.animating` is `true`, and `RenderOutcome::Static` otherwise.
  - **CPU Fallback Path (`paint_fallback(&mut self, width: u32, height: u32, scale: f64) -> Scene`)**:
    - Project the 8 corners of the 3D `HOLO` proxy bounding box through `(yaw, pitch, zoom)` onto the 2D `anyrender::Scene`, stroke the 12 bounding-box edges and 12 proxy-lens triangle diagonals in cyan (`Color::from_rgba8(47, 224, 200, 220)`), and draw `1..=(harmonic_k + 1)` concentric Eulerian wavefield circles in violet (`Color::from_rgba8(139, 124, 255, 160)`).
  - **Interactive Input (`handle_event`)**:
    - Pointer drag rotates `(yaw, pitch)`; wheel adjusts `zoom`; keyboard `Space` toggles animation and `R` resets camera orientation.
- Implement `#[component] pub fn Holo10dViewport(mut state: Signal<ShellState>, specimen_label: String) -> Element`:
  - Cache `(stable_id, CustomWidgetAttr, Arc<Mutex<HoloViewportControl>>)` inside `use_hook` so `CustomWidgetAttr::with_stable_id(stable_id, EtalonSurfaceWidget::new(delegate))` is only constructed once per `stable_id`.
  - Render `<object class: "holo-viewport-object", data: cached_attr>` plus controls for scale-space harmonic order $k \in \{0, 1, 2, 3, 4\}$, raymarch step budget ($\le 32$), camera reset, and live `RendererHealth` (`use_hook` / `try_consume_context::<aether_native::AetherNativeWindowRenderer>()`).

### 4.2 `BrowserChoraSurface` Component (`src/components/viewports.rs`)
- Implement `#[component] pub fn BrowserChoraSurface(mut state: Signal<ShellState>) -> Element`:
  - In `BrowserExecutionMode::QualiaNative` (Mode 2): embeds `Holo10dViewport` for `qualia://` / `webizen://` native 10D v2 resources.
  - In `BrowserExecutionMode::TraditionalWeb` (Mode 1) and `BrowserExecutionMode::HybridAugmented` (Mode 3):
    - When `#[cfg(feature = "chora")]` is active and the configured `worker_binary_slot` path is absolute and exists on disk, mounts `aether_native::chora_frame::ChoraFrameWidget` via `<object data: CustomWidgetAttr::with_stable_id(...)>` (which now forwards route-stamped pointer, wheel, keyboard, IME, and focus events via `WP4`).
    - When the external `chora-worker` binary is not running, renders the live Mode 1 / Mode 3 Sibling Worker Isolation & Copied-Frame Contract viewport card with interactive Mode 3 `evaluate_hybrid_page_request` channel testers (`qualia-10d`, `solid-ldp`, `commons-receipt`, `nquin-provenance`, `vibescript-eval`).

---

## WP-OS5 & WP-OS6: Modularizing `stage_host.rs` into `src/components/volumes/*` + Wiring `refract_bridge` & New Native DOM Events

### Files Owned
- `src/components/mod.rs`
- `src/components/stage_host.rs`
- `src/components/volumes/mod.rs` (NEW)
- `src/components/volumes/communications.rs` (NEW — `TalkVolume`, `MailVolume`, `DirectoryVolume`)
- `src/components/volumes/browser.rs` (NEW — `BrowserVolume`)
- `src/components/volumes/llm_agents.rs` (NEW — `LlmAndAgentManagementPanel`)
- `src/components/volumes/instruments.rs` (NEW — `InstrumentsVolume`)
- `src/components/volumes/projects.rs` (NEW — `ProjectsVolume`)
- `src/components/volumes/admin.rs` (NEW — `HostStewardshipAndFleetPanel`, `AdminVolume`)
- `src/components/volumes/poet_synth.rs` (NEW — `PoetSynthVolume`, `AnatomyVolume`)
- `src/components/volumes/settings.rs` (NEW — `SettingsVolume`, `SettingsSectionBody`)
- `src/components/volumes/wallet.rs` (NEW — `WalletVolume`)
- `src/components/volumes/library.rs` (NEW — `LibraryVolume`)
- `src/components/volumes/sanctuary.rs` (NEW — `SanctuaryVolume`)
- `src/components/volumes/wellfair.rs` (NEW — `GuardianshipAndCarePanel`, `WellfairVolume`)

### 5.1 Preserve 100% of Existing Volume Functionality While Extracting
- Move each volume function from the 4,522-line `src/components/stage_host.rs` into its dedicated module under `src/components/volumes/`, keeping `src/components/stage_host.rs` focused on `StageHost`, `CommandOrbBar` (adding buttons to pop out the floating `OrbOverlay`, `AuxiliaryToolPalette`, and `UrgentAlertModal` windows via `AuxWindowBridge`), `ContinuityInspectorBanner`, `EmptyManifold`, `ManifoldCard`, and `VolumeBody`.
- Extract `#[component] pub fn HostStewardshipAndFleetPanel(mut state: Signal<ShellState>) -> Element` in `src/components/volumes/admin.rs` and `#[component] pub fn GuardianshipAndCarePanel(mut state: Signal<ShellState>) -> Element` in `src/components/volumes/wellfair.rs`.

### 5.2 Wire `aether::refract_bridge` Semantic UI RSX Components & 6-Column `NQuin` Lens Inspector
- In **`InstrumentsVolume`** (`src/components/volumes/instruments.rs`):
  - Render `render_instrument_readiness_card` for `SI-01`..`SI-06` from `RefractStore<WebizenRefractModel>` (`collect != activate`, `data-inspect-before-fetch="true"`, soft `Held` copy) and `render_contextual_assertion_badge_with_evidence`, plus an interactive 6-Column `NQuin` `QuinWindow<64>` slot/word inspector (`read_quin_slot`, `update_refract_quin_word`).
- In **`WellfairVolume`** (`src/components/volumes/wellfair.rs`):
  - Render `render_fairness_review_panel` across all 6 `WellFairDimension::ALL` (`data-collapsed-score="forbidden"`) with interactive per-dimension outcome toggles updating `RefractStore<WebizenRefractModel>`, plus `GuardianshipAndCarePanel` and a button to launch the native `WindowRole::UrgentAlertModal` window.
- In **`ProjectsVolume` & `WalletVolume`** (`src/components/volumes/projects.rs`, `wallet.rs`):
  - Render `render_contribution_lifecycle_bar` (`data-pool-equals-paid="false"`) with interactive buttons to fund the project obligation pool vs. settle contributor payouts vs. approve commons transition, proving `Pool Funded != Contributor Paid`.
- In **`BrowserVolume` & `TalkVolume`** (`src/components/volumes/browser.rs`, `communications.rs`):
  - Render `render_contextual_assertion_badge_with_evidence` on Dialectical Web Annotations and `@agent` responses so `Observation`, `Inference`, `Allegation`, `Attestation`, and `AdjudicatedFinding` remain visually and semantically distinct.
- In **`PoetSynthVolume`, `AnatomyVolume`, and `LibraryVolume`** (`src/components/volumes/poet_synth.rs`, `library.rs`):
  - Embed `Holo10dViewport` (`EtalonSurfaceWidget`) so users can inspect live Holographic 10D v2 (`HOLO` `0x0306`) phase-space specimens directly inside Poet, Anatomy, and the Semantic Library.
- In **`AdminVolume` & `SettingsVolume`** (`src/components/volumes/admin.rs`, `settings.rs`):
  - Surface `DesktopApplianceProfile` workspace presets, `MobileAvfSanctuaryProfile` (`pKVM` EL2 probe, `vsock` channels, `.p64` thermal/battery KV-cache governor, thumb-arc Command Orb geometry, and one-tap `"I am OK"` carer check-in), `webizen-hostd` status/tick trigger, and pure-Rust `.deb` package builder (`build_tri_target_debs_from_receipt`).

### 5.3 Wire the New Native DOM Events (`WP2`)
- **`onbeforeinput`**: On steward DID / proposal / directory inputs, check incoming text via `validate_human_centric_host_nomenclature`; if it contains forbidden collapse terms (`"sovereign"`, `"owl:thing"`), call `evt.prevent_default()` and record `state.write().beforeinput_guard_notice`.
- **`onpaste` / `oncopy` / `oncut`**: On `BrowserVolume` URL bar, `PoetSynthVolume` VibeScript cell bar, and `LibraryVolume` ingest URI input, read `evt.data().data_transfer().get_as_text()` to auto-detect pasted `qualia://` / `https://` / `.10d` / `.gguf` URIs and update `state.write().clipboard_toast`.
- **`ontoggle` / `onbeforetoggle`**: On `<details>` blocks in `AdminVolume`, `WellfairVolume`, and `AnatomyVolume`, downcast via `evt.data().downcast::<aether_native::NativeToggleData>()` to inspect `native.0.new_state` and `native.0.old_state`.
- **`onsubmit` / `onreset` / `onchange`**: Wrap URL navigation, VibeScript evaluation, and Library asset ingestion bars in `<form onsubmit: move |evt| { evt.prevent_default(); ... }>`.
- **`onselect`**: On `PoetSynthVolume` gazetteer document `<textarea>`, read `evt.data().selection()` and update `state.write().selection_range_label`.
- **`onresize`**: Wired on the top-level `<div class: "{root_class}">` in `src/app.rs` updating `state.write().shell_viewport_size`.

### 5.4 Unit Tests (`crates/webizen-ua-os/src/lib.rs` / submodule `#[cfg(test)]`)
Add unit tests in `deb_builder.rs`, `mobile_avf.rs`, `desktop_appliance.rs`, and `host_daemon_runtime.rs` verifying:
1. `deb_builder::tests::builds_deterministic_ar_and_ustar_tri_target_debian_packages` (verifies `!<arch>\n`, `debian-binary`, `control.tar.gz`, `data.tar.gz`, gzip headers, and non-empty SHA3-256 digests for all 3 `.deb` packages).
2. `mobile_avf::tests::governor_scales_kv_cache_and_one_tap_checkin_reverses_care_escalation` (verifies `ThermalPressureLevel::Critical` drops KV-cache cap to 128 MiB and one-tap principal `"I am OK"` check-in reverses an active carer alert).
3. `desktop_appliance::tests::builds_multi_locus_fleet_and_workspace_presets`.
4. `host_daemon_runtime::tests::executes_hostd_cli_commands_and_enforces_public_only_peer_p64_mcp` (verifies `parse_hostd_args`, `execute_hostd_command` for `status`, `run-tick`, `register-project`, `audit-jural-license`, and JSON-RPC 2.0 `handle_peer_p64_mcp_json_rpc_line`).
