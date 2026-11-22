// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use crate::state::{session, ShellState};
use aether::prelude::*;
use webizen_ua_os::{
    issue_for_requestor, pair_and_test_pocket_cosigner, simulate_shamir_recovery_ceremony, Ask,
    Requestor,
};

#[component]
pub fn SanctuaryVolume(mut state: Signal<ShellState>) -> Element {
    let mut selected_shares = use_signal(Vec::<u8>::new);
    let mut recovery_error = use_signal(String::new);
    let mut mobile_did = use_signal(String::new);
    let mut wireguard_endpoint = use_signal(String::new);
    let mut pairing_notice = use_signal(String::new);

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

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Keep · Sanctuary" }
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
                        let result = simulate_shamir_recovery_ceremony(
                            &mut state.write().sanctuary_onboarding,
                            &selected_shares.read(),
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
                            let result = pair_and_test_pocket_cosigner(
                                &mut state.write().sanctuary_onboarding,
                                &mobile_did.read(),
                                &wireguard_endpoint.read(),
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
        }
    }
}
