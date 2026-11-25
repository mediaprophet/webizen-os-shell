// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::wellfair::respond_to_alert;
use crate::{
    components::auxiliary_windows::{jump_to_stage, AuxWindowBridge, CareResponseAuthority},
    state::{refresh_refract_host_records, session, ShellState, VolumeStage, WebizenRefractModel},
};
use aether::prelude::*;
use aether::refract_bridge::RefractStore;
use webizen_ua_os::{
    generate_dual_sku_delivery_artifacts, generate_tri_target_debian_suite, run_daemon_tick,
    run_three_node_mesh_simulation, run_updater, validate_human_centric_host_nomenclature,
    ActorJuralPersonalityKind, DebianTargetProfile, HostDaemonConfig, HostNodeLocusKind,
    HostStewardshipArchetype, HostedProjectOwnershipStage, IncorporatedEntityPricingTier,
    UpdateAction,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdminSubTab {
    FleetAndDebSuite,
    StewardsAndQuorum,
    ProjectsAndJuralGate,
}

#[component]
pub fn AdminVolume(state: Signal<ShellState>) -> Element {
    let mut active_tab = use_signal(|| AdminSubTab::FleetAndDebSuite);
    let tab = active_tab();
    rsx! {
        div {
            class: "settings-volume",
            div {
                class: "volume-subtabs",
                role: "tablist",
                aria_label: "Admin & Fleet Stewardship sections",
                button {
                    r#type: "button",
                    class: if tab == AdminSubTab::FleetAndDebSuite { "ribbon-btn active" } else { "ribbon-btn" },
                    onclick: move |_| active_tab.set(AdminSubTab::FleetAndDebSuite),
                    "1. Fleet Nodes, Daemon, 3-Node Mesh & Debian 13 .deb Suite"
                }
                button {
                    r#type: "button",
                    class: if tab == AdminSubTab::StewardsAndQuorum { "ribbon-btn active" } else { "ribbon-btn" },
                    onclick: move |_| active_tab.set(AdminSubTab::StewardsAndQuorum),
                    "2. Natural-Person Stewards & M-of-N Update Quorum"
                }
                button {
                    r#type: "button",
                    class: if tab == AdminSubTab::ProjectsAndJuralGate { "ribbon-btn active" } else { "ribbon-btn" },
                    onclick: move |_| active_tab.set(AdminSubTab::ProjectsAndJuralGate),
                    "3. Hosted Workloads, Q-CDN Swarm & Jural Commercial Gate"
                }
            }
            HostStewardshipAndFleetPanel { state, tab }
        }
    }
}

#[component]
fn HostStewardshipAndFleetPanel(mut state: Signal<ShellState>, tab: AdminSubTab) -> Element {
    let mut fleet_notice = use_signal(String::new);
    let store = try_consume_context::<RefractStore<WebizenRefractModel>>();
    let store_fleet = store.clone();
    let store_projects = store;
    let s = state.read();
    let daemon = s.daemon_label.clone();
    let gpu = s.gpu_label.clone();
    let qap_count = s.installed_qaps.len();
    let updater = s.updater_status.clone();
    let delivery = s.delivery_status.clone();
    let stewardship = s.host_stewardship.clone();
    let prop_deb = s.host_proposal_deb_input.clone();
    let prop_ver = s.host_proposal_version_input.clone();
    let vote_steward = s.host_vote_steward_input.clone();
    let vote_comment = s.host_vote_comment_input.clone();
    let new_steward_did = s.host_new_steward_did_input.clone();
    let new_steward_hint = s.host_new_steward_hint_input.clone();
    let new_steward_apparatus = s.host_new_steward_apparatus_input.clone();
    let toast = s.host_stewardship_toast.clone();
    let input_guard = s.beforeinput_guard_notice.clone();
    drop(s);

    let host_domain = stewardship.primary_host_domain.clone();
    let host_endpoint = stewardship.host_endpoint.clone();
    let active_archetype = stewardship.archetype;
    let archetype_title = active_archetype.title();
    let archetype_rule = active_archetype.update_authority_rule();
    let threshold_m = stewardship.threshold_m;
    let steward_count = stewardship.stewards.len();
    let bft_f = stewardship.bft_fault_tolerance();
    let shamir_policy = stewardship.shamir_q42j_backup_threshold.clone();
    let stewards = stewardship.stewards.clone();
    let handles = stewardship.attached_handles.clone();
    let queue = stewardship.update_queue.clone();
    let fleet_nodes = stewardship.host_fleet_nodes.clone();
    let hosted_projects = stewardship.hosted_projects.clone();
    let incorporated_entities = stewardship.incorporated_entities.clone();

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Admin · Natural-Person & Cooperative Quorum Host Stewardship (webizen-host) + Tri-Target Debian 13 (Trixie) .deb Suite" }
                p {
                    "Updating or mutating a webizen-host server ({host_domain} · {host_endpoint}) never happens via an unauthenticated vendor push or unilateral bot auto-update. It relies upon the accountable Natural Person (or M-of-N group of Natural Persons) who steward it through their Desktop (webizen-desktop-appliance.deb) and Mobile AVF Pocket Sanctuary (webizen-mobile-avf.deb) apparatus."
                }
            }
            dl {
                class: "kv",
                dt { "Stewarded Host Domain" }
                dd { "{host_domain} — {host_endpoint}" }
                dt { "Active Stewardship Archetype" }
                dd { "{archetype_title} · Threshold: {threshold_m}-of-{steward_count} Responsible Natural Persons (BFT f={bft_f})" }
                dt { "Update Authority Rule" }
                dd { "{archetype_rule}" }
                dt { "Sanctuary vs Host Split" }
                dd { "Sanctuary Keep stays strictly on local Desktop/Mobile AVF hardware (:8080); off-site backups on webizen-host (:4242) are {shamir_policy}" }
                dt { "QualiaDB Sidecar (:4242)" }
                dd { "{daemon}" }
                dt { "Webizen Sentinel & GPU" }
                dd { "42 MiB Zero-Heap SlgArena · {gpu} · {qap_count} active Qaps" }
            }

            if !input_guard.is_empty() {
                p { class: "muted", role: "status", "{input_guard}" }
            }
            if !toast.is_empty() {
                p { class: "path", "{toast}" }
            }

            if tab == AdminSubTab::FleetAndDebSuite {
                ApplianceProfilesPanel { state }
                HostDaemonPanel { state }
                MeshSimulationPanel { state }

                // 1. Tri-Target Debian 13 (Trixie) .deb Packaging Matrix (Desktop amd64 + Mobile AVF arm64 + Host amd64)
                section {
                class: "id-list",
                h4 { "Tri-Target Debian 13 (Trixie) .deb Packaging Architecture (Single Rust Workspace -> Desktop, Mobile AVF pKVM EL2 & Host)" }
                div {
                    class: "plane-grid",
                    for profile in DebianTargetProfile::ALL {
                        {
                            let p_title = profile.title();
                            let p_pkg = profile.deb_package_name();
                            let p_arch = profile.architecture();
                            let p_role = profile.role_in_human_fabric();
                            rsx! {
                                div {
                                    key: "{p_pkg}",
                                    class: "plane-card",
                                    h5 { "{p_title}" }
                                    p { "Debian Package: {p_pkg}_0.1.0_{p_arch}.deb (Debian 13 Trixie)" }
                                    p { "{p_role}" }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "orb-sectors",
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| build_appliance_packages(state),
                        "Generate Tri-Target Debian 13 (Trixie) .deb Suite (Desktop amd64 + Mobile AVF arm64 + webizen-hostd)"
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                if let Ok(rec) = generate_dual_sku_delivery_artifacts(&sess.host, &sess.gpu) {
                                    state.write().delivery_status = format!(
                                        "SKU A Guest Manifest ({}) + SKU B Debian 13 Trixie Overlay ({}) + Three-Ring LICENSE-BOUNDARY ({})",
                                        rec.guest_manifest_path.display(),
                                        rec.debian_overlay_dir.display(),
                                        rec.license_boundary_path.display()
                                    );
                                }
                            }
                        },
                        "Emit Dual-SKU Delivery Artifacts (SKU A Guest + SKU B Debian Trixie Overlay)"
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let rec = run_updater(&sess.host, sess.sku, UpdateAction::Check, None);
                                state.write().updater_status = rec.detail;
                            }
                        },
                        "Run Ed25519 Self-Updater Check (C7)"
                    }
                }
                if !updater.is_empty() {
                    p { class: "path", "Updater: {updater}" }
                }
                if !delivery.is_empty() {
                    p { class: "path", "Delivery: {delivery}" }
                }
            }
            }

            if tab == AdminSubTab::StewardsAndQuorum {
            // 2. Four Human-Centric Stewardship Archetypes Switcher
            section {
                class: "id-list",
                h4 { "Select Host Stewardship Governance Archetype (4 Human-Centric Archetypes)" }
                div {
                    class: "plane-grid",
                    for arch in HostStewardshipArchetype::ALL {
                        {
                            let is_active = active_archetype == arch;
                            let badge = if is_active { "ACTIVE ARCHETYPE" } else { "SELECT" };
                            let a_title = arch.title();
                            let a_rule = arch.update_authority_rule();
                            rsx! {
                                div {
                                    key: "{a_title}",
                                    class: "plane-card",
                                    h5 { "[{badge}] {a_title}" }
                                    p { "{a_rule}" }
                                    button {
                                        r#type: "button",
                                        class: "sanctum-issue",
                                        onclick: move |_| {
                                            let mut s = state.write();
                                            s.host_stewardship.archetype = arch;
                                            if let Some(sess) = session() {
                                                let _ = s.host_stewardship.save(&sess.host.storage);
                                            }
                                        },
                                        "Activate {a_title}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 3. Responsible Natural-Person Stewards (Plane 1: NaturalAgent / living-SHACL, Disjoint from owl:Thing)
            section {
                class: "id-list",
                h4 { "Responsible Natural-Person Stewards ({steward_count} NaturalAgents · living-SHACL, Disjoint from owl:Thing)" }
                p {
                    class: "muted",
                    "Every webizen-host steward is a living natural person equipped with a Desktop (webizen-desktop-appliance.deb) or Mobile AVF pKVM EL2 Pocket Sanctuary (webizen-mobile-avf.deb) apparatus. Invariant: 'Natural persons are not identity assets; instruments are not peers.' AI bots and owl:Thing labels are rejected fail-closed."
                }
                div {
                    class: "plane-grid",
                    for st in &stewards {
                        {
                            let st_did = st.steward_did.clone();
                            let st_did_for_vote = st_did.clone();
                            let st_hint = st.display_hint.clone();
                            let st_role = st.role.clone();
                            let st_app = st.apparatus_profile.clone();
                            let st_fp = st.dual_pubkey_fingerprint.clone();
                            let cap_str = if st.has_full_legal_capacity {
                                "Full Adult Legal Capacity (Direct Quorum Signer)".to_string()
                            } else {
                                format!(
                                    "Supported Dependent (UN CRC/CRPD Guardianship Co-Sign Required from {})",
                                    st.guardian_did.as_deref().unwrap_or("AdultStewardGuardian")
                                )
                            };
                            rsx! {
                                div {
                                    key: "{st_did}",
                                    class: "plane-card",
                                    h5 { "{st_hint} [{st_role}]" }
                                    p { "Steward DID: {st_did}" }
                                    p { "Apparatus: {st_app}" }
                                    p { "Dual Key (Ed25519 + ML-DSA-65): {st_fp}" }
                                    p { "Capacity: {cap_str}" }
                                    button {
                                        r#type: "button",
                                        class: "ribbon-btn",
                                        onclick: move |_| {
                                            state.write().host_vote_steward_input = st_did_for_vote.clone();
                                        },
                                        "Select DID for Local Review"
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
                        value: "{new_steward_did}",
                        placeholder: "Natural-Person Steward DID (did:qi:steward:...)…",
                        onbeforeinput: move |evt| guard_host_beforeinput(state, evt),
                        oninput: move |evt| set_governed_input(state, &evt.value(), |s, value| s.host_new_steward_did_input = value),
                    }
                    input {
                        class: "input-field",
                        value: "{new_steward_hint}",
                        placeholder: "Natural-person display hint (never owl:Thing)…",
                        onbeforeinput: move |evt| guard_host_beforeinput(state, evt),
                        oninput: move |evt| set_governed_input(state, &evt.value(), |s, value| s.host_new_steward_hint_input = value),
                    }
                    input {
                        class: "input-field",
                        value: "{new_steward_apparatus}",
                        placeholder: "Apparatus (Desktop Debian 13 or Mobile AVF pKVM EL2)…",
                        onbeforeinput: move |evt| guard_host_beforeinput(state, evt),
                        oninput: move |evt| set_governed_input(state, &evt.value(), |s, value| s.host_new_steward_apparatus_input = value),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let did = s.host_new_steward_did_input.trim().to_string();
                                let hint = s.host_new_steward_hint_input.trim().to_string();
                                let app = s.host_new_steward_apparatus_input.trim().to_string();
                                match s.host_stewardship.register_natural_steward(
                                    &sess.host.storage,
                                    &did,
                                    &hint,
                                    "CoStewardGuardian",
                                    &app,
                                    true,
                                    None,
                                ) {
                                    Ok(created) => {
                                        s.refresh_host_profiles();
                                        s.host_vote_steward_input = created.steward_did.clone();
                                        s.host_stewardship_toast = format!(
                                            "Registered Natural-Person Co-Steward {} ({}) on {}",
                                            created.display_hint, created.steward_did, created.apparatus_profile
                                        );
                                    }
                                    Err(e) => {
                                        s.host_stewardship_toast = e;
                                    }
                                }
                            }
                        },
                        "Register Responsible Natural-Person Steward"
                    }
                }
            }

            // 4. 32-Slot SuspendedTransactionQueue for webizen-host .deb Updates & Paraconsistent 0x32 Dissent
            section {
                class: "id-list",
                h4 { "webizen-host Update Quorum Queue ({queue.len()}/32 Slots · M-of-N Review Threshold + Paraconsistent 0x32 Dissent)" }
                p {
                    class: "muted",
                    "Propose a candidate package and record local review positions. Dissent requires human review. A separate cryptographic verifier must authorize any host update."
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{prop_deb}",
                        placeholder: "Candidate Debian package (e.g. webizen-hostd_0.2.0_amd64.deb)…",
                        onbeforeinput: move |evt| guard_host_beforeinput(state, evt),
                        oninput: move |evt| set_governed_input(state, &evt.value(), |s, value| s.host_proposal_deb_input = value),
                    }
                    input {
                        class: "input-field",
                        value: "{prop_ver}",
                        placeholder: "Target version (e.g. 0.2.0)…",
                        onbeforeinput: move |evt| guard_host_beforeinput(state, evt),
                        oninput: move |evt| set_governed_input(state, &evt.value(), |s, value| s.host_proposal_version_input = value),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let pkg = s.host_proposal_deb_input.trim().to_string();
                                let ver = s.host_proposal_version_input.trim().to_string();
                                let proposer = s.host_vote_steward_input.trim().to_string();
                                match s.host_stewardship.propose_host_update(
                                    &sess.host.storage,
                                    &pkg,
                                    &ver,
                                    &proposer,
                                    false,
                                ) {
                                    Ok(prop) => {
                                        s.refresh_host_profiles();
                                        s.host_stewardship_toast = format!(
                                            "Enqueued {} (v{}) into SuspendedTransactionQueue slot {} — requires review by {} of {} stewards",
                                            prop.target_deb_package, prop.target_version, prop.slot_index, prop.required_m, prop.total_n
                                        );
                                    }
                                    Err(e) => {
                                        s.host_stewardship_toast = e;
                                    }
                                }
                            }
                        },
                        "Enqueue webizen-host .deb Update Proposal"
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{vote_steward}",
                        placeholder: "Steward DID for review (select above)…",
                        onbeforeinput: move |evt| guard_host_beforeinput(state, evt),
                        oninput: move |evt| set_governed_input(state, &evt.value(), |s, value| s.host_vote_steward_input = value),
                    }
                    input {
                        class: "input-field",
                        value: "{vote_comment}",
                        placeholder: "Steward review note or dissent reason…",
                        onbeforeinput: move |evt| guard_host_beforeinput(state, evt),
                        oninput: move |evt| set_governed_input(state, &evt.value(), |s, value| s.host_vote_comment_input = value),
                    }
                }
                ul {
                    for prop in &queue {
                        {
                            let p_id = prop.proposal_id.clone();
                            let p_id_approve = p_id.clone();
                            let p_id_dissent = p_id.clone();
                            let p_pkg = prop.target_deb_package.clone();
                            let p_ver = prop.target_version.clone();
                            let p_host = prop.target_host_domain.clone();
                            let p_req_m = prop.required_m;
                            let p_tot_n = prop.total_n;
                            let approvals = prop.votes.iter().filter(|v| v.approve).count();
                            let denials = prop.votes.iter().filter(|v| !v.approve).count();
                            let status_badge = if prop.paraconsistent_review_flagged {
                                "PARACONSISTENT 0x32 DISSENT (RequiresHumanReview — Rollout Blocked)"
                            } else if approvals >= usize::from(p_req_m) {
                                "LOCAL REVIEW THRESHOLD MET · HOST VERIFICATION REQUIRED"
                            } else {
                                "AWAITING LOCAL STEWARD REVIEW"
                            };
                            let votes_summary: Vec<String> = prop
                                .votes
                                .iter()
                                .map(|v| {
                                    let verdict = if v.approve { "APPROVED" } else { "DISSENT 0x32" };
                                    format!(
                                        "[{}] {} ({}) · {}",
                                        verdict, v.steward_did, v.apparatus_profile, v.comment
                                    )
                                })
                                .collect();
                            let p_parity = format!("{:#018x}", prop.sealed_nquin_parity);
                            rsx! {
                                li {
                                    key: "{p_id}",
                                    div {
                                        strong { "[{status_badge}] {p_pkg} (v{p_ver} -> {p_host}) " }
                                        span { "· Approvals: {approvals}/{p_req_m} (of {p_tot_n} stewards) · Denials: {denials} · CBOR-LD 0xD90501 · NQuin48 {p_parity}" }
                                    }
                                    div {
                                        class: "orb-sectors",
                                        button {
                                            r#type: "button",
                                            class: "sanctum-issue",
                                            onclick: move |_| {
                                                if let Some(sess) = session() {
                                                    let mut s = state.write();
                                                    let signer = s.host_vote_steward_input.trim().to_string();
                                                    let comment = s.host_vote_comment_input.trim().to_string();
                                                    match s.host_stewardship.sign_or_dissent_host_update(
                                                        &sess.host.storage,
                                                        &p_id_approve,
                                                        &signer,
                                                        None,
                                                        true,
                                                        &comment,
                                                    ) {
                                                        Ok(updated) => {
                                                            s.refresh_host_profiles();
                                                            s.host_stewardship_toast = format!(
                                                                "Recorded local approval from {} on {} for quorum review.",
                                                                signer, updated.proposal_id
                                                            );
                                                        }
                                                        Err(e) => {
                                                            s.host_stewardship_toast = e;
                                                        }
                                                    }
                                                }
                                            },
                                            "Record Local Approval"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            onclick: move |_| {
                                                if let Some(sess) = session() {
                                                    let mut s = state.write();
                                                    let signer = s.host_vote_steward_input.trim().to_string();
                                                    let comment = s.host_vote_comment_input.trim().to_string();
                                                    match s.host_stewardship.sign_or_dissent_host_update(
                                                        &sess.host.storage,
                                                        &p_id_dissent,
                                                        &signer,
                                                        None,
                                                        false,
                                                        &comment,
                                                    ) {
                                                        Ok(updated) => {
                                                            s.refresh_host_profiles();
                                                            s.host_stewardship_toast = format!(
                                                                "Recorded Paraconsistent 0x32 Dissent (RequiresHumanReview) from {} on {}",
                                                                signer, updated.proposal_id
                                                            );
                                                        }
                                                        Err(e) => {
                                                            s.host_stewardship_toast = e;
                                                        }
                                                    }
                                                }
                                            },
                                            "Record Dissent for Human Review"
                                        }
                                    }
                                    if !votes_summary.is_empty() {
                                        ul {
                                            for v_line in votes_summary {
                                                li { "{v_line}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 5. Who-Plane != Handle-Plane Collapse Detector (Handle revoke != who-erase)
            section {
                class: "id-list",
                h4 { "Who-Plane != Handle-Plane Collapse Detector (Domain Expiry / Seizure Never Erases Natural-Person Stewards)" }
                ul {
                    for (dom, st) in &handles {
                        {
                            let dom_key = dom.clone();
                            let dom_trigger = dom.clone();
                            let st_val = st.clone();
                            rsx! {
                                li {
                                    key: "{dom_key}",
                                    strong { "{dom_key} " }
                                    span { "— {st_val} " }
                                    button {
                                        r#type: "button",
                                        class: "ribbon-btn",
                                        onclick: move |_| {
                                            if let Some(sess) = session() {
                                                let mut s = state.write();
                                                if let Ok(msg) = s.host_stewardship.trigger_domain_collapse_detector(
                                                    &sess.host.storage,
                                                    &dom_trigger,
                                                ) {
                                                    s.host_stewardship_toast = msg;
                                                }
                                            }
                                        },
                                        "Test Handle Collapse Detector"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 6. Multi-Locus Personal & Roaming Host Fleet (Home Server, 12V Camper/Walkabout Rig, Online Cloud VPS, or Standalone Offline Mesh)
            p { class: "muted", role: "status", "{fleet_notice}" }
            details {
                class: "id-list",
                onbeforetoggle: move |evt| {
                    if let Some(toggle) = evt.data().downcast::<aether_native::NativeToggleData>() {
                        if toggle.0.old_state != toggle.0.new_state {
                            fleet_notice.set(if toggle.0.new_state { "Opening fleet inventory".into() } else { "Closing fleet inventory".into() });
                        }
                    }
                },
                ontoggle: move |evt| {
                    if let Some(toggle) = evt.data().downcast::<aether_native::NativeToggleData>() {
                        fleet_notice.set(if toggle.0.new_state { "Fleet inventory open".into() } else { "Fleet inventory closed".into() });
                    }
                },
                summary { "Multi-Locus Personal & Roaming Host Fleet ({fleet_nodes.len()} Nodes · Home Server, 12V Camper Rig, Online Cloud VPS, or Local-Only Mesh)" }
                p {
                    class: "muted",
                    "Users may run a Home Server, a 12V Camper/Walkabout Rig on the road, an Online Cloud VPS, or operate 100% standalone on their Home/Camper node without any online VPS. Sanctuary Keep stays on local Home/Camper/Desktop/Mobile AVF hardware while optional online VPS nodes serve as encrypted Shamir .q42j mirrors and Q-CDN ingress."
                }
                div {
                    class: "plane-grid",
                    for node in &fleet_nodes {
                        {
                            let n_id = node.node_id.clone();
                            let n_label = node.label.clone();
                            let n_locus = node.locus.title();
                            let n_routing = node.locus.routing_profile();
                            let n_ops = node.runtime_operator_steward_dids.join(", ");
                            let n_ula = node.qdnf_ula_ipv6.clone();
                            let n_ep = node.endpoint_hint.clone();
                            let n_standalone = if node.standalone_without_online_vps {
                                "Standalone Local-First Capable (No Online Cloud VPS Required)"
                            } else {
                                "Online Ingress Relay & Shamir .q42j Mirror"
                            };
                            let n_keep = if node.holds_local_sanctuary_keep {
                                "Holds Local Sanctuary Keep (:8080)"
                            } else {
                                "Zero Raw Sanctuary Keys (Encrypted 2-of-3 Shamir .q42j Only)"
                            };
                            let n_tier = node.qcdn_seeder_tier.clone();
                            rsx! {
                                div {
                                    key: "{n_id}",
                                    class: "plane-card",
                                    h5 { "{n_label} ({n_id})" }
                                    p { "Locus: {n_locus}" }
                                    p { "Server Runtime Operators: [{n_ops}]" }
                                    p { "QDNF ULA: {n_ula} · Endpoint: {n_ep}" }
                                    p { "{n_standalone} · {n_keep}" }
                                    p { "Q-CDN Role: {n_tier} — {n_routing}" }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "orb-sectors",
                    for locus in HostNodeLocusKind::ALL {
                        {
                            let l_title = locus.title();
                            let l_code = locus.code();
                            let store = store_fleet.clone();
                            rsx! {
                                button {
                                    key: "{l_code}",
                                    r#type: "button",
                                    class: "ribbon-btn",
                                    onclick: move |_| {
                                        if let Some(sess) = session() {
                                            let mut s = state.write();
                                            let seq = s.host_stewardship.host_fleet_nodes.len() + 1;
                                            let new_id = format!("node:{l_code}-{seq}");
                                            let steward_did = s.host_vote_steward_input.clone();
                                            let standalone = matches!(
                                                locus,
                                                HostNodeLocusKind::HomeServerAppliance
                                                    | HostNodeLocusKind::CamperMobileRigNode
                                            );
                                            if let Ok(added) = s.host_stewardship.register_host_fleet_node(
                                                &sess.host.storage,
                                                &new_id,
                                                &format!("{} #{}", l_title, seq),
                                                locus,
                                                vec![steward_did],
                                                &format!("qpr://{l_code}-{seq}.mesh:4242"),
                                                standalone,
                                            ) {
                                                s.refresh_host_profiles();
                                                if let Some(store) = &store {
                                                    refresh_refract_host_records(store, &s.host_stewardship);
                                                }
                                                s.host_stewardship_toast = format!(
                                                    "Added Fleet Host Node '{}' ({}) at QDNF ULA {} (standalone_without_online_vps={})",
                                                    added.label, added.node_id, added.qdnf_ula_ipv6, added.standalone_without_online_vps
                                                );
                                            }
                                        }
                                    },
                                    "+ Register {l_title}"
                                }
                            }
                        }
                    }
                }
            }
            }

            if tab == AdminSubTab::ProjectsAndJuralGate {
            // 7. Server Runtime Responsibility vs Hosted Project / Website / Domain Stewardship & Q-CDN Swarm Lifecycle
            section {
                class: "id-list",
                h4 { "Server Runtime Responsibility != Hosted Project / Website / Domain Ownership ({hosted_projects.len()} Workloads · Personal -> Group Transitions & Q-CDN Swarm)" }
                p {
                    class: "muted",
                    "Separates who operates the server runtime (OS/kernel .deb updates, power, firewall) from who stewards a project, website, or domain living on it. A project can be another person's or group's project being supported on your host, a personal project that evolves into a group project, or a native cooperative project — forming a user-powered Q-CDN Swarm."
                }
                div {
                    class: "plane-grid",
                    for proj in &hosted_projects {
                        {
                            let pr_id = proj.project_id.clone();
                            let pr_id_group = pr_id.clone();
                            let pr_id_support = pr_id.clone();
                            let pr_title = proj.title.clone();
                            let pr_dom = proj.attached_domain.clone();
                            let pr_stage = proj.ownership_stage.title();
                            let pr_stewards = proj.project_steward_dids.join(", ");
                            let pr_m = proj.project_quorum_threshold_m;
                            let pr_n = proj.project_steward_dids.len();
                            let pr_nodes = proj.hosted_on_runtime_node_ids.join(", ");
                            let pr_qcdn = proj.qcdn_swarm_mirror_dids.join(", ");
                            let pr_merkle = proj.qcdn_signed_root_merkle_hex.clone();
                            let pr_corp = proj
                                .incorporated_entity_id
                                .clone()
                                .unwrap_or_else(|| "None (Human-Centric / Permissive Commons Lane)".into());
                            let pr_hist = proj
                                .provenance_history
                                .last()
                                .cloned()
                                .unwrap_or_default();
                            let store_group = store_projects.clone();
                            let store_swarm = store_projects.clone();
                            rsx! {
                                div {
                                    key: "{pr_id}",
                                    class: "plane-card",
                                    h5 { "{pr_title} ({pr_dom})" }
                                    p { "Stage: {pr_stage}" }
                                    p { "Project / Domain Stewards ({pr_m}-of-{pr_n} Quorum): [{pr_stewards}]" }
                                    p { "Hosted on Runtime Nodes: [{pr_nodes}]" }
                                    p { "Q-CDN Swarm Seeders (<10 KiB SignedRootIndex {pr_merkle}): [{pr_qcdn}]" }
                                    p { "Incorporated Entity Binding: {pr_corp}" }
                                    p { "Latest Provenance: {pr_hist}" }
                                    div {
                                        class: "orb-sectors",
                                        button {
                                            r#type: "button",
                                            class: "sanctum-issue",
                                            onclick: move |_| {
                                                if let Some(sess) = session() {
                                                    let mut s = state.write();
                                                    if let Ok(updated) = s
                                                        .host_stewardship
                                                        .transition_project_to_group_or_qcdn_swarm(
                                                            &sess.host.storage,
                                                            &pr_id_group,
                                                            HostedProjectOwnershipStage::TransitionedPersonalToGroupProject,
                                                            vec![
                                                                "did:qi:steward:community-grounds-costeward".into(),
                                                                "did:qi:peer:talbingo-ecologist".into(),
                                                            ],
                                                            2,
                                                            vec![
                                                                "did:qi:coop_vps_mirror_1".into(),
                                                                "did:qi:steward:mobile-avf-pocket".into(),
                                                            ],
                                                        )
                                                    {
                                                        s.refresh_host_profiles();
                                                        if let Some(store) = &store_group {
                                                            refresh_refract_host_records(store, &s.host_stewardship);
                                                        }
                                                        s.host_stewardship_toast = format!(
                                                            "Transitioned '{}' ({}) into Multi-Person Group Project ({}-of-{} quorum) with {} registered Q-CDN Swarm seeders",
                                                            updated.title,
                                                            updated.attached_domain,
                                                            updated.project_quorum_threshold_m,
                                                            updated.project_steward_dids.len(),
                                                            updated.qcdn_swarm_mirror_dids.len()
                                                        );
                                                    }
                                                }
                                            },
                                            "Evolve Personal Project -> Multi-Person Group Project + Q-CDN Swarm"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            onclick: move |_| {
                                                if let Some(sess) = session() {
                                                    let mut s = state.write();
                                                    if let Ok(updated) = s
                                                        .host_stewardship
                                                        .transition_project_to_group_or_qcdn_swarm(
                                                            &sess.host.storage,
                                                            &pr_id_support,
                                                            HostedProjectOwnershipStage::SupportedThirdPartyOrGroupProject,
                                                            vec!["did:qi:peer:external-group-steward".into()],
                                                            2,
                                                            vec!["did:qi:coop_vps_mirror_2".into()],
                                                        )
                                                    {
                                                        s.refresh_host_profiles();
                                                        if let Some(store) = &store_swarm {
                                                            refresh_refract_host_records(store, &s.host_stewardship);
                                                        }
                                                        s.host_stewardship_toast = format!(
                                                            "Configured '{}' ({}) as Mutual-Aid Supported Project & Q-CDN Swarm Mirror",
                                                            updated.title, updated.attached_domain
                                                        );
                                                    }
                                                }
                                            },
                                            "Support as Mutual-Aid / Another Group's Project on Q-CDN"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 8. Three-Way Jural Actor Distinction & Mandatory Commercial Licensing for Incorporated Entities
            section {
                class: "id-list",
                h4 { "Jural Personality & Commercial Licensing Gate ({incorporated_entities.len()} Incorporated Entities · Stakeholder Legal Personality != Natural Person)" }
                p {
                    class: "muted",
                    "Strictly separates: (1) Living Natural Persons ($0 Free Human-Centric Commons Lane), (2) Natural Persons acting as authorized agents for an Incorporated Entity (explicit principal-agent binding; corporate commercial licence applies), and (3) Incorporated Legal Entities (governed by structural stakeholder factors — board, shares, charter — and ALWAYS subject to mandatory commercial licensing irrespective of incorporation subtype, with tiered pricing)."
                }
                div {
                    class: "orb-sectors",
                    for actor_kind in ActorJuralPersonalityKind::ALL {
                        {
                            let ak_title = actor_kind.title();
                            rsx! {
                                button {
                                    key: "{ak_title}",
                                    r#type: "button",
                                    class: "sanctum-issue",
                                    onclick: move |_| {
                                        let s_read = state.read();
                                        let first_corp = s_read
                                            .host_stewardship
                                            .incorporated_entities
                                            .first()
                                            .map(|c| c.entity_did.clone());
                                        let res = s_read
                                            .host_stewardship
                                            .evaluate_jural_actor_commercial_obligation(
                                                actor_kind,
                                                first_corp.as_deref(),
                                            );
                                        drop(s_read);
                                        if let Ok((_, _, summary)) = res {
                                            state.write().host_stewardship_toast = summary;
                                        }
                                    },
                                    "Evaluate Gate: {ak_title}"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "plane-grid",
                    for corp in &incorporated_entities {
                        {
                            let c_did = corp.entity_did.clone();
                            let c_name = corp.legal_name.clone();
                            let c_reg = corp.jurisdiction_and_reg_number.clone();
                            let c_tier = corp.pricing_tier.title();
                            let c_fee_dollars = corp.monthly_base_fee_cents / 100;
                            let c_fee_cents = corp.monthly_base_fee_cents % 100;
                            let c_gib = corp.qcdn_carriage_cents_per_gib;
                            let c_stakeholders = corp.stakeholder_governance_factors.join(" · ");
                            let c_agents = corp.authorized_natural_person_agent_dids.join(", ");
                            let c_rail = corp.settlement_rail.clone();
                            rsx! {
                                div {
                                    key: "{c_did}",
                                    class: "plane-card",
                                    h5 { "[COMMERCIAL LICENCE ACTIVE] {c_name} ({c_did})" }
                                    p { "Registration & Jurisdiction: {c_reg}" }
                                    p { "Commercial Pricing Tier: {c_tier} (${c_fee_dollars}.{c_fee_cents:02}/mo base + {c_gib}c/GiB Q-CDN carriage)" }
                                    p { "Structural Stakeholder Factors (Legal Personality != Users): {c_stakeholders}" }
                                    p { "Authorized Natural-Person Corporate Agents: [{c_agents}]" }
                                    p { "Two-Loop Settlement Rail: {c_rail}" }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "orb-sectors",
                    for tier in IncorporatedEntityPricingTier::ALL {
                        {
                            let t_title = tier.title();
                            let t_fee = tier.monthly_base_commercial_fee_cents() / 100;
                            rsx! {
                                button {
                                    key: "{t_title}",
                                    r#type: "button",
                                    class: "ribbon-btn",
                                    onclick: move |_| {
                                        if let Some(sess) = session() {
                                            let mut s = state.write();
                                            let seq = s.host_stewardship.incorporated_entities.len() + 1;
                                            let new_did = format!("did:qi:corp:incorporated-entity-{seq}");
                                            let agent_did = s.host_vote_steward_input.clone();
                                            if let Ok(rec) = s
                                                .host_stewardship
                                                .register_incorporated_entity_commercial_licence(
                                                    &sess.host.storage,
                                                    &new_did,
                                                    &format!("Incorporated Stakeholder Entity #{seq}"),
                                                    &format!("AU-ASIC · ACN 650 000 {seq:03}"),
                                                    tier,
                                                    vec![
                                                        "Board of Directors & Constitutional Charter".into(),
                                                        "Stakeholder Voting & Fiduciary Structure".into(),
                                                    ],
                                                    vec![agent_did],
                                                    "QualiaPQ-Ledger (QPQ) / eCash (XEC) / BTC-LN / USDC",
                                                )
                                            {
                                                s.host_stewardship_toast = format!(
                                                    "Registered Incorporated Entity '{}' ({}) under Mandatory Commercial Licence: ${}.{:02}/mo ({})",
                                                    rec.legal_name,
                                                    rec.entity_did,
                                                    rec.monthly_base_fee_cents / 100,
                                                    rec.monthly_base_fee_cents % 100,
                                                    rec.pricing_tier.title()
                                                );
                                            }
                                        }
                                    },
                                    "+ Register {t_title} (${t_fee}/mo)"
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

fn guard_host_beforeinput(mut state: Signal<ShellState>, event: BeforeInputEvent) {
    let data = event.data();
    let Some(text) = data.data() else {
        return;
    };
    let candidate = format!("{}{text}", data.value());
    if validate_human_centric_host_nomenclature(&text).is_err()
        || validate_human_centric_host_nomenclature(&candidate).is_err()
    {
        event.prevent_default();
        state.write().beforeinput_guard_notice =
            "Use Human-Centric vocabulary for stewardship and project records.".into();
    } else {
        state.write().beforeinput_guard_notice.clear();
    }
}

fn set_governed_input(
    mut state: Signal<ShellState>,
    value: &str,
    set: impl FnOnce(&mut ShellState, String),
) {
    let mut s = state.write();
    if validate_human_centric_host_nomenclature(value).is_ok() {
        set(&mut s, value.to_owned());
        s.beforeinput_guard_notice.clear();
    } else {
        s.beforeinput_guard_notice =
            "Use Human-Centric vocabulary for stewardship and project records.".into();
    }
}

pub(crate) fn build_appliance_packages(mut state: Signal<ShellState>) {
    let Some(sess) = session() else {
        state.write().host_stewardship_toast =
            "Open a local host session to build packages.".into();
        return;
    };
    let fabric = state.read().host_stewardship.clone();
    match generate_tri_target_debian_suite(&sess.host, &sess.gpu, &fabric) {
        Ok(receipt) => {
            let archives: Vec<_> = receipt
                .built_deb_packages
                .iter()
                .map(|package| {
                    format!(
                        "{} ({} bytes, SHA3-256 {})",
                        package.output_deb_path.display(),
                        package.archive_bytes,
                        package.sha3_256_hex,
                    )
                })
                .collect();
            let mut s = state.write();
            s.delivery_status = archives.join(" · ");
            if let Some(apt) = receipt.apt_repository {
                let apt_status = if apt.signed {
                    format!("Verified APT InRelease · {}", apt.inrelease_path.display())
                } else {
                    format!(
                        "APT indexes awaiting signatures · {}",
                        apt.release_path.display()
                    )
                };
                s.delivery_status.push_str(" · ");
                s.delivery_status.push_str(&apt_status);
            }
            s.host_stewardship_toast = format!(
                "Built {} Debian archives in {}. Host state: {}. Update gate: {}.",
                archives.len(),
                receipt.suite_root_dir.display(),
                receipt.host_state_cbor_ld_path.display(),
                receipt.host_quorum_gate_script_path.display(),
            );
        }
        Err(error) => {
            state.write().host_stewardship_toast =
                format!("Debian archive generation failed: {error}")
        }
    }
}

#[component]
fn MeshSimulationPanel(mut state: Signal<ShellState>) -> Element {
    let result = state.read().mesh_simulation.clone();
    rsx! {
        section {
            class: "id-list mesh-sim-timeline",
            h4 { "3-Node Mesh Lifecycle Simulation" }
            p { "Desktop Appliance · Mobile AVF Pocket Co-Signer · Camper/Home webizen-hostd" }
            p { "The checks use isolated fixture data and a synthetic peer backend." }
            button {
                r#type: "button",
                class: "sanctum-issue",
                onclick: move |_| {
                    let outcome = session()
                        .ok_or_else(|| "Open a local shell session before running the fixture.".to_string())
                        .and_then(|session| {
                            run_three_node_mesh_simulation(&session.host.storage, &session.identifiers, &session.gpu)
                        });
                    state.write().mesh_simulation = Some(outcome);
                },
                "Run 3-Node Mesh Lifecycle Simulation"
            }
            match result {
                Some(Ok(receipt)) => rsx! {
                    p {
                        role: "status",
                        if receipt.all_stages_passed {
                            "All {receipt.total_nquin_seals} fixture stages passed."
                        } else {
                            "The fixture recorded {receipt.total_nquin_seals} stage receipts. Review blocked or failed stages below."
                        }
                    }
                    if !receipt.apt_repo_inrelease_path.is_empty() {
                        p { "Verified APT InRelease · {receipt.apt_repo_inrelease_path}" }
                    }
                    ol {
                        for stage in receipt.stages {
                            {
                                let number = stage.stage_index;
                                let status = stage.status.label();
                                let parity = format!("{:016x}", stage.sealed_nquin_parity);
                                rsx! {
                                    li {
                                        key: "{number}",
                                        class: "mesh-stage-card",
                                        h5 { "{stage.title}" }
                                        p { "{status} · Parity {parity}" }
                                        p { "{stage.evidence}" }
                                    }
                                }
                            }
                        }
                    }
                },
                Some(Err(error)) => rsx! { p { role: "status", "Mesh fixture could not start · {error}" } },
                None => rsx! {},
            }
        }
    }
}

#[component]
pub(super) fn HostDaemonPanel(mut state: Signal<ShellState>) -> Element {
    let snapshot = state.read().host_daemon_snapshot.clone();
    let domain = snapshot.host_domain;
    let nodes = snapshot.fleet_nodes;
    let projects = snapshot.hosted_projects;
    let circles = snapshot.care_circles;
    let alerts = snapshot.active_care_alerts;
    let threshold = snapshot.quorum_threshold;
    let stewards = snapshot.steward_count;
    let queued = snapshot.queued_updates;
    rsx! {
        section {
            class: "id-list",
            h4 { "webizen-hostd · Saved Stewardship State" }
            p { "{domain} · {nodes} fleet nodes · {projects} hosted projects" }
            p { "{queued} queued proposals · Review threshold {threshold} of {stewards}" }
            p { "{circles} care circles · {alerts} active care alerts" }
            button {
                r#type: "button",
                onclick: move |_| {
                    let Some(sess) = session() else {
                        state.write().host_stewardship_toast = "Open a local host session to run a daemon tick.".into();
                        return;
                    };
                    match run_daemon_tick(&HostDaemonConfig::for_storage(&sess.host.storage)) {
                        Ok(report) => {
                            let mut s = state.write();
                            s.host_daemon_snapshot = report.status;
                            s.host_stewardship_toast = "Daemon tick refreshed the saved-state summary.".into();
                        }
                        Err(error) => state.write().host_stewardship_toast = format!("Daemon tick stopped: {error}"),
                    }
                },
                "Run Local Daemon Tick"
            }
        }
    }
}

#[component]
pub(super) fn ApplianceProfilesPanel(mut state: Signal<ShellState>) -> Element {
    let (desktop, mobile, open_alerts) = {
        let s = state.read();
        let open_alerts = s
            .host_stewardship
            .care_alerts
            .iter()
            .filter(|alert| !alert.resolved_or_reversed)
            .map(|alert| (alert.alert_id.clone(), alert.trigger_kind.label()))
            .collect::<Vec<_>>();
        (
            s.desktop_appliance.clone(),
            s.mobile_avf.clone(),
            open_alerts,
        )
    };
    let bridge = try_consume_context::<AuxWindowBridge>();
    let authority = try_consume_context::<CareResponseAuthority>();
    let store = try_consume_context::<RefractStore<WebizenRefractModel>>();
    let active_preset = desktop.active_workspace.title();
    let host = desktop.hostname;
    let standalone_count = desktop.fleet.standalone_node_count;
    let fleet_count = desktop.fleet.nodes.len();
    let has_vps = desktop.fleet.has_online_vps;
    let mobile_summary = mobile.probe.summary;
    let attested = mobile.probe.anvil_np64_attested;
    let firmware_present = mobile.probe.pvmfw_dice_present;
    let el2_detected = mobile.probe.pkvm_el2_detected;
    let gpu_transport = mobile.probe.virtio_gpu_venus_active;
    let residency = mobile.governor.recommended_residency.label();
    let kv_cap = mobile.governor.max_kv_cache_cap_mib;
    let thermal = format!("{:?}", mobile.governor.thermal_level);
    let battery = mobile.governor.battery_percent;
    let external_power = mobile.governor.on_external_power;
    let thumb = mobile.orb_geometry;
    let target_size = thumb.target_hit_dp;
    let orb_sweep = thumb.arc_sweep_deg;
    let last_checkin = mobile.last_ok_checkin.map(|rec| rec.duty_of_inquiry_status);
    rsx! {
        section {
            class: "id-list",
            h4 { "Desktop Appliance Workspaces" }
            p { "{host} · {active_preset} · {fleet_count} nodes, {standalone_count} standalone · Online VPS configured: {has_vps}" }
            div {
                class: "orb-sectors",
                for preset in desktop.workspace_presets {
                    {
                        let code = preset.code();
                        let label = preset.title();
                        let active = desktop.active_workspace == preset;
                        rsx! {
                            button {
                                key: "{code}",
                                r#type: "button",
                                aria_pressed: if active { "true" } else { "false" },
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.desktop_appliance.active_workspace = preset;
                                    let stage = match preset.default_stage_slug() {
                                        "poet" => VolumeStage::Poet,
                                        "browser" => VolumeStage::Browser,
                                        "wellfair" => VolumeStage::Wellfair,
                                        "keep" => VolumeStage::Keep,
                                        _ => VolumeStage::None,
                                    };
                                    jump_to_stage(&mut s, stage);
                                    s.launcher_open = false;
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
        section {
            class: "id-list",
            h4 { "Mobile AVF Pocket Sanctuary" }
            p { "{mobile_summary}" }
            p { "pKVM EL2 markers: {el2_detected} · pvmfw DICE markers: {firmware_present} · Anvil NP64 gate: {attested} · Venus active: {gpu_transport}" }
            p { ".p64 governor policy · {thermal} · Battery {battery}% · External power: {external_power} · KV-cache cap {kv_cap} MiB · {residency}" }
            if !mobile.probe.vsock_channels.is_empty() {
                ul {
                    for channel in mobile.probe.vsock_channels {
                        {
                            let label = channel.transport_label;
                            let cid = channel.cid;
                            let port = channel.port;
                            let cap = channel.max_sensitivity_allowed;
                            let tier = format!("{:?}", channel.tier);
                            rsx! {
                                li { "{label} · {tier} · CID {cid}, port {port} · sensitivity cap {cap}" }
                            }
                        }
                    }
                }
            }
            details {
                summary { "8-Sector Command Orb (◉) thumb-arc · {orb_sweep}° sweep · {target_size} dp targets" }
                ul {
                    for sector in thumb.sectors {
                        {
                            let angle = f32::from(sector.angle_deg_tenths) / 10.0;
                            let label = sector.label;
                            let glyph = sector.glyph;
                            rsx! { li { "{glyph} {label} · {angle}°" } }
                        }
                    }
                }
            }
            if let Some(status) = last_checkin {
                p { role: "status", "{status}" }
            }
            if !open_alerts.is_empty() {
                if authority.is_some() {
                    ul {
                        for (alert_id, trigger) in open_alerts {
                            {
                                let authority = authority.clone();
                                let store = store.clone();
                                rsx! {
                                    li {
                                        key: "{alert_id}",
                                        "{trigger} "
                                        button {
                                            r#type: "button",
                                            onclick: move |_| respond_to_alert(state, authority.as_ref(), store.as_ref(), &alert_id, true),
                                            "I am OK"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    p { class: "muted", "One-tap check-in becomes available when the principal is authenticated." }
                }
            }
            if let Some(bridge) = bridge {
                button {
                    r#type: "button",
                    onclick: move |_| bridge.open_urgent_care_alert_modal(),
                    "Open Principal Check-In and Care Responses"
                }
            }
        }
    }
}
