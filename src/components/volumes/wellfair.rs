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
    trigger_one_tap_principal_ok_checkin, CareAlertSeverity, CareAlertTriggerKind,
    SharedLivingAndCareTopologyKind,
};

#[component]
pub fn WellfairVolume(state: Signal<ShellState>) -> Element {
    rsx! {
        FairnessDraftPanel {}
        GuardianshipAndCarePanel { state }
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
