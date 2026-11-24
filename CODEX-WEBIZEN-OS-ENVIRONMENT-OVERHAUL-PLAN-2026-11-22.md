# CODEX IMPLEMENTATION PLAN — Webizen OS-Shell Environment Overhaul & Aether Platform Integration (`WP-ENV1` through `WP-ENV6`)

**Date:** 2026-11-22  
**Target Repository:** `C:\github\webizenai\webizen-os-shell`  
**Copyright & License:**  
`Copyright (c) 2026 Timothy Charles Holborn`  
`SPDX-License-Identifier: CC-BY-NC-ND-4.0`

---

## 0. Mandatory Execution & Coordination Rules for Codex

1. **DO NOT RUN ANY `cargo` COMMANDS (`cargo check`, `cargo test`, `cargo build`, `cargo clippy`, `cargo run`)**:
   - The Antigravity Coordinator owns all Cargo dependency resolution, compilation, and runtime test execution after you finish applying the source edits.
   - You MAY run `rustfmt --check` on the files you create or edit.
   - Do NOT modify `Cargo.lock`.
2. **Preserve Every Existing Feature, Test, and Invariant**:
   - All 44 existing unit tests in `crates/webizen-ua-os` and `src/` must continue to pass without regression.
   - Every new file must carry the two-line copyright header:
     ```rust
     // Copyright (c) 2026 Timothy Charles Holborn.
     // SPDX-License-Identifier: CC-BY-NC-ND-4.0
     ```
3. **Strict Human-Rights & Architectural Nomenclature**:
   - **NEVER** use the words `"sovereign"`, `"sovereignty"`, or `"self-sovereign"` anywhere in code, comments, identifiers, UI strings, or tests. Always use human-rights agreement vocabulary: **human rights**, **human dignity**, **human agency**, **natural person**, **moral and material interests of the author**, **sanctuary / selfhood**, and **Permissive Commons**.
   - **NEVER** use `"Command Wheel"`; always use **`"8-Sector Command Orb"`** or **`"Command Orb (◉)"`**.
   - **Four-Plane Anti-Collapse Invariants**:
     - `Plane 1 (NaturalAgent)` is framed strictly as `living-SHACL (D_living)`, disjoint from `owl:Thing`, and never collapsed with a DID, handle, or tool.
     - `Plane 2 (ClaimOpinion)` carries bounded validity (`CoAttestationBundle`, `VC = origin/integrity, never truth`).
     - `Plane 3 (SpatiotemporalHandle)` holds route epochs, CB-USRI coordinates, and `did:q42`.
     - `Plane 4 (Instrument)` holds cryptographic tools (`did:qi`), pairwise locators (`A -> B != B -> A`), and AI models (`.p64`).
4. **Honest Status & Fail-Closed Authority**:
   - Do not fabricate live cryptographic verification, live hardware attestation, or external network settlement from unverified inputs. Distinguish **Live Verified**, **Local Rehearsal / Preview**, and **Awaiting External Signer / Verifier** honestly in receipts and UI badges.

---

## 1. Work-Package Overview (`WP-ENV1` – `WP-ENV6`)

