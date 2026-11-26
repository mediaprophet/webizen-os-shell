// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! 360° Orbital 10D Volumetric Amphitheatre: SDF pavilions with floating
//! angled diegetic consoles.

use crate::components::viewports::Holo10dViewport;
use crate::state::ShellState;
use aether::prelude::*;
use webizen_ua_os::{project_semantic_scene_with_cartridge, render_theme_acoustic_frame};

#[component]
pub fn SpatialCommonsStage(mut state: Signal<ShellState>) -> Element {
    let graph = {
        let shell = state.read();
        project_semantic_scene_with_cartridge(
            &shell.project_semantic_scene_graph(),
            shell.active_cartridge(),
        )
    };
    rsx! {
        div {
            class: "spatial-stage spatial-commons",
            aria_label: "Orbital volumetric amphitheatre",
            p { class: "spatial-stage-lead", "Walk the ring of pavilions. Each floating console opens an app without leaving the horizon." }
            Holo10dViewport {
                state,
                specimen_label: "Orbital amphitheatre".to_string(),
            }
            div {
                class: "spatial-commons-ring",
                for node in graph.nodes {
                    {
                        let angle = node.coords_10d[3];
                        let label = node.label.clone();
                        let intent = node.primary_intent.clone();
                        let style = format!(
                            "transform: rotateY({}deg) translateZ(220px) rotateX(-18deg);",
                            angle.to_degrees()
                        );
                        rsx! {
                            button {
                                key: "{node.semantic_id}",
                                r#type: "button",
                                class: "diegetic-console",
                                style: "{style}",
                                onclick: move |_| {
                                    let mut shell = state.write();
                                    shell.last_picked_semantic_id = Some(node.semantic_id);
                                    let _ = render_theme_acoustic_frame(
                                        shell.active_cartridge().topology,
                                        angle.to_degrees(),
                                        0.55,
                                    );
                                    let _ = shell.apply_semantic_intent(&intent);
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
