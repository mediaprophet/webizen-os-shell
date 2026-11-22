// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::llm_agents::LlmAndAgentManagementPanel;
use crate::state::{
    assertion_evidence_lens, assertion_lens, instrument_readiness_lens, quin_window_lens,
    update_refract_quin_word, SettingsSection, ShellState, VolumeStage, WebizenRefractModel,
};
use aether::prelude::*;
use webizen_ua_os::{core_semantic_instruments_catalog, shacl_extension_families_catalog};

#[component]
pub fn InstrumentsVolume(mut state: Signal<ShellState>) -> Element {
    let instruments: Vec<String> = core_semantic_instruments_catalog()
        .iter()
        .map(|si| {
            format!(
                "{} ({}) [{}] — {}",
                si.id, si.evaluator, si.plane_scope, si.description
            )
        })
        .collect();
    let shacl_families: Vec<String> = shacl_extension_families_catalog()
        .iter()
        .map(|f| {
            format!(
                "{} · {} -> {} — {}",
                f.module, f.domain, f.opcode_target, f.coverage
            )
        })
        .collect();
    let llm_count = state.read().llm_models.len();

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Instruments · LLM (.gguf -> .p64) Management, Governed Agent Roster, External MCP Accounts & 6 Core Semantic Instruments" }
                p {
                    "Manage local .p64 AI models (converting and purging legacy .gguf files), configure per-principal Governed Software Agents callable across every OS-Shell volume, set External LLM Platform Accounts & Deontic MCP Egress Rules, and execute the 6 Core Semantic Instruments (SI-01 through SI-06) inside the 42 MiB Zero-Heap SlgArena Sentinel."
                }
            }
            LlmAndAgentManagementPanel { state }
            InstrumentReadinessOverview {}
            QuinSlotInspector {}
            div {
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.current_stage = VolumeStage::Settings;
                        s.settings_section = SettingsSection::Models;
                    },
                    "Open Settings Tab 3: AI Instruments ({llm_count} vault files)"
                }
            }
            section {
                class: "id-list",
                h4 { "The 6 Core Semantic Instruments (Collectable HMC Packages · SI-01 through SI-06)" }
                ul {
                    for line in instruments {
                        li { "{line}" }
                    }
                }
            }
            section {
                class: "id-list",
                h4 { "6 Declarative SHACL Extension Bridge Families (40+ Formal Logics -> Zero-Heap WebizenOpcodes)" }
                ul {
                    for fam in shacl_families {
                        li { "{fam}" }
                    }
                }
            }
        }
    }
}

const CORE_INSTRUMENT_IDS: [&str; 6] = ["SI-01", "SI-02", "SI-03", "SI-04", "SI-05", "SI-06"];
const QUIN_SLOT_COUNT: usize = 64;
const QUIN_FIELDS: [&str; 6] = [
    "Subject",
    "Predicate",
    "Object",
    "Context",
    "Metadata",
    "Parity",
];

#[component]
fn InstrumentReadinessOverview() -> Element {
    let Some(store) = aether::core::try_consume_context::<RefractStore<WebizenRefractModel>>()
    else {
        return aether::core::VNode::empty();
    };
    let readiness = store.get_cloned(instrument_readiness_lens());
    let assertion = store.get_cloned(assertion_lens());
    let evidence = store.get_cloned(assertion_evidence_lens());
    let catalog = core_semantic_instruments_catalog();
    let cards = catalog
        .iter()
        .zip(CORE_INSTRUMENT_IDS.into_iter().zip(readiness))
        .map(|(instrument, (instrument_id, instrument_status))| {
            rsx! {
                div {
                    key: "{instrument_id}",
                    p { "{instrument.evaluator}" }
                    {render_instrument_readiness_card(instrument_id, instrument_status)}
                }
            }
        });

    rsx! {
        section {
            class: "id-list",
            h4 { "Semantic instrument readiness" }
            div {
                class: "plane-grid",
                {cards}
            }
            {render_contextual_assertion_badge_with_evidence(&assertion, evidence)}
        }
    }
}

