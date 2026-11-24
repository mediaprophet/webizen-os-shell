// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! The shell-owned, per-storage OS worker. Inputs which carry a live authority
//! or select a private source remain in memory. Restarts never replay them.

use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions, TryLockError},
    io::{Read, Write},
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use qualia_core_db::p64_weight::{compile_gguf_to_p64, IntegrityMode, P64TensorIndex};

use crate::{
    host_daemon_runtime::{run_daemon_tick, CareHeartbeatStatus, HostDaemonConfig},
    host_stewardship::{
        generate_tri_target_debian_suite, validate_human_centric_host_nomenclature,
        NaturalHostStewardshipFabric, HOST_STEWARDSHIP_STORE_REL_PATH,
    },
    native_gpu::NativeEtalonStatus,
    os_jobs_leases::{OsControlCenterState, OsJobKind, OsJobStatus},
    os_qcdn_mirror::sync_local_qcdn_mirror,
    portability_qecp::export_shamir_q42j_backup,
    qualia_host_foundation::AdoptedQualiaFoundation,
    session::{DeviceSession, LoopbackHost},
};

const MAX_STEWARDSHIP_BYTES: u64 = 4 * 1024 * 1024;
const MAX_GGUF_BYTES: u64 = 512 * 1024 * 1024;
const MAX_GGUF_TENSORS: u64 = 4096;
const MAX_GGUF_KEYS: u64 = 65_536;
const MAX_WAITING_JOBS: usize = 8;
const MAX_SOURCE_BYTES: usize = 64 * 1024;

/// Native owner-selected inputs. None is taken from page data or persisted.
pub enum OsJobInput {
    Standard,
    Gguf {
        source: PathBuf,
    },
    QcdnMirror {
        project_id: String,
    },
    GovernedVibe {
        foundation: AdoptedQualiaFoundation,
        source: String,
    },
}

#[derive(Clone)]
pub struct OsJobRunner {
    owner: Arc<RunnerOwner>,
}

struct RunnerOwner {
    host: LoopbackHost,
    gpu: NativeEtalonStatus,
    _storage_lock: File,
    queue: Mutex<WorkerQueue>,
    error: Mutex<Option<String>>,
    revision: AtomicU64,
    seen: AtomicU64,
    wake: Box<dyn Fn() + Send + Sync>,
}

#[derive(Default)]
struct WorkerQueue {
    active: Option<ActiveJob>,
    waiting: VecDeque<JobRequest>,
}

struct JobRequest {
    id: String,
    input: OsJobInput,
}

struct ActiveJob {
    id: String,
    cancelled: Arc<AtomicBool>,
    executing: bool,
}

