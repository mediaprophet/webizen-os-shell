# Codex Flagship Excellence Plan: Webizen OS-Shell, Holographic 10D v2 Studio, Cryptographic Quorum APT Repo, Sanctuary Escrow & 3-Node Mesh (`WP-EX1`–`WP-EX6`)
**Date:** 2026-11-22  
**Copyright (c) 2026 Timothy Charles Holborn — CC BY-NC-ND 4.0**  
**Target Workspace:** `C:\github\webizenai\webizen-os-shell`  
**Execution Role:** External Coding Agent (**Codex**) — **SOURCE-ONLY IMPLEMENTATION (NO CARGO COMMANDS)**  
**Verification & Build Owner:** **Antigravity Coordinator** (runs all `cargo test`, `cargo build`, and runtime verification passes after Codex reports completion)

---

## 0. Mandatory Rules & Constraints for Codex

1. **STRICT NO-CARGO RULE**:
   - **Do NOT run `cargo check`, `cargo test`, `cargo build`, `cargo clippy`, `cargo update`, or `cargo metadata`.**
   - Your role is strictly to author complete, type-correct Rust source files, CSS upgrades, module exports, and `#[cfg(test)]` unit tests across `WP-EX1` through `WP-EX6`.
   - When all 6 Work-Packages are saved to disk, stop and report the list of created and modified files.