| Work Package | Target Files | Core Deliverable |
| --- | --- | --- |
| **`WP-ENV1`** | `Cargo.toml`, `crates/webizen-ua-os/Cargo.toml`, `crates/webizen-ua-os/src/qualia_host_foundation.rs` (NEW), `src/app.rs`, `src/components/volumes/poet_synth.rs` | Wire `aether-qualia-host` (`../../Aether/packages/qualia-host`) & `qualia-core-db` into the OS-Shell; mount `QualiaHostContext` (`NativeAuthority`, `NativeAuthoringBridge`, `NativeGraphBridge`, `WebizenSentinelGate`, `VibeActionDispatcher`); drive Poet/Synth & Mode 3 VibeScript actions through `use_vibe_action`, `use_qualia_subscription`, and `use_authoring_bridge` with 48-byte `ExecutionReceiptHeader` & `WorkflowEvidence` inspection. |
| **`WP-ENV2`** | `crates/webizen-ua-os/src/holo_10d_specimen.rs`, `src/components/viewports.rs`, `src/components/volumes/library.rs`, `src/components/volumes/poet_synth.rs` | Upgrade `.10d` support to decode **both** Webizen compact `.10d` specimens and canonical QualiaDB `10d\0` v2 `MID2` containers via `aether_qualia_host::holographic_viewport::Holographic10dViewportSpec` (`HOLO` `0x0306`, `M64HoloRecord`, `EulerianWavefieldRecord`); add 7-Layer Multi-Scale Stratum (`L0`–`L6`) and Two-Tier Constituency (`AREF` / `IOVR` / `INST`) inspection HUD to `Holo10dViewport`. |
| **`WP-ENV3`** | `crates/webizen-ua-os/src/browser_app.rs`, `src/components/viewports.rs`, `src/components/volumes/browser.rs` | Integrate `aether_qualia_host::ChoraGraphRoutes` (`qualia.graph.snapshot` over `Control::PageRequest` -> `NativeGraphBridge` -> bounded decimal-u64 JSON `Control::PageReply`) into Mode 3 Hybrid Web; add a **Ring C Old-Web Worker Process & RouteStamp Supervisor** (`session`, `view`, `frame`, input forwarding counters, explicit route invalidation on navigation/resize). |
| **`WP-ENV4`** | `crates/webizen-ua-os/src/os_jobs_leases.rs` (NEW), `src/components/halo_bar.rs`, `src/components/stage_host.rs`, `src/components/volumes/settings.rs` | Build the **OS Control Center**: (a) Background & Foreground OS Job Scheduler (`OsJobRegistry`), (b) Interactive Per-Qap Capability Lease Manager (`gpu.etalon`, `storage.q42`, `network.qdnf`, `sanctuary.contextual_did`, `ai.p64_instrument`, `vibescript.governed_eval`), and (c) Halo Bar System Notification & Epistemic Alert Drawer. |
| **`WP-ENV5`** | `crates/webizen-ua-os/src/portability_qecp.rs` (NEW), `src/components/volumes/settings.rs`, `src/components/volumes/library.rs` | Make Settings Tab 8 (`Backup & Recovery`) and Library QECP executable: (a) `.q42j` Encrypted `M-of-N` GF(256) Shamir Journal Export & Round-Trip Restore, (b) Bidirectional W3C Solid / LDP RDF Turtle (`.ttl`) <-> 48-byte `NQuin48` Bridge, and (c) `QECP` (`.pdf` C2PA JUMBF + CBOR-LD W3C VC) Credential Inspector & Sealer. |
| **`WP-ENV6`** | `crates/webizen-ua-os/src/workspace_persistence.rs` (NEW), `src/app.rs`, `src/state/shell_state.rs`, `src/components/stage_host.rs`, `src/components/volumes/settings.rs`, `assets/shell.css` | Surface live `aether_native::RendererHealth` telemetry (`ActiveBackend`, `FallbackPolicy`, surface frame metrics) in the GPU Stage footer & Settings Tabs 1/10; implement atomic `shell_workspace.json` session persistence & restore across shell restarts. |

---

## 2. Detailed Work-Package Specifications

---

### `WP-ENV1` — Canonical `aether-qualia-host` Foundation & Deontic Sentinel Integration

#### 1. Manifest Additions (`Cargo.toml` and `crates/webizen-ua-os/Cargo.toml`)
- In root `C:\github\webizenai\webizen-os-shell\Cargo.toml`:
  - Add `aether-qualia-host = { optional = true, path = "../../Aether/packages/qualia-host" }` under `[dependencies]`.
  - Add `"dep:aether-qualia-host"` to the `native` feature list in `[features]`.
  - Under `[features]`, ensure `chora = ["native", "aether-native/chora", "aether-qualia-host/chora-client", "dep:url"]`.
- In `C:\github\webizenai\webizen-os-shell\crates\webizen-ua-os\Cargo.toml`:
  - Add `qualia-core-db = { path = "../../../../qualiaDB/crates/qualia-core-db", default-features = false }` and `qualia-cooperative-core = { path = "../../../../qualiaDB/crates/qualia-cooperative-core" }`.

