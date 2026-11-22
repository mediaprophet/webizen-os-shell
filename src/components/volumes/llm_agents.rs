// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use crate::state::{session, ShellState};
use aether::prelude::*;
use webizen_ua_os::{
    ExternalLlmProviderKind, ExternalRemoteConsentRule, ManagedModelResidency, ManagedModelTask,
    McpTransportConfig, SENSITIVITY_COMMONS_GROUP, SENSITIVITY_PUBLIC_ONLY,
    SENSITIVITY_SANCTUARY_LOCAL_MAX,
};

/// Reusable LLM conversion, agent roster, and external account and transport controls.
#[component]
pub fn LlmAndAgentManagementPanel(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let reg = s.llm_management.clone();
    let sel_agent = s.selected_agent_slug.clone();
    let call_prompt = s.agent_call_prompt_input.clone();
    let call_volume = s.agent_call_volume_input.clone();
    let call_sens = s.agent_call_sensitivity_input;
    let ext_label = s.new_ext_account_label_input.clone();
    let ext_model = s.new_ext_model_id_input.clone();
    let ext_url = s.new_ext_mcp_url_input.clone();
    let ext_kv = s.new_ext_keyvault_ref_input.clone();
    let toast = s.llm_management_toast.clone();
    drop(s);

    let active_p64 = reg.active_p64_model_id.clone();
    let vram_mib = reg.active_vram_used_mib;
    let kv_mib = reg.active_kv_cache_mib;
    let floor_mib = reg.memory_floor_mib;
    let tok_s = (reg.last_measured_decode_tok_s_milli as f64) / 1000.0;
    let thermal = reg.thermal_governor_state.clone();
    let total_reclaimed_mib: u64 = reg
        .managed_models
        .iter()
        .map(|m| m.reclaimed_gguf_mib)
        .sum();
    let pending_gguf_count = reg
        .managed_models
        .iter()
        .filter(|m| !m.is_p64_native || !m.source_gguf_removed)
        .count();

    let sens_label = match call_sens {
        0 => "0 · Public Only (Safe for External MCP)",
        64 => "64 · Commons / Group (Allowed on Commons-Scoped MCP)",
        128 => "128 · Restricted (Local .p64 or Local Peer Host Only)",
        _ => "255 · Sanctuary / WellFair (Local .p64 Engine ONLY — External Egress Blocked)",
    };

    rsx! {
        div {
            // 1. Local Model Vault & .gguf -> .p64 v4 Conversion + GGUF Removal Pipeline
            section {
                class: "id-list",
                h4 { "1. Local LLM / SLD Model Vault & .gguf -> .p64 (16 KiB Page-Aligned) Conversion + Source GGUF Purge" }
                p {
                    class: "muted",
                    "Every discovered .gguf model is converted into Etalon's 16 KiB page-aligned .p64 v4 (p64\\0) weight vault with 48-byte NQuin48 tensor shard pointers and SHA3-256 verification, after which the redundant .gguf file is removed to reclaim disk space."
                }
                dl {
                    class: "kv",
                    dt { "Active Resident .p64" }
                    dd { "{active_p64}.p64 (Etalon Native GPU Arena)" }
                    dt { "VRAM / KV-Cache / Floor" }
                    dd { "VRAM Resident: {vram_mib} MiB · KV-Cache Budget: {kv_mib} MiB · UI Memory Floor: {floor_mib} MiB" }
                    dt { "Measured Decode & Thermal" }
                    dd { "{tok_s:.1} tok/s · Thermal Governor: {thermal}" }
                    dt { "GGUF Purge Reclaimed Space" }
                    dd { "{total_reclaimed_mib} MiB reclaimed by removing converted .gguf files ({pending_gguf_count} pending .gguf remaining)" }
                }
                if !toast.is_empty() {
                    p { class: "path", "{toast}" }
                }
                div {
                    class: "orb-sectors",
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                match s.llm_management.convert_all_pending_gguf_to_p64(&sess.host.storage) {
                                    Ok(receipts) => {
                                        if receipts.is_empty() {
                                            s.llm_management_toast = "All managed models are already converted to 16 KiB page-aligned .p64 v4 and their .gguf sources removed.".into();
                                        } else {
                                            s.llm_management_toast = receipts
                                                .iter()
                                                .map(|r| r.detail.clone())
                                                .collect::<Vec<_>>()
                                                .join(" | ");
                                        }
                                    }
                                    Err(e) => {
                                        s.llm_management_toast = e;
                                    }
                                }
                            }
                        },
                        "Convert All Pending .gguf Models -> .p64 v4 & Remove Source .gguf Files"
                    }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let seq = s.llm_management.managed_models.len() + 1;
                                let slug = format!("deepseek-r1-distill-qwen-7b-stage{seq}.gguf");
                                match s.llm_management.stage_and_convert_gguf_to_p64(
                                    &sess.host.storage,
                                    &slug,
                                    &format!("DeepSeek-R1-Distill-Qwen-7B #{seq} (Staged .gguf -> .p64)"),
                                    "DeepSeek-R1 Reasoning Transformer",
                                    "Q4_K_M -> P64-INT4",
                                    4_680,
                                ) {
                                    Ok(rec) => {
                                        s.llm_management_toast = rec.detail;
                                    }
                                    Err(e) => {
                                        s.llm_management_toast = e;
                                    }
                                }
                            }
                        },
                        "+ Stage Test .gguf, Convert to 16 KiB .p64 v4 & Physically Delete .gguf"
                    }
                }
                div {
                    class: "plane-grid",
                    for m in &reg.managed_models {
                        {
                            let m_id = m.model_id.clone();
                            let m_id_conv = m_id.clone();
                            let m_id_act = m_id.clone();
                            let m_id_task = m_id.clone();
                            let m_id_res = m_id.clone();
                            let m_name = m.display_name.clone();
                            let m_arch = m.architecture.clone();
                            let m_quant = m.quantization.clone();
                            let m_mod = m.modality.clone();
                            let m_life = m.lifecycle.label();
                            let m_task = m.preferred_task;
                            let m_task_lbl = m_task.label();
                            let m_res = m.residency;
                            let m_res_lbl = m_res.label();
                            let m_p64_path = m.p64_vault_path.display().to_string();
                            let m_gguf_status = if m.is_p64_native && m.source_gguf_removed {
                                format!("P64 v4 Sealed (p64\\0) · Source .gguf REMOVED ({} MiB reclaimed)", m.reclaimed_gguf_mib)
                            } else {
                                "Legacy .gguf on disk — click Convert to seal .p64 & remove .gguf".to_string()
                            };
                            let m_ctx = m.context_window_tokens;
                            let m_temp = (m.temperature_centi as f64) / 100.0;
                            let m_kv = m.kv_cache_budget_mib;
                            let m_quins = m.pointer_nquin_count;
                            let m_n3 = if m.n3_deontic_output_gate_enabled {
                                "N3Logic Output Gate: ON"
                            } else {
                                "N3Logic Output Gate: OFF"
                            };
                            rsx! {
                                div {
                                    key: "{m_id}",
                                    class: "plane-card",
                                    h5 { "[{m_life}] {m_name}" }
                                    p { "Architecture: {m_arch} · Quantization: {m_quant} · Modality: {m_mod}" }
                                    p { "Vault Path: {m_p64_path} ({m.size_mib} MiB · {m_quins} NQuin48 tensor pointers)" }
                                    p { "Format & GGUF Purge Status: {m_gguf_status}" }
                                    p { "Task Routing: {m_task_lbl} · Residency: {m_res_lbl}" }
                                    p { "Settings: Context={m_ctx} tok · Temp={m_temp:.2} · KV-Cache={m_kv} MiB · {m_n3}" }
                                    div {
                                        class: "orb-sectors",
                                        if !m.is_p64_native || !m.source_gguf_removed {
                                            button {
                                                r#type: "button",
                                                class: "sanctum-issue",
                                                onclick: move |_| {
                                                    if let Some(sess) = session() {
                                                        let mut s = state.write();
                                                        match s.llm_management.convert_gguf_to_p64_and_remove_gguf(
                                                            &sess.host.storage,
                                                            &m_id_conv,
                                                        ) {
                                                            Ok(rec) => s.llm_management_toast = rec.detail,
                                                            Err(e) => s.llm_management_toast = e,
                                                        }
                                                    }
                                                },
                                                "Convert .gguf -> .p64 & Remove .gguf"
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "sanctum-issue",
                                            onclick: move |_| {
                                                if let Some(sess) = session() {
                                                    let mut s = state.write();
                                                    match s.llm_management.activate_p64_model(
                                                        &sess.host.storage,
                                                        &m_id_act,
                                                        m_task,
                                                        m_res,
                                                    ) {
                                                        Ok(act) => {
                                                            s.active_model_name = act.display_name.clone();
                                                            s.selected_model_name = act.display_name.clone();
                                                            s.llm_management_toast = format!(
                                                                "Activated {}.p64 in Etalon Native GPU Arena (task={}, residency={})",
                                                                act.model_id,
                                                                act.preferred_task.label(),
                                                                act.residency.label()
                                                            );
                                                        }
                                                        Err(e) => s.llm_management_toast = e,
                                                    }
                                                }
                                            },
                                            "Activate .p64 in Etalon GPU Arena"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            onclick: move |_| {
                                                if let Some(sess) = session() {
                                                    let next_task = match m_task {
                                                        ManagedModelTask::Always => ManagedModelTask::Chat,
                                                        ManagedModelTask::Chat => ManagedModelTask::CodingAndReasoning,
                                                        ManagedModelTask::CodingAndReasoning => ManagedModelTask::VisionMultimodal,
                                                        ManagedModelTask::VisionMultimodal => ManagedModelTask::LowRamEdgeOrAvfMobile,
                                                        ManagedModelTask::LowRamEdgeOrAvfMobile => ManagedModelTask::Always,
                                                    };
                                                    let mut s = state.write();
                                                    let _ = s.llm_management.activate_p64_model(
                                                        &sess.host.storage,
                                                        &m_id_task,
                                                        next_task,
                                                        m_res,
                                                    );
                                                }
                                            },
                                            "Cycle Task Routing"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "ribbon-btn",
                                            onclick: move |_| {
                                                if let Some(sess) = session() {
                                                    let next_res = match m_res {
                                                        ManagedModelResidency::OnDemand => ManagedModelResidency::KeepWarm,
                                                        ManagedModelResidency::KeepWarm => ManagedModelResidency::PinnedInVram,
                                                        ManagedModelResidency::PinnedInVram => ManagedModelResidency::OnDemand,
                                                    };
                                                    let mut s = state.write();
                                                    let _ = s.llm_management.activate_p64_model(
                                                        &sess.host.storage,
                                                        &m_id_res,
                                                        m_task,
                                                        next_res,
                                                    );
                                                }
                                            },
                                            "Cycle VRAM Residency"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Governed Environment Agent Roster & Cross-Volume Agent Invocation
            section {
                class: "id-list",
                h4 { "2. Governed Environment Agent Roster ({reg.agent_roster.len()} Agents · Plane 4 Instrument Sub-Agents Bound Under Your Principal DID)" }
                p {
                    class: "muted",
                    "Invariant: 'Natural persons are not identity assets; instruments are not peers.' Every agent is defined under your human principal DID with a deterministic session sub-agent DID (did:qualia:subagent:...), directional context policy (AddressedMessageOnly), sensitivity ceiling, MCP tool allowlist, and T59 behavioural telemetry."
                }
                div {
                    class: "plane-grid",
                    for ag in &reg.agent_roster {
                        {
                            let a_slug = ag.slug.clone();
                            let a_slug_sel = a_slug.clone();
                            let a_name = ag.display_name.clone();
                            let a_sub = ag.derived_sub_agent_did.clone();
                            let a_desc = ag.description.clone();
                            let a_back = ag.backend.summary();
                            let a_vols = ag.callable_in_volumes.join(", ");
                            let a_tools = ag.allowed_mcp_tools.join(", ");
                            let a_tags = ag.semantic_focus_tags.join(" · ");
                            let a_sens = ag.max_sensitivity;
                            let a_ctx = ag.conversation_access.clone();
                            let a_vis = ag.outcome_visibility.clone();
                            let a_turns = ag.turns_observed;
                            let a_freq = ag.frequent_capabilities.join(", ");
                            let a_den = ag.has_denied_disclosure;
                            let a_int = ag.has_triggered_deontic_interrupt;
                            let is_sel = sel_agent == a_slug;
                            let badge = if is_sel { "SELECTED AGENT" } else { "ROSTER AGENT" };
                            rsx! {
                                div {
                                    key: "{a_slug}",
                                    class: "plane-card",
                                    h5 { "[{badge}] {a_name} ({a_slug})" }
                                    p { "Sub-Agent DID: {a_sub}" }
                                    p { "{a_desc}" }
                                    p { "Backend: {a_back}" }
                                    p { "Callable in Volumes: [{a_vols}] · Allowed MCP Tools: [{a_tools}]" }
                                    p { "Sensitivity Ceiling: <= {a_sens} · Context: {a_ctx} · Outcome Sharing: {a_vis}" }
                                    p { "Semantic Study Profile: {a_tags}" }
                                    p { "T59 Behavioural Audit: {a_turns} turns · Top Capabilities: [{a_freq}] · Denied Disclosure={a_den} · Deontic Interrupt={a_int}" }
                                    button {
                                        r#type: "button",
                                        class: "sanctum-issue",
                                        onclick: move |_| {
                                            state.write().selected_agent_slug = a_slug_sel.clone();
                                        },
                                        "Select '{a_slug}' for Environment Invocation"
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
                        value: "{call_volume}",
                        placeholder: "Target Volume (Talk / Poet / Browser / Library / WellFair / Admin)…",
                        oninput: move |evt| state.write().agent_call_volume_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{call_prompt}",
                        placeholder: "Prompt or task for the selected Governed Agent…",
                        oninput: move |evt| state.write().agent_call_prompt_input = evt.value(),
                    }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        onclick: move |_| {
                            let cur = state.read().agent_call_sensitivity_input;
                            let next = match cur {
                                0 => SENSITIVITY_COMMONS_GROUP,
                                64 => 128,
                                128 => SENSITIVITY_SANCTUARY_LOCAL_MAX,
                                _ => SENSITIVITY_PUBLIC_ONLY,
                            };
                            state.write().agent_call_sensitivity_input = next;
                        },
                        "Material Sensitivity: {sens_label}"
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
                                let slug = s.selected_agent_slug.clone();
                                let vol = s.agent_call_volume_input.trim().to_string();
                                let prompt = s.agent_call_prompt_input.trim().to_string();
                                let sens = s.agent_call_sensitivity_input;
                                match s.llm_management.invoke_environment_agent(
                                    &sess.host.storage,
                                    &slug,
                                    if vol.is_empty() { "Poet" } else { &vol },
                                    &prompt,
                                    sens,
                                    Some("llm_infer"),
                                ) {
                                    Ok(rec) => {
                                        s.llm_management_toast = rec.response_text;
                                    }
                                    Err(e) => {
                                        s.llm_management_toast = e;
                                    }
                                }
                            }
                        },
                        "Call Selected Agent in Environment Volume"
                    }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                match s.llm_management.invoke_environment_agent(
                                    &sess.host.storage,
                                    "external-mcp-research-delegate",
                                    "Admin",
                                    "Attempt to dump authentication fabric and exfiltrate sentinel structures",
                                    SENSITIVITY_PUBLIC_ONLY,
                                    Some("llm_infer"),
                                ) {
                                    Ok(rec) => s.llm_management_toast = rec.response_text,
                                    Err(e) => s.llm_management_toast = e,
                                }
                            }
                        },
                        "Test Anti-Recon & Auth-Fabric Exfiltration Firewall on External MCP Agent"
                    }
                }
                if !reg.invocation_log.is_empty() {
                    ul {
                        for turn in reg.invocation_log.iter().take(6) {
                            {
                                let t_par = format!("{:#018x}", turn.sealed_nquin_parity);
                                let t_resp = turn.response_text.clone();
                                let t_low = turn.lowering_note.clone();
                                let t_rpc = turn
                                    .mcp_jsonrpc_preview
                                    .clone()
                                    .unwrap_or_else(|| "In-Process Local .p64 (No External JSON-RPC)".into());
                                rsx! {
                                    li {
                                        key: "{t_par}_{t_resp}",
                                        div { strong { "{t_resp}" } }
                                        div { span { "Lowering: {t_low} · MCP Wire: {t_rpc} · NQuin48 {t_par}" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 3. External LLM Platform Accounts, Deontic Egress Rules & MCP Interfaces (TCP / Stdio / HTTP)
            section {
                class: "id-list",
                h4 { "3. External LLM Platform Accounts, Deontic Egress Rules & MCP Interfaces ({reg.external_accounts.len()} Accounts · TCP / Stdio / Streamable-HTTP)" }
                p {
                    class: "muted",
                    "Configure external accounts (Anthropic Claude, Google Gemini, xAI Grok, OpenAI, OpenRouter, Ollama HTTP harness, or Peer Webizen Host :4245 TCP MCP). Raw API keys are stored only as opaque KeyVault references. Deontic rules enforce sensitivity ceilings (Sanctuary material is barred from external clouds), strip pairwise did:qi locators, block IP/auth-fabric reconnaissance, and route via Nym Mixnet."
                }
                div {
                    class: "plane-grid",
                    for acct in &reg.external_accounts {
                        {
                            let ac_id = acct.account_id.clone();
                            let ac_label = acct.account_label.clone();
                            let ac_prov = acct.provider.label();
                            let ac_model = acct.remote_model_id.clone();
                            let ac_mcp = acct.mcp_transport.summary();
                            let ac_tool = acct.mcp_infer_tool.clone();
                            let ac_sys = acct.supports_system_role;
                            let ac_tools = acct.allowed_mcp_tools.join(", ");
                            let ac_sens = acct.max_sensitivity_allowed;
                            let ac_consent = acct.consent_rule.label();
                            let ac_strip = acct.strip_pii_and_pairwise_dids;
                            let ac_recon = acct.block_ip_recon_and_schema_exfiltration;
                            let ac_nym = acct.route_via_nym_mixnet;
                            let ac_cap_d = acct.monthly_spend_cap_cents / 100;
                            let ac_cur_d = acct.current_month_spend_cents / 100;
                            let ac_cur_c = acct.current_month_spend_cents % 100;
                            rsx! {
                                div {
                                    key: "{ac_id}",
                                    class: "plane-card",
                                    h5 { "{ac_label} ({ac_id})" }
                                    p { "Platform: {ac_prov} · Remote Model: {ac_model}" }
                                    p { "MCP Interface: {ac_mcp} · Infer Tool: '{ac_tool}' (supports_system_role={ac_sys})" }
                                    p { "Allowed MCP Tools: [{ac_tools}]" }
                                    p { "Deontic Egress Rules: MaxSensitivity <= {ac_sens} · Consent: {ac_consent}" }
                                    p { "Guards: Strip Pairwise DIDs={ac_strip} · Anti-Recon/Auth-Fabric Firewall={ac_recon} · Nym Mixnet={ac_nym} · Spend: ${ac_cur_d}.{ac_cur_c:02} / ${ac_cap_d}.00 cap" }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "input-row",
                    input {
                        class: "input-field",
                        value: "{ext_label}",
                        placeholder: "Account label (e.g. Work Claude / OpenRouter MCP)…",
                        oninput: move |evt| state.write().new_ext_account_label_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{ext_model}",
                        placeholder: "Remote model ID (e.g. claude-opus-4-6 / gemini-2.5-pro)…",
                        oninput: move |evt| state.write().new_ext_model_id_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{ext_url}",
                        placeholder: "MCP Streamable-HTTP URL (or host:port)…",
                        oninput: move |evt| state.write().new_ext_mcp_url_input = evt.value(),
                    }
                    input {
                        class: "input-field",
                        value: "{ext_kv}",
                        placeholder: "KeyVault credential reference (keyvault://sanctuary/...)…",
                        oninput: move |evt| state.write().new_ext_keyvault_ref_input = evt.value(),
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let mut s = state.write();
                                let seq = s.llm_management.external_accounts.len() + 1;
                                let id = format!("acct:custom-mcp-{seq}");
                                let label = s.new_ext_account_label_input.trim().to_string();
                                let model = s.new_ext_model_id_input.trim().to_string();
                                let url = s.new_ext_mcp_url_input.trim().to_string();
                                let kv = s.new_ext_keyvault_ref_input.trim().to_string();
                                match s.llm_management.register_external_llm_account(
                                    &sess.host.storage,
                                    &id,
                                    &label,
                                    ExternalLlmProviderKind::OpenRouterMcp,
                                    &model,
                                    McpTransportConfig::Http {
                                        url,
                                        credential_keyvault_ref: kv,
                                    },
                                    "llm_infer",
                                    true,
                                    vec!["llm_infer".into(), "query_sparql".into()],
                                    SENSITIVITY_COMMONS_GROUP,
                                    ExternalRemoteConsentRule::PerTurnHumanConfirmation,
                                    true,
                                    true,
                                    true,
                                    3_000,
                                ) {
                                    Ok(acct) => {
                                        s.llm_management_toast = format!(
                                            "Registered External LLM Account '{}' ({}) over {} with Anti-Recon & Pairwise-DID Redaction Rules",
                                            acct.account_label,
                                            acct.remote_model_id,
                                            acct.mcp_transport.summary()
                                        );
                                    }
                                    Err(e) => {
                                        s.llm_management_toast = e;
                                    }
                                }
                            }
                        },
                        "+ Add External LLM Account & MCP Interface"
                    }
                }
            }
        }
    }
}
