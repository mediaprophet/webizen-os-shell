// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Canonical VibeScript execution and authority-scoped source review.
//! Session labels support local rehearsal. Only an independently adopted native
//! authority can use signed graph grants or admit a canonical review.

use crate::host_stewardship::validate_human_centric_host_nomenclature;
use crate::{qap_host::require_qap_capability, CapabilityLeaseScope};
use qualia_cooperative_core::authoring::{
    authorize, AuthoringAction, AuthoringRequest, DocumentRevision,
};
use qualia_core_db::{
    modalities::logic::deontic::OP_PERMIT,
    vibe_host::{
        native_authority::{
            graph::{NativeGraphHost, SignedGraphGrant},
            DocumentTransaction, NativeAuthority, SessionHandle,
        },
        vibe_sentinel::{make_deontic_norm_quin, WebizenSentinelGate},
    },
    NQuin,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::{
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use vibe::{
    extension_registry::CallExecutionContext,
    session::{SessionLimits, SourceSession},
    Budget, DiagCode, Diagnostic, Engine, Env, Host, Span, Value, SENTINEL_BYTES,
};
use vibescript_abi::{
    q_hash_str, AdmissionRequest, AdmissionTicket, CompletionStatus, EffectSet, ExecutionMode,
    ExecutionReceiptHeader, NQuin48, ResourceEnvelope, SentinelPolicyGate, VibeOpCode, VibeTrap,
};

const MAX_SOURCE: usize = 64 * 1024;
const MAX_RESULT: usize = 64 * 1024;
const MAX_ID: usize = 256;
const MAX_EVIDENCE: usize = 4096;
const MAX_REVIEWS: usize = 256;
const MAX_JOURNAL: u64 = 2 * 1024 * 1024;
const MAX_RECEIPTS: usize = 64;
const MAX_STEPS: u64 = 10_000;
const NATIVE_PROVIDER: u64 = q_hash_str("vibe:native");
const REVIEW_JOURNAL: &str = "authoring_reviews.json";
static NEXT_EPOCH: AtomicU64 = AtomicU64::new(1);
static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

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

impl From<&ExecutionReceiptHeader> for VibeExecutionReceiptSummary {
    fn from(receipt: &ExecutionReceiptHeader) -> Self {
        let status = match receipt.status {
            CompletionStatus::Committed => "Committed",
            CompletionStatus::SimulatedCommitted => "Local Rehearsal",
            CompletionStatus::StandaloneCompleted => "Standalone",
            CompletionStatus::AbortedPolicyDenied => "Policy denied",
            CompletionStatus::AbortedValidationFailed => "Validation aborted",
            CompletionStatus::AbortedReceiptFailed => "Receipt commit aborted",
            CompletionStatus::TrappedExecution => "Execution trapped",
        };
        Self {
            ticket_id: receipt.ticket_id,
            actor_did_hash: receipt.actor_did_hash,
            provider_iri_hash: receipt.provider_iri_hash,
            func_iri_hash: receipt.func_iri_hash,
            policy_generation: receipt.policy_generation,
            ticks_consumed: u64::from(receipt.ticks_consumed),
            output_bytes_written: receipt.output_bytes_written,
            status_label: status.into(),
            simulated: receipt.simulated,
            lamport_timestamp: receipt.lamport_timestamp,
        }
    }
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

/// Host-adopted authority and independently signed graph scope. The application
/// retains ownership of the session and disconnects it when its window closes.
#[derive(Clone)]
pub struct AdoptedQualiaFoundation {
    authority: Arc<NativeAuthority>,
    session: SessionHandle,
    graph_grant: Option<SignedGraphGrant>,
}

impl AdoptedQualiaFoundation {
    pub fn connect(
        authority: Arc<NativeAuthority>,
        session: SessionHandle,
        graph_grant: Option<SignedGraphGrant>,
    ) -> Result<Self, String> {
        authority.workflow_evidence(&session)?;
        if let Some(grant) = graph_grant.as_ref() {
            authority
                .bind_graph_scope(&session, grant)
                .map_err(|e| e.to_string())?;
        }
        Ok(Self {
            authority,
            session,
            graph_grant,
        })
    }

    pub fn authority(&self) -> Arc<NativeAuthority> {
        Arc::clone(&self.authority)
    }

    pub fn session(&self) -> SessionHandle {
        self.session.clone()
    }

    pub fn graph_grant(&self) -> Option<&SignedGraphGrant> {
        self.graph_grant.as_ref()
    }

    /// Execute only the exact text adopted under this document's raw SHA-256.
    /// The host must continue to enforce the signed graph scope per operation.
    pub fn authorize_source(&self, source: &str) -> Result<(CallExecutionContext, u64), String> {
        self.authorized_source_revision(source)
            .map(|(context, generation, _)| (context, generation))
    }

    fn authorized_source_revision(
        &self,
        source: &str,
    ) -> Result<(CallExecutionContext, u64, DocumentRevision), String> {
        validate_source(source)?;
        let grant = self
            .graph_grant()
            .ok_or("A signed graph scope is required.")?;
        let scope = self
            .authority
            .bind_graph_scope(&self.session, grant)
            .map_err(|e| e.to_string())?;
        self.authority.transact(&self.session, |view| {
            let result = (|| {
                require_exact_source(&view.current_document.source_digest, source)?;
                let permit = authorize(
                    view,
                    &AuthoringRequest {
                        action: AuthoringAction::ExecuteSource,
                        target: view.current_document,
                        expected_authority_generation: view.generation,
                    },
                )
                .map_err(|error| error.to_string())?;
                Ok((
                    CallExecutionContext {
                        actor_did_hash: q_hash_str(permit.principal_subject()),
                        context_world_hash: scope.context(),
                        lamport_timestamp: next_epoch()?,
                        mode: ExecutionMode::GovernedApplication,
                    },
                    view.generation,
                    (*view.current_document).clone(),
                ))
            })();
            DocumentTransaction {
                result,
                successor: None,
                reconciliation_required: false,
            }
        })
    }

    /// Submit the adopted text to the canonical native journal. A request does
    /// not approve or publish the document.
    pub fn request_source_review(
        &self,
        document_id: &str,
        source: &str,
        evidence_ref: &str,
    ) -> Result<AuthoringWorkflowSnapshot, String> {
        validate_review_inputs(document_id, source, evidence_ref)?;
        let evidence = self.authority.workflow_evidence(&self.session)?;
        if evidence.target.document_id != document_id {
            return Err("The document ID differs from the adopted source.".into());
        }
        require_exact_source(&evidence.target.source_digest, source)?;
        let (request_id, quin) = review_binding(
            &evidence.principal_subject,
            document_id,
            &evidence.target.source_digest,
            evidence_ref,
        );
        let request = self.authority.request_source_review(
            &self.session,
            &evidence.target,
            &request_id,
            evidence_ref,
        )?;
        Ok(AuthoringWorkflowSnapshot {
            document_id: request.target.document_id,
            revision: request.target.revision,
            authority_generation: request.authority_generation,
            last_review_request_id: Some(request.id),
            last_evidence_ref: Some(request.evidence_ref),
            status_summary: "Live Verified · native request recorded; review pending".into(),
            sealed_nquin_parity: quin.parity(),
        })
    }
}

fn require_exact_source(expected: &[u8; 32], source: &str) -> Result<(), String> {
    if vibe::crypto::sha256_bytes(source.as_bytes()) == *expected {
        Ok(())
    } else {
        Err("Source bytes differ from the adopted native document.".into())
    }
}

impl AdoptedQualiaFoundation {
    /// Direct synchronous execution for an embedding owner. UI dispatchers can
    /// use authorize_source plus Aether's native graph runtime/mailbox instead.
    pub fn run_source(
        &self,
        program_id: &str,
        source: &str,
        cancelled: Arc<AtomicBool>,
    ) -> Result<GovernedVibeActionReport, String> {
        self.run_source_with_policy(None, program_id, source, cancelled)
    }

    /// Execute the installed Poet surface with fresh Qap policy at each graph boundary.
    pub fn run_qap_source(
        &self,
        storage: &Path,
        program_id: &str,
        source: &str,
        cancelled: Arc<AtomicBool>,
    ) -> Result<GovernedVibeActionReport, String> {
        self.run_source_with_policy(Some(storage), program_id, source, cancelled)
    }

    fn run_source_with_policy(
        &self,
        storage: Option<&Path>,
        program_id: &str,
        source: &str,
        cancelled: Arc<AtomicBool>,
    ) -> Result<GovernedVibeActionReport, String> {
        validate_label(program_id, "Program ID", MAX_ID)?;
        if let Some(storage) = storage {
            QapPoetGraphHost::check_policy(storage)?;
        }
        let (context, generation, document) = self.authorized_source_revision(source)?;
        let grant = self
            .graph_grant()
            .ok_or("A signed graph scope is required.")?;
        let scope = self
            .authority
            .bind_graph_scope(&self.session, grant)
            .map_err(|e| e.to_string())?;
        let mut capabilities = Vec::new();
        let mut effects = EffectSet::PURE_COMPUTE;
        if grant.grant.rights.read {
            effects = effects.union(EffectSet::GRAPH_READ);
            capabilities.push("graph.read".into());
            if grant.grant.rights.replace {
                effects = effects.union(EffectSet::GRAPH_MUTATE);
                capabilities.push("graph.write".into());
            }
        }
        let script = prepare_source(source, capabilities)?;
        let mut budget = execution_budget();
        let host = NativeGraphHost::new_with_budget(
            self.authority(),
            scope,
            Arc::clone(&cancelled),
            &mut budget,
        )
        .map_err(|error| error.to_string())?;
        let mut host = QapPoetGraphHost {
            inner: host,
            storage: storage.map(Path::to_owned),
            adopted: self.clone(),
            generation,
            document: document.clone(),
        };
        if !self
            .authority
            .source_publication_is_current(&self.session, generation, &document)
            || cancelled.load(Ordering::Acquire)
        {
            return Err("The authenticated source scope ended before execution.".into());
        }
        let mut report = run_script(
            &mut host,
            script,
            budget,
            program_id,
            ExecutionPolicy {
                context,
                generation,
                effects,
                cancelled: &cancelled,
            },
        );
        if let Some(receipt) = host.inner.last_receipt() {
            report.graph_revision = receipt.revision;
        }
        if report.diagnostic.is_none() && grant.grant.rights.read && host.check().is_ok() {
            let current = self
                .authority
                .bind_graph_scope(&self.session, grant)
                .and_then(|scope| {
                    let mut quins = vec![NQuin::default(); scope.max_quins()];
                    self.authority.read_graph(&scope, &mut quins)
                });
            match current {
                Ok(read) => {
                    report.graph_revision = read.revision;
                    report.graph_quin_count = read.written;
                }
                Err(error) => {
                    report.formatted_value.clear();
                    report.reconciliation_required |= host.inner.last_receipt().is_some();
                    report.diagnostic = Some(format!("Graph publication withheld: {error}"));
                }
            }
        }
        if !self
            .authority
            .source_publication_is_current(&self.session, generation, &document)
            || cancelled.load(Ordering::Acquire)
        {
            report.formatted_value.clear();
            report.reconciliation_required |= host.inner.last_receipt().is_some();
            report.diagnostic = Some("The authenticated source scope ended before publication. Retain any committed receipts.".into());
        }
        if let Err(error) = host.check() {
            report.formatted_value.clear();
            report.reconciliation_required |= host.inner.last_receipt().is_some();
            report.diagnostic = Some(format!("Qap publication withheld: {error}"));
        }
        Ok(report)
    }
}

/// The installed Poet surface narrows NativeGraphHost with fresh local revocation checks.
pub struct QapPoetGraphHost {
    inner: NativeGraphHost,
    storage: Option<PathBuf>,
    adopted: AdoptedQualiaFoundation,
    generation: u64,
    document: DocumentRevision,
}

impl QapPoetGraphHost {
    pub fn admit(
        storage: &Path,
        inner: NativeGraphHost,
        adopted: &AdoptedQualiaFoundation,
        source: &str,
    ) -> Result<(Self, CallExecutionContext, u64), String> {
        Self::check_policy(storage)?;
        let (context, generation, document) = adopted.authorized_source_revision(source)?;
        Ok((
            Self {
                inner,
                storage: Some(storage.to_owned()),
                adopted: adopted.clone(),
                generation,
                document,
            },
            context,
            generation,
        ))
    }

    pub fn check_policy(storage: &Path) -> Result<(), String> {
        for scope in [
            CapabilityLeaseScope::GovernedVibeScriptEval,
            CapabilityLeaseScope::QualiaQ42Storage,
        ] {
            require_qap_capability(storage, "Poet", scope).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn check(&self) -> Result<(), String> {
        self.storage.as_deref().map_or(Ok(()), Self::check_policy)?;
        if !self.adopted.authority.source_publication_is_current(
            &self.adopted.session,
            self.generation,
            &self.document,
        ) {
            return Err(
                "The current QualiaDB session or exact source revision is unavailable.".into(),
            );
        }
        Ok(())
    }

    fn guard(&self, span: Span) -> Result<(), Diagnostic> {
        self.check()
            .map_err(|message| Diagnostic::new(DiagCode::E300, span, message))
    }
}

impl Host for QapPoetGraphHost {
    fn graph_query(&mut self, args: &[Value], take: u64, span: Span) -> Result<Value, Diagnostic> {
        self.guard(span)?;
        let result = self.inner.graph_query(args, take, span)?;
        self.guard(span)?;
        Ok(result)
    }

    fn graph_begin(&mut self, span: Span) -> Result<(), Diagnostic> {
        self.guard(span)?;
        self.inner.graph_begin(span)
    }

    fn graph_stage(&mut self, value: &Value, span: Span) -> Result<Value, Diagnostic> {
        self.guard(span)?;
        self.inner.graph_stage(value, span)
    }

    fn graph_commit(&mut self, span: Span) -> Result<Value, Diagnostic> {
        if let Err(error) = self.guard(span) {
            let _ = self.inner.graph_abort(span);
            return Err(error);
        }
        self.inner.graph_commit(span)
    }

    fn graph_abort(&mut self, span: Span) -> Result<(), Diagnostic> {
        self.inner.graph_abort(span)
    }

    fn graph_end(&mut self, span: Span) -> Result<(), Diagnostic> {
        if let Err(error) = self.guard(span) {
            let _ = self.inner.graph_abort(span);
            return Err(error);
        }
        self.inner.graph_end(span)
    }

    fn graph_snapshot(&mut self, span: Span) -> Result<Value, Diagnostic> {
        self.guard(span)?;
        let result = self.inner.graph_snapshot(span)?;
        self.guard(span)?;
        Ok(result)
    }

    fn capability_resolve(&mut self, id: &str, span: Span) -> Result<Value, Diagnostic> {
        self.guard(span)?;
        self.inner.capability_resolve(id, span)
    }

    fn aura_validate(
        &mut self,
        value: &Value,
        shape: &Value,
        span: Span,
    ) -> Result<Value, Diagnostic> {
        self.inner.aura_validate(value, shape, span)
    }

    fn pulse_publish(
        &mut self,
        topic: &str,
        value: &Value,
        span: Span,
    ) -> Result<Value, Diagnostic> {
        self.inner.pulse_publish(topic, value, span)
    }
}

struct RehearsalHost;

fn no_host_effect(span: Span) -> Diagnostic {
    Diagnostic::new(
        DiagCode::E300,
        span,
        "Local rehearsal grants no graph or external host effects",
    )
}

impl Host for RehearsalHost {
    fn graph_query(&mut self, _: &[Value], _: u64, span: Span) -> Result<Value, Diagnostic> {
        Err(no_host_effect(span))
    }
    fn graph_stage(&mut self, _: &Value, span: Span) -> Result<Value, Diagnostic> {
        Err(no_host_effect(span))
    }
    fn graph_begin(&mut self, span: Span) -> Result<(), Diagnostic> {
        Err(no_host_effect(span))
    }
    fn graph_commit(&mut self, span: Span) -> Result<Value, Diagnostic> {
        Err(no_host_effect(span))
    }
    fn aura_validate(&mut self, _: &Value, _: &Value, span: Span) -> Result<Value, Diagnostic> {
        Err(no_host_effect(span))
    }
    fn pulse_publish(&mut self, _: &str, _: &Value, span: Span) -> Result<Value, Diagnostic> {
        Err(no_host_effect(span))
    }
}

/// Run a local effect-free rehearsal. A DID label does not authenticate the
/// caller or produce a durable live receipt.
pub fn run_governed_vibe_source(
    _storage: &Path,
    actor_did: &str,
    program_id: &str,
    source: &str,
) -> Result<GovernedVibeActionReport, String> {
    validate_actor_did(actor_did)?;
    validate_label(program_id, "Program ID", MAX_ID)?;
    validate_source(source)?;
    let script = prepare_source(source, Vec::new())?;
    let cancelled = AtomicBool::new(false);
    Ok(run_script(
        &mut RehearsalHost,
        script,
        execution_budget(),
        program_id,
        ExecutionPolicy {
            context: CallExecutionContext {
                actor_did_hash: q_hash_str(actor_did),
                context_world_hash: q_hash_str("urn:webizen:local-vibe-rehearsal"),
                lamport_timestamp: next_epoch()?,
                mode: ExecutionMode::SimulatedGovernanceTest,
            },
            generation: 1,
            effects: EffectSet::PURE_COMPUTE,
            cancelled: &cancelled,
        },
    ))
}

enum CheckedSource {
    Program(vibe::Program),
    Cell(vibe::Expr),
}

fn prepare_source(source: &str, capabilities: Vec<String>) -> Result<CheckedSource, String> {
    let session = SourceSession::new(
        capabilities.clone(),
        SessionLimits {
            submission_bytes: MAX_SOURCE,
            retained_source_bytes: MAX_SOURCE,
            ..SessionLimits::default()
        },
    );
    // The checked candidate is opaque. Use it for canonical ingress bounds,
    // then execute a fresh Engine with the caller-owned Sentinel gate.
    session.prepare(source).map_err(|error| error.to_string())?;
    let trimmed = source.strip_prefix('\u{feff}').unwrap_or(source).trim();
    if !trimmed.starts_with('=') && vibe::parse_program(trimmed).is_ok() {
        let program = vibe::load_program(trimmed).map_err(|error| error.to_string())?;
        if program
            .requires
            .iter()
            .any(|request| !request.args.is_empty() || !capabilities.contains(&request.id))
        {
            return Err(
                "Source requests an ungranted or unsupported capability constraint.".into(),
            );
        }
        Ok(CheckedSource::Program(program))
    } else {
        let text = if trimmed.starts_with('=') {
            trimmed.to_owned()
        } else {
            format!("= {trimmed}")
        };
        vibe::parse_cell(&text)
            .map(CheckedSource::Cell)
            .map_err(|error| error.to_string())
    }
}

fn execution_budget() -> Budget {
    Budget {
        steps_left: MAX_STEPS,
        workspace_left: SENTINEL_BYTES,
    }
}

fn next_epoch() -> Result<u64, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "The device clock is before the Unix epoch.")?
        .as_secs();
    let sequence = NEXT_EPOCH.fetch_add(1, Ordering::Relaxed);
    // The pinned gate reads the low 32 bits as Unix32.
    Ok((sequence << 32) | (seconds & u64::from(u32::MAX)))
}

struct ExecutionPolicy<'a> {
    context: CallExecutionContext,
    generation: u64,
    effects: EffectSet,
    cancelled: &'a AtomicBool,
}

const PURE_MATH: &[&str] = &[
    "math.abs",
    "math.min",
    "math.max",
    "math.floor",
    "math.ceil",
    "math.round",
    "math.clamp",
    "math.sqrt",
    "math.sin",
    "math.cos",
    "math.log",
    "math.exp",
    "math.pow",
    "math.log10",
    "math.tan",
    "math.atan",
    "math.atan2",
];

fn run_script(
    host: &mut impl Host,
    script: CheckedSource,
    budget: Budget,
    program_id: &str,
    policy: ExecutionPolicy<'_>,
) -> GovernedVibeActionReport {
    let context = policy.context;
    let mut norms = Vec::new();
    let mut permit = |path: &str| {
        norms.push(make_deontic_norm_quin(
            context.actor_did_hash,
            OP_PERMIT,
            false,
            q_hash_str(path),
            NATIVE_PROVIDER,
            context.context_world_hash,
            0,
        ));
    };
    for path in PURE_MATH {
        permit(path);
    }
    if policy.effects.contains_all(EffectSet::GRAPH_READ) {
        permit("graph.query");
        permit("graph.snapshot");
    }
    if policy.effects.contains_all(EffectSet::GRAPH_MUTATE) {
        permit("graph.stage");
        permit("graph.commit");
    }
    let mut receipt_quins = [NQuin::default(); MAX_RECEIPTS * 2];
    let mut gate = ReceiptCapture {
        gate: WebizenSentinelGate::new(
            &norms,
            &mut receipt_quins,
            policy.generation,
            policy.effects,
            ResourceEnvelope::hot_zero_heap(MAX_STEPS, 4096),
        ),
        receipts: Vec::with_capacity(MAX_RECEIPTS),
        cancelled: policy.cancelled,
    };
    let mut env = Env::default();
    let (entry_fn, value) = match script {
        CheckedSource::Cell(expr) => {
            let mut engine = Engine::new(host, budget);
            engine.set_extension_governance(context, Some(&mut gate));
            ("cell", engine.eval_expr(&expr, &mut env))
        }
        CheckedSource::Program(program) => {
            let mut engine = Engine::with_program(host, budget, &program);
            engine.set_extension_governance(context, Some(&mut gate));
            ("top-level", engine.eval_program(&program, &mut env))
        }
    };
    let value = value
        .map_err(|error| error.to_string())
        .and_then(|v| bounded_result(&v));
    let mut report = GovernedVibeActionReport {
        program_id: program_id.into(),
        entry_fn: entry_fn.into(),
        epoch: context.lamport_timestamp,
        formatted_value: String::new(),
        receipts: gate
            .receipts
            .iter()
            .map(VibeExecutionReceiptSummary::from)
            .collect(),
        graph_revision: 0,
        graph_quin_count: 0,
        reconciliation_required: false,
        diagnostic: None,
    };
    match value {
        Ok(value) => report.formatted_value = value,
        Err(error) => {
            report.diagnostic = Some(error);
            // Canonical journal effects can precede a later interpreter or
            // receipt failure. No rollback is implied by a failed evaluation.
            report.reconciliation_required = policy.effects.contains_all(EffectSet::GRAPH_MUTATE);
        }
    }
    report
}

struct ReceiptCapture<'a> {
    gate: WebizenSentinelGate<'a>,
    receipts: Vec<ExecutionReceiptHeader>,
    cancelled: &'a AtomicBool,
}

impl SentinelPolicyGate for ReceiptCapture<'_> {
    fn is_controller_alive(&self) -> bool {
        !self.cancelled.load(Ordering::Acquire) && self.gate.is_controller_alive()
    }

    fn current_policy_generation(&self) -> u64 {
        self.gate.current_policy_generation()
    }

    fn admit_call(&mut self, request: &AdmissionRequest) -> Result<AdmissionTicket, VibeTrap> {
        if !self.is_controller_alive() {
            return Err(VibeTrap::SentinelUnavailable);
        }
        if self.receipts.len() >= MAX_RECEIPTS {
            return Err(VibeTrap::ReceiptPersistenceFailed);
        }
        self.gate.admit_call(request)
    }

    fn validate_post_execution(
        &mut self,
        ticket: &AdmissionTicket,
        opcode: VibeOpCode,
        ticks_consumed: u64,
        output_bytes_written: usize,
        output_digest: u64,
    ) -> Result<(), VibeTrap> {
        self.gate.validate_post_execution(
            ticket,
            opcode,
            ticks_consumed,
            output_bytes_written,
            output_digest,
        )
    }

    fn commit_receipt(
        &mut self,
        receipt: &ExecutionReceiptHeader,
    ) -> Result<[NQuin48; 2], VibeTrap> {
        let quins = self.gate.commit_receipt(receipt)?;
        debug_assert!(quins.iter().all(NQuin48::verify_parity));
        self.receipts.push(*receipt);
        Ok(quins)
    }
}

