// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::theme_studio::{SharedThemeCollectionPanel, ThemeCodesignerPanel};
use crate::components::viewports::Holo10dViewport;
use crate::state::{session, ShellState};
use aether::{aether_core::try_consume_context, prelude::*};
use aether_native::winit::{event::WindowEvent, window::Window};
use aether_qualia_host::{
    prelude::{
        try_use_qualia_context, use_authoring_bridge, use_qualia_subscription, use_vibe_action,
        VibeActionDispatcher, VibeActionState,
    },
    qualia_core_db::{
        vibe_host::{native_authority::graph::NativeGraphHost, vibe_sentinel::WebizenSentinelGate},
        NQuin,
    },
    ui_channel,
    vibe::{Budget, DiagCode, Diagnostic, Span, Value},
    vibescript_abi::{EffectSet, ResourceEnvelope},
    ExecutionLimits, HostError, NativeAuthoringView, NativeGraphBridge, NativeGraphView,
    QualiaRuntime, UiReceiver, UiSender,
};
use std::{
    cell::{Cell, RefCell},
    future::poll_fn,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    task::{Poll, Waker},
};
use webizen_ua_os::{
    authorize_qap_graph_read, authorize_qap_vibe_source, hypermedia_pillar_catalog,
    list_native_vibe_providers, poet_analyze_and_seal_gazetteer, poet_eval_native_cell,
    poet_open_lexicon_pack, produce_hypermedia_vibe, record_authoring_source_review,
    require_qap_capability, run_governed_vibe_source, AdoptedQualiaFoundation,
    CapabilityLeaseScope, GovernedVibeActionReport, PoetCellEntry, QapPoetGraphHost, UaOsHost,
    VibeExecutionReceiptSummary, DID_Q42_QRC_LABEL, HYPERMEDIA_MORPHISM_COUNT,
};

/// Report a recognised resource kind without retaining clipboard contents.
pub(super) fn resource_clipboard_notice(action: &str, text: Option<&str>) -> String {
    let Some(text) = text.map(str::trim).filter(|text| !text.is_empty()) else {
        return format!("{action}: no clipboard text available");
    };
    let kind = if text.contains(['\r', '\n']) {
        "text"
    } else {
        let normalized = text.to_ascii_lowercase();
        let path = normalized.split(['?', '#']).next().unwrap_or_default();
        if path.ends_with(".10d") {
            "10D resource reference"
        } else if path.ends_with(".gguf") {
            "GGUF model reference"
        } else if normalized.starts_with("qualia://") && path.len() > "qualia://".len() {
            "Qualia resource reference"
        } else if normalized.starts_with("https://") && path.len() > "https://".len() {
            "HTTPS resource reference"
        } else {
            "text"
        }
    };
    format!("{action}: {kind}")
}

