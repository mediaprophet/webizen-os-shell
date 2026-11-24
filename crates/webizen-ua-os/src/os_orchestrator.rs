// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Two-Level Branching Hierarchical OS & Per-App Thread Orchestrator (`os_orchestrator`).
//!
//! Architectural invariants:
//! 1. **Level 1 — Root Environmental Tiers (OS-Governed)**:
//!    - `Tier0SafetyAndRealTime`: Preemptive floor (Care Heartbeat Watchdog, Sanctuary Duress/Lock,
//!      Specialist Lease Revocation kill-switch, and 60/120Hz Etalon VSync composition). Can never
//!      be starved by any application branch.
//!    - `Tier1FocusedForegroundApp`: Primary interactive budget dynamically assigned to the currently
//!      focused volume or auxiliary window.
//!    - `Tier2OrbitResidentApps`: Weighted deficit round-robin across softened-to-orbit volumes and
//!      resident Qaps, gated by each Qap's active [`CapabilityLeaseScope`].
//!    - `Tier3EnvironmentalAndElasticCompute`: Opportunistic background & bulk compute (`.gguf -> .p64`,
//!      Bio2RDF/legal ontology `.q42` compilation, Shamir `.q42j` backups, Q-CDN Merkle sync, `.deb` builds),
//!      throttled under thermal/battery pressure and routed across [`ComputeExecutionTarget`].
//! 2. **Level 2 — Per-App Child Thread Sub-Trees (App-Governed Autonomy)**:
//!    - Each App/Qap owns an [`AppOrchestratorBranch`] containing its own child execution threads
//!      ([`AppThreadLane`]) with priority classes `P0CriticalRender` .. `P3DeferredIndex`.
//!    - An application can dynamically reprioritize, boost, pause, resume, or fork its own child
//!      threads within its branch without ever escalating above `Tier0SafetyAndRealTime` or
//!      stealing environmental quota from other branches.
//! 3. **Lease-Coupled Branch Pruning & Elastic Offload Routing**:
//!    - Revoking a Qap's [`CapabilityLeaseScope`] immediately suspends only the child threads in
//!      that app's branch that require that lease while leaving the rest of the app's branch alive.
//!    - Private Sanctuary/WellFair casework is pinned fail-closed to [`ComputeExecutionTarget::LocalSanctuaryArena`];
//!      heavy public ontology compilations (`Bio2RDF`, `DOID`, `HP`, legal corpora -> `.q42`) may route to
//!      [`ComputeExecutionTarget::StewardedWebizenHost`] or [`ComputeExecutionTarget::SynthColabUltraBurst`]
//!      and publish their compiled public `.q42` vaults into the Permissive Commons Q-CDN.

use std::{
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use vibescript_abi::NQuin48;

use crate::{
    host_stewardship::validate_human_centric_host_nomenclature,
    identifiers::q_hash_60,
    mobile_avf::{P64ThermalBatteryGovernor, ThermalPressureLevel},
    os_jobs_leases::{CapabilityLeaseScope, OsControlCenterState},
};

pub const OS_ORCHESTRATOR_STORE_REL_PATH: &str = "wellfair/os_branching_orchestrator.json";

/// Root environmental scheduling tiers governed by the Webizen OS-Shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentalTier {
    /// Preemptive floor: Care Heartbeat Watchdog, Sanctuary Duress/Lock, Lease Revocation, Etalon VSync.
    Tier0SafetyAndRealTime,
    /// Primary interactive slice: Currently focused volume or auxiliary window.
    Tier1FocusedForegroundApp,
    /// Resident background slice: Softened-to-orbit volumes and background Qaps.
    Tier2OrbitResidentApps,
    /// Opportunistic bulk & elastic slice: .gguf->.p64, Bio2RDF->.q42, Shamir .q42j, Q-CDN, .deb builds.
    Tier3EnvironmentalAndElasticCompute,
}

impl EnvironmentalTier {
    pub const ALL: [Self; 4] = [
        Self::Tier0SafetyAndRealTime,
        Self::Tier1FocusedForegroundApp,
        Self::Tier2OrbitResidentApps,
        Self::Tier3EnvironmentalAndElasticCompute,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Tier0SafetyAndRealTime => "tier0_safety_realtime",
            Self::Tier1FocusedForegroundApp => "tier1_foreground_app",
            Self::Tier2OrbitResidentApps => "tier2_orbit_resident",
            Self::Tier3EnvironmentalAndElasticCompute => "tier3_elastic_maintenance",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Tier0SafetyAndRealTime => {
                "Tier 0 · Preemptive Safety, Sanctuary Gate & 60/120Hz Etalon VSync"
            }
            Self::Tier1FocusedForegroundApp => {
                "Tier 1 · Active Foreground Volume / Qap Branch"
            }
            Self::Tier2OrbitResidentApps => {
                "Tier 2 · Softened-to-Orbit Resident App Branches"
            }
            Self::Tier3EnvironmentalAndElasticCompute => {
                "Tier 3 · Environmental Maintenance & Elastic Offload (Host / Colab Burst)"
            }
        }
    }
}

/// App-local priority class within an application's own [`AppOrchestratorBranch`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppThreadPriority {
    /// Highest intra-app priority: Etalon 10D HOLO surface render, audio frame, or active input.
    P0CriticalRender,
    /// Interactive app compute: Live VibeScript cell evaluation, CML legal correlation, SPARQL query.
    P1InteractiveCompute,
    /// App background sync: QDNF social frame broadcast, Solid Turtle sync, QECP envelope sealing.
    P2BackgroundSync,
    /// Deferred app indexing: NLP gazetteer background pass, thumbnail/harmonic LOD precomputation.
    P3DeferredIndex,
}