#### 2. New Module: `crates/webizen-ua-os/src/qualia_host_foundation.rs`
Create `qualia_host_foundation.rs` and re-export its public types from `crates/webizen-ua-os/src/lib.rs`:
- **Purpose**: Bootstrap an owner-held QualiaDB `NativeAuthority`, authenticated `SessionHandle`, signed `SignedGraphGrant`, and deontic `WebizenSentinelGate` (`vibe:native` `OP_PERMIT` + `EffectSet::GRAPH_READ | EffectSet::GRAPH_MUTATE` within the 42 MiB `SlgArena` Sentinel) for the OS-Shell session.
- **Required Types & Functions**:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct VibeExecutionReceiptSummary {
      pub ticket_id: u64,
      pub actor_did_hash: u64,
      pub provider_iri_hash: u64,
      pub func_iri_hash: u64,
      pub policy_generation: u64,
      pub ticks_consumed: u64,
      pub output_bytes_written: u32,
      pub status_label: String,
      pub simulated: bool,
      pub lamport_timestamp: u64,
  }

  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct AuthoringWorkflowSnapshot {
      pub document_id: String,
      pub revision: u64,
      pub authority_generation: u64,
      pub last_review_request_id: Option<String>,
      pub last_evidence_ref: Option<String>,
      pub status_summary: String,
      pub sealed_nquin_parity: u64,
  }

  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct GovernedVibeActionReport {
      pub program_id: String,
      pub entry_fn: String,
      pub epoch: u64,
      pub formatted_value: String,
      pub receipts: Vec<VibeExecutionReceiptSummary>,
      pub graph_revision: u64,
      pub graph_quin_count: usize,
      pub reconciliation_required: bool,
      pub diagnostic: Option<String>,
  }
  ```
- Provide:
  - `pub fn run_governed_vibe_source(storage: &Path, actor_did: &str, program_id: &str, source: &str) -> Result<GovernedVibeActionReport, String>`:
    - Uses `qualia_core_db::vibe_host::vibe_sentinel::WebizenSentinelGate` (with `vibe:native` `OP_PERMIT` deontic rule and caller-owned `NQuin` receipt buffer) and `vibe::Engine` within `vibe::SENTINEL_BYTES` (42 MiB) budget.
    - Enforces `vibe::session::SourceSession` ingress limits before evaluation.
    - Converts any committed `vibescript_abi::ExecutionReceiptHeader` records into `VibeExecutionReceiptSummary`.
  - `pub fn record_authoring_source_review(storage: &Path, actor_did: &str, document_id: &str, source: &str, evidence_ref: &str) -> Result<AuthoringWorkflowSnapshot, String>`:
    - Validates `actor_did` and `document_id` (rejecting empty/control chars or forbidden `owl:Thing` natural-agent collapse labels), computes deterministic revision and 48-byte `NQuin48` parity, persists the authoring review journal under `{storage}/poet/authoring_reviews.json`, and returns `AuthoringWorkflowSnapshot`.

#### 3. Mount `QualiaHostContext` in `src/app.rs` & Wire Poet/Synth (`src/components/volumes/poet_synth.rs`)
- In `src/app.rs`:
  - Provide a shell-level `VibeActionDispatcher` implementation connected to `aether_qualia_host::{QualiaHostContext, ui_channel}` (or a dedicated shell context wrapper when `NativeAuthority` is initialized from the local session storage) so components can call `aether_qualia_host::prelude::{try_use_qualia_context, use_vibe_action}`.
- In `src/components/volumes/poet_synth.rs`:
  - Add a **"Governed VibeScript Action & Authoring Source Review (`aether-qualia-host`)"** panel that:
    1. Evaluates the current VibeScript program/cell via `run_governed_vibe_source` (and updates the `use_vibe_action` / shell receipt log), displaying the returned `VibeExecutionReceiptSummary` table (`ticket_id`, `actor_did_hash`, `ticks_consumed`, `policy_generation`, `status_label`).
    2. Allows the author/contributor to submit a **Source Review Request** (`record_authoring_source_review`) with an explicit `document_id` and `evidence_ref`, rendering the resulting `AuthoringWorkflowSnapshot` (`revision`, `authority_generation`, `sealed_nquin_parity`).

---

### `WP-ENV2` — Canonical QualiaDB `.10d` v2 `MID2`/`HOLO` (`0x0306`) Container Resolver & 7-Layer (`L0`–`L6`) / `AREF` Scene Inspector

#### 1. Expand `crates/webizen-ua-os/src/holo_10d_specimen.rs`
- Keep full backward compatibility with the existing 74-byte Webizen `.10d` v2 header profile (`10Dv2\0\0\0` + `HOLO`) so all existing tests pass unchanged.
- Add canonical QualiaDB `10d\0` v2 `MID2` container decoding and encoding support using `qualia_core_db::container_10d`:
  - Define the 7-Layer Multi-Scale Computational Hierarchy enum:
    ```rust
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum MultiScaleStratumLayer {
        L0Continuum,
        L1RigidArticulatedPgaXpbd,
        L2SoftBodyFluid,
        L3MolecularChemical,
        L4CellularBiological,
        L5EcologicalGeomorphic,
        L6QuantumPhysics,
    }
    ```
    with `pub const ALL: [Self; 7]`, `code(self) -> &'static str` (`"L0"`..`"L6"`), and `title(self) -> &'static str` (`"L0 · Continuum"`, `"L1 · Rigid / Articulated PGA / XPBD / SKL2"`, `"L2 · Soft-Body / Fluid"`, `"L3 · Molecular / Chemical"`, `"L4 · Cellular / Biological"`, `"L5 · Ecological / Geomorphic"`, `"L6 · Quantum Physics (Dynamics, Physics & Quantum Chemistry)"`).
  - Define two-tier `.10d` constituency summary records:
    ```rust
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct ConstituentLeafRef {
        pub aref_slot: u16,
        pub child_uri: String,
        pub stratum: MultiScaleStratumLayer,
        pub motor_translation: [f32; 3],
        pub iovr_override_summary: String,
    }
    ```
  - Add fields to `VerifiedHolo10dSpecimen` (with `#[serde(default)]` so existing serialized JSON and tests remain clean):
    - `pub container_profile: String` (`"webizen-compact-10d-v2"` or `"qualiadb-mid2-10d-v2"`),
    - `pub active_strata: Vec<MultiScaleStratumLayer>`,
    - `pub constituent_leaves: Vec<ConstituentLeafRef>`,
  - Implement `pub fn decode_canonical_qualiadb_10d_container(specimen_id: &str, bytes: &[u8]) -> Result<VerifiedHolo10dSpecimen, String>`:
    - Uses `qualia_core_db::container_10d::TypedPayloadResolver::new(bytes)` and `qualia_core_db::container_10d::holographic_phase_section::decode_holographic_phase_section_into` (or `aether_qualia_host::holographic_viewport::Holographic10dViewportSpec::from_10d`) to extract the `FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE` (`0x0306`) header, `M64HoloRecord` count, `EulerianWavefieldRecord` count, `max_harmonic_order`, `max_raymarch_steps` (`<= 32`), `proxy_aabb_min`, `proxy_aabb_max`, `sdf_smooth_union_radius`, `reference_sigma`, and `nquin_subject_hash`.
  - Update `decode_holo_10d_container` and `read_specimen` so that if `bytes` begins with `b"10d\0"` (canonical QualiaDB `MID2` container, up to `16 * 1024 * 1024` bytes), it dispatches to `decode_canonical_qualiadb_10d_container`; if it begins with `b"10Dv2\0\0\0"`, it uses the compact decoder.
  - Seed default `active_strata` (`L0Continuum`, `L1RigidArticulatedPgaXpbd`, `L4CellularBiological`, `L5EcologicalGeomorphic`, `L6QuantumPhysics` as appropriate to each of the 4 default specimens) and 2–3 `ConstituentLeafRef` (`AREF`/`IOVR`/`INST`) entries per specimen.