#[component]
pub fn PoetSynthVolume(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let cell_lines: Vec<String> = s
        .poet_cells
        .iter()
        .map(|c| {
            format!(
                "{} [{}] => {} (rev {})",
                c.name, c.source, c.value, c.revision
            )
        })
        .collect();
    let cell_input = s.poet_cell_input.clone();
    let document_input = s.poet_doc_input.clone();
    let clipboard_toast = s.clipboard_toast.clone();
    let selection_label = s.selection_range_label.clone();
    let lexicon_status = s.poet_lexicon_status.clone();
    let gazetteer_summary = s.poet_gazetteer.as_ref().map(|g| {
        format!(
            "{} tokens · {} sentences · {} parity-verified 48-byte NQuin48 span records sealed",
            g.token_count,
            g.sentence_count,
            g.sealed_quins.len()
        )
    });
    let gazetteer_hits: Vec<String> = s
        .poet_gazetteer
        .as_ref()
        .map(|g| {
            g.hits
                .iter()
                .map(|h| {
                    format!(
                        "{} [{}..{}] -> <{}>",
                        h.surface, h.start_byte, h.end_byte, h.iri
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let llm_lines: Vec<String> = s
        .llm_models
        .iter()
        .take(12)
        .map(|m| format!("{} · {} · {} MiB", m.name, m.format, m.size_mib))
        .collect();
    let llm_count = s.llm_models.len();
    let hm_receipt = s.hypermedia_receipt.clone();
    drop(s);

    let providers = list_native_vibe_providers();
    let provider_count = providers.len();
    let provider_lines: Vec<String> = providers
        .iter()
        .map(|p| {
            format!(
                "{} · {} fns · {} static NQuins · <{}>",
                p.alias, p.function_count, p.ontology_nquins, p.canonical_iri
            )
        })
        .collect();
    let pillars = hypermedia_pillar_catalog();
    let pillar_count = pillars.len();
    let pillar_lines: Vec<String> = pillars
        .iter()
        .map(|p| {
            format!(
                "{} ({} ops · {}) — {}",
                p.domain, p.morphism_count, p.containers, p.description
            )
        })
        .collect();

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Poet Studio" }
                p { "Write a document, run a formula, and preview 3D work on the canvas." }
            }
            textarea {
                class: "input-field poet-editor-canvas",
                aria_label: "Document editor",
                value: "{document_input}",
                rows: "12",
                placeholder: "Start writing…",
                oninput: move |evt| {
                    let mut shell = state.write();
                    shell.poet_doc_input = evt.value();
                    shell.selection_range_label.clear();
                },
            }
            Holo10dViewport { state, specimen_label: "Studio canvas".to_string() }
            ThemeCodesignerPanel { state }
            SharedThemeCollectionPanel { state }
            details {
                class: "tech-details-drawer",
                summary { "Studio details" }
                dl {
                    class: "kv",
                    dt { "Lexicon" }
                    dd { "{lexicon_status}" }
                    dt { "Providers" }
                    dd { "{provider_count}" }
                    dt { "Catalog" }
                    dd { "{HYPERMEDIA_MORPHISM_COUNT} operations across {pillar_count} areas" }
                    dt { "Local models" }
                    dd { "{llm_count}" }
                }
            }
            form {
                class: "input-row",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    let (expr, rev) = {
                        let shell = state.read();
                        (shell.poet_cell_input.clone(), shell.poet_cells.len() as u64)
                    };
                    let res = poet_eval_native_cell(&expr, rev);
                    state.write().poet_cells.push(PoetCellEntry {
                        name: format!("cell_{}", rev + 1),
                        source: expr,
                        value: if res.ok {
                            res.value
                        } else {
                            res.diagnostic.clone().unwrap_or_else(|| "error".into())
                        },
                        ok: res.ok,
                        diagnostic: res.diagnostic,
                        revision: res.revision,
                    });
                },
                onreset: move |evt| {
                    evt.prevent_default();
                    state.write().poet_cell_input.clear();
                },
                input {
                    class: "input-field",
                    aria_label: "VibeScript cell formula",
                    value: "{cell_input}",
                    placeholder: "Enter VibeScript cell formula (e.g. = 64 * 16)…",
                    oninput: move |evt| state.write().poet_cell_input = evt.value(),
                    onchange: move |evt| state.write().poet_cell_input = evt.value(),
                    onpaste: move |evt| {
                        let text = evt.data().data_transfer().get_as_text();
                        state.write().clipboard_toast = resource_clipboard_notice("Paste", text.as_deref());
                    },
                    oncopy: move |evt| {
                        let text = evt.data().data_transfer().get_as_text();
                        state.write().clipboard_toast = resource_clipboard_notice("Copy", text.as_deref());
                    },
                    oncut: move |evt| {
                        let text = evt.data().data_transfer().get_as_text();
                        state.write().clipboard_toast = resource_clipboard_notice("Cut", text.as_deref());
                    },
                }
                button {
                    r#type: "submit",
                    class: "sanctum-issue",
                    "Evaluate Native VibeScript Cell"
                }
                button { r#type: "reset", class: "sanctum-issue", "Clear cell" }
            }
            if !clipboard_toast.is_empty() {
                p { class: "muted", role: "status", "{clipboard_toast}" }
            }
            GovernedAuthoringPanel { state }
            section {
                class: "id-list",
                h4 { "Gazetteer document" }
                textarea {
                    class: "input-field",
                    aria_label: "Gazetteer document",
                    value: "{document_input}",
                    rows: "6",
                    oninput: move |evt| {
                        let mut shell = state.write();
                        shell.poet_doc_input = evt.value();
                        shell.selection_range_label.clear();
                    },
                    onselect: move |evt| {
                        state.write().selection_range_label = evt.data().selection()
                            .filter(|selection| !selection.is_collapsed())
                            .map(|selection| {
                                let range = selection.range();
                                format!("Selected UTF-16 range {}..{}", range.start, range.end)
                            })
                            .unwrap_or_default();
                    },
                    onselectionchange: move |evt| {
                        if evt.data().selection().is_none_or(|selection| selection.is_collapsed()) {
                            state.write().selection_range_label.clear();
                        }
                    },
                }
                if !selection_label.is_empty() {
                    p { class: "muted", role: "status", "{selection_label}" }
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        if let Some(sess) = session() {
                            let doc = state.read().poet_doc_input.clone();
                            let gaz = poet_analyze_and_seal_gazetteer(&doc);
                            let lex = poet_open_lexicon_pack(&sess.host, "");
                            let receipt = produce_hypermedia_vibe(
                                &sess.host,
                                &state.read().hypermedia_script.clone(),
                                "poet-workbench-stage",
                            );
                            let mut s = state.write();
                            s.poet_gazetteer = Some(gaz);
                            s.poet_lexicon_status = if lex.ok {
                                lex.value
                            } else {
                                lex.diagnostic.unwrap_or_else(|| "Planned — lexicon pack".into())
                            };
                            s.hypermedia_receipt = Some(receipt);
                        }
                    },
                    "Run Integrated Hypermedia Production & Seal Gazetteer NQuins"
                }
            }
            if let Some(rec) = hm_receipt {
                {
                    let mode = rec.mode;
                    let summary = rec.output_summary;
                    rsx! {
                        section {
                            class: "id-list",
                            h4 { "Hypermedia Production Receipt ({mode})" }
                            p { class: "path", "{summary}" }
                        }
                    }
                }
            }
            section {
                class: "id-list",
                h4 { "Reactive VibeScript Cells (Native Poet DAG)" }
                ul {
                    for line in cell_lines {
                        li { "{line}" }
                    }
                }
            }
            if let Some(summary) = gazetteer_summary {
                section {
                    class: "id-list",
                    h4 { "Deterministic NLP Gazetteer & CML Span-to-NQuin48 Sealer" }
                    p { class: "muted", "{summary}" }
                    ul {
                        for hit_line in gazetteer_hits {
                            li { "{hit_line}" }
                        }
                    }
                }
            }
            section {
                class: "id-list",
                h4 { "12 Hypermedia Production Pillars ({HYPERMEDIA_MORPHISM_COUNT} Morphisms Integrated)" }
                ul {
                    for p_line in pillar_lines {
                        li { "{p_line}" }
                    }
                }
            }
            section {
                class: "id-list",
                h4 { "11 Native Etalon / Qualia VibeScript ABI Providers (TargetProfileMask::NATIVE_DESKTOP)" }
                ul {
                    for prov_line in provider_lines {
                        li { "{prov_line}" }
                    }
                }
            }
            if !llm_lines.is_empty() {
                section {
                    class: "id-list",
                    h4 { "Unified Local SLD / LLM Vault Models ({llm_count})" }
                    ul {
                        for m_line in llm_lines {
                            li { "{m_line}" }
                        }
                    }
                }
            }
        }
    }
}

const LOCAL_PROGRAM_ID: &str = "poet-workbench";

fn selected_poet_source(state: Signal<ShellState>, program: &str) -> String {
    if program.trim().is_empty() {
        state.peek().poet_cell_input.clone()
    } else {
        program.to_owned()
    }
}

#[component]
fn GovernedAuthoringPanel(mut state: Signal<ShellState>) -> Element {
    let mut program_draft = use_signal(String::new);
    let mut document_id = use_signal(String::new);
    let mut evidence_ref = use_signal(String::new);
    let mut execution_error = use_signal(String::new);
    let mut review_error = use_signal(String::new);
    let draft = program_draft.read().clone();
    let review_document = document_id.read().clone();
    let review_evidence = evidence_ref.read().clone();
    let execution_notice = execution_error.read().clone();
    let review_notice = review_error.read().clone();
    let shell = state.read();
    let last_report = shell.last_vibe_action_report.clone();
    let last_review = shell.last_authoring_snapshot.clone();
    drop(shell);

    rsx! {
        section {
            class: "id-list",
            h4 { "Governed VibeScript Action & Authoring Source Review (aether-qualia-host)" }
            p {
                "Run a local Sentinel rehearsal of the current cell or enter a VibeScript program. Source review records the selected source digest and evidence reference in the local journal."
            }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let source = selected_poet_source(state, &program_draft.peek());
                    let result = session().ok_or_else(|| "Start a local Webizen session to run a governed rehearsal.".to_owned())
                        .and_then(|session| run_governed_vibe_source(
                            &session.host.storage,
                            session.fabric_identifier(),
                            LOCAL_PROGRAM_ID,
                            &source,
                        ));
                    match result {
                        Ok(report) => {
                            state.write().last_vibe_action_report = Some(report);
                            execution_error.set(String::new());
                        }
                        Err(error) => {
                            state.write().last_vibe_action_report = None;
                            execution_error.set(error);
                        }
                    }
                },
                label { r#for: "governed-vibe-program", "Program source" }
                textarea {
                    id: "governed-vibe-program",
                    class: "input-field",
                    rows: "4",
                    placeholder: "Leave blank to use the VibeScript cell above",
                    value: "{draft}",
                    oninput: move |event| program_draft.set(event.value()),
                }
                button {
                    r#type: "submit",
                    class: "sanctum-issue",
                    "Run Sentinel rehearsal"
                }
            }
            if !execution_notice.is_empty() {
                p { role: "status", "{execution_notice}" }
            }
            if let Some(report) = last_report {
                {
                    let program = report.program_id;
                    let entry = report.entry_fn;
                    let epoch = report.epoch;
                    let value = report.formatted_value;
                    let graph_revision = report.graph_revision;
                    let graph_quins = report.graph_quin_count;
                    rsx! {
                        dl {
                            class: "kv",
                            dt { "Run" }
                            dd { "{program} · {entry} · epoch {epoch}" }
                            dt { "Value" }
                            dd { "{value}" }
                            dt { "Graph snapshot" }
                            dd { "Revision {graph_revision} · {graph_quins} NQuin records" }
                        }
                        if let Some(diagnostic) = report.diagnostic {
                            p { role: "status", "{diagnostic}" }
                        }
                        if report.reconciliation_required {
                            p { role: "alert", "Reconciliation is required for the retained execution receipts." }
                        }
                        if report.receipts.is_empty() {
                            p { class: "muted", "No provider execution receipts were produced." }
                        } else {
                            table {
                                caption { "VibeScript execution receipts" }
                                thead {
                                    tr {
                                        th { scope: "col", "Ticket" }
                                        th { scope: "col", "Actor hash" }
                                        th { scope: "col", "Ticks" }
                                        th { scope: "col", "Policy generation" }
                                        th { scope: "col", "Status" }
                                    }
                                }
                                tbody {
                                    for (index, receipt) in report.receipts.into_iter().enumerate() {
                                        {
                                            let ticket = receipt.ticket_id;
                                            let actor = format!("{:016x}", receipt.actor_did_hash);
                                            let ticks = receipt.ticks_consumed;
                                            let policy = receipt.policy_generation;
                                            let status = receipt.status_label;
                                            rsx! {
                                                tr {
                                                    key: "{ticket}-{index}",
                                                    td { "{ticket}" }
                                                    td { "{actor}" }
                                                    td { "{ticks}" }
                                                    td { "{policy}" }
                                                    td {
                                                        "{status}"
                                                        if receipt.simulated {
                                                            " · Local rehearsal"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let source = selected_poet_source(state, &program_draft.peek());
                    let result = session().ok_or_else(|| "Start a local Webizen session to record a source review.".to_owned())
                        .and_then(|session| record_authoring_source_review(
                            &session.host.storage,
                            session.fabric_identifier(),
                            &document_id.peek(),
                            &source,
                            &evidence_ref.peek(),
                        ));
                    match result {
                        Ok(snapshot) => {
                            state.write().last_authoring_snapshot = Some(snapshot);
                            review_error.set(String::new());
                        }
                        Err(error) => {
                            state.write().last_authoring_snapshot = None;
                            review_error.set(error);
                        }
                    }
                },
                label { r#for: "authoring-review-document", "Document ID" }
                input {
                    id: "authoring-review-document",
                    class: "input-field",
                    value: "{review_document}",
                    required: true,
                    oninput: move |event| document_id.set(event.value()),
                }
                label { r#for: "authoring-review-evidence", "Evidence reference" }
                input {
                    id: "authoring-review-evidence",
                    class: "input-field",
                    value: "{review_evidence}",
                    required: true,
                    oninput: move |event| evidence_ref.set(event.value()),
                }
                button {
                    r#type: "submit",
                    class: "sanctum-issue",
                    "Record source review request"
                }
            }
            if !review_notice.is_empty() {
                p { role: "status", "{review_notice}" }
            }
            if let Some(review) = last_review {
                {
                    let document = review.document_id;
                    let revision = review.revision;
                    let generation = review.authority_generation;
                    let parity = format!("{:016x}", review.sealed_nquin_parity);
                    let summary = review.status_summary;
                    rsx! {
                        p { role: "status", "{summary}" }
                        dl {
                            class: "kv",
                            dt { "Reviewed source" }
                            dd { "{document} · revision {revision}" }
                            dt { "Authority generation" }
                            dd { "{generation}" }
                            dt { "NQuin48 parity" }
                            dd { "{parity}" }
                        }
                    }
                }
            }
            NativePoetTools { state, program_draft }
            NativeAuthoringEvidence {}
        }
    }
}

#[derive(Clone, Default)]
struct PoetHostTick(Rc<PoetTickState>);

impl PartialEq for PoetHostTick {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Default)]
struct PoetTickState {
    ready: Cell<bool>,
    waiter: RefCell<Option<Waker>>,
}

impl PoetHostTick {
    fn request(&self) {
        self.0.ready.set(true);
        let waiter = self.0.waiter.borrow_mut().take();
        if let Some(waiter) = waiter {
            waiter.wake();
        }
    }

    async fn next(&self) -> bool {
        poll_fn(|task| {
            if self.0.ready.replace(false) {
                Poll::Ready(true)
            } else {
                *self.0.waiter.borrow_mut() = Some(task.waker().clone());
                Poll::Pending
            }
        })
        .await
    }
}

struct PoetHostView<T>(Rc<RefCell<Option<T>>>);

impl<T> PoetHostView<T> {
    fn new(view: T) -> Self {
        Self(Rc::new(RefCell::new(Some(view))))
    }

    fn take(self) -> T {
        self.0
            .borrow_mut()
            .take()
            .expect("owner-scoped Qualia view")
    }
}

impl<T> Clone for PoetHostView<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for PoetHostView<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

const ADOPTED_PROGRAM_ID: &str = "authorized-poet-source";
const ADOPTED_ENTRY: &str = "program";

struct PoetActionDispatcher {
    storage: PathBuf,
    adopted: AdoptedQualiaFoundation,
    sender: Mutex<Option<UiSender>>,
    worker_error: Mutex<Option<HostError>>,
    finished: AtomicBool,
    window: Arc<dyn Window>,
}

impl VibeActionDispatcher for ArcPoetDispatcher {
    fn dispatch_action(
        &self,
        program_id: &str,
        entry_fn: &str,
        mut args: Vec<Value>,
        cancelled: Arc<AtomicBool>,
    ) -> Result<u64, HostError> {
        if program_id != ADOPTED_PROGRAM_ID || entry_fn != ADOPTED_ENTRY || args.len() != 1 {
            return Err(HostError::MissingCapability("poet.admitted-source".into()));
        }
        let Some(Value::String(source)) = args.pop() else {
            return Err(HostError::MissingCapability("poet.admitted-source".into()));
        };
        if source.len() > ExecutionLimits::default().source_bytes {
            return Err(HostError::SourceTooLarge);
        }
        let sender = self
            .0
            .sender
            .lock()
            .map_err(|_| HostError::Closed)?
            .take()
            .ok_or(HostError::Closed)?;
        let dispatcher = self.0.clone();
        std::thread::Builder::new()
            .name("poet-qualified-source".into())
            .spawn(move || {
                if let Err(error) = execute_adopted_poet_source(
                    &dispatcher.storage,
                    &dispatcher.adopted,
                    sender,
                    &source,
                    cancelled,
                ) {
                    if let Ok(mut pending) = dispatcher.worker_error.lock() {
                        *pending = Some(error);
                    }
                }
                dispatcher.finished.store(true, Ordering::Release);
                dispatcher.window.request_redraw();
            })
            .map_err(|_| HostError::Closed)?;
        // Each child owns one fresh canonical channel, whose first epoch is zero.
        Ok(0)
    }
}

struct ArcPoetDispatcher(Arc<PoetActionDispatcher>);

fn execute_adopted_poet_source(
    storage: &Path,
    adopted: &AdoptedQualiaFoundation,
    sender: UiSender,
    source: &str,
    cancelled: Arc<AtomicBool>,
) -> Result<(), HostError> {
    QapPoetGraphHost::check_policy(storage).map_err(|message| {
        HostError::Diagnostic(Diagnostic::new(DiagCode::E300, Span::point(0), message))
    })?;
    let grant = adopted
        .graph_grant()
        .ok_or(HostError::MissingQualiaFoundation)?;
    if cancelled.load(Ordering::Acquire) {
        return Err(HostError::Cancelled);
    }
    let authority = adopted.authority();
    let scope = authority
        .bind_graph_scope(&adopted.session(), grant)
        .map_err(HostError::Graph)?;
    let mut limits = ExecutionLimits::default();
    let mut budget = Budget {
        steps_left: limits.steps,
        workspace_left: limits.workspace_bytes,
    };
    let native = NativeGraphHost::new_with_budget(authority, scope, cancelled.clone(), &mut budget)
        .map_err(HostError::Graph)?;
    limits.workspace_bytes = budget.workspace_left;
    let (host, context, generation) = QapPoetGraphHost::admit(storage, native, adopted, source)
        .map_err(|message| {
            HostError::Diagnostic(Diagnostic::new(DiagCode::E300, Span::point(0), message))
        })?;
    let mut capabilities = Vec::new();
    if grant.grant.rights.read {
        capabilities.push("graph.read".to_owned());
        if grant.grant.rights.replace {
            capabilities.push("graph.write".to_owned());
        }
    }
    let mut receipts = [NQuin::default(); 128];
    // Signed graph rights are checked in NativeGraphHost. Without separate adopted
    // deontic extension rules, the live Sentinel admits no extension calls.
    let gate = WebizenSentinelGate::new(
        &[],
        &mut receipts,
        generation,
        EffectSet::PURE_COMPUTE,
        ResourceEnvelope::hot_zero_heap(
            limits.steps,
            limits.extension_output_bytes.try_into().unwrap_or(u32::MAX),
        ),
    );
    let mut runtime = QualiaRuntime::new(host, gate, capabilities, limits, sender)?;
    let prepared = runtime.prepare(source)?;
    if cancelled.load(Ordering::Acquire) {
        return Err(HostError::Cancelled);
    }
    authorize_qap_vibe_source(storage, adopted, source).map_err(|message| {
        HostError::Diagnostic(Diagnostic::new(DiagCode::E300, Span::point(0), message))
    })?;
    runtime.execute(prepared, context)
}

#[derive(Clone)]
struct PoetActionOwner {
    dispatcher: Arc<PoetActionDispatcher>,
    receiver: PoetHostView<UiReceiver>,
}

#[component]
fn NativePoetTools(state: Signal<ShellState>, program_draft: Signal<String>) -> Element {
    let Some(adopted) = try_consume_context::<AdoptedQualiaFoundation>() else {
        return rsx! { p { class: "muted", "Awaiting signed source and graph admission for native actions" } };
    };
    if adopted.graph_grant().is_none() {
        return rsx! { p { class: "muted", "Awaiting a signed graph grant for native actions" } };
    }
    if session().is_none() {
        return rsx! { p { class: "muted", "Native Poet actions require an installed Qap session." } };
    }
    rsx! { ScopedPoetTools { state, program_draft } }
}

#[component]
fn ScopedPoetTools(mut state: Signal<ShellState>, program_draft: Signal<String>) -> Element {
    use_drop(move || {
        let contains_admitted_report = state
            .peek()
            .last_vibe_action_report
            .as_ref()
            .is_some_and(|report| report.program_id == ADOPTED_PROGRAM_ID);
        if contains_admitted_report {
            state.write().last_vibe_action_report = None;
        }
    });
    let run_slot = use_signal(|| 0u64);
    let slot = run_slot();
    let graph_summary = use_signal(|| None::<(u64, usize)>);
    let graph_tick = use_hook(PoetHostTick::default);
    rsx! {
        section {
            class: "id-list",
            h5 { "Adopted QualiaDB graph and source" }
            if state.read().blocks_live_qap_lease("Poet", CapabilityLeaseScope::QualiaQ42Storage) {
                p { role: "status", "The Poet Qap policy blocks graph access." }
            } else {
                NativeGraphEvidence { graph_tick: graph_tick.clone(), graph_summary }
            }
            NativePoetAction {
                key: "{slot}",
                state,
                program_draft,
                run_slot,
                graph_tick,
                graph_summary,
            }
        }
    }
}

#[component]
fn NativeGraphEvidence(
    graph_tick: PoetHostTick,
    graph_summary: Signal<Option<(u64, usize)>>,
) -> Element {
    let view = use_hook(|| {
        let adopted = try_consume_context::<AdoptedQualiaFoundation>()
            .ok_or_else(|| "Native graph authority is unavailable".to_owned())?;
        let storage = &session()
            .ok_or_else(|| "Open a local Poet Qap session.".to_owned())?
            .host
            .storage;
        authorize_qap_graph_read(storage, "Poet", &adopted)?;
        let grant = adopted
            .graph_grant()
            .ok_or_else(|| "A signed graph grant is required".to_owned())?;
        NativeGraphBridge::connect(adopted.authority(), &adopted.session(), grant)
            .and_then(NativeGraphBridge::subscribe)
            .map(PoetHostView::new)
            .map_err(|error| format!("{error:?}"))
    });
    match view {
        Ok(view) => rsx! { ScopedPoetGraph { view, graph_tick, graph_summary } },
        Err(error) => {
            rsx! { p { class: "muted", role: "status", "Graph snapshot unavailable · {error}" } }
        }
    }
}

#[component]
fn ScopedPoetGraph(
    view: PoetHostView<NativeGraphView>,
    graph_tick: PoetHostTick,
    mut graph_summary: Signal<Option<(u64, usize)>>,
) -> Element {
    let cancelled = use_hook(|| Arc::new(AtomicBool::new(false)));
    let wait_tick = graph_tick.clone();
    use_hook(|| graph_tick.request());
    let subscription = use_qualia_subscription(
        move || view.take(),
        cancelled,
        move || {
            let wait_tick = wait_tick.clone();
            async move {
                wait_tick.next().await
                    && session().is_some_and(|session| {
                        require_qap_capability(
                            &session.host.storage,
                            "Poet",
                            CapabilityLeaseScope::QualiaQ42Storage,
                        )
                        .is_ok()
                    })
            }
        },
        |snapshot| snapshot.quins.len(),
    );
    let snapshot = subscription.snapshot;
    use_effect(move || {
        let latest = snapshot
            .read()
            .as_ref()
            .map(|data| (data.read.revision, data.data));
        if *graph_summary.peek() != latest {
            graph_summary.set(latest);
        }
    });
    let status = format!("{:?}", *subscription.status.read());
    let current = *graph_summary.read();
    rsx! {
        p { role: "status", "Scoped graph · {status}" }
        if let Some((revision, count)) = current {
            p { "Revision {revision} · {count} NQuin records" }
        }
        button {
            r#type: "button",
            class: "sanctum-issue",
            onclick: move |_| graph_tick.request(),
            "Refresh scoped graph"
        }
    }
}

#[component]
fn NativePoetAction(
    mut state: Signal<ShellState>,
    program_draft: Signal<String>,
    mut run_slot: Signal<u64>,
    graph_tick: PoetHostTick,
    graph_summary: Signal<Option<(u64, usize)>>,
) -> Element {
    let window = aether_native::use_window();
    let owner = use_hook(move || {
        let adopted = try_consume_context::<AdoptedQualiaFoundation>()
            .expect("adopted source foundation for this component");
        let (sender, receiver) = ui_channel(1, || {}).expect("positive mailbox capacity");
        PoetActionOwner {
            dispatcher: Arc::new(PoetActionDispatcher {
                storage: session()
                    .expect("local Poet Qap session")
                    .host
                    .storage
                    .clone(),
                adopted,
                sender: Mutex::new(Some(sender)),
                worker_error: Mutex::new(None),
                finished: AtomicBool::new(false),
                window,
            }),
            receiver: PoetHostView::new(receiver),
        }
    });
    let dispatcher = owner.dispatcher.clone();
    use_context_provider(move || {
        let mut context = try_use_qualia_context().expect("adopted QualiaHostContext");
        context.action_dispatcher = Some(Arc::new(ArcPoetDispatcher(dispatcher)));
        context
    });
    let action = use_vibe_action(move || owner.receiver.take(), NonZeroUsize::MIN);
    let mut finish_action = action.clone();
    let mut worker_finished = use_signal(|| false);
    let worker = owner.dispatcher;
    aether_native::use_window_event(move |event, _| {
        if !matches!(event, WindowEvent::RedrawRequested)
            || !worker.finished.load(Ordering::Acquire)
        {
            return;
        }
        let pending = worker
            .worker_error
            .lock()
            .ok()
            .and_then(|mut error| error.take());
        if let Some(error) = pending {
            finish_action.record_host_error(error);
        }
        if !*worker_finished.peek() {
            worker_finished.set(true);
            graph_tick.request();
        }
    });

    let action_state = action.state;
    use_effect(move || {
        let finished = match &*action_state.read() {
            VibeActionState::Delivered(delivery) => Some((
                format!("{:?}", delivery.value),
                delivery.receipts.clone(),
                false,
                None,
            )),
            VibeActionState::NeedsReconcile {
                cause,
                committed_receipts,
            } => Some((
                String::new(),
                committed_receipts.clone(),
                true,
                Some(cause.clone()),
            )),
            VibeActionState::Failed(error) => {
                Some((String::new(), Vec::new(), false, Some(error.clone())))
            }
            _ => None,
        };
        if let Some((mut formatted_value, receipts, reconciliation_required, mut diagnostic)) =
            finished
        {
            let allowed = session().is_some_and(|session| {
                QapPoetGraphHost::check_policy(&session.host.storage).is_ok()
            });
            let (graph_revision, graph_quin_count) = if allowed {
                graph_summary.peek().unwrap_or_default()
            } else {
                formatted_value.clear();
                if diagnostic.is_none() {
                    diagnostic = Some(
                        "The Poet Qap lease revoked delivery. Retain any committed receipts."
                            .into(),
                    );
                }
                (0, 0)
            };
            state.write().last_vibe_action_report = Some(GovernedVibeActionReport {
                program_id: ADOPTED_PROGRAM_ID.into(),
                entry_fn: ADOPTED_ENTRY.into(),
                epoch: 0,
                formatted_value,
                receipts: receipts
                    .iter()
                    .map(VibeExecutionReceiptSummary::from)
                    .collect(),
                graph_revision,
                graph_quin_count,
                reconciliation_required,
                diagnostic,
            });
        }
    });

    let realized = action.state.read().clone();
    let shell = state.read();
    let lease_blocked = shell
        .blocks_live_qap_lease("Poet", CapabilityLeaseScope::GovernedVibeScriptEval)
        || shell.blocks_live_qap_lease("Poet", CapabilityLeaseScope::QualiaQ42Storage);
    drop(shell);
    let can_start = !lease_blocked && matches!(&realized, VibeActionState::Idle);
    let running = matches!(&realized, VibeActionState::Running { .. });
    let can_rearm = matches!(
        &realized,
        VibeActionState::Failed(_) | VibeActionState::Delivered(_)
    ) || (matches!(&realized, VibeActionState::Cancelled) && worker_finished());
    let status = match &realized {
        VibeActionState::Idle => "Ready for the exact admitted source",
        VibeActionState::Running { .. } => "Native action running",
        VibeActionState::Delivered(_) => "Native action delivered",
        VibeActionState::Failed(_) => "Native action failed",
        VibeActionState::NeedsReconcile { .. } => "Native action requires host reconciliation",
        VibeActionState::Cancelled => "Native action cancellation requested",
    };

    let mut dispatch = action.clone();
    let mut cancel = action;
    rsx! {
        p { role: "status", "{status}" }
        if lease_blocked {
            p { role: "status", "The installed Poet Qap policy blocks a live action." }
        }
        button {
            r#type: "button",
            class: "sanctum-issue",
            disabled: !can_start,
            onclick: move |_| {
                if session().is_none_or(|session| QapPoetGraphHost::check_policy(&session.host.storage).is_err()) {
                    return;
                }
                let source = selected_poet_source(state, &program_draft.peek());
                let _ = dispatch.dispatch(ADOPTED_PROGRAM_ID, ADOPTED_ENTRY, vec![Value::String(source)]);
            },
            "Run admitted source with signed graph scope"
        }
        if running {
            button {
                r#type: "button",
                class: "sanctum-issue",
                onclick: move |_| cancel.cancel(),
                "Cancel native action"
            }
        }
        if can_rearm {
            button {
                r#type: "button",
                class: "sanctum-issue",
                onclick: move |_| {
                    let next = run_slot.peek().saturating_add(1);
                    run_slot.set(next);
                },
                "Prepare another native action"
            }
        }
    }
}

#[component]
fn NativeAuthoringEvidence() -> Element {
    let view = use_hook(|| {
        try_use_qualia_context()
            .and_then(|context| context.authoring)
            .ok_or_else(|| "Awaiting authenticated source authoring session".to_owned())
            .and_then(|bridge| {
                bridge
                    .subscribe(64 * 1024)
                    .map(PoetHostView::new)
                    .map_err(|error| error.to_string())
            })
    });

    match view {
        Ok(view) => rsx! { AdoptedAuthoringEvidence { view } },
        Err(status) => rsx! { p { class: "muted", "{status}" } },
    }
}

#[component]
fn AdoptedAuthoringEvidence(view: PoetHostView<NativeAuthoringView>) -> Element {
    let tick = use_hook(PoetHostTick::default);
    let wait_tick = tick.clone();
    let bridge = use_authoring_bridge(
        move || view.take(),
        move || {
            let tick = wait_tick.clone();
            async move { tick.next().await }
        },
    );
    let mut request_id = use_signal(String::new);
    let mut evidence_ref = use_signal(String::new);
    let mut feedback = use_signal(String::new);
    let evidence = bridge.evidence.read().clone();
    let review = bridge.last_review.read().clone();
    let bridge_status = format!("{:?}", *bridge.status.read());
    let current_request_id = request_id.read().clone();
    let current_evidence_ref = evidence_ref.read().clone();
    let current_feedback = feedback.read().clone();
    let authoring_ready = evidence.as_ref().is_some_and(|value| {
        value.can_contribute
            && value.target.author_subject.as_deref() == Some(value.principal_subject.as_str())
    });
    let refresh_tick = tick.clone();

    rsx! {
        section {
            class: "id-list",
            h5 { "Authenticated authoring workflow" }
            p { role: "status", "{bridge_status}" }
            button {
                r#type: "button",
                class: "sanctum-issue",
                onclick: move |_| refresh_tick.request(),
                "Refresh workflow evidence"
            }
            if let Some(evidence) = evidence {
                {
                    let document = evidence.target.document_id;
                    let revision = evidence.target.revision;
                    let generation = evidence.authority_generation;
                    let requests = evidence.requests.len();
                    let decisions = evidence.reviews.len();
                    rsx! {
                        dl {
                            class: "kv",
                            dt { "Document revision" }
                            dd { "{document} · revision {revision}" }
                            dt { "Authority generation" }
                            dd { "{generation}" }
                            dt { "Source review activity" }
                            dd { "{requests} requests · {decisions} decisions" }
                        }
                    }
                }
            }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let result = bridge.request_source_review(&request_id.peek(), &evidence_ref.peek());
                    feedback.set(match result {
                        Ok(request) => format!("Source review request {} recorded.", request.id),
                        Err(error) => error.to_string(),
                    });
                    tick.request();
                },
                label { r#for: "native-authoring-request-id", "Request ID" }
                input {
                    id: "native-authoring-request-id",
                    class: "input-field",
                    value: "{current_request_id}",
                    required: true,
                    oninput: move |event| request_id.set(event.value()),
                }
                label { r#for: "native-authoring-evidence-ref", "Evidence reference" }
                input {
                    id: "native-authoring-evidence-ref",
                    class: "input-field",
                    value: "{current_evidence_ref}",
                    required: true,
                    oninput: move |event| evidence_ref.set(event.value()),
                }
                button {
                    r#type: "submit",
                    class: "sanctum-issue",
                    disabled: !authoring_ready,
                    "Request authenticated source review"
                }
            }
            if !current_feedback.is_empty() {
                p { role: "status", "{current_feedback}" }
            }
            if let Some(review) = review {
                {
                    let id = review.id;
                    let generation = review.authority_generation;
                    let responses = review.decision_ids.len();
                    rsx! {
                        p { "Request {id} · authority generation {generation} · {responses} recorded decisions" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AnatomyVolume(state: Signal<ShellState>) -> Element {
    let current_plan = state.read().active_launch_plan.clone();
    let plan = current_plan.or_else(|| session().and_then(|s| s.host.launch("Anatomy", None).ok()));
    let mut details_status = use_signal(String::new);
    let status = details_status.read().clone();

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Anatomy · Installed Qap (NativeRsxCard — 10D v2 HOLO Medical Atlas)" }
                p {
                    "Installed into storage/Qapps/Anatomy via install_package_atomic and resolved through LaunchPlan without APP_STATE. Couples 3D Holographic 10D v2 HOLO signed-distance fields, Visible Human cervical spine topology, and DICOM Part-10 LPS millimeter registration."
                }
            }
            Holo10dViewport { state, specimen_label: "Anatomy cervical spine specimen".to_string() }
            details {
                open: true,
                class: "id-list",
                onbeforetoggle: move |evt| {
                    if let Some(native) = evt.data().downcast::<aether_native::NativeToggleData>() {
                        let current = if native.0.old_state { "open" } else { "closed" };
                        let next = if native.0.new_state { "open" } else { "closed" };
                        details_status.set(format!("Package details changing from {current} to {next}"));
                    }
                },
                ontoggle: move |evt| {
                    if let Some(native) = evt.data().downcast::<aether_native::NativeToggleData>() {
                        let next = if native.0.new_state { "open" } else { "closed" };
                        details_status.set(format!("Package details {next}"));
                    }
                },
                summary { "Anatomy package details" }
                if let Some(p) = plan {
                    {
                        let pkg_id = p.package_id;
                        let disp = p.display_name;
                        let ver = p.version;
                        let coord = p.fabric_coordinate;
                        let kind_label = p.kind.label();
                        let dir_str = p.package_dir.display().to_string();
                        let shapes_str = p.required_shapes.join(", ");
                        let onts_str = p.required_ontologies.join(", ");
                        rsx! {
                            dl {
                                class: "kv",
                                dt { "Package ID" }
                                dd { "{pkg_id} ({disp})" }
                                dt { "Active Version" }
                                dd { "v{ver}" }
                                dt { "QRC Coordinate" }
                                dd { class: "path", "{coord} ({DID_Q42_QRC_LABEL})" }
                                dt { "Launch Kind" }
                                dd { "{kind_label}" }
                                dt { "Package Directory" }
                                dd { class: "path", "{dir_str}" }
                                dt { "Required SHACL Shapes" }
                                dd { "{shapes_str}" }
                                dt { "Required Ontologies" }
                                dd { "{onts_str}" }
                            }
                        }
                    }
                } else {
                    p { class: "muted", "Anatomy Qap ready to seed from bundled/qapps/Anatomy." }
                }
            }
            if !status.is_empty() {
                p { class: "muted", role: "status", "{status}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resource_clipboard_notice;

    #[test]
    fn clipboard_notice_reports_resource_kind_without_echoing_private_text() {
        for (input, kind) in [
            ("qualia://private/context", "Qualia resource reference"),
            ("https://example.test/part", "HTTPS resource reference"),
            (
                "https://example.test/model.10D?key=private",
                "10D resource reference",
            ),
            (
                "qualia://private/model.gguf#version",
                "GGUF model reference",
            ),
            ("a private note", "text"),
            ("private text\nhttps://example.test/model.10d", "text"),
        ] {
            assert_eq!(
                resource_clipboard_notice("Paste", Some(input)),
                format!("Paste: {kind}")
            );
        }
    }
}