fn bounded_result(value: &Value) -> Result<String, String> {
    struct BoundedString(String);

    impl std::fmt::Write for BoundedString {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            if text.len() > MAX_RESULT.saturating_sub(self.0.len()) {
                return Err(std::fmt::Error);
            }
            self.0.push_str(text);
            Ok(())
        }
    }

    let mut output = BoundedString(String::new());
    write!(&mut output, "{value}").map_err(|_| "VibeScript result exceeds the display budget.")?;
    Ok(output.0)
}

fn validate_label(value: &str, field: &str, maximum: usize) -> Result<(), String> {
    validate_human_centric_host_nomenclature(value)?;
    if value.trim().is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
        return Err(format!(
            "{field} must be nonempty and at most {maximum} bytes."
        ));
    }
    Ok(())
}

fn validate_actor_did(value: &str) -> Result<(), String> {
    validate_label(value, "Actor DID", MAX_ID)?;
    let valid = value
        .strip_prefix("did:")
        .and_then(|tail| tail.split_once(':'))
        .is_some_and(|(method, id)| !method.is_empty() && !id.is_empty());
    if !valid || value.chars().any(char::is_whitespace) {
        return Err("Actor label must use a contextual DID.".into());
    }
    Ok(())
}

fn validate_source(source: &str) -> Result<(), String> {
    if source.trim().is_empty() || source.len() > MAX_SOURCE || source.contains('\0') {
        return Err("Source text is empty or exceeds the bounded VibeScript input profile.".into());
    }
    Ok(())
}

