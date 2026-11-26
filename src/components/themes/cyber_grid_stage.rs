// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! 3D Neon Wireframe Telemetry Grid: isometric towers with packet light-lanes
//! and phosphor scanline framing.

use crate::state::ShellState;
use aether::prelude::*;
use webizen_ua_os::{
    etalon_vibe::oracle_skeletal_lerp_joint, project_semantic_scene_with_cartridge,
    render_theme_acoustic_frame,
};

#[component]
pub fn CyberGridStage(mut state: Signal<ShellState>) -> Element {
    let mut dock = use_signal(|| [0.0f32; 3]);
    let graph = {
        let shell = state.read();
        project_semantic_scene_with_cartridge(
            &shell.project_semantic_scene_graph(),
            shell.active_cartridge(),
        )
    };
    let camera = *dock.read();
    rsx! {
        div {
            class: "spatial-stage cyber-grid",
            aria_label: "Neon wireframe telemetry grid",
            style: "transform: translate3d({camera[0]}px, {camera[1]}px, {camera[2]}px);",
            p { class: "spatial-stage-lead", "Each glowing tower is an app or a file vault. Click a tower to dock the camera and open it." }
            div {
                class: "cyber-grid-city",
                for node in graph.nodes {
                    {
                        let x = node.coords_10d[0];
                        let y = node.coords_10d[1];
                        let z = node.coords_10d[2];
                        let label = node.label.clone();
                        let intent = node.primary_intent.clone();
                        let target = [x * 40.0, y * -18.0, z * 12.0];
                        let height = (80.0 + y.abs() * 140.0).clamp(72.0, 220.0);
                        let style = format!(
                            "left: {}%; top: {}%; height: {height}px; transform: translate(-50%, -100%) rotateX(60deg) rotateZ(-45deg);",
                            50.0 + x * 28.0 + z * 10.0,
                            58.0 - z * 16.0 + x * 8.0
                        );
                        rsx! {
                            button {
                                key: "{node.semantic_id}",
                                r#type: "button",
                                class: "cyber-tower",
                                style: "{style}",
                                onclick: move |_| {
                                    let current = *dock.peek();
                                    if let Ok(next) = oracle_skeletal_lerp_joint(current, target, 0.35) {
                                        dock.set(next);
                                    }
                                    let mut shell = state.write();
                                    shell.last_picked_semantic_id = Some(node.semantic_id);
                                    let _ = render_theme_acoustic_frame(
                                        shell.active_cartridge().topology,
                                        x * 45.0,
                                        0.4,
                                    );
                                    let _ = shell.apply_semantic_intent(&intent);
                                },
                                span { class: "cyber-tower-shaft", aria_hidden: "true" }
                                span { class: "cyber-tower-label", "{label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