impl AppThreadPriority {
    pub const ALL: [Self; 4] = [
        Self::P0CriticalRender,
        Self::P1InteractiveCompute,
        Self::P2BackgroundSync,
        Self::P3DeferredIndex,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::P0CriticalRender => "P0_critical_render",
            Self::P1InteractiveCompute => "P1_interactive_compute",
            Self::P2BackgroundSync => "P2_background_sync",
            Self::P3DeferredIndex => "P3_deferred_index",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::P0CriticalRender => "P0 · Critical Render / VSync",
            Self::P1InteractiveCompute => "P1 · Interactive Symbolic / Vibe Compute",
            Self::P2BackgroundSync => "P2 · Background QDNF / Portal Sync",
            Self::P3DeferredIndex => "P3 · Deferred Indexing / Gazetteer",
        }
    }

    /// Relative intra-branch weight for active child threads.
    pub const fn weight_units(self) -> u32 {
        match self {
            Self::P0CriticalRender => 8,
            Self::P1InteractiveCompute => 5,
            Self::P2BackgroundSync => 3,
            Self::P3DeferredIndex => 1,
        }
    }

    pub const fn boost(self) -> Self {
        match self {
            Self::P0CriticalRender => Self::P0CriticalRender,
            Self::P1InteractiveCompute => Self::P0CriticalRender,
            Self::P2BackgroundSync => Self::P1InteractiveCompute,
            Self::P3DeferredIndex => Self::P2BackgroundSync,
        }
    }

    pub const fn demote(self) -> Self {
        match self {
            Self::P0CriticalRender => Self::P1InteractiveCompute,
            Self::P1InteractiveCompute => Self::P2BackgroundSync,
            Self::P2BackgroundSync => Self::P3DeferredIndex,
            Self::P3DeferredIndex => Self::P3DeferredIndex,
        }
    }
}

/// Execution state of an individual child thread lane inside an [`AppOrchestratorBranch`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppThreadState {
    Running,
    PausedByApp,
    SuspendedByLeaseRevocation,
    ThrottledByThermalGovernor,
}

impl AppThreadState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Running => "RUNNING",
            Self::PausedByApp => "PAUSED BY APP",
            Self::SuspendedByLeaseRevocation => "SUSPENDED (LEASE REVOKED)",
            Self::ThrottledByThermalGovernor => "THROTTLED (THERMAL/BATTERY)",
        }
    }
}

/// Target execution locus for compute and ontology-compilation tasks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputeExecutionTarget {
    /// Runs strictly inside the local Desktop / Mobile AVF 42 MiB `SlgArena` Sentinel.
    /// Mandatory for all personal Sanctuary/WellFair casework and private credentials.
    LocalSanctuaryArena,
    /// Offloads heavy public/cooperative compilation or Q-CDN indexing to the user's `webizen-hostd` server.
    StewardedWebizenHost,
    /// Dispatches massive high-RAM jobs (e.g., full Bio2RDF / DOID / HP -> `.q42` or 3D generation)
    /// to the Synth Colab Ultra Burst runner and publishes public `.q42` outputs to Permissive Commons Q-CDN.
    SynthColabUltraBurst,
}

impl ComputeExecutionTarget {
    pub const ALL: [Self; 3] = [
        Self::LocalSanctuaryArena,
        Self::StewardedWebizenHost,
        Self::SynthColabUltraBurst,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::LocalSanctuaryArena => "local_sanctuary_arena",
            Self::StewardedWebizenHost => "stewarded_webizen_host",
            Self::SynthColabUltraBurst => "synth_colab_ultra_burst",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::LocalSanctuaryArena => {
                "Local Desktop / Mobile AVF Sanctuary (42 MiB SlgArena Sentinel · Private Data Allowed)"
            }
            Self::StewardedWebizenHost => {
                "Stewarded webizen-hostd Server (:4242 · Public/Cooperative & Scoped Portal Compute)"
            }
            Self::SynthColabUltraBurst => {
                "Synth Colab Ultra Burst (High-RAM Bio2RDF/Ontology -> .q42 + Permissive Commons Q-CDN)"
            }
        }
    }
}

/// An individual child thread/fiber managed inside an application's [`AppOrchestratorBranch`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppThreadLane {
    pub thread_id: String,
    pub title: String,
    pub priority: AppThreadPriority,
    pub state: AppThreadState,
    pub required_lease: Option<CapabilityLeaseScope>,
    pub execution_target: ComputeExecutionTarget,
    pub contains_private_sanctuary_data: bool,
    /// Effective share (in basis points, `0..=10000`, where `10000 == 100.00%`) of the global OS tick budget.
    pub effective_global_share_bps: u16,
    pub sentinel_arena_budget_kib: u32,
    pub sealed_nquin_parity: u64,
}

/// A per-application / per-volume scheduling branch under the Root Environmental Orchestrator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppOrchestratorBranch {
    pub branch_id: String,
    pub app_or_volume_title: String,
    pub qap_package_id: Option<String>,
    pub assigned_tier: EnvironmentalTier,
    /// Relative branch weight among sibling branches in the same [`EnvironmentalTier`].
    pub branch_weight: u16,
    /// Computed total share of the OS environment (in basis points, `0..=10000`) allocated to this branch.
    pub allocated_env_budget_bps: u16,
    pub child_threads: Vec<AppThreadLane>,
    pub sealed_nquin_parity: u64,
}

