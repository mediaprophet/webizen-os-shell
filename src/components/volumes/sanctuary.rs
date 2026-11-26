// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::theme_studio::EnvironmentQuestionnairePanel;
use crate::state::{session, ShellState};
use aether::prelude::*;
use webizen_ua_os::{
    build_seeded_civics_au_human_rights_observatory,
    build_seeded_machine_onboarding_and_account_assignments, issue_for_requestor,
    mint_zero_identity_verifiable_truthfulness_proof, pair_and_test_pocket_cosigner,
    simulate_shamir_recovery_ceremony, Ask, AustralianCivicsJurisdiction,
    HumanRightsCivicsIssueDomain, NewUserOnboardingScreenStep, Requestor,
};

#[component]
pub fn SanctuaryVolume(mut state: Signal<ShellState>) -> Element {
    let mut selected_shares = use_signal(Vec::<u8>::new);
    let mut recovery_error = use_signal(String::new);
    let mut mobile_did = use_signal(String::new);
    let mut wireguard_endpoint = use_signal(String::new);
    let mut pairing_notice = use_signal(String::new);
    let mut active_onboarding_step =
        use_signal(|| NewUserOnboardingScreenStep::Screen1HardwareAnvilAndWsl2Probe);
    let mut active_seat_index = use_signal(|| 0usize);
    let mut civics_summary_draft = use_signal(|| {
        String::from(
            "De-identified constituent lived-experience report: 140-day delay on disability accommodation review despite specialist occupational therapy attestation.",
        )
    });
    let mut civics_proof_notice = use_signal(String::new);

    let snapshot = state.read();
    let attested = snapshot.may_advertise_attested_vault;
    let issued = snapshot.sanctum_issued.clone();
    let onboarding = snapshot.sanctuary_onboarding.clone();
    drop(snapshot);

    let selected = selected_shares.read().clone();
    let recovery_error_text = recovery_error.read().clone();
    let pairing_notice_text = pairing_notice.read().clone();
    let did_draft = mobile_did.read().clone();
    let endpoint_draft = wireguard_endpoint.read().clone();
    let threshold = onboarding.shamir_threshold_m;
    let total = onboarding.shamir_total_n;
    let picked = selected.len();
    let recovered = onboarding.last_reconstruction_test;
    let vault_slots = onboarding.vault_slots;
    let share_holders = onboarding.shamir_shares;
    let pocket = onboarding.pocket_cosigner;
    let attestation_status = pocket.pkvm_attestation_status;
    let vsock_cid = pocket.vsock_cid;
    let paired = pocket.paired;
    let has_birth_binding = !onboarding.digital_birth_did.is_empty();
    let current_step = *active_onboarding_step.read();
    let current_seat_idx = *active_seat_index.read();
    let machine_registry = build_seeded_machine_onboarding_and_account_assignments(
        "mercer-homestead-workstation",
        "did:q42:anvil:10de-2684-99a1e4c2b7d01122",
    );
    let selected_seat = machine_registry
        .assigned_seats
        .get(current_seat_idx)
        .unwrap_or(&machine_registry.assigned_seats[0])
        .clone();

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Keep" }
                p { "Private files, unlock status, and a guided recovery wizard. Looks and access comfort live here too." }
            }
            EnvironmentQuestionnairePanel { state }

            section {
                class: "id-list",
                aria_label: "7-Screen New-User Onboarding Wizard",
                h4 { "7-Screen New-User Onboarding Wizard (Desktop & Mobile Progressive Confidence Ladder)" }
                p {
                    "Guides a new user from initial hardware probe (Screen 1) through Four-Plane anchor, Dual-PIN Sanctuary Keep, Mobile AVF Pocket Co-Signer pairing, Multi-Account shared-machine seat assignment, and Always-On Persistence Mesh to Tier 3 High-Assurance Confidence (< 1.0)."
                }
                div {
                    class: "sanctuary-vault-grid",
                    for step in NewUserOnboardingScreenStep::ALL {
                        {
                            let num = step.step_number();
                            let title = step.title();
                            let is_active = step == current_step;
                            rsx! {
                                button {
                                    key: "{num}",
                                    r#type: "button",
                                    class: "sanctum-issue",
                                    onclick: move |_| active_onboarding_step.set(step),
                                    if is_active {
                                        "▶ {title}"
                                    } else {
                                        "{title}"
                                    }
                                }
                            }
                        }
                    }
                }
                article {
                    class: "shamir-share-card",
                    h5 { "{current_step.title()}" }
                    p { "{current_step.objective_summary()}" }
                    p { class: "path", "Identifier Developed: {current_step.identifier_developed_on_screen()}" }
                }
            }

            section {
                class: "id-list",
                aria_label: "Multi-Account Shared-Machine Seat Assignments",
                h4 { "Multi-Account Shared-Machine Seat Assignments (OS Account != Principal Identity)" }
                p {
                    "Machine '{machine_registry.machine_hostname}' ({machine_registry.machine_anvil_np64_did}) hosts 4 isolated account seats across Personal, Fiduciary Clinical, Child Student, and Delegated Agent roles."
                }
                div {
                    class: "sanctuary-vault-grid",
                    for (idx, seat) in machine_registry.assigned_seats.iter().enumerate() {
                        {
                            let seat_id = seat.seat_id.clone();
                            let display_name = seat.display_name.clone();
                            let os_user = seat.os_account_username.clone();
                            let profile_slug = seat.webizen_profile_slug.clone();
                            let port = seat.dedicated_loopback_port;
                            let conf = format!("{:.3}", seat.noisy_or_aggregate_confidence);
                            let tier_label = seat.confidence_tier.label();
                            let id_count = seat.identifier_bindings.len();
                            let is_picked = idx == current_seat_idx;
                            rsx! {
                                article {
                                    key: "{seat_id}",
                                    class: "shamir-share-card",
                                    h5 {
                                        if is_picked { "● {display_name}" } else { "{display_name}" }
                                    }
                                    p { "OS Login: '{os_user}' · Webizen Profile: '{profile_slug}' · Loopback Port: :{port} · 42 MiB SlgArena" }
                                    p { class: "path", "{id_count} Attested Identifiers · Noisy-OR Confidence: {conf} (< 1.0) · {tier_label}" }
                                    button {
                                        r#type: "button",
                                        class: "sanctum-issue",
                                        onclick: move |_| active_seat_index.set(idx),
                                        "Inspect Seat Identifiers ({id_count})"
                                    }
                                }
                            }
                        }
                    }
                }
                article {
                    class: "shamir-share-card",
                    h5 { "Active Seat Identifier Bindings · {selected_seat.display_name}" }
                    p { "Isolated Vault Path: {selected_seat.isolated_vault_rel_path}" }
                    for b in selected_seat.identifier_bindings {
                        p {
                            class: "muted",
                            "• [{b.plane}] {b.role} -> {b.uri} ({b.scheme.as_str()}, c={b.confidence:.2})"
                        }
                    }
                }
            }

            section {
                class: "id-list",
                aria_label: "VaultContainerV2 inspector",
                h4 { "VaultContainerV2 · Four-slot policy inspector" }
                p {
                    "Structural policy metadata for the CBOR vault. The inspector keeps record counts private."
                }
                p {
                    class: "muted",
                    if attested {
                        "Local Anvil NP64 gate passed (IOMMU, GPU UUID and attestation tier ≤ 1)."
                    } else {
                        "The local Anvil NP64 hardware attestation gate is closed."
                    }
                }
                div {
                    class: "sanctuary-vault-grid",
                    for slot in vault_slots {
                        {
                            let index = slot.slot_index;
                            let label = slot.label;
                            let profile = slot.encryption_profile;
                            let posture = slot.coercion_posture;
                            let sensitivity = slot.max_sensitivity;
                            let parity = format!("{:#018x}", slot.sealed_nquin_parity);
                            rsx! {
                                article {
                                    key: "{index}",
                                    class: "shamir-share-card",
                                    h5 { "Slot {index} · {label}" }
                                    p { "{profile}" }
                                    p { "{posture}" }
                                    p { class: "muted", "Sensitivity ceiling {sensitivity} · NQuin48 parity {parity}" }
                                }
                            }
                        }
                    }
                }
            }

            section {
                class: "id-list",
                aria_label: "Contextual DID Guard",
                h4 { "Contextual Pairwise DID Guard" }
                p {
                    if has_birth_binding {
                        "The Digital Birth Record binding stays private."
                    } else {
                        "The current session has no Digital Birth Record binding."
                    }
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    disabled: session().is_none(),
                    onclick: move |_| {
                        if let Some(sess) = session() {
                            let reply = issue_for_requestor(
                                &sess.identifiers,
                                &Requestor {
                                    ask: Ask::Master,
                                    requestor: "keep-sanctuary-ui".into(),
                                    context: "volume-keep".into(),
                                },
                            );
                            state.write().sanctum_issued = format!(
                                "{} · master refused={} · context={} · NQuin48 parity={:#018x}",
                                reply.issued_did, reply.refused_master, reply.bound_context, reply.quin_parity
                            );
                        }
                    },
                    "Test contextual handle issuance"
                }
                if !issued.is_empty() {
                    p { class: "path", role: "status", "{issued}" }
                }
            }

            section {
                class: "id-list",
                aria_label: "Shamir social guardianship recovery",
                h4 { "{threshold}-of-{total} Shamir recovery rehearsal" }
                p {
                    "Select fixture shares to verify threshold reconstruction. The rehearsal uses public test material held on this device."
                }
                form {
                    onsubmit: move |event| {
                        event.prevent_default();
                        let shares = selected_shares.read().clone();
                        let result = simulate_shamir_recovery_ceremony(
                            &mut state.write().sanctuary_onboarding,
                            &shares,
                        );
                        recovery_error.set(result.err().unwrap_or_default());
                    },
                    div {
                        class: "sanctuary-vault-grid",
                        for holder in share_holders {
                            {
                                let index = holder.share_index;
                                let checked = selected.contains(&index);
                                let available = holder.verified_present;
                                let label = holder.holder_label;
                                let commitment = holder.share_commitment_hex;
                                rsx! {
                                    label {
                                        key: "{index}",
                                        class: "shamir-share-card",
                                        span {
                                            input {
                                                r#type: "checkbox",
                                                checked,
                                                disabled: !available,
                                                onchange: move |event| {
                                                    let mut indices = selected_shares.write();
                                                    if event.checked() {
                                                        if !indices.contains(&index) {
                                                            indices.push(index);
                                                        }
                                                    } else {
                                                        indices.retain(|item| *item != index);
                                                    }
                                                },
                                            }
                                            " {label}"
                                        }
                                        code { "Share {index} · Commitment {commitment}" }
                                    }
                                }
                            }
                        }
                    }
                    p { "{picked} of {threshold} required shares selected" }
                    button {
                        r#type: "submit",
                        class: "sanctum-issue",
                        disabled: picked < usize::from(threshold),
                        "Verify M-of-N threshold reconstruction"
                    }
                }
                if !recovery_error_text.is_empty() {
                    p { class: "path", role: "alert", "{recovery_error_text}" }
                }
                if !recovered.is_empty() {
                    p { class: "path", role: "status", "{recovered}" }
                }
            }

            section {
                class: "id-list",
                aria_label: "Mobile AVF Pocket Co-Signer",
                h4 { "Mobile AVF Pocket Co-Signer · arm64" }
                p { "{attestation_status} · Planned vsock CID {vsock_cid}" }
                if paired {
                    p { role: "status", "Pocket Co-Signer paired." }
                } else {
                    p { "Prepare and check a local handshake transcript. Pairing requires a signed challenge from the Pocket Co-Signer." }
                    form {
                        onsubmit: move |event| {
                            event.prevent_default();
                            let did = mobile_did.read().clone();
                            let endpoint = wireguard_endpoint.read().clone();
                            let result = pair_and_test_pocket_cosigner(
                                &mut state.write().sanctuary_onboarding,
                                &did,
                                &endpoint,
                            );
                            pairing_notice.set(match result {
                                Ok(receipt) => receipt,
                                Err(message) => message,
                            });
                        },
                        label {
                            "Pocket Co-Signer DID"
                            input {
                                r#type: "text",
                                value: "{did_draft}",
                                maxlength: 256,
                                required: true,
                                oninput: move |event| mobile_did.set(event.value()),
                            }
                        }
                        label {
                            "WireGuard endpoint"
                            input {
                                r#type: "text",
                                placeholder: "pocket.local:51820",
                                value: "{endpoint_draft}",
                                maxlength: 260,
                                required: true,
                                oninput: move |event| wireguard_endpoint.set(event.value()),
                            }
                        }
                        button {
                            r#type: "submit",
                            class: "sanctum-issue",
                            "Check pairing transcript"
                        }
                    }
                }
                if !pairing_notice_text.is_empty() {
                    p { class: "path", role: "status", "{pairing_notice_text}" }
                }
            }

            {
                let civics_obs = build_seeded_civics_au_human_rights_observatory();
                let draft_text = civics_summary_draft.read().clone();
                let proof_text = civics_proof_notice.read().clone();
                rsx! {
                    section {
                        class: "id-list",
                        aria_label: "civics.au Australian Human-Rights & Civics Statistical Observatory",
                        h4 { "civics.au · Australian Human-Rights Statistical Observatory & Parliamentary Constituent Engagement" }
                        p {
                            "Hosts k-anonymous (>= 5) human-rights statistics across Australian jurisdictions (AU-CTH, AU-VIC, AU-NSW, AU-QLD, AU-ACT, AU-WA, AU-SA, AU-TAS, AU-NT), accepts Zero-Identifier Verifiable Testimony backed by Evidence Act (.qar / .q42) primary artifacts, and issues blind return receipts for Parliamentary (Civic) responses."
                        }
                        div {
                            class: "sanctuary-vault-grid",
                            for s in civics_obs.statistical_series {
                                {
                                    let sid = s.series_id.clone();
                                    let title = s.issue_domain.title();
                                    let jur = s.jurisdiction.iso_code();
                                    let count = s.verified_unique_constituent_count;
                                    let sev = s.published_mean_severity_score.unwrap_or(0);
                                    let delay = s.published_median_unresolved_days.unwrap_or(0);
                                    let backed = s.corroborated_evidence_act_backing_pct.unwrap_or(0);
                                    let remedy = s.top_constituent_remedy_consensus.clone();
                                    rsx! {
                                        article {
                                            key: "{sid}",
                                            class: "shamir-share-card",
                                            h5 { "[{sid}] {title} ({jur})" }
                                            p { "Verified Unique Constituents: {count} (k-Anonymity >= 5) · Mean Severity: {sev}/100 · Median Delay: {delay} days · Evidence Act Backed: {backed}%" }
                                            p { class: "muted", "Consensus Remedy: {remedy}" }
                                        }
                                    }
                                }
                            }
                        }
                        div {
                            class: "sanctuary-vault-grid",
                            for p in civics_obs.parliamentary_engagements {
                                {
                                    let eid = p.engagement_id.clone();
                                    let mp = p.parliamentarian_name_and_title.clone();
                                    let cred = p.parliamentary_register_credential_id.clone();
                                    let resp = p.parliamentary_response_and_action_commitment.clone();
                                    let hansard = p.hansard_or_committee_reference.clone();
                                    let tokens = p.constituent_nullifier_blind_reply_tokens.join(", ");
                                    rsx! {
                                        article {
                                            key: "{eid}",
                                            class: "shamir-share-card",
                                            h5 { "Parliamentary Engagement · {mp} ({cred})" }
                                            p { "{resp}" }
                                            p { class: "path", "{hansard} · Blind Nullifier Receipts: [{tokens}]" }
                                        }
                                    }
                                }
                            }
                        }
                        article {
                            class: "shamir-share-card",
                            h5 { "Mint Zero-Identifier (Zero-Who-Leak) Verifiable Testimony Proof for civics.au" }
                            label {
                                "De-identified lived-experience summary (Zero-Identifier Privacy Gate rejects any personal DID or email)"
                                input {
                                    r#type: "text",
                                    value: "{draft_text}",
                                    oninput: move |e| civics_summary_draft.set(e.value()),
                                }
                            }
                            button {
                                r#type: "button",
                                class: "sanctum-issue",
                                onclick: move |_| {
                                    let res = mint_zero_identity_verifiable_truthfulness_proof(
                                        "local_sanctuary_lane_seed_alex",
                                        HumanRightsCivicsIssueDomain::SocialSecurityNdisAndAutomatedWelfareFairness.code(),
                                        "2026-11",
                                        AustralianCivicsJurisdiction::Victoria,
                                        "Melbourne",
                                        997,
                                        &[
                                            "qar://evidence-act-s69/agency-letter.qar",
                                            "qar://evidence-act-s146/clinical-report.qar",
                                        ],
                                        &civics_summary_draft.read(),
                                    );
                                    civics_proof_notice.set(match res {
                                        Ok(p) => format!(
                                            "Minted Zero-ID Truthfulness Proof · Nullifier: {} · Electorate ZK: {} · Evidence Act Anchor ({} artifacts): {} · Statutory Undertaking: {}",
                                            p.epoch_issue_nullifier_hex,
                                            p.electorate_and_jurisdiction_membership_zk_commitment_hex,
                                            p.corroborated_primary_artifact_count,
                                            p.corroborating_evidence_act_anchor_commitment_hex,
                                            p.statutory_truthfulness_undertaking_hash_hex
                                        ),
                                        Err(e) => format!("Rejected by Gate: {e}"),
                                    });
                                },
                                "Mint Zero-Identifier Verifiable Testimony Proof"
                            }
                            if !proof_text.is_empty() {
                                p { class: "path", role: "status", "{proof_text}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
