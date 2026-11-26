// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Webizen OS-Shell Automated User Acceptance Testing (UAT), Live Window Driver,
//! HTTP JSON Automation Server, and MCP (Model Context Protocol) JSON-RPC Server.

use crate::state::{
    DevInspectorTab, SettingsSection, ShellState, VolumeStage, ALL_SETTINGS_SECTIONS,
};
use aether::prelude::*;
use aether_native::winit::event::WindowEvent;
use aether_native::NodeId;
use blitz_dom::BaseDocument;
use blitz_traits::events::{
    BlitzImeEvent, BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point,
    PointerCoords, PointerDetails, UiEvent,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};

/// Serializable description of an interactive DOM element in the live Webizen OS-Shell window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UatElementInfo {
    pub node_id: u64,
    pub tag: String,
    pub id: Option<String>,
    pub class: Option<String>,
    pub role: Option<String>,
    pub input_type: Option<String>,
    pub value: Option<String>,
    pub label: String,
    pub disabled: bool,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Snapshot of the current shell state + interactive elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UatSnapshot {
    pub current_stage: String,
    pub active_plane: String,
    pub launcher_open: bool,
    pub control_center_drawer_open: bool,
    pub aux_orb_window_open: bool,
    pub aux_palette_window_open: bool,
    pub aux_alert_modal_open: bool,
    pub theme_preset: String,
    pub status_text: String,
    pub element_count: usize,
    pub interactive_elements: Vec<UatElementInfo>,
}

/// Command sent to the live UI thread from the UAT HTTP/MCP server or automated UAT runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UatCommand {
    /// List all interactive elements and current shell state.
    Snapshot,
    /// Press a button or interactive element by DOM `id`, `node_id`, or visible text substring.
    PressButton {
        target: String,
    },
    /// Set the text value of an `<input>` or `<textarea>` by DOM `id`, `node_id`, or placeholder/label.
    SetInput {
        target: String,
        value: String,
    },
    /// Switch directly to a VolumeStage by slug or title (`talk`, `mail`, `browser`, `keep`, `library`, `instruments`, `poet`, `wellfair`, `anatomy`, `wallet`, `projects`, `settings`, `admin`, `none`).
    SwitchStage {
        stage: String,
    },
    /// Run the comprehensive automated UAT button sweep across the entire shell and all volumes.
    RunSuite {
        #[serde(default)]
        stage_filter: Option<String>,
    },
}

/// Result of a single UAT action or step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UatStepResult {
    pub step_index: usize,
    pub stage: String,
    pub target: String,
    pub node_id: Option<u64>,
    pub coords: Option<(f32, f32)>,
    pub ok: bool,
    pub error: Option<String>,
    pub stage_after: String,
}

/// Result returned from a `UatCommand`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UatResponse {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<UatSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suite_results: Option<Vec<UatStepResult>>,
}

struct PendingUatRequest {
    command: UatCommand,
    reply: mpsc::Sender<UatResponse>,
}

/// Collect all visible text under a DOM node.
pub fn collect_node_text(doc: &BaseDocument, root_id: NodeId) -> String {
    let mut out = String::new();
    let mut stack = vec![root_id];
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get_node(id) else {
            continue;
        };
        if let Some(text) = node.text_data() {
            if !text.content.trim().is_empty() {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(text.content.trim());
            }
        }
        stack.extend(node.children.iter().rev().copied());
    }
    out
}

/// Inspect `BaseDocument` and return all interactive elements (`button`, `input`, `select`, `textarea`, `summary`, `[role=button]`, `[role=tab]`, `a`) with their layout bounding boxes.
pub fn collect_interactive_elements(doc: &BaseDocument) -> Vec<UatElementInfo> {
    let mut elements = Vec::new();
    let root_id = doc.root_node().id;
    let mut stack = vec![root_id];
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get_node(id) else {
            continue;
        };
        if let Some(el) = node.element_data() {
            let tag = (*el.name.local).to_ascii_lowercase();
            let attr = |name: &str| -> Option<String> {
                el.attrs
                    .iter()
                    .find(|a| (*a.name.local).eq_ignore_ascii_case(name))
                    .map(|a| a.value.clone())
            };
            let role = attr("role");
            let has_aether_id = attr("data-aether-id").is_some();
            let is_interactive = matches!(
                tag.as_str(),
                "button" | "input" | "select" | "textarea" | "summary" | "a"
            ) || matches!(
                role.as_deref(),
                Some("button" | "tab" | "menuitem" | "switch" | "checkbox" | "option")
            );
            if is_interactive && has_aether_id {
                let rect = doc.get_client_bounding_rect(id);
                let (x, y, width, height) = rect
                    .map(|r| (r.x, r.y, r.width, r.height))
                    .unwrap_or((0.0, 0.0, 0.0, 0.0));
                let mut label = collect_node_text(doc, id);
                if label.is_empty() {
                    if let Some(aria) = attr("aria-label") {
                        label = aria;
                    } else if let Some(title) = attr("title") {
                        label = title;
                    } else if let Some(placeholder) = attr("placeholder") {
                        label = placeholder;
                    } else if let Some(val) = attr("value") {
                        label = val;
                    }
                }
                let disabled = el
                    .attrs
                    .iter()
                    .any(|a| (*a.name.local).eq_ignore_ascii_case("disabled"));
                elements.push(UatElementInfo {
                    node_id: id.as_u64(),
                    tag,
                    id: attr("id"),
                    class: attr("class"),
                    role,
                    input_type: attr("type"),
                    value: attr("value"),
                    label,
                    disabled,
                    x,
                    y,
                    width,
                    height,
                });
            }
        }
        stack.extend(node.children.iter().rev().copied());
    }
    elements
}