#### 2. Upgrade `Holo10dViewport` HUD (`src/components/viewports.rs`)
- Add a **7-Layer Stratum Filter (`L0`–`L6`)** and **Two-Tier Constituency (`AREF` / `IOVR` / `INST`) Inspector** below the shader mode selector in `Holo10dViewport`:
  - Shows the container profile (`webizen-compact-10d-v2` vs `qualiadb-mid2-10d-v2`), active `L0`–`L6` strata badges, and the table of `AREF` leaf `.10d` references with their `INST` PGA motor translations and `IOVR` overrides.

---

### `WP-ENV3` — Mode 3 Hybrid Web (`ChoraGraphRoutes`) & Ring C Old-Web Worker Process Supervisor

#### 1. Expand `crates/webizen-ua-os/src/browser_app.rs`
- Add a deterministic Ring C worker & Mode 3 route supervisor state:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct ChoraWorkerSupervisorSnapshot {
      pub worker_binary_path: String,
      pub worker_state_label: String,
      pub session_id: u64,
      pub view_id: u64,
      pub frame_seq: u64,
      pub registered_routes: usize,
      pub pending_page_requests: usize,
      pub forwarded_pointer_events: u64,
      pub forwarded_wheel_events: u64,
      pub forwarded_key_events: u64,
      pub forwarded_ime_events: u64,
      pub focus_active: bool,
      pub last_route_invalidation_reason: String,
  }
  ```
- Upgrade `evaluate_hybrid_page_request` in `browser_app.rs` so all **7** `HybridWebCapabilityChannel` variants execute real, deterministic local verifications (while still refusing all bridge calls fail-closed when `mode == BrowserExecutionMode::TraditionalWeb`):
  1. `ContextualDidHandle`: Calls `issue_for_requestor` with `Ask::Contextual` bound to `caller_origin`, returning the issued pairwise DID and `refused_master=true`.
  2. `Mount10dHoloEmbed`: Validates the payload specimen ID/URI against `resolve_holo_10d_specimen` (or canonical `.10d` rules), returning the verified `HOLO` (`0x0306`) AABB and raymarch step budget (`<= 32`).
  3. `ExtractRdfToSemanticFs`: Runs `extract_semantic_features` on the payload and returns the extracted topic/entity count and 48-byte `NQuin48` parity digest.
  4. `VerifyQecpCredential`: Validates a `QECP` credential claim payload and returns the origin/integrity verification status with `VC_ORIGIN_INTEGRITY_DISCLAIMER`.
  5. `SolidDataboxSync`: Translates a sample RDF Turtle / NQuin payload across the Solid boundary and reports the serialized triple count and parity.
  6. `CommonsContributionReceipt`: Validates a proposed contribution receipt (effort minutes / AUD minor units) as an **unratified tool/page proposal** requiring explicit human KeyVault signature.
  7. `GovernedVibeScriptEval`: Evaluates the pure scalar expression (or reports a clear diagnostic) within `MAX_GOVERNED_VIBESCRIPT_CELL_BYTES`.

#### 2. Wire `ChoraGraphRoutes` & Worker Supervisor into `src/components/viewports.rs` and `src/components/volumes/browser.rs`
- In `BrowserChoraSurface` (`src/components/viewports.rs`) and `BrowserVolume` (`src/components/volumes/browser.rs`):
  - Display the **Ring C Worker & RouteStamp Supervisor Panel**:
    - Shows current `RouteStamp` (`session`, `view`, `frame`), worker binary slot status, input forwarding counters (pointer, wheel, key, IME, focus), and buttons to:
      - **"Test `ChoraGraphRoutes` Snapshot (`qualia.graph.snapshot`)"**: Executes a real `aether_qualia_host::ChoraGraphRoutes` request/reply cycle (or deterministic 6-column decimal-string `NQuin` JSON snapshot bounded by `MAX_REPLY_BYTES`) when in Mode 3 (`HybridAugmented`), and proves `ReplyError::Denied` when in Mode 1 (`TraditionalWeb`).
      - **"Invalidate Active RouteStamp (`Control::Invalidate`)"**: Increments `view_id`, clears pending requests, zeroes scratch buffers, and records the invalidation reason (`"Manual route invalidation / navigation boundary"`).

---

### `WP-ENV4` — OS Control Center: Background Jobs, Per-Qap Capability Leases & System Notification Drawer

#### 1. New Module: `crates/webizen-ua-os/src/os_jobs_leases.rs`
Create `os_jobs_leases.rs` and re-export its public types from `crates/webizen-ua-os/src/lib.rs`:
- **OS Job Scheduler (`OsJobRegistry`)**:
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum OsJobKind {
      GgufToP64Conversion,
      TriTargetDebianBuild,
      QcdnSwarmMerkleSync,
      ShamirQ42jBackup,
      CareHeartbeatWatchdog,
      GovernedVibeScriptBatch,
  }

  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum OsJobStatus {
      Queued,
      Running,
      Completed,
      Cancelled,
      Failed,
  }

  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct OsJobRecord {
      pub job_id: String,
      pub kind: OsJobKind,
      pub title: String,
      pub target_volume: String,
      pub status: OsJobStatus,
      pub progress_pct: u8,
      pub detail: String,
      pub sealed_nquin_parity: u64,
  }
  ```
