use crate::state::{ShellState, VolumeStage};
use aether::prelude::*;
use webizen_ua_os::CareAlertSeverity;

const HUMAN: &[(&str, VolumeStage, &str)] = &[
    ("Talk", VolumeStage::Talk, "💬"),
    ("Mail", VolumeStage::Mail, "✉"),
    ("Directory", VolumeStage::Directory, "👥"),
];

const VOLUMES: &[(&str, VolumeStage, &str)] = &[
    ("Browser", VolumeStage::Browser, "🌐"),
    ("Keep", VolumeStage::Keep, "🛡"),
    ("Library", VolumeStage::Library, "📚"),
    ("Instruments", VolumeStage::Instruments, "🎛"),
    ("Poet+Synth", VolumeStage::Poet, "🪶"),
    ("WellFair", VolumeStage::Wellfair, "💚"),
    ("Wallet", VolumeStage::Wallet, "⚡"),
    ("Settings", VolumeStage::Settings, "⚙"),
    ("Admin", VolumeStage::Admin, "🔧"),
];

#[component]
pub fn HaloBar(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let current = s.current_stage;
    let peers = s.p2p_peers_count;
    let nym = s.nym_mixnet_active;
    let os_user = s.os_username.clone();
    let spatial = s.spatial_mode;
    let launcher_open = s.launcher_open;
    let control_center_open = s.control_center_drawer_open;
    let pending_notices = s.os_control_center.notification_count();
    let active_jobs = s.os_control_center.active_job_count();
    let control_center_label =
        format!("OS notifications and jobs: {pending_notices} unread, {active_jobs} active jobs");
    let active_care_alerts = s
        .host_stewardship
        .care_alerts
        .iter()
        .filter(|alert| !alert.resolved_or_reversed && alert.severity >= CareAlertSeverity::Urgent)
        .count();
    let care_alert_label = if active_care_alerts == 1 {
        "1 urgent care alert".to_owned()
    } else {
        format!("{active_care_alerts} urgent care alerts")
    };
    drop(s);

    rsx! {
        nav {
            class: "halo",
            aria_label: "Humans-first spatial halo",
            span { class: "brand-slot", "WEBIZEN" }
            if spatial {
                span { class: "brand-slot", "spatial · etalon" }
            }
            button {
                r#type: "button",
                class: "fav omni-intent-launch",
                title: "Open the semantic command palette (Ctrl+K)",
                aria_keyshortcuts: "Control+K",
                aria_expanded: if launcher_open { "true" } else { "false" },
                onclick: move |_| state.write().launcher_open = true,
                span { class: "orb", aria_hidden: "true", "◉" }
                span { class: "label", "Commands" }
            }
            button {
                r#type: "button",
                class: "fav os-control-center-launch",
                title: "{control_center_label}",
                aria_label: "{control_center_label}",
                aria_expanded: if control_center_open { "true" } else { "false" },
                aria_controls: "os-control-center-drawer",
                onclick: move |_| {
                    let mut shell = state.write();
                    shell.control_center_drawer_open = !shell.control_center_drawer_open;
                },
                span { class: "orb", aria_hidden: "true", "🔔" }
                span { class: "label", "OS Notifications & Jobs" }
                span { class: "label", "{pending_notices} unread · {active_jobs} jobs" }
            }
            h2 { "Humans" }
            for (name, stage_item, glyph) in HUMAN.iter() {
                {
                    let stage = *stage_item;
                    let pressed = current == stage;
                    rsx! {
                        button {
                            key: "{name}",
                            r#type: "button",
                            class: "fav human",
                            aria_pressed: if pressed { "true" } else { "false" },
                            onclick: move |_| {
                                let mut s = state.write();
                                if s.current_stage == stage {
                                    s.current_stage = VolumeStage::None;
                                } else {
                                    s.current_stage = stage;
                                    s.active_plane = stage.primary_plane();
                                }
                                s.orb_open = false;
                            },
                            span { class: "orb", aria_hidden: "true", "{glyph}" }
                            span { class: "label", "{name}" }
                        }
                    }
                }
            }
            div { class: "sep", role: "separator", aria_hidden: "true" }
            h2 { "Volumes" }
            for (name, stage_item, glyph) in VOLUMES.iter() {
                {
                    let stage = *stage_item;
                    let pressed = current == stage;
                    rsx! {
                        button {
                            key: "{name}",
                            r#type: "button",
                            class: "fav",
                            aria_pressed: if pressed { "true" } else { "false" },
                            onclick: move |_| {
                                let mut s = state.write();
                                if s.current_stage == stage {
                                    s.current_stage = VolumeStage::None;
                                } else {
                                    s.current_stage = stage;
                                    s.active_plane = stage.primary_plane();
                                }
                                s.orb_open = false;
                            },
                            span { class: "orb", aria_hidden: "true", "{glyph}" }
                            span { class: "label", "{name}" }
                            if stage == VolumeStage::Wellfair && active_care_alerts > 0 {
                                span {
                                    class: "urgent-care-pill",
                                    aria_label: "{care_alert_label}",
                                    title: "{care_alert_label}",
                                    "{active_care_alerts}"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "sep", role: "separator", aria_hidden: "true" }
            span { class: "brand-slot", "os:{os_user}" }
            span { class: "brand-slot", "{peers} peers" }
            span { class: "brand-slot", if nym { "Nym live" } else { "Nym standby" } }
        }
    }
}