/// Receipt for an elastic compute or high-RAM `.q42` compilation dispatch routed by the orchestrator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElasticComputeDispatchReceipt {
    pub dispatch_id: String,
    pub workload_title: String,
    pub requested_target: ComputeExecutionTarget,
    pub resolved_target: ComputeExecutionTarget,
    pub estimated_ram_mib: u32,
    pub contains_private_sanctuary_data: bool,
    pub publish_to_permissive_commons_qcdn: bool,
    pub colab_routine_or_host_endpoint: String,
    pub status_summary: String,
    pub sealed_nquin_parity: u64,
}

/// Complete state of the Two-Level Branching OS & Per-App Thread Orchestrator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HierarchicalOrchestratorState {
    /// Environmental tier budget split in basis points (`[Tier0, Tier1, Tier2, Tier3]`, sum = `10000`).
    pub tier_budget_bps: [u16; 4],
    pub focused_branch_id: String,
    pub thermal_pressure: String,
    pub battery_percent: u8,
    pub on_external_power: bool,
    pub branches: Vec<AppOrchestratorBranch>,
    pub elastic_dispatches: Vec<ElasticComputeDispatchReceipt>,
    pub last_orchestrator_notice: String,
    pub sealed_nquin_parity: u64,
}

impl Default for HierarchicalOrchestratorState {
    fn default() -> Self {
        let mut state = Self {
            tier_budget_bps: [1500, 4500, 2500, 1500],
            focused_branch_id: "branch:wellfair".into(),
            thermal_pressure: "Nominal".into(),
            battery_percent: 100,
            on_external_power: true,
            branches: default_seeded_branches(),
            elastic_dispatches: Vec::new(),
            last_orchestrator_notice: "Branching OS & Per-App Thread Orchestrator active (4 Environmental Tiers · 6 App Sub-Trees)".into(),
            sealed_nquin_parity: 0,
        };
        state.rebalance_all_quotas(None);
        state
    }
}