impl OsJobRunner {
    /// Adopt exclusive background execution for one user storage directory.
    /// An interrupted predecessor is reconciled before newly queued work starts.
    pub fn new(
        session: &DeviceSession,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, String> {
        Self::for_host(session.host.clone(), session.gpu.clone(), wake)
    }

    fn for_host(
        host: LoopbackHost,
        gpu: NativeEtalonStatus,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let root = host.storage.join("wellfair");
        fs::create_dir_all(&root).map_err(|error| format!("Create job storage: {error}"))?;
        let lock_path = root.join("os-job-runner.lock");
        if let Ok(meta) = fs::symlink_metadata(&lock_path) {
            if !meta.is_file() {
                return Err("The job runner lock must be a regular file.".into());
            }
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)
            .map_err(|error| format!("Open job runner lock: {error}"))?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                return Err("Another shell owns this storage's background queue.".into());
            }
            Err(TryLockError::Error(error)) => {
                return Err(format!("Lock the background queue: {error}"));
            }
        }
        OsControlCenterState::recover_background_jobs(&host.storage)?;
        Ok(Self {
            owner: Arc::new(RunnerOwner {
                host,
                gpu,
                _storage_lock: lock,
                queue: Mutex::new(WorkerQueue::default()),
                error: Mutex::new(None),
                revision: AtomicU64::new(1),
                seen: AtomicU64::new(0),
                wake: Box::new(wake),
            }),
        })
    }

    /// Start one eligible persisted row. The native handler returns after
    /// admitting the worker thread, while the thread performs the operation.
    pub fn start(&self, job_id: &str, input: OsJobInput) -> Result<(), String> {
        if let Some(error) = self.owner.worker_error()? {
            return Err(error);
        }
        let bounded = match &input {
            OsJobInput::Standard => true,
            OsJobInput::Gguf { source } => source.as_os_str().len() <= 4096,
            OsJobInput::QcdnMirror { project_id } => project_id.len() <= 256,
            OsJobInput::GovernedVibe { source, .. } => source.len() <= MAX_SOURCE_BYTES,
        };
        if !bounded {
            return Err("The selected job input exceeds its submission limit.".into());
        }
        let job = OsControlCenterState::fresh_background_job(&self.owner.host.storage, job_id)?;
        if !job.status.can_start() {
            return Err("Only a queued or blocked job can start.".into());
        }
        let mut queue = self
            .owner
            .queue
            .lock()
            .map_err(|_| "Job worker unavailable.")?;
        if queue
            .active
            .as_ref()
            .is_some_and(|active| active.id == job.job_id)
            || queue.waiting.iter().any(|pending| pending.id == job.job_id)
        {
            return Err("This job is already submitted to the background worker.".into());
        }
        let request = JobRequest {
            id: job.job_id,
            input,
        };
        if queue.active.is_some() {
            if queue.waiting.len() >= MAX_WAITING_JOBS {
                return Err("The background submission queue is full.".into());
            }
            queue.waiting.push_back(request);
            return Ok(());
        }
        // Refuse even if the worker's fatal error arrived while we read disk.
        if let Some(error) = self.owner.worker_error()? {
            return Err(error);
        }
        let cancelled = claim_worker(&mut queue, &request.id);
        let owner = Arc::clone(&self.owner);
        if let Err(error) = std::thread::Builder::new()
            .name("webizen-os-job".into())
            .spawn(move || owner.run(request, cancelled))
        {
            queue.active = None;
            return Err(format!("Could not start the background worker: {error}"));
        }
        Ok(())
    }

    /// Cancel before entering an operation. Refuse cancellation after work
    /// starts so a published artifact cannot be reported as cancelled.
    pub fn cancel(&self, job_id: &str) -> Result<(), String> {
        let mut queue = self
            .owner
            .queue
            .lock()
            .map_err(|_| "Job worker unavailable.")?;
        if let Some(job) = queue.active.as_ref().filter(|running| running.id == job_id) {
            if job.executing {
                return Err("This operation has started and cannot safely be interrupted.".into());
            }
            job.cancelled.store(true, Ordering::Release);
            return Ok(());
        }
        OsControlCenterState::cancel_unstarted_background_job(&self.owner.host.storage, job_id)?;
        queue.waiting.retain(|pending| pending.id != job_id);
        drop(queue);
        self.owner.changed();
        Ok(())
    }

    /// Called by the native window after a worker wake. A regular redraw only
    /// reads atomics. When changed, the persisted snapshot includes concurrent
    /// lease and acknowledgment updates as well as the job's new status.
    pub fn refresh(&self, center: &mut OsControlCenterState) -> Result<bool, String> {
        if let Some(error) = self.owner.worker_error()? {
            return Err(error);
        }
        let revision = self.owner.revision.load(Ordering::Acquire);
        if self.owner.seen.load(Ordering::Acquire) == revision {
            return Ok(false);
        }
        let applied = center.refresh_background_jobs(&self.owner.host.storage)?;
        self.owner.seen.store(revision, Ordering::Release);
        Ok(applied)
    }
}

impl RunnerOwner {
    fn worker_error(&self) -> Result<Option<String>, String> {
        self.error
            .lock()
            .map(|report| report.clone())
            .map_err(|_| "Job worker unavailable.".into())
    }

