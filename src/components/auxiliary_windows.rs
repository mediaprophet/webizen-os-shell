// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use aether::{aether_core::VirtualDom, prelude::*};
use aether_native::{
    winit::{
        event::WindowEvent,
        window::{Window, WindowId},
    },
    Config, LogicalSize, NativeAppHandle, RendererFallbackPolicy, WindowAttributes,
    WindowClosePolicy, WindowRole,
};
use std::{
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};
use webizen_ua_os::{
    trigger_one_tap_principal_ok_checkin, CareAlertSeverity, DispatchedCareAndGuardianAlert,
    SemanticToolboxId, SENSITIVITY_PUBLIC_ONLY,
};

use crate::state::{session, ShellState, StageChromeMode, VolumeStage};

const MAX_PENDING_COMMANDS: usize = 64;
const MAX_PROMPT_BYTES: usize = 4096;
const OVERLAY_CSS: &str = "
html, body { margin: 0; background: transparent; color: #e9f7f5; font: 14px system-ui; }
.aux-window { box-sizing: border-box; min-height: 100vh; padding: 1rem; background: #14242b; }
.aux-orb { width: calc(100% - 1rem); min-height: 0; margin: .5rem; border: 1px solid #498482;
           border-radius: 22px; background: rgba(13, 34, 39, .88); }
.aux-row { display: flex; flex-wrap: wrap; gap: .4rem; margin-bottom: .75rem; }
.aux-orb-sectors { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr));
    grid-template-rows: repeat(3, 2rem); gap: .25rem; margin-bottom: .75rem; }