impl HierarchicalOrchestratorState {
    pub fn load_or_seed(
        storage: &Path,
        control_center: Option<&OsControlCenterState>,
    ) -> Self {
        let path = storage.join(OS_ORCHESTRATOR_STORE_REL_PATH);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(mut loaded) = serde_json::from_slice::<Self>(&bytes) {
                if !loaded.branches.is_empty() {
                    loaded.rebalance_all_quotas(control_center);
                    return loaded;
                }
            }
        }
        let mut seeded = Self::default();
        seeded.rebalance_all_quotas(control_center);
        let _ = seeded.save(storage);
        seeded
    }

    pub fn save(&self, storage: &Path) -> Result<(), String> {
        let path = storage.join(OS_ORCHESTRATOR_STORE_REL_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }

    /// Focus an application/volume branch: elevates that app branch to [`EnvironmentalTier::Tier1FocusedForegroundApp`]
    /// and moves previously focused app branches (except `Tier0` system safety and `Tier3` bulk) to `Tier2OrbitResidentApps`.
    pub fn focus_app_branch(
        &mut self,
        storage: Option<&Path>,
        branch_id: &str,
        control_center: Option<&OsControlCenterState>,
    ) -> Result<(), String> {
        if !self.branches.iter().any(|b| b.branch_id == branch_id) {
            return Err(format!("Unknown orchestrator branch '{branch_id}'"));
        }
        for branch in &mut self.branches {
            if branch.assigned_tier == EnvironmentalTier::Tier0SafetyAndRealTime
                || branch.assigned_tier == EnvironmentalTier::Tier3EnvironmentalAndElasticCompute
            {
                continue;
            }
            if branch.branch_id == branch_id {
                branch.assigned_tier = EnvironmentalTier::Tier1FocusedForegroundApp;
            } else {
                branch.assigned_tier = EnvironmentalTier::Tier2OrbitResidentApps;
            }
        }
        self.focused_branch_id = branch_id.to_owned();
        self.rebalance_all_quotas(control_center);
        self.last_orchestrator_notice = format!(
            "Focused branch '{}' into Tier 1 Foreground slice ({:.1}% OS budget)",
            branch_id,
            f64::from(
                self.branches
                    .iter()
                    .find(|b| b.branch_id == branch_id)
                    .map_or(0, |b| b.allocated_env_budget_bps)
            ) / 100.0
        );
        if let Some(storage) = storage {
            let _ = self.save(storage);
        }
        Ok(())
    }

    /// App-local autonomy: reprioritize (boost or demote) a child thread within its parent app branch.
    /// Invariant: Reprioritizing a child thread re-weights threads *inside* that app's branch budget,
    /// and can never alter the parent branch's environmental tier or steal quota from other apps.
    pub fn reprioritize_app_child_thread(
        &mut self,
        storage: Option<&Path>,
        branch_id: &str,
        thread_id: &str,
        boost: bool,
        control_center: Option<&OsControlCenterState>,
    ) -> Result<AppThreadPriority, String> {
        let branch = self
            .branches
            .iter_mut()
            .find(|b| b.branch_id == branch_id)
            .ok_or_else(|| format!("Unknown branch '{branch_id}'"))?;
        let thread = branch
            .child_threads
            .iter_mut()
            .find(|t| t.thread_id == thread_id)
            .ok_or_else(|| format!("Unknown child thread '{thread_id}' in '{branch_id}'"))?;
        thread.priority = if boost {
            thread.priority.boost()
        } else {
            thread.priority.demote()
        };
        let new_prio = thread.priority;
        self.rebalance_all_quotas(control_center);
        self.last_orchestrator_notice = format!(
            "App branch '{}' reprioritized child thread '{}' -> {} (parent environmental quota invariant preserved)",
            branch_id,
            thread_id,
            new_prio.label()
        );
        if let Some(storage) = storage {
            let _ = self.save(storage);
        }
        Ok(new_prio)
    }

    /// App-local autonomy: pause or resume a child thread inside its branch.
    pub fn toggle_app_child_thread_pause(
        &mut self,
        storage: Option<&Path>,
        branch_id: &str,
        thread_id: &str,
        control_center: Option<&OsControlCenterState>,
    ) -> Result<AppThreadState, String> {
        let branch = self
            .branches
            .iter_mut()
            .find(|b| b.branch_id == branch_id)
            .ok_or_else(|| format!("Unknown branch '{branch_id}'"))?;
        let thread = branch
            .child_threads
            .iter_mut()
            .find(|t| t.thread_id == thread_id)
            .ok_or_else(|| format!("Unknown child thread '{thread_id}' in '{branch_id}'"))?;
        thread.state = match thread.state {
            AppThreadState::Running => AppThreadState::PausedByApp,
            AppThreadState::PausedByApp => AppThreadState::Running,
            other => other,
        };
        let new_state = thread.state;
        self.rebalance_all_quotas(control_center);
        self.last_orchestrator_notice = format!(
            "Child thread '{}' in '{}' is now {}",
            thread_id,
            branch_id,
            new_state.label()
        );
        if let Some(storage) = storage {
            let _ = self.save(storage);
        }
        Ok(new_state)
    }

    /// Adjust environmental branch weight in `Tier2OrbitResidentApps` or `Tier3EnvironmentalAndElasticCompute`.
    pub fn set_branch_environmental_weight(
        &mut self,
        storage: Option<&Path>,
        branch_id: &str,
        new_weight: u16,
        control_center: Option<&OsControlCenterState>,
    ) -> Result<(), String> {
        let weight = new_weight.clamp(10, 500);
        let branch = self
            .branches
            .iter_mut()
            .find(|b| b.branch_id == branch_id)
            .ok_or_else(|| format!("Unknown branch '{branch_id}'"))?;
        branch.branch_weight = weight;
        self.rebalance_all_quotas(control_center);
        self.last_orchestrator_notice = format!(
            "Updated environmental weight for '{}' to {}",
            branch_id, weight
        );
        if let Some(storage) = storage {
            let _ = self.save(storage);
        }
        Ok(())
    }

    /// Apply thermal & battery governor feedback (`P64ThermalBatteryGovernor`) to the root tiers.
    pub fn apply_thermal_battery_governor(
        &mut self,
        storage: Option<&Path>,
        governor: &P64ThermalBatteryGovernor,
        control_center: Option<&OsControlCenterState>,
    ) {
        self.battery_percent = governor.battery_percent;
        self.on_external_power = governor.on_external_power;
        self.thermal_pressure = format!("{:?}", governor.thermal_level);
        self.tier_budget_bps = match governor.thermal_level {
            ThermalPressureLevel::Nominal => [1500, 4500, 2500, 1500],
            ThermalPressureLevel::Fair => [1800, 4700, 2500, 1000],
            ThermalPressureLevel::Serious => [2200, 5000, 2300, 500],
            ThermalPressureLevel::Critical => [2500, 5500, 1800, 200],
        };
        let throttle_deferred = governor.thermal_level >= ThermalPressureLevel::Serious;
        for branch in &mut self.branches {
            for thread in &mut branch.child_threads {
                if throttle_deferred && thread.priority == AppThreadPriority::P3DeferredIndex {
                    if thread.state == AppThreadState::Running {
                        thread.state = AppThreadState::ThrottledByThermalGovernor;
                    }
                } else if !throttle_deferred
                    && thread.state == AppThreadState::ThrottledByThermalGovernor
                {
                    thread.state = AppThreadState::Running;
                }
            }
        }
        self.rebalance_all_quotas(control_center);
        if let Some(storage) = storage {
            let _ = self.save(storage);
        }
    }

    /// Route an intensive compute or ontology-compilation task across:
    /// - `LocalSanctuaryArena` (42 MiB Sentinel on Desktop/Mobile AVF),
    /// - `StewardedWebizenHost` (`webizen-hostd`), or
    /// - `SynthColabUltraBurst` (High-RAM `.q42` compilation + Permissive Commons Q-CDN distribution).
    ///
    /// **Privacy & Human-Rights Invariant**:
    /// Any workload carrying private Sanctuary/WellFair casework (`contains_private_sanctuary_data == true`)
    /// is **refused** offload to `SynthColabUltraBurst` and pinned fail-closed to `LocalSanctuaryArena`
    /// (or encrypted scoped disclosure on `StewardedWebizenHost`), and is **never** published to
    /// the public Permissive Commons Q-CDN!
    pub fn route_elastic_compute_workload(
        &mut self,
        storage: Option<&Path>,
        workload_title: &str,
        requested_target: ComputeExecutionTarget,
        estimated_ram_mib: u32,
        contains_private_sanctuary_data: bool,
        publish_to_permissive_commons_qcdn: bool,
    ) -> Result<ElasticComputeDispatchReceipt, String> {
        validate_human_centric_host_nomenclature(workload_title)?;
        if workload_title.trim().is_empty() {
            return Err("Workload title must not be empty.".into());
        }
        if contains_private_sanctuary_data && publish_to_permissive_commons_qcdn {
            return Err(
                "Refused fail-closed: Private Sanctuary or personal WellFair casework can never be published to the public Permissive Commons Q-CDN."
                    .into(),
            );
        }
        if contains_private_sanctuary_data
            && requested_target == ComputeExecutionTarget::SynthColabUltraBurst
        {
            return Err(
                "Refused fail-closed: Private Sanctuary or personal WellFair casework cannot be offloaded to external Synth Colab Ultra Burst; keep private evaluations in LocalSanctuaryArena."
                    .into(),
            );
        }

        // Auto-escalate public high-RAM (> 42 MiB) ontology compilations when requested.
        let resolved_target = if !contains_private_sanctuary_data
            && estimated_ram_mib > 512
            && requested_target == ComputeExecutionTarget::LocalSanctuaryArena
        {
            ComputeExecutionTarget::SynthColabUltraBurst
        } else if !contains_private_sanctuary_data
            && estimated_ram_mib > 42
            && requested_target == ComputeExecutionTarget::LocalSanctuaryArena
        {
            ComputeExecutionTarget::StewardedWebizenHost
        } else {
            requested_target
        };

        let colab_routine_or_host_endpoint = match resolved_target {
            ComputeExecutionTarget::LocalSanctuaryArena => {
                "local://slg-arena-42mib/zero-heap-sentinel".to_owned()
            }
            ComputeExecutionTarget::StewardedWebizenHost => {
                "qpr://ns.webcivics.net:4242/q42-compiler-worker".to_owned()
            }
            ComputeExecutionTarget::SynthColabUltraBurst => {
                "synth://colab-ultra-burst/routine/high-ram-q42-bio2rdf-compiler".to_owned()
            }
        };

        let dispatch_id = format!("dispatch:{:08x}", {
            let mut h = Sha3_256::new();
            h.update(workload_title.as_bytes());
            h.update(resolved_target.code().as_bytes());
            h.update(estimated_ram_mib.to_le_bytes());
            let d = h.finalize();
            u32::from_le_bytes([d[0], d[1], d[2], d[3]])
        });

        let qcdn_note = if publish_to_permissive_commons_qcdn {
            " · Public .q42 output registered in Permissive Commons Q-CDN Swarm (<10 KiB SignedRootIndex)"
        } else {
            " · Local/Scoped output only (not shared on public Q-CDN)"
        };

        let status_summary = format!(
            "Routed '{}' ({} MiB) -> {}{}",
            workload_title,
            estimated_ram_mib,
            resolved_target.title(),
            qcdn_note
        );

        let sealed_nquin_parity = NQuin48::from_parts(
            q_hash_60(&dispatch_id),
            q_hash_60("urn:webizen:orchestrator:elastic-compute-dispatch"),
            q_hash_60(resolved_target.code()),
            u64::from(estimated_ram_mib),
            u64::from(publish_to_permissive_commons_qcdn),
        )
        .parity();

        let receipt = ElasticComputeDispatchReceipt {
            dispatch_id,
            workload_title: workload_title.trim().to_owned(),
            requested_target,
            resolved_target,
            estimated_ram_mib,
            contains_private_sanctuary_data,
            publish_to_permissive_commons_qcdn,
            colab_routine_or_host_endpoint,
            status_summary: status_summary.clone(),
            sealed_nquin_parity,
        };
        self.elastic_dispatches.insert(0, receipt.clone());
        self.elastic_dispatches.truncate(32);
        self.last_orchestrator_notice = status_summary;
        if let Some(storage) = storage {
            let _ = self.save(storage);
        }
        Ok(receipt)
    }

    /// Recompute every branch's environmental share (`allocated_env_budget_bps`) and every child
    /// thread's effective global share (`effective_global_share_bps`), enforcing:
    /// 1. `Tier0SafetyAndRealTime` floor is always preserved.
    /// 2. Any child thread whose `required_lease` is currently revoked in `OsControlCenterState`
    ///    transitions immediately to [`AppThreadState::SuspendedByLeaseRevocation`] with `0` quota.
    /// 3. Child thread reprioritization only redistributes budget *within* that branch's `allocated_env_budget_bps`.
    pub fn rebalance_all_quotas(&mut self, control_center: Option<&OsControlCenterState>) {
        // 1. Update lease-coupled thread states first.
        if let Some(cc) = control_center {
            for branch in &mut self.branches {
                let Some(pkg) = branch.qap_package_id.as_deref() else {
                    continue;
                };
                for thread in &mut branch.child_threads {
                    if let Some(scope) = thread.required_lease {
                        let granted = cc.is_qap_lease_granted(pkg, scope);
                        if !granted {
                            thread.state = AppThreadState::SuspendedByLeaseRevocation;
                        } else if thread.state == AppThreadState::SuspendedByLeaseRevocation {
                            thread.state = AppThreadState::Running;
                        }
                    }
                }
            }
        }

        // 2. Partition each EnvironmentalTier's budget across the branches in that tier.
        for (tier_idx, tier) in EnvironmentalTier::ALL.into_iter().enumerate() {
            let tier_bps = u32::from(self.tier_budget_bps[tier_idx]);
            let total_branch_weight: u32 = self
                .branches
                .iter()
                .filter(|b| b.assigned_tier == tier)
                .map(|b| u32::from(b.branch_weight.max(1)))
                .sum();

            for branch in &mut self.branches {
                if branch.assigned_tier != tier {
                    continue;
                }
                let branch_bps = if total_branch_weight == 0 {
                    0
                } else {
                    (tier_bps * u32::from(branch.branch_weight.max(1))) / total_branch_weight
                } as u16;
                branch.allocated_env_budget_bps = branch_bps;

                // 3. Distribute `branch_bps` across active child threads inside `branch`.
                let active_thread_weight: u32 = branch
                    .child_threads
                    .iter()
                    .filter(|t| t.state == AppThreadState::Running)
                    .map(|t| t.priority.weight_units())
                    .sum();

                for thread in &mut branch.child_threads {
                    if thread.state != AppThreadState::Running || active_thread_weight == 0 {
                        thread.effective_global_share_bps = 0;
                    } else {
                        thread.effective_global_share_bps = ((u32::from(branch_bps)
                            * thread.priority.weight_units())
                            / active_thread_weight)
                            as u16;
                    }
                    thread.sealed_nquin_parity = NQuin48::from_parts(
                        q_hash_60(&thread.thread_id),
                        q_hash_60("urn:webizen:orchestrator:child-thread-lane"),
                        q_hash_60(thread.priority.code()),
                        u64::from(thread.effective_global_share_bps),
                        u64::from(thread.sentinel_arena_budget_kib),
                    )
                    .parity();
                }

                branch.sealed_nquin_parity = NQuin48::from_parts(
                    q_hash_60(&branch.branch_id),
                    q_hash_60("urn:webizen:orchestrator:app-branch"),
                    q_hash_60(branch.assigned_tier.code()),
                    u64::from(branch.allocated_env_budget_bps),
                    branch.child_threads.len() as u64,
                )
                .parity();
            }
        }

        let mut parity_acc = 0u64;
        for b in &self.branches {
            parity_acc ^= b.sealed_nquin_parity;
        }
        self.sealed_nquin_parity = parity_acc;
    }
}

