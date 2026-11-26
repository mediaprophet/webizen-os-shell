// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Headless semantic UI scene-graph and typed intent bus for the product-first shell.
//!
//! The graph is a projection of live workspace state. It never stores Decentralised
//! Identifier strings, NQuin48 hex words, or RouteStamp counters on the user-facing
//! surface. Those proofs remain in the Developer Drawer.

use crate::host_stewardship::validate_human_centric_host_nomenclature;
use crate::identifiers::q_hash_60;
use serde::{Deserialize, Serialize};

/// Kind of a pickable spatial or planar UI node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticNodeKind {
    Volume,
    Bookmark,
    LibraryAsset,
    Contact,
    ConversationThread,
    WalletInstrument,
    ProjectMilestone,
    AnatomyRegion,
    DiegeticConsole,
    ThemeCartridge,
}

/// Typed intent dispatched when a semantic node is activated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticIntent {
    OpenVolume {
        stage_slug: String,
    },
    Navigate {
        uri: String,
    },
    PreviewAsset {
        uri: String,
    },
    ComposeMessage {
        peer: String,
    },
    InspectNode {
        semantic_id: u64,
    },
    AdoptTheme {
        cartridge_id: String,
    },
    PreviewTheme {
        cartridge_id: String,
    },
}

/// One pickable node in the semantic UI scene-graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SemanticUiNode {
    pub semantic_id: u64,
    pub kind: SemanticNodeKind,
    pub label: String,
    pub coords_10d: [f32; 10],
    pub primary_intent: SemanticIntent,
}

/// Bounded scene-graph projected from the current workspace.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SemanticSceneGraph {
    pub nodes: Vec<SemanticUiNode>,
    pub active_semantic_id: Option<u64>,
}

impl SemanticSceneGraph {
    pub const MAX_NODES: usize = 64;

    pub fn get(&self, semantic_id: u64) -> Option<&SemanticUiNode> {
        self.nodes.iter().find(|node| node.semantic_id == semantic_id)
    }

    pub fn push_volume(
        &mut self,
        slug: &str,
        label: &str,
        index: usize,
        count: usize,
    ) -> Result<(), String> {
        validate_human_centric_host_nomenclature(label)?;
        if self.nodes.len() >= Self::MAX_NODES {
            return Err("Semantic scene-graph is full".into());
        }
        let semantic_id = q_hash_60(&format!("urn:webizen:volume:{slug}"));
        let t = if count <= 1 {
            0.0
        } else {
            index as f32 / (count.saturating_sub(1) as f32)
        };
        self.nodes.push(SemanticUiNode {
            semantic_id,
            kind: SemanticNodeKind::Volume,
            label: label.to_owned(),
            coords_10d: planar_slot(t, index),
            primary_intent: SemanticIntent::OpenVolume {
                stage_slug: slug.to_owned(),
            },
        });
        Ok(())
    }
}

fn planar_slot(t: f32, index: usize) -> [f32; 10] {
    let angle = t * std::f32::consts::TAU;
    [
        angle.cos() * 0.85,
        0.12 * ((index % 3) as f32 - 1.0),
        angle.sin() * 0.85,
        t,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        1.0,
    ]
}

