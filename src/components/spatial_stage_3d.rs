// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Bijective 3D object picking for semantic UI nodes on the Etalon viewport.
//! Slot encoding matches `etalon_render::picking::{encode_mesh_slot, resolve_pick}`.

use webizen_ua_os::{SemanticSceneGraph, SemanticUiNode};

pub const MESH_PICK_FLAG: u32 = 1 << 31;
pub const MESH_PICK_SLOT_MASK: u32 = !MESH_PICK_FLAG;

pub fn encode_mesh_slot(slot: u32) -> Option<u32> {
    (slot < MESH_PICK_FLAG).then_some(MESH_PICK_FLAG | slot)
}

pub fn decode_mesh_slot(encoded: u32) -> Option<usize> {
    (encoded & MESH_PICK_FLAG != 0).then_some((encoded & MESH_PICK_SLOT_MASK) as usize)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickResult {
    NoHit,
    Tensor(u32),
    Mesh(u64),
    InvalidSlot,
}

pub fn resolve_pick(raw: u32, snapshot: &[u64]) -> PickResult {
    if raw == 0 {
        return PickResult::NoHit;
    }
    if let Some(slot) = decode_mesh_slot(raw) {
        snapshot
            .get(slot)
            .copied()
            .map(PickResult::Mesh)
            .unwrap_or(PickResult::InvalidSlot)
    } else {
        PickResult::Tensor(raw - 1)
    }
}

/// Ray-intersect projected nodes in widget-local pixels.
pub fn pick_semantic_id(
    graph: &SemanticSceneGraph,
    local_x: f32,
    local_y: f32,
    width: f32,
    height: f32,
) -> Option<u64> {
    if width <= 1.0 || height <= 1.0 {
        return None;
    }
    let ndc_x = (local_x / width) * 2.0 - 1.0;
    let ndc_y = 1.0 - (local_y / height) * 2.0;
    let mut best: Option<(f32, u64, u32)> = None;
    for (slot, node) in graph.nodes.iter().enumerate() {
        let [sx, sy] = project_node(node);
        let dx = sx - ndc_x;
        let dy = sy - ndc_y;
        let dist2 = dx * dx + dy * dy;
        if dist2 < 0.045 {
            if best.is_none_or(|(best_dist, _, _)| dist2 < best_dist) {
                best = Some((dist2, node.semantic_id, slot as u32));
            }
        }
    }
    let (_, semantic_id, slot) = best?;
    let encoded = encode_mesh_slot(slot)?;
    let snapshot: Vec<u64> = graph.nodes.iter().map(|node| node.semantic_id).collect();
    match resolve_pick(encoded, &snapshot) {
        PickResult::Mesh(id) => Some(id),
        _ => Some(semantic_id),
    }
}

fn project_node(node: &SemanticUiNode) -> [f32; 2] {
    let x = node.coords_10d[0];
    let y = node.coords_10d[1];
    let z = node.coords_10d[2];
    let depth = (z + 2.4).max(0.35);
    [x / depth, y / depth]
}

#[cfg(test)]
mod tests {
    use super::*;
    use webizen_ua_os::project_core_volume_graph;

    #[test]
    fn mesh_slot_round_trips_through_resolve_pick() {
        let encoded = encode_mesh_slot(3).unwrap();
        let snapshot = [11u64, 22, 33, 44];
        assert_eq!(resolve_pick(encoded, &snapshot), PickResult::Mesh(44));
        assert_eq!(resolve_pick(0, &snapshot), PickResult::NoHit);
    }

    #[test]
    fn centre_pixel_hits_a_projected_volume() {
        let graph = project_core_volume_graph();
        let hit = pick_semantic_id(&graph, 200.0, 150.0, 400.0, 300.0);
        assert!(hit.is_some());
    }
}