    fn changed(&self) {
        self.revision.fetch_add(1, Ordering::Release);
        let _ = catch_unwind(AssertUnwindSafe(|| (self.wake)()));
    }

    fn run(&self, mut request: JobRequest, mut cancelled: Arc<AtomicBool>) {
        loop {
            let disposition = self.run_job(&request.id, request.input, cancelled);
            let Ok(mut queue) = self.queue.lock() else {
                return;
            };
            queue.active = None;
            if let Err(error) = disposition {
                if let Ok(mut report) = self.error.lock() {
                    *report = Some(error);
                }
                // Work may have committed. Halt dispatch and drop remaining
                // private inputs; their persisted rows remain available for
                // explicit resubmission once the session is recovered.
                queue.waiting.clear();
                drop(queue);
                self.changed();
                return;
            }
            let next = queue.waiting.pop_front();
            let next = next.map(|job| {
                let token = claim_worker(&mut queue, &job.id);
                (job, token)
            });
            drop(queue);
            self.changed();
            let Some((next_request, next_token)) = next else {
                break;
            };
            request = next_request;
            cancelled = next_token;
        }
    }

    fn run_job(
        &self,
        id: &str,
        input: OsJobInput,
        cancelled: Arc<AtomicBool>,
    ) -> Result<(), String> {
        let storage = &self.host.storage;
        let prepare = if cancelled.load(Ordering::Acquire) {
            OsControlCenterState::cancel_unstarted_background_job(storage, id).map(|_| None)
        } else {
            OsControlCenterState::claim_background_job(storage, id).map(Some)
        };
        let job = match prepare {
            Ok(Some(job)) => job,
            Ok(None) => return Ok(()),
            Err(error) => {
                // A waiting item may be cancelled by a separate owner using
                // the persisted registry. A verified terminal or removed
                // row needs no work, while an unreadable or still-eligible
                // row keeps the original failure visible.
                let current = OsControlCenterState::find_background_job(storage, id)?;
                if current.is_none_or(|row| !row.status.can_start()) {
                    return Ok(());
                }
                return Err(error);
            }
        };
        self.changed();
        {
            let mut runner = self.queue.lock().map_err(|_| "Job worker unavailable.")?;
            if cancelled.load(Ordering::Acquire) {
                OsControlCenterState::finish_background_job(
                    storage,
                    id,
                    OsJobStatus::Cancelled,
                    "Cancelled before execution.",
                )?;
                return Ok(());
            }
            runner
                .active
                .as_mut()
                .ok_or("Job worker lost its claim.")?
                .executing = true;
        }

        let outcome = catch_unwind(AssertUnwindSafe(|| {
            self.execute(job.kind, id, input, cancelled)
        }))
        .unwrap_or_else(|_| {
            Err(JobError::Failed(
                "The worker stopped unexpectedly. An operation may have produced output.".into(),
            ))
        });
        let (status, detail) = match outcome {
            Ok(detail) => (OsJobStatus::Completed, detail),
            Err(JobError::Blocked(detail)) => (OsJobStatus::Blocked, detail),
            Err(JobError::Failed(detail)) => (OsJobStatus::Failed, detail),
        };
        OsControlCenterState::finish_background_job(storage, id, status, &bounded_detail(detail))
            .map_err(|error| format!("Could not record the background job outcome: {error}"))?;
        Ok(())
    }

