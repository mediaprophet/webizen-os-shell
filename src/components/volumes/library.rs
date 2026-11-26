// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::poet_synth::resource_clipboard_notice;
use crate::components::viewports::Holo10dViewport;
use crate::state::{session, ShellState};
use aether::prelude::*;
use webizen_ua_os::{
    export_semantic_entry_to_solid_turtle, import_solid_turtle_to_semantic_fs, may_export_entry,
    recommend_toolboxes_for_entry, seal_and_inspect_qecp_envelope, SemanticAssetEntry,
    SemanticSection, SemanticToolboxId,
};

#[component]
pub fn LibraryVolume(mut state: Signal<ShellState>) -> Element {
    let mut guide_open = use_signal(|| false);
    let mut guide_loaded = use_signal(|| None);
    let s = state.read();
    let filter = s.library_filter.clone();
    let (matched, tallies) = s.semantic_fs.query(&filter);
    let selected_uri = if s.selected_library_uri.is_empty() {
        matched
            .first()
            .map(|e| e.asset_uri.clone())
            .unwrap_or_default()
    } else {
        s.selected_library_uri.clone()
    };
    let selected_entry = matched
        .iter()
        .find(|e| e.asset_uri == selected_uri)
        .cloned()
        .or_else(|| matched.first().cloned());
    let portability_entry = selected_entry.clone();

    let _store_path = s.semantic_fs.store_path.display().to_string();
    let total_entries = s.semantic_fs.entries.len();
    let match_count = tallies.matching_entries;
    let _total_quins = tallies.total_quins;
    let text_query = filter.text_query.clone();
    let active_sec = filter.section.clone();
    let active_entity = filter.entity_depicts.clone().unwrap_or_default();
    let active_topic = filter.topic.clone().unwrap_or_default();
    let active_project = filter.project.clone().unwrap_or_default();

    let viewport_label = selected_entry
        .as_ref()
        .map(|entry| entry.title.clone())
        .unwrap_or_else(|| "Library specimen".into());
    let is_holo = |uri: &str, media_type: &str| {
        matches!(
            media_type,
            "model/qualia-10d+holo" | "application/x-qualia-holo"
        ) || uri
            .split(['?', '#'])
            .next()
            .is_some_and(|path| path.to_ascii_lowercase().ends_with(".10d"))
    };
    let viewport_specimen_id = selected_entry
        .as_ref()
        .and_then(|entry| {
            if is_holo(&entry.asset_uri, &entry.media_type) {
                return Some(entry.asset_uri.clone());
            }
            entry
                .bundled_assets
                .iter()
                .find(|asset| is_holo(&asset.uri, &asset.media_type))
                .map(|asset| asset.uri.clone())
        })
        .unwrap_or_default();
    let clipboard_toast = s.clipboard_toast.clone();
    let ingest_uri = s.ingest_uri.clone();
    let ingest_title = s.ingest_title.clone();
    let ingest_body = s.ingest_body.clone();
    let ingest_entity = s.ingest_entity.clone();
    let ingest_sens = s.ingest_sensitivity.clone();
    let enabled_toolboxes = s.semantic_fs.enabled_toolboxes.clone();
    let spatiotemporal_enabled = s
        .semantic_fs
        .is_toolbox_enabled(SemanticToolboxId::SpatiotemporalAndGeodesy);
    let active_min_time = filter.min_occurred_unix;
    drop(s);

    let recommended_boxes = selected_entry
        .as_ref()
        .map(recommend_toolboxes_for_entry)
        .unwrap_or_default();

    let toolbox_cards: Vec<(
        SemanticToolboxId,
        String,
        String,
        String,
        String,
        bool,
        bool,
    )> = SemanticToolboxId::ALL
        .iter()
        .copied()
        .map(|tb_id| {
            let spec = tb_id.spec();
            let is_on = enabled_toolboxes.contains(&tb_id);
            let is_rec = recommended_boxes.contains(&tb_id);
            let title = spec.title.to_string();
            let shed = format!("Poet Shed Family: {}", spec.poet_shed_family);
            let facets = format!("Unlocks Facets: {}", spec.unlocked_facets.join(" · "));
            let ops = format!(
                "{} — ToolChain: {}",
                spec.description,
                spec.toolchain_ops.join(" -> ")
            );
            (tb_id, title, shed, facets, ops, is_on, is_rec)
        })
        .collect();

    let section_buttons: Vec<(String, String, bool)> = SemanticSection::ALL
        .iter()
        .map(|sec| {
            let key = sec.as_str().to_string();
            let count = tallies.sections.get(&key).copied().unwrap_or(0);
            let label = format!("{} ({})", sec.label(), count);
            let is_sel = active_sec.eq_ignore_ascii_case(&key);
            (key, label, is_sel)
        })
        .collect();

    let entity_chips: Vec<(String, String, bool)> = tallies
        .depicts_entities
        .iter()
        .map(|(ent, count)| {
            let is_sel = active_entity.eq_ignore_ascii_case(ent);
            (ent.clone(), format!("{ent} ({count})"), is_sel)
        })
        .collect();

    let topic_chips: Vec<(String, String, bool)> = tallies
        .topics
        .iter()
        .map(|(top, count)| {
            let is_sel = active_topic.eq_ignore_ascii_case(top);
            (top.clone(), format!("{top} ({count})"), is_sel)
        })
        .collect();

    let project_chips: Vec<(String, String, bool)> = tallies
        .projects
        .iter()
        .map(|(proj, count)| {
            let is_sel = active_project.eq_ignore_ascii_case(proj);
            (proj.clone(), format!("{proj} ({count})"), is_sel)
        })
        .collect();

    rsx! {
        div {
            class: "settings-shell",

            // Left Rail: Ontologically Defined Purpose-Shaped Sections (Not Folders)
            aside {
                class: "settings-sidebar",
                aria_label: "Semantic File-System Sections",
                h4 { "Semantic Sections" }
                input {
                    class: "settings-search",
                    r#type: "search",
                    value: "{text_query}",
                    placeholder: "Search meaning, entities, URI…",
                    aria_label: "Search semantic file-system",
                    oninput: move |evt| {
                        state.write().library_filter.text_query = evt.value();
                    },
                }
                for (sec_key, sec_label, is_sel) in section_buttons {
                    {
                        let tab_class = if is_sel { "settings-tab active" } else { "settings-tab" };
                        let key_clone = sec_key.clone();
                        rsx! {
                            button {
                                key: "{sec_key}",
                                r#type: "button",
                                class: "{tab_class}",
                                onclick: move |_| {
                                    state.write().library_filter.section = key_clone.clone();
                                },
                                span { "{sec_label}" }
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.library_filter.section = "all".into();
                        s.library_filter.text_query.clear();
                        s.library_filter.entity_depicts = None;
                        s.library_filter.topic = None;
                        s.library_filter.purpose = None;
                        s.library_filter.project = None;
                        s.library_filter.place = None;
                        s.library_filter.min_occurred_unix = None;
                    },
                    "Reset All Facets"
                }
            }

            // Main Stage: Opt-In Poet ToolChest + Faceted Entity/Feature Discovery + Resource List + Deep NQuin Graph Inspector
            div {
                class: "settings-content",
                header {
                    class: "settings-head",
                    h3 { "Library" }
                    p { "Browse files, pages, and 3D models. Click a card to preview it." }
                }
                p {
                    class: "muted",
                    "Showing {match_count} of {total_entries} items"
                }

                div {
                    class: "library-split-workspace",
                    section {
                        class: "id-list",
                        h4 { "Your files, books & 3D models" }
                        div {
                            class: "plane-grid",
                            for item in matched.iter() {
                                {
                                    let uri = item.asset_uri.clone();
                                    let title = item.title.clone();
                                    let media = item.media_type.clone();
                                    let badge = if is_holo(&item.asset_uri, &item.media_type) {
                                        "3D Holographic Model (.10d)"
                                    } else if media.contains("pdf") {
                                        "Verified Credential PDF"
                                    } else if media.contains("text") || media.contains("markdown") {
                                        "Document"
                                    } else {
                                        "Library item"
                                    };
                                    let topics = item.topics.join(" · ");
                                    rsx! {
                                        div {
                                            key: "{uri}",
                                            class: "plane-card",
                                            h5 { "{title}" }
                                            p { class: "muted", "{badge}" }
                                            if !topics.is_empty() {
                                                p { "{topics}" }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "sanctum-issue",
                                                onclick: move |_| {
                                                    state.write().selected_library_uri = uri.clone();
                                                },
                                                "Inspect Container Graph & Lineage"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        Holo10dViewport {
                            state,
                            specimen_label: viewport_label,
                            specimen_id: viewport_specimen_id,
                        }
                        LibraryPortabilityPanel { state, selected: portability_entry }
                        button {
                            r#type: "button",
                            aria_expanded: if guide_open() { "true" } else { "false" },
                            onclick: move |_| {
                                let opening = !guide_open();
                                guide_loaded.set(None);
                                guide_open.set(opening);
                            },
                            if guide_open() { "Hide Library reference sheet" }
                            else { "Show Library reference sheet" }
                        }
                if guide_open() {
                    img {
                        src: "aether://app/assets/library-sheet.png",
                        alt: "Semantic Library reference sheet",
                        style: "max-width: 100%; height: auto",
                        onload: move |event| guide_loaded.set(Some(!event.data().load_error())),
                        onerror: move |_| guide_loaded.set(Some(false)),
                    }
                    match guide_loaded() {
                        None => rsx! { p { role: "status", "Loading the Library reference sheet…" } },
                        Some(false) => rsx! { p { role: "status", "The packaged Library reference sheet could not be loaded." } },
                        Some(true) => rsx! {},
                    }
                }
                    }
                }

                details {
                    class: "tech-details-drawer",
                    open: true,
                    summary { "Opt-in specialist tools & graph details" }
                // Opt-In Semantic ToolChest (Poet Shed Pattern)
                section {
                    class: "id-list",
                    h4 { "Opt-In Semantic ToolChest (Poet Shed Pattern — Elect Toolboxes to Enable Specialized Extractors & Facets)" }
                    div {
                        class: "plane-grid",
                        for (tb_id, tb_title, tb_shed, tb_facets, tb_ops, is_on, is_rec) in toolbox_cards {
                            {
                                let state_badge = if is_on { "ENABLED" } else { "OPT-IN" };
                                let rec_badge = if is_rec { " · RECOMMENDED FOR SELECTED" } else { "" };
                                let btn_label = if is_on {
                                    "Disable Toolbox"
                                } else {
                                    "Enable Toolbox"
                                };
                                rsx! {
                                    div {
                                        key: "{tb_title}",
                                        class: "plane-card",
                                        h5 { "[{state_badge}{rec_badge}] {tb_title}" }
                                        p { "{tb_shed}" }
                                        p { "{tb_facets}" }
                                        p { "{tb_ops}" }
                                        button {
                                            r#type: "button",
                                            class: "sanctum-issue",
                                            onclick: move |_| {
                                                state.write().semantic_fs.toggle_toolbox(tb_id);
                                            },
                                            "{btn_label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Optional Temporal Timeline Window Filter (Unlocked when SpatiotemporalAndGeodesy Toolbox is Enabled)
                if spatiotemporal_enabled {
                    {
                        let all_active = active_min_time.is_none();
                        let mid_active = active_min_time == Some(1_763_600_000);
                        let recent_active = active_min_time == Some(1_763_700_000);
                        rsx! {
                            section {
                                class: "id-list",
                                h4 { "Temporal Timeline Window & Allen Interval Filter (Unlocked by Spatiotemporal & Geodesy Toolbox)" }
                                div {
                                    class: "orb-sectors",
                                    button {
                                        r#type: "button",
                                        class: if all_active { "ribbon-btn active" } else { "ribbon-btn" },
                                        onclick: move |_| state.write().library_filter.min_occurred_unix = None,
                                        "All Timeline Epochs"
                                    }
                                    button {
                                        r#type: "button",
                                        class: if mid_active { "ribbon-btn active" } else { "ribbon-btn" },
                                        onclick: move |_| state.write().library_filter.min_occurred_unix = Some(1_763_600_000),
                                        "Timeline >= 1763600000 (Talbingo & Microgrid Epoch)"
                                    }
                                    button {
                                        r#type: "button",
                                        class: if recent_active { "ribbon-btn active" } else { "ribbon-btn" },
                                        onclick: move |_| state.write().library_filter.min_occurred_unix = Some(1_763_700_000),
                                        "Timeline >= 1763700000 (Latest QECP, SI-05 & AI Vault Epoch)"
                                    }
                                }
                            }
                        }
                    }
                }

                // Facet Dimension 1: Entities & Features Inside the Asset (urn:qualia:hypermedia:depicts)
                section {
                    class: "id-list",
                    h4 { "Entity & Feature Facets Inside Assets (urn:qualia:hypermedia:depicts)" }
                    div {
                        class: "wheel-sectors",
                        for (ent_key, ent_label, is_sel) in entity_chips {
                            {
                                let btn_class = if is_sel { "ribbon-btn active" } else { "ribbon-btn" };
                                let k = ent_key.clone();
                                rsx! {
                                    button {
                                        key: "{ent_key}",
                                        r#type: "button",
                                        class: "{btn_class}",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            if s.library_filter.entity_depicts.as_deref() == Some(k.as_str()) {
                                                s.library_filter.entity_depicts = None;
                                            } else {
                                                s.library_filter.entity_depicts = Some(k.clone());
                                            }
                                        },
                                        "{ent_label}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Facet Dimension 2: Ontological Topics & Projects
                section {
                    class: "id-list",
                    h4 { "Topic & Project Facets (AND across dimensions · OR within dimension)" }
                    div {
                        class: "wheel-sectors",
                        for (top_key, top_label, is_sel) in topic_chips {
                            {
                                let btn_class = if is_sel { "ribbon-btn active" } else { "ribbon-btn" };
                                let k = top_key.clone();
                                rsx! {
                                    button {
                                        key: "{top_key}",
                                        r#type: "button",
                                        class: "{btn_class}",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            if s.library_filter.topic.as_deref() == Some(k.as_str()) {
                                                s.library_filter.topic = None;
                                            } else {
                                                s.library_filter.topic = Some(k.clone());
                                            }
                                        },
                                        "topic:{top_label}"
                                    }
                                }
                            }
                        }
                        for (proj_key, proj_label, is_sel) in project_chips {
                            {
                                let btn_class = if is_sel { "ribbon-btn active" } else { "ribbon-btn" };
                                let k = proj_key.clone();
                                rsx! {
                                    button {
                                        key: "{proj_key}",
                                        r#type: "button",
                                        class: "{btn_class}",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            if s.library_filter.project.as_deref() == Some(k.as_str()) {
                                                s.library_filter.project = None;
                                            } else {
                                                s.library_filter.project = Some(k.clone());
                                            }
                                        },
                                        "project:{proj_label}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Deep Semantic Container Inspector (Primary, Bundled Roles, Lineage, Depicted Entities, Analytics, CML)
                if let Some(entry) = selected_entry {
                    {
                        let e_uri = entry.asset_uri.clone();
                        let e_uri_for_exec = entry.asset_uri.clone();
                        let e_title = entry.title.clone();
                        let e_subj = format!("{:#018x}", entry.primary_subject);
                        let e_parity = format!("{:#018x}", entry.graph_parity_digest);
                        let e_media = entry.media_type.clone();
                        let e_sec = entry.section.clone();
                        let e_sens = entry.sensitivity.clone();
                        let e_commons = entry.commons_visibility.as_str();
                        let e_excerpt = entry.excerpt.clone();
                        let e_depicts = entry.depicts.join(" · ");
                        let e_topics = entry.topics.join(" · ");
                        let e_purposes = entry.purposes.join(" · ");
                        let e_projects = entry.projects.join(" · ");
                        let e_derived = entry.derived_from.join(" · ");
                        let e_signals = entry.cml_signals.join(" · ");
                        let e_concepts = entry.cml_concept_count;
                        let e_cof = entry.cof_segment_count;
                        let e_quins = entry.sealed_nquin_count;
                        let bundled_lines: Vec<String> = entry
                            .bundled_assets
                            .iter()
                            .map(|b| {
                                format!(
                                    "[Role: {}] {} ({}) — {}",
                                    b.role.label(),
                                    b.uri,
                                    b.media_type,
                                    b.summary
                                )
                            })
                            .collect();
                        let analytics_lines: Vec<String> = entry
                            .analytics
                            .iter()
                            .map(|a| format!("{} [method: {}] — {}", a.id, a.method, a.summary))
                            .collect();

                        rsx! {
                            section {
                                class: "id-list",
                                h4 { "Inspected Semantic Container · {e_title}" }
                                p { class: "muted", "{e_excerpt}" }
                                button {
                                    r#type: "button",
                                    class: "sanctum-issue",
                                    onclick: move |_| {
                                        state
                                            .write()
                                            .semantic_fs
                                            .execute_enabled_toolboxes_on_entry(&e_uri_for_exec);
                                    },
                                    "Execute Active ToolChest Toolboxes on This Container (Enrich Facets & Reseal NQuin48 Graph)"
                                }
                                dl {
                                    class: "kv",
                                    dt { "Container URI" }
                                    dd { class: "path", "{e_uri}" }
                                    dt { "60-Bit FNV-1a Subject" }
                                    dd { class: "path", "{e_subj} (shared across .10d / .q42 / .hmc)" }
                                    dt { "Media Type & Section" }
                                    dd { "{e_media} · Section: {e_sec} · Sensitivity: {e_sens} · Commons Scope: {e_commons}" }
                                    dt { "Entities & Features Inside" }
                                    dd { "{e_depicts}" }
                                    dt { "Topics / Purposes / Projects" }
                                    dd { "Topics: {e_topics} | Purposes: {e_purposes} | Projects: {e_projects}" }
                                    dt { "W3C PROV Lineage" }
                                    dd { class: "path", "prov:wasDerivedFrom -> {e_derived}" }
                                    dt { "CML & COF Extraction" }
                                    dd { "{e_concepts} CML concepts · Signals: [{e_signals}] · {e_cof} token-bounded COF HTML+RDFa segments" }
                                    dt { "48-Byte NQuin48 Graph" }
                                    dd { class: "path", "{e_quins} parity-checked edges · XOR parity accumulator: {e_parity}" }
                                }
                                h4 { "Bundled Sub-Assets & Ontological Roles (urn:qualia:hypermedia:bundledWith)" }
                                ul {
                                    for b_line in bundled_lines {
                                        li { "{b_line}" }
                                    }
                                }
                                h4 { "Bound Analytics & Verification Receipts (urn:qualia:hypermedia:analysisTarget)" }
                                ul {
                                    for a_line in analytics_lines {
                                        li { "{a_line}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Live Semantic Asset Ingestion Bar (Ingest Derives Searchability + Entity/Feature Extraction)
                section {
                    class: "id-list",
                    h4 { "Ingest New Resource into Semantic File-System (Derives Topics, Depicted Entities, CML & NQuin48 Edges)" }
                    form {
                        onsubmit: move |evt| {
                            evt.prevent_default();
                            let mut s = state.write();
                            let uri = s.ingest_uri.trim().to_string();
                            let title = s.ingest_title.trim().to_string();
                            let body = s.ingest_body.trim().to_string();
                            let entity = s.ingest_entity.trim().to_string();
                            let sens = s.ingest_sensitivity.clone();
                            if !uri.is_empty() && !body.is_empty() {
                                let manual_entities = if entity.is_empty() {
                                    Vec::new()
                                } else {
                                    vec![entity]
                                };
                                let entry = s.semantic_fs.ingest_semantic_asset(
                                    &uri,
                                    &title,
                                    "application/x-qualia-hmc",
                                    &body,
                                    &manual_entities,
                                    Some("user-ingested"),
                                    Some("Local Apparatus Keep"),
                                    &sens,
                                );
                                s.selected_library_uri = entry.asset_uri;
                            }
                        },
                        onreset: move |evt| {
                            evt.prevent_default();
                            let mut shell = state.write();
                            shell.ingest_uri.clear();
                            shell.ingest_title.clear();
                            shell.ingest_body.clear();
                            shell.ingest_entity.clear();
                        },
                        div {
                            class: "input-row",
                            input {
                                class: "input-field",
                                value: "{ingest_uri}",
                                placeholder: "Asset URI (e.g. urn:qualia:asset:my-specimen.10d)",
                                aria_label: "Asset URI",
                                oninput: move |evt| state.write().ingest_uri = evt.value(),
                                onchange: move |evt| state.write().ingest_uri = evt.value(),
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
                            input {
                                class: "input-field",
                                value: "{ingest_title}",
                                placeholder: "Resource title…",
                                oninput: move |evt| state.write().ingest_title = evt.value(),
                            }
                            input {
                                class: "input-field",
                                value: "{ingest_entity}",
                                placeholder: "Depicted entity / internal feature…",
                                oninput: move |evt| state.write().ingest_entity = evt.value(),
                            }
                        }
                        div {
                            class: "input-row",
                            input {
                                class: "input-field",
                                value: "{ingest_body}",
                                placeholder: "Text, caption, or semantic summary for CML & topic extraction…",
                                oninput: move |evt| state.write().ingest_body = evt.value(),
                            }
                            button {
                                r#type: "button",
                                class: "sanctum-issue",
                                onclick: move |_| {
                                    let cur = state.read().ingest_sensitivity.clone();
                                    state.write().ingest_sensitivity = if cur == "classified" {
                                        "public".into()
                                    } else {
                                        "classified".into()
                                    };
                                },
                                "Sensitivity: {ingest_sens}"
                            }
                            button {
                                r#type: "submit",
                                class: "sanctum-issue",
                                "Ingest & Seal NQuin48 Container"
                            }
                            button { r#type: "reset", class: "sanctum-issue", "Clear ingestion draft" }
                        }
                    }
                    if !clipboard_toast.is_empty() {
                        p { class: "muted", role: "status", "{clipboard_toast}" }
                    }
                }
                }
            }
        }
    }
}

#[component]
fn LibraryPortabilityPanel(
    mut state: Signal<ShellState>,
    selected: Option<SemanticAssetEntry>,
) -> Element {
    let mut endpoint = use_signal(String::new);
    let mut turtle = use_signal(String::new);
    let mut claimed_issuer = use_signal(String::new);
    let mut claim = use_signal(String::new);
    let mut status = use_signal(String::new);
    let export_allowed = selected.as_ref().is_some_and(may_export_entry);
    let certificate_title = selected.as_ref().map(|entry| entry.title.clone());
    let title_label = certificate_title
        .clone()
        .unwrap_or_else(|| "Select a Library resource".into());
    let can_seal = certificate_title.is_some();
    let last_solid = state.read().last_solid_receipt.clone();
    let last_qecp = state.read().last_qecp_envelope.clone();

    rsx! {
        section {
            class: "id-list",
            h4 { "QECP Credential & Solid Turtle Bridge" }
            p { "Selected resource · {title_label}" }
            label {
                "HTTPS Pod address (optional, offline metadata)"
                input {
                    class: "input-field",
                    value: "{endpoint}",
                    oninput: move |event| endpoint.set(event.value()),
                }
            }
            button {
                r#type: "button",
                class: "sanctum-issue",
                disabled: !export_allowed,
                onclick: move |_| {
                    let result = session()
                        .zip(selected.as_ref())
                        .ok_or_else(|| "Select a resource in an active local session.".to_string())
                        .and_then(|(device, entry)| {
                            export_semantic_entry_to_solid_turtle(&device.host.storage, entry, endpoint().trim())
                        });
                    match result {
                        Ok(receipt) => {
                            turtle.set(receipt.turtle_document.clone());
                            state.write().last_solid_receipt = Some(receipt);
                            status.set("Wrote a local Turtle export for the public resource.".into());
                        }
                        Err(error) => status.set(error),
                    }
                },
                "Export selected resource to Turtle"
            }
            if !export_allowed {
                p { class: "muted", "Export requires explicit public Permissive Commons disclosure." }
            }
            label {
                "Turtle source"
                textarea {
                    class: "input-field",
                    rows: "5",
                    value: "{turtle}",
                    oninput: move |event| turtle.set(event.value()),
                }
            }
            button {
                r#type: "button",
                class: "sanctum-issue",
                disabled: turtle().trim().is_empty(),
                onclick: move |_| {
                    if session().is_none() {
                        status.set("Open a local session to import RDF metadata.".into());
                        return;
                    }
                    let mut shell = state.write();
                    match import_solid_turtle_to_semantic_fs(&mut shell.semantic_fs, &turtle(), endpoint().trim()) {
                        Ok(receipt) => {
                            shell.last_solid_receipt = Some(receipt);
                            status.set("Imported RDF metadata as a private Library record.".into());
                        }
                        Err(error) => status.set(error),
                    }
                },
                "Import Turtle locally"
            }
            label {
                "Claimed pairwise issuer DID"
                input {
                    class: "input-field",
                    value: "{claimed_issuer}",
                    oninput: move |event| claimed_issuer.set(event.value()),
                }
            }
            label {
                "Credential claim"
                input {
                    class: "input-field",
                    value: "{claim}",
                    oninput: move |event| claim.set(event.value()),
                }
            }
            button {
                r#type: "button",
                class: "sanctum-issue",
                disabled: !can_seal || claimed_issuer().trim().is_empty() || claim().trim().is_empty(),
                onclick: move |_| {
                    let result = session()
                        .zip(certificate_title.as_deref())
                        .ok_or_else(|| "Select a resource in an active local session.".to_string())
                        .and_then(|(device, title)| {
                            seal_and_inspect_qecp_envelope(
                                &device.host.storage,
                                title,
                                claimed_issuer().trim(),
                                claim().trim(),
                            )
                        });
                    match result {
                        Ok(envelope) => {
                            state.write().last_qecp_envelope = Some(envelope);
                            status.set("Saved an unsigned QECP companion for external review.".into());
                        }
                        Err(error) => status.set(error),
                    }
                },
                "Seal & inspect local QECP companion"
            }
            p { role: "status", "{status}" }
            if let Some(receipt) = last_solid {
                {
                    let parity = format!("{:016x}", receipt.sealed_nquin_parity);
                    let path = receipt.file_path.map(|path| path.display().to_string()).unwrap_or_default();
                    rsx! {
                        p { "{receipt.direction} · {receipt.triple_count} triples · NQuin48 parity {parity}" }
                        if !path.is_empty() { p { "{path}" } }
                    }
                }
            }
            if let Some(envelope) = last_qecp {
                {
                    let parity = format!("{:016x}", envelope.sealed_nquin_parity);
                    rsx! {
                        p { "{envelope.c2pa_jumbf_label} · NQuin48 parity {parity}" }
                        p { "{envelope.credential_id} · {envelope.signature_scheme}" }
                        p { "{envelope.origin_integrity_disclaimer}" }
                    }
                }
            }
        }
    }
}