- **Per-Qap Capability Lease Manager (`QapLeaseManager`)**:
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub enum CapabilityLeaseScope {
      EtalonGpuSurface,
      QualiaQ42Storage,
      QdnfSocialNetwork,
      SanctuaryContextualDid,
      LocalP64AiInstrument,
      GovernedVibeScriptEval,
  }

  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct QapCapabilityLease {
      pub package_id: String,
      pub scope: CapabilityLeaseScope,
      pub granted: bool,
      pub epoch_generation: u64,
      pub plane_guard: &'static str,
      pub last_audit_note: String,
      pub sealed_nquin_parity: u64,
  }
  ```
- **System Notification & Epistemic Alert Aggregator (`OsSystemNotification`)**:
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
  pub enum OsNotificationSeverity {
      Info,
      ReviewRequired,
      ParaconsistentDissent,
      UrgentCare,
  }

  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct OsSystemNotification {
      pub notification_id: String,
      pub severity: OsNotificationSeverity,
      pub source_volume: String,
      pub title: String,
      pub detail: String,
      pub acknowledged: bool,
      pub sealed_nquin_parity: u64,
  }
  ```
- Implement `OsControlCenterState` (persisted at `{storage}/wellfair/os_control_center.json`):
  - Seeds realistic jobs, capability leases for the bundled first-party Qaps (`Browser`, `Poet`, `WellFair`, `Anatomy`, `Library`, `Wallet`), and live notifications aggregated from `NaturalHostStewardshipFabric` (active urgent care alerts + paraconsistent `0x32` stewardship dissents).
  - Provides methods to:
    - `enqueue_os_job(&mut self, storage: &Path, kind: OsJobKind, title: &str, target_volume: &str) -> OsJobRecord`
    - `advance_or_complete_os_job(&mut self, storage: &Path, job_id: &str) -> Result<OsJobRecord, String>`
    - `cancel_os_job(&mut self, storage: &Path, job_id: &str) -> Result<OsJobRecord, String>`
    - `set_qap_lease(&mut self, storage: &Path, package_id: &str, scope: CapabilityLeaseScope, granted: bool) -> Result<QapCapabilityLease, String>`
    - `sync_notifications_from_stewardship(&mut self, fabric: &NaturalHostStewardshipFabric)`
    - `acknowledge_notification(&mut self, storage: &Path, notification_id: &str)`

