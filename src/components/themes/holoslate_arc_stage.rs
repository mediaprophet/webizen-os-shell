// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Curved Refractive Glass Arc: concave cylindrical translucent panels with
//! Fresnel edge glow and spring-damper inertial throw.

use crate::state::ShellState;
use aether::prelude::*;
use webizen_ua_os::{
    etalon_vibe::{oracle_spring_damper_force, oracle_verlet_step},
    project_semantic_scene_with_cartridge, render_theme_acoustic_frame,
};

#[component]
pub fn HoloslateArcStage(mut state: Signal<ShellState>) -> Element {
    let mut throw = use_signal(|| [0.0f64; 3]);
    let graph = {
        let shell = state.read();
        project_semantic_scene_with_cartridge(
            &shell.project_semantic_scene_graph(),
            shell.active_cartridge(),
        )
    };
    let offset = throw.read()[0];
    rsx! {
        div {
            class: "spatial-stage holoslate-arc",
            aria_label: "Curved refractive glass arc",
            p { class: "spatial-stage-lead", "Swipe the glass arc to open an app. Panels sit in depth with a soft glowing edge." }
            div {
                class: "input-row",
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    onclick: move |_| throw_arc(&mut throw, -1.2),
                    "Throw left"
                }
                button {
                    r#type: "button",
                    class: "ribbon-btn",
                    onclick: move |_| throw_arc(&mut throw, 1.2),
                    "Throw right"
                }
            }
            div {
                class: "holoslate-track",
                style: "transform: translateX({offset * 80.0}px);",
                for node in graph.nodes {
                    {
                        let x = node.coords_10d[0];
                        let y = node.coords_10d[1];
                        let z = node.coords_10d[2];
                        let yaw = node.coords_10d[3];
                        let label = node.label.clone();
                        let intent = node.primary_intent.clone();
                        let style = format!(
                            "transform: translate3d({}px, {}px, {}px) rotateY({}deg);",
                            x * 220.0,
                            y * 80.0,
                            z * 90.0,
                            yaw.to_degrees()
                        );
                        rsx! {
                            button {
                                key: "{node.semantic_id}",
                                r#type: "button",
                                class: "holoslate-panel",
                                style: "{style}",
                                onclick: move |_| {
                                    let mut shell = state.write();
                                    shell.last_picked_semantic_id = Some(node.semantic_id);
                                    let _ = render_theme_acoustic_frame(
                                        shell.active_cartridge().topology,
                                        yaw.to_degrees(),
                                        0.45,
                                    );
                                    let _ = shell.apply_semantic_intent(&intent);
                                },
                                span { class: "holoslate-glass", "{label}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn throw_arc(throw: &mut Signal<[f64; 3]>, impulse: f64) {
    let mut pose = throw.write();
    pose[1] += impulse;
    if let Ok(force) =
        oracle_spring_damper_force([pose[0], 0.0, 0.0], [pose[1], 0.0, 0.0], 9.0, 2.6)
    {
        pose[2] = force[0];
    }
    if let Ok(step) = oracle_verlet_step(
        [pose[0], 0.0, 0.0],
        [pose[1], 0.0, 0.0],
        [pose[2], 0.0, 0.0],
        1.0 / 60.0,
    ) {
        pose[0] = step[0].clamp(-2.4, 2.4);
        pose[1] = step[3];
    }
}
