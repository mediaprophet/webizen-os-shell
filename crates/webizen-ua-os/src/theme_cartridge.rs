// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! VibeScript Theme Cartridge engine: original spatial design archetypes, affordance
//! equivalence, and Permissive Commons sharing. Themes are described only by geometric,
//! optical, kinetic, and acoustic properties.

use crate::host_stewardship::validate_human_centric_host_nomenclature;
use crate::semantic_scene_graph::{SemanticSceneGraph, SemanticUiNode};
use serde::{Deserialize, Serialize};

/// Spatial layout used to place semantic UI nodes in 10D coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpatialManifoldTopology {
    PlanarDesktopGrid,
    CylindricalGlassArc,
    IsometricWireframeCityGrid,
    OrbitalVolumetricAmphitheatre10d,
}

/// Optical post-process profile synthesised by `etalon-wgsl-forge`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostProcessShaderProfile {
    CleanSpatialSrgb,
    RefractiveCurvedGlassBloom,
    WireframePhosphorScanline,
    VolumetricSpectralHdr,
}

/// Declarative cartridge: geometry + optics + VibeScript projection + acoustics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeCartridgeManifest {
    pub cartridge_id: String,
    pub title: String,
    pub geometry_blurb: String,
    pub topology: SpatialManifoldTopology,
    pub shader_profile: PostProcessShaderProfile,
    pub vibe_source: String,
    pub acoustic_profile: String,
    pub ink_srgb: [u8; 3],
    pub panel_srgb: [u8; 3],
    pub wcag_contrast_min: f32,
    pub reduced_motion_equivalent: bool,
    pub built_in: bool,
}

/// Registry of built-in plus collected custom cartridges.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ThemeCartridgeRegistry {
    pub cartridges: Vec<ThemeCartridgeManifest>,
    pub active_id: String,
}

impl ThemeCartridgeRegistry {
    pub fn with_builtins() -> Self {
        Self {
            cartridges: builtin_cartridges().to_vec(),
            active_id: "theme-holosonic".into(),
        }
    }

    pub fn get(&self, id: &str) -> Option<&ThemeCartridgeManifest> {
        self.cartridges
            .iter()
            .find(|cartridge| cartridge.cartridge_id == id)
    }

    pub fn active(&self) -> Option<&ThemeCartridgeManifest> {
        self.get(&self.active_id)
    }

    pub fn insert_collected(&mut self, cartridge: ThemeCartridgeManifest) -> Result<(), String> {
        validate_cartridge(&cartridge)?;
        if self
            .cartridges
            .iter()
            .any(|existing| existing.cartridge_id == cartridge.cartridge_id)
        {
            self.cartridges.retain(|existing| existing.cartridge_id != cartridge.cartridge_id);
        }
        self.cartridges.push(cartridge);
        Ok(())
    }

    pub fn activate(&mut self, id: &str) -> Result<&ThemeCartridgeManifest, String> {
        if self.get(id).is_none() {
            return Err("Theme is not in the local collection".into());
        }
        verify_cartridge_affordance_equivalence(self.get(id).unwrap())?;
        self.active_id = id.to_owned();
        Ok(self.get(id).unwrap())
    }
}