.aux-orb-sectors > :nth-child(1) { grid-area: 1 / 2; }
.aux-orb-sectors > :nth-child(2) { grid-area: 1 / 3; }
.aux-orb-sectors > :nth-child(3) { grid-area: 2 / 3; }
.aux-orb-sectors > :nth-child(4) { grid-area: 3 / 3; }
.aux-orb-sectors > :nth-child(5) { grid-area: 3 / 2; }
.aux-orb-sectors > :nth-child(6) { grid-area: 3 / 1; }
.aux-orb-sectors > :nth-child(7) { grid-area: 2 / 1; }
.aux-orb-sectors > :nth-child(8) { grid-area: 1 / 1; }
.aux-orb-hub { grid-area: 2 / 2; text-align: center; line-height: 2rem; font-size: 1.5rem; color: #2fe0c8; }
.aux-window h2 { font-size: 1.12rem; margin: 0 0 .75rem; }
.aux-window p { margin: .55rem 0; }
.aux-window button, .aux-window input { color: inherit; background: #1c3941;
    border: 1px solid #4f777a; border-radius: .45rem; padding: .4rem .55rem; }
.aux-window button:focus, .aux-window input:focus { border-color: #2fe0c8; }
.aux-alert article { border: 1px solid #946f60; border-radius: .5rem; padding: .7rem; margin: .6rem 0; }
.aux-window label { display: block; margin: .45rem 0; }
";

/// Host-supplied resolver for an authenticated, scoped responder.
/// The shell session's fabric identifiers and an alert's stored DIDs do not grant this authority.
#[derive(Clone)]
pub(crate) struct CareResponseAuthority {
    pub(crate) resolve: Rc<dyn Fn(&DispatchedCareAndGuardianAlert, bool) -> Result<String, String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuxiliaryKind {
    Orb,
    Alert,
    Palette,
}

impl AuxiliaryKind {
    fn config(self) -> Config {
        let (role, close, fallback, width, height, title) = match self {
            Self::Orb => (
                WindowRole::OrbOverlay,
                WindowClosePolicy::HideOnClose,
                RendererFallbackPolicy::FailClosed,
                420.0,
                340.0,
                "Webizen Command Orb",
            ),
            Self::Alert => (
                WindowRole::UrgentAlertModal,
                WindowClosePolicy::CloseAndDrop,
                RendererFallbackPolicy::CpuOnFailure,
                520.0,
                400.0,
                "Webizen Urgent Care Alerts",
            ),
            Self::Palette => (
                WindowRole::AuxiliaryToolPalette,
                WindowClosePolicy::HideOnClose,
                RendererFallbackPolicy::CpuOnFailure,
                380.0,
                560.0,
                "Webizen Semantic ToolChest",
            ),
        };
        Config::new()
            .with_window_role(role)
            .with_close_policy(close)
            .with_renderer_fallback(fallback)
            .with_window_attributes(
                WindowAttributes::default()
                    .with_title(title)
                    .with_surface_size(LogicalSize::new(width, height)),
            )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrbSector {
    Close,
    Soften,
    Full,
    Float,
    Keep,
    Share,
    Inspect,
    More,
}

const ORB_SECTORS: [(OrbSector, &str); 8] = [
    (OrbSector::Close, "Close"),
    (OrbSector::Soften, "Soften"),
    (OrbSector::Full, "Full"),
    (OrbSector::Float, "Float"),
    (OrbSector::Keep, "Keep"),
    (OrbSector::Share, "Share"),
    (OrbSector::Inspect, "Inspect"),
    (OrbSector::More, "More"),
];

pub(crate) enum AuxShellCommand {
    Open(AuxiliaryKind),
    Dismiss(AuxiliaryKind),
    RunOrbSector(OrbSector),
    JumpToStage(VolumeStage),
    AcknowledgeCareAlert {
        alert_id: String,
        principal_ok: bool,
    },
    ToggleToolbox(SemanticToolboxId),
    InvokeAgent {
        slug: String,
        prompt: String,
        volume: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AuxCareAlert {
    id: String,
    severity: &'static str,
    trigger: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AuxSnapshot {
    current_stage: VolumeStage,
    orb_status: String,
    alerts: Vec<AuxCareAlert>,
    care_authority_available: bool,
    care_status: String,
    toolboxes: [(SemanticToolboxId, bool); 6],
    agent_volume: Option<&'static str>,
    agents: Vec<(String, String)>,
    agent_status: String,
}

impl Default for AuxSnapshot {
    fn default() -> Self {
        Self {
            current_stage: VolumeStage::None,
            orb_status: String::new(),
            alerts: Vec::new(),
            care_authority_available: false,
            care_status: String::new(),
            toolboxes: SemanticToolboxId::ALL.map(|id| (id, false)),
            agent_volume: None,
            agents: Vec::new(),
            agent_status: String::new(),
        }
    }
}

struct AuxWindowSlot {
    id: WindowId,
    handle: Arc<dyn Window>,
}

#[derive(Default)]
struct AuxWindowState {
    window: Option<AuxWindowSlot>,
    opening: bool,
    visible: bool,
}

#[derive(Default)]
struct AuxBridgeInner {
    snapshot: AuxSnapshot,
    pending_commands: VecDeque<AuxShellCommand>,
    main_window: Option<Arc<dyn Window>>,
    orb: AuxWindowState,
    alert: AuxWindowState,
    palette: AuxWindowState,
}

impl AuxBridgeInner {
    fn window_mut(&mut self, kind: AuxiliaryKind) -> &mut AuxWindowState {
        match kind {
            AuxiliaryKind::Orb => &mut self.orb,
            AuxiliaryKind::Alert => &mut self.alert,
            AuxiliaryKind::Palette => &mut self.palette,
        }
    }
}

/// Exchanges commands and bounded UI state across independent native VirtualDOMs.
#[derive(Clone, Default)]
pub struct AuxWindowBridge(Arc<Mutex<AuxBridgeInner>>);

impl AuxWindowBridge {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, AuxBridgeInner> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn bind_main(&self, window: Arc<dyn Window>) {
        self.lock().main_window = Some(window);
    }

    pub fn open_or_toggle_orb_overlay(&self) {
        self.enqueue(AuxShellCommand::Open(AuxiliaryKind::Orb));
    }

    pub fn open_urgent_care_alert_modal(&self) {
        self.enqueue(AuxShellCommand::Open(AuxiliaryKind::Alert));
    }

    pub fn open_or_toggle_tool_palette(&self) {
        self.enqueue(AuxShellCommand::Open(AuxiliaryKind::Palette));
    }

    fn enqueue(&self, command: AuxShellCommand) {
        let window = {
            let mut inner = self.lock();
            if inner.pending_commands.len() == MAX_PENDING_COMMANDS {
                return;
            }
            inner.pending_commands.push_back(command);
            inner.main_window.clone()
        };
        if let Some(window) = window {
            window.request_redraw();
        }
    }

    pub(crate) fn drain_commands(&self) -> VecDeque<AuxShellCommand> {
        std::mem::take(&mut self.lock().pending_commands)
    }

    pub(crate) fn open_flags(&self) -> (bool, bool, bool) {
        let inner = self.lock();
        (
            inner.orb.visible,
            inner.alert.visible,
            inner.palette.visible,
        )
    }

    pub(crate) fn sync_from_state(&self, state: &ShellState, care_authority_available: bool) {
        let agent_volume = quick_invocation_volume(state.current_stage);
        let snapshot = AuxSnapshot {
            current_stage: state.current_stage,
            orb_status: state.orb_toast.clone(),
            alerts: state
                .host_stewardship
                .care_alerts
                .iter()
                .filter(|alert| {
                    !alert.resolved_or_reversed && alert.severity >= CareAlertSeverity::Urgent
                })
                .take(20)
                .map(|alert| AuxCareAlert {
                    id: alert.alert_id.clone(),
                    severity: alert.severity.label(),
                    trigger: alert.trigger_kind.label(),
                })
                .collect(),
            care_authority_available,
            care_status: if care_authority_available {
                state.host_stewardship_toast.clone()
            } else {
                String::new()
            },
            toolboxes: SemanticToolboxId::ALL
                .map(|id| (id, state.semantic_fs.is_toolbox_enabled(id))),
            agent_volume,
            agents: state
                .llm_management
                .agent_roster
                .iter()
                .filter(|agent| {
                    agent_volume.is_some_and(|volume| quick_agent_allowed(agent, volume))
                })
                .take(32)
                .map(|agent| (agent.slug.clone(), agent.display_name.clone()))
                .collect(),
            agent_status: state.llm_management_toast.clone(),
        };
        let windows = {
            let mut inner = self.lock();
            if snapshot == inner.snapshot {
                return;
            }
            inner.snapshot = snapshot;
            [&inner.orb, &inner.alert, &inner.palette]
                .into_iter()
                .filter(|slot| slot.visible)
                .filter_map(|slot| {
                    slot.window
                        .as_ref()
                        .map(|window| Arc::clone(&window.handle))
                })
                .collect::<Vec<_>>()
        };
        for window in windows {
            window.request_redraw();
        }
    }

    fn snapshot(&self) -> AuxSnapshot {
        self.lock().snapshot.clone()
    }

    pub(crate) fn open(&self, app: &NativeAppHandle, kind: AuxiliaryKind) {
        let existing = {
            let mut inner = self.lock();
            let slot = inner.window_mut(kind);
            if let Some(window) = &slot.window {
                slot.visible = kind == AuxiliaryKind::Alert || !slot.visible;
                Some((window.id, slot.visible))
            } else if slot.opening {
                slot.visible = kind == AuxiliaryKind::Alert || !slot.visible;
                return;
            } else {
                slot.opening = true;
                slot.visible = true;
                None
            }
        };
        if let Some((id, visible)) = existing {
            app.set_window_visible(id, visible);
        } else {
            app.open_window(
                VirtualDom::new_with_props(
                    overlay_root,
                    OverlayProps {
                        kind,
                        bridge: self.clone(),
                    },
                ),
                kind.config(),
            );
        }
    }

    fn register(&self, kind: AuxiliaryKind, window: Arc<dyn Window>) -> bool {
        let mut inner = self.lock();
        let slot = inner.window_mut(kind);
        slot.opening = false;
        slot.window = Some(AuxWindowSlot {
            id: window.id(),
            handle: window,
        });
        slot.visible
    }

    pub(crate) fn dismiss(&self, app: &NativeAppHandle, kind: AuxiliaryKind) {
        let request = {
            let mut inner = self.lock();
            let slot = inner.window_mut(kind);
            slot.visible = false;
            let id = slot.window.as_ref().map(|window| window.id);
            if kind == AuxiliaryKind::Alert {
                slot.window = None;
            }
            id
        };
        if let Some(id) = request {
            app.request_close(id);
        }
    }

    fn on_close(&self, kind: AuxiliaryKind, id: WindowId, dropped: bool) {
        let main = {
            let mut inner = self.lock();
            let slot = inner.window_mut(kind);
            if slot.window.as_ref().is_none_or(|window| window.id != id) {
                return;
            }
            slot.visible = false;
            if dropped || kind == AuxiliaryKind::Alert {
                slot.window = None;
            }
            inner.main_window.clone()
        };
        if let Some(main) = main {
            main.request_redraw();
        }
    }
}

fn quick_invocation_volume(stage: VolumeStage) -> Option<&'static str> {
    match stage {
        VolumeStage::Talk => Some("Talk"),
        VolumeStage::Poet => Some("Poet"),
        VolumeStage::Browser => Some("Browser"),
        VolumeStage::Library => Some("Library"),
        VolumeStage::Wellfair => Some("WellFair"),
        VolumeStage::Admin => Some("Admin"),
        _ => None,
    }
}

fn quick_agent_allowed(agent: &webizen_ua_os::EnvironmentAgentRosterEntry, volume: &str) -> bool {
    agent.enabled
        && agent
            .callable_in_volumes
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(volume))
        && agent
            .allowed_mcp_tools
            .iter()
            .any(|tool| matches!(tool.as_str(), "*" | "llm_infer"))
}

pub(crate) fn apply_shell_command(
    state: &mut ShellState,
    command: AuxShellCommand,
    care_authority: Option<&CareResponseAuthority>,
) {
    match command {
        AuxShellCommand::Open(_) | AuxShellCommand::Dismiss(_) => (),
        AuxShellCommand::JumpToStage(stage) => jump_to_stage(state, stage),
        AuxShellCommand::RunOrbSector(sector) => match sector {
            OrbSector::Close => {
                jump_to_stage(state, VolumeStage::None);
                state.orb_open = false;
                state.orb_toast = "Closed the volume".into();
            }
            OrbSector::Soften => {
                jump_to_stage(state, VolumeStage::None);
                state.stage_chrome_mode = StageChromeMode::DefaultDepth;
                state.orb_open = false;
                state.orb_toast = "Returned the volume to the spatial orbit".into();
            }
            OrbSector::Full => {
                state.stage_chrome_mode = if state.stage_chrome_mode == StageChromeMode::FullStage {
                    StageChromeMode::DefaultDepth
                } else {
                    StageChromeMode::FullStage
                };
                state.orb_toast = "Toggled full stage geometry".into();
            }
            OrbSector::Float => {
                state.stage_chrome_mode = if state.stage_chrome_mode == StageChromeMode::FloatLayer
                {
                    StageChromeMode::DefaultDepth
                } else {
                    StageChromeMode::FloatLayer
                };
                state.orb_toast = "Toggled the elevated float layer".into();
            }
            OrbSector::Keep => {
                jump_to_stage(state, VolumeStage::Keep);
                state.orb_toast = "Opened Sanctuary Keep".into();
            }
            OrbSector::Share => {
                state.orb_toast =
                    "Choose a recipient and consent scope in the active volume before sharing."
                        .into();
            }
            OrbSector::Inspect => {
                crate::components::toggle_dev_inspector(state);
            }
            OrbSector::More => {
                jump_to_stage(state, VolumeStage::Settings);
                state.orb_open = false;
                state.orb_toast = "Opened volume settings".into();
            }
        },
        AuxShellCommand::ToggleToolbox(toolbox) => {
            state.semantic_fs.toggle_toolbox(toolbox);
        }
        AuxShellCommand::AcknowledgeCareAlert {
            alert_id,
            principal_ok,
        } => {
            let result = record_care_response(state, &alert_id, principal_ok, care_authority);
            state.host_stewardship_toast = match result {
                Ok(status) => {
                    state.refresh_host_profiles();
                    status
                }
                Err(reason) => reason,
            };
        }
        AuxShellCommand::InvokeAgent {
            slug,
            prompt,
            volume,
        } => {
            let result = (|| {
                let session = session().ok_or("A local Webizen session is required.")?;
                if prompt.trim().is_empty() || prompt.len() > MAX_PROMPT_BYTES {
                    return Err("Enter a public prompt within 4,096 bytes.".into());
                }
                if quick_invocation_volume(state.current_stage) != Some(volume)
                    || !state
                        .llm_management
                        .agent_roster
                        .iter()
                        .any(|agent| agent.slug == slug && quick_agent_allowed(agent, volume))
                {
                    return Err("This instrument is unavailable for the active volume.".into());
                }
                state
                    .llm_management
                    .invoke_environment_agent(
                        &session.host.storage,
                        &slug,
                        volume,
                        prompt.trim(),
                        SENSITIVITY_PUBLIC_ONLY,
                        Some("llm_infer"),
                    )
                    .map(|receipt| receipt.response_text)
            })();
            state.llm_management_toast = result.unwrap_or_else(|reason| reason);
        }
    }
}

fn record_care_response(
    state: &mut ShellState,
    alert_id: &str,
    principal_ok: bool,
    authority: Option<&CareResponseAuthority>,
) -> Result<String, String> {
    let session = session().ok_or("A local Webizen session is required.")?;
    let authority = authority.ok_or("Authenticate a principal or scoped carer to respond.")?;
    let alert = state
        .host_stewardship
        .care_alerts
        .iter()
        .find(|alert| {
            alert.alert_id == alert_id
                && !alert.resolved_or_reversed
                && alert.severity >= CareAlertSeverity::Urgent
        })
        .ok_or("That alert is no longer active.")?;
    let responder = (authority.resolve)(alert, principal_ok)?;
    if principal_ok {
        let receipt = trigger_one_tap_principal_ok_checkin(
            &mut state.host_stewardship,
            &session.host.storage,
            alert_id,
            &responder,
            None,
        )?;
        let status = receipt.duty_of_inquiry_status.clone();
        state.mobile_avf.last_ok_checkin = Some(receipt);
        Ok(status)
    } else {
        state
            .host_stewardship
            .acknowledge_or_reverse_care_alert(
                &session.host.storage,
                alert_id,
                &responder,
                false,
                "Authenticated carer submitted duty-of-inquiry verification for this alert",
            )
            .map(|record| record.duty_of_inquiry_status)
    }
}

pub(crate) fn jump_to_stage(state: &mut ShellState, stage: VolumeStage) {
    state.current_stage = stage;
    state.active_plane = stage.primary_plane();
}

#[derive(Clone)]
struct OverlayProps {
    kind: AuxiliaryKind,
    bridge: AuxWindowBridge,
}

fn overlay_root(props: OverlayProps) -> Element {
    let OverlayProps { kind, bridge } = props;
    let window = aether_native::use_window();
    let main_app = aether_native::use_native_app();
    let id = window.id();
    let mut snapshot = use_signal(|| bridge.snapshot());
    let mut prompt = use_signal(String::new);
    let mut selected_agent = use_signal(String::new);

    use_hook({
        let bridge = bridge.clone();
        let window = Arc::clone(&window);
        move || {
            if !bridge.register(kind, window) {
                main_app.request_close(id);
            }
        }
    });
    aether::aether_core::use_drop({
        let bridge = bridge.clone();
        move || bridge.on_close(kind, id, true)
    });
    aether_native::use_window_event({
        let bridge = bridge.clone();
        move |event, _| {
            if matches!(event, WindowEvent::CloseRequested) {
                bridge.on_close(kind, id, false);
                return;
            }
            let latest = bridge.snapshot();
            let changed = *snapshot.peek() != latest;
            if changed {
                snapshot.set(latest);
            }
        }
    });

    let info = snapshot.read().clone();
    let dismiss = bridge.clone();
    let orb_title = info.current_stage.title();
    let orb_status = &info.orb_status;
    let care_enabled = info.care_authority_available;
    let care_status = &info.care_status;
    let agent_status = &info.agent_status;
    let active_agent = selected_agent.read().clone();
    let active_agent = info
        .agents
        .iter()
        .find(|(slug, _)| *slug == active_agent)
        .or_else(|| info.agents.first())
        .map(|(slug, _)| slug.clone());
    let aux_class = match kind {
        AuxiliaryKind::Orb => "aux-window aux-orb",
        AuxiliaryKind::Alert => "aux-window aux-alert",
        AuxiliaryKind::Palette => "aux-window",
    };

    rsx! {
        style { "{OVERLAY_CSS}" }
        main { class: "{aux_class}",
            {match kind {
                AuxiliaryKind::Orb => rsx! {
                    h2 { "8-Sector Command Orb" }
                    p { "{orb_title}" }
                    div { class: "aux-orb-sectors",
                        for (sector, label) in ORB_SECTORS {
                            button { key: "{label}", r#type: "button",
                                onclick: {
                                    let bridge = bridge.clone();
                                    move |_| bridge.enqueue(AuxShellCommand::RunOrbSector(sector))
                                },
                                "{label}"
                            }
                        }
                        span { class: "aux-orb-hub", aria_hidden: "true", "◉" }
                    }
                    if !orb_status.is_empty() { p { role: "status", "{orb_status}" } }
                    div { class: "aux-row",
                        for stage in [VolumeStage::Talk, VolumeStage::Browser, VolumeStage::Library, VolumeStage::Poet, VolumeStage::Wellfair] {
                            {
                                let title = stage.title();
                                rsx! {
                                    button { key: "{title}", r#type: "button",
                                        onclick: {
                                            let bridge = bridge.clone();
                                            move |_| bridge.enqueue(AuxShellCommand::JumpToStage(stage))
                                        },
                                        "{title}"
                                    }
                                }
                            }
                        }
                    }
                },
                AuxiliaryKind::Alert => rsx! {
                    h2 { "Urgent care alerts" }
                    if !care_enabled {
                        p { "Authenticate a principal or scoped carer before recording a response." }
                    }
                    if info.alerts.is_empty() {
                        p { "No urgent alerts are awaiting a response." }
                    }
                    for alert in &info.alerts {
                        {
                            let principal_bridge = bridge.clone();
                            let carer_bridge = bridge.clone();
                            let principal_alert = alert.id.clone();
                            let carer_alert = alert.id.clone();
                            let key = &alert.id;
                            let severity = alert.severity;
                            let trigger = alert.trigger;
                            rsx! {
                                article { key: "{key}",
                                    strong { "{severity}" }
                                    p { "{trigger}" }
                                    button {
                                        r#type: "button", disabled: !care_enabled,
                                        onclick: move |_| principal_bridge.enqueue(AuxShellCommand::AcknowledgeCareAlert {
                                            alert_id: principal_alert.clone(), principal_ok: true,
                                        }),
                                        "I am OK (Principal Reversible Check-In)"
                                    }
                                    button {
                                        r#type: "button", disabled: !care_enabled,
                                        onclick: move |_| carer_bridge.enqueue(AuxShellCommand::AcknowledgeCareAlert {
                                            alert_id: carer_alert.clone(), principal_ok: false,
                                        }),
                                        "Remote Carer Duty-of-Inquiry Verify"
                                    }
                                }
                            }
                        }
                    }
                    if !care_status.is_empty() { p { role: "status", "{care_status}" } }
                },
                AuxiliaryKind::Palette => rsx! {
                    h2 { "Opt-In Semantic ToolChest" }
                    for (toolbox, enabled) in info.toolboxes {
                        {
                            let spec = toolbox.spec();
                            let key = spec.slug;
                            let title = spec.title;
                            let status = if enabled { "Disable" } else { "Enable" };
                            rsx! {
                                p { key: "{key}",
                                    button {
                                        r#type: "button", aria_pressed: if enabled { "true" } else { "false" },
                                        onclick: {
                                            let bridge = bridge.clone();
                                            move |_| bridge.enqueue(AuxShellCommand::ToggleToolbox(toolbox))
                                        },
                                        "{status} {title}"
                                    }
                                }
                            }
                        }
                    }
                    h2 { "@agent Quick-Invoke (.p64 / MCP)" }
                    if let (Some(volume), Some(slug)) = (info.agent_volume, active_agent) {
                        div { class: "aux-row",
                            for (option_slug, label) in &info.agents {
                                {
                                    let option_slug = option_slug.clone();
                                    let active = option_slug == slug;
                                    let key = option_slug.clone();
                                    rsx! {
                                        button { key: "{key}", r#type: "button",
                                            aria_pressed: if active { "true" } else { "false" },
                                            onclick: move |_| selected_agent.set(option_slug.clone()),
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                        form {
                            onsubmit: {
                                let bridge = bridge.clone();
                                move |event| {
                                    event.prevent_default();
                                    let text = prompt.peek().trim().to_owned();
                                    if !text.is_empty() && text.len() <= MAX_PROMPT_BYTES {
                                        bridge.enqueue(AuxShellCommand::InvokeAgent {
                                            slug: slug.clone(), prompt: text, volume,
                                        });
                                    }
                                }
                            },
                            label { r#for: "aux-agent-prompt", "Public prompt for {volume}" }
                            input { id: "aux-agent-prompt", value: "{prompt}", maxlength: "4096",
                                oninput: move |event| prompt.set(event.value()),
                            }
                            button { r#type: "submit", "Invoke selected instrument" }
                        }
                    } else {
                        p { "Open a volume with an available agent to invoke it here." }
                    }
                    if !agent_status.is_empty() { p { role: "status", "{agent_status}" } }
                },
            }}
            button { r#type: "button",
                onclick: move |_| dismiss.enqueue(AuxShellCommand::Dismiss(kind)),
                "Dismiss window"
            }
        }
    }
}
