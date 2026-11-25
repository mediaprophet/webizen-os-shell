// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use crate::{
    components::auxiliary_windows::{AuxWindowBridge, CareResponseAuthority},
    state::{
        refresh_refract_host_records, session, wellfair_review_lens, ShellState,
        WebizenRefractModel,
    },
};
use aether::prelude::*;
use aether::{
    refract::{DimensionOutcome, WellFairDimension},
    refract_bridge::{render_fairness_review_panel, RefractStore},
};
use webizen_ua_os::{
    capture_mobile_field_evidence_qecp, execute_pocket_cosigner_ceremony,
    trigger_one_tap_principal_ok_checkin, CareAlertSeverity, CareAlertTriggerKind,
    ComputeExecutionTarget, InstrumentAuthorRole, MobileConnectivityProfile,
    PocketCosignerActionKind, SharedLivingAndCareTopologyKind, SpecialistDomainScope,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WellfairSubTab {
    CaseworkAndPortal,
    GuardianshipAndCare,
    FairnessReview,
}

#[component]
pub fn WellfairVolume(state: Signal<ShellState>) -> Element {
    let mut active_tab = use_signal(|| WellfairSubTab::CaseworkAndPortal);
    let tab = active_tab();
    rsx! {
        div {
            class: "settings-volume",
            div {
                class: "volume-subtabs",
                role: "tablist",
                aria_label: "WellFair workspace sections",
                button {
                    r#type: "button",
                    class: if tab == WellfairSubTab::CaseworkAndPortal { "ribbon-btn active" } else { "ribbon-btn" },
                    onclick: move |_| active_tab.set(WellfairSubTab::CaseworkAndPortal),
                    "1. Casework, Instruments & Specialist Portal"
                }
                button {
                    r#type: "button",
                    class: if tab == WellfairSubTab::GuardianshipAndCare { "ribbon-btn active" } else { "ribbon-btn" },
                    onclick: move |_| active_tab.set(WellfairSubTab::GuardianshipAndCare),
                    "2. Shared Living, Guardianship & Remote Carer Alerts"
                }
                button {
                    r#type: "button",
                    class: if tab == WellfairSubTab::FairnessReview { "ribbon-btn active" } else { "ribbon-btn" },
                    onclick: move |_| active_tab.set(WellfairSubTab::FairnessReview),
                    "3. Six-Dimension Fairness & Rights Review"
                }
            }
            match tab {
                WellfairSubTab::CaseworkAndPortal => rsx! {
                    WellfairHumanRightsAndSpecialistPortalPanel { state }
                },
                WellfairSubTab::GuardianshipAndCare => rsx! {
                    GuardianshipAndCarePanel { state }
                },
                WellfairSubTab::FairnessReview => rsx! {
                    FairnessDraftPanel {}
                },
            }
        }
    }
}

#[component]
fn FairnessDraftPanel() -> Element {
    let Some(store) = try_consume_context::<RefractStore<WebizenRefractModel>>() else {
        return rsx! { p { "Fairness review becomes available with the local session." } };
    };
    let lens = wellfair_review_lens();
    let review = store.get_cloned(lens);
    rsx! {
        section {
            class: "id-list",
            h4 { "Fairness and Rights Review Draft" }
            p { "Review all six dimensions. Draft choices need supporting evidence before they enter the governed review." }
            {render_fairness_review_panel(&review)}
            div {
                class: "orb-sectors",
                for dimension in WellFairDimension::ALL {
                    {
                        let code = dimension.code();
                        let title = dimension.title();
                        let store = store.clone();
                        rsx! {
                            button {
                                key: "{code}",
                                r#type: "button",
                                onclick: move |_| {
                                    store.write(lens, |review| {
                                        let (outcome, remedy) = next_review_state(review.outcome(dimension));
                                        review.outcomes[dimension as usize] = outcome;
                                        review.remedy_notes[dimension as usize] = remedy;
                                    });
                                },
                                "Change draft outcome for {title}"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn next_review_state(outcome: DimensionOutcome) -> (DimensionOutcome, &'static str) {
    match outcome {
        DimensionOutcome::Held => (
            DimensionOutcome::Satisfied,
            "Confirm the supporting evidence for this draft outcome.",
        ),
        DimensionOutcome::Satisfied => (
            DimensionOutcome::Failed,
            "Record the required remedy and schedule review.",
        ),
        DimensionOutcome::Failed => (
            DimensionOutcome::NotApplicable,
            "Document why this dimension falls outside the review scope.",
        ),
        DimensionOutcome::NotApplicable => (
            DimensionOutcome::Held,
            "Collect prerequisite attestations for this dimension.",
        ),
    }
}

pub(super) fn respond_to_alert(
    mut state: Signal<ShellState>,
    authority: Option<&CareResponseAuthority>,
    store: Option<&RefractStore<WebizenRefractModel>>,
    alert_id: &str,
    principal_ok: bool,
) {
    let Some(sess) = session() else {
        state.write().host_stewardship_toast =
            "Open a local session to record a care response.".into();
        return;
    };
    let alert = state
        .read()
        .host_stewardship
        .care_alerts
        .iter()
        .find(|alert| alert.alert_id == alert_id && !alert.resolved_or_reversed)
        .cloned();
    let Some(alert) = alert else {
        state.write().host_stewardship_toast =
            "This alert is no longer awaiting a response.".into();
        return;
    };
    let Some(authority) = authority else {
        state.write().host_stewardship_toast =
            "An authenticated care responder is required.".into();
        return;
    };
    let responder_did = match (authority.resolve)(&alert, principal_ok) {
        Ok(did) => did,
        Err(error) => {
            state.write().host_stewardship_toast = error;
            return;
        }
    };
    let mut s = state.write();
    let result = if principal_ok {
        trigger_one_tap_principal_ok_checkin(
            &mut s.host_stewardship,
            &sess.host.storage,
            alert_id,
            &responder_did,
            Some("Principal confirmed I am OK"),
        )
        .map(|receipt| {
            let status = receipt.duty_of_inquiry_status.clone();
            s.mobile_avf.last_ok_checkin = Some(receipt);
            status
        })
    } else {
        s.host_stewardship
            .acknowledge_or_reverse_care_alert(
                &sess.host.storage,
                alert_id,
                &responder_did,
                false,
                "Carer confirmed that they carried out a duty-of-inquiry check",
            )
            .map(|updated| updated.duty_of_inquiry_status)
    };
    match result {
        Ok(status) => {
            s.host_stewardship_toast = status;
            s.refresh_host_profiles();
            if let Some(store) = store {
                refresh_refract_host_records(store, &s.host_stewardship);
            }
        }
        Err(error) => s.host_stewardship_toast = error,
    }
}

#[component]
pub fn GuardianshipAndCarePanel(mut state: Signal<ShellState>) -> Element {
    let authority = try_consume_context::<CareResponseAuthority>();
    let store = try_consume_context::<RefractStore<WebizenRefractModel>>();
    let bridge = try_consume_context::<AuxWindowBridge>();
    let mut ledger_notice = use_signal(String::new);
    let s = state.read();
    let vault = s.storage_display.clone();
    let circles = s.host_stewardship.shared_care_circles.clone();
    let alerts = s.host_stewardship.care_alerts.clone();
    let toast = s.host_stewardship_toast.clone();
    drop(s);

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "WellFair · Shared Systems, Relational Guardianship (UN CRC / CRPD) & Remote Carer Alerts" }
                p {
                    "Qualia wellfare-core fiduciary vault where the natural person is the principal: models Share-Houses (equal adult peers with isolated Sanctuary vaults and zero peer guardianship), Families with Children (AgencyStage gradient, own DigitalBirthRecord, split StewardRoles, flagged-ingest GuardianNotifications), and Elderly / Supported Persons with Remote Family Members or Carers elsewhere (scoped QDNF heartbeat/medication/fall alerts with reversible 'I am OK' check-in and DutyOfInquiry accountability)."
                }
            }

            if let Some(bridge) = bridge {
                button {
                    r#type: "button",
                    onclick: move |_| bridge.open_urgent_care_alert_modal(),
                    "Open Urgent Care Alerts Window"
                }
            }
            if authority.is_none() {
                p { class: "muted", "Care responses require an authenticated responder." }
            }
            if !toast.is_empty() {
                p { class: "path", "{toast}" }
            }

            // 1. Three Shared Living & Distributed Care Topologies
            section {
                class: "id-list",
                h4 { "Three Shared Living & Distributed Guardianship / Care Topologies ({circles.len()} Active Circles)" }
                div {
                    class: "plane-grid",
                    for c in &circles {
                        {
                            let c_id = c.circle_id.clone();
                            let c_id_alert = c_id.clone();
                            let c_title = c.title.clone();
                            let c_topo = c.topology;
                            let c_topo_title = c_topo.title();
                            let c_invariant = c_topo.privacy_and_alert_invariant();
                            let c_principal = c.principal_did.clone();
                            let c_stage = c.principal_agency_stage.clone();
                            let c_node = c.host_node_id.clone();
                            let c_checkin = c.last_principal_checkin_note.clone();
                            let c_esc = if c.escalation_currently_active {
                                "ESCALATION ACTIVE (Awaiting Remote Carer Response or Principal 'I am OK' Reversal)"
                            } else {
                                "Normal / Check-In Verified"
                            };
                            let members_lines: Vec<String> = c
                                .participants
                                .iter()
                                .map(|p| {
                                    format!(
                                        "{} ({}) — Role: {} · StewardRoles: [{}] · Locus: {} · MinAlertSeverity >= {} · Credential: {}",
                                        p.display_name,
                                        p.participant_did,
                                        p.relationship_role,
                                        p.steward_roles.join(", "),
                                        p.physical_locus,
                                        p.alert_min_severity_level,
                                        p.consent_credential_id
                                    )
                                })
                                .collect();
                            rsx! {
                                div {
                                    key: "{c_id}",
                                    class: "plane-card",
                                    h5 { "{c_title}" }
                                    p { "Topology: {c_topo_title}" }
                                    p { "Principal: {c_principal} · {c_stage} · Host Node: {c_node}" }
                                    p { "Status: [{c_esc}] · Latest Check-In: \"{c_checkin}\"" }
                                    p { "Privacy & Alert Invariant: {c_invariant}" }
                                    ul {
                                        for m_line in members_lines {
                                            li { "{m_line}" }
                                        }
                                    }
                                    div {
                                        class: "orb-sectors",
                                        if c_topo == SharedLivingAndCareTopologyKind::ElderlyOrSupportedPersonWithRemoteCarers {
                                            button {
                                                r#type: "button",
                                                class: "sanctum-issue",
                                                onclick: {
                                                    let cid = c_id_alert.clone();
                                                    move |_| {
                                                        if let Some(sess) = session() {
                                                            let mut s = state.write();
                                                            if let Ok(al) = s
                                                                .host_stewardship
                                                                .dispatch_shared_care_or_guardian_alert(
                                                                    &sess.host.storage,
                                                                    &cid,
                                                                    CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse,
                                                                    CareAlertSeverity::Urgent,
                                                                    "Local simulation: 12h liveness heartbeat lapse and hallway inactivity",
                                                                )
                                                            {
                                                                s.refresh_host_profiles();
                                                                s.host_stewardship_toast = format!(
                                                                    "Recorded local urgent alert simulation {} with {} scoped recipients",
                                                                    al.alert_id,
                                                                    al.notified_remote_guardian_dids.len()
                                                                );
                                                            }
                                                        }
                                                    }
                                                },
                                                "Simulate Urgent Elderly Heartbeat / Fall Alert to Remote Family & Carers"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "ribbon-btn",
                                                onclick: {
                                                    let cid = c_id_alert.clone();
                                                    move |_| {
                                                        if let Some(sess) = session() {
                                                            let mut s = state.write();
                                                            if let Ok(al) = s
                                                                .host_stewardship
                                                                .dispatch_shared_care_or_guardian_alert(
                                                                    &sess.host.storage,
                                                                    &cid,
                                                                    CareAlertTriggerKind::MissedMedicationSchedule,
                                                                    CareAlertSeverity::Concern,
                                                                    "Local simulation: evening medication reminder unacknowledged after 60 minutes",
                                                                )
                                                            {
                                                                s.refresh_host_profiles();
                                                                s.host_stewardship_toast = format!(
                                                                    "Recorded local medication alert simulation {} with {} scoped recipients",
                                                                    al.alert_id,
                                                                    al.notified_remote_guardian_dids.len()
                                                                );
                                                            }
                                                        }
                                                    }
                                                },
                                                "Simulate Missed Medication Alert (Concern Level 2)"
                                            }
                                        }
                                        if c_topo == SharedLivingAndCareTopologyKind::FamilyWithChildrenGuardianship {
                                            button {
                                                r#type: "button",
                                                class: "sanctum-issue",
                                                onclick: {
                                                    let cid = c_id_alert.clone();
                                                    move |_| {
                                                        if let Some(sess) = session() {
                                                            let mut s = state.write();
                                                            if let Ok(al) = s
                                                                .host_stewardship
                                                                .dispatch_shared_care_or_guardian_alert(
                                                                    &sess.host.storage,
                                                                    &cid,
                                                                    CareAlertTriggerKind::ChildFlaggedContentIngest,
                                                                    CareAlertSeverity::Concern,
                                                                    "Flagged content ingest (Concern level 2) & proxy write held in M-of-N GuardianshipProposal escrow for parent review",
                                                                )
                                                            {
                                                                s.refresh_host_profiles();
                                                                s.host_stewardship_toast = format!(
                                                                    "Recorded family notification {} with {} scoped parent stewards",
                                                                    al.alert_id,
                                                                    al.notified_remote_guardian_dids.len()
                                                                );
                                                            }
                                                        }
                                                    }
                                                },
                                                "Trigger Child Flagged-Ingest GuardianNotification & Escrow"
                                            }
                                        }
                                        if c_topo == SharedLivingAndCareTopologyKind::ShareHousePeerCoLiving {
                                            button {
                                                r#type: "button",
                                                class: "sanctum-issue",
                                                onclick: {
                                                    let cid = c_id_alert.clone();
                                                    move |_| {
                                                        if let Some(sess) = session() {
                                                            let mut s = state.write();
                                                            if let Ok(al) = s
                                                                .host_stewardship
                                                                .dispatch_shared_care_or_guardian_alert(
                                                                    &sess.host.storage,
                                                                    &cid,
                                                                    CareAlertTriggerKind::ShareHouseCommonsUtilityNotice,
                                                                    CareAlertSeverity::Notice,
                                                                    "Shared solar battery & internet utility split ready for housemate M-of-N sign-off (personal Sanctuary vaults remain isolated)",
                                                                )
                                                            {
                                                                s.refresh_host_profiles();
                                                                s.host_stewardship_toast = format!(
                                                                    "Recorded Share-House Commons Utility Notice {}",
                                                                    al.alert_id
                                                                );
                                                            }
                                                        }
                                                    }
                                                },
                                                "Post Share-House Commons Utility Quorum Notice"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "ribbon-btn",
                                                onclick: {
                                                    let cid = c_id_alert.clone();
                                                    move |_| {
                                                        if let Some(sess) = session() {
                                                            let mut s = state.write();
                                                            match s
                                                                .host_stewardship
                                                                .dispatch_shared_care_or_guardian_alert(
                                                                    &sess.host.storage,
                                                                    &cid,
                                                                    CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse,
                                                                    CareAlertSeverity::Urgent,
                                                                    "Attempt to monitor flatmate's personal health/heartbeat",
                                                                ) {
                                                                Ok(_) => {}
                                                                Err(e) => {
                                                                    s.host_stewardship_toast = e;
                                                                }
                                                            }
                                                        }
                                                    }
                                                },
                                                "Test Share-House Peer Privacy Fence (Rejects Cross-Peer Guardianship)"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Auditable Guardianship & Remote Carer Alert Stream (AccountabilityLedger + Reversible "I am OK" + DutyOfInquiry)
            p { class: "muted", role: "status", "{ledger_notice}" }
            details {
                class: "id-list",
                onbeforetoggle: move |evt| {
                    if let Some(toggle) = evt.data().downcast::<aether_native::NativeToggleData>() {
                        if toggle.0.old_state != toggle.0.new_state {
                            ledger_notice.set(if toggle.0.new_state { "Opening care alert ledger".into() } else { "Closing care alert ledger".into() });
                        }
                    }
                },
                ontoggle: move |evt| {
                    if let Some(toggle) = evt.data().downcast::<aether_native::NativeToggleData>() {
                        ledger_notice.set(if toggle.0.new_state { "Care alert ledger open".into() } else { "Care alert ledger closed".into() });
                    }
                },
                summary { "Auditable Guardianship & Remote Carer Alert Ledger ({alerts.len()} Dispatched Alerts · Reversible 'I am OK' & DutyOfInquiry)" }
                p {
                    class: "muted",
                    "The local AccountabilityLedger records scoped notifications for parent guardians or remote carers ('guardian_notified'). An elderly or supported principal can always reverse an active escalation by checking in ('I am OK'), and remote carers record Diligent DutyOfInquiry responses."
                }
                ul {
                    for al in &alerts {
                        {
                            let al_id = al.alert_id.clone();
                            let al_id_rev = al_id.clone();
                            let al_id_carer = al_id.clone();
                            let al_sev = al.severity.label();
                            let al_trig = al.trigger_kind.label();
                            let al_principal = al.principal_did.clone();
                            let response_available = !al.resolved_or_reversed && authority.is_some();
                            let principal_authority = authority.clone();
                            let carer_authority = authority.clone();
                            let principal_store = store.clone();
                            let carer_store = store.clone();
                            let al_recips = al.notified_remote_guardian_dids.join("; ");
                            let al_sum = al.summary.clone();
                            let al_duty = al.duty_of_inquiry_status.clone();
                            let al_state = if al.resolved_or_reversed {
                                "RESOLVED / REVERSED"
                            } else {
                                "ACTIVE — AWAITING RESPONSE"
                            };
                            let al_parity = format!("{:#018x}", al.sealed_nquin_parity);
                            rsx! {
                                li {
                                    key: "{al_id}",
                                    div {
                                        strong { "[{al_state}] [{al_sev}] {al_trig} " }
                                        span { "· Principal: {al_principal} — \"{al_sum}\"" }
                                    }
                                    div {
                                        span { "Scoped guardian / carer recipients for QDNF: [{al_recips}] · AccountabilityLedger: recorded=true · DutyOfInquiry: {al_duty} · NQuin48 {al_parity}" }
                                    }
                                    div {
                                        class: "orb-sectors",
                                        button {
                                            r#type: "button",
                                            class: "sanctum-issue",
                                            disabled: !response_available,
                                            onclick: move |_| respond_to_alert(state, principal_authority.as_ref(), principal_store.as_ref(), &al_id_rev, true),
                                            "Principal Self Check-In: 'I am OK' (Reverse Escalation)"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            disabled: !response_available,
                                            onclick: move |_| respond_to_alert(state, carer_authority.as_ref(), carer_store.as_ref(), &al_id_carer, false),
                                            "Remote Carer Respond & Verify (DutyOfInquiry -> Diligent)"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            section {
                class: "id-list",
                h4 { "Fiduciary Capabilities Active in WellFair" }
                ul {
                    li { "48-Byte NQuin Epistemic Envelopes: EpistemicStatus, EvidenceType, SensitivityClass with volatile RAM scrubbing on drop" }
                    li { "3D Human Anatomy & Maternal-Fetal Dyad: 17-system atlas (human-welfare.hmc), Progenitor vs gestational mother autonomy, DigitalBirthRecord" }
                    li { "Relational Guardianship (UN CRC / CRPD): AgencyStage gradient, bilateral M-of-N GuardianshipProposal escrow, protective co-guardian veto" }
                    li { "Contested & Personal Safety Records: Disputed-diagnosis reports, housing-safety reports, WelfareStream & GovernmentLetter index" }
                    li { "Duty of Inquiry & Six Vectors of Transparency (Who/When/Why/What/Where/Cost): Distinguishes NoFault, Diligent, UncheckedNoHarm, and Negligent" }
                }
            }
            p { class: "muted", "Sanctuary-guarded WellFair vault root: {vault}" }
        }
    }
}

#[component]
pub fn WellfairHumanRightsAndSpecialistPortalPanel(mut state: Signal<ShellState>) -> Element {
    let mut inst_title = use_signal(|| "Self-Authored ADHD / PTSD Functional & Accommodation Statement".to_string());
    let mut inst_role = use_signal(|| InstrumentAuthorRole::PrincipalSelfAuthored);
    let mut inst_domain = use_signal(|| SpecialistDomainScope::PsychologyAndTraumaInformedSupport);
    let mut inst_author_did = use_signal(|| "did:q42:qrc:local-principal-steward".to_string());
    let mut inst_context = use_signal(|| "Principal Natural Person (Lived-Experience & Accommodation Record)".to_string());
    let mut inst_concepts = use_signal(|| "https://bio2rdf.org/doid:1094, https://bio2rdf.org/doid:2055, https://ns.webcivics.net/crpd#Article13".to_string());
    let mut inst_summary = use_signal(|| "Documents executive-function and trauma-informed communication requirements alongside housing stability priority.".to_string());
    let mut inst_rule = use_signal(|| "IF Subject(?p) AND AttestedOrSelfAuthored(?p, bio2rdf:DOID_1094) THEN RequireAccommodation(?p, crpd:WrittenAsyncNotice)".to_string());

    let mut ont_id = use_signal(|| "bio2rdf-doid-hp-mental-health".to_string());
    let mut ont_title = use_signal(|| "Bio2RDF DOID & HP Neurodevelopmental / Trauma Reference Slice".to_string());
    let mut ont_iri = use_signal(|| "https://bio2rdf.org/doid_hp_wellfair".to_string());
    let mut ont_ttl = use_signal(|| "bio2rdf:DOID_1094 rdfs:label \"ADHD\" .\nbio2rdf:DOID_2055 rdfs:label \"PTSD\" .\ncml:CRPD_Art28 rdfs:label \"Adequate Standard of Living & Social Protection\" .".to_string());
    let mut ont_target = use_signal(|| ComputeExecutionTarget::SynthColabUltraBurst);

    let mut ev_kind = use_signal(|| "Email + QECP PDF Exhibit (.eml / .pdf)".to_string());
    let mut ev_title = use_signal(|| "Correspondence & Clinical Evidence Regarding Housing & Procedural Fairness".to_string());
    let mut ev_uri = use_signal(|| "urn:qualia:qecp:correspondence-exhibit-2026.pdf".to_string());
    let ev_domain = use_signal(|| SpecialistDomainScope::LegalAndHumanRightsAdvocacy);
    let mut ev_body = use_signal(|| "Attached email and PDF letter confirming ADHD and PTSD accommodations, requesting safe housing habitability and diligent duty of inquiry.".to_string());

    let mut spec_name = use_signal(|| "Human-Rights & Housing Legal Advocate".to_string());
    let mut spec_disc = use_signal(|| "Advocacy · Housing · Administrative Law".to_string());
    let mut spec_did = use_signal(|| "did:qi:specialist:human-rights-advocate".to_string());
    let mut spec_token = use_signal(|| "wellfair-specialist-demo-token-2026".to_string());
    let mut spec_domain = use_signal(|| SpecialistDomainScope::LegalAndHumanRightsAdvocacy);
    let mut portal_verify_lease_id = use_signal(|| "lease:specialist:1".to_string());
    let mut portal_verify_did = use_signal(|| "did:qi:specialist:multidisciplinary-team".to_string());
    let mut portal_verify_token = use_signal(|| "wellfair-specialist-demo-token-2026".to_string());
    let mut derived_preview = use_signal(|| None::<String>);

    let s = state.read();
    let portal = s.wellfair_portal.clone();
    let mobile = s.mobile_avf.clone();
    drop(s);

    let notice = portal.last_portal_notice.clone();
    let instruments = portal.semantic_instruments.clone();
    let ontologies = portal.compiled_ontologies.clone();
    let cml_records = portal.cml_evidence_records.clone();
    let leases = portal.specialist_leases.clone();
    let audits = portal.six_vector_audit_ledger.clone();
    let mobile_conn = mobile.connectivity_profile.title();
    let mobile_ceremonies = mobile.pocket_ceremonies.clone();
    let mobile_evidence = mobile.captured_field_evidence.clone();

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "WellFair Human-Rights Casework · Role-Qualified Semantic Instruments · Bio2RDF/CML Pipeline & Online Specialist Portal" }
                p {
                    "Local Desktop & Mobile AVF Sanctuary first: author or ingest Role-Qualified Semantic Instruments (Psychologist/GP clinical determinations such as ADHD or PTSD vs Principal Self-Authored lived-experience instruments vs Contested records), compile Bio2RDF & legal ontologies into .q42 (locally, via webizen-hostd, or via Synth Colab Ultra Burst) and share public ontologies on the Permissive Commons Q-CDN, correlate emails/PDFs (QECP) against CML human-rights law, and grant strong-authenticated online portal leases to your specialists."
                }
            }
            if !notice.is_empty() {
                p { class: "path", role: "status", "{notice}" }
            }

            // 1. Role-Qualified Clinical, Diagnostic & Wellbeing Semantic Instruments
            section {
                class: "id-list",
                h4 { "1. Role-Qualified Clinical, Diagnostic & Wellbeing Semantic Instruments ({instruments.len()} Sealed · VC + Symbolic AI Rules)" }
                p {
                    class: "muted",
                    "Distinguishes Principal Self-Authored instruments (lived experience & accommodation requirements), Qualified Specialist Attested instruments (e.g. psychologist/GP ADHD or PTSD determination), Collaborative Co-Attested bundles, and Contested/Disputed records (quarantined from positive symbolic AI inference)."
                }
                div {
                    class: "orb-sectors",
                    for role in InstrumentAuthorRole::ALL {
                        {
                            let active = inst_role() == role;
                            let badge = role.badge();
                            rsx! {
                                button {
                                    key: "{badge}",
                                    r#type: "button",
                                    class: if active { "ribbon-btn active" } else { "ribbon-btn" },
                                    onclick: move |_| inst_role.set(role),
                                    "{badge}"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "orb-sectors",
                    for dom in SpecialistDomainScope::ALL {
                        {
                            let active = inst_domain() == dom;
                            let title = dom.title();
                            rsx! {
                                button {
                                    key: "{title}",
                                    r#type: "button",
                                    class: if active { "ribbon-btn active" } else { "ribbon-btn" },
                                    onclick: move |_| inst_domain.set(dom),
                                    "{title}"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{inst_title}",
                        placeholder: "Instrument title (e.g. Psychologist ADHD/PTSD Determination)…",
                        oninput: move |evt| inst_title.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{inst_author_did}",
                        placeholder: "Author DID (did:q42:... or did:qi:specialist:...)…",
                        oninput: move |evt| inst_author_did.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{inst_context}",
                        placeholder: "Qualification / context…",
                        oninput: move |evt| inst_context.set(evt.value()),
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{inst_concepts}",
                        placeholder: "Ontology IRIs (bio2rdf:DOID_1094, bio2rdf:DOID_2055, crpd:Art13)…",
                        oninput: move |evt| inst_concepts.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{inst_summary}",
                        placeholder: "Clinical / functional determination summary…",
                        oninput: move |evt| inst_summary.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{inst_rule}",
                        placeholder: "Symbolic AI inference rule (IF ... THEN RequireAccommodation)...",
                        oninput: move |evt| inst_rule.set(evt.value()),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let iris: Vec<String> = inst_concepts()
                                    .split(',')
                                    .map(|s| s.trim().to_owned())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                let mut s = state.write();
                                let _ = s.wellfair_portal.author_or_register_semantic_instrument(
                                    &sess.host.storage,
                                    &inst_title(),
                                    inst_domain(),
                                    inst_role(),
                                    &inst_author_did(),
                                    &inst_context(),
                                    iris,
                                    &inst_summary(),
                                    &inst_rule(),
                                    vec![
                                        "Written asynchronous correspondence & trauma-informed procedural pacing".into(),
                                        "Safe, quiet, stable housing priority".into(),
                                    ],
                                );
                            }
                        },
                        "Seal Role-Qualified Semantic Instrument"
                    }
                }
                div {
                    class: "plane-grid",
                    for inst in &instruments {
                        {
                            let id = inst.instrument_id.clone();
                            let title = inst.title.clone();
                            let role_badge = inst.author_role.badge();
                            let dom_title = inst.domain.title();
                            let author = inst.author_did.clone();
                            let qual = inst.author_qualification_or_context.clone();
                            let iris = inst.ontology_concept_iris.join(" · ");
                            let summary = inst.determination_summary.clone();
                            let rule = inst.symbolic_inference_rule.clone();
                            let accs = inst.derived_accommodations.join("; ");
                            let parity = format!("{:#018x}", inst.sealed_nquin_parity);
                            rsx! {
                                div {
                                    key: "{id}",
                                    class: "plane-card",
                                    h5 { "{title}" }
                                    p { strong { "[{role_badge}]" } }
                                    p { "Domain: {dom_title}" }
                                    p { "Author: {author} ({qual})" }
                                    p { "Ontology Concepts: [{iris}]" }
                                    p { "Determination: {summary}" }
                                    p { class: "path", "Symbolic AI Rule: {rule}" }
                                    p { "Derived Accommodations: {accs} · NQuin48 {parity}" }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Bio2RDF / Biomedical & Legal Ontology -> .q42 Compiler & Permissive Commons Q-CDN Sharing
            section {
                class: "id-list",
                h4 { "2. Bio2RDF / DOID / HP & Legal Ontology -> .q42 Compiler + Permissive Commons Q-CDN Distribution ({ontologies.len()} Vaults)" }
                p {
                    class: "muted",
                    "Compile biomedical (Bio2RDF, DOID, HP, DrugBank) and legal/human-rights (ns.webcivics.net CML) ontologies into .q42 containers locally, on webizen-hostd, or via Synth Colab Ultra Burst. Public reference ontologies are shared across the Permissive Commons Q-CDN Swarm (<10 KiB SignedRootIndex) while personal casework stays strictly inside your Sanctuary."
                }
                div {
                    class: "orb-sectors",
                    for target in ComputeExecutionTarget::ALL {
                        {
                            let active = ont_target() == target;
                            let t_title = target.title();
                            rsx! {
                                button {
                                    key: "{t_title}",
                                    r#type: "button",
                                    class: if active { "ribbon-btn active" } else { "ribbon-btn" },
                                    onclick: move |_| ont_target.set(target),
                                    "{t_title}"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{ont_id}",
                        placeholder: "Ontology slug (e.g. bio2rdf-doid-hp-mental-health)…",
                        oninput: move |evt| ont_id.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{ont_title}",
                        placeholder: "Ontology title…",
                        oninput: move |evt| ont_title.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{ont_iri}",
                        placeholder: "Namespace IRI (https://bio2rdf.org/...)…",
                        oninput: move |evt| ont_iri.set(evt.value()),
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{ont_ttl}",
                        placeholder: "RDF Turtle / N3 axioms to compile into .q42…",
                        oninput: move |evt| ont_ttl.set(evt.value()),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let target = ont_target();
                                let title = ont_title();
                                let _ = s.orchestrator.route_elastic_compute_workload(
                                    Some(&sess.host.storage),
                                    &format!("Compile Ontology -> .q42: {title}"),
                                    target,
                                    if target == ComputeExecutionTarget::SynthColabUltraBurst { 2048 } else { 32 },
                                    false,
                                    true,
                                );
                                let _ = s.wellfair_portal.compile_ontology_to_q42_and_share_commons(
                                    &sess.host.storage,
                                    &ont_id(),
                                    &title,
                                    &ont_iri(),
                                    "Bio2RDF / DOID / HP & CML Legal Reference",
                                    &ont_ttl(),
                                    target,
                                    true,
                                );
                            }
                        },
                        "Compile Ontology to .q42 & Publish Public Schema to Permissive Commons Q-CDN"
                    }
                }
                ul {
                    for ont in &ontologies {
                        {
                            let id = ont.ontology_id.clone();
                            let title = ont.title.clone();
                            let iri = ont.source_namespace_iri.clone();
                            let target = ont.execution_target_used.title();
                            let path = ont.compiled_q42_path.display().to_string();
                            let count = ont.triple_nquin_count;
                            let bytes = ont.q42_byte_len;
                            let merkle = ont.qcdn_signed_root_merkle_hex.clone();
                            let parity = format!("{:#018x}", ont.sealed_nquin_parity);
                            rsx! {
                                li {
                                    key: "{id}",
                                    strong { "{title} ({id}.q42) " }
                                    span { "· IRI: {iri} · Compiled via: [{target}] · {count} NQuin48 triples ({bytes} bytes) · Path: {path} · Permissive Commons Q-CDN: {merkle} · NQuin48 {parity}" }
                                }
                            }
                        }
                    }
                }
            }

            // 3. CML (ns.webcivics.net) + Legal & Human-Rights Evidence Correlator
            section {
                class: "id-list",
                h4 { "3. CML (ns.webcivics.net) + Legal & Human-Rights Evidence Correlator ({cml_records.len()} Exhibits: Emails, QECP PDFs, Letters)" }
                p {
                    class: "muted",
                    "Ingests emails, QECP verifiable PDFs, and statutory correspondence; extracts CML concepts; correlates them with your active Role-Qualified Semantic Instruments (excluding Contested/Disputed records); and maps applicable UDHR, ICESCR, UN CRPD, and administrative Duty of Inquiry obligations."
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{ev_kind}",
                        placeholder: "Source kind (Email / QECP PDF / Agency Letter)…",
                        oninput: move |evt| ev_kind.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{ev_title}",
                        placeholder: "Document subject or exhibit title…",
                        oninput: move |evt| ev_title.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{ev_uri}",
                        placeholder: "Exhibit URI or SHA3 digest…",
                        oninput: move |evt| ev_uri.set(evt.value()),
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{ev_body}",
                        placeholder: "Paste email, PDF text, or legal/clinical evidence excerpt for CML correlation…",
                        oninput: move |evt| ev_body.set(evt.value()),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let _ = s.wellfair_portal.correlate_cml_evidence_with_instruments_and_rights(
                                    &sess.host.storage,
                                    &ev_kind(),
                                    &ev_title(),
                                    &ev_uri(),
                                    "2026-11-24",
                                    ev_domain(),
                                    &ev_body(),
                                );
                            }
                        },
                        "Run CML + Human-Rights & Semantic Instrument Correlation"
                    }
                }
                ul {
                    for rec in &cml_records {
                        {
                            let id = rec.evidence_id.clone();
                            let kind = rec.source_kind.clone();
                            let title = rec.source_title_or_subject.clone();
                            let dom = rec.domain.title();
                            let concepts = rec.extracted_cml_concepts.join(", ");
                            let insts = rec.correlated_instrument_ids.join(", ");
                            let norms = rec.applicable_legal_and_human_rights_norms.join(" | ");
                            let eval = rec.symbolic_evaluation_summary.clone();
                            let parity = format!("{:#018x}", rec.sealed_nquin_parity);
                            rsx! {
                                li {
                                    key: "{id}",
                                    div {
                                        strong { "[{kind}] {title} " }
                                        span { "· Domain: {dom} · Correlated Instruments: [{insts}]" }
                                    }
                                    div { "CML Concepts: [{concepts}] · Applicable Law & Human Rights: [{norms}]" }
                                    div { class: "path", "{eval} · NQuin48 {parity}" }
                                }
                            }
                        }
                    }
                }
            }

            // 4. Strong-Authenticated Online Specialist Permissive-Disclosure Portal (webizen-hostd)
            section {
                class: "id-list",
                h4 { "4. Online Specialist Permissive-Disclosure Portal on webizen-hostd ({leases.len()} Specialist Leases · Six-Vector Audit)" }
                p {
                    class: "muted",
                    "Grant strong-authenticated, domain-scoped online access to specialists (Advocacy, Housing, Psychology, General Practice). Raw Sanctuary keys never leave your local apparatus; every specialist access is logged across the Six Vectors of Transparency (Who/When/Why/What/Where/Cost) and can be revoked/crypto-shredded in one click."
                }
                div {
                    class: "orb-sectors",
                    for dom in SpecialistDomainScope::ALL {
                        {
                            let active = spec_domain() == dom;
                            let title = dom.title();
                            rsx! {
                                button {
                                    key: "{title}",
                                    r#type: "button",
                                    class: if active { "ribbon-btn active" } else { "ribbon-btn" },
                                    onclick: move |_| spec_domain.set(dom),
                                    "Scope: {title}"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{spec_name}",
                        placeholder: "Specialist name / practice…",
                        oninput: move |evt| spec_name.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{spec_disc}",
                        placeholder: "Discipline (Advocacy / Housing / Psychology / GP)…",
                        oninput: move |evt| spec_disc.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{spec_did}",
                        placeholder: "Pairwise Specialist DID (did:qi:specialist:...)…",
                        oninput: move |evt| spec_did.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{spec_token}",
                        placeholder: "Strong authentication token (>=12 chars)…",
                        oninput: move |evt| spec_token.set(evt.value()),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                if let Ok(lease) = s.wellfair_portal.grant_specialist_disclosure_lease(
                                    &sess.host.storage,
                                    &spec_name(),
                                    &spec_disc(),
                                    &spec_did(),
                                    "",
                                    &spec_token(),
                                    vec![spec_domain()],
                                    1_700_000_000,
                                    1_900_000_000,
                                    true,
                                    true,
                                ) {
                                    portal_verify_lease_id.set(lease.lease_id);
                                    portal_verify_did.set(lease.specialist_pairwise_did);
                                }
                            }
                        },
                        "Grant Scoped Specialist Portal Lease"
                    }
                }
                div {
                    class: "plane-grid",
                    for lease in &leases {
                        {
                            let lid = lease.lease_id.clone();
                            let lid_revoke = lid.clone();
                            let lid_cosign = lid.clone();
                            let name = lease.specialist_display_name.clone();
                            let disc = lease.specialist_discipline.clone();
                            let did = lease.specialist_pairwise_did.clone();
                            let doms: Vec<&str> = lease.allowed_domains.iter().map(|d| d.code()).collect();
                            let doms_str = doms.join(", ");
                            let status = if lease.revoked_or_shredded {
                                "REVOKED & CRYPTO-SHREDDED"
                            } else if lease.mobile_pocket_cosigned {
                                "ACTIVE · POCKET CO-SIGNED"
                            } else {
                                "ACTIVE · AWAITING MOBILE POCKET CO-SIGN"
                            };
                            rsx! {
                                div {
                                    key: "{lid}",
                                    class: "plane-card",
                                    h5 { "[{status}] {name} ({lid})" }
                                    p { "Discipline: {disc} · Pairwise DID: {did}" }
                                    p { "Authorized Scopes: [{doms_str}]" }
                                    div {
                                        class: "orb-sectors",
                                        if !lease.revoked_or_shredded {
                                            button {
                                                r#type: "button",
                                                class: "ribbon-btn",
                                                onclick: move |_| {
                                                    if let Some(sess) = session() {
                                                        let mut s = state.write();
                                                        let _ = s.wellfair_portal.revoke_specialist_lease(
                                                            &sess.host.storage,
                                                            &lid_revoke,
                                                            "Revoked by principal from WellFair Portal console",
                                                        );
                                                    }
                                                },
                                                "Revoke & Crypto-Shred Lease"
                                            }
                                            button {
                                                r#type: "button",
                                                class: "sanctum-issue",
                                                onclick: move |_| {
                                                    if let Some(sess) = session() {
                                                        let mut s = state.write();
                                                        let ShellState {
                                                            ref mut mobile_avf,
                                                            ref mut wellfair_portal,
                                                            ..
                                                        } = *s;
                                                        let _ = execute_pocket_cosigner_ceremony(
                                                            mobile_avf,
                                                            wellfair_portal,
                                                            &sess.host.storage,
                                                            PocketCosignerActionKind::ApproveSpecialistDisclosureLease,
                                                            "did:qi:steward:mobile-avf-pocket",
                                                            &lid_cosign,
                                                            "pocket-cosign-2026",
                                                        );
                                                    }
                                                },
                                                "Pocket Co-Sign Lease (Mobile AVF)"
                                            }
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
                        value: "{portal_verify_lease_id}",
                        placeholder: "Lease ID (e.g. lease:specialist:1)…",
                        oninput: move |evt| portal_verify_lease_id.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{portal_verify_did}",
                        placeholder: "Specialist Pairwise DID…",
                        oninput: move |evt| portal_verify_did.set(evt.value()),
                    }
                    input {
                        class: "input-field",
                        value: "{portal_verify_token}",
                        placeholder: "Specialist strong access token…",
                        oninput: move |evt| portal_verify_token.set(evt.value()),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                match s.wellfair_portal.verify_and_derive_specialist_portal_bundle(
                                    &sess.host.storage,
                                    &portal_verify_lease_id(),
                                    &portal_verify_did(),
                                    &portal_verify_token(),
                                    "Online specialist portal consultation & casework verification",
                                    1_750_000_000,
                                ) {
                                    Ok(bundle) => {
                                        derived_preview.set(Some(format!(
                                            "Verified Lease '{}' for {}: derived {} scoped Semantic Instrument(s), {} CML Evidence exhibit(s), and {} mounted public .q42 ontology package(s). Six-Vector Audit ID: {}",
                                            bundle.lease_id,
                                            bundle.specialist_pairwise_did,
                                            bundle.visible_instruments.len(),
                                            bundle.visible_cml_evidence.len(),
                                            bundle.mounted_public_ontologies.len(),
                                            bundle.audit_receipt.audit_id
                                        )));
                                    }
                                    Err(err) => derived_preview.set(Some(err)),
                                }
                            }
                        },
                        "Test Strong-Auth Specialist Portal Session & Log Six-Vector Audit"
                    }
                }
                if let Some(preview) = derived_preview() {
                    p { class: "path", role: "status", "{preview}" }
                }
                if !audits.is_empty() {
                    h5 { "Six-Vector Transparency Audit Ledger ({audits.len()} Portal Access Events: Who · When · Why · What · Where · Cost)" }
                    ul {
                        for a in audits.iter().take(8) {
                            {
                                let aid = a.audit_id.clone();
                                let verdict = if a.access_permitted { "PERMITTED" } else { "DENIED" };
                                let who = a.who_specialist_did.clone();
                                let why = a.why_purpose.clone();
                                let what = a.what_resources_accessed.clone();
                                let where_loc = a.where_host_locus.clone();
                                let cost = a.cost_classification.clone();
                                let parity = format!("{:#018x}", a.sealed_nquin_parity);
                                rsx! {
                                    li {
                                        key: "{aid}",
                                        strong { "[{verdict}] {aid} " }
                                        span { "· Who: {who} · Why: \"{why}\" · What: {what} · Where: {where_loc} · Cost: {cost} · NQuin48 {parity}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 5. Mobile AVF v2 Pocket Co-Signer, Emergency Freeze & Field Evidence QECP Capture
            section {
                class: "id-list",
                h4 { "5. Mobile AVF v2 Pocket Co-Signer, Roaming Profile & Field Evidence QECP Capture ({mobile_conn})" }
                div {
                    class: "orb-sectors",
                    for prof in MobileConnectivityProfile::ALL {
                        {
                            let p_title = prof.title();
                            let active = state.read().mobile_avf.connectivity_profile == prof;
                            rsx! {
                                button {
                                    key: "{p_title}",
                                    r#type: "button",
                                    class: if active { "ribbon-btn active" } else { "ribbon-btn" },
                                    onclick: move |_| state.write().mobile_avf.connectivity_profile = prof,
                                    "{p_title}"
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
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let ShellState {
                                    ref mut mobile_avf,
                                    ref mut wellfair_portal,
                                    ..
                                } = *s;
                                let _ = capture_mobile_field_evidence_qecp(
                                    mobile_avf,
                                    wellfair_portal,
                                    &sess.host.storage,
                                    "Mobile AVF Field Evidence: Housing & Clinical Accommodation Snapshot",
                                    SpecialistDomainScope::HousingAndEnvironmentalSafety,
                                    "did:qi:steward:mobile-avf-pocket",
                                    "Hardware-attested mobile field note documenting housing habitability and ADHD/PTSD written accommodation requirements.",
                                );
                            }
                        },
                        "Capture Mobile AVF Field Evidence (Seal QECP + Correlate CML)"
                    }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let ShellState {
                                    ref mut mobile_avf,
                                    ref mut wellfair_portal,
                                    ..
                                } = *s;
                                let _ = execute_pocket_cosigner_ceremony(
                                    mobile_avf,
                                    wellfair_portal,
                                    &sess.host.storage,
                                    PocketCosignerActionKind::EmergencyFreezeAllOnlineLeases,
                                    "did:qi:steward:mobile-avf-pocket",
                                    "*",
                                    "emergency-freeze-all-2026",
                                );
                            }
                        },
                        "Emergency One-Tap Freeze of All Online Specialist Portal Leases (Mobile AVF)"
                    }
                }
                if !mobile_ceremonies.is_empty() || !mobile_evidence.is_empty() {
                    ul {
                        for c in mobile_ceremonies.iter().take(4) {
                            li {
                                key: "{c.ceremony_id}",
                                strong { "[{c.action.label()}] " }
                                span { "{c.status_summary} (Challenge SHA3: {c.challenge_digest_hex})" }
                            }
                        }
                        for ev in mobile_evidence.iter().take(4) {
                            li {
                                key: "{ev.record_id}",
                                strong { "[Mobile QECP Field Evidence] {ev.title} " }
                                span { "· QECP ID: {ev.qecp_envelope.credential_id} · Correlated CML ID: {ev.correlated_evidence_id}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fairness_draft_outcomes_cycle_with_separate_remedies() {
        let mut outcome = DimensionOutcome::Held;
        for expected in [
            DimensionOutcome::Satisfied,
            DimensionOutcome::Failed,
            DimensionOutcome::NotApplicable,
            DimensionOutcome::Held,
        ] {
            let (next, remedy) = next_review_state(outcome);
            assert_eq!(next, expected);
            assert!(!remedy.is_empty());
            outcome = next;
        }
    }
}