2. **LICENSING & HUMAN-RIGHTS VOCABULARY INVARIANTS**:
   - Start all new Rust files with:
     ```rust
     // Copyright (c) 2026 Timothy Charles Holborn.
     // SPDX-License-Identifier: CC-BY-NC-ND-4.0
     ```
   - **Never** use the words `"sovereign"`, `"sovereignty"`, or `"self-sovereign"` anywhere in code, comments, identifiers, test strings, `.deb` control/APT files, or UI labels (`validate_human_centric_host_nomenclature` rejects them fail-closed). Always use human-rights agreement vocabulary (**Human-Centric**, **Human Rights**, **Human Dignity**, **Human Agency**, **Natural Person**, **Sanctuary**, **Guardianship**, **Permissive Commons**).
   - Always use **"8-Sector Command Orb"** / **"Command Orb (`◉`)"** (never "Command Wheel").
   - Inside Aether `rsx! { ... }`, never use a variable or format interpolation named `{fragment}` (collides with Aether's `Fragment` component; use `fragment_sel` or `ann_frag`).

3. **LOCKFILE & ZERO-NEW-CRATE CONSTRAINT**:
   - Do **NOT** add any new external crates to `Cargo.toml` or `crates/webizen-ua-os/Cargo.toml`.
   - Use only the crates already present (`std`, `flate2`, `sha3`, `serde`, `serde_json`, `etalon`, `vibe`, `vibescript-abi`, `aether`, `aether-native`, `anyrender`, `peniko`, `kurbo`, `blitz-dom`, `blitz-traits`).

---

## Work-Package Overview & File Ownership

| Work-Package | Title | Owned / Modified Files |
| --- | --- | --- |
| **WP-EX1** | Real `.10d` Holographic v2 (`HOLO` `0x0306`) Specimen Resolver, 4-Mode Shader Studio & Mode 3 `GovernedVibeScriptEval` Bridge | `crates/webizen-ua-os/src/holo_10d_specimen.rs` (NEW), `crates/webizen-ua-os/src/browser_app.rs`, `src/components/viewports.rs` |
| **WP-EX2** | Cryptographic `M-of-N` Quorum Verifier, Signed Debian 13 APT Repository Generator (`Packages` / `InRelease`), Default `CareResponseAuthority` & Direct Local `.p64` Peer MCP | `crates/webizen-ua-os/src/deb_builder.rs`, `crates/webizen-ua-os/src/updater.rs`, `crates/webizen-ua-os/src/host_daemon_runtime.rs`, `src/app.rs` |
| **WP-EX3** | Flagship `SanctuaryVolume` — 4-Slot `VaultContainerV2` Inspector, `M-of-N` Shamir Social Guardianship Recovery Escrow & Mobile AVF Pocket Co-Signer Pairing | `crates/webizen-ua-os/src/sanctuary_onboarding.rs` (NEW), `src/components/volumes/sanctuary.rs` |
| **WP-EX4** | Universal Semantic Command Palette & Cross-Volume `IntentBus` Omni-Router (`intent_router.rs` + Launcher & Ribbon Bar) | `crates/webizen-ua-os/src/intent_router.rs` (NEW), `src/components/orbit_wheel.rs`, `src/components/stage_host.rs` |
| **WP-EX5** | Built-In 3-Node Local Mesh Loopback Simulator (`Desktop` + `Mobile AVF Co-Signer` + `Camper/Home webizen-hostd`) & Flagship Permissive Commons `ProjectsVolume` / `WalletVolume` | `crates/webizen-ua-os/src/mesh_simulator.rs` (NEW), `src/components/volumes/admin.rs`, `src/components/volumes/projects.rs`, `src/components/volumes/wallet.rs` |
| **WP-EX6** | Spatial Glassmorphism Design System, Live Telemetry HUD Badges & State Wiring | `assets/shell.css`, `src/components/halo_bar.rs`, `src/state/shell_state.rs`, `crates/webizen-ua-os/src/lib.rs` |

---

## WP-EX1: Real `.10d` Holographic v2 (`HOLO` `0x0306`) Specimen Resolver, 4-Mode Shader Studio & Mode 3 `GovernedVibeScriptEval` Bridge

### Files
- `crates/webizen-ua-os/src/holo_10d_specimen.rs` (NEW)
- `crates/webizen-ua-os/src/browser_app.rs`
- `src/components/viewports.rs`

### 1.1 Pure-Rust `.10d` Holographic v2 (`HOLO` `0x0306`) Specimen Resolver (`crates/webizen-ua-os/src/holo_10d_specimen.rs`)
- Implement a self-contained `.10d` v2 `HOLO` (`FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE = 0x0306`, magic `*b"HOLO"`) specimen encoder, decoder, and file resolver:
  - Constants:
    - `pub const FIELD_KIND_HOLO_PHASE_SPACE: u16 = 0x0306;`
    - `pub const HOLO_HEADER_BYTES: usize = 64;`
    - `pub const HOLO_MAX_RAYMARCH_STEPS: u32 = 32;`
  - Types:
    ```rust
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum HoloShaderVisualizationMode {
        /// Hamiltonian stationary-action (delta S = 0) wavefield interference.
        PhaseSpaceWavefield,
        /// Hyperbolic v=2 Mobius morphogenesis w(z) = (az + b) / (cz + d).
        HyperbolicMobiusV2,
        /// Anatomical intrinsic radial tissue depth d in [0, 1] (periosteum -> dermis).
        AnatomicalRadialDepth,
        /// Scale-space harmonic SDF cross-section with 12-triangle proxy lens.
        ScaleSpaceHarmonicSlice,
    }

    impl HoloShaderVisualizationMode {
        pub const ALL: [Self; 4] = [
            Self::PhaseSpaceWavefield,
            Self::HyperbolicMobiusV2,
            Self::AnatomicalRadialDepth,
            Self::ScaleSpaceHarmonicSlice,
        ];
        pub const fn code(self) -> &'static str { ... }
        pub const fn title(self) -> &'static str { ... }
        pub const fn shader_mode_index(self) -> u32 { ... } // 0, 1, 2, 3
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct VerifiedHolo10dSpecimen {
        pub specimen_id: String,
        pub title: String,
        pub source_path: Option<PathBuf>,
        pub field_kind: u16,
        pub version: u16,
        pub holo_segment_count: u16,
        pub wavefield_count: u16,
        pub max_harmonic_order: u8,
        pub max_raymarch_steps: u8,
        pub proxy_aabb_min: [f32; 3],
        pub proxy_aabb_max: [f32; 3],
        pub sdf_smooth_union_radius: f32,
        pub reference_sigma: f32,
        pub default_shader_mode: HoloShaderVisualizationMode,
        pub nquin_subject_hash: u64,
        pub sealed_nquin_parity: u64,
        pub sha3_256_hex: String,
    }
    ```
  - Functions:
    - `pub fn encode_holo_10d_container(spec: &VerifiedHolo10dSpecimen) -> Vec<u8>`: writes a deterministic binary `.10d` v2 container starting with `"10Dv2\0\0\0"` + field tag `0x0306` (`HOLO`) + 64-byte `HOLO` header (`magic: *b"HOLO"`, `version`, `holo_segment_count`, `wavefield_count`, `max_harmonic_order`, `max_raymarch_steps`, `proxy_aabb_min`, `proxy_aabb_max`, `sdf_smooth_union_radius`, `reference_sigma`, `nquin_subject_hash`, and SHA3-derived 32-bit integrity tag).
    - `pub fn decode_holo_10d_container(specimen_id: &str, bytes: &[u8]) -> Result<VerifiedHolo10dSpecimen, String>`: validates magic, `0x0306` tag, `max_raymarch_steps <= 32`, finite `proxy_aabb_min <= proxy_aabb_max`, and integrity tag.
    - `pub fn seed_default_holo_10d_specimens(storage: &Path) -> Result<Vec<VerifiedHolo10dSpecimen>, String>`: seeds 4 real `.10d` v2 files under `<storage>/library/specimens-10d/`:
      1. `chora-spatiotemporal-commons.10d` (`proxy_aabb_min: [-0.95, -0.65, -0.85]`, `proxy_aabb_max: [0.95, 0.65, 0.85]`, `PhaseSpaceWavefield`)
      2. `anatomy-cardiovascular-holo.10d` (`proxy_aabb_min: [-0.55, -0.88, -0.50]`, `proxy_aabb_max: [0.55, 0.88, 0.50]`, `AnatomicalRadialDepth`)
      3. `ecopunk-riparian-biosphere.10d` (`proxy_aabb_min: [-0.90, -0.45, -0.90]`, `proxy_aabb_max: [0.90, 0.45, 0.90]`, `ScaleSpaceHarmonicSlice`)
      4. `poet-hyperbolic-mobius-v2.10d` (`proxy_aabb_min: [-0.75, -0.75, -0.75]`, `proxy_aabb_max: [0.75, 0.75, 0.75]`, `HyperbolicMobiusV2`)
    - `pub fn resolve_holo_10d_specimen(storage: &Path, query_or_uri: &str) -> Result<VerifiedHolo10dSpecimen, String>`: resolves a file path, `qualia://` URI, or specimen slug by decoding the `.10d` container on disk (seeding defaults if needed) and returning the verified `VerifiedHolo10dSpecimen`.
    - `pub fn proxy_corners_for_specimen(spec: &VerifiedHolo10dSpecimen) -> [[f32; 3]; 8]`: returns the 8 3D bounding-box corners from `spec.proxy_aabb_min` and `spec.proxy_aabb_max`.

### 1.2 Add `GovernedVibeScriptEval` to Mode 3 Hybrid Browser (`crates/webizen-ua-os/src/browser_app.rs`)
- Add `GovernedVibeScriptEval` to `pub enum HybridWebCapabilityChannel` (updating `HybridWebCapabilityChannel::ALL` from `[Self; 6]` to `[Self; 7]`).
- Map `GovernedVibeScriptEval` in `spec(self)`:
  - `channel_id: "vibescript-eval"`
  - `title: "Governed VibeScript Cell Evaluation (42 MiB Sentinel)"`
  - `required_lease: "lease:vibescript:governed-eval"`
  - `plane_guard: "Plane 4 Instrument (Never Ambient JS/V8; Sealed NQuin48 Receipt)"`
- In `evaluate_hybrid_page_request`: when `channel == HybridWebCapabilityChannel::GovernedVibeScriptEval` in `BrowserExecutionMode::HybridAugmented`, evaluate the expression via `crate::poet_synth::poet_eval_native_cell(payload)` (falling back to a deterministic expression evaluation summary if `payload` is a resource URI) and include the result and `NQuin48` parity in `HybridBridgeReceipt`.

### 1.3 Upgrade `Holo10dViewport` & `BrowserChoraSurface` (`src/components/viewports.rs`)
- In `src/components/viewports.rs`:
  - Resolve the active specimen via `resolve_holo_10d_specimen(&sess.host.storage, &active_specimen_id)` so `HoloViewportControl` uses the specimen's real `proxy_aabb_min`, `proxy_aabb_max`, `holo_segment_count`, `wavefield_count`, `nquin_subject_hash`, `sha3_256_hex`, and `HoloShaderVisualizationMode`!
  - Support switching between the 4 seeded `.10d` specimens and the 4 `HoloShaderVisualizationMode`s (`PhaseSpaceWavefield`, `HyperbolicMobiusV2`, `AnatomicalRadialDepth`, `ScaleSpaceHarmonicSlice`) directly in `Holo10dViewport` (passing the mode index and AABB scale into the 32-byte uniform buffer and 2D vector fallback).
  - In `BridgePreview` and `TrustedBrowserSurface` (`viewports.rs:852, 916`), wire `"vibescript-eval"` to `Some(HybridWebCapabilityChannel::GovernedVibeScriptEval)` so all 5 Mode 3 capability channels evaluate live and return verified `NQuin48` parity receipts!

---

## WP-EX2: Cryptographic `M-of-N` Quorum Verifier, Signed Debian 13 APT Repository Generator, Default `CareResponseAuthority` & Direct Local `.p64` Peer MCP

### Files
- `crates/webizen-ua-os/src/deb_builder.rs`
- `crates/webizen-ua-os/src/updater.rs`
- `crates/webizen-ua-os/src/host_daemon_runtime.rs`
- `src/app.rs`

### 2.1 `DEBIAN/md5sums` & Signed Debian 13 (Trixie) APT Repository Index (`crates/webizen-ua-os/src/deb_builder.rs`)
- In `deb_builder.rs`:
  - Automatically generate a deterministic `md5sums` / `sha3sums` integrity listing inside `control.tar.gz` for every regular file in `data.tar.gz`.
  - Implement:
    ```rust
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AptRepositoryReceipt {
        pub repo_root: PathBuf,
        pub packages_amd64_path: PathBuf,
        pub packages_arm64_path: PathBuf,
        pub inrelease_path: PathBuf,
        pub quorum_signature_hex: String,
        pub sealed_nquin_parity: u64,
    }

    pub fn generate_signed_apt_repository(
        suite_root: &Path,
        built_debs: &[DebBuildReceipt],
        host_domain: &str,
        quorum_threshold_m: u8,
        steward_dids: &[String],
    ) -> Result<AptRepositoryReceipt, String>;
    ```
  - Writes `dists/trixie/main/binary-amd64/Packages`, `Packages.gz`, `dists/trixie/main/binary-arm64/Packages`, `Packages.gz`, and `dists/trixie/InRelease` containing the SHA3-256 package entries and deterministic `M-of-N` steward quorum signature block (`-----BEGIN WEBIZEN QUORUM ATTESTATION-----`).
  - Call `generate_signed_apt_repository` inside `generate_tri_target_debian_suite` (`host_stewardship.rs`) so `package-tri-target-deb` produces a complete installable APT repository alongside the 3 `.deb` files!

### 2.2 Cryptographic Quorum Verifier in `updater.rs`
- In `crates/webizen-ua-os/src/updater.rs`:
  - Add `pub fn verify_quorum_update_manifest(manifest: &UpdateManifest, fabric: &NaturalHostStewardshipFabric) -> Result<String, String>` verifying that the candidate artifact's SHA3-256 digest and quorum signature satisfy `fabric.threshold_m` natural-person steward approvals (and zero unresolved paraconsistent `0x32` dissent blocks) before `UpdateAction::Verify` or `UpdateAction::Apply` admits a quorum-governed update.

### 2.3 Direct Local `.p64` v4 Peer MCP Fallback (`crates/webizen-ua-os/src/host_daemon_runtime.rs`)
- In `host_daemon_runtime.rs`:
  - When `WEBIZEN_PUBLIC_P64_MCP_UPSTREAM` is not set in the environment, `handle_peer_p64_mcp_json_rpc_line` (for `tools/call` `"llm_infer"`) must load `LlmAndAgentManagementRegistry::load_or_init(storage)` and evaluate the prompt locally against the active `.p64` v4 (`p64\0`) model record pinned strictly to `SENSITIVITY_PUBLIC_ONLY = 0` (rejecting any prompt containing `Sanctuary100` markers or private DIDs), returning a structured JSON-RPC 2.0 inference response with the `.p64` model's BLAKE3/SHA3 digest and `NQuin48` receipt!

### 2.4 Default Session-Backed `CareResponseAuthority` (`src/app.rs`)
- In `src/app.rs`:
  - When `try_consume_context::<CareResponseAuthority>()` is `None` and `session()` is `Some(sess)`, construct a default `CareResponseAuthority` bound to the adopted session's `NaturalHostStewardshipFabric`:
    - When `principal_ok == true` (`"I am OK"` reversible check-in), resolve the circle's `supported_principal_did` from `state.host_stewardship.shared_care_circles` matching `alert.circle_id`.
    - When `principal_ok == false` (`"Remote Carer Duty-of-Inquiry Verify"`), resolve the first authorized remote carer DID from the matching circle's `remote_carers`.
  - This ensures the floating **`UrgentAlertModal`** window and `WellfairVolume` carer buttons work out-of-the-box in live sessions!

---

## WP-EX3: Flagship `SanctuaryVolume` — 4-Slot `VaultContainerV2` Inspector, `M-of-N` Shamir Social Guardianship Escrow & Mobile AVF Pocket Co-Signer Pairing

### Files
- `crates/webizen-ua-os/src/sanctuary_onboarding.rs` (NEW)
- `src/components/volumes/sanctuary.rs`

### 3.1 `crates/webizen-ua-os/src/sanctuary_onboarding.rs`
- Implement:
  - `SanctuaryVaultSlotSpec { pub slot_index: u8, pub label: &'static str, pub encryption_profile: &'static str, pub coercion_posture: &'static str, pub max_sensitivity: u8, pub active_record_count: usize, pub sealed_nquin_parity: u64 }` (4 constant-shape slots: `Slot 0 Public Commons`, `Slot 1 Contextual Social`, `Slot 2 Plausible-Deniability Decoy`, `Slot 3 Sanctuary100 Personal Keep`).
  - `ShamirGuardianShareHolder { pub share_index: u8, pub holder_label: String, pub holder_did: String, pub locus_kind: String, pub share_commitment_hex: String, pub verified_present: bool }`
  - `PocketCosignerPairingState { pub mobile_node_id: String, pub mobile_did: String, pub wireguard_endpoint: String, pub vsock_cid: u32, pub pkvm_attestation_status: String, pub paired: bool, pub last_cosign_receipt_hex: String }`
  - `SanctuaryOnboardingState { pub digital_birth_did: String, pub vault_slots: [SanctuaryVaultSlotSpec; 4], pub shamir_threshold_m: u8, pub shamir_total_n: u8, pub shamir_shares: Vec<ShamirGuardianShareHolder>, pub last_reconstruction_test: String, pub pocket_cosigner: PocketCosignerPairingState }`
  - Functions:
    - `pub fn build_sanctuary_onboarding_state(ids: &IdentifierSet, fabric: &NaturalHostStewardshipFabric, gpu: &NativeEtalonStatus) -> SanctuaryOnboardingState`
    - `pub fn simulate_shamir_recovery_ceremony(state: &mut SanctuaryOnboardingState, participating_share_indices: &[u8]) -> Result<String, String>` (verifies `participating_share_indices.len() >= state.shamir_threshold_m as usize`, computes deterministic GF(256)/XOR commitment reconstruction, and records a parity-sealed recovery ceremony receipt).
    - `pub fn pair_and_test_pocket_cosigner(state: &mut SanctuaryOnboardingState, mobile_did: &str, wg_endpoint: &str) -> Result<String, String>` (validates human-centric nomenclature, pairs the `arm64` Mobile AVF Pocket Co-Signer, and generates a co-signed `NQuin48` handshake receipt).

### 3.2 Upgrade `SanctuaryVolume` (`src/components/volumes/sanctuary.rs`)
- Upgrade `SanctuaryVolume` to render:
  1. **Constant-Shape 4-Slot `VaultContainerV2` Inspector** (`Slot 0..3` cards with anti-coercion posture & zero-Nuke-PIN guarantee).
  2. **Contextual Pairwise DID Issuance Guard** (refusing Master DID collapse).
  3. **`M-of-N` Shamir Social Guardianship Recovery Ceremony** (interactive checkboxes for Home Server, 12V Camper Rig, Pocket Co-Signer, and Family/Housemate Guardian shares + **"Verify M-of-N Threshold Reconstruction"** button).
  4. **Mobile AVF Pocket Co-Signer (`arm64`) Pairing & Co-Signing Handshake** panel.

---

## WP-EX4: Universal Semantic Command Palette & Cross-Volume `IntentBus` Omni-Router

### Files
- `crates/webizen-ua-os/src/intent_router.rs` (NEW)
- `src/components/orbit_wheel.rs`
- `src/components/stage_host.rs`

### 4.1 `crates/webizen-ua-os/src/intent_router.rs`
- Implement a universal semantic intake classifier and `IntentBus` dispatcher:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum SemanticIntentAction {
      OpenVolume { stage_slug: &'static str, summary: String },
      NavigateBrowser { uri: String, mode: BrowserExecutionMode },
      OpenHolo10dSpecimen { specimen_query: String },
      StageGgufForP64Conversion { gguf_path: String },
      InvokeGovernedAgent { agent_slug: String, prompt: String },
      SearchSemanticLibrary { query: String },
      RunThreeNodeMeshSimulation,
      BuildTriTargetDebianSuite,
      TriggerPrincipalOkCheckIn,
  }

  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct ClassifiedSemanticIntent {
      pub raw_input: String,
      pub badge: &'static str,
      pub target_plane: IdentifierFabricPlane,
      pub action: SemanticIntentAction,
      pub explanation: String,
      pub sealed_nquin_parity: u64,
  }

  pub fn classify_semantic_intent(raw_input: &str) -> Result<ClassifiedSemanticIntent, String>;
  ```
  - Rules:
    - First check `validate_human_centric_host_nomenclature(raw_input)` (reject forbidden collapse terms fail-closed).
    - Input starting with `@<slug> <prompt>` -> `InvokeGovernedAgent`.
    - Input ending with `.gguf` -> `StageGgufForP64Conversion`.
    - Input ending with `.10d` or starting with `holo:` -> `OpenHolo10dSpecimen`.
    - Input starting with `https://`, `http://`, `qualia://`, or `webizen://` -> `NavigateBrowser` with `resolve_mode_for_uri`.
    - Input matching `"mesh:sim"` or `"simulate mesh"` -> `RunThreeNodeMeshSimulation`.
    - Input matching `"deb:build"` or `"package deb"` -> `BuildTriTargetDebianSuite`.
    - Input matching `"care:ok"` or `"i am ok"` -> `TriggerPrincipalOkCheckIn`.
    - Otherwise -> fuzzy-match against volume names (`talk`, `mail`, `browser`, `keep`, `library`, `instruments`, `poet`, `wellfair`, `anatomy`, `wallet`, `projects`, `settings`, `admin`) or fallback to `SearchSemanticLibrary`.

### 4.2 Wire the Omni-Intent Bar into `OrbitWheel` & `StageHost` (`src/components/orbit_wheel.rs`, `src/components/stage_host.rs`)
- Add an interactive **Universal Semantic Intent & Omni-Command Bar** at the top of `OrbitWheel` (`Constellation Launcher`) and a quick-trigger button in `StageHost`'s top ribbon so typing or pasting any URI, `.gguf` path, `.10d` specimen, `@agent` prompt, or command (`mesh:sim`, `deb:build`, `care:ok`) classifies it live and executes the cross-volume transition on `onsubmit`!

---

## WP-EX5: Built-In 3-Node Local Mesh Loopback Simulator & Flagship `ProjectsVolume` / `WalletVolume`

### Files
- `crates/webizen-ua-os/src/mesh_simulator.rs` (NEW)
- `src/components/volumes/admin.rs`
- `src/components/volumes/projects.rs`
- `src/components/volumes/wallet.rs`

### 5.1 3-Node Local Mesh Loopback Simulator (`crates/webizen-ua-os/src/mesh_simulator.rs`)
- Implement `pub fn run_three_node_mesh_simulation(storage: &Path, ids: &IdentifierSet, gpu: &NativeEtalonStatus) -> Result<ThreeNodeMeshSimulationReceipt, String>` executing a 5-stage multi-node verification across:
  - **Node 1**: `Desktop Appliance (amd64 · Home Sanctuary Anchor)`
  - **Node 2**: `Mobile AVF Pocket Co-Signer (arm64 · pKVM EL2 Co-Signer)`
  - **Node 3**: `Camper/Home webizen-hostd (amd64 · Q-CDN & Care Relay)`
- The 5 executed stages in `ThreeNodeMeshSimulationReceipt { pub stages: Vec<MeshSimulationStageReceipt>, pub apt_repo_inrelease_path: String, pub total_nquin_seals: usize, pub all_stages_passed: bool }`:
  1. **Stage 1 — `M-of-N` Quorum Host Update & AI-Agent Vote Rejection**: Proposes `webizen-hostd_0.1.1_amd64.deb`, proves an AI `:agent:` vote is rejected fail-closed, records `2-of-2` natural-person votes from Desktop + Mobile AVF Co-Signer, ratifies the update, and builds the signed `.deb` + APT `InRelease`.
  2. **Stage 2 — Personal Project → Cooperative Group & `Q-CDN` Swarm Replication**: Registers a hosted project and transitions it across the Home Server and 12V Camper Rig `Q-CDN` nodes.
  3. **Stage 3 — 3-Way Jural Commercial Licensing Gate**: Verifies `$0` free execution for `NaturalPersonPrincipal` and enforces `monthly_base_fee_cents > 0` for `IncorporatedLegalEntity`.
  4. **Stage 4 — Elderly/Supported-Person Heartbeat Lapse & Mobile AVF `"I am OK"` Reversal**: Dispatches a remote carer alert on heartbeat lapse and reverses it via `trigger_one_tap_principal_ok_checkin` with a sealed `DutyOfInquiry` `NQuin48` receipt.
  5. **Stage 5 — Public-Only Peer `.p64` MCP JSON-RPC Inference & Sanctuary Egress Block**: Calls `handle_peer_p64_mcp_json_rpc_line` (`"llm_infer"`) at `SENSITIVITY_PUBLIC_ONLY = 0` and verifies that a `Sanctuary100` request is blocked from peer egress.
- Add a **"Run 3-Node Mesh Lifecycle Simulation (Desktop + Mobile AVF + webizen-hostd)"** card and interactive stage timeline in `AdminVolume` (`src/components/volumes/admin.rs`).

### 5.2 Upgrade `ProjectsVolume` & `WalletVolume` (`src/components/volumes/projects.rs`, `src/components/volumes/wallet.rs`)
- In `ProjectsVolume`: add an interactive **Permissive Commons Obligation Pool & Contributor Attribution Ledger** alongside `render_contribution_lifecycle_bar` showing originating contributors, bounded commercial obligation pool drawdown, human-rights no-charge exemption flags (proving no-charge use is not a payment or discharge), and `Pool Funded != Contributor Paid` gate verification.
- In `WalletVolume`: add an interactive **Pre-Funded Provider Credit Allocation & Periodic Reconciliation Inspector** (enforcing `drawdown <= authorized_backing`, zero automatic overdraft, provider-attributed unused funds carry/return/delegation, and explicit unit/valuation display before any effort score is presented as monetary compensation) alongside `render_contribution_lifecycle_bar`.

---

## WP-EX6: Spatial Glassmorphism Design System, Live Telemetry HUD Badges & State Wiring

### Files
- `assets/shell.css`
- `src/components/halo_bar.rs`
- `src/state/shell_state.rs`
- `crates/webizen-ua-os/src/lib.rs`

### 6.1 Polish `assets/shell.css` & `HaloBar` (`src/components/halo_bar.rs`)
- In `HaloBar`: display a live urgent care alert count pill on the `WellFair` button when `active_care_alerts > 0`, and a quick button to open the **Universal Omni-Intent Bar (`Ctrl+K` / `◉`)**.
- In `assets/shell.css`: add clean, high-contrast glassmorphism rules for:
  - `.holo-viewport`, `.holo-viewport-object`, `.shader-mode-pills`
  - `.si-readiness-card`, `.wellfair-review-panel`, `.wellfair-dimension-row`, `.contextual-assertion-badge`, `.contribution-lifecycle-bar`
  - `.omni-intent-bar`, `.mesh-sim-timeline`, `.mesh-stage-card`, `.sanctuary-vault-grid`, `.shamir-share-card`

### 6.2 Unit Tests
- Add unit tests in:
  - `holo_10d_specimen::tests::encodes_decodes_and_resolves_four_holo_10d_v2_specimens`
  - `deb_builder::tests::generates_signed_apt_repository_with_packages_and_inrelease`
  - `sanctuary_onboarding::tests::verifies_shamir_m_of_n_recovery_and_pocket_cosigner_pairing`
  - `intent_router::tests::classifies_uris_models_specimens_agents_and_rejects_forbidden_nomenclature`
  - `mesh_simulator::tests::executes_five_stage_three_node_mesh_lifecycle_end_to_end`