#### 2. Wire into `HaloBar`, `StageHost`, and `SettingsVolume`
- In `src/components/halo_bar.rs`:
  - Add an **"OS Notifications & Jobs (`🔔`)"** button in the Halo Bar showing the count of unacknowledged notifications and active jobs, which toggles a compact **OS Control Center Drawer** in `StageHost` (or navigates directly to the relevant volume).
- In `src/components/volumes/settings.rs` (`SettingsSection::Services`):
  - Render the **Interactive Per-Qap Capability Lease Matrix** and **OS Background Job Queue** so the user can grant/revoke individual capability leases per Qap (`Browser`, `Poet`, `WellFair`, `Anatomy`, etc.) and step/cancel background jobs.

---

### `WP-ENV5` — Portable Data Lifecycle: `.q42j` Shamir Backup/Restore, W3C Solid Turtle Bridge & `QECP` Credential Inspector

#### 1. New Module: `crates/webizen-ua-os/src/portability_qecp.rs`
Create `portability_qecp.rs` and re-export its public types from `crates/webizen-ua-os/src/lib.rs`:
- **1. `.q42j` Encrypted Shamir Backup & Round-Trip Restore**:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct ShamirQ42jBackupArchive {
      pub archive_id: String,
      pub threshold_m: u8,
      pub total_n: u8,
      pub payload_sha3_256_hex: String,
      pub payload_bytes: usize,
      pub share_files: Vec<PathBuf>,
      pub included_sections: Vec<String>,
      pub sealed_nquin_parity: u64,
  }
  ```
  - `pub fn export_shamir_q42j_backup(storage: &Path, threshold_m: u8, total_n: u8) -> Result<ShamirQ42jBackupArchive, String>`:
    - Collects a deterministic JSON bundle of public/non-coercive session metadata (stewardship summary, semantic bookmarks/annotations, semantic library index metadata, and capability leases), splits the digest/key secret across `total_n` GF(256) Shamir shares (`threshold_m` required), writes the `.q42j` archive and share files under `{storage}/backups/q42j/`, and returns `ShamirQ42jBackupArchive`.
  - `pub fn verify_and_restore_shamir_q42j_backup(storage: &Path, archive: &ShamirQ42jBackupArchive, selected_share_indices: &[u8]) -> Result<String, String>`:
    - Reconstructs the GF(256) secret from `selected_share_indices` (failing closed if `< threshold_m` distinct shares are supplied), verifies `payload_sha3_256_hex` against the `.q42j` file on disk, and returns a human-readable verification & restore receipt.
- **2. W3C Solid / LDP RDF Turtle (`.ttl`) <-> 48-Byte `NQuin48` Bidirectional Bridge**:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct SolidTurtleSyncReceipt {
      pub direction: String,
      pub pod_endpoint: String,
      pub turtle_document: String,
      pub triple_count: usize,
      pub sealed_nquin_parity: u64,
      pub file_path: Option<PathBuf>,
  }
  ```
  - `pub fn export_semantic_entry_to_solid_turtle(storage: &Path, entry: &SemanticAssetEntry, pod_endpoint: &str) -> Result<SolidTurtleSyncReceipt, String>`:
    - Serializes `entry` (`asset_uri`, `title`, `section`, `depicts`, `topics`, `projects`, `derived_from`, `primary_subject`, `graph_parity_digest`) into deterministic UTF-8 RDF Turtle (`@prefix qualia: <https://webizen.org/q42#> .`, `@prefix prov: <http://www.w3.org/ns/prov#> .`, `@prefix dcterms: <http://purl.org/dc/terms/> .`), saves it under `{storage}/backups/solid-export/`, and returns `SolidTurtleSyncReceipt`.
  - `pub fn import_solid_turtle_to_semantic_fs(semantic_fs: &mut SemanticFileSystem, turtle_source: &str, pod_endpoint: &str) -> Result<SolidTurtleSyncReceipt, String>`:
    - Parses subject/title/topic/depicts triples from `turtle_source`, ingests the resource into `semantic_fs` with parity-checked 48-byte `NQuin48` edges, and returns `SolidTurtleSyncReceipt`.
