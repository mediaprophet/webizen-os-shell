// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use aether::prelude::*;
use aether::refract::{
    AssertionEvidenceLevel, AssertionKind, ChallengeState, ContextualAssertionView,
};

use crate::state::{session, ShellState, VolumeStage};
use webizen_ua_os::{validate_human_centric_host_nomenclature, IdentifierFabricPlane};

#[component]
pub fn TalkVolume(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let messages = s.talk_messages.clone();
    let input = s.talk_input.clone();
    let tool_on = s.talk_agent_tool_active;
    let contacts = conversation_people(&s.directory_contacts, &s.talk_active_contact);
    let active_contact = if s.talk_active_contact.trim().is_empty() {
        contacts.first().cloned().unwrap_or_else(|| "Elena Vance".into())
    } else {
        s.talk_active_contact.clone()
    };
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
        "No local model bound yet (select in Settings -> AI helpers)".to_string()
    } else {
        s.active_model_name.clone()
    };
    drop(s);

    let tool_label = if tool_on {
        "Local helper tool active (marked separately from people)"
    } else {
        "Direct person-to-person chat (no AI listening)"
    };
    let visible_messages: Vec<(bool, String)> = messages
        .iter()
        .filter(|line| message_belongs_to_contact(line, &active_contact))
        .map(|line| {
            let outgoing = line.starts_with("You:") || line.starts_with("You ·");
            (outgoing, line.clone())
        })
        .collect();

    rsx! {
        div {
            class: "comms-workspace",
            aside {
                class: "comms-pane-list",
                h3 { "Talk" }
                p { class: "muted", "People you are talking with" }
                ul {
                    for person in contacts {
                        {
                            let name = person.clone();
                            let is_active = name == active_contact;
                            rsx! {
                                li {
                                    key: "{person}",
                                    button {
                                        r#type: "button",
                                        class: if is_active { "ribbon-btn active" } else { "ribbon-btn" },
                                        onclick: move |_| {
                                            state.write().talk_active_contact = name.clone();
                                        },
                                        "{person}"
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.talk_agent_tool_active = !s.talk_agent_tool_active;
                        if s.talk_agent_tool_active {
                            s.active_plane = IdentifierFabricPlane::Instrument;
                        } else {
                            s.active_plane = IdentifierFabricPlane::ClaimOpinion;
                        }
                    },
                    if tool_on { "Hide helper tools" } else { "Show helper tools" }
                }
                p { class: "muted", "{tool_label}" }
                p { class: "muted", "Helper: {active_model}" }
            }
            section {
                class: "comms-pane-thread",
                h3 { "Conversation with {active_contact}" }
                ul {
                    class: "comms-messages",
                    for (outgoing, line) in visible_messages {
                        li {
                            class: if outgoing { "chat-bubble outgoing" } else { "chat-bubble incoming" },
                            "{line}"
                        }
                    }
                }
                div {
                    class: "input-row comms-compose",
                    input {
                        class: "input-field",
                        value: "{input}",
                        placeholder: "Write a message…",
                        oninput: move |evt| state.write().talk_input = evt.value(),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            let mut s = state.write();
                            let msg = s.talk_input.trim().to_string();
                            if !msg.is_empty() {
                                let peer = if s.talk_active_contact.trim().is_empty() {
                                    "Elena Vance".into()
                                } else {
                                    s.talk_active_contact.clone()
                                };
                                s.talk_messages.push(format!("You: {msg}"));
                                s.talk_messages.push(format!("{peer}: Received — I'll reply from this thread."));
                                s.talk_input.clear();
                            }
                        },
                        "Send"
                    }
                }
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
    let body = s.mail_body.clone();
    let selected = s.mail_selected.min(messages.len().saturating_sub(1));
    drop(s);
    let selected_letter = messages.get(selected).cloned().unwrap_or_default();

    rsx! {
        div {
            class: "comms-workspace",
            aside {
                class: "comms-pane-list",
                h3 { "Mail" }
                p { class: "muted", "Inbox" }
                ul {
                    if messages.is_empty() {
                        li { class: "muted", "No letters yet" }
                    }
                    for (index, m) in messages.iter().enumerate() {
                        {
                            let preview = m.clone();
                            let is_sel = index == selected;
                            rsx! {
                                li {
                                    key: "{index}",
                                    button {
                                        r#type: "button",
                                        class: if is_sel { "ribbon-btn active" } else { "ribbon-btn" },
                                        onclick: move |_| state.write().mail_selected = index,
                                        "{preview}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            section {
                class: "comms-pane-thread",
                h3 { "Read and write" }
                if !selected_letter.is_empty() {
                    article {
                        class: "chat-bubble incoming",
                        strong { "Selected letter" }
                        p { "{selected_letter}" }
                    }
                }
                h4 { "Write a Letter / Reply" }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{locator}",
                        placeholder: "To (person or address)…",
                        oninput: move |evt| state.write().mail_to_locator = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{subject}",
                        placeholder: "Subject…",
                        oninput: move |evt| state.write().mail_subject = evt.value(),
                    }
                }
                textarea {
                    class: "input-field",
                    rows: "5",
                    value: "{body}",
                    placeholder: "Message…",
                    oninput: move |evt| state.write().mail_body = evt.value(),
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        let mut s = state.write();
                        let to = s.mail_to_locator.trim().to_string();
                        let subj = s.mail_subject.trim().to_string();
                        let note = s.mail_body.trim().to_string();
                        if !to.is_empty() && !subj.is_empty() {
                            let row = if note.is_empty() {
                                format!("To {to} · {subj}")
                            } else {
                                format!("To {to} · {subj} · {note}")
                            };
                            s.mail_messages.push(row);
                            s.mail_selected = s.mail_messages.len().saturating_sub(1);
                            s.mail_body.clear();
                        }
                    },
                    "Send"
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
                h3 { "Directory" }
                p {
                    "People and introductions. Organisations and helper tools stay in their own lists."
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
                                "{name} · Person & Peer · Private connection ready"
                            ));
                            s.directory_new_name.clear();
                            s.beforeinput_guard_notice.clear();
                        }
                    },
                    "Mint Pairwise Relationship"
                }
            }
            p { class: "muted", "Each relationship gets its own private connection address so contacts cannot be tracked across apps." }
            if !guard_notice.is_empty() {
                p { class: "muted", role: "status", "{guard_notice}" }
            }
            section {
                class: "id-list",
                h4 { "People, family & cooperative contacts" }
                div {
                    class: "contact-card-grid",
                    for c in contacts {
                        {
                            let raw = c.clone();
                            let name = contact_display_name(&raw);
                            let role = contact_role(&raw);
                            let chat_name = name.clone();
                            let mail_name = name.clone();
                            rsx! {
                                article {
                                    key: "{raw}",
                                    class: "contact-card",
                                    h5 { "{name}" }
                                    p { class: "muted", "{role}" }
                                    div {
                                        class: "orb-sectors",
                                        button {
                                            r#type: "button",
                                            class: "sanctum-issue",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                s.talk_active_contact = chat_name.clone();
                                                s.current_stage = VolumeStage::Talk;
                                            },
                                            "Chat in Talk"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            onclick: move |_| {
                                                let mut s = state.write();
                                                s.mail_to_locator = mail_name.clone();
                                                s.mail_subject = format!("Hello {mail_name}");
                                                s.current_stage = VolumeStage::Mail;
                                            },
                                            "Write in Mail"
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

fn contact_display_name(raw: &str) -> String {
    raw.split(" · ").next().unwrap_or(raw).trim().to_string()
}

fn contact_role(raw: &str) -> String {
    raw.split(" · ")
        .nth(1)
        .unwrap_or("Person & Peer")
        .trim()
        .to_string()
}

fn conversation_people(directory: &[String], active: &str) -> Vec<String> {
    let mut people: Vec<String> = directory
        .iter()
        .map(|row| contact_display_name(row))
        .filter(|name| !name.is_empty())
        .collect();
    if people.is_empty() {
        people.extend([
            "Elena Vance".into(),
            "Jordan Hale".into(),
            "Maya Chen".into(),
            "Nest".into(),
        ]);
    }
    if !active.trim().is_empty() && !people.iter().any(|name| name == active) {
        people.insert(0, active.trim().to_string());
    }
    people
}

fn message_belongs_to_contact(line: &str, contact: &str) -> bool {
    let first = contact.split_whitespace().next().unwrap_or(contact);
    line.starts_with("You:")
        || line.starts_with("You ·")
        || line.contains(contact)
        || (!first.is_empty() && line.starts_with(first))
}

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