    fn execute(
        &self,
        kind: OsJobKind,
        id: &str,
        input: OsJobInput,
        cancelled: Arc<AtomicBool>,
    ) -> Result<String, JobError> {
        let storage = &self.host.storage;
        match (kind, input) {
            (OsJobKind::TriTargetDebianBuild, OsJobInput::Standard) => {
                let stewardship = current_stewardship(storage)?;
                let receipt =
                    generate_tri_target_debian_suite(&self.host, &self.gpu, &stewardship)
                        .map_err(JobError::Failed)?;
                Ok(format!(
                    "{} Debian packages built in delivery/tri-target-debian-trixie.",
                    receipt.built_deb_packages.len(),
                ))
            }
            (OsJobKind::ShamirQ42jBackup, OsJobInput::Standard) => {
                let receipt = export_shamir_q42j_backup(storage, 2, 3).map_err(JobError::Failed)?;
                Ok(format!(
                    "Encrypted 2-of-3 metadata backup exported under backups/q42j ({}).",
                    receipt.archive_id
                ))
            }
            (OsJobKind::CareHeartbeatWatchdog, OsJobInput::Standard) => {
                let tick = run_daemon_tick(&HostDaemonConfig::for_storage(storage))
                    .map_err(JobError::Failed)?;
                if !tick.status.stewardship_persisted {
                    return Err(JobError::Blocked(
                        "A provisioned stewardship ledger is required for a live care check.".into(),
                    ));
                }
                let lapsed = tick.care_heartbeats.iter()
                    .filter(|item| item.status == CareHeartbeatStatus::Lapsed).count();
                let unknown = tick.care_heartbeats.iter().filter(|item|
                    matches!(item.status, CareHeartbeatStatus::NoObservation | CareHeartbeatStatus::ClockUncertain)
                ).count();
                Ok(format!(
                    "Evaluated {} care heartbeat windows: {lapsed} lapsed, {unknown} without usable evidence. No alerts dispatched.",
                    tick.care_heartbeats.len()
                ))
            }
            (OsJobKind::GgufToP64Conversion, OsJobInput::Gguf { source }) => {
                compile_model_for_job(storage, id, &source)
            }
            (OsJobKind::QcdnSwarmMerkleSync, OsJobInput::QcdnMirror { project_id }) => {
                if std::env::var_os("WEBIZEN_QCDN_MIRROR_CONFIG")
                    .is_none_or(|path| path.is_empty())
                {
                    return Err(JobError::Blocked(
                        "An operator must configure an authorized local Q-CDN mirror.".into(),
                    ));
                }
                sync_local_qcdn_mirror(storage, &project_id, &cancelled).map_err(JobError::Failed)
            }
            (OsJobKind::GovernedVibeScriptBatch, OsJobInput::GovernedVibe { foundation, source }) => {
                let report = foundation.run_qap_source(storage, id, &source, cancelled)
                    .map_err(|_| JobError::Blocked(
                        "Poet must admit this exact source under the current signed graph scope and Qap policy.".into()
                    ))?;
                // Neither protected source text nor a private result belongs in
                // the shared control-center snapshot.
                if report.reconciliation_required {
                    return Err(JobError::Failed(
                        "The signed action stopped after a possible commit. Review the project journal before retrying.".into(),
                    ));
                }
                if report.diagnostic.is_some() {
                    return Err(JobError::Failed("The signed VibeScript action did not complete.".into()));
                }
                Ok(format!(
                    "Signed VibeScript action completed. {} verified runtime receipts.",
                    report.receipts.len()
                ))
            }
            (OsJobKind::GgufToP64Conversion, _) => Err(JobError::Blocked(
                "Select a staged GGUF source before starting this job.".into(),
            )),
            (OsJobKind::QcdnSwarmMerkleSync, _) => Err(JobError::Blocked(
                "Select a project with an operator-configured Q-CDN mirror before starting this job.".into(),
            )),
            (OsJobKind::GovernedVibeScriptBatch, _) => Err(JobError::Blocked(
                "Sign into the native project and supply the exact admitted VibeScript source.".into(),
            )),
            _ => Err(JobError::Blocked("This job requires its standard input.".into())),
        }
    }
}

fn claim_worker(queue: &mut WorkerQueue, id: &str) -> Arc<AtomicBool> {
    let cancelled = Arc::new(AtomicBool::new(false));
    queue.active = Some(ActiveJob {
        id: id.into(),
        cancelled: Arc::clone(&cancelled),
        executing: false,
    });
    cancelled
}

enum JobError {
    Blocked(String),
    Failed(String),
}

