use super::auxiliary_windows::AuxWindowBridge;
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
    let gpu = s.gpu_label.clone();
    let parity_ok = s.nquin_parity_verified;
    let providers = s.native_vibe_providers;
    let empty = current == VolumeStage::None;
    let current_title = current.title();
    let current_plane_code = current.primary_plane().code();
    let orb_open = s.orb_open;
    let inspect_open = s.inspect_continuity_open;
    let orb_toast = s.orb_toast.clone();
    let chrome_mode = s.stage_chrome_mode;
    let control_center_open = s.control_center_drawer_open;
    let renderer = s.renderer_health.clone();
    let (width, height) = renderer.viewport_dimensions;
    drop(s);

    let parity_str = if parity_ok { "VERIFIED" } else { "PLANNED" };
    let stage_class = match chrome_mode {
        StageChromeMode::DefaultDepth => "stage",
        StageChromeMode::FullStage => "stage full",
        StageChromeMode::FloatLayer => "stage float-mode",
    };
    let plane_title = active_plane.title();
    let plane_rule = active_plane.anti_collapse_rule();

    rsx! {
        div {
            class: "stage-wrap",

            // Top Continuity Ribbon (who · claim · handle · tool)
            nav {
                class: "ribbon-bar",
                aria_label: "Continuity planes",
                div {
                    class: "ribbon-pills",
                    span { class: "ribbon-tag", "Continuity" }
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
                                    "{short} · {code}"
                                }
                            }
                        }
                    }
                }
                div {
                    class: "ribbon-note",
                    strong { "{plane_title}: " }
                    span { "{plane_rule}" }
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
                        span { class: "pill plane", "{current_plane_code}" }
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
                    if inspect_open {
                        ContinuityInspectorBanner { current, state }
                    }
                }
                div {
                    class: "stage-body",
                    if empty {
                        EmptyManifold { state, gpu: gpu.clone() }
                    } else {
                        VolumeBody { current, state }
                    }
                }
            }
            section {
                id: "gpu",
                role: "region",
                aria_label: "GPU and renderer health",
                div {
                    class: "gpu-inline-summary",
                    strong { "GPU Stage: " }
                    span { class: "pill plane", "NATIVE_DESKTOP" }
                    span {
                        " {gpu} · {renderer.backend_label} · NQuin48: {parity_str} · {providers} Vibe providers · {width}×{height} px"
                    }
                    if !orb_toast.is_empty() {
                        span { class: "pill plane", "{orb_toast}" }
                    }
                }
                details {
                    class: "gpu-details-pop",
                    summary { "Renderer Details" }
                    dl {
                        class: "renderer-health",
                        dt { "Backend" }
                        dd { "{renderer.backend_label}" }
                        dt { "Fallback policy" }
                        dd { "{renderer.fallback_policy_label}" }
                        dt { "Fallback engaged" }
                        dd { if renderer.fallback_engaged { "Yes" } else { "No" } }
                        dt { "HOLO surface" }
                        dd { "{renderer.active_holo_surface_status}" }
                        dt { "Viewport" }
                        dd { "{width} × {height} px" }
                    }
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
                    onclick: move |_| {
                        let mut s = state.write();
                        s.inspect_continuity_open = !s.inspect_continuity_open;
                        s.orb_toast = "Orb Sector 7 · Toggled Four-Plane Continuity Inspector".into();
                    },
                    "7. 🔍 Inspect"
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
fn ContinuityInspectorBanner(current: VolumeStage, mut state: Signal<ShellState>) -> Element {
    let plane = current.primary_plane();
    let plane_title = plane.title();
    let plane_rule = plane.anti_collapse_rule();
    let blurb = current.blurb();
    let fabric = state.read().fabric_did.clone();
    rsx! {
        div {
            class: "inspect-banner",
            div {
                strong { "Continuity Inspector · Primary Plane: {plane_title} " }
                span { "— {blurb} · {plane_rule} · Local QRC Node: {fabric}" }
            }
            button {
                r#type: "button",
                class: "btn-win",
                onclick: move |_| state.write().inspect_continuity_open = false,
                "Hide Inspector"
            }
        }
    }
}

#[component]
fn EmptyManifold(mut state: Signal<ShellState>, gpu: String) -> Element {
    let qap_count = state.read().installed_qaps.len();
    let llm_count = state.read().llm_models.len();
    let summary_right =
        format!("{qap_count} Qaps seeded · {llm_count} local AI models · 150 Synth morphisms");
    rsx! {
        div {
            class: "stage-empty",
            id: "stage-empty",
            div {
                class: "manifold",
                aria_label: "Working volumes",
                div {
                    class: "gpu-banner",
                    div {
                        strong { "Etalon Native GPU Stage · {gpu}" }
                        span { " — Spatial depth manifold (Not OSX / Windows / Linux chrome). Humans-first halo. Poet 8-Sector Command Orb." }
                    }
                    span { class: "pill live", "{summary_right}" }
                }
                ManifoldCard {
                    name: "Talk",
                    plane: "claim · relation",
                    heading: "A person, waiting for a person",
                    body: "Human <-> human over QDNF. Chatbot sits on the Tool plane — never impersonates a peer.",
                    stage: VolumeStage::Talk,
                    state,
                }
                ManifoldCard {
                    name: "Mail",
                    plane: "handle · claim",
                    heading: "Relationship-scoped inbox",
                    body: "Directional pairwise locators (A->B != B->A). Domain death never erases the person.",
                    stage: VolumeStage::Mail,
                    state,
                }
                ManifoldCard {
                    name: "Directory",
                    plane: "who (NaturalAgent)",
                    heading: "People & Living Beings first",
                    body: "Framed as living-SHACL (disjoint from owl:Thing). Pairwise introductions and social book.",
                    stage: VolumeStage::Directory,
                    state,
                }
                ManifoldCard {
                    name: "Browser",
                    plane: "handle · claim",
                    heading: "Dual-Web Reach (Chora + WWW Engine)",
                    body: "Ring B qualia://chora/universe 10D canvas + Ring C webizen-WWW-engine (MPL-2.0 sibling worker).",
                    stage: VolumeStage::Browser,
                    state,
                }
                ManifoldCard {
                    name: "Poet & Synth Workbench",
                    plane: "tool · claim",
                    heading: "Unified VibeScript + 150 Hypermedia Morphisms",
                    body: "Live VibeScript 0.1/0.2 cells, NLP Gazetteer NQuin48 sealer, and .10d v2 HOLO / .hmc production.",
                    stage: VolumeStage::Poet,
                    state,
                }
                ManifoldCard {
                    name: "WellFair & Anatomy",
                    plane: "who · claim · tool",
                    heading: "Fiduciary Health, 3D Anatomy & Guardianship",
                    body: "48B NQuin epistemic envelopes, maternal-fetal dyad, UN CRC/CRPD guardianship, and 6 Vectors.",
                    stage: VolumeStage::Wellfair,
                    state,
                }
                ManifoldCard {
                    name: "Wallet & Commons",
                    plane: "tool · claim",
                    heading: "Lightning · Nym · eCash (XEC) · Tokens",
                    body: "Human-owned keys in KeyVault. Two-Loop Permissive Commons & 9-Stage Contribution. No ETH.",
                    stage: VolumeStage::Wallet,
                    state,
                }
                ManifoldCard {
                    name: "Keep · Sanctuary Vault",
                    plane: "tool",
                    heading: "Anti-Coercion 4-Layer VaultContainerV2",
                    body: "Argon2id + AES-256-GCM Real/Decoy lanes, Audit DAG, Anvil NP64 gate, Dead-Man & Incapacity switches.",
                    stage: VolumeStage::Keep,
                    state,
                }
                ManifoldCard {
                    name: "Instruments & Logics",
                    plane: "tool · claim",
                    heading: "6 Core Semantic Instruments & 40+ Formal Logics",
                    body: "Collectable HMC logic packages (SI-01..12), 6 SHACL opcode bridges, and local .p64/.gguf AI models.",
                    stage: VolumeStage::Instruments,
                    state,
                }
                ManifoldCard {
                    name: "Library & QECP",
                    plane: "claim · tool",
                    heading: "Hypermedia (.q42 / .10d / .hmc) & QECP PDFs",
                    body: "Faceted SPARQL-Star & GeoSPARQL keep with C2PA JUMBF embedded verifiable credential extraction.",
                    stage: VolumeStage::Library,
                    state,
                }
                ManifoldCard {
                    name: "Projects · Work-Graph",
                    plane: "claim · relation",
                    heading: "Cooperative Projects & 9-Stage Lifecycle",
                    body: "Steward/Contributor/Observer roles, add-wins effort/capital merge, and licence-transition readiness.",
                    stage: VolumeStage::Projects,
                    state,
                }
                ManifoldCard {
                    name: "Settings (10 Sections)",
                    plane: "tool",
                    heading: "Environment Setup & Four-Plane Identifier Fabric",
                    body: "Setup health, Data strata, AI instruments, QDNF/Solid, Sanctuary, Appearance, Fleet, and Dual-SKU.",
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
                span { class: "mark", aria_hidden: "true" }
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
