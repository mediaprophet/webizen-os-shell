// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use crate::components::viewports::Holo10dViewport;
use crate::state::{session, ShellState};
use aether::prelude::*;
use webizen_ua_os::{
    hypermedia_pillar_catalog, list_native_vibe_providers, poet_analyze_and_seal_gazetteer,
    poet_eval_native_cell, poet_open_lexicon_pack, produce_hypermedia_vibe, PoetCellEntry,
    UaOsHost, DID_Q42_QRC_LABEL, HYPERMEDIA_MORPHISM_COUNT,
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
                h3 { "Poet · Unified Native Authoring & Hypermedia Production (Synth) Workbench" }
                p {
                    "Runs VibeScript (vibe-0.1 / proposed 0.2) reactive cells, deterministic NLP gazetteer NQuin48 sealing, and the {HYPERMEDIA_MORPHISM_COUNT}-morphism Hypermedia Production catalog directly in native mode (TargetProfileMask::NATIVE_DESKTOP) — no separate Synth app required."
                }
            }
            dl {
                class: "kv",
                dt { "VibeScript Engine" }
                dd { "vibe-0.1 / vibe-host-0.1 · 42 MiB SlgArena Sentinel budget · Zero V8 / Zero JIT" }
                dt { "Lexicon Pack" }
                dd { "{lexicon_status}" }
                dt { "Native Etalon Providers" }
                dd { "{provider_count} namespaces admitted on TargetProfileMask::NATIVE_DESKTOP" }
                dt { "Hypermedia Catalog" }
                dd { "{HYPERMEDIA_MORPHISM_COUNT} hypermedia.* morphisms across {pillar_count} domain pillars (.hmc / .10d v2 HOLO / .m64 / .emf / .qac / .q42)" }
                dt { "Unified SLD/LLM Vault" }
                dd { "{llm_count} local models discovered (QUALIA_LLM_VAULT / C:\\LLM_files / Models)" }
            }
            Holo10dViewport { state, specimen_label: "Poet authoring specimen".to_string() }
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
