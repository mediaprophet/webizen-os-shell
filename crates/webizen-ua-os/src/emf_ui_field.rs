// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Project the semantic UI scene-graph onto a Q42EMF64 wavefield and HOLO SDF nodes.
//!
//! Byte layouts match `etalon-render::emf::EmfFieldCell` (64 bytes, Stokes Q/U/V)
//! and `etalon-render::volumetric::HolographicNodeGpu` (96 bytes). GPU upload lives
//! in the shell viewport; this crate stays free of the Aether renderer.

use crate::semantic_scene_graph::{SemanticNodeKind, SemanticSceneGraph};
use crate::theme_cartridge::SpatialManifoldTopology;

pub const MAX_EMF_CELLS: usize = 256;
pub const EMF_CELL_BYTES: usize = 64;
pub const MAX_HOLO_NODES: usize = 64;
pub const HOLO_NODE_BYTES: usize = 96;

/// 64-byte EMF cell matching `etalon_render::emf::EmfFieldCell`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmfFieldCell {
    pub amplitude: f32,
    pub phase: f32,
    pub frequency: f32,
    pub scale: f32,
    pub attention_depth: f32,
    pub epistemic_weight: f32,
    pub topological_spin: f32,
    pub temporal_decay: f32,
    pub entropy_bias: f32,
    pub spatial_phase: f32,
    pub recurrence_frequency: f32,
    pub density_threshold: f32,
    pub manifold_curvature: f32,
    pub stokes_q: f32,
    pub stokes_u: f32,
    pub stokes_v: f32,
}

const _: () = {
    assert!(core::mem::size_of::<EmfFieldCell>() == 64);
    assert!(core::mem::offset_of!(EmfFieldCell, attention_depth) == 16);
    assert!(core::mem::offset_of!(EmfFieldCell, stokes_q) == 52);
};

impl Default for EmfFieldCell {
    fn default() -> Self {
        Self {
            amplitude: 0.0,
            phase: 0.0,
            frequency: 0.0,
            scale: 1.0,
            attention_depth: 0.0,
            epistemic_weight: 1.0,
            topological_spin: 0.0,
            temporal_decay: 0.0,
            entropy_bias: 0.0,
            spatial_phase: 0.0,
            recurrence_frequency: 0.0,
            density_threshold: 0.0,
            manifold_curvature: 0.0,
            stokes_q: 0.0,
            stokes_u: 0.0,
            stokes_v: 0.0,
        }
    }
}

impl EmfFieldCell {
    pub fn as_bytes(&self) -> [u8; EMF_CELL_BYTES] {
        let lanes = [
            self.amplitude,
            self.phase,
            self.frequency,
            self.scale,
            self.attention_depth,
            self.epistemic_weight,
            self.topological_spin,
            self.temporal_decay,
            self.entropy_bias,
            self.spatial_phase,
            self.recurrence_frequency,
            self.density_threshold,
            self.manifold_curvature,
            self.stokes_q,
            self.stokes_u,
            self.stokes_v,
        ];
        let mut bytes = [0u8; EMF_CELL_BYTES];
        for (slot, lane) in bytes.chunks_exact_mut(4).zip(lanes) {
            slot.copy_from_slice(&lane.to_le_bytes());
        }
        bytes
    }
}

/// Projected EMF + HOLO packing for one semantic scene.
#[derive(Clone, Debug)]
pub struct EmfUiField {
    pub cells: [EmfFieldCell; MAX_EMF_CELLS],
    pub cell_count: u32,
    holo: [[f32; 4]; MAX_HOLO_NODES * 6],
    pub holo_count: u32,
}

impl Default for EmfUiField {
    fn default() -> Self {
        Self {
            cells: [EmfFieldCell::default(); MAX_EMF_CELLS],
            cell_count: 0,
            holo: [[0.0; 4]; MAX_HOLO_NODES * 6],
            holo_count: 0,
        }
    }
}