fn bounded_detail(detail: String) -> String {
    const MAX_JOB_DETAIL_BYTES: usize = 256;
    let mut display = String::new();
    for character in detail.chars() {
        let character = if character.is_control() {
            ' '
        } else {
            character
        };
        if display.len() + character.len_utf8() > MAX_JOB_DETAIL_BYTES - '…'.len_utf8() {
            display.push('…');
            break;
        }
        display.push(character);
    }
    if display.trim().is_empty() || validate_human_centric_host_nomenclature(&display).is_err() {
        "The background operation reported an error without a displayable detail.".into()
    } else {
        display
    }
}

fn current_stewardship(storage: &Path) -> Result<NaturalHostStewardshipFabric, JobError> {
    let path = storage.join(HOST_STEWARDSHIP_STORE_REL_PATH);
    let metadata = fs::symlink_metadata(&path).map_err(|_| {
        JobError::Blocked("A readable local stewardship ledger is required.".into())
    })?;
    if !metadata.is_file() || metadata.len() > MAX_STEWARDSHIP_BYTES {
        return Err(JobError::Failed(
            "The stewardship ledger is not a supported regular file.".into(),
        ));
    }
    let file = File::open(path).map_err(|error| {
        JobError::Blocked(format!(
            "A readable local stewardship ledger is required: {error}"
        ))
    })?;
    let mut bytes = Vec::new();
    file.take(MAX_STEWARDSHIP_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| JobError::Failed(format!("Read stewardship ledger: {error}")))?;
    if bytes.len() as u64 > MAX_STEWARDSHIP_BYTES {
        return Err(JobError::Failed(
            "The stewardship ledger exceeds the supported size.".into(),
        ));
    }
    let fabric: NaturalHostStewardshipFabric = serde_json::from_slice(&bytes)
        .map_err(|_| JobError::Blocked("The persisted stewardship ledger is invalid.".into()))?;
    if fabric.stewards.is_empty() || fabric.threshold_m == 0 {
        return Err(JobError::Blocked(
            "A steward roster and nonzero quorum are required.".into(),
        ));
    }
    Ok(fabric)
}

