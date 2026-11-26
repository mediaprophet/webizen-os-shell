// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use aether::prelude::*;
use aether::refract::{
    AssertionEvidenceLevel, AssertionKind, ChallengeState, ContextualAssertionView,
};

use super::poet_synth::resource_clipboard_notice;
use crate::components::viewports::BrowserChoraSurface;
use crate::state::{session, ShellState};
use webizen_ua_os::{
    evaluate_hybrid_page_request_in_storage, resolve_mode_for_uri,
    upgrade_browser_product_independent, BrowserExecutionMode, HybridWebCapabilityChannel,
    SocialDisclosureScope, UaOsHost,
};

#[component]
pub fn BrowserVolume(mut state: Signal<ShellState>) -> Element {
    let committed_address = state.read().browser_url.clone();
    let mut address_draft = use_signal(|| committed_address.clone());
    let mut last_committed_address = use_signal(|| committed_address.clone());
    use_effect(use_reactive((&committed_address,), move |(address,)| {
        let changed = *last_committed_address.peek() != address;
        if changed {
            last_committed_address.set(address.clone());
            address_draft.set(address);
        }
    }));
    let url = address_draft.read().clone();
    let s = state.read();
    let clipboard_toast = s.clipboard_toast.clone();
    let active_mode = s.browser_mode;
    let prod = s.browser_product.clone();
    let history = s.browser_history.clone();
    let social_groups = s.browser_social.semantic_groups.clone();
    let bookmarks = s.browser_social.bookmarks.clone();
    let annotations = s.browser_social.annotations.clone();
    let bm_title = s.bookmark_title_input.clone();
    let bm_assertion = s.bookmark_assertion_input.clone();
    let bm_entity = s.bookmark_entity_input.clone();
    let bm_group = s.bookmark_group_target.clone();
    let ann_fragment = s.annotation_fragment_input.clone();
    let ann_kind = s.annotation_kind_input.clone();
    let ann_body = s.annotation_body_input.clone();
    let ann_group = s.annotation_group_target.clone();
    let grp_id = s.new_group_id_input.clone();
    let grp_label = s.new_group_label_input.clone();
    let grp_ontology = s.new_group_ontology_input.clone();
    let bridge_receipts: Vec<String> = s
        .browser_bridge_receipts
        .iter()
        .map(|r| {
            let verdict = if r.permitted { "PERMITTED" } else { "DENIED" };
            format!(
                "[{}] [{}] origin={} · channel={} -> {} (NQuin48 parity {:#018x})",
                verdict,
                r.mode.code(),
                r.caller_origin,
                r.channel_id,
                r.reply_payload,
                r.sealed_nquin_parity
            )
        })
        .collect();
    drop(s);

    let mode_title = active_mode.title();
    let mode_engine = active_mode.engine_and_ring();
    let mode_desc = active_mode.description();
    let prod_ver = prod.installed_version.clone();
    let prod_latest = prod.latest_available_version.clone();
    let prod_did = prod.app_did.clone();
    let prod_git = prod.git_did.clone();
    let prod_dir = prod.package_dir.clone();
    let prod_worker = prod.worker_binary_slot.clone();
    let prod_channel = prod.independent_update_channel.clone();
    let prod_leases = prod.required_leases.join(", ");
    let prod_receipt = prod.update_receipt_detail.clone();

    let channel_cards: Vec<(HybridWebCapabilityChannel, String, String, String, String)> =
        HybridWebCapabilityChannel::ALL
            .iter()
            .copied()
            .map(|ch| {
                let spec = ch.spec();
                (
                    ch,
                    spec.title.to_string(),
                    format!(
                        "Channel: {} · Required Lease: {}",
                        spec.channel_id, spec.required_lease
                    ),
                    format!("Plane Guard: {}", spec.plane_guard),
                    spec.description.to_string(),
                )
            })
            .collect();

    let bookmarks_open = state.read().bookmarks_drawer_open;
    let engine_open = state.read().browser_engine_drawer_open;
    let product_mode = match active_mode {
        BrowserExecutionMode::TraditionalWeb => "Web",
        BrowserExecutionMode::QualiaNative => "Spatial",
        BrowserExecutionMode::HybridAugmented => "Hybrid",
    };

    rsx! {
        div {
            class: "browser-workspace",
            nav {
                class: "browser-toolbar",
                aria_label: "Browser",
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    title: "Back",
                    onclick: move |_| {
                        let mut s = state.write();
                        if let Some(previous) = s.browser_history.get(1).cloned() {
                            if let Some(target) = url_from_history_line(&previous) {
                                let resolved = resolve_mode_for_uri(&target, s.browser_mode);
                                s.browser_url = target.clone();
                                s.browser_mode = resolved;
                                address_draft.set(target);
                            }
                        }
                    },
                    "← Back"
                }
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    title: "Forward",
                    onclick: move |_| {
                        let s = state.write();
                        if let Some(next) = s.browser_history.first().cloned() {
                            if let Some(target) = url_from_history_line(&next) {
                                address_draft.set(target);
                            }
                        }
                    },
                    "→ Forward"
                }
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    title: "Reload",
                    onclick: move |_| {
                        let current = state.read().browser_url.clone();
                        address_draft.set(current.clone());
                        state.write().browser_url = current;
                    },
                    "↻ Reload"
                }
                form {
                    class: "browser-address",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        let target = address_draft.read().trim().to_owned();
                        if target.is_empty() {
                            return;
                        }
                        let mut s = state.write();
                        let resolved = resolve_mode_for_uri(&target, s.browser_mode);
                        s.browser_url = target.clone();
                        s.browser_mode = resolved;
                        s.browser_history.insert(
                            0,
                            format!(
                                "[{}] {} — {}",
                                resolved.title(),
                                target,
                                resolved.engine_and_ring()
                            ),
                        );
                        address_draft.set(target);
                    },
                    input {
                        class: "input-field",
                        value: "{url}",
                        "aria-label": "Address or search",
                        placeholder: "Search or enter an address…",
                        oninput: move |evt| address_draft.set(evt.value()),
                        onchange: move |evt| address_draft.set(evt.value()),
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
                    button { r#type: "submit", class: "sanctum-issue", "Go" }
                }
                button {
                    r#type: "button",
                    class: if bookmarks_open { "ribbon-btn active" } else { "ribbon-btn" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.bookmarks_drawer_open = !s.bookmarks_drawer_open;
                    },
                    "Bookmarks"
                }
                button {
                    r#type: "button",
                    class: if engine_open { "ribbon-btn active" } else { "ribbon-btn" },
                    onclick: move |_| {
                        let mut s = state.write();
                        s.browser_engine_drawer_open = !s.browser_engine_drawer_open;
                    },
                    "Engine"
                }
                div {
                    class: "browser-mode-switch",
                    role: "group",
                    aria_label: "Mode",
                    for (mode, label) in [
                        (BrowserExecutionMode::TraditionalWeb, "Web"),
                        (BrowserExecutionMode::QualiaNative, "Spatial"),
                        (BrowserExecutionMode::HybridAugmented, "Hybrid"),
                    ] {
                        {
                            let selected = active_mode == mode;
                            rsx! {
                                button {
                                    key: "{label}",
                                    r#type: "button",
                                    class: if selected { "ribbon-btn active" } else { "ribbon-btn" },
                                    onclick: move |_| state.write().browser_mode = mode,
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
            if !clipboard_toast.is_empty() {
                p { class: "muted", role: "status", "{clipboard_toast}" }
            }
            if bookmarks_open {
                aside {
                    class: "browser-popover",
                    h4 { "Bookmarks" }
                    ul {
                        for bm in &bookmarks {
                            {
                                let bm_uri = bm.uri.clone();
                                let bm_title_val = bm.title.clone();
                                rsx! {
                                    li {
                                        key: "{bm.bookmark_id}",
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                s.browser_url = bm_uri.clone();
                                                address_draft.set(bm_uri.clone());
                                                s.bookmarks_drawer_open = false;
                                            },
                                            "{bm_title_val}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if engine_open {
                details {
                    class: "tech-details-drawer",
                    open: true,
                    summary { "Engine status" }
                    p { "Mode: {product_mode} · {mode_title}" }
                    p { class: "muted", "{mode_desc}" }
                    p { class: "path", "Worker: {prod_worker}" }
                    p { class: "muted", "{prod_receipt}" }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                if let Ok(updated) = upgrade_browser_product_independent(&sess.host, "0.1.1") {
                                    let mut s = state.write();
                                    s.browser_product = updated;
                                    if let Ok(qaps) = sess.host.list_qapps() {
                                        s.installed_qaps = qaps;
                                    }
                                }
                            }
                        },
                        "Check for engine update"
                    }
                }
            }
            div {
                class: "browser-viewport",
                BrowserChoraSurface { state }
            }

            details {
                class: "tech-details-drawer",
                summary { "Bookmarks, groups, and page notes" }
                p { class: "muted", "Active mode: {mode_title} · {mode_engine}" }
                dl {
                    class: "kv",
                    dt { "Browser version" }
                    dd { "v{prod_ver} (channel v{prod_latest})" }
                    dt { "Product id" }
                    dd { class: "path", "{prod_did}" }
                    dt { "Source" }
                    dd { class: "path", "{prod_git}" }
                    dt { "Install folder" }
                    dd { class: "path", "{prod_dir}" }
                    dt { "Update channel" }
                    dd { class: "path", "{prod_channel}" }
                    dt { "Leases" }
                    dd { "{prod_leases}" }
                }

            // 4. Mode 3 Hybrid Capability Bridge (QualiaDB Capabilities Embedded in Traditional Web Pages)
            section {
                class: "id-list",
                h4 { "Mode 3 Hybrid Bridge · QualiaDB Capabilities in Traditional Web Pages (Control::PageRequest -> Host Sentinel Gate -> PageReply)" }
                p {
                    class: "muted",
                    "Run bounded local checks with fixed samples. Mode 1 refuses bridge requests. A live page still needs a separate host grant."
                }
                div {
                    class: "plane-grid",
                    for (ch, ch_title, ch_lease, ch_plane, ch_desc) in channel_cards {
                        {
                            rsx! {
                                div {
                                    key: "{ch_title}",
                                    class: "plane-card",
                                    h5 { "{ch_title}" }
                                    p { "{ch_lease}" }
                                    p { "{ch_plane}" }
                                    p { "{ch_desc}" }
                                    button {
                                        r#type: "button",
                                        class: "sanctum-issue",
                                        onclick: move |_| {
                                            if let Some(sess) = session() {
                                                let (mode, current_address) = {
                                                    let shell = state.peek();
                                                    (shell.browser_mode, shell.browser_url.clone())
                                                };
                                                let origin = webizen_ua_os::hybrid_https_origin(&current_address)
                                                    .unwrap_or_else(|| "https://preview.invalid".into());
                                                let rec = evaluate_hybrid_page_request_in_storage(
                                                    mode,
                                                    &sess.identifiers,
                                                    &sess.host.storage,
                                                    &origin,
                                                    ch,
                                                    ch.local_test_payload(),
                                                );
                                                state.write().browser_bridge_receipts.insert(0, rec);
                                            }
                                        },
                                        "Run local capability preview"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !bridge_receipts.is_empty() {
                section {
                    class: "id-list",
                    h4 { "Capability Routing Preview (NQuin48 Receipts)" }
                    ul {
                        for r_line in bridge_receipts {
                            li { "{r_line}" }
                        }
                    }
                }
            }

            // 5. Semantically Defined Social Groups (spc:SemanticGroup over SocialWebNet WireGuard + QDNF QFrame + X25519 Envelopes)
            section {
                class: "id-list",
                h4 { "Semantically Defined Social Groups ({social_groups.len()}) · spc:SemanticGroup over SocialWebNet WireGuard + QDNF ports::QDNF + X25519 Envelopes" }
                p {
                    class: "muted",
                    "Social groups are ontologically defined by spc:relationType (qualia-client-core/src/social_peers.rs) and routed over SocialWebNet userspace-WireGuard with QDNF QFrame multicast (qualia-core-db/src/p2p/social_qdnf.rs) and X25519 group envelope encryption."
                }
                div {
                    class: "plane-grid",
                    for g in &social_groups {
                        {
                            let g_id = g.group_id.clone();
                            let g_id_for_bm = g_id.clone();
                            let g_id_for_ann = g_id.clone();
                            let g_label = g.title.clone();
                            let g_onto = g.ontology_relation_iri.clone();
                            let g_topics = g.domain_topics.join(", ");
                            let g_members = g.member_pairwise_dids.join(", ");
                            let g_transport = g.transport_profile.clone();
                            let g_desc = g.description.clone();
                            rsx! {
                                div {
                                    key: "{g_id}",
                                    class: "plane-card",
                                    h5 { "{g_label} ({g_id})" }
                                    p { "Ontology Relation: {g_onto}" }
                                    p { "Domain Topics: [{g_topics}]" }
                                    p { "{g_desc}" }
                                    p { "Pairwise Members: {g_members}" }
                                    p { "Transport: {g_transport}" }
                                    div {
                                        class: "orb-sectors",
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                s.bookmark_group_target = g_id_for_bm.clone();
                                                s.annotation_group_target = g_id_for_ann.clone();
                                            },
                                            "Target Group for Bookmarks & Annotations"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{grp_id}",
                        placeholder: "Group ID (e.g. group:bioregional-energy-guild)…",
                        oninput: move |evt| state.write().new_group_id_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{grp_label}",
                        placeholder: "Semantic group label…",
                        oninput: move |evt| state.write().new_group_label_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{grp_ontology}",
                        placeholder: "Ontology relation IRI (urn:qualia:spc:relation:...)…",
                        oninput: move |evt| state.write().new_group_ontology_input = evt.value(),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let id = s.new_group_id_input.trim().to_string();
                                let label = s.new_group_label_input.trim().to_string();
                                let onto = s.new_group_ontology_input.trim().to_string();
                                if !id.is_empty() && !label.is_empty() {
                                    let created = s.browser_social.create_semantic_group(
                                        &sess.host.storage,
                                        &id,
                                        &label,
                                        if onto.is_empty() {
                                            "https://ns.webcivics.net/spc#cooperativeCommonsCircle"
                                        } else {
                                            &onto
                                        },
                                        vec!["energy".into(), "civics".into()],
                                        vec![
                                            "did:qi:peer:guild-steward-a".into(),
                                            "did:qi:peer:guild-steward-b".into(),
                                        ],
                                        "Semantically defined cooperative group over SocialWebNet WireGuard + QDNF QFrame",
                                    );
                                    s.bookmark_group_target = created.group_id.clone();
                                    s.annotation_group_target = created.group_id;
                                }
                            }
                        },
                        "Mint Semantically Defined Social Group"
                    }
                }
            }

            // 6. Semantic Bookmarks (QLink / qualia:WebBookmark -> Semantic File-System)
            section {
                class: "id-list",
                h4 { "Semantic Bookmarks ({bookmarks.len()} QLinks · qualia:WebBookmark -> Automatically Indexed in Semantic File-System)" }
                p {
                    class: "muted",
                    "Every Semantic Bookmark (QLink) records the active URI, execution mode, your contextual assertion, extracted entities/topics, social group disclosure scope, and a 48-byte NQuin48 seal, while automatically indexing the bookmark into the Semantic File-System."
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{bm_title}",
                        placeholder: "Bookmark title…",
                        oninput: move |evt| state.write().bookmark_title_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{bm_assertion}",
                        placeholder: "Contextual claim / why this resource matters…",
                        oninput: move |evt| state.write().bookmark_assertion_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{bm_entity}",
                        placeholder: "Depicts entity tag (e.g. solar-microgrid-sdf)…",
                        oninput: move |evt| state.write().bookmark_entity_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{bm_group}",
                        placeholder: "Social group ID (or leave blank for private)…",
                        oninput: move |evt| state.write().bookmark_group_target = evt.value(),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let target_uri = s.browser_url.trim().to_string();
                                let title = s.bookmark_title_input.trim().to_string();
                                let assertion = s.bookmark_assertion_input.trim().to_string();
                                let entity = s.bookmark_entity_input.trim().to_string();
                                let group_id = s.bookmark_group_target.trim().to_string();
                                let mode = s.browser_mode;
                                if !target_uri.is_empty() && !title.is_empty() {
                                    let disclosure = if group_id.is_empty() {
                                        SocialDisclosureScope {
                                            lane: "private".into(),
                                            target_id: String::new(),
                                        }
                                    } else {
                                        SocialDisclosureScope {
                                            lane: "group".into(),
                                            target_id: group_id,
                                        }
                                    };
                                    let mut fs_clone = s.semantic_fs.clone();
                                    let _bm = s.browser_social.save_semantic_bookmark(
                                        &sess.host.storage,
                                        &mut fs_clone,
                                        &target_uri,
                                        &title,
                                        mode,
                                        &assertion,
                                        &entity,
                                        "https://webizen.org/q42#WebBookmark",
                                        disclosure,
                                    );
                                    s.semantic_fs = fs_clone;
                                }
                            }
                        },
                        "Save Current URL as Semantic Bookmark (QLink -> Semantic Library)"
                    }
                }
                ul {
                    for bm in &bookmarks {
                        {
                            let bm_id = bm.bookmark_id.clone();
                            let bm_mode = bm.mode.code();
                            let bm_title_val = bm.title.clone();
                            let bm_uri = bm.uri.clone();
                            let bm_scope = bm.disclosure.label();
                            let bm_assertion_val = bm.context_assertion.clone();
                            let topics = bm.topics.join(", ");
                            let entities = bm.depicted_entities.join(", ");
                            let bm_onto = bm.ontology_iri.clone();
                            let bm_parity = format!("{:#018x}", bm.sealed_nquin_parity);
                            rsx! {
                                li {
                                    key: "{bm_id}",
                                    strong { "[{bm_mode}] {bm_title_val} " }
                                    span { "({bm_uri}) · Scope: [{bm_scope}] · Assertion: \"{bm_assertion_val}\" · Topics: [{topics}] · Entities: [{entities}] · Ontology: {bm_onto} · NQuin48 {bm_parity}" }
                                }
                            }
                        }
                    }
                }
            }

            // 7. Dialectical Web Annotations & Social-Web Interactive Engagement
            section {
                class: "id-list",
                h4 { "Dialectical Web Annotations & Social-Web Comments ({annotations.len()} Anchored Claims over SocialQdnfLink)" }
                p {
                    class: "muted",
                    "Engage interactively on any traditional web page or native 10D v2 HOLO URI. Annotations anchor to url#fragment, carry an illocutionary speech-act kind, optional CML inclusion, and distribute to your selected Semantically Defined Social Group over QDNF ports::QDNF in X25519 envelopes."
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{ann_fragment}",
                        placeholder: "Anchor fragment / selector (e.g. section-energy-storage)…",
                        oninput: move |evt| state.write().annotation_fragment_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{ann_kind}",
                        placeholder: "Speech-act kind (ContextualClaim / Endorsement / Comment / Dispute)…",
                        oninput: move |evt| state.write().annotation_kind_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{ann_group}",
                        placeholder: "Target Semantic Group ID (e.g. group:catchment-stewards)…",
                        oninput: move |evt| state.write().annotation_group_target = evt.value(),
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{ann_body}",
                        placeholder: "Write annotation claim or social-web comment for the group…",
                        oninput: move |evt| state.write().annotation_body_input = evt.value(),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let target_uri = s.browser_url.trim().to_string();
                                let fragment_sel = s.annotation_fragment_input.trim().to_string();
                                let kind = s.annotation_kind_input.trim().to_string();
                                let body = s.annotation_body_input.trim().to_string();
                                let group_id = s.annotation_group_target.trim().to_string();
                                let author_did = if s.fabric_did.is_empty() {
                                    "did:q42:qrc:local-steward".to_string()
                                } else {
                                    s.fabric_did.clone()
                                };
                                if !target_uri.is_empty() && !body.is_empty() {
                                    let disclosure = if group_id.is_empty() {
                                        SocialDisclosureScope {
                                            lane: "private".into(),
                                            target_id: String::new(),
                                        }
                                    } else {
                                        SocialDisclosureScope {
                                            lane: "group".into(),
                                            target_id: group_id,
                                        }
                                    };
                                    let _ann = s.browser_social.add_web_annotation(
                                        &sess.host.storage,
                                        &target_uri,
                                        if fragment_sel.is_empty() { "document-root" } else { &fragment_sel },
                                        &author_did,
                                        if kind.is_empty() { "ContextualClaim" } else { &kind },
                                        &body,
                                        "https://ns.webcivics.net/commons#ContextualAssertion",
                                        true,
                                        disclosure,
                                    );
                                }
                            }
                        },
                        "Post Dialectical Annotation & Broadcast to Semantic Group (QDNF QFrame)"
                    }
                }
                ul {
                    for ann in &annotations {
                        {
                            let ann_id = ann.annotation_id.clone();
                            let ann_kind_val = ann.illocutionary_kind.clone();
                            let ann_target = ann.target_uri.clone();
                            let ann_frag = ann.fragment_selector.clone();
                            let ann_body_val = ann.body.clone();
                            let (assertion, evidence) = annotation_assertion(&ann.illocutionary_kind);
                            let badge = render_contextual_assertion_badge_with_evidence(&assertion, Some(evidence));
                            let ann_scope = ann.disclosure.label();
                            let ann_onto = ann.ontology_authority_iri.clone();
                            let cml_str = if ann.include_in_cml { "CML+NQuin48" } else { "NQuin48" };
                            let ann_qdnf = ann.qdnf_frame_digest.clone();
                            let ann_parity = format!("{:#018x}", ann.sealed_nquin_parity);
                            rsx! {
                                li {
                                    key: "{ann_id}",
                                    strong { "[{ann_kind_val}] {ann_target}#{ann_frag} " }
                                    {badge}
                                    span { "— \"{ann_body_val}\" · Scope: [{ann_scope}] · Ontology: {ann_onto} · {cml_str} · QDNF: {ann_qdnf} · NQuin48 {ann_parity}" }
                                }
                            }
                        }
                    }
                }
            }

            section {
                class: "id-list",
                h4 { "Recent pages" }
                ul {
                    for h in history {
                        li { "{h}" }
                    }
                }
            }
            }
        }
    }
}

fn url_from_history_line(line: &str) -> Option<String> {
    let after_title = line.split_once("] ")?.1;
    let url = after_title.split_once(" — ").map(|(url, _)| url).unwrap_or(after_title);
    let url = url.trim();
    (!url.is_empty()).then(|| url.to_string())
}

fn annotation_assertion(kind: &str) -> (ContextualAssertionView, AssertionEvidenceLevel) {
    // The stored speech act is declared by the author. Group routing and record
    // parity do not establish an independent witness or a resolution decision.
    let (kind, challenge, summary) = match kind.trim().to_ascii_lowercase().as_str() {
        "observation" | "comment" => (
            AssertionKind::Observation,
            ChallengeState::Unchallenged,
            "Browser recorded this annotation. Its contents are author-supplied.",
        ),
        "inference" | "endorsement" => (
            AssertionKind::Inference,
            ChallengeState::Unchallenged,
            "Author-supplied interpretation with no independent corroboration recorded.",
        ),
        "attestation" => (
            AssertionKind::Allegation,
            ChallengeState::Restricted,
            "Claimed witness attestation requires signature and witness verification.",
        ),
        "adjudicatedfinding" | "adjudicated finding" | "adjudicated-finding" => (
            AssertionKind::Allegation,
            ChallengeState::Restricted,
            "Claimed finding requires a verified resolution record.",
        ),
        _ => (
            AssertionKind::Allegation,
            ChallengeState::Restricted,
            "Author-supplied claim with no independent corroboration recorded.",
        ),
    };
    (
        ContextualAssertionView {
            kind,
            challenge,
            independent_source_count: 0,
            summary,
        },
        AssertionEvidenceLevel::Contextual,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_attestation_and_finding_do_not_gain_verified_epistemic_status() {
        for kind in [
            "Attestation",
            "AdjudicatedFinding",
            "Dispute / Counter-Evidence",
        ] {
            let (view, evidence) = annotation_assertion(kind);
            assert_eq!(view.kind, AssertionKind::Allegation);
            assert_eq!(view.challenge, ChallengeState::Restricted);
            assert_eq!(view.independent_source_count, 0);
            assert_eq!(evidence, AssertionEvidenceLevel::Contextual);
        }
        assert_eq!(
            annotation_assertion("Observation").0.kind,
            AssertionKind::Observation
        );
        assert_eq!(
            annotation_assertion("Inference").0.kind,
            AssertionKind::Inference
        );
    }
}
