// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Background job records, Qap capability preferences and stewardship notices.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions, TryLockError},
    io::{self, Read, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Deserializer, Serialize};
use sha3::{Digest, Sha3_256};
use vibescript_abi::NQuin48;

use crate::{
    host_stewardship::{
        validate_human_centric_host_nomenclature, CareAlertSeverity, NaturalHostStewardshipFabric,
    },
    identifiers::q_hash_60,
    qap_host::{load_install_registry, validate_package_id},
};

pub const OS_CONTROL_CENTER_REL_PATH: &str = "wellfair/os_control_center.json";
const MAX_SNAPSHOT_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ROWS: usize = 512;
const QAPS: [&str; 6] = [
    "Browser", "Poet", "WellFair", "Anatomy", "Library", "Wallet",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsJobKind {
    GgufToP64Conversion,
    TriTargetDebianBuild,
    QcdnSwarmMerkleSync,
    ShamirQ42jBackup,
    CareHeartbeatWatchdog,
    GovernedVibeScriptBatch,
}

impl OsJobKind {
    pub const ALL: [Self; 6] = [
        Self::GgufToP64Conversion,
        Self::TriTargetDebianBuild,
        Self::QcdnSwarmMerkleSync,
        Self::ShamirQ42jBackup,
        Self::CareHeartbeatWatchdog,
        Self::GovernedVibeScriptBatch,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::GgufToP64Conversion => "gguf-to-p64",
            Self::TriTargetDebianBuild => "debian-build",
            Self::QcdnSwarmMerkleSync => "qcdn-merkle-sync",
            Self::ShamirQ42jBackup => "shamir-q42j-backup",
            Self::CareHeartbeatWatchdog => "care-heartbeat",
            Self::GovernedVibeScriptBatch => "governed-vibe-batch",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::GgufToP64Conversion => "GGUF to .p64 conversion",
            Self::TriTargetDebianBuild => "Tri-target Debian build",
            Self::QcdnSwarmMerkleSync => "Q-CDN Merkle sync",
            Self::ShamirQ42jBackup => "Shamir .q42j backup",
            Self::CareHeartbeatWatchdog => "Care heartbeat watchdog",
            Self::GovernedVibeScriptBatch => "Governed VibeScript batch",
        }
    }

    pub const fn default_volume_slug(self) -> &'static str {
        match self {
            Self::GgufToP64Conversion => "instruments",
            Self::TriTargetDebianBuild => "admin",
            Self::QcdnSwarmMerkleSync => "projects",
            Self::ShamirQ42jBackup => "settings",
            Self::CareHeartbeatWatchdog => "wellfair",
            Self::GovernedVibeScriptBatch => "poet",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsJobStatus {
    Queued,
    Running,
    Blocked,
    Completed,
    Cancelled,
    Failed,
}

impl OsJobStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Blocked => "Blocked",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
            Self::Failed => "Failed",
        }
    }

    pub const fn is_active(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }

    pub const fn can_start(self) -> bool {
        matches!(self, Self::Queued | Self::Blocked)
    }
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

impl OsJobRecord {
    fn queued(job_id: String, kind: OsJobKind, title: &str, target_volume: &str) -> Self {
        let mut job = Self {
            job_id,
            kind,
            title: title.into(),
            target_volume: target_volume.into(),
            status: OsJobStatus::Queued,
            progress_pct: 0,
            detail: "Queued for background execution.".into(),
            sealed_nquin_parity: 0,
        };
        job.sealed_nquin_parity = job.parity();
        job
    }

    fn is_legacy_rehearsal(&self) -> bool {
        self.job_id.starts_with("rehearsal:") || self.detail.starts_with("Local Rehearsal")
    }