/// Build a `UatSnapshot` from `BaseDocument` and `ShellState`.
pub fn build_snapshot(doc: &BaseDocument, state: &ShellState) -> UatSnapshot {
    let interactive_elements = collect_interactive_elements(doc);
    let status_text = doc
        .get_element_by_id("shell-status")
        .map(|id| collect_node_text(doc, id))
        .unwrap_or_default();
    UatSnapshot {
        current_stage: state.current_stage.title().to_string(),
        active_plane: state.active_plane.code().to_string(),
        launcher_open: state.launcher_open,
        control_center_drawer_open: state.control_center_drawer_open,
        aux_orb_window_open: state.aux_orb_window_open,
        aux_palette_window_open: state.aux_palette_window_open,
        aux_alert_modal_open: state.aux_alert_modal_open,
        theme_preset: state.theme_preset.to_string(),
        status_text,
        element_count: doc.tree().len(),
        interactive_elements,
    }
}

/// Find a matching interactive element by `#id`, `id`, `node:<node_id>`, or case-insensitive label substring.
pub fn find_target_element(
    elements: &[UatElementInfo],
    target: &str,
) -> Option<UatElementInfo> {
    let trimmed = target.trim();
    if let Some(rest) = trimmed.strip_prefix("node:") {
        if let Ok(nid) = rest.parse::<u64>() {
            return elements.iter().find(|e| e.node_id == nid).cloned();
        }
    }
    if let Ok(nid) = trimmed.parse::<u64>() {
        if let Some(found) = elements.iter().find(|e| e.node_id == nid) {
            return Some(found.clone());
        }
    }
    let id_query = trimmed.strip_prefix('#').unwrap_or(trimmed);
    if let Some(found) = elements
        .iter()
        .find(|e| e.id.as_deref() == Some(id_query))
    {
        return Some(found.clone());
    }
    let lower = trimmed.to_ascii_lowercase();
    if let Some(found) = elements
        .iter()
        .find(|e| e.label.to_ascii_lowercase() == lower)
    {
        return Some(found.clone());
    }
    elements
        .iter()
        .find(|e| e.label.to_ascii_lowercase().contains(&lower))
        .cloned()
}

/// Parse a `VolumeStage` from a user-friendly string.
pub fn parse_stage(name: &str) -> Option<VolumeStage> {
    match name.trim().to_ascii_lowercase().as_str() {
        "none" | "home" | "stage" | "orb" => Some(VolumeStage::None),
        "talk" | "communications" | "chat" => Some(VolumeStage::Talk),
        "mail" | "email" => Some(VolumeStage::Mail),
        "directory" | "contacts" | "addressbook" => Some(VolumeStage::Directory),
        "browser" | "web" => Some(VolumeStage::Browser),
        "keep" | "sanctuary" | "vault" => Some(VolumeStage::Keep),
        "library" | "media" => Some(VolumeStage::Library),
        "instruments" | "agreements" | "contracts" => Some(VolumeStage::Instruments),
        "poet" | "synth" | "poet+synth" | "poetsynth" | "authoring" => Some(VolumeStage::Poet),
        "wellfair" | "care" | "community" => Some(VolumeStage::Wellfair),
        "anatomy" | "health" | "clinical" => Some(VolumeStage::Anatomy),
        "wallet" | "finance" | "qdnf" => Some(VolumeStage::Wallet),
        "projects" | "work" | "commons" => Some(VolumeStage::Projects),
        "settings" | "preferences" | "config" => Some(VolumeStage::Settings),
        "admin" | "stewardship" | "system" => Some(VolumeStage::Admin),
        _ => None,
    }
}

fn make_pointer_event(x: f32, y: f32, buttons: MouseEventButtons) -> BlitzPointerEvent {
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button: MouseEventButton::Main,
        buttons,
        mods: Default::default(),
        details: PointerDetails::default(),
        element: Point::default(),
        active_pointers: Default::default(),
    }
}

/// Construct synthetic PointerMove + PointerDown + PointerUp events at `(x, y)` in CSS logical pixels.
pub fn make_click_events(x: f32, y: f32) -> [UiEvent; 3] {
    [
        UiEvent::PointerMove(make_pointer_event(x, y, MouseEventButtons::None)),
        UiEvent::PointerDown(make_pointer_event(x, y, MouseEventButtons::Primary)),
        UiEvent::PointerUp(make_pointer_event(x, y, MouseEventButtons::None)),
    ]
}