impl EmfUiField {
    pub fn emf_cell_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; MAX_EMF_CELLS * EMF_CELL_BYTES];
        for (index, cell) in self.cells.iter().enumerate() {
            let slot = &mut bytes[index * EMF_CELL_BYTES..(index + 1) * EMF_CELL_BYTES];
            slot.copy_from_slice(&cell.as_bytes());
        }
        bytes
    }

    pub fn holo_node_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0u8; MAX_HOLO_NODES * HOLO_NODE_BYTES];
        for index in 0..MAX_HOLO_NODES {
            let base = index * 6;
            let slot = &mut bytes[index * HOLO_NODE_BYTES..(index + 1) * HOLO_NODE_BYTES];
            for (word_slot, lane) in slot.chunks_exact_mut(16).zip(self.holo[base..base + 6].iter())
            {
                for (word, axis) in word_slot.chunks_exact_mut(4).zip(lane) {
                    word.copy_from_slice(&axis.to_le_bytes());
                }
            }
        }
        bytes
    }

    pub fn cie1931_xyz_6band(bands: [f32; 6]) -> [f32; 3] {
        const XBAR: [f32; 6] = [0.0143, 0.3362, 0.0049, 0.4334, 1.0622, 0.2835];
        const YBAR: [f32; 6] = [0.0004, 0.0380, 0.3230, 0.9950, 0.6310, 0.1070];
        const ZBAR: [f32; 6] = [0.0679, 1.7721, 0.2720, 0.0087, 0.0008, 0.0000];
        let mut xyz = [0.0f32; 3];
        for i in 0..6 {
            let s = bands[i].max(0.0);
            xyz[0] += s * XBAR[i];
            xyz[1] += s * YBAR[i];
            xyz[2] += s * ZBAR[i];
        }
        let norm = (xyz[0] + xyz[1] + xyz[2]).max(1.0e-6);
        [xyz[0] / norm, xyz[1] / norm, xyz[2] / norm]
    }
}

/// Stokes polarization pose for each spatial topology (linear / circular / elliptical).
pub fn stokes_for_topology(topology: SpatialManifoldTopology) -> [f32; 3] {
    match topology {
        SpatialManifoldTopology::PlanarDesktopGrid => [0.0, 0.0, 0.0],
        SpatialManifoldTopology::CylindricalGlassArc => [0.05, 0.08, 0.62],
        SpatialManifoldTopology::IsometricWireframeCityGrid => [0.72, 0.12, 0.04],
        SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d => [0.28, 0.46, 0.38],
    }
}

