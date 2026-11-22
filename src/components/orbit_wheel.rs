use super::{
    auxiliary_windows::{jump_to_stage, AuxWindowBridge},
    volumes::admin::build_appliance_packages,
};
use crate::state::{session, ShellState, VolumeStage};
use aether::prelude::*;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use webizen_ua_os::{
    classify_semantic_intent, resolve_holo_10d_specimen, run_three_node_mesh_simulation,
    BrowserExecutionMode, SemanticIntentAction, UaOsHost, SENSITIVITY_SANCTUARY_LOCAL_MAX,
};

const APPS: &[(&str, VolumeStage, &str, &str)] = &[
    ("Talk", VolumeStage::Talk, "💬", "Human <-> human · QDNF"),
    ("Mail", VolumeStage::Mail, "✉", "Pairwise locator + inbox"),
    (
        "Directory",
        VolumeStage::Directory,
        "👥",
        "NaturalAgent social book",
    ),
    ("Browser", VolumeStage::Browser, "🌐", "Chora + WWW worker"),
    (
        "Keep · Sanctuary",
        VolumeStage::Keep,
        "🛡",
        "4-slot VaultContainerV2",
    ),
    (
        "Library",
        VolumeStage::Library,
        "📚",
        ".q42 / .10d / .hmc / QECP",
    ),
    (
        "Instruments",
        VolumeStage::Instruments,
        "🎛",
        "6 Core SI + 40 Logics",
    ),
    (
        "Poet & Synth",
        VolumeStage::Poet,
        "🪶",
        "VibeScript + 150 morphisms",
    ),
    (
        "WellFair",
        VolumeStage::Wellfair,
        "💚",
        "Health, dyad & guardianship",
    ),
    (
        "Anatomy (Qap)",
        VolumeStage::Anatomy,
        "🫀",
        "10D v2 HOLO medical atlas",
    ),
    ("Wallet", VolumeStage::Wallet, "⚡", "Lightning · Nym · XEC"),
    (
        "Projects",
        VolumeStage::Projects,
        "📋",
        "9-Stage Contribution Ledger",
    ),
    (
        "Settings",
        VolumeStage::Settings,
        "⚙",
        "10 tabs · Four-Plane Fabric",
    ),
    (
        "Admin",
        VolumeStage::Admin,
        "🔧",
        "Sidecar · Jobs · Dual-SKU",
    ),
];