/// Six original, IP-clean built-in cartridges.
pub fn builtin_cartridges() -> [ThemeCartridgeManifest; 6] {
    [
        ThemeCartridgeManifest {
            cartridge_id: "theme-holosonic".into(),
            title: "Clear Spatial Desk".into(),
            geometry_blurb: "Planar desktop grid with soft teal depth fog and steady icon kinetics.".into(),
            topology: SpatialManifoldTopology::PlanarDesktopGrid,
            shader_profile: PostProcessShaderProfile::CleanSpatialSrgb,
            vibe_source: HOLOSONIC_VIBE.into(),
            acoustic_profile: "soft-chime".into(),
            ink_srgb: [238, 244, 250],
            panel_srgb: [16, 26, 42],
            wcag_contrast_min: 7.0,
            reduced_motion_equivalent: true,
            built_in: true,
        },
        ThemeCartridgeManifest {
            cartridge_id: "theme-obsidian".into(),
            title: "High-Clarity Night Desk".into(),
            geometry_blurb: "Planar desktop grid with maximum ink-on-panel luminance contrast.".into(),
            topology: SpatialManifoldTopology::PlanarDesktopGrid,
            shader_profile: PostProcessShaderProfile::CleanSpatialSrgb,
            vibe_source: OBSIDIAN_VIBE.into(),
            acoustic_profile: "mute-click".into(),
            ink_srgb: [255, 255, 255],
            panel_srgb: [8, 12, 20],
            wcag_contrast_min: 7.0,
            reduced_motion_equivalent: true,
            built_in: true,
        },
        ThemeCartridgeManifest {
            cartridge_id: "theme-commons".into(),
            title: "Teal Commons Desk".into(),
            geometry_blurb: "Planar desktop grid washed in teal-cyan cooperative lighting.".into(),
            topology: SpatialManifoldTopology::PlanarDesktopGrid,
            shader_profile: PostProcessShaderProfile::CleanSpatialSrgb,
            vibe_source: COMMONS_VIBE.into(),
            acoustic_profile: "warm-pad".into(),
            ink_srgb: [240, 251, 249],
            panel_srgb: [13, 33, 43],
            wcag_contrast_min: 7.0,
            reduced_motion_equivalent: true,
            built_in: true,
        },
        ThemeCartridgeManifest {
            cartridge_id: "theme-holoslate".into(),
            title: "Curved Refractive Glass Arc".into(),
            geometry_blurb: "Concave cylindrical translucent glass panels in depth strata with Fresnel edge refraction, soft bloom, and spring-damper inertial throw.".into(),
            topology: SpatialManifoldTopology::CylindricalGlassArc,
            shader_profile: PostProcessShaderProfile::RefractiveCurvedGlassBloom,
            vibe_source: HOLOSLATE_VIBE.into(),
            acoustic_profile: "glass-tap".into(),
            ink_srgb: [236, 246, 255],
            panel_srgb: [18, 28, 48],
            wcag_contrast_min: 4.5,
            reduced_motion_equivalent: true,
            built_in: true,
        },
        ThemeCartridgeManifest {
            cartridge_id: "theme-cyber-grid".into(),
            title: "3D Neon Wireframe Telemetry Grid".into(),
            geometry_blurb: "Isometric vector grid where volumes rise as glowing translucent wireframe towers with packet light-lanes and optional phosphor scanline filtering.".into(),
            topology: SpatialManifoldTopology::IsometricWireframeCityGrid,
            shader_profile: PostProcessShaderProfile::WireframePhosphorScanline,
            vibe_source: CYBER_GRID_VIBE.into(),
            acoustic_profile: "packet-tick".into(),
            ink_srgb: [180, 255, 210],
            panel_srgb: [4, 12, 10],
            wcag_contrast_min: 4.5,
            reduced_motion_equivalent: true,
            built_in: true,
        },
        ThemeCartridgeManifest {
            cartridge_id: "theme-spatial-commons".into(),
            title: "360° Orbital 10D Volumetric Amphitheatre".into(),
            geometry_blurb: "Full 360-degree orbital spatial commons horizon of architectural 3D SDF pavilions with floating angled diegetic consoles.".into(),
            topology: SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d,
            shader_profile: PostProcessShaderProfile::VolumetricSpectralHdr,
            vibe_source: SPATIAL_COMMONS_VIBE.into(),
            acoustic_profile: "binaural-horizon".into(),
            ink_srgb: [255, 244, 220],
            panel_srgb: [22, 16, 36],
            wcag_contrast_min: 4.5,
            reduced_motion_equivalent: true,
            built_in: true,
        },
    ]
}

const HOLOSONIC_VIBE: &str = r#"
present holosonic {
  topology: planar_desktop_grid
  optics: clean_spatial_srgb
  motion: gentle_icon_omega
}
"#;

const OBSIDIAN_VIBE: &str = r#"
present obsidian {
  topology: planar_desktop_grid
  optics: clean_spatial_srgb
  contrast: aaa
  motion: static_when_requested
}
"#;

const COMMONS_VIBE: &str = r#"
present commons {
  topology: planar_desktop_grid
  optics: teal_cyan_wash
  motion: gentle_icon_omega
}
"#;

const HOLOSLATE_VIBE: &str = r#"
present holoslate {
  topology: cylindrical_glass_arc
  optics: fresnel_bloom
  kinetics: spring_damper_throw
  strata: frosted_depth
}
"#;

