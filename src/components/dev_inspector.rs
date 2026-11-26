// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Collapsible Developer & Continuity Inspector Drawer (F12 / Inspect).
//! Houses cryptographic proofs, four-plane bindings, NQuin48 inspectors,
//! GPU telemetry, and capability leases. Hidden during normal use.

use crate::state::{DevInspectorTab, ShellState, VolumeStage};
use aether::prelude::*;
use webizen_ua_os::IdentifierFabricPlane;

#[component]
pub fn DevInspectorDrawer(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    if !s.dev_inspector_open {
        return rsx! {};
    }
    let tab = s.dev_inspector_tab;
    let plane_sentence = volume_plane_sentence(s.current_stage);
    let gpu = s.gpu_label.clone();
    let parity_ok = s.nquin_parity_verified;
    let providers = s.native_vibe_providers;
    let renderer = s.renderer_health.clone();
    let fabric = s.fabric_did.clone();
    let validation = s.validation_summary.clone();
    let identifier_lines = s.identifier_lines.clone();
    let adapters: Vec<String> = s
        .gpu_adapters
        .iter()
        .map(|a| {
            format!(
                "{} · {} ({}) · PCI {:04x}:{:04x} · driver {}",
                a.name, a.backend, a.device_type, a.vendor_id, a.device_id, a.driver
            )
        })
        .collect();
    let leases: Vec<String> = s
        .os_control_center
        .leases
        .iter()
        .take(24)
        .map(|lease| {
            let grant = if lease.granted {
                "allowed locally"
            } else {
                "denied locally"
            };
            format!(
                "{} · {} · {grant} · parity {:#018x}",
                lease.package_id,
                lease.scope.label(),
                lease.sealed_nquin_parity
            )
        })
        .collect();
    let worker_state = s.chora_supervisor.worker_state_label.clone();
    let worker_binary = s.chora_supervisor.worker_binary_path.clone();
    let worker_frames = s.chora_supervisor.frame_seq;
    let nquin_slots = identifier_lines.clone();
    let picked = s.last_picked_semantic_id;
    let theme = s.theme_preset;
    let plane = s.active_plane;
    drop(s);

    let (width, height) = renderer.viewport_dimensions;
    let parity_str = if parity_ok { "verified" } else { "planned" };

    rsx! {
        aside {
            id: "dev-inspector-drawer",
            class: "dev-inspector-drawer",
            aria_label: "Developer and continuity inspector",
            div {
                class: "dev-inspector-head",
                h4 { "Inspect" }
                p { class: "muted", "Technical proofs and renderer details. Hidden during everyday use." }
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    onclick: move |_| {
                        let mut shell = state.write();
                        shell.dev_inspector_open = false;
                        shell.inspect_continuity_open = false;
                    },
                    "Close"
                }
            }
            div {
                class: "dev-inspector-tabs",
                role: "tablist",
                InspectorTabButton {
                    tab: DevInspectorTab::ContinuityAndFourPlanes,
                    current: tab,
                    label: "Continuity & Four Planes",
                    state,
                }
                InspectorTabButton {
                    tab: DevInspectorTab::GpuAndEtalonRenderer,
                    current: tab,
                    label: "GPU & Etalon Renderer",
                    state,
                }
                InspectorTabButton {
                    tab: DevInspectorTab::NQuin48AndRefractStore,
                    current: tab,
                    label: "NQuin48 & Refract Store",
                    state,
                }
                InspectorTabButton {
                    tab: DevInspectorTab::WorkersAndCapabilityLeases,
                    current: tab,
                    label: "Workers & Capability Leases",
                    state,
                }
            }
            match tab {
                DevInspectorTab::ContinuityAndFourPlanes => rsx! {
                    section {
                        class: "id-list",
                        h5 { "Four-plane identifier bindings" }
                        p { class: "muted", "{plane_sentence}" }
                        p { class: "muted", "Active plane: {plane.title()} · {plane.code()}" }
                        p { class: "path", "Local QRC node: {fabric}" }
                        p { "{validation}" }
                        dl {
                            class: "kv",
                            for line in identifier_lines {
                                dt { "Binding" }
                                dd { class: "path", "{line}" }
                            }
                        }
                    }
                },
                DevInspectorTab::GpuAndEtalonRenderer => rsx! {
                    section {
                        class: "id-list",
                        h5 { "Native GPU stage" }
                        dl {
                            class: "kv",
                            dt { "Adapter" }
                            dd { "{gpu}" }
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
                            dt { "Vibe providers" }
                            dd { "{providers}" }
                            dt { "Active theme cartridge" }
                            dd { "{theme}" }
                        }
                        ul {
                            for line in adapters {
                                li { class: "path", "{line}" }
                            }
                        }
                    }
                },
                DevInspectorTab::NQuin48AndRefractStore => rsx! {
                    section {
                        class: "id-list",
                        h5 { "NQuin48 parity inspector" }
                        p { "Parity: {parity_str}" }
                        if let Some(id) = picked {
                            p { class: "path", "Last picked semantic id: {id:#018x}" }
                        }
                        ul {
                            for line in nquin_slots {
                                li { class: "path", "{line}" }
                            }
                        }
                    }
                },
                DevInspectorTab::WorkersAndCapabilityLeases => rsx! {
                    section {
                        class: "id-list",
                        h5 { "Workers and leases" }
                        dl {
                            class: "kv",
                            dt { "Web engine worker" }
                            dd { "{worker_state}" }
                            dt { "Worker binary" }
                            dd { class: "path", "{worker_binary}" }
                            dt { "Copied frames" }
                            dd { "{worker_frames}" }
                        }
                        if leases.is_empty() {
                            p { class: "muted", "No capability leases recorded this session." }
                        } else {
                            ul {
                                for line in leases {
                                    li { "{line}" }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn InspectorTabButton(
    tab: DevInspectorTab,
    current: DevInspectorTab,
    label: &'static str,
    mut state: Signal<ShellState>,
) -> Element {
    let selected = tab == current;
    rsx! {
        button {
            r#type: "button",
            role: "tab",
            class: if selected { "ribbon-btn active" } else { "ribbon-btn" },
            aria_selected: if selected { "true" } else { "false" },
            onclick: move |_| state.write().dev_inspector_tab = tab,
            "{label}"
        }
    }
}

pub fn toggle_dev_inspector(state: &mut ShellState) {
    state.dev_inspector_open = !state.dev_inspector_open;
    state.inspect_continuity_open = state.dev_inspector_open;
    if state.dev_inspector_open {
        state.orb_toast = "Opened the Inspect drawer".into();
    } else {
        state.orb_toast = "Closed the Inspect drawer".into();
    }
}

/// Map a volume onto a human-readable plane sentence for the inspector only.
pub fn volume_plane_sentence(stage: VolumeStage) -> &'static str {
    match stage.primary_plane() {
        IdentifierFabricPlane::NaturalAgent => "This volume talks about people.",
        IdentifierFabricPlane::ClaimOpinion => "This volume holds statements and records.",
        IdentifierFabricPlane::SpatiotemporalHandle => "This volume holds places, routes, and addresses.",
        IdentifierFabricPlane::Instrument => "This volume holds tools.",
    }
}