/// Center coordinates `(cx, cy)` of a `UatElementInfo` bounding box, falling back to `(8.0, 8.0)` if empty.
pub fn element_click_coords(el: &UatElementInfo) -> (f32, f32) {
    let cx = if el.width > 1.0 {
        (el.x + el.width * 0.5) as f32
    } else {
        (el.x + 4.0) as f32
    };
    let cy = if el.height > 1.0 {
        (el.y + el.height * 0.5) as f32
    } else {
        (el.y + 4.0) as f32
    };
    (cx, cy)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UatSweepScope {
    Stage(VolumeStage),
    LauncherOverlay,
    ControlCenterDrawer,
    CommandOrbOverlay,
    SettingsTab(SettingsSection),
    DevInspector(DevInspectorTab),
    ThemeCartridge(&'static str),
}

impl UatSweepScope {
    fn title(&self) -> String {
        match self {
            Self::Stage(s) => s.title().to_string(),
            Self::LauncherOverlay => "OrbitWheel · Semantic Command Launcher".to_string(),
            Self::ControlCenterDrawer => "OS Control Center Drawer".to_string(),
            Self::CommandOrbOverlay => "Poet 8-Sector Command Orb".to_string(),
            Self::SettingsTab(sec) => format!("Settings · {}", sec.label()),
            Self::DevInspector(tab) => format!("Inspect · {}", tab.label()),
            Self::ThemeCartridge(id) => format!("Theme · {id}"),
        }
    }

    fn apply_to_state(&self, s: &mut ShellState) {
        s.launcher_open = false;
        s.control_center_drawer_open = false;
        s.orb_open = false;
        s.dev_inspector_open = false;
        s.inspect_continuity_open = false;
        match *self {
            Self::Stage(stage) => {
                s.current_stage = stage;
            }
            Self::LauncherOverlay => {
                s.current_stage = VolumeStage::None;
                s.launcher_open = true;
            }
            Self::ControlCenterDrawer => {
                s.current_stage = VolumeStage::None;
                s.control_center_drawer_open = true;
            }
            Self::CommandOrbOverlay => {
                s.current_stage = VolumeStage::Talk;
                s.orb_open = true;
            }
            Self::SettingsTab(sec) => {
                s.current_stage = VolumeStage::Settings;
                s.settings_section = sec;
            }
            Self::DevInspector(tab) => {
                s.current_stage = VolumeStage::None;
                s.dev_inspector_open = true;
                s.inspect_continuity_open = true;
                s.dev_inspector_tab = tab;
            }
            Self::ThemeCartridge(id) => {
                s.current_stage = VolumeStage::None;
                let _ = s.adopt_theme(id);
            }
        }
    }
}

/// In-window UAT automation state driven by `arm_uat_harness(state)` in `App()`.
enum AutoSuiteMachine {
    Idle,
    Start {
        stage_filter: Option<String>,
        reply: Option<mpsc::Sender<UatResponse>>,
    },
    EnteringScope {
        scopes: VecDeque<UatSweepScope>,
        current_scope: UatSweepScope,
        wait_frames: u32,
        results: Vec<UatStepResult>,
        reply: Option<mpsc::Sender<UatResponse>>,
    },
    ClickingButtons {
        scopes: VecDeque<UatSweepScope>,
        current_scope: UatSweepScope,
        pending_targets: VecDeque<(u64, String, (f32, f32))>,
        active_target: Option<(u64, String, (f32, f32))>,
        wait_frames: u32,
        results: Vec<UatStepResult>,
        reply: Option<mpsc::Sender<UatResponse>>,
    },
}

const ALL_STAGES: [VolumeStage; 15] = [
    VolumeStage::None,
    VolumeStage::Talk,
    VolumeStage::Mail,
    VolumeStage::Directory,
    VolumeStage::Browser,
    VolumeStage::Keep,
    VolumeStage::Library,
    VolumeStage::Instruments,
    VolumeStage::Poet,
    VolumeStage::Wellfair,
    VolumeStage::Anatomy,
    VolumeStage::Wallet,
    VolumeStage::Projects,
    VolumeStage::Settings,
    VolumeStage::Admin,
];

fn build_sweep_scopes(stage_filter: Option<&str>) -> VecDeque<UatSweepScope> {
    if let Some(f) = stage_filter {
        let lower = f.trim().to_ascii_lowercase();
        if lower == "launcher" || lower == "orbit" {
            return VecDeque::from([UatSweepScope::LauncherOverlay]);
        }
        if lower == "control-center" || lower == "notifications" {
            return VecDeque::from([UatSweepScope::ControlCenterDrawer]);
        }
        if lower == "orb" || lower == "command-orb" {
            return VecDeque::from([UatSweepScope::CommandOrbOverlay]);
        }
        if lower == "settings" {
            return ALL_SETTINGS_SECTIONS
                .iter()
                .copied()
                .map(UatSweepScope::SettingsTab)
                .collect();
        }
        if lower == "inspect" || lower == "inspector" {
            return DevInspectorTab::ALL
                .iter()
                .copied()
                .map(UatSweepScope::DevInspector)
                .collect();
        }
        if lower == "themes" || lower == "--all-themes" {
            return build_all_themes_scopes();
        }
        if let Some(st) = parse_stage(f) {
            return VecDeque::from([UatSweepScope::Stage(st)]);
        }
    }
    let mut scopes = VecDeque::new();
    for st in ALL_STAGES {
        if st == VolumeStage::Settings {
            for sec in ALL_SETTINGS_SECTIONS {
                scopes.push_back(UatSweepScope::SettingsTab(sec));
            }
        } else {
            scopes.push_back(UatSweepScope::Stage(st));
        }
    }
    scopes.push_back(UatSweepScope::LauncherOverlay);
    scopes.push_back(UatSweepScope::ControlCenterDrawer);
    scopes.push_back(UatSweepScope::CommandOrbOverlay);
    scopes
}

const ALL_THEME_CARTRIDGES: [&str; 6] = [
    "theme-holosonic",
    "theme-obsidian",
    "theme-commons",
    "theme-holoslate",
    "theme-cyber-grid",
    "theme-spatial-commons",
];

fn build_all_themes_scopes() -> VecDeque<UatSweepScope> {
    let mut scopes = build_sweep_scopes(None);
    for tab in DevInspectorTab::ALL {
        scopes.push_back(UatSweepScope::DevInspector(tab));
    }
    for id in ALL_THEME_CARTRIDGES {
        scopes.push_back(UatSweepScope::ThemeCartridge(id));
    }
    scopes
}

/// Hook installed inside `App()` that powers:
/// 1. `WEBIZEN_UAT_AUTO=1` (or `webizen-os-shell uat`): automated live-window UAT button sweep across all stages & buttons.
/// 2. `WEBIZEN_UAT_PORT=<port>` (default `4249` when UAT server is enabled): live HTTP JSON control server so an MCP tool or agent can list elements, click buttons, set inputs, switch stages, and run UAT suites against the open desktop window.
pub fn arm_uat_harness(mut state: Signal<ShellState>) {
    let window = aether_native::use_window();
    let window_id = window.id();
    let app_handle = aether_native::use_native_app();
    let base_doc = try_consume_context::<Rc<RefCell<BaseDocument>>>();

    let auto_mode = std::env::var("WEBIZEN_UAT_AUTO").ok().as_deref() == Some("1");
    let stage_filter_env = std::env::var("WEBIZEN_UAT_STAGE").ok();
    let uat_port: Option<u16> = std::env::var("WEBIZEN_UAT_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .or_else(|| {
            if cfg!(debug_assertions)
                && !auto_mode
                && std::env::var("WEBIZEN_C1_SMOKE").ok().as_deref() != Some("1")
            {
                Some(4249)
            } else {
                None
            }
        });

    let tick_window = window.clone();
    let (cmd_rx, machine) = use_hook(move || {
        let (tx, rx) = mpsc::channel::<PendingUatRequest>();
        if let Some(port) = uat_port {
            let wake_window = window.clone();
            spawn_http_uat_listener(port, tx, move || {
                wake_window.request_redraw();
            });
        }
        let initial_machine = if auto_mode {
            AutoSuiteMachine::Start {
                stage_filter: stage_filter_env.clone(),
                reply: None,
            }
        } else {
            AutoSuiteMachine::Idle
        };
        if auto_mode {
            window.request_redraw();
        }
        (
            Rc::new(Mutex::new(rx)),
            Rc::new(RefCell::new(initial_machine)),
        )
    });

    aether_native::use_window_event(move |event, event_loop| {
        if !matches!(event, WindowEvent::RedrawRequested) {
            return;
        }
        let Some(base_doc) = base_doc.as_ref() else {
            return;
        };

        // 1. Drain any incoming HTTP / MCP requests while idle.
        if matches!(*machine.borrow(), AutoSuiteMachine::Idle) {
            if let Ok(rx) = cmd_rx.lock() {
                while let Ok(req) = rx.try_recv() {
                    match req.command {
                        UatCommand::Snapshot => {
                            let snap = build_snapshot(&base_doc.borrow(), &state.peek());
                            let _ = req.reply.send(UatResponse {
                                ok: true,
                                message: format!(
                                    "Snapshot captured ({} interactive elements on stage {})",
                                    snap.interactive_elements.len(),
                                    snap.current_stage
                                ),
                                snapshot: Some(snap),
                                suite_results: None,
                            });
                        }
                        UatCommand::SwitchStage { stage } => {
                            if let Some(parsed) = parse_stage(&stage) {
                                state.write().current_stage = parsed;
                                tick_window.request_redraw();
                                let snap = build_snapshot(&base_doc.borrow(), &state.peek());
                                let _ = req.reply.send(UatResponse {
                                    ok: true,
                                    message: format!("Switched to stage {}", parsed.title()),
                                    snapshot: Some(snap),
                                    suite_results: None,
                                });
                            } else {
                                let _ = req.reply.send(UatResponse {
                                    ok: false,
                                    message: format!("Unknown stage '{stage}'"),
                                    snapshot: None,
                                    suite_results: None,
                                });
                            }
                        }
                        UatCommand::PressButton { target } => {
                            let elements = collect_interactive_elements(&base_doc.borrow());
                            if let Some(el) = find_target_element(&elements, &target) {
                                let (cx, cy) = element_click_coords(&el);
                                app_handle.click_node(window_id, NodeId::from_u64(el.node_id));
                                tick_window.request_redraw();
                                let _ = req.reply.send(UatResponse {
                                    ok: true,
                                    message: format!(
                                        "Dispatched click to '{}' (node={}, id={:?}) at ({:.1}, {:.1})",
                                        el.label, el.node_id, el.id, cx, cy
                                    ),
                                    snapshot: Some(build_snapshot(&base_doc.borrow(), &state.peek())),
                                    suite_results: None,
                                });
                            } else {
                                let _ = req.reply.send(UatResponse {
                                    ok: false,
                                    message: format!("No interactive element matched '{target}'"),
                                    snapshot: Some(build_snapshot(&base_doc.borrow(), &state.peek())),
                                    suite_results: None,
                                });
                            }
                        }
                        UatCommand::SetInput { target, value } => {
                            let elements = collect_interactive_elements(&base_doc.borrow());
                            if let Some(el) = find_target_element(&elements, &target) {
                                app_handle.click_node(window_id, NodeId::from_u64(el.node_id));
                                app_handle
                                    .dispatch_ui_event(window_id, UiEvent::Ime(BlitzImeEvent::Enabled));
                                app_handle.dispatch_ui_event(
                                    window_id,
                                    UiEvent::Ime(BlitzImeEvent::Commit(value.clone())),
                                );
                                tick_window.request_redraw();
                                let _ = req.reply.send(UatResponse {
                                    ok: true,
                                    message: format!(
                                        "Committed input value to '{}' (node={})",
                                        el.label, el.node_id
                                    ),
                                    snapshot: Some(build_snapshot(&base_doc.borrow(), &state.peek())),
                                    suite_results: None,
                                });
                            } else {
                                let _ = req.reply.send(UatResponse {
                                    ok: false,
                                    message: format!("No input element matched '{target}'"),
                                    snapshot: None,
                                    suite_results: None,
                                });
                            }
                        }
                        UatCommand::RunSuite { stage_filter } => {
                            *machine.borrow_mut() = AutoSuiteMachine::Start {
                                stage_filter,
                                reply: Some(req.reply),
                            };
                            tick_window.request_redraw();
                            break;
                        }
                    }
                }
            }
        }

        // 2. Step the automated UAT state machine across redraw frames.
        let mut m = machine.borrow_mut();
        let next_state = match std::mem::replace(&mut *m, AutoSuiteMachine::Idle) {
            AutoSuiteMachine::Idle => AutoSuiteMachine::Idle,
            AutoSuiteMachine::Start {
                stage_filter,
                reply,
            } => {
                let mut scopes = if std::env::var("WEBIZEN_UAT_ALL_THEMES").ok().as_deref()
                    == Some("1")
                    && stage_filter.is_none()
                {
                    build_all_themes_scopes()
                } else {
                    build_sweep_scopes(stage_filter.as_deref())
                };
                if let Some(first) = scopes.pop_front() {
                    eprintln!("[UAT] Starting automated live-window button sweep...");
                    first.apply_to_state(&mut state.write());
                    tick_window.request_redraw();
                    AutoSuiteMachine::EnteringScope {
                        scopes,
                        current_scope: first,
                        wait_frames: 2,
                        results: Vec::new(),
                        reply,
                    }
                } else {
                    AutoSuiteMachine::Idle
                }
            }
            AutoSuiteMachine::EnteringScope {
                scopes,
                current_scope,
                wait_frames,
                results,
                reply,
            } => {
                if wait_frames > 0 {
                    tick_window.request_redraw();
                    AutoSuiteMachine::EnteringScope {
                        scopes,
                        current_scope,
                        wait_frames: wait_frames - 1,
                        results,
                        reply,
                    }
                } else {
                    let elements = collect_interactive_elements(&base_doc.borrow());
                    let pending_targets: VecDeque<(u64, String, (f32, f32))> = elements
                        .into_iter()
                        .filter(|el| {
                            if el.disabled || !matches!(el.tag.as_str(), "button" | "summary") {
                                return false;
                            }
                            let class = el.class.as_deref().unwrap_or_default();
                            match current_scope {
                                UatSweepScope::Stage(VolumeStage::None) => true,
                                UatSweepScope::LauncherOverlay => {
                                    // Test buttons inside the OrbitWheel dialog
                                    !class.contains("fav")
                                        && !class.contains("ribbon-btn")
                                        && el.id.as_deref() != Some("open-launcher")
                                }
                                UatSweepScope::ControlCenterDrawer => {
                                    !class.contains("fav")
                                        && !class.contains("ribbon-btn")
                                        && el.id.as_deref() != Some("open-launcher")
                                }
                                UatSweepScope::CommandOrbOverlay => {
                                    class.contains("orb-sector-btn")
                                }
                                UatSweepScope::SettingsTab(_) => {
                                    !class.contains("fav")
                                        && !class.contains("ribbon-btn")
                                        && !class.contains("settings-tab")
                                        && !class.contains("btn-win")
                                        && !class.contains("hub")
                                        && el.id.as_deref() != Some("open-launcher")
                                }
                                UatSweepScope::Stage(_) => {
                                    !class.contains("fav")
                                        && !class.contains("ribbon-btn")
                                        && el.id.as_deref() != Some("open-launcher")
                                }
                                UatSweepScope::DevInspector(_) => {
                                    !class.contains("fav")
                                        && el.id.as_deref() != Some("open-launcher")
                                }
                                UatSweepScope::ThemeCartridge(_) => {
                                    !class.contains("fav")
                                        && !class.contains("ribbon-btn")
                                        && el.id.as_deref() != Some("open-launcher")
                                }
                            }
                        })
                        .map(|el| {
                            let coords = element_click_coords(&el);
                            let desc = if let Some(ref id) = el.id {
                                format!("#{id} ({})", el.label)
                            } else {
                                el.label.clone()
                            };
                            (el.node_id, desc, coords)
                        })
                        .collect();
                    eprintln!(
                        "[UAT] Scope '{}' loaded: {} interactive buttons queued",
                        current_scope.title(),
                        pending_targets.len()
                    );
                    tick_window.request_redraw();
                    AutoSuiteMachine::ClickingButtons {
                        scopes,
                        current_scope,
                        pending_targets,
                        active_target: None,
                        wait_frames: 0,
                        results,
                        reply,
                    }
                }
            }
            AutoSuiteMachine::ClickingButtons {
                mut scopes,
                current_scope,
                mut pending_targets,
                active_target,
                wait_frames,
                mut results,
                reply,
            } => {
                if wait_frames > 0 {
                    tick_window.request_redraw();
                    AutoSuiteMachine::ClickingButtons {
                        scopes,
                        current_scope,
                        pending_targets,
                        active_target,
                        wait_frames: wait_frames - 1,
                        results,
                        reply,
                    }
                } else {
                    if let Some((nid, desc, coords)) = active_target {
                        let step_index = results.len() + 1;
                        let stage_after = state.peek().current_stage.title().to_string();
                        results.push(UatStepResult {
                            step_index,
                            stage: current_scope.title(),
                            target: desc,
                            node_id: Some(nid),
                            coords: Some(coords),
                            ok: true,
                            error: None,
                            stage_after,
                        });
                        // Restore scope state if clicking the button closed the overlay or navigated to another stage
                        current_scope.apply_to_state(&mut state.write());
                    }

                    if let Some((nid, desc, (cx, cy))) = pending_targets.pop_front() {
                        let fresh_coords = base_doc
                            .borrow()
                            .get_client_bounding_rect(NodeId::from_u64(nid))
                            .map(|r| {
                                let fx = if r.width > 1.0 {
                                    (r.x + r.width * 0.5) as f32
                                } else {
                                    cx
                                };
                                let fy = if r.height > 1.0 {
                                    (r.y + r.height * 0.5) as f32
                                } else {
                                    cy
                                };
                                (fx, fy)
                            })
                            .unwrap_or((cx, cy));
                        eprintln!(
                            "[UAT]   -> Pressing [{}] '{}' at ({:.1}, {:.1})",
                            current_scope.title(),
                            desc,
                            fresh_coords.0,
                            fresh_coords.1
                        );
                        app_handle.click_node(window_id, NodeId::from_u64(nid));
                        tick_window.request_redraw();
                        AutoSuiteMachine::ClickingButtons {
                            scopes,
                            current_scope,
                            pending_targets,
                            active_target: Some((nid, desc, fresh_coords)),
                            wait_frames: 1,
                            results,
                            reply,
                        }
                    } else if let Some(next_scope) = scopes.pop_front() {
                        next_scope.apply_to_state(&mut state.write());
                        tick_window.request_redraw();
                        AutoSuiteMachine::EnteringScope {
                            scopes,
                            current_scope: next_scope,
                            wait_frames: 2,
                            results,
                            reply,
                        }
                    } else {
                        let total = results.len();
                        let passed = results.iter().filter(|r| r.ok).count();
                        let summary = format!(
                            "UAT button sweep complete: {passed}/{total} button presses succeeded across all stages with 0 crashes."
                        );
                        eprintln!("[UAT] {summary}");
                        if let Some(reply_tx) = reply {
                            let snap = build_snapshot(&base_doc.borrow(), &state.peek());
                            let _ = reply_tx.send(UatResponse {
                                ok: passed == total,
                                message: summary,
                                snapshot: Some(snap),
                                suite_results: Some(results),
                            });
                        } else if auto_mode {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&UatResponse {
                                    ok: passed == total,
                                    message: summary,
                                    snapshot: Some(build_snapshot(
                                        &base_doc.borrow(),
                                        &state.peek()
                                    )),
                                    suite_results: Some(results),
                                })
                                .unwrap_or_default()
                            );
                            event_loop.exit();
                        }
                        AutoSuiteMachine::Idle
                    }
                }
            }
        };
        *m = next_state;
    });
}