const CYBER_GRID_VIBE: &str = r#"
present cyber_grid {
  topology: isometric_wireframe_city
  optics: phosphor_scanline
  kinetics: fly_through_dock
  lanes: packet_light
}
"#;

const SPATIAL_COMMONS_VIBE: &str = r#"
present spatial_commons {
  topology: orbital_volumetric_amphitheatre
  optics: spectral_hdr
  architecture: sdf_pavilions
  consoles: diegetic_angled
}
"#;

/// Project scene-graph nodes onto the cartridge's spatial topology.
pub fn project_semantic_scene_with_cartridge(
    graph: &SemanticSceneGraph,
    cartridge: &ThemeCartridgeManifest,
) -> SemanticSceneGraph {
    let count = graph.nodes.len().max(1);
    let mut projected = graph.clone();
    for (index, node) in projected.nodes.iter_mut().enumerate() {
        node.coords_10d = project_index(cartridge.topology, index, count);
    }
    projected
}

pub fn project_index(topology: SpatialManifoldTopology, index: usize, count: usize) -> [f32; 10] {
    let t = if count <= 1 {
        0.0
    } else {
        index as f32 / (count.saturating_sub(1) as f32)
    };
    let angle = (index as f32 / count as f32) * std::f32::consts::TAU;
    match topology {
        SpatialManifoldTopology::PlanarDesktopGrid => {
            let cols = 4usize;
            let x = (index % cols) as f32 * 0.55 - 0.82;
            let y = (index / cols) as f32 * -0.42 + 0.55;
            [x, y, 0.0, t, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]
        }
        SpatialManifoldTopology::CylindricalGlassArc => {
            let yaw = -1.15 + t * 2.3;
            [
                yaw.sin() * 1.35,
                0.08 * ((index % 3) as f32 - 1.0),
                -yaw.cos() * 1.35 + 0.4,
                yaw,
                0.22,
                0.85,
                1.0,
                0.12,
                0.0,
                1.0,
            ]
        }
        SpatialManifoldTopology::IsometricWireframeCityGrid => {
            let col = (index % 4) as f32;
            let row = (index / 4) as f32;
            let height = 0.35 + (index % 5) as f32 * 0.18;
            [
                (col - row) * 0.42,
                height,
                (col + row) * 0.24,
                t,
                height,
                0.0,
                1.0,
                0.0,
                1.0,
                1.0,
            ]
        }
        SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d => [
            angle.cos() * 1.6,
            0.15 * (index as f32 * 0.7).sin(),
            angle.sin() * 1.6,
            angle,
            0.55,
            0.8,
            1.0,
            0.25,
            t,
            1.0,
        ],
    }
}

/// Every cartridge must keep volume labels, pick targets, and a zero-motion twin.
pub fn verify_cartridge_affordance_equivalence(
    cartridge: &ThemeCartridgeManifest,
) -> Result<(), String> {
    validate_cartridge(cartridge)?;
    if !cartridge.reduced_motion_equivalent {
        return Err("Cartridge must keep a zero-motion static equivalent".into());
    }
    let graph = crate::semantic_scene_graph::project_core_volume_graph();
    let projected = project_semantic_scene_with_cartridge(&graph, cartridge);
    if projected.nodes.len() != graph.nodes.len() {
        return Err("Projection dropped semantic nodes".into());
    }
    for (original, mapped) in graph.nodes.iter().zip(projected.nodes.iter()) {
        if original.semantic_id != mapped.semantic_id || original.label != mapped.label {
            return Err("Projection changed node identity or label".into());
        }
        if original.primary_intent != mapped.primary_intent {
            return Err("Projection changed node intent".into());
        }
        if !mapped.coords_10d.iter().all(|axis| axis.is_finite()) {
            return Err("Projection produced a non-finite coordinate".into());
        }
    }
    let _ = relative_luminance_contrast(cartridge.ink_srgb, cartridge.panel_srgb)?;
    Ok(())
}