#[component]
pub fn OrbitWheel(mut state: Signal<ShellState>) -> Element {
    let qap_count = state.read().installed_qaps.len();
    let sub_text = format!(
        "First-class volumes + {} verified Qap packages in storage/Qapps/registry.json · Live vs Planned honesty only.",
        qap_count
    );
    rsx! {
        div {
            id: "launcher",
            class: "open",
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "launcher-title",
            onclick: move |_| state.write().launcher_open = false,
            div {
                id: "launcher-panel",
                onclick: move |evt| evt.stop_propagation(),
                h2 { id: "launcher-title", "Constellation · Foundational Volumes & Qaps" }
                p {
                    class: "sub",
                    "{sub_text}"
                }
                OmniIntentBar { state }
                ul {
                    id: "launcher-list",
                    role: "listbox",
                    aria_label: "Apps",
                    for (name, stage_item, glyph, blurb) in APPS.iter() {
                        {
                            let stage = *stage_item;
                            rsx! {
                                li {
                                    key: "{name}",
                                    button {
                                        r#type: "button",
                                        onclick: move |_| {
                                            open_volume(&mut state.write(), stage);
                                        },
                                        span { "{glyph}" }
                                        span { "{name}" }
                                        span { class: "pill live", "{blurb}" }
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
fn OmniIntentBar(mut state: Signal<ShellState>) -> Element {
    let bridge = try_consume_context::<AuxWindowBridge>();
    let (draft, status) = {
        let shell = state.read();
        (
            shell.omni_intent_draft.clone(),
            shell.omni_intent_status.clone(),
        )
    };
    let preview = (!draft.trim().is_empty()).then(|| classify_semantic_intent(&draft));
    let can_submit = matches!(&preview, Some(Ok(_)));

    rsx! {
        form {
            class: "omni-intent-bar",
            onsubmit: move |event| {
                event.prevent_default();
                let command = state.peek().omni_intent_draft.clone();
                match classify_semantic_intent(&command)
                    .and_then(|intent| dispatch_intent(state, bridge.as_ref(), intent.action))
                {
                    Ok(()) => {
                        let mut shell = state.write();
                        shell.omni_intent_draft.clear();
                        shell.omni_intent_status.clear();
                        shell.launcher_open = false;
                    }
                    Err(error) => state.write().omni_intent_status = error,
                }
            },
            label { r#for: "omni-intent-draft", "Command or search" }
            div {
                class: "input-row",
                input {
                    id: "omni-intent-draft",
                    class: "input-field",
                    value: "{draft}",
                    maxlength: 4096,
                    aria_describedby: "omni-intent-preview",
                    placeholder: "Volume, address, .10d file, or @agent prompt",
                    onmounted: move |element| async move {
                        let _ = element.set_focus(true).await;
                    },
                    oninput: move |event| {
                        let mut shell = state.write();
                        shell.omni_intent_draft = event.value();
                        shell.omni_intent_status.clear();
                    },
                }
                button { r#type: "submit", disabled: !can_submit, "Run" }
            }
            div {
                id: "omni-intent-preview",
                role: "status",
                if !status.is_empty() {
                    p { "{status}" }
                } else {
                    match preview {
                        Some(Ok(intent)) => rsx! {
                            strong { "{intent.badge}" }
                            span { " · {intent.explanation}" }
                        },
                        Some(Err(error)) => rsx! { p { "{error}" } },
                        None => rsx! {},
                    }
                }
            }
        }
    }
}

fn dispatch_intent(
    mut state: Signal<ShellState>,
    bridge: Option<&AuxWindowBridge>,
    action: SemanticIntentAction,
) -> Result<(), String> {
    match action {
        SemanticIntentAction::OpenVolume { stage_slug, .. } => {
            open_volume(&mut state.write(), stage_by_slug(stage_slug)?);
        }
        SemanticIntentAction::NavigateBrowser { uri, mode } => {
            navigate(&mut state.write(), uri, mode);
        }
        SemanticIntentAction::OpenHolo10dSpecimen { specimen_query } => {
            let session = session().ok_or("Open a local session to inspect a HOLO specimen.")?;
            let specimen = resolve_holo_10d_specimen(&session.host.storage, &specimen_query)?;
            let route = format!("qualia://specimens/{}", specimen.specimen_id);
            let mut shell = state.write();
            shell.active_holo_specimen_id = route.clone();
            shell.holo_shader_mode = None;
            navigate(&mut shell, route, BrowserExecutionMode::QualiaNative);
        }
        SemanticIntentAction::StageGgufForP64Conversion { gguf_path } => {
            let draft = inspect_gguf_source(Path::new(&gguf_path))?;
            let mut shell = state.write();
            shell.staged_gguf_source = Some(draft);
            open_volume(&mut shell, VolumeStage::Instruments);
        }
        SemanticIntentAction::InvokeGovernedAgent { agent_slug, prompt } => {
            let mut shell = state.write();
            let agent = shell
                .llm_management
                .agent_roster
                .iter()
                .find(|agent| agent.enabled && agent.slug == agent_slug)
                .ok_or("The requested agent is absent or disabled.")?;
            let volume = agent
                .callable_in_volumes
                .first()
                .cloned()
                .ok_or("The requested agent has no assigned volume.")?;
            shell.selected_agent_slug = agent_slug;
            shell.agent_call_prompt_input = prompt;
            shell.agent_call_volume_input = volume;
            // Start with a local-only sensitivity class. The user reviews scope before submission.
            shell.agent_call_sensitivity_input = SENSITIVITY_SANCTUARY_LOCAL_MAX;
            shell.llm_management_toast =
                "Review the agent, target volume, and prompt classification before sending.".into();
            open_volume(&mut shell, VolumeStage::Instruments);
        }
        SemanticIntentAction::SearchSemanticLibrary { query } => {
            let mut shell = state.write();
            shell.library_filter.text_query = query;
            shell.selected_library_uri.clear();
            open_volume(&mut shell, VolumeStage::Library);
        }
        SemanticIntentAction::RunThreeNodeMeshSimulation => {
            let session = session().ok_or("Open a local session to run the mesh simulation.")?;
            let receipt = run_three_node_mesh_simulation(
                &session.host.storage,
                &session.identifiers,
                &session.gpu,
            );
            let mut shell = state.write();
            shell.mesh_simulation = Some(receipt);
            open_volume(&mut shell, VolumeStage::Admin);
        }
        SemanticIntentAction::BuildTriTargetDebianSuite => {
            if session().is_none() {
                return Err("Open a local host session to build Debian archives.".into());
            }
            build_appliance_packages(state);
            open_volume(&mut state.write(), VolumeStage::Admin);
        }
        SemanticIntentAction::TriggerPrincipalOkCheckIn => {
            let bridge = bridge.ok_or("The native care alert window is unavailable.")?;
            open_volume(&mut state.write(), VolumeStage::Wellfair);
            bridge.open_urgent_care_alert_modal();
        }
    }
    Ok(())
}

fn open_volume(state: &mut ShellState, stage: VolumeStage) {
    let qap = match stage {
        VolumeStage::Anatomy => Some("Anatomy"),
        VolumeStage::Poet => Some("Poet"),
        _ => None,
    };
    if let (Some(qap), Some(session)) = (qap, session()) {
        state.active_launch_plan = session.host.launch(qap, None).ok();
    }
    jump_to_stage(state, stage);
    state.orb_open = false;
    state.launcher_open = false;
}

fn navigate(shell: &mut ShellState, target: String, mode: BrowserExecutionMode) {
    shell.browser_history.insert(
        0,
        format!("[{}] {} — {}", mode.title(), target, mode.engine_and_ring()),
    );
    shell.browser_url = target;
    shell.browser_mode = mode;
    open_volume(shell, VolumeStage::Browser);
}

fn stage_by_slug(slug: &str) -> Result<VolumeStage, String> {
    match slug {
        "talk" => Ok(VolumeStage::Talk),
        "mail" => Ok(VolumeStage::Mail),
        "directory" => Ok(VolumeStage::Directory),
        "browser" => Ok(VolumeStage::Browser),
        "keep" => Ok(VolumeStage::Keep),
        "library" => Ok(VolumeStage::Library),
        "instruments" => Ok(VolumeStage::Instruments),
        "poet" => Ok(VolumeStage::Poet),
        "wellfair" => Ok(VolumeStage::Wellfair),
        "wallet" => Ok(VolumeStage::Wallet),
        "projects" => Ok(VolumeStage::Projects),
        "anatomy" => Ok(VolumeStage::Anatomy),
        "settings" => Ok(VolumeStage::Settings),
        "admin" => Ok(VolumeStage::Admin),
        _ => Err("The intent names an unknown volume.".into()),
    }
}

fn inspect_gguf_source(path: &Path) -> Result<(PathBuf, u64), String> {
    #[cfg(windows)]
    if let Some(std::path::Component::Prefix(prefix)) = path.components().next() {
        if !matches!(
            prefix.kind(),
            std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)
        ) {
            return Err("Select a GGUF file from a local drive.".into());
        }
    }
    // Keep the draft separate from the legacy conversion queue, which can remove its input.
    let source = fs::canonicalize(path)
        .map_err(|error| format!("Could not resolve the local GGUF source: {error}"))?;
    if !source.is_file() {
        return Err("Select a regular GGUF file.".into());
    }
    let mut file = fs::File::open(&source)
        .map_err(|error| format!("Could not open the local GGUF source: {error}"))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("Could not inspect the local GGUF source: {error}"))?;
    let mut header = [0; 8];
    file.read_exact(&mut header)
        .map_err(|_| "The GGUF header is truncated.")?;
    let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    if &header[..4] != b"GGUF" || !(1..=3).contains(&version) {
        return Err("The source does not carry a recognized GGUF header.".into());
    }
    Ok((source, metadata.len()))
}
