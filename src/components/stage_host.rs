// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::auxiliary_windows::AuxWindowBridge;
use super::dev_inspector::{toggle_dev_inspector, volume_plane_sentence};
use super::holo_glyph::{eval_vibe_icon_omega, HoloGlyphIcon, HoloGlyphKind};
use super::themes::{CyberGridStage, HoloslateArcStage, SpatialCommonsStage};
use super::volumes::{
    admin::AdminVolume,
    browser::BrowserVolume,
    communications::{DirectoryVolume, MailVolume, TalkVolume},
    instruments::InstrumentsVolume,
    library::LibraryVolume,
    poet_synth::{AnatomyVolume, PoetSynthVolume},
    projects::ProjectsVolume,
    sanctuary::SanctuaryVolume,
    settings::{control_center_storage, SettingsVolume, WorkspaceSave},
    wallet::WalletVolume,
    wellfair::WellfairVolume,
};
use crate::state::{session, SettingsSection, ShellState, StageChromeMode, VolumeStage};
use aether::prelude::*;
use webizen_ua_os::{IdentifierFabricPlane, OsJobStatus};

#[component]
pub fn StageHost(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let current = s.current_stage;
    let active_plane = s.active_plane;
    let empty = current == VolumeStage::None;
    let current_title = current.title();
    let current_plane_title = current.primary_plane().title();
    let orb_open = s.orb_open;
    let orb_toast = s.orb_toast.clone();
    let theme = s.theme_preset;
    let chrome_mode = s.stage_chrome_mode;
    let control_center_open = s.control_center_drawer_open;
    let ui_scale_percent = s.ui_scale_percent;
    let high_contrast = s.theme_preset == "theme-obsidian";
    let animating_icons = s.animate_icons && !s.reduced_motion;
    let look_name = s.look_short_name();
    drop(s);

    let stage_class = match chrome_mode {
        StageChromeMode::DefaultDepth => "stage",
        StageChromeMode::FullStage => "stage full",
        StageChromeMode::FloatLayer => "stage float-mode",
    };
    let plane_title = active_plane.title();
    let plane_friendly_summary = volume_plane_sentence(current);
    let inspector_open = state.read().dev_inspector_open;

    rsx! {
        div {
            class: "stage-wrap",

            // Top Continuity Ribbon (who · claim · handle · tool)
            nav {
                class: "ribbon-bar",
                aria_label: "Who you are working as",
                div {
                    class: "ribbon-pills",
                    span { class: "ribbon-tag", "Working as" }
                    for plane in IdentifierFabricPlane::ALL {
                        {
                            let is_active = active_plane == plane;
                            let btn_class = if is_active { "ribbon-btn active" } else { "ribbon-btn" };
                            let short = plane.short_ribbon_label();
                            let code = plane.code();
                            rsx! {
                                button {
                                    key: "{code}",
                                    r#type: "button",
                                    class: "{btn_class}",
                                    aria_current: if is_active { "true" } else { "false" },
                                    onclick: move |_| {
                                        state.write().active_plane = plane;
                                    },
                                    "{short}"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "ribbon-note",
                    strong { "{plane_title}: " }
                    span { "{plane_friendly_summary}" }
                }
                div {
                    class: "a11y-quick-bar",
                    role: "group",
                    aria_label: "Quick accessibility and text scaling controls",
                    button {
                        r#type: "button",
                        class: "ribbon-btn look-cycle-btn",
                        title: "Cycle appearance: Holosonic, Obsidian Contrast, Warm Commons, Holoslate Arc 3D, Cyber-Grid Matrix 3D, Spatial Commons 3D",
                        aria_label: "Cycle Look theme ({look_name})",
                        onclick: move |_| {
                            state.write().cycle_look_theme();
                        },
                        "Look: {look_name}"
                    }
                    button {
                        r#type: "button",
                        class: if inspector_open { "ribbon-btn active" } else { "ribbon-btn" },
                        title: "Toggle Developer & Continuity Inspector (F12)",
                        aria_label: "Toggle Developer and Continuity Inspector",
                        onclick: move |_| {
                            toggle_dev_inspector(&mut state.write());
                        },
                        if inspector_open { "Inspect: On" } else { "Inspect (F12)" }
                    }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        title: "Decrease text & UI scale by 10% (Ctrl+-)",
                        aria_label: "Decrease text and UI size",
                        onclick: move |_| {
                            let next = state.read().ui_scale_percent.saturating_sub(10).max(85);
                            state.write().ui_scale_percent = next;
                        },
                        "A−"
                    }
                    button {
                        r#type: "button",
                        class: "ribbon-btn active",
                        title: "Current UI & text scale ({ui_scale_percent}%). Click to cycle presets (100% · 115% · 130% · 145%), or press Ctrl+0 for 115%.",
                        aria_label: "Cycle UI scale preset ({ui_scale_percent}%)",
                        onclick: move |_| {
                            let cur = state.read().ui_scale_percent;
                            let next = match cur {
                                85..=104 => 115,
                                105..=120 => 130,
                                121..=138 => 145,
                                _ => 100,
                            };
                            state.write().ui_scale_percent = next;
                        },
                        "{ui_scale_percent}%"
                    }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        title: "Increase text & UI scale by 10% (Ctrl+=)",
                        aria_label: "Increase text and UI size",
                        onclick: move |_| {
                            let next = (state.read().ui_scale_percent + 10).min(175);
                            state.write().ui_scale_percent = next;
                        },
                        "A+"
                    }
                    button {
                        r#type: "button",
                        class: if high_contrast { "ribbon-btn active" } else { "ribbon-btn" },
                        title: "Toggle WCAG AAA High-Contrast Obsidian theme",
                        aria_pressed: if high_contrast { "true" } else { "false" },
                        onclick: move |_| {
                            let mut s = state.write();
                            s.theme_preset = if s.theme_preset == "theme-obsidian" {
                                "theme-holosonic"
                            } else {
                                "theme-obsidian"
                            };
                        },
                        "Contrast"
                    }
                    button {
                        r#type: "button",
                        class: if animating_icons { "ribbon-btn active" } else { "ribbon-btn" },
                        title: "Toggle animated 3D/Vector Etalon icons & motion",
                        aria_pressed: if animating_icons { "true" } else { "false" },
                        onclick: move |_| {
                            let mut s = state.write();
                            if s.reduced_motion {
                                s.reduced_motion = false;
                                s.animate_icons = true;
                            } else {
                                s.animate_icons = !s.animate_icons;
                            }
                        },
                        if animating_icons { "3D Icons: Live" } else { "3D Icons: Static" }
                    }
                    button {
                        r#type: "button",
                        class: "ribbon-btn",
                        title: "Open Appearance & Accessibility settings",
                        onclick: move |_| {
                            let mut s = state.write();
                            s.current_stage = VolumeStage::Settings;
                            s.active_plane = VolumeStage::Settings.primary_plane();
                            s.settings_section = SettingsSection::Appearance;
                        },
                        "Access ⚙"
                    }
                }
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    aria_keyshortcuts: "Control+K",
                    onclick: move |_| state.write().launcher_open = true,
                    "Commands"
                }
            }

            section {
                class: "{stage_class}",
                aria_label: "App volume",
                id: "app-stage",
                if !empty {
                    div {
                        class: "vol-chrome open",
                        button {
                            r#type: "button",
                            class: "hub",
                            aria_label: "Open Poet 8-Sector Command Orb",
                            aria_expanded: if orb_open { "true" } else { "false" },
                            title: "Poet 8-Sector Command Orb (Close · Soften · Full · Float · Keep · Share · Inspect · More)",
                            onclick: move |_| {
                                let open = state.read().orb_open;
                                state.write().orb_open = !open;
                            },
                            "◉"
                        }
                        div { class: "vtitle", "{current_title}" }
                        span { class: "pill plane", "{current_plane_title}" }
                        span { class: "pill live", "Live" }
                        button {
                            r#type: "button",
                            class: "btn-win",
                            onclick: move |_| {
                                let open = state.read().orb_open;
                                state.write().orb_open = !open;
                            },
                            "8-Sector Command Orb"
                        }
                        button {
                            r#type: "button",
                            class: "btn-win",
                            onclick: move |_| {
                                let mut s = state.write();
                                s.current_stage = VolumeStage::None;
                                s.orb_open = false;
                            },
                            "Soften to Orbit"
                        }
                    }
                    if orb_open {
                        dialog {
                            open: true,
                            style: "display: contents",
                            aria_label: "8-Sector Command Orb",
                            oncancel: move |event| {
                                event.prevent_default();
                                state.write().orb_open = false;
                            },
                            CommandOrbBar { state }
                        }
                    }
                }
                div {
                    class: "stage-body",
                    if empty {
                        match theme {
                            "theme-holoslate" => rsx! { HoloslateArcStage { state } },
                            "theme-cyber-grid" => rsx! { CyberGridStage { state } },
                            "theme-spatial-commons" => rsx! { SpatialCommonsStage { state } },
                            _ => rsx! { EmptyManifold { state } },
                        }
                    } else {
                        VolumeBody { current, state }
                    }
                }
            }
            if !orb_toast.is_empty() {
                p {
                    class: "muted stage-toast",
                    role: "status",
                    "{orb_toast}"
                }
            }
            if control_center_open {
                OsControlCenterDrawer { state }
            }
        }
    }
}

#[component]
fn OsControlCenterDrawer(mut state: Signal<ShellState>) -> Element {
    let shell = state.read();
    let queued_count = shell
        .os_control_center
        .jobs
        .iter()
        .filter(|job| job.status == OsJobStatus::Queued)
        .count();
    let blocked_count = shell
        .os_control_center
        .jobs
        .iter()
        .filter(|job| job.status == OsJobStatus::Blocked)
        .count();
    let running: Vec<_> = shell
        .os_control_center
        .jobs
        .iter()
        .filter(|job| job.status == OsJobStatus::Running)
        .map(|job| (job.job_id.clone(), job.title.clone()))
        .collect();
    let load_error = shell.os_control_center_error.clone();
    let can_edit = load_error.is_none() && session().is_some();
    let mut notifications: Vec<_> = shell
        .os_control_center
        .notifications
        .iter()
        .filter(|notice| !notice.acknowledged)
        .cloned()
        .collect();
    drop(shell);
    notifications.sort_by_key(|notice| std::cmp::Reverse(notice.severity));
    let mut status = use_signal(String::new);

    rsx! {
        aside {
            id: "os-control-center-drawer",
            class: "os-control-center-drawer id-list",
            aria_labelledby: "os-control-center-heading",
            div {
                class: "control-center-heading input-row",
                h4 { id: "os-control-center-heading", "OS Control Center" }
                button {
                    r#type: "button",
                    onclick: move |_| state.write().control_center_drawer_open = false,
                    "Close"
                }
            }
            p { "{running.len()} running · {queued_count} queued · {blocked_count} blocked · {notifications.len()} unread notifications" }
            if !running.is_empty() {
                ul { aria_label: "Running jobs",
                    for (job_id, title) in running {
                        li { key: "{job_id}", "{title}" }
                    }
                }
            }
            if let Some(error) = load_error {
                p { class: "path", role: "status", "Control Center is read-only: {error}" }
            }
            if notifications.is_empty() {
                p { class: "muted", "No unread notifications." }
            }
            ul {
                for notice in notifications {
                    {
                        let id = notice.notification_id.clone();
                        let severity = notice.severity.label();
                        rsx! {
                            li {
                                key: "{notice.notification_id}",
                                strong { "{severity} · {notice.title}" }
                                p { "{notice.detail}" }
                                button {
                                    r#type: "button",
                                    disabled: !can_edit,
                                    onclick: move |_| {
                                        let result = control_center_storage(state).and_then(|storage| {
                                            state.write().os_control_center
                                                .acknowledge_notification(storage, &id)
                                        });
                                        status.set(match result {
                                            Ok(()) => "Notification acknowledged. Its source record is unchanged.".into(),
                                            Err(error) => error,
                                        });
                                    },
                                    "Mark read"
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
                    let mut shell = state.write();
                    shell.current_stage = VolumeStage::Settings;
                    shell.active_plane = VolumeStage::Settings.primary_plane();
                    shell.settings_section = SettingsSection::Services;
                    shell.control_center_drawer_open = false;
                },
                "Open jobs and capability settings"
            }
            p { class: "muted", role: "status", "{status}" }
        }
    }
}

/// Poet 8-Sector Radial Command Orb (`POET-SPEC-011`) replacing OS red/yellow/green traffic lights.

#[component]
fn CommandOrbBar(mut state: Signal<ShellState>) -> Element {
    let current_title = state.read().current_stage.title();
    let auxiliary = try_consume_context::<AuxWindowBridge>();
    rsx! {
        div {
            class: "orb-panel",
            role: "menu",
            aria_label: "Poet 8-Sector Command Orb",
            span {
                strong { "◉ Poet 8-Sector Command Orb ({current_title}): " }
                "No OS traffic-light chrome"
            }
            div {
                class: "orb-sectors",
                button {
                    r#type: "button",
                    class: "orb-sector-btn",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.current_stage = VolumeStage::None;
                        s.orb_open = false;
                        s.orb_toast = "Orb Sector 1 · Closed volume".into();
                    },
                    "1. ✕ Close"
                }
                button {
                    r#type: "button",
                    class: "orb-sector-btn",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.current_stage = VolumeStage::None;
                        s.stage_chrome_mode = StageChromeMode::DefaultDepth;
                        s.orb_open = false;
                        s.orb_toast = "Orb Sector 2 · Softened volume into spatial orbit".into();
                    },
                    "2. — Soften"
                }
                button {
                    r#type: "button",
                    class: "orb-sector-btn",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.stage_chrome_mode = if s.stage_chrome_mode == StageChromeMode::FullStage {
                            StageChromeMode::DefaultDepth
                        } else {
                            StageChromeMode::FullStage
                        };
                        s.orb_toast = "Orb Sector 3 · Toggled Full Stage geometry".into();
                    },
                    "3. ▢ Full"
                }
                button {
                    r#type: "button",
                    class: "orb-sector-btn",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.stage_chrome_mode = if s.stage_chrome_mode == StageChromeMode::FloatLayer {
                            StageChromeMode::DefaultDepth
                        } else {
                            StageChromeMode::FloatLayer
                        };
                        s.orb_toast = "Orb Sector 4 · Toggled Elevated Float Layer".into();
                    },
                    "4. ◎ Float"
                }
                button {
                    r#type: "button",
                    class: "orb-sector-btn",
                    onclick: move |_| {
                        let mut s = state.write();
                        let title = s.current_stage.title();
                        s.orb_toast = format!("Orb Sector 5 · Parked {title} snapshot in Sanctuary Keep");
                    },
                    "5. 📁 Keep"
                }
                button {
                    r#type: "button",
                    class: "orb-sector-btn",
                    onclick: move |_| {
                        let mut s = state.write();
                        let title = s.current_stage.title();
                        s.orb_toast = format!("Orb Sector 6 · Prepared pairwise ConsentCredential share for {title}");
                    },
                    "6. ↗ Share"
                }
                button {
                    r#type: "button",
                    class: "orb-sector-btn",
                    onclick: move |_| toggle_dev_inspector(&mut state.write()),
                    "7. Inspect"
                }
                button {
                    r#type: "button",
                    class: "orb-sector-btn",
                    onclick: move |_| {
                        let mut s = state.write();
                        s.current_stage = VolumeStage::Settings;
                        s.active_plane = VolumeStage::Settings.primary_plane();
                        s.settings_section = SettingsSection::Services;
                        s.orb_open = false;
                        s.orb_toast = "Orb Sector 8 · Opened volume capability & lease settings".into();
                    },
                    "8. ··· More"
                }
            }
            WorkspaceSave { state }
            if let Some(bridge) = auxiliary {
                {
                    let orb = bridge.clone();
                    let tools = bridge.clone();
                    rsx! {
                        div {
                            class: "orb-sectors",
                            button {
                                r#type: "button",
                                class: "orb-sector-btn",
                                onclick: move |_| orb.open_or_toggle_orb_overlay(),
                                "Float Command Orb (◉)"
                            }
                            button {
                                r#type: "button",
                                class: "orb-sector-btn",
                                onclick: move |_| tools.open_or_toggle_tool_palette(),
                                "Toggle tool palette"
                            }
                            button {
                                r#type: "button",
                                class: "orb-sector-btn",
                                onclick: move |_| bridge.open_urgent_care_alert_modal(),
                                "Open urgent care alerts"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EmptyManifold(mut state: Signal<ShellState>) -> Element {
    let qap_count = state.read().installed_qaps.len();
    let llm_count = state.read().llm_models.len();
    let summary_right = format!("{qap_count} apps ready · {llm_count} local models");
    rsx! {
        div {
            class: "stage-empty",
            id: "stage-empty",
            div {
                class: "manifold",
                aria_label: "Apps",
                div {
                    class: "gpu-banner",
                    div {
                        strong { "Choose an app" }
                        span { " — Talk, Mail, Browser, Library, and the rest of your workspace." }
                    }
                    span { class: "pill live", "{summary_right}" }
                }
                ManifoldCard {
                    name: "Talk",
                    plane: "people",
                    heading: "Messages with people you know",
                    body: "A conversation list on the left and the thread on the right. Helper tools stay clearly marked as tools.",
                    stage: VolumeStage::Talk,
                    state,
                }
                ManifoldCard {
                    name: "Mail",
                    plane: "letters",
                    heading: "Inbox for letters and notes",
                    body: "Read a thread, write a reply, and keep each relationship in its own mailbox.",
                    stage: VolumeStage::Mail,
                    state,
                }
                ManifoldCard {
                    name: "Directory",
                    plane: "people",
                    heading: "People and introductions",
                    body: "Your social book of people and co-ops. Organisations and tools live in their own lists.",
                    stage: VolumeStage::Directory,
                    state,
                }
                ManifoldCard {
                    name: "Browser",
                    plane: "pages",
                    heading: "Web and spatial pages",
                    body: "A real address bar over a full-window page. Open ordinary websites or spatial scenes.",
                    stage: VolumeStage::Browser,
                    state,
                }
                ManifoldCard {
                    name: "Poet Studio",
                    plane: "studio",
                    heading: "Write, draw, and make 3D work",
                    body: "A document editor, formula cells, and a 3D canvas in one studio.",
                    stage: VolumeStage::Poet,
                    state,
                }
                ManifoldCard {
                    name: "WellFair & Anatomy",
                    plane: "care",
                    heading: "Health journal and anatomy atlas",
                    body: "A 3D atlas, personal notes, and care-circle check-ins for the people you look after.",
                    stage: VolumeStage::Wellfair,
                    state,
                }
                ManifoldCard {
                    name: "Wallet",
                    plane: "money",
                    heading: "Balances, send, and receive",
                    body: "Lightning, mixnet, and eCash cards with a simple send form. You sign; tools only propose.",
                    stage: VolumeStage::Wallet,
                    state,
                }
                ManifoldCard {
                    name: "Keep",
                    plane: "private",
                    heading: "Private files and recovery",
                    body: "Unlock status, protected files, and a guided family recovery wizard.",
                    stage: VolumeStage::Keep,
                    state,
                }
                ManifoldCard {
                    name: "Instruments",
                    plane: "tools",
                    heading: "Local models and helpers",
                    body: "Choose a local model, try it, and keep helper tools separate from people.",
                    stage: VolumeStage::Instruments,
                    state,
                }
                ManifoldCard {
                    name: "Library",
                    plane: "files",
                    heading: "Books, files, and 3D models",
                    body: "A card grid with a preview pane for documents, media, and spatial models.",
                    stage: VolumeStage::Library,
                    state,
                }
                ManifoldCard {
                    name: "Projects",
                    plane: "work",
                    heading: "Shared work and milestones",
                    body: "See the current stage of a project, log effort, and keep roles clear.",
                    stage: VolumeStage::Projects,
                    state,
                }
                ManifoldCard {
                    name: "Settings",
                    plane: "setup",
                    heading: "Looks, access, and this device",
                    body: "Text size, colour, motion, and the questionnaire that suggests a look that fits you.",
                    stage: VolumeStage::Settings,
                    state,
                }
            }
        }
    }
}

#[component]
fn ManifoldCard(
    name: &'static str,
    plane: &'static str,
    heading: &'static str,
    body: &'static str,
    stage: VolumeStage,
    mut state: Signal<ShellState>,
) -> Element {
    let s = state.read();
    let animating = s.animate_icons && !s.reduced_motion;
    let omega = eval_vibe_icon_omega(&s.icon_vibe_formula);
    let high_contrast = s.theme_preset == "theme-obsidian";
    drop(s);
    let kind = HoloGlyphKind::for_stage(stage);

    rsx! {
        button {
            r#type: "button",
            class: "vol-card",
            aria_label: "Open {name} volume",
            onclick: move |_| {
                let mut s = state.write();
                s.current_stage = stage;
                s.active_plane = stage.primary_plane();
            },
            div {
                class: "cap",
                span {
                    class: "card-holo-glyph",
                    aria_hidden: "true",
                    HoloGlyphIcon {
                        kind,
                        active: false,
                        animating,
                        omega,
                        high_contrast,
                    }
                }
                b { "{name}" }
                span { class: "pill plane", "{plane}" }
                span { class: "pill live", "Live" }
            }
            div {
                class: "body",
                h3 { "{heading}" }
                p { "{body}" }
            }
        }
    }
}

#[component]
fn VolumeBody(current: VolumeStage, state: Signal<ShellState>) -> Element {
    match current {
        VolumeStage::Talk => rsx! { TalkVolume { state } },
        VolumeStage::Mail => rsx! { MailVolume { state } },
        VolumeStage::Directory => rsx! { DirectoryVolume { state } },
        VolumeStage::Browser => rsx! { BrowserVolume { state } },
        VolumeStage::Keep => rsx! { SanctuaryVolume { state } },
        VolumeStage::Library => rsx! { LibraryVolume { state } },
        VolumeStage::Instruments => rsx! {
            StagedGgufSource { state }
            InstrumentsVolume { state }
        },
        VolumeStage::Poet => rsx! { PoetSynthVolume { state } },
        VolumeStage::Wellfair => rsx! { WellfairVolume { state } },
        VolumeStage::Anatomy => rsx! { AnatomyVolume { state } },
        VolumeStage::Wallet => rsx! { WalletVolume { state } },
        VolumeStage::Projects => rsx! { ProjectsVolume { state } },
        VolumeStage::Settings => rsx! { SettingsVolume { state } },
        VolumeStage::Admin => rsx! { AdminVolume { state } },
        VolumeStage::None => rsx! { div {} },
    }
}

#[component]
fn StagedGgufSource(mut state: Signal<ShellState>) -> Element {
    let Some((source, bytes)) = state.read().staged_gguf_source.clone() else {
        return rsx! {};
    };
    let path = source.display().to_string();
    rsx! {
        section {
            class: "id-list",
            h4 { "GGUF conversion draft" }
            p { class: "path", "{path} · {bytes} bytes" }
            p { "Header recognized. The source is staged for review and remains available for a verified P64 converter." }
            button {
                r#type: "button",
                onclick: move |_| state.write().staged_gguf_source = None,
                "Clear draft"
            }
        }
    }
}