pub fn validate_cartridge(cartridge: &ThemeCartridgeManifest) -> Result<(), String> {
    validate_human_centric_host_nomenclature(&cartridge.cartridge_id)?;
    validate_human_centric_host_nomenclature(&cartridge.title)?;
    validate_human_centric_host_nomenclature(&cartridge.geometry_blurb)?;
    validate_human_centric_host_nomenclature(&cartridge.vibe_source)?;
    if !(cartridge.cartridge_id.starts_with("theme-")
        || cartridge.cartridge_id.starts_with("custom-theme-"))
    {
        return Err("Theme cartridge id must use the theme- or custom-theme- prefix".into());
    }
    if cartridge.geometry_blurb.len() < 24 {
        return Err("Describe the theme by its geometric and optical properties".into());
    }
    if cartridge.wcag_contrast_min < 4.5 {
        return Err("Theme contrast floor is below WCAG AA 4.5:1".into());
    }
    Ok(())
}

pub fn relative_luminance_srgb(channel: u8) -> f32 {
    let c = f32::from(channel) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn relative_luminance(rgb: [u8; 3]) -> f32 {
    0.2126 * relative_luminance_srgb(rgb[0])
        + 0.7152 * relative_luminance_srgb(rgb[1])
        + 0.0722 * relative_luminance_srgb(rgb[2])
}

pub fn contrast_ratio(a: [u8; 3], b: [u8; 3]) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (hi + 0.05) / (lo + 0.05)
}

pub fn relative_luminance_contrast(ink: [u8; 3], panel: [u8; 3]) -> Result<f32, String> {
    let ratio = contrast_ratio(ink, panel);
    if ratio + f32::EPSILON < 4.5 {
        Err(format!("Contrast {ratio:.2}:1 is below WCAG AA 4.5:1"))
    } else {
        Ok(ratio)
    }
}

/// Nudge ink/panel until the requested WCAG floor is met. Fail-closed if impossible.
pub fn auto_repair_wcag_contrast(
    mut ink: [u8; 3],
    mut panel: [u8; 3],
    floor: f32,
) -> Result<([u8; 3], [u8; 3], f32), String> {
    let floor = floor.max(4.5);
    for _ in 0..24 {
        let ratio = contrast_ratio(ink, panel);
        if ratio >= floor {
            return Ok((ink, panel, ratio));
        }
        if relative_luminance(ink) >= relative_luminance(panel) {
            ink = ink.map(|c| c.saturating_add(12).min(255));
            panel = panel.map(|c| c.saturating_sub(12));
        } else {
            ink = ink.map(|c| c.saturating_sub(12));
            panel = panel.map(|c| c.saturating_add(12).min(255));
        }
    }
    let ratio = contrast_ratio(ink, panel);
    if ratio >= floor {
        Ok((ink, panel, ratio))
    } else {
        Err("Could not repair theme contrast to the requested WCAG floor".into())
    }
}

/// Affordance check used by live VibeScript projection of a node set.
pub fn projected_nodes_keep_labels(
    original: &[SemanticUiNode],
    projected: &[SemanticUiNode],
) -> bool {
    original.len() == projected.len()
        && original
            .iter()
            .zip(projected)
            .all(|(a, b)| a.semantic_id == b.semantic_id && a.label == b.label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_builtins_pass_affordance_and_wcag() {
        let registry = ThemeCartridgeRegistry::with_builtins();
        assert_eq!(registry.cartridges.len(), 6);
        for cartridge in &registry.cartridges {
            verify_cartridge_affordance_equivalence(cartridge)
                .unwrap_or_else(|err| panic!("{}: {err}", cartridge.cartridge_id));
            assert!(contrast_ratio(cartridge.ink_srgb, cartridge.panel_srgb) >= 4.5);
        }
    }

    #[test]
    fn cylindrical_arc_places_nodes_on_a_curve() {
        let cartridge = builtin_cartridges()
            .into_iter()
            .find(|c| c.cartridge_id == "theme-holoslate")
            .unwrap();
        let graph = crate::semantic_scene_graph::project_core_volume_graph();
        let projected = project_semantic_scene_with_cartridge(&graph, &cartridge);
        let xs: Vec<f32> = projected.nodes.iter().map(|n| n.coords_10d[0]).collect();
        assert!(xs.iter().any(|x| *x < -0.2));
        assert!(xs.iter().any(|x| *x > 0.2));
    }

    #[test]
    fn auto_repair_lifts_weak_contrast() {
        let (ink, panel, ratio) = auto_repair_wcag_contrast([120, 120, 120], [100, 100, 100], 4.5)
            .expect("repair");
        assert!(ratio >= 4.5);
        assert_ne!(ink, panel);
    }
}