/// Map a semantic scene-graph onto the 64-byte EMF grid and 96-byte HOLO nodes.
pub fn project_emf_ui_field(
    graph: &SemanticSceneGraph,
    topology: SpatialManifoldTopology,
) -> EmfUiField {
    let stokes = stokes_for_topology(topology);
    let mut field = EmfUiField::default();
    for (index, node) in graph.nodes.iter().take(MAX_EMF_CELLS).enumerate() {
        let coords = node.coords_10d;
        let active = graph.active_semantic_id == Some(node.semantic_id);
        let kind_gain = match node.kind {
            SemanticNodeKind::Volume => 1.0,
            SemanticNodeKind::DiegeticConsole => 1.15,
            SemanticNodeKind::ThemeCartridge => 0.85,
            _ => 0.72,
        };
        let attention = if coords[4] > 0.0 {
            coords[4]
        } else if active {
            1.0
        } else {
            0.35
        };
        let epistemic = if coords[5] > 0.0 { coords[5] } else { 1.0 };
        let cell = EmfFieldCell {
            amplitude: (0.45_f32 + kind_gain * 0.4).clamp(0.0, 1.5),
            phase: coords[3] * core::f32::consts::TAU,
            frequency: 1.0 + (index as f32) * 0.07,
            scale: if coords[3] > 0.0 { coords[3] } else { 1.0 },
            attention_depth: attention,
            epistemic_weight: epistemic,
            topological_spin: coords[6],
            temporal_decay: coords[7],
            entropy_bias: coords[8],
            spatial_phase: coords[0].atan2(coords[2]),
            recurrence_frequency: if coords[9] > 0.0 {
                coords[9]
            } else {
                2.0 + (index as f32 % 4.0)
            },
            density_threshold: 0.12,
            manifold_curvature: match topology {
                SpatialManifoldTopology::CylindricalGlassArc => 0.42,
                SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d => 0.78,
                SpatialManifoldTopology::IsometricWireframeCityGrid => 0.08,
                SpatialManifoldTopology::PlanarDesktopGrid => 0.0,
            },
            stokes_q: stokes[0],
            stokes_u: stokes[1],
            stokes_v: stokes[2],
        };
        field.cells[index] = cell;
        field.cell_count = (index as u32) + 1;
        if index < MAX_HOLO_NODES {
            let primitive = match node.kind {
                SemanticNodeKind::DiegeticConsole => 2.0,
                SemanticNodeKind::Volume => 0.0,
                _ => 1.0,
            };
            let xyz = EmfUiField::cie1931_xyz_6band(spectral_bands_for_cell(&cell));
            let rgb = xyz_to_linear_rgb(xyz);
            let base = index * 6;
            field.holo[base] = [coords[0], coords[1], coords[2], 1.0];
            field.holo[base + 1] = [0.45, 0.45, 0.45, primitive];
            field.holo[base + 2] = [cell.frequency, cell.amplitude, 0.0, cell.temporal_decay];
            field.holo[base + 3] = [
                1.0,
                cell.topological_spin,
                cell.manifold_curvature,
                cell.attention_depth,
            ];
            field.holo[base + 4] = [0.0, 0.0, 0.0, node.semantic_id as f32];
            field.holo[base + 5] = [rgb[0], rgb[1], rgb[2], 1.0];
            field.holo_count = (index as u32) + 1;
        }
    }
    field
}

fn spectral_bands_for_cell(cell: &EmfFieldCell) -> [f32; 6] {
    let sigma = if cell.recurrence_frequency > 0.0 {
        cell.recurrence_frequency
    } else {
        (cell.phase.abs()) % 6.0
    };
    let mut bands = [0.0f32; 6];
    for (i, band) in bands.iter_mut().enumerate() {
        let d = sigma - i as f32;
        *band = cell.amplitude.abs() * (-d * d * 0.45).exp();
    }
    bands
}

fn xyz_to_linear_rgb(xyz: [f32; 3]) -> [f32; 3] {
    [
        (3.2406 * xyz[0] - 1.5372 * xyz[1] - 0.4986 * xyz[2]).clamp(0.0, 1.0),
        (-0.9689 * xyz[0] + 1.8758 * xyz[1] + 0.0415 * xyz[2]).clamp(0.0, 1.0),
        (0.0557 * xyz[0] - 0.2040 * xyz[1] + 1.0570 * xyz[2]).clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic_scene_graph::project_core_volume_graph;

    #[test]
    fn emf_cell_layout_is_q42emf64() {
        assert_eq!(core::mem::size_of::<EmfFieldCell>(), 64);
        let graph = project_core_volume_graph();
        let field = project_emf_ui_field(
            &graph,
            SpatialManifoldTopology::CylindricalGlassArc,
        );
        assert!(field.cell_count >= 1);
        assert_eq!(field.emf_cell_bytes().len(), MAX_EMF_CELLS * EMF_CELL_BYTES);
        assert_eq!(field.holo_node_bytes().len(), MAX_HOLO_NODES * HOLO_NODE_BYTES);
        let stokes = stokes_for_topology(SpatialManifoldTopology::CylindricalGlassArc);
        assert!(stokes[2] > 0.5);
        let xyz = EmfUiField::cie1931_xyz_6band([0.1, 0.2, 0.4, 0.8, 0.3, 0.1]);
        assert!(xyz.iter().all(|c| *c >= 0.0));
    }
}