fn compile_model_for_job(storage: &Path, job_id: &str, source: &Path) -> Result<String, JobError> {
    if !source.is_absolute()
        || !source
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"))
    {
        return Err(JobError::Blocked(
            "Stage an absolute path to a local GGUF file.".into(),
        ));
    }
    #[cfg(windows)]
    if !matches!(
        source.components().next(),
        Some(std::path::Component::Prefix(prefix))
            if matches!(prefix.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_))
    ) {
        return Err(JobError::Blocked(
            "Select a GGUF file from a local drive.".into(),
        ));
    }
    let ancestors: Vec<_> = source.ancestors().skip(1).collect();
    for parent in ancestors.into_iter().rev() {
        let metadata = fs::symlink_metadata(parent).map_err(|error| {
            JobError::Blocked(format!("Inspect staged model directory: {error}"))
        })?;
        if !metadata.is_dir() || is_reparse_point(&metadata) {
            return Err(JobError::Blocked(
                "Select a GGUF file in a regular local directory.".into(),
            ));
        }
    }

    let metadata = fs::symlink_metadata(source)
        .map_err(|error| JobError::Blocked(format!("Open staged GGUF source: {error}")))?;
    if !metadata.is_file() || is_reparse_point(&metadata) || metadata.len() > MAX_GGUF_BYTES {
        return Err(JobError::Blocked(
            "The source must be a regular GGUF file of at most 512 MiB for the in-shell converter."
                .into(),
        ));
    }
    let mut gguf = Vec::new();
    File::open(source)
        .and_then(|file| file.take(MAX_GGUF_BYTES + 1).read_to_end(&mut gguf))
        .map_err(|error| JobError::Failed(format!("Read staged GGUF: {error}")))?;
    let header = gguf
        .get(..24)
        .ok_or_else(|| JobError::Failed("The GGUF header is truncated.".into()))?;
    let version = u32::from_le_bytes(header[4..8].try_into().expect("fixed header slice"));
    let tensors = u64::from_le_bytes(header[8..16].try_into().expect("fixed header slice"));
    let keys = u64::from_le_bytes(header[16..24].try_into().expect("fixed header slice"));
    if gguf.len() as u64 > MAX_GGUF_BYTES
        || &header[..4] != b"GGUF"
        || !matches!(version, 2 | 3)
        || tensors == 0
        || tensors > MAX_GGUF_TENSORS
        || keys > MAX_GGUF_KEYS
    {
        return Err(JobError::Failed(
            "The GGUF container exceeds the supported source bounds.".into(),
        ));
    }
    let p64 = compile_gguf_to_p64(&gguf, 14).map_err(JobError::Failed)?;
    let index = P64TensorIndex::from_p64_with_integrity(&p64, IntegrityMode::Full)
        .map_err(JobError::Failed)?;
    index
        .validate_against_gguf(&p64, &gguf)
        .map_err(JobError::Failed)?;

    let sequence = job_id
        .strip_prefix("job:")
        .and_then(|id| id.parse::<u64>().ok())
        .ok_or_else(|| JobError::Failed("The job has no canonical artifact identifier.".into()))?;
    let storage_root = storage
        .canonicalize()
        .map_err(|error| JobError::Failed(format!("Resolve model storage: {error}")))?;
    let mut output_dir = storage_root;
    for component in ["Models", "job-output"] {
        output_dir.push(component);
        match fs::create_dir(&output_dir) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(JobError::Failed(format!(
                    "Create model output directory: {error}"
                )));
            }
        }
        let metadata = fs::symlink_metadata(&output_dir).map_err(|error| {
            JobError::Failed(format!("Inspect model output directory: {error}"))
        })?;
        if !metadata.is_dir() || is_reparse_point(&metadata) {
            return Err(JobError::Failed(
                "Model outputs require a regular storage directory.".into(),
            ));
        }
    }
    let name = format!("job-{sequence}.p64");
    let target = output_dir.join(&name);
    let temporary = output_dir.join(format!(".job-{sequence}.p64.{}.tmp", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|error| JobError::Failed(format!("Stage validated model: {error}")))?;
    let staged = file.write_all(&p64).and_then(|()| file.sync_all());
    drop(file);
    // Publish the verified complete inode without replacing any existing output.
    let published = staged.and_then(|()| fs::hard_link(&temporary, &target));
    let _ = fs::remove_file(&temporary);
    published.map_err(|error| JobError::Failed(format!("Publish validated model: {error}")))?;
    Ok(format!(
        "Verified {} tensors in Models/job-output/{name}. Source retained.",
        index.entries.len(),
    ))
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qap_host::TrustRoots;
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };

    #[test]
    fn background_worker_recovers_interruption_and_exports_real_backup() {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let storage = std::env::temp_dir().join(format!(
            "webizen-background-job-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&storage).unwrap();
        let fabric = NaturalHostStewardshipFabric::default();
        fabric.save(&storage).unwrap();
        let mut center = OsControlCenterState::load_or_seed(&storage, &fabric).unwrap();
        let interrupted = center
            .enqueue_os_job(
                &storage,
                OsJobKind::CareHeartbeatWatchdog,
                "Interrupted watchdog",
                "wellfair",
            )
            .unwrap();
        OsControlCenterState::claim_background_job(&storage, &interrupted.job_id).unwrap();
        let host = LoopbackHost {
            daemon_base: "http://127.0.0.1:4242".into(),
            port: 4242,
            storage: storage.clone(),
            install_root: storage.clone(),
            trust: TrustRoots::for_storage(&storage),
        };
        let gpu = NativeEtalonStatus {
            native_gpu_active: false,
            primary_adapter: None,
            adapters: Vec::new(),
            anvil_profile: anvil_core::NodeProfile64::empty(),
            specializer_hints: anvil_gpu::hints_for_pci(0, 0),
            native_vibe_provider_count: 0,
            nquin_parity_verified: false,
            may_advertise_hardware_attested_vault: false,
            summary_label: "Source test fixture".into(),
        };
        let (wake, await_wake) = mpsc::channel();
        let (release_first_job, wait_for_first_submission) = mpsc::channel();
        let pause = Mutex::new(wait_for_first_submission);
        let first_wake = AtomicBool::new(true);
        let runner = OsJobRunner::for_host(host.clone(), gpu.clone(), move || {
            let _ = wake.send(());
            if first_wake.swap(false, Ordering::AcqRel) {
                let _ = pause.lock().unwrap().recv_timeout(Duration::from_secs(5));
            }
        })
        .unwrap();
        assert!(OsJobRunner::for_host(host, gpu, || {}).is_err());
        assert!(runner.refresh(&mut center).unwrap());
        let recovered = center
            .jobs
            .iter()
            .find(|row| row.job_id == interrupted.job_id)
            .unwrap();
        assert_eq!(recovered.status, OsJobStatus::Failed);

        let backup = center
            .enqueue_os_job(
                &storage,
                OsJobKind::ShamirQ42jBackup,
                "Encrypted backup",
                "settings",
            )
            .unwrap();
        let externally_cancelled = center
            .enqueue_os_job(
                &storage,
                OsJobKind::CareHeartbeatWatchdog,
                "Cancelled while waiting",
                "wellfair",
            )
            .unwrap();
        let heartbeat = center
            .enqueue_os_job(
                &storage,
                OsJobKind::CareHeartbeatWatchdog,
                "Inspect heartbeats",
                "wellfair",
            )
            .unwrap();
        runner.start(&backup.job_id, OsJobInput::Standard).unwrap();
        await_wake.recv_timeout(Duration::from_secs(5)).unwrap();
        runner.refresh(&mut center).unwrap();
        assert_eq!(
            center
                .jobs
                .iter()
                .find(|row| row.job_id == backup.job_id)
                .unwrap()
                .status,
            OsJobStatus::Running,
        );
        runner
            .start(&externally_cancelled.job_id, OsJobInput::Standard)
            .unwrap();
        runner
            .start(&heartbeat.job_id, OsJobInput::Standard)
            .unwrap();
        center
            .cancel_os_job(&storage, &externally_cancelled.job_id)
            .unwrap();
        assert!(runner
            .start(&heartbeat.job_id, OsJobInput::Standard)
            .is_err());
        assert_eq!(
            center
                .jobs
                .iter()
                .find(|row| row.job_id == heartbeat.job_id)
                .unwrap()
                .status,
            OsJobStatus::Queued,
        );
        release_first_job.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            await_wake
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .unwrap();
            runner.refresh(&mut center).unwrap();
            let current = center
                .jobs
                .iter()
                .find(|row| row.job_id == heartbeat.job_id)
                .unwrap();
            if !current.status.is_active() {
                assert_eq!(current.status, OsJobStatus::Completed);
                assert!(current.detail.contains("No alerts dispatched"));
                break;
            }
        }
        let cancelled = center
            .jobs
            .iter()
            .find(|row| row.job_id == externally_cancelled.job_id)
            .unwrap();
        assert_eq!(cancelled.status, OsJobStatus::Cancelled);
        let completed = center
            .jobs
            .iter()
            .find(|row| row.job_id == backup.job_id)
            .unwrap();
        assert_eq!(completed.status, OsJobStatus::Completed);
        assert_eq!(completed.progress_pct, 100);
        let archive = fs::read_dir(storage.join("backups/q42j"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        assert!(archive.path().join("journal.q42j").is_file());
        assert!(archive.path().join("archive.json").is_file());
        // The finishing wake is sent before the worker releases its ownership.
        drop(runner);
        let cleanup_deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match fs::remove_dir_all(&storage) {
                Ok(()) => break,
                Err(error)
                    if Instant::now() < cleanup_deadline
                        && error.kind() != std::io::ErrorKind::NotFound =>
                {
                    std::thread::sleep(Duration::from_millis(15));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                Err(error) => panic!("remove job fixture: {error}"),
            }
        }
    }
}