/// Apply a semantic intent to a mutable workspace projection.
///
/// Callers map the returned stage slug / URI onto `ShellState`. This crate stays
/// free of the Aether UI runtime.
pub fn apply_semantic_intent(
    graph: &mut SemanticSceneGraph,
    intent: &SemanticIntent,
) -> Result<SemanticIntent, String> {
    match intent {
        SemanticIntent::InspectNode { semantic_id } => {
            if graph.get(*semantic_id).is_none() {
                return Err("Unknown semantic node".into());
            }
            graph.active_semantic_id = Some(*semantic_id);
            Ok(intent.clone())
        }
        SemanticIntent::OpenVolume { stage_slug } => {
            validate_stage_slug(stage_slug)?;
            if let Some(node) = graph
                .nodes
                .iter()
                .find(|node| matches!(&node.primary_intent, SemanticIntent::OpenVolume { stage_slug: slug } if slug == stage_slug))
            {
                graph.active_semantic_id = Some(node.semantic_id);
            }
            Ok(intent.clone())
        }
        SemanticIntent::Navigate { uri } => {
            if uri.trim().is_empty() || uri.len() > 4096 {
                return Err("Navigation URI is empty or too long".into());
            }
            Ok(intent.clone())
        }
        SemanticIntent::PreviewAsset { uri } => {
            if uri.trim().is_empty() {
                return Err("Asset URI is empty".into());
            }
            Ok(intent.clone())
        }
        SemanticIntent::ComposeMessage { peer } => {
            validate_human_centric_host_nomenclature(peer)?;
            Ok(intent.clone())
        }
        SemanticIntent::AdoptTheme { cartridge_id } | SemanticIntent::PreviewTheme { cartridge_id } => {
            validate_cartridge_id(cartridge_id)?;
            Ok(intent.clone())
        }
    }
}

fn validate_stage_slug(slug: &str) -> Result<(), String> {
    const SLUGS: &[&str] = &[
        "none",
        "talk",
        "mail",
        "directory",
        "browser",
        "keep",
        "library",
        "instruments",
        "poet",
        "wellfair",
        "wallet",
        "projects",
        "anatomy",
        "settings",
        "admin",
    ];
    if SLUGS.contains(&slug) {
        Ok(())
    } else {
        Err("Unknown workspace volume".into())
    }
}

fn validate_cartridge_id(id: &str) -> Result<(), String> {
    validate_human_centric_host_nomenclature(id)?;
    if id.starts_with("theme-") || id.starts_with("custom-theme-") {
        Ok(())
    } else {
        Err("Theme cartridge id must use the theme- or custom-theme- prefix".into())
    }
}

/// Seed the twelve product volumes as a planar scene-graph.
pub fn project_core_volume_graph() -> SemanticSceneGraph {
    let mut graph = SemanticSceneGraph::default();
    const VOLUMES: &[(&str, &str)] = &[
        ("talk", "Talk"),
        ("mail", "Mail"),
        ("directory", "Directory"),
        ("browser", "Browser"),
        ("keep", "Keep"),
        ("library", "Library"),
        ("instruments", "Instruments"),
        ("poet", "Poet Studio"),
        ("wellfair", "WellFair"),
        ("wallet", "Wallet"),
        ("projects", "Projects"),
        ("settings", "Settings"),
    ];
    for (index, (slug, label)) in VOLUMES.iter().enumerate() {
        let _ = graph.push_volume(slug, label, index, VOLUMES.len());
    }
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_graph_projects_twelve_pickable_nodes() {
        let graph = project_core_volume_graph();
        assert_eq!(graph.nodes.len(), 12);
        assert!(graph.nodes.iter().all(|node| node.coords_10d.len() == 10));
        let talk = graph
            .nodes
            .iter()
            .find(|node| node.label == "Talk")
            .expect("Talk volume");
        assert!(matches!(
            talk.primary_intent,
            SemanticIntent::OpenVolume { ref stage_slug } if stage_slug == "talk"
        ));
    }

    #[test]
    fn apply_intent_selects_the_matching_node() {
        let mut graph = project_core_volume_graph();
        let intent = SemanticIntent::OpenVolume {
            stage_slug: "browser".into(),
        };
        apply_semantic_intent(&mut graph, &intent).unwrap();
        let browser = graph
            .nodes
            .iter()
            .find(|node| matches!(&node.primary_intent, SemanticIntent::OpenVolume { stage_slug } if stage_slug == "browser"))
            .unwrap();
        assert_eq!(graph.active_semantic_id, Some(browser.semantic_id));
    }

    #[test]
    fn forbidden_nomenclature_is_rejected() {
        let mut graph = SemanticSceneGraph::default();
        let err = graph
            .push_volume("talk", "self-\u{73}overeign peer", 0, 1)
            .unwrap_err();
        assert!(err.contains("Human-Centric"));
    }
}
