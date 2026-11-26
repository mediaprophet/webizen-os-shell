// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Zero-heap procedural acoustic and binaural UI feedback for spatial themes.
//! Hot path uses stack arrays and `etalon-vibe` SIGNAL_PROVIDER oracles.

use crate::theme_cartridge::SpatialManifoldTopology;
use etalon_vibe::{
    oracle_biquad_df1_step, oracle_render_binaural_frame, oracle_sonify_emf_envelope,
};

/// One stereo output sample plus interaural time, produced without heap growth.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeAcousticFrame {
    pub left: f32,
    pub right: f32,
    pub itd_sec: f32,
}

pub fn acoustic_pose_for_topology(
    topology: SpatialManifoldTopology,
    azimuth_deg: f32,
) -> [f32; 3] {
    let azimuth = azimuth_deg.clamp(-90.0, 90.0);
    match topology {
        SpatialManifoldTopology::PlanarDesktopGrid => [0.0, 0.0, 1.2],
        SpatialManifoldTopology::CylindricalGlassArc => [azimuth * 0.45, 4.0, 1.6],
        SpatialManifoldTopology::IsometricWireframeCityGrid => [azimuth * 0.7, -6.0, 2.2],
        SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d => [azimuth, 8.0, 2.8],
    }
}

/// Deterministic binaural UI tick for a theme topology. No per-frame allocation.
pub fn render_theme_acoustic_frame(
    topology: SpatialManifoldTopology,
    azimuth_deg: f32,
    gain: f32,
) -> Result<ThemeAcousticFrame, String> {
    let pose = acoustic_pose_for_topology(topology, azimuth_deg);
    let binaural = oracle_render_binaural_frame(pose, gain.clamp(0.05, 1.0))
        .map_err(|err| format!("{err:?}"))?;
    let filtered = oracle_biquad_df1_step(
        [binaural[0], 0.0, 0.0],
        [0.0, 0.0],
        [0.8, 0.0, 0.0, -0.2, 0.0],
    )
    .map_err(|err| format!("{err:?}"))?;
    let envelope = oracle_sonify_emf_envelope([550.0, 0.2, 0.4]).map_err(|err| format!("{err:?}"))?;
    Ok(ThemeAcousticFrame {
        left: (filtered as f64 * envelope[0]).clamp(-1.0, 1.0) as f32,
        right: (binaural[1] as f64 * envelope[1]).clamp(-1.0, 1.0) as f32,
        itd_sec: binaural[2],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binaural_frames_are_finite_and_sided() {
        let leftish = render_theme_acoustic_frame(
            SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d,
            -60.0,
            1.0,
        )
        .unwrap();
        let rightish = render_theme_acoustic_frame(
            SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d,
            60.0,
            1.0,
        )
        .unwrap();
        assert!(leftish.left.is_finite() && leftish.right.is_finite());
        assert_ne!(leftish.itd_sec, rightish.itd_sec);
    }
}