fn validate_review_inputs(
    document_id: &str,
    source: &str,
    evidence_ref: &str,
) -> Result<(), String> {
    validate_label(document_id, "Document ID", MAX_ID)?;
    validate_label(evidence_ref, "Evidence reference", MAX_EVIDENCE)?;
    validate_source(source)
}

fn review_binding(
    actor: &str,
    document_id: &str,
    source_digest: &[u8; 32],
    evidence_ref: &str,
) -> (String, NQuin48) {
    let mut digest = Sha3_256::new();
    digest.update(b"webizen.authoring.source-review.v1\0");
    for lane in [
        actor.as_bytes(),
        document_id.as_bytes(),
        source_digest,
        evidence_ref.as_bytes(),
    ] {
        digest.update((lane.len() as u64).to_le_bytes());
        digest.update(lane);
    }
    let request_id = format!("review-{:x}", digest.finalize());
    let quin = NQuin48::from_parts(
        q_hash_str(document_id),
        q_hash_str("urn:webizen:authoring:requested-source-review"),
        q_hash_str(&request_id),
        q_hash_str(actor),
        q_hash_str(evidence_ref),
    );
    (request_id, quin)
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
struct LocalReview {
    source_sha256_hex: String,
    quin: [u64; 6],
    snapshot: AuthoringWorkflowSnapshot,
}

/// Persist a local review draft. An asserted DID or evidence reference does not
/// create native authority or verify an external signer.
pub fn record_authoring_source_review(
    storage: &Path,
    actor_did: &str,
    document_id: &str,
    source: &str,
    evidence_ref: &str,
) -> Result<AuthoringWorkflowSnapshot, String> {
    validate_actor_did(actor_did)?;
    validate_review_inputs(document_id, source, evidence_ref)?;

    let source_digest = vibe::crypto::sha256_bytes(source.as_bytes());
    let (request_id, quin) = review_binding(actor_did, document_id, &source_digest, evidence_ref);
    let draft = LocalReview {
        source_sha256_hex: vibe::crypto::sha256_hex(source.as_bytes()),
        quin: quin.0,
        snapshot: AuthoringWorkflowSnapshot {
            document_id: document_id.into(),
            revision: u64::from_le_bytes(source_digest[..8].try_into().expect("SHA-256 prefix")),
            authority_generation: 0,
            last_review_request_id: Some(request_id),
            last_evidence_ref: Some(evidence_ref.into()),
            status_summary: "Local Rehearsal · draft awaiting authenticated submission".into(),
            sealed_nquin_parity: quin.parity(),
        },
    };
    let directory = storage.join("poet");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    if is_link(&fs::symlink_metadata(&directory).map_err(|error| error.to_string())?) {
        return Err("The authoring review directory may not be a link.".into());
    }
    let lock_path = directory.join("authoring_reviews.lock");
    let lock = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
        .map_err(|error| format!("Unable to lock the authoring review journal: {error}"))?;
    let _lock = JournalLock {
        file: Some(lock),
        path: lock_path,
    };

    let journal_path = directory.join(REVIEW_JOURNAL);
    let mut records = read_review_journal(&journal_path)?;
    let snapshot = draft.snapshot.clone();
    if let Some(existing) = records
        .iter()
        .find(|record| record.snapshot.last_review_request_id == snapshot.last_review_request_id)
    {
        if *existing != draft {
            return Err("Retained authoring review differs from the current draft.".into());
        }
        return Ok(snapshot);
    }
    if records.len() >= MAX_REVIEWS {
        return Err("The authoring review journal reached its record limit.".into());
    }
    records.push(draft);
    let bytes = serde_json::to_vec_pretty(&records).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_JOURNAL {
        return Err("The authoring review journal reached its byte limit.".into());
    }
    replace_review_journal(&journal_path, &bytes)?;
    Ok(snapshot)
}

fn is_link(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

struct JournalLock {
    file: Option<File>,
    path: PathBuf,
}

impl Drop for JournalLock {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}

fn read_review_journal(path: &Path) -> Result<Vec<LocalReview>, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.is_file() && !is_link(&metadata) && metadata.len() <= MAX_JOURNAL => {}
        Ok(_) => return Err("The authoring review journal is not a bounded regular file.".into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(MAX_JOURNAL + 1).read_to_end(&mut bytes))
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_JOURNAL {
        return Err("The authoring review journal reached its byte limit.".into());
    }
    let records: Vec<LocalReview> = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Invalid authoring review journal: {error}"))?;
    if records.len() > MAX_REVIEWS
        || records.iter().any(|record| {
            let quin = NQuin48(record.quin);
            !quin.verify_parity()
                || quin.parity() != record.snapshot.sealed_nquin_parity
                || record.snapshot.authority_generation != 0
        })
    {
        return Err("The authoring review journal violates its local receipt profile.".into());
    }
    Ok(records)
}

fn replace_review_journal(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let temp = path.with_extension(format!(
        "tmp-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| error.to_string())?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let result = written.and_then(|()| fs::rename(&temp, path));
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_governed_vibescript_with_sentinel_gate_and_records_receipts() {
        let storage = Path::new("unused-by-effect-free-rehearsal");
        let report =
            run_governed_vibe_source(storage, "did:qi:reviewer", "addition", "= math.abs(-42)")
                .expect("bounded VibeScript ingress");
        assert_eq!(report.diagnostic, None);
        assert_eq!(report.formatted_value, "42");
        assert_eq!(report.graph_revision, 0);
        assert_eq!(report.receipts.len(), 1);
        let receipt = &report.receipts[0];
        assert!(receipt.simulated);
        assert_eq!(receipt.provider_iri_hash, q_hash_str("vibe:native"));
        assert_eq!(receipt.func_iri_hash, q_hash_str("math.abs"));
        assert_eq!(receipt.ticks_consumed, 1);
        assert_eq!(receipt.policy_generation, 1);

        let operator =
            run_governed_vibe_source(storage, "did:qi:reviewer", "addition", "= 21 + 21")
                .expect("bounded cell");
        assert_eq!(operator.formatted_value, "42");
        assert!(operator.receipts.is_empty(), "no host call was dispatched");
        assert!(run_governed_vibe_source(
            storage,
            "did:qi:reviewer",
            "graph",
            "requires [capability(\"graph.read\")]; graph.query(take: 8);",
        )
        .is_err());
    }

    #[test]
    fn records_authoring_source_review_and_rejects_natural_agent_collapse_terms() {
        let dir = std::env::temp_dir().join(format!(
            "webizen-authoring-review-test-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).expect("isolated review journal");
        let review = record_authoring_source_review(
            &dir,
            "did:qi:author",
            "project:document",
            "= 21 + 21",
            "urn:webizen:evidence:draft",
        )
        .expect("local review");
        assert_eq!(review.authority_generation, 0);
        let journal_path = dir.join("poet").join(REVIEW_JOURNAL);
        let records = read_review_journal(&journal_path).unwrap();
        assert_eq!(records.len(), 1);
        let quin = NQuin48(records[0].quin);
        assert!(quin.verify_parity());
        assert_eq!(review.sealed_nquin_parity, quin.parity());
        let repeated = record_authoring_source_review(
            &dir,
            "did:qi:author",
            "project:document",
            "= 21 + 21",
            "urn:webizen:evidence:draft",
        )
        .expect("idempotent retry");
        assert_eq!(review, repeated);
        assert!(record_authoring_source_review(
            &dir,
            "did:qi:owl:Thing",
            "project:document",
            "= 21 + 21",
            "urn:webizen:evidence:draft",
        )
        .is_err());
        assert_eq!(read_review_journal(&journal_path).unwrap().len(), 1);
        fs::remove_file(journal_path).unwrap();
        fs::remove_dir(dir.join("poet")).unwrap();
        fs::remove_dir(dir).unwrap();
    }
}