#[component]
fn QuinSlotInspector() -> Element {
    let mut active_slot = use_signal(|| 0usize);
    let mut slot_input = use_signal(|| String::from("0"));
    let mut active_field = use_signal(|| 0u8);
    let mut word_input = use_signal(String::new);
    let mut notice = use_signal(String::new);

    let Some(store) = aether::core::try_consume_context::<RefractStore<WebizenRefractModel>>()
    else {
        return aether::core::VNode::empty();
    };
    let slot_number = *active_slot.read();
    let slot = store.read_quin_slot(quin_window_lens(), slot_number);
    let words = [
        slot.subject,
        slot.predicate,
        slot.object,
        slot.context,
        slot.metadata,
        slot.parity,
    ];
    let field_name = QUIN_FIELDS[usize::from(*active_field.read())];
    let parity_status = if slot.verify_parity() {
        "Valid"
    } else {
        "Unverified"
    };
    let slot_value = slot_input.read().clone();
    let word_value = word_input.read().clone();
    let feedback = notice.read().clone();

    rsx! {
        section {
            class: "id-list",
            h4 { "NQuin slot inspector" }
            p { "Edits apply to the shell’s local projection." }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let number = slot_input.read().trim().parse::<usize>();
                    match number {
                        Ok(index) if index < QUIN_SLOT_COUNT => {
                            active_slot.set(index);
                            word_input.set(String::new());
                            notice.set(String::new());
                        }
                        _ => notice.set(String::from("Enter a slot number from 0 to 63.")),
                    }
                },
                label {
                    "Slot number (0–63)"
                    input {
                        r#type: "text",
                        value: "{slot_value}",
                        oninput: move |event| slot_input.set(event.value()),
                    }
                }
                button { r#type: "submit", "Inspect slot" }
            }
            table {
                caption { "Slot {slot_number}, parity {parity_status}" }
                tbody {
                    for (field, (label, word)) in QUIN_FIELDS.into_iter().zip(words).enumerate() {
                        tr {
                            key: "{label}",
                            th { scope: "row", "{label}" }
                            td { "{word:016x}" }
                            td {
                                if field < 5 {
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            active_field.set(field as u8);
                                            word_input.set(format!("0x{word:016x}"));
                                            notice.set(String::new());
                                        },
                                        "Edit {label}"
                                    }
                                } else {
                                    "Recomputed automatically"
                                }
                            }
                        }
                    }
                }
            }
            form {
                onsubmit: move |event| {
                    event.prevent_default();
                    let Some(value) = parse_quin_word(&word_input.read()) else {
                        notice.set(String::from("Enter a 64-bit unsigned decimal or hexadecimal word."));
                        return;
                    };
                    update_refract_quin_word(&store, *active_slot.read(), *active_field.read(), value);
                    notice.set(String::from("Projection updated and parity recomputed."));
                },
                label {
                    "{field_name} word"
                    input {
                        r#type: "text",
                        value: "{word_value}",
                        oninput: move |event| word_input.set(event.value()),
                    }
                }
                button { r#type: "submit", "Update projection" }
            }
            if !feedback.is_empty() {
                p { role: "status", "{feedback}" }
            }
        }
    }
}

fn parse_quin_word(text: &str) -> Option<u64> {
    let text = text.trim();
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).ok()
    } else {
        text.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::parse_quin_word;

    #[test]
    fn quin_words_accept_full_width_decimal_and_hex() {
        assert_eq!(parse_quin_word("18446744073709551615"), Some(u64::MAX));
        assert_eq!(parse_quin_word(" 0Xffffffffffffffff "), Some(u64::MAX));
        assert_eq!(parse_quin_word("18446744073709551616"), None);
        assert_eq!(parse_quin_word("0x"), None);
        assert_eq!(parse_quin_word("-1"), None);
    }
}