    fn parity(&self) -> u64 {
        NQuin48::from_parts(
            q_hash_60(&self.job_id),
            q_hash_60("https://webizen.org/q42#localJobRehearsal"),
            q_hash_60(self.kind.code()),
            q_hash_60(&self.target_volume),
            q_hash_60(&format!(
                "{}\0{}\0{}\0{}",
                self.status.label(),
                self.progress_pct,
                self.title,
                self.detail
            )),
        )
        .parity()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CapabilityLeaseScope {
    EtalonGpuSurface,
    QualiaQ42Storage,
    QdnfSocialNetwork,
    SanctuaryContextualDid,
    LocalP64AiInstrument,
    GovernedVibeScriptEval,
}

impl CapabilityLeaseScope {
    pub const ALL: [Self; 6] = [
        Self::EtalonGpuSurface,
        Self::QualiaQ42Storage,
        Self::QdnfSocialNetwork,
        Self::SanctuaryContextualDid,
        Self::LocalP64AiInstrument,
        Self::GovernedVibeScriptEval,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::EtalonGpuSurface => "gpu.etalon",
            Self::QualiaQ42Storage => "storage.q42",
            Self::QdnfSocialNetwork => "network.qdnf",
            Self::SanctuaryContextualDid => "sanctuary.contextual_did",
            Self::LocalP64AiInstrument => "ai.p64_instrument",
            Self::GovernedVibeScriptEval => "vibescript.governed_eval",
        }
    }

    /// The canonical capability identifier and exact legacy operation aliases.
    /// Service names that cover many operations cannot imply a specific capability.
    pub fn from_family_id(id: &str) -> Option<Self> {
        match id {
            "gpu.etalon" => Some(Self::EtalonGpuSurface),
            "storage.q42"
            | "graph.read"
            | "graph.write"
            | "qualia.graph.snapshot"
            | "qualia.localDaemon.query"
            | "qualia.wasm.execute_ntriples_query" => Some(Self::QualiaQ42Storage),
            "network.qdnf" => Some(Self::QdnfSocialNetwork),
            "sanctuary.contextual_did" => Some(Self::SanctuaryContextualDid),
            "ai.p64_instrument" => Some(Self::LocalP64AiInstrument),
            "vibescript.governed_eval" | "lease:vibescript:governed-eval" => {
                Some(Self::GovernedVibeScriptEval)
            }
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::EtalonGpuSurface => "Etalon GPU surface",
            Self::QualiaQ42Storage => "QualiaDB Q42 storage",
            Self::QdnfSocialNetwork => "QDNF social network",
            Self::SanctuaryContextualDid => "Sanctuary contextual DID",
            Self::LocalP64AiInstrument => "Local .p64 instrument",
            Self::GovernedVibeScriptEval => "Governed VibeScript evaluation",
        }
    }

    pub const fn plane_guard(self) -> &'static str {
        match self {
            Self::EtalonGpuSurface => "Plane 4 Instrument · authorized Etalon surface",
            Self::QualiaQ42Storage => "Plane 4 Instrument · scoped QualiaDB operation",
            Self::QdnfSocialNetwork => "Plane 3 SpatiotemporalHandle · scoped peer route",
            Self::SanctuaryContextualDid => "Plane 3 SpatiotemporalHandle · directional pairing",
            Self::LocalP64AiInstrument => "Plane 4 Instrument · local model scope",
            Self::GovernedVibeScriptEval => "Plane 4 Instrument · governed provider effects",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct QapCapabilityLease {
    pub package_id: String,
    pub scope: CapabilityLeaseScope,
    /// Local preference for a lease, pending any required host authorization.
    pub granted: bool,
    pub epoch_generation: u64,
    pub plane_guard: &'static str,
    pub last_audit_note: String,
    pub sealed_nquin_parity: u64,
}

impl<'de> Deserialize<'de> for QapCapabilityLease {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct StoredPreference {
            package_id: String,
            scope: CapabilityLeaseScope,
            granted: bool,
            epoch_generation: u64,
            plane_guard: String,
            last_audit_note: String,
            sealed_nquin_parity: u64,
        }

        let stored = StoredPreference::deserialize(deserializer)?;
        let plane_guard = stored.scope.plane_guard();
        if stored.plane_guard != plane_guard {
            return Err(serde::de::Error::custom("unknown capability plane guard"));
        }
        Ok(Self {
            package_id: stored.package_id,
            scope: stored.scope,
            granted: stored.granted,
            epoch_generation: stored.epoch_generation,
            plane_guard,
            last_audit_note: stored.last_audit_note,
            sealed_nquin_parity: stored.sealed_nquin_parity,
        })
    }
}

impl QapCapabilityLease {
    fn local_preference(
        package_id: &str,
        scope: CapabilityLeaseScope,
        granted: bool,
        epoch_generation: u64,
    ) -> Self {
        let mut lease = Self {
            package_id: package_id.into(),
            scope,
            granted,
            epoch_generation,
            plane_guard: scope.plane_guard(),
            last_audit_note: if granted {
                "Local allow preference recorded. Awaiting authorized activation.".into()
            } else {
                "Local deny preference recorded.".into()
            },
            sealed_nquin_parity: 0,
        };
        lease.sealed_nquin_parity = lease.parity();
        lease
    }

    fn parity(&self) -> u64 {
        NQuin48::from_parts(
            q_hash_60(&self.package_id),
            q_hash_60("https://webizen.org/q42#localQapLeasePreference"),
            q_hash_60(self.scope.code()),
            self.epoch_generation,
            q_hash_60(&format!(
                "{}\0{}\0{}",
                self.granted, self.plane_guard, self.last_audit_note
            )),
        )
        .parity()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OsNotificationSeverity {
    Info,
    ReviewRequired,
    ParaconsistentDissent,
    UrgentCare,
}

impl OsNotificationSeverity {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "Information",
            Self::ReviewRequired => "Review required",
            Self::ParaconsistentDissent => "Stewardship dissent (0x32)",
            Self::UrgentCare => "Urgent care",
        }
    }
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

impl OsSystemNotification {
    fn stewardship(
        kind: &str,
        record_id: &str,
        severity: OsNotificationSeverity,
        source_volume: &str,
        title: &str,
        detail: &str,
    ) -> Self {
        let digest = Sha3_256::digest(record_id.as_bytes());
        let mut notice = Self {
            notification_id: format!("stewardship:{kind}:{digest:x}"),
            severity,
            source_volume: source_volume.into(),
            title: title.into(),
            detail: detail.into(),
            acknowledged: false,
            sealed_nquin_parity: 0,
        };
        notice.sealed_nquin_parity = notice.parity();
        notice
    }

    fn parity(&self) -> u64 {
        NQuin48::from_parts(
            q_hash_60(&self.notification_id),
            q_hash_60("https://webizen.org/q42#osNotification"),
            q_hash_60(self.severity.label()),
            q_hash_60(&self.source_volume),
            q_hash_60(&format!(
                "{}\0{}\0{}",
                self.acknowledged, self.title, self.detail
            )),
        )
        .parity()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OsControlCenterState {
    pub jobs: Vec<OsJobRecord>,
    pub leases: Vec<QapCapabilityLease>,
    pub notifications: Vec<OsSystemNotification>,
    generation: u64,
}

impl Default for OsControlCenterState {
    fn default() -> Self {
        let jobs = Vec::new();
        let leases = QAPS
            .into_iter()
            .flat_map(|package_id| {
                CapabilityLeaseScope::ALL.into_iter().map(move |scope| {
                    QapCapabilityLease::local_preference(package_id, scope, false, 0)
                })
            })
            .collect();

        Self {
            jobs,
            leases,
            notifications: Vec::new(),
            generation: 0,
        }
    }
}

impl OsControlCenterState {
    /// Seed capability rows in memory when no snapshot exists. A corrupt snapshot stays on disk.
    pub fn load_or_seed(
        storage: &Path,
        fabric: &NaturalHostStewardshipFabric,
    ) -> Result<Self, String> {
        let mut state = read_snapshot(&storage.join(OS_CONTROL_CENTER_REL_PATH))
            .map_err(|error| format!("Read OS control center: {error}"))?
            .unwrap_or_default();
        state.sync_notifications_from_stewardship(fabric);
        Ok(state)
    }

    pub(crate) fn read_qap_policy(storage: &Path) -> Result<Self, String> {
        read_snapshot(&storage.join(OS_CONTROL_CENTER_REL_PATH))
            .map(|state| state.unwrap_or_default())
            .map_err(|error| format!("Read Qap capability policy: {error}"))
    }

    /// Generation zero is an untouched seed row. It does not hide a signed grant.
    pub fn qap_lease_revoked(&self, package_id: &str, scope: CapabilityLeaseScope) -> bool {
        self.leases.iter().any(|lease| {
            lease.package_id == package_id
                && lease.scope == scope
                && lease.epoch_generation != 0
                && !lease.granted
        })
    }

    pub fn active_job_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|job| job.status.is_active())
            .count()
    }

    pub fn notification_count(&self) -> usize {
        self.notifications
            .iter()
            .filter(|notice| !notice.acknowledged)
            .count()
    }

    pub fn enqueue_os_job(
        &mut self,
        storage: &Path,
        kind: OsJobKind,
        title: &str,
        target_volume: &str,
    ) -> Result<OsJobRecord, String> {
        let title = title.trim();
        let target_volume = target_volume.trim();
        validate_label(title, "Job name", 140)?;
        validate_label(target_volume, "Target volume", 80)?;
        self.commit(storage, |next| {
            if next.jobs.len() >= MAX_ROWS {
                return Err("The local job history is full.".into());
            }
            let job = OsJobRecord::queued(
                format!("job:{}", next.generation),
                kind,
                title,
                target_volume,
            );
            next.jobs.push(job.clone());
            Ok(job)
        })
    }

    /// Kept for existing callers. Progress and completion come from the background runner.
    pub fn advance_or_complete_os_job(
        &mut self,
        _storage: &Path,
        _job_id: &str,
    ) -> Result<OsJobRecord, String> {
        Err("Start the job through the OS background runner.".into())
    }

    pub fn cancel_os_job(&mut self, storage: &Path, job_id: &str) -> Result<OsJobRecord, String> {
        self.update_job(storage, job_id, cancel_unstarted_job)
    }

    pub(crate) fn fresh_background_job(storage: &Path, id: &str) -> Result<OsJobRecord, String> {
        Self::find_background_job(storage, id)?
            .ok_or_else(|| "Job is absent from the persisted queue.".into())
    }

    pub(crate) fn find_background_job(
        storage: &Path,
        id: &str,
    ) -> Result<Option<OsJobRecord>, String> {
        Ok(read_snapshot(&storage.join(OS_CONTROL_CENTER_REL_PATH))
            .map_err(|error| format!("Read OS control center: {error}"))?
            .and_then(|center| center.jobs.into_iter().find(|job| job.job_id == id)))
    }

    pub(crate) fn claim_background_job(storage: &Path, id: &str) -> Result<OsJobRecord, String> {
        Self::background_job_edit(storage, id, |job| {
            if !job.status.can_start() || job.is_legacy_rehearsal() {
                return Err("Only a queued or blocked background job can start.".into());
            }
            job.status = OsJobStatus::Running;
            job.progress_pct = 0;
            job.detail = "Background worker started.".into();
            Ok(())
        })
    }

    pub(crate) fn cancel_unstarted_background_job(
        storage: &Path,
        id: &str,
    ) -> Result<OsJobRecord, String> {
        Self::background_job_edit(storage, id, cancel_unstarted_job)
    }

    pub(crate) fn finish_background_job(
        storage: &Path,
        id: &str,
        status: OsJobStatus,
        detail: &str,
    ) -> Result<OsJobRecord, String> {
        if status.is_active() {
            return Err("A background operation must report a final or blocked status.".into());
        }
        validate_label(detail, "Job status", 512)?;
        Self::background_job_edit(storage, id, |job| {
            if job.status != OsJobStatus::Running {
                return Err("Job no longer belongs to the background worker.".into());
            }
            job.status = status;
            job.detail = detail.into();
            if status == OsJobStatus::Completed {
                job.progress_pct = 100;
            }
            Ok(())
        })
    }

    /// The caller holds the per-storage runner lock, so no previous shell worker
    /// can still complete a row marked Running.
    pub(crate) fn recover_background_jobs(storage: &Path) -> Result<(), String> {
        let recovered = |job: &OsJobRecord| {
            job.status == OsJobStatus::Running
                || (job.status.can_start() && job.is_legacy_rehearsal())
        };
        let snapshot = read_snapshot(&storage.join(OS_CONTROL_CENTER_REL_PATH))
            .map_err(|error| format!("Read OS control center: {error}"))?;
        if !snapshot.is_some_and(|state| state.jobs.iter().any(recovered)) {
            return Ok(());
        }
        Self::retry_background_commit(storage, |center| {
            for job in center.jobs.iter_mut().filter(|job| recovered(job)) {
                if job.is_legacy_rehearsal() {
                    job.status = OsJobStatus::Cancelled;
                    job.detail = "Legacy rehearsal · never dispatched.".into();
                } else {
                    job.status = OsJobStatus::Failed;
                    job.detail =
                        "Worker interrupted. The previous operation may have produced output."
                            .into();
                }
                job.sealed_nquin_parity = job.parity();
            }
            Ok(())
        })
    }

    pub(crate) fn refresh_background_jobs(&mut self, storage: &Path) -> Result<bool, String> {
        let Some(mut disk) = read_snapshot(&storage.join(OS_CONTROL_CENTER_REL_PATH))
            .map_err(|error| format!("Read OS control center: {error}"))?
        else {
            return Ok(false);
        };
        if disk.generation <= self.generation {
            return Ok(false);
        }
        // Derived stewardship notices may have been added or removed in memory
        // since the last disk commit. Preserve the live projection and any
        // acknowledgments already committed by another control-center writer.
        let acknowledgements: BTreeSet<_> = disk
            .notifications
            .iter()
            .filter(|notice| notice.acknowledged)
            .map(|notice| notice.notification_id.clone())
            .collect();
        disk.notifications
            .retain(|notice| !notice.notification_id.starts_with("stewardship:"));
        let available = MAX_ROWS.saturating_sub(disk.notifications.len());
        disk.notifications.extend(
            self.notifications
                .iter()
                .filter(|notice| notice.notification_id.starts_with("stewardship:"))
                .take(available)
                .cloned()
                .map(|mut notice| {
                    notice.acknowledged = acknowledgements.contains(&notice.notification_id);
                    notice.sealed_nquin_parity = notice.parity();
                    notice
                }),
        );
        *self = disk;
        Ok(true)
    }

    fn background_job_edit(
        storage: &Path,
        id: &str,
        edit: impl Fn(&mut OsJobRecord) -> Result<(), String>,
    ) -> Result<OsJobRecord, String> {
        Self::retry_background_commit(storage, |center| {
            let job = center
                .jobs
                .iter_mut()
                .find(|job| job.job_id == id)
                .ok_or("Job is absent from the local queue.")?;
            edit(job)?;
            job.sealed_nquin_parity = job.parity();
            Ok(job.clone())
        })
    }

    fn retry_background_commit<T>(
        storage: &Path,
        edit: impl Fn(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        // The control-center lock protects disk generation and compare-and-
        // replace. A busy/stale snapshot is safe to retry because the edit
        // above touches only the row owned by this job.
        for attempt in 0..10 {
            let mut state = read_snapshot(&storage.join(OS_CONTROL_CENTER_REL_PATH))
                .map_err(|error| format!("Read OS control center: {error}"))?
                .unwrap_or_default();
            match state.commit(storage, &edit) {
                Err(error)
                    if attempt < 9
                        && (error
                            .starts_with("Another session is saving the OS control center")
                            || error.starts_with(
                                "The OS control center changed in another session",
                            )) =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(15))
                }
                result => return result,
            }
        }
        unreachable!("the last attempt always returns")
    }

    fn update_job(
        &mut self,
        storage: &Path,
        job_id: &str,
        update: impl FnOnce(&mut OsJobRecord) -> Result<(), String>,
    ) -> Result<OsJobRecord, String> {
        self.commit(storage, |next| {
            let job = next
                .jobs
                .iter_mut()
                .find(|job| job.job_id == job_id)
                .ok_or("Job is absent from the local queue.")?;
            update(job)?;
            job.sealed_nquin_parity = job.parity();
            Ok(job.clone())
        })
    }

    /// Change the local matrix. A host-authorized lease remains a separate decision.
    pub fn set_qap_lease(
        &mut self,
        storage: &Path,
        package_id: &str,
        scope: CapabilityLeaseScope,
        granted: bool,
    ) -> Result<QapCapabilityLease, String> {
        let package_id = package_id.trim();
        validate_label(package_id, "Qap package", 128)?;
        validate_package_id(package_id).map_err(|error| error.to_string())?;

        self.commit(storage, |next| {
            let lease =
                QapCapabilityLease::local_preference(package_id, scope, granted, next.generation);
            if let Some(previous) = next
                .leases
                .iter_mut()
                .find(|previous| previous.package_id == package_id && previous.scope == scope)
            {
                previous.clone_from(&lease);
            } else {
                let registry = load_install_registry(storage).map_err(|error| error.to_string())?;
                let installed = registry
                    .packages
                    .get(package_id)
                    .is_some_and(|entry| !entry.revoked);
                if !installed && !QAPS.contains(&package_id) {
                    return Err("Choose an installed Qap or a built-in volume.".into());
                }
                if next.leases.len() >= MAX_ROWS {
                    return Err("The local Qap matrix is full.".into());
                }
                next.leases.push(lease.clone());
            }
            Ok(lease)
        })
    }

    /// Mirror unresolved stewardship notices without copying private care or vote text.
    pub fn sync_notifications_from_stewardship(&mut self, fabric: &NaturalHostStewardshipFabric) {
        let acknowledged: BTreeSet<_> = self
            .notifications
            .iter()
            .filter(|notice| notice.acknowledged)
            .map(|notice| notice.notification_id.clone())
            .collect();
        self.notifications
            .retain(|notice| !notice.notification_id.starts_with("stewardship:"));
        let mut derived = BTreeMap::new();

        for alert in fabric.care_alerts.iter().filter(|alert| {
            !alert.resolved_or_reversed && alert.severity >= CareAlertSeverity::Urgent
        }) {
            let notice = OsSystemNotification::stewardship(
                "care",
                &alert.alert_id,
                OsNotificationSeverity::UrgentCare,
                "wellfair",
                "Urgent care alert",
                "Open WellFair to review the active care alert.",
            );
            derived.insert(notice.notification_id.clone(), notice);
        }

        for proposal in fabric.update_queue.iter().filter(|proposal| {
            !proposal.applied_to_host
                && (proposal.paraconsistent_review_flagged
                    || proposal.votes.iter().any(|vote| !vote.approve))
        }) {
            let notice = OsSystemNotification::stewardship(
                "dissent",
                &proposal.proposal_id,
                OsNotificationSeverity::ParaconsistentDissent,
                "admin",
                "Stewardship review required",
                "An unresolved dissent needs human review in Admin.",
            );
            derived.insert(notice.notification_id.clone(), notice);
        }

        let mut derived: Vec<_> = derived.into_values().collect();
        derived.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.notification_id.cmp(&b.notification_id))
        });
        let available = MAX_ROWS.saturating_sub(self.notifications.len());
        self.notifications
            .extend(derived.into_iter().take(available).map(|mut notice| {
                notice.acknowledged = acknowledged.contains(&notice.notification_id);
                notice.sealed_nquin_parity = notice.parity();
                notice
            }));
        self.notifications.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.notification_id.cmp(&b.notification_id))
        });
    }

    /// Mark the drawer notice read. The source care alert or dissent remains independent.
    pub fn acknowledge_notification(
        &mut self,
        storage: &Path,
        notification_id: &str,
    ) -> Result<(), String> {
        self.commit(storage, |next| {
            let notice = next
                .notifications
                .iter_mut()
                .find(|notice| notice.notification_id == notification_id)
                .ok_or("Notification is no longer active.")?;
            notice.acknowledged = true;
            notice.sealed_nquin_parity = notice.parity();
            Ok(())
        })
    }

    fn commit<T>(
        &mut self,
        storage: &Path,
        edit: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        let path = storage.join(OS_CONTROL_CENTER_REL_PATH);
        let parent = path
            .parent()
            .ok_or("The OS control center has no storage directory.")?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let lock = snapshot_options()
            .create(true)
            .truncate(false)
            .open(path.with_extension("lock"))
            .map_err(|error| format!("Open OS control center lock: {error}"))?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                return Err("Another session is saving the OS control center. Try again.".into())
            }
            Err(TryLockError::Error(error)) => {
                return Err(format!("Lock OS control center: {error}"))
            }
        }

        let disk_generation = read_snapshot(&path)
            .map_err(|error| format!("Read OS control center: {error}"))?
            .map_or(0, |current| current.generation);
        if disk_generation != self.generation {
            return Err("The OS control center changed in another session. Reload the session before editing.".into());
        }

        let mut next = self.clone();
        next.generation = self
            .generation
            .checked_add(1)
            .ok_or("The OS control center generation is exhausted.")?;
        let result = edit(&mut next)?;
        next.validate().map_err(|error| error.to_string())?;
        write_snapshot(&path, &next).map_err(|error| format!("Save OS control center: {error}"))?;
        *self = next;
        Ok(result)
    }

    fn validate(&self) -> io::Result<()> {
        if self.jobs.len() > MAX_ROWS
            || self.leases.len() > MAX_ROWS
            || self.notifications.len() > MAX_ROWS
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "too many OS control center records",
            ));
        }
        let mut job_ids = BTreeSet::new();
        let mut lease_keys = BTreeSet::new();
        let mut notice_ids = BTreeSet::new();
        let valid_jobs = self.jobs.iter().all(|job| {
            job_ids.insert(job.job_id.as_str())
                && job.progress_pct <= 100
                && job.sealed_nquin_parity == job.parity()
                && validate_label(&job.title, "Job name", 140).is_ok()
                && validate_label(&job.target_volume, "Target volume", 80).is_ok()
        });
        let valid_leases = self.leases.iter().all(|lease| {
            lease_keys.insert((lease.package_id.as_str(), lease.scope))
                && lease.plane_guard == lease.scope.plane_guard()
                && lease.epoch_generation <= self.generation
                && lease.sealed_nquin_parity == lease.parity()
                && validate_package_id(&lease.package_id).is_ok()
                && validate_label(&lease.package_id, "Qap package", 128).is_ok()
        });
        let valid_notifications = self.notifications.iter().all(|notice| {
            notice_ids.insert(notice.notification_id.as_str())
                && notice.sealed_nquin_parity == notice.parity()
                && validate_human_centric_host_nomenclature(&notice.title).is_ok()
                && validate_human_centric_host_nomenclature(&notice.detail).is_ok()
        });
        if valid_jobs && valid_leases && valid_notifications {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid or duplicate OS control center record",
            ))
        }
    }
}