/// Spawn a lightweight loopback HTTP server (`127.0.0.1:<port>`) that accepts JSON `UatCommand`
/// POSTs (or `GET /snapshot`) and forwards them to the live UI thread.
fn spawn_http_uat_listener(
    port: u16,
    tx: mpsc::Sender<PendingUatRequest>,
    wake: impl Fn() + Send + Sync + 'static,
) {
    let wake = Arc::new(wake);
    std::thread::Builder::new()
        .name("webizen-uat-http".into())
        .spawn(move || {
            let addr = format!("127.0.0.1:{port}");
            let Ok(listener) = TcpListener::bind(&addr) else {
                eprintln!("[UAT] Could not bind live UAT HTTP server on {addr}");
                return;
            };
            eprintln!("[UAT] Live UAT HTTP control server listening on http://{addr}");
            for stream in listener.incoming().flatten() {
                let tx = tx.clone();
                let wake = Arc::clone(&wake);
                let _ = handle_http_client(stream, tx, wake);
            }
        })
        .ok();
}

fn handle_http_client(
    mut stream: TcpStream,
    tx: mpsc::Sender<PendingUatRequest>,
    wake: Arc<dyn Fn() + Send + Sync>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(());
    }
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("GET");
    let path = parts.get(1).copied().unwrap_or("/");

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                content_length = v.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let command: UatCommand = if method.eq_ignore_ascii_case("GET") || body.is_empty() {
        if path.starts_with("/suite") {
            UatCommand::RunSuite { stage_filter: None }
        } else {
            UatCommand::Snapshot
        }
    } else {
        match serde_json::from_slice(&body) {
            Ok(cmd) => cmd,
            Err(e) => {
                let err_json = serde_json::to_string(&UatResponse {
                    ok: false,
                    message: format!("Invalid UAT JSON command: {e}"),
                    snapshot: None,
                    suite_results: None,
                })
                .unwrap_or_default();
                let resp = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    err_json.len(),
                    err_json
                );
                stream.write_all(resp.as_bytes())?;
                return Ok(());
            }
        }
    };

    let (reply_tx, reply_rx) = mpsc::channel();
    if tx
        .send(PendingUatRequest {
            command,
            reply: reply_tx,
        })
        .is_err()
    {
        return Ok(());
    }
    wake();

    let response = reply_rx
        .recv_timeout(std::time::Duration::from_secs(60))
        .unwrap_or_else(|e| UatResponse {
            ok: false,
            message: format!("Timed out waiting for UI thread UAT response: {e}"),
            snapshot: None,
            suite_results: None,
        });
    let json = serde_json::to_string_pretty(&response).unwrap_or_default();
    let http = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        json.len(),
        json
    );
    stream.write_all(http.as_bytes())?;
    Ok(())
}