fn default_seeded_branches() -> Vec<AppOrchestratorBranch> {
    vec![
        AppOrchestratorBranch {
            branch_id: "branch:tier0-safety".into(),
            app_or_volume_title: "OS Root Safety, Sanctuary Gate & Etalon VSync".into(),
            qap_package_id: None,
            assigned_tier: EnvironmentalTier::Tier0SafetyAndRealTime,
            branch_weight: 100,
            allocated_env_budget_bps: 1500,
            child_threads: vec![
                AppThreadLane {
                    thread_id: "sys:care-heartbeat-watchdog".into(),
                    title: "Care Heartbeat & Duty-of-Inquiry Watchdog".into(),
                    priority: AppThreadPriority::P0CriticalRender,
                    state: AppThreadState::Running,
                    required_lease: None,
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: true,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 2048,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "sys:sanctuary-duress-lease-gate".into(),
                    title: "Sanctuary Duress & Specialist Lease Kill-Switch Gate".into(),
                    priority: AppThreadPriority::P0CriticalRender,
                    state: AppThreadState::Running,
                    required_lease: None,
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: true,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 2048,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "sys:etalon-vsync-compositor".into(),
                    title: "Etalon Native Vulkan/DX12 VSync Compositor".into(),
                    priority: AppThreadPriority::P0CriticalRender,
                    state: AppThreadState::Running,
                    required_lease: None,
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 4096,
                    sealed_nquin_parity: 0,
                },
            ],
            sealed_nquin_parity: 0,
        },
        AppOrchestratorBranch {
            branch_id: "branch:wellfair".into(),
            app_or_volume_title: "WellFair · Human-Rights Casework, Semantic Instruments & Specialist Portal".into(),
            qap_package_id: Some("WellFair".into()),
            assigned_tier: EnvironmentalTier::Tier1FocusedForegroundApp,
            branch_weight: 120,
            allocated_env_budget_bps: 4500,
            child_threads: vec![
                AppThreadLane {
                    thread_id: "wellfair:cml-human-rights-correlator".into(),
                    title: "CML Legal & Human-Rights Evidence Correlator (UDHR / ICESCR / UN CRPD)".into(),
                    priority: AppThreadPriority::P1InteractiveCompute,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::QualiaQ42Storage),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: true,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 8192,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "wellfair:diagnostic-instrument-symbolic-eval".into(),
                    title: "Role-Qualified Semantic Instrument Symbolic Evaluator (ADHD / PTSD / ICF)".into(),
                    priority: AppThreadPriority::P1InteractiveCompute,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::GovernedVibeScriptEval),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: true,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 6144,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "wellfair:specialist-portal-lease-publisher".into(),
                    title: "Scoped Specialist Portal Bundle Derivation & Six-Vector Audit Sync".into(),
                    priority: AppThreadPriority::P2BackgroundSync,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::QdnfSocialNetwork),
                    execution_target: ComputeExecutionTarget::StewardedWebizenHost,
                    contains_private_sanctuary_data: true,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 4096,
                    sealed_nquin_parity: 0,
                },
            ],
            sealed_nquin_parity: 0,
        },
        AppOrchestratorBranch {
            branch_id: "branch:poet".into(),
            app_or_volume_title: "Poet & Synth · VibeScript & 10D v2 HOLO Workbench".into(),
            qap_package_id: Some("Poet".into()),
            assigned_tier: EnvironmentalTier::Tier2OrbitResidentApps,
            branch_weight: 100,
            allocated_env_budget_bps: 0,
            child_threads: vec![
                AppThreadLane {
                    thread_id: "poet:holo-10d-raymarch".into(),
                    title: "Etalon 10D v2 HOLO (0x0306) <=32-Step Raymarch Viewport".into(),
                    priority: AppThreadPriority::P0CriticalRender,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::EtalonGpuSurface),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 4096,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "poet:vibescript-cell-dag".into(),
                    title: "Governed VibeScript Reactive Cell DAG & Authoring Review".into(),
                    priority: AppThreadPriority::P1InteractiveCompute,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::GovernedVibeScriptEval),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 6144,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "poet:nlp-gazetteer-sealer".into(),
                    title: "NLP Gazetteer Span-to-NQuin48 Background Indexer".into(),
                    priority: AppThreadPriority::P3DeferredIndex,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::QualiaQ42Storage),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 2048,
                    sealed_nquin_parity: 0,
                },
            ],
            sealed_nquin_parity: 0,
        },
        AppOrchestratorBranch {
            branch_id: "branch:browser".into(),
            app_or_volume_title: "Browser · Dual-Web Reach (Chora + Ring C WWW Worker)".into(),
            qap_package_id: Some("Browser".into()),
            assigned_tier: EnvironmentalTier::Tier2OrbitResidentApps,
            branch_weight: 90,
            allocated_env_budget_bps: 0,
            child_threads: vec![
                AppThreadLane {
                    thread_id: "browser:chora-copied-frame-pipe".into(),
                    title: "Ring C Copied-Frame RGBA8 Pipe & Input Forwarder".into(),
                    priority: AppThreadPriority::P0CriticalRender,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::EtalonGpuSurface),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 4096,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "browser:mode3-chora-graph-routes".into(),
                    title: "Mode 3 Hybrid ChoraGraphRoutes & Contextual DID Gate".into(),
                    priority: AppThreadPriority::P1InteractiveCompute,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::SanctuaryContextualDid),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 3072,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "browser:social-qdnf-annotations".into(),
                    title: "SocialQdnf Semantic Group Bookmark & Annotation Broadcaster".into(),
                    priority: AppThreadPriority::P2BackgroundSync,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::QdnfSocialNetwork),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 2048,
                    sealed_nquin_parity: 0,
                },
            ],
            sealed_nquin_parity: 0,
        },
        AppOrchestratorBranch {
            branch_id: "branch:library".into(),
            app_or_volume_title: "Library · Semantic File-System, QECP & Bio2RDF Ontologies".into(),
            qap_package_id: Some("Library".into()),
            assigned_tier: EnvironmentalTier::Tier2OrbitResidentApps,
            branch_weight: 80,
            allocated_env_budget_bps: 0,
            child_threads: vec![
                AppThreadLane {
                    thread_id: "library:qecp-c2pa-verifier".into(),
                    title: "QECP C2PA JUMBF & ML-DSA-65 Evidence Verifier".into(),
                    priority: AppThreadPriority::P1InteractiveCompute,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::QualiaQ42Storage),
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: true,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 3072,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "library:solid-turtle-bridge".into(),
                    title: "W3C Solid / LDP RDF Turtle <-> NQuin48 Sync".into(),
                    priority: AppThreadPriority::P2BackgroundSync,
                    state: AppThreadState::Running,
                    required_lease: Some(CapabilityLeaseScope::QdnfSocialNetwork),
                    execution_target: ComputeExecutionTarget::StewardedWebizenHost,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 2048,
                    sealed_nquin_parity: 0,
                },
            ],
            sealed_nquin_parity: 0,
        },
        AppOrchestratorBranch {
            branch_id: "branch:tier3-elastic".into(),
            app_or_volume_title: "Environmental Bulk & Elastic Ontology / Model Compiler".into(),
            qap_package_id: None,
            assigned_tier: EnvironmentalTier::Tier3EnvironmentalAndElasticCompute,
            branch_weight: 100,
            allocated_env_budget_bps: 1500,
            child_threads: vec![
                AppThreadLane {
                    thread_id: "elastic:bio2rdf-q42-compiler".into(),
                    title: "Bio2RDF / DOID / HP / Legal Ontology -> .q42 Compiler & Q-CDN Seeder".into(),
                    priority: AppThreadPriority::P2BackgroundSync,
                    state: AppThreadState::Running,
                    required_lease: None,
                    execution_target: ComputeExecutionTarget::SynthColabUltraBurst,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 4096,
                    sealed_nquin_parity: 0,
                },
                AppThreadLane {
                    thread_id: "elastic:gguf-to-p64-vault".into(),
                    title: "Local GGUF -> 16 KiB Page-Aligned .p64 Weight Compiler".into(),
                    priority: AppThreadPriority::P3DeferredIndex,
                    state: AppThreadState::Running,
                    required_lease: None,
                    execution_target: ComputeExecutionTarget::LocalSanctuaryArena,
                    contains_private_sanctuary_data: false,
                    effective_global_share_bps: 0,
                    sentinel_arena_budget_kib: 2048,
                    sealed_nquin_parity: 0,
                },
            ],
            sealed_nquin_parity: 0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_stewardship::NaturalHostStewardshipFabric;

    #[test]
    fn app_child_thread_reprioritization_never_steals_environmental_quota_or_starves_tier0() {
        let mut orch = HierarchicalOrchestratorState::default();
        let tier0_before = orch
            .branches
            .iter()
            .find(|b| b.branch_id == "branch:tier0-safety")
            .unwrap()
            .allocated_env_budget_bps;
        let poet_branch_before = orch
            .branches
            .iter()
            .find(|b| b.branch_id == "branch:poet")
            .unwrap()
            .allocated_env_budget_bps;
        let gaz_share_before = orch
            .branches
            .iter()
            .find(|b| b.branch_id == "branch:poet")
            .unwrap()
            .child_threads
            .iter()
            .find(|t| t.thread_id == "poet:nlp-gazetteer-sealer")
            .unwrap()
            .effective_global_share_bps;

        // Boost Poet's deferred gazetteer thread twice (P3 -> P2 -> P1).
        orch.reprioritize_app_child_thread(
            None,
            "branch:poet",
            "poet:nlp-gazetteer-sealer",
            true,
            None,
        )
        .unwrap();
        let new_prio = orch
            .reprioritize_app_child_thread(
                None,
                "branch:poet",
                "poet:nlp-gazetteer-sealer",
                true,
                None,
            )
            .unwrap();
        assert_eq!(new_prio, AppThreadPriority::P1InteractiveCompute);

        let tier0_after = orch
            .branches
            .iter()
            .find(|b| b.branch_id == "branch:tier0-safety")
            .unwrap()
            .allocated_env_budget_bps;
        let poet_branch_after = orch
            .branches
            .iter()
            .find(|b| b.branch_id == "branch:poet")
            .unwrap()
            .allocated_env_budget_bps;
        let gaz_share_after = orch
            .branches
            .iter()
            .find(|b| b.branch_id == "branch:poet")
            .unwrap()
            .child_threads
            .iter()
            .find(|t| t.thread_id == "poet:nlp-gazetteer-sealer")
            .unwrap()
            .effective_global_share_bps;

        // Invariant: Tier 0 and Poet's total branch allocation are unchanged; only intra-branch shares shifted.
        assert_eq!(tier0_before, tier0_after);
        assert_eq!(poet_branch_before, poet_branch_after);
        assert!(gaz_share_after > gaz_share_before);
    }

    #[test]
    fn revoking_qap_lease_suspends_only_matching_child_threads_and_private_casework_refuses_colab_qcdn() {
        let tmp = std::env::temp_dir().join(format!(
            "webizen-orch-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let fabric = NaturalHostStewardshipFabric::default();
        let mut cc = OsControlCenterState::load_or_seed(&tmp, &fabric).unwrap();
        let mut orch = HierarchicalOrchestratorState::load_or_seed(&tmp, Some(&cc));

        // Revoke `GovernedVibeScriptEval` on `Poet`.
        cc.set_qap_lease(
            &tmp,
            "Poet",
            CapabilityLeaseScope::GovernedVibeScriptEval,
            false,
        )
        .unwrap();
        orch.rebalance_all_quotas(Some(&cc));

        let poet = orch
            .branches
            .iter()
            .find(|b| b.branch_id == "branch:poet")
            .unwrap();
        let vibe_thread = poet
            .child_threads
            .iter()
            .find(|t| t.thread_id == "poet:vibescript-cell-dag")
            .unwrap();
        let holo_thread = poet
            .child_threads
            .iter()
            .find(|t| t.thread_id == "poet:holo-10d-raymarch")
            .unwrap();
        assert_eq!(
            vibe_thread.state,
            AppThreadState::SuspendedByLeaseRevocation
        );
        assert_eq!(vibe_thread.effective_global_share_bps, 0);
        assert_eq!(holo_thread.state, AppThreadState::Running);
        assert!(holo_thread.effective_global_share_bps > 0);

        // Private Sanctuary data must be refused Colab offload and public Q-CDN sharing.
        assert!(orch
            .route_elastic_compute_workload(
                Some(&tmp),
                "Private PTSD & Housing Evidence Evaluation",
                ComputeExecutionTarget::SynthColabUltraBurst,
                128,
                true,
                false,
            )
            .is_err());
        assert!(orch
            .route_elastic_compute_workload(
                Some(&tmp),
                "Private PTSD & Housing Evidence Evaluation",
                ComputeExecutionTarget::LocalSanctuaryArena,
                32,
                true,
                true,
            )
            .is_err());

        // Public Bio2RDF ontology compilation > 512 MiB escalates cleanly to SynthColabUltraBurst and shares to Q-CDN.
        let receipt = orch
            .route_elastic_compute_workload(
                Some(&tmp),
                "Bio2RDF DrugBank & DOID/HP Clinical Ontology -> .q42",
                ComputeExecutionTarget::LocalSanctuaryArena,
                2048,
                false,
                true,
            )
            .unwrap();
        assert_eq!(
            receipt.resolved_target,
            ComputeExecutionTarget::SynthColabUltraBurst
        );
        assert!(receipt.publish_to_permissive_commons_qcdn);
        let _ = fs::remove_dir_all(&tmp);
    }
}