fn cancel_unstarted_job(job: &mut OsJobRecord) -> Result<(), String> {
    if !job.status.can_start() {
        return Err("Only a queued or blocked job can be cancelled from the registry.".into());
    }
    job.status = OsJobStatus::Cancelled;
    job.detail = "Cancelled before execution.".into();
    Ok(())
}

fn validate_label(value: &str, label: &str, max_bytes: usize) -> Result<(), String> {
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return Err(format!(
            "{label} is empty, too long, or contains a control character."
        ));
    }
    validate_human_centric_host_nomenclature(value)
}

fn read_snapshot(path: &Path) -> io::Result<Option<OsControlCenterState>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && metadata.len() <= MAX_SNAPSHOT_BYTES => {}
        Ok(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid snapshot file or size",
            ))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_SNAPSHOT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "snapshot exceeds the storage limit",
        ));
    }
    let state: OsControlCenterState = serde_json::from_slice(&bytes)?;
    state.validate()?;
    Ok(Some(state))
}

fn snapshot_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn write_snapshot(path: &Path, state: &OsControlCenterState) -> io::Result<()> {
    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let bytes = serde_json::to_vec_pretty(state)?;
    if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "snapshot exceeds the storage limit",
        ));
    }
    let temporary = path.with_extension(format!(
        "json.{}.{}.tmp",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed),
    ));
    let mut file = snapshot_options().create_new(true).open(&temporary)?;
    let written = file.write_all(&bytes).and_then(|()| file.sync_all());
    drop(file);
    if let Err(error) = written.and_then(|()| fs::rename(&temporary, path)) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manages_os_jobs_qap_capability_leases_and_stewardship_notifications() {
        let storage =
            std::env::temp_dir().join(format!("webizen-os-control-center-{}", std::process::id()));
        fs::create_dir(&storage).unwrap();

        let mut fabric = NaturalHostStewardshipFabric::default();
        let alert = &mut fabric.care_alerts[0];
        alert.severity = CareAlertSeverity::Urgent;
        alert.resolved_or_reversed = false;
        let private_summary = alert.summary.clone();
        fabric.update_queue[0].paraconsistent_review_flagged = true;

        let mut center = OsControlCenterState::load_or_seed(&storage, &fabric).unwrap();
        let mut stale = center.clone();
        assert_eq!(center.notification_count(), 2);
        assert_eq!(
            center.notifications[0].severity,
            OsNotificationSeverity::UrgentCare
        );
        assert!(center
            .notifications
            .iter()
            .all(|notice| notice.detail != private_summary));
        assert_eq!(
            center.leases.len(),
            QAPS.len() * CapabilityLeaseScope::ALL.len()
        );
        assert!(center.leases.iter().all(|lease| !lease.granted));

        let job = center
            .enqueue_os_job(&storage, OsJobKind::ShamirQ42jBackup, "Backup", "settings")
            .unwrap();
        let stale_id = job.job_id.clone();
        assert!(stale.cancel_os_job(&storage, &stale_id).is_err());
        assert!(center
            .advance_or_complete_os_job(&storage, &job.job_id)
            .is_err());
        let current = OsControlCenterState::claim_background_job(&storage, &job.job_id).unwrap();
        assert_eq!(current.status, OsJobStatus::Running);
        assert!(center.refresh_background_jobs(&storage).unwrap());
        assert!(center.cancel_os_job(&storage, &job.job_id).is_err());
        let current = OsControlCenterState::finish_background_job(
            &storage,
            &job.job_id,
            OsJobStatus::Completed,
            "Worker receipt recorded.",
        )
        .unwrap();
        assert_eq!(current.status, OsJobStatus::Completed);
        assert_eq!(current.progress_pct, 100);
        assert!(center.refresh_background_jobs(&storage).unwrap());

        let lease = center
            .set_qap_lease(
                &storage,
                "Poet",
                CapabilityLeaseScope::GovernedVibeScriptEval,
                true,
            )
            .unwrap();
        assert!(lease.granted);
        assert!(lease
            .last_audit_note
            .contains("Awaiting authorized activation"));
        let revoked = center
            .set_qap_lease(&storage, "Poet", lease.scope, false)
            .unwrap();
        assert!(!revoked.granted);
        assert!(revoked.epoch_generation > lease.epoch_generation);

        let urgent_id = center.notifications[0].notification_id.clone();
        center
            .acknowledge_notification(&storage, &urgent_id)
            .unwrap();
        assert_eq!(center.notification_count(), 1);
        assert!(!fabric.care_alerts[0].resolved_or_reversed);

        let reloaded = OsControlCenterState::load_or_seed(&storage, &fabric).unwrap();
        assert_eq!(reloaded, center);
        fabric.care_alerts[0].resolved_or_reversed = true;
        center.sync_notifications_from_stewardship(&fabric);
        assert!(center
            .notifications
            .iter()
            .all(|notice| notice.notification_id != urgent_id));
        let snapshot = storage.join(OS_CONTROL_CENTER_REL_PATH);
        let mut encoded: serde_json::Value =
            serde_json::from_slice(&fs::read(&snapshot).unwrap()).unwrap();
        encoded["jobs"][0]["sealed_nquin_parity"] =
            serde_json::json!(center.jobs[0].sealed_nquin_parity ^ 1);
        let corrupt = serde_json::to_vec(&encoded).unwrap();
        fs::write(&snapshot, &corrupt).unwrap();
        assert!(OsControlCenterState::load_or_seed(&storage, &fabric).is_err());
        assert!(center.cancel_os_job(&storage, &stale_id).is_err());
        assert_eq!(fs::read(&snapshot).unwrap(), corrupt);

        fs::remove_file(snapshot.with_extension("lock")).unwrap();
        fs::remove_file(snapshot).unwrap();
        fs::remove_dir(storage.join("wellfair")).unwrap();
        fs::remove_dir(storage).unwrap();
    }
}