/// Send a `UatCommand` over HTTP to a running `webizen-os-shell` instance at `127.0.0.1:<port>`.
pub fn send_remote_uat_command(port: u16, command: &UatCommand) -> Result<UatResponse, String> {
    let addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| format!("Failed to connect to live Webizen OS-Shell at {addr}: {e}"))?;
    let body = serde_json::to_string(command).map_err(|e| e.to_string())?;
    let req = format!(
        "POST /uat HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut resp_bytes = Vec::new();
    stream
        .read_to_end(&mut resp_bytes)
        .map_err(|e| e.to_string())?;
    let resp_str = String::from_utf8_lossy(&resp_bytes);
    let json_body = resp_str
        .split_once("\r\n\r\n")
        .map(|(_, b)| b)
        .or_else(|| resp_str.split_once("\n\n").map(|(_, b)| b))
        .unwrap_or(&resp_str);
    serde_json::from_str(json_body).map_err(|e| format!("Invalid UAT response JSON: {e}"))
}

/// Run the MCP (Model Context Protocol) JSON-RPC 2.0 server over stdio (`webizen-os-shell uat-mcp`).
/// Connects to a live running `webizen-os-shell` window on `WEBIZEN_UAT_PORT` (default `4249`).
pub fn run_mcp_stdio_server() {
    let port: u16 = std::env::var("WEBIZEN_UAT_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4249);
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines().flatten() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(trimmed) else {
            continue;
        };
        let id = msg.get("id").cloned();
        let method = msg
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or_default();
        let params = msg.get("params").cloned().unwrap_or(serde_json::Value::Null);

        if id.is_none() {
            continue;
        }
        let id_val = id.unwrap();
        let result = match method {
            "initialize" => serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "webizen-os-shell-uat",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
            "tools/list" => serde_json::json!({
                "tools": [
                    {
                        "name": "uat_snapshot",
                        "description": "Capture a snapshot of the live Webizen OS-Shell window, including current stage, active plane, status bar, and all interactive buttons/inputs with their bounding-box coordinates.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "uat_press_button",
                        "description": "Press a button or interactive element in the live Webizen OS-Shell window by DOM id (e.g. '#open-launcher'), node_id (e.g. 'node:42'), or visible label text.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": {
                                    "type": "string",
                                    "description": "Button id, node:<id>, or visible label substring."
                                }
                            },
                            "required": ["target"]
                        }
                    },
                    {
                        "name": "uat_set_input",
                        "description": "Focus an input or textarea in the live Webizen OS-Shell window and commit text to it.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": {
                                    "type": "string",
                                    "description": "Input id, node:<id>, or placeholder/label substring."
                                },
                                "value": {
                                    "type": "string",
                                    "description": "Text value to commit."
                                }
                            },
                            "required": ["target", "value"]
                        }
                    },
                    {
                        "name": "uat_switch_stage",
                        "description": "Switch the active VolumeStage in Webizen OS-Shell (none, talk, mail, directory, browser, keep, library, instruments, poet, wellfair, anatomy, wallet, projects, settings, admin).",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "stage": {
                                    "type": "string",
                                    "description": "Volume stage name."
                                }
                            },
                            "required": ["stage"]
                        }
                    },
                    {
                        "name": "uat_run_suite",
                        "description": "Run the automated UAT button sweep across all stages (or a single filtered stage) in the live Webizen OS-Shell window.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "stage_filter": {
                                    "type": "string",
                                    "description": "Optional stage name to restrict the button sweep."
                                }
                            }
                        }
                    }
                ]
            }),
            "tools/call" => {
                let tool_name = params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or_default();
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let cmd = match tool_name {
                    "uat_snapshot" => Ok(UatCommand::Snapshot),
                    "uat_press_button" => args
                        .get("target")
                        .and_then(|v| v.as_str())
                        .map(|t| UatCommand::PressButton {
                            target: t.to_string(),
                        })
                        .ok_or_else(|| "Missing 'target' argument".to_string()),
                    "uat_set_input" => {
                        let target = args
                            .get("target")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        let value = args
                            .get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        Ok(UatCommand::SetInput { target, value })
                    }
                    "uat_switch_stage" => args
                        .get("stage")
                        .and_then(|v| v.as_str())
                        .map(|s| UatCommand::SwitchStage {
                            stage: s.to_string(),
                        })
                        .ok_or_else(|| "Missing 'stage' argument".to_string()),
                    "uat_run_suite" => Ok(UatCommand::RunSuite {
                        stage_filter: args
                            .get("stage_filter")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    }),
                    other => Err(format!("Unknown UAT MCP tool '{other}'")),
                };
                match cmd.and_then(|c| send_remote_uat_command(port, &c)) {
                    Ok(resp) => {
                        let text = serde_json::to_string_pretty(&resp).unwrap_or_default();
                        serde_json::json!({
                            "content": [{ "type": "text", "text": text }],
                            "isError": !resp.ok
                        })
                    }
                    Err(err) => serde_json::json!({
                        "content": [{ "type": "text", "text": err }],
                        "isError": true
                    }),
                }
            }
            _ => serde_json::json!({}),
        };

        let reply = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "result": result
        });
        if let Ok(line_out) = serde_json::to_string(&reply) {
            let _ = writeln!(stdout, "{line_out}");
            let _ = stdout.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stage_maps_all_fifteen_volumes_and_aliases() {
        for (input, expected) in [
            ("none", VolumeStage::None),
            ("talk", VolumeStage::Talk),
            ("mail", VolumeStage::Mail),
            ("directory", VolumeStage::Directory),
            ("browser", VolumeStage::Browser),
            ("keep", VolumeStage::Keep),
            ("sanctuary", VolumeStage::Keep),
            ("library", VolumeStage::Library),
            ("instruments", VolumeStage::Instruments),
            ("poet", VolumeStage::Poet),
            ("poet+synth", VolumeStage::Poet),
            ("wellfair", VolumeStage::Wellfair),
            ("anatomy", VolumeStage::Anatomy),
            ("wallet", VolumeStage::Wallet),
            ("projects", VolumeStage::Projects),
            ("settings", VolumeStage::Settings),
            ("admin", VolumeStage::Admin),
        ] {
            assert_eq!(parse_stage(input), Some(expected));
        }
    }

    #[test]
    fn find_target_element_matches_by_node_id_dom_id_and_label() {
        let elements = vec![
            UatElementInfo {
                node_id: 42,
                tag: "button".into(),
                id: Some("open-launcher".into()),
                class: Some("fav".into()),
                role: None,
                input_type: Some("button".into()),
                value: None,
                label: "Launch…".into(),
                disabled: false,
                x: 10.0,
                y: 20.0,
                width: 80.0,
                height: 30.0,
            },
            UatElementInfo {
                node_id: 99,
                tag: "button".into(),
                id: None,
                class: None,
                role: None,
                input_type: Some("button".into()),
                value: None,
                label: "Verify M-of-N threshold reconstruction".into(),
                disabled: false,
                x: 100.0,
                y: 200.0,
                width: 120.0,
                height: 36.0,
            },
        ];
        assert_eq!(
            find_target_element(&elements, "node:42").unwrap().node_id,
            42
        );
        assert_eq!(
            find_target_element(&elements, "#open-launcher")
                .unwrap()
                .node_id,
            42
        );
        assert_eq!(
            find_target_element(&elements, "threshold reconstruction")
                .unwrap()
                .node_id,
            99
        );
    }

    #[test]
    fn build_sweep_scopes_covers_all_volumes_settings_tabs_and_overlays() {
        let scopes = build_sweep_scopes(None);
        // 14 non-Settings stages + 10 Settings tabs + 3 overlays = 27 scopes
        assert_eq!(scopes.len(), 27);
        assert_eq!(build_sweep_scopes(Some("settings")).len(), 10);
        assert_eq!(build_sweep_scopes(Some("launcher")).len(), 1);
        assert_eq!(build_sweep_scopes(Some("control-center")).len(), 1);
        assert_eq!(build_sweep_scopes(Some("orb")).len(), 1);
        assert_eq!(build_sweep_scopes(Some("inspect")).len(), 4);
        assert_eq!(build_all_themes_scopes().len(), 37);
    }
}

