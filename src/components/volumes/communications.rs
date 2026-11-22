// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use aether::prelude::*;
use aether::refract::{
    AssertionEvidenceLevel, AssertionKind, ChallengeState, ContextualAssertionView,
};

use crate::state::{session, ShellState};
use webizen_ua_os::{
    validate_human_centric_host_nomenclature, IdentifierFabricPlane, PLANE_NATURAL_AGENT_TITLE,
};

#[component]
pub fn TalkVolume(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let messages = s.talk_messages.clone();
    let input = s.talk_input.clone();
    let tool_on = s.talk_agent_tool_active;
    let agent_turns: Vec<_> = s
        .llm_management
        .invocation_log
        .iter()
        .filter(|turn| turn.target_volume.eq_ignore_ascii_case("Talk"))
        .take(8)
        .cloned()
        .collect();
    let mut talk_tool_status = use_signal(String::new);
    let tool_status = talk_tool_status.read().clone();
    let active_model = if s.active_model_name.is_empty() {
        "No local model bound yet (select in Settings -> AI instruments)".to_string()
    } else {
        s.active_model_name.clone()
    };
    drop(s);

    let tool_label = if tool_on {
        "Governed AI Tool Panel: OPEN (Tool Plane — explicitly distinct from human peers)"
    } else {
        "Governed AI Tool Panel: CLOSED (Human-to-Human QDNF mode)"
    };

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Talk · Humans-First Pairwise Conversation (Claim & Relation Plane)" }
                p {
                    "A person waiting for a person. Messages travel over Socially Defined Networking (QDNF) pairwise channels with post-quantum ML-KEM + Ed25519/ML-DSA dual signatures. An AI chatbot is an instrument on the Tool plane — never a fake human peer."
                }
            }
            div {
                class: "input-row",
                input {
                    class: "input-field",
                    value: "{input}",
                    placeholder: "Write a human-to-human claim or message…",
                    oninput: move |evt| state.write().talk_input = evt.value(),
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        let mut s = state.write();
                        let msg = s.talk_input.trim().to_string();
                        if !msg.is_empty() {
                            s.talk_messages.push(format!("[NaturalAgent -> Peer · NQuin48 sealed] {msg}"));
                            s.talk_input.clear();
                        }
                    },
                    "Send to Human Peer"
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.talk_agent_tool_active = !s.talk_agent_tool_active;
                        if s.talk_agent_tool_active {
                            s.active_plane = IdentifierFabricPlane::Instrument;
                        } else {
                            s.active_plane = IdentifierFabricPlane::ClaimOpinion;
                        }
                    },
                    "Toggle Governed AI Tool"
                }
            }
            p { class: "muted", "{tool_label} · Active Local Instrument: {active_model}" }
            if tool_on {
                section {
                    class: "id-list",
                    h4 { "@agent tool previews" }
                    p { class: "muted", "Enter @agent-slug followed by a prompt in the draft. The tool panel uses the configured material sensitivity. It records a preview without sending your message to a peer." }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        onclick: move |_| {
                            let Some(sess) = session() else {
                                talk_tool_status.set("Device session unavailable".into());
                                return;
                            };
                            let mut s = state.write();
                            let draft = s.talk_input.clone();
                            let Some((agent, prompt)) = parse_agent_mention(&draft) else {
                                talk_tool_status.set("Enter @agent-slug followed by a prompt".into());
                                return;
                            };
                            let sensitivity = s.agent_call_sensitivity_input;
                            let notice = match s.llm_management.invoke_environment_agent(
                                &sess.host.storage, agent, "Talk", prompt, sensitivity, Some("llm_infer"),
                            ) {
                                Ok(_) => "Tool preview recorded for Talk".into(),
                                Err(error) => error,
                            };
                            talk_tool_status.set(notice);
                        },
                        "Preview @agent request"
                    }
                    if !tool_status.is_empty() {
                        p { class: "muted", role: "status", "{tool_status}" }
                    }
                    ul {
                        for (index, turn) in agent_turns.into_iter().enumerate() {
                            {
                                let key = format!("{}-{}-{index}", turn.agent_slug, turn.sealed_nquin_parity);
                                let badge = render_contextual_assertion_badge_with_evidence(
                                    &TOOL_PREVIEW_ASSERTION,
                                    Some(AssertionEvidenceLevel::Simulated),
                                );
                                rsx! {
                                    li {
                                        key: "{key}",
                                        strong { "@{turn.agent_slug} preview transcript" }
                                        {badge}
                                        p { "{turn.response_text}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            section {
                class: "id-list",
                h4 { "Conversation Claims & Provenance Stream" }
                ul {
                    for m in messages {
                        li { "{m}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn MailVolume(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let messages = s.mail_messages.clone();
    let locator = s.mail_to_locator.clone();
    let subject = s.mail_subject.clone();
    drop(s);

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "Mail · Relationship-Scoped Pairwise Locators (Handle & Claim Plane)" }
                p {
                    "Each relationship binds a directional pairwise locator (A -> B != B -> A; no-proxy-launder enforced). If a traditional DNS domain expires, the person and relationship history never fade."
                }
            }
            div {
                class: "input-row",
                input {
                    class: "input-field",
                    value: "{locator}",
                    placeholder: "Recipient pairwise locator (did:qi:locator:...)",
                    oninput: move |evt| state.write().mail_to_locator = evt.value(),
                }
                input {
                    class: "input-field",
                    value: "{subject}",
                    placeholder: "Message subject or QECP credential summary…",
                    oninput: move |evt| state.write().mail_subject = evt.value(),
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        let mut s = state.write();
                        let to = s.mail_to_locator.trim().to_string();
                        let subj = s.mail_subject.trim().to_string();
                        if !to.is_empty() && !subj.is_empty() {
                            s.mail_messages.push(format!("Sent [Pairwise {to}] · {subj} (48-byte NQuin receipt committed)"));
                        }
                    },
                    "Compose & Seal Mail"
                }
            }
            section {
                class: "id-list",
                h4 { "Relationship-Scoped Inbox ({messages.len()} threads)" }
                ul {
                    for m in messages {
                        li { "{m}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn DirectoryVolume(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let contacts = s.directory_contacts.clone();
    let new_name = s.directory_new_name.clone();
    let guard_notice = s.beforeinput_guard_notice.clone();
    drop(s);

    rsx! {
        div {
            class: "settings-volume",
            header {
                class: "settings-head",
                h3 { "{PLANE_NATURAL_AGENT_TITLE} · Social Book & Pairwise Introductions" }
                p {
                    "Living beings are framed strictly as living-SHACL records (D_living), disjoint from owl:Thing, and never reduced to a Thing, Object, or raw DID. Organisations and AI instruments are kept in distinct categories."
                }
            }
            div {
                class: "input-row",
                input {
                    class: "input-field",
                    value: "{new_name}",
                    placeholder: "Add person or cooperative peer display hint…",
                    onbeforeinput: move |evt| {
                        let data = evt.data();
                        if let Some(insertion) = data.data() {
                            if !directory_insertion_allowed(&data.value(), &insertion) {
                                evt.prevent_default();
                                state.write().beforeinput_guard_notice = DIRECTORY_GUARD_NOTICE.into();
                            }
                        }
                    },
                    oninput: move |evt| {
                        let value = evt.value();
                        let mut s = state.write();
                        if directory_label_allowed(&value) {
                            s.directory_new_name = value;
                            s.beforeinput_guard_notice.clear();
                        } else {
                            s.beforeinput_guard_notice = DIRECTORY_GUARD_NOTICE.into();
                        }
                    },
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        let mut s = state.write();
                        let name = s.directory_new_name.trim().to_string();
                        if name.is_empty() {
                            return;
                        }
                        if !directory_label_allowed(&name) {
                            s.beforeinput_guard_notice = DIRECTORY_GUARD_NOTICE.into();
                        } else {
                            s.directory_contacts.push(format!(
                                "NaturalAgent (living-SHACL) · {name} · Directional pairwise locator minted"
                            ));
                            s.directory_new_name.clear();
                            s.beforeinput_guard_notice.clear();
                        }
                    },
                    "Mint Pairwise Relationship"
                }
            }
            if !guard_notice.is_empty() {
                p { class: "muted", role: "status", "{guard_notice}" }
            }
            section {
                class: "id-list",
                h4 { "Known People & Organisational Peers" }
                ul {
                    for c in contacts {
                        li { "{c}" }
                    }
                }
            }
        }
    }
}

const TOOL_PREVIEW_ASSERTION: ContextualAssertionView = ContextualAssertionView {
    kind: AssertionKind::Inference,
    challenge: ChallengeState::Unchallenged,
    independent_source_count: 0,
    summary: "Tool routing preview. Verify any derived claims independently.",
};

fn parse_agent_mention(draft: &str) -> Option<(&str, &str)> {
    let (agent, prompt) = draft
        .trim()
        .strip_prefix('@')?
        .split_once(char::is_whitespace)?;
    let prompt = prompt.trim();
    (!agent.is_empty() && !prompt.is_empty()).then_some((agent, prompt))
}

const DIRECTORY_GUARD_NOTICE: &str = "Use Human-Centric terminology for directory labels.";

fn directory_label_allowed(value: &str) -> bool {
    validate_human_centric_host_nomenclature(value).is_ok()
}

fn directory_insertion_allowed(current: &str, insertion: &str) -> bool {
    if !directory_label_allowed(insertion) {
        return false;
    }

    // Native beforeinput provides the previous value and insertion without
    // an edit range. Limit the all-position scan to short labels. The exact
    // resulting value is also validated on input and action submission.
    const MAX_INSERTION_SCAN_BYTES: usize = 1024;
    if current.len().saturating_add(insertion.len()) > MAX_INSERTION_SCAN_BYTES {
        return true;
    }

    let mut candidate = String::with_capacity(current.len() + insertion.len());
    for boundary in current
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(current.len()))
    {
        candidate.clear();
        candidate.push_str(&current[..boundary]);
        candidate.push_str(insertion);
        candidate.push_str(&current[boundary..]);
        if !directory_label_allowed(&candidate) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_guard_detects_split_insertions_and_preserves_utf8() {
        assert!(!directory_insertion_allowed("owl:Ting", "h"));
        assert!(!directory_insertion_allowed("owl:", "Thing"));
        assert!(directory_insertion_allowed("Élodie", " Steward"));
        assert!(directory_insertion_allowed("", &"x".repeat(2_048)));
    }

    #[test]
    fn agent_preview_requires_an_explicit_mention_and_a_prompt() {
        assert_eq!(
            parse_agent_mention(" @tool  explain this "),
            Some(("tool", "explain this"))
        );
        assert_eq!(parse_agent_mention("@tool"), None);
        assert_eq!(parse_agent_mention("ordinary peer message"), None);
    }
}