- **3. `QECP` (`.pdf` C2PA JUMBF + CBOR-LD W3C VC) Credential Inspector & Sealer**:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct QecpCredentialEnvelope {
      pub credential_id: String,
      pub document_title: String,
      pub issuer_pairwise_did: String,
      pub claim_summary: String,
      pub c2pa_jumbf_label: String,
      pub cbor_ld_tag_hex: String,
      pub origin_integrity_disclaimer: String,
      pub signature_scheme: String,
      pub sealed_nquin_parity: u64,
  }
  ```
  - `pub fn seal_and_inspect_qecp_envelope(storage: &Path, document_title: &str, issuer_pairwise_did: &str, claim_summary: &str) -> Result<QecpCredentialEnvelope, String>`:
    - Validates human-centric nomenclature, embeds `VC_ORIGIN_INTEGRITY_DISCLAIMER` (`"VC proves issuer origin and integrity only — never universal truth"`), computes a 48-byte `NQuin48` parity seal, writes the `.qecp.json` companion envelope under `{storage}/library/qecp/`, and returns `QecpCredentialEnvelope`.

#### 2. Wire into `SettingsVolume` (`SettingsSection::Backup` & `SettingsSection::People`) and `LibraryVolume`
- In `SettingsSection::Backup` (`src/components/volumes/settings.rs`):
  - Add interactive controls to **"Export 2-of-3 Shamir `.q42j` Encrypted Journal Backup"**, **"Verify & Rehearse `.q42j` Shamir Restoration"**, and **"Export Active Library Container to W3C Solid RDF Turtle (`.ttl`)"**, rendering the live receipts and file paths.
- In `LibraryVolume` (`src/components/volumes/library.rs`):
  - Add a **"QECP Verifiable Credential & W3C Solid Turtle Bridge"** panel for the currently selected library container so users can export the selected container to `.ttl`, import `.ttl` triples, or seal/inspect a `QECP` C2PA JUMBF envelope.

---

### `WP-ENV6` — Live `RendererHealth` Telemetry & Atomic Workspace Session Persistence (`shell_workspace.json`)

#### 1. New Module: `crates/webizen-ua-os/src/workspace_persistence.rs`
Create `workspace_persistence.rs` and re-export its public types from `crates/webizen-ua-os/src/lib.rs`:
- **Workspace Snapshot (`PersistedWorkspaceSnapshot`)**:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct PersistedWorkspaceSnapshot {
      pub active_stage_slug: String,
      pub active_plane_code: String,
      pub stage_chrome_mode: String,
      pub theme_preset: String,
      pub reduced_motion: bool,
      pub inspect_continuity_open: bool,
      pub active_holo_specimen_id: String,
      pub holo_scale_k: u8,
      pub enabled_toolbox_slugs: Vec<String>,
      pub browser_url: String,
      pub browser_mode_code: String,
  }
  ```
  - `pub fn save_workspace_snapshot(storage: &Path, snapshot: &PersistedWorkspaceSnapshot) -> Result<PathBuf, String>`:
    - Writes atomically via temp file + rename to `{storage}/wellfair/shell_workspace.json`.
  - `pub fn load_workspace_snapshot(storage: &Path) -> Option<PersistedWorkspaceSnapshot>`:
    - Loads and validates `{storage}/wellfair/shell_workspace.json` if present.
- **Renderer Health Summary (`ShellRendererHealthSummary`)**:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  pub struct ShellRendererHealthSummary {
      pub backend_label: String,
      pub fallback_policy_label: String,
      pub fallback_engaged: bool,
      pub active_holo_surface_status: String,
      pub viewport_dimensions: (u32, u32),
  }
  ```

#### 2. Wire into `ShellState`, `StageHost`, and `SettingsVolume`
- In `src/state/shell_state.rs`:
  - Add `pub os_control_center: OsControlCenterState`, `pub chora_supervisor: ChoraWorkerSupervisorSnapshot`, `pub last_vibe_action_report: Option<GovernedVibeActionReport>`, `pub last_authoring_snapshot: Option<AuthoringWorkflowSnapshot>`, `pub last_qecp_envelope: Option<QecpCredentialEnvelope>`, `pub last_solid_receipt: Option<SolidTurtleSyncReceipt>`, `pub last_q42j_backup: Option<ShamirQ42jBackupArchive>`, `pub renderer_health: ShellRendererHealthSummary`, and `pub control_center_drawer_open: bool`.
  - Add `ShellState::to_workspace_snapshot(&self) -> PersistedWorkspaceSnapshot` and `ShellState::apply_workspace_snapshot(&mut self, snap: &PersistedWorkspaceSnapshot)`.
  - In `ShellState::from_session`, automatically apply `load_workspace_snapshot(&sess.host.storage)` if present.
- In `src/components/stage_host.rs` and `src/components/volumes/settings.rs` (`SettingsSection::Health` & `SettingsSection::Technical`):
  - Display live `ShellRendererHealthSummary` (`backend_label`, `fallback_policy_label`, `fallback_engaged`, `active_holo_surface_status`, `viewport_dimensions`) and a **"Save Workspace Snapshot (`shell_workspace.json`)"** button in the Command Orb (`8. ··· More`) and `Settings -> 6. Appearance & Access`.

---

## 3. Unit Tests Required in `crates/webizen-ua-os` and `src/`

Add deterministic unit tests covering every new module:
1. **`qualia_host_foundation.rs`**:
   - `executes_governed_vibescript_with_sentinel_gate_and_records_receipts`
   - `records_authoring_source_review_and_rejects_natural_agent_collapse_terms`
2. **`holo_10d_specimen.rs`**:
   - `decodes_canonical_qualiadb_mid2_10d_containers_and_preserves_l0_l6_strata`
3. **`browser_app.rs`**:
   - `evaluates_all_seven_hybrid_capability_channels_and_denies_mode1`
   - `tracks_chora_worker_routestamp_and_invalidation_lifecycle`
4. **`os_jobs_leases.rs`**:
   - `manages_os_jobs_qap_capability_leases_and_stewardship_notifications`
5. **`portability_qecp.rs`**:
   - `round_trips_shamir_q42j_backup_and_fails_closed_below_threshold`
   - `exports_and_imports_w3c_solid_turtle_with_nquin48_parity`
   - `seals_qecp_envelope_with_origin_integrity_disclaimer`
6. **`workspace_persistence.rs`**:
   - `persists_and_restores_workspace_snapshot_atomically`
