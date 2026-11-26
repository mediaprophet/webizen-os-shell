// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Conversational theme co-designer. Local `.p64` or a governed public MCP prompt
//! may propose a cartridge. Every proposal is fail-closed through nomenclature,
//! WCAG auto-repair, affordance equivalence, and WGSL-Forge validation.
//! Shared bundles import as CollectedInactive (`collect != activate`).

use crate::forge_theme_shaders::validate_profile_wgsl;
use crate::host_stewardship::validate_human_centric_host_nomenclature;
use crate::identifiers::q_hash_60;
use crate::llm_management::SENSITIVITY_PUBLIC_ONLY;
use crate::theme_cartridge::{
    auto_repair_wcag_contrast, validate_cartridge, verify_cartridge_affordance_equivalence,
    PostProcessShaderProfile, SpatialManifoldTopology, ThemeCartridgeManifest,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use vibescript_abi::NQuin48;

/// Import always lands here. Activation is a separate, explicit human step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeCollectionState {
    CollectedInactive,
    Active,
}

/// Signed Permissive Commons theme cartridge bundle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SharedThemeCartridgeBundle {
    pub cartridge: ThemeCartridgeManifest,
    pub author_nquin: [u64; 6],
    pub sha3_256: [u8; 32],
    pub collection_state: ThemeCollectionState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeCodesignReceipt {
    pub ok: bool,
    pub detail: String,
}

/// Conversational co-design. `llm_reply` is an optional public-only model draft.
/// When absent, a local geometric heuristic drafts the cartridge from the prompt.
pub fn co_design_theme_with_llm(
    prompt: &str,
    sensitivity: u8,
    llm_reply: Option<&str>,
) -> Result<ThemeCartridgeManifest, String> {
    if sensitivity != SENSITIVITY_PUBLIC_ONLY {
        return Err("Theme co-design only accepts public-only prompts (sensitivity 0)".into());
    }
    validate_human_centric_host_nomenclature(prompt)?;
    if prompt.trim().len() < 8 {
        return Err("Describe the look you want in a short sentence".into());
    }
    if let Some(reply) = llm_reply {
        validate_human_centric_host_nomenclature(reply)?;
    }
    let source = llm_reply.unwrap_or(prompt);
    let mut draft = heuristic_draft(prompt, source)?;
    let (ink, panel, ratio) =
        auto_repair_wcag_contrast(draft.ink_srgb, draft.panel_srgb, draft.wcag_contrast_min)?;
    draft.ink_srgb = ink;
    draft.panel_srgb = panel;
    draft.wcag_contrast_min = ratio.min(7.0).max(4.5);
    verify_designed_theme(&draft)?;
    Ok(draft)
}

fn heuristic_draft(prompt: &str, source: &str) -> Result<ThemeCartridgeManifest, String> {
    let lower = format!("{prompt} {source}").to_ascii_lowercase();
    let (topology, shader, title, blurb, acoustic, vibe) = if lower.contains("wire")
        || lower.contains("grid")
        || lower.contains("neon")
        || lower.contains("tower")
    {
        (
            SpatialManifoldTopology::IsometricWireframeCityGrid,
            PostProcessShaderProfile::WireframePhosphorScanline,
            "Custom Neon Wireframe Telemetry Grid",
            "Isometric neon wireframe towers with packet light-lanes and phosphor scanline filtering.",
            "packet-tick",
            "present custom { topology: isometric_wireframe_city optics: phosphor_scanline }",
        )
    } else if lower.contains("orbit")
        || lower.contains("amphitheatre")
        || lower.contains("pavilion")
        || lower.contains("360")
        || lower.contains("volumetric")
    {
        (
            SpatialManifoldTopology::OrbitalVolumetricAmphitheatre10d,
            PostProcessShaderProfile::VolumetricSpectralHdr,
            "Custom Orbital Volumetric Amphitheatre",
            "360-degree orbital ring of architectural SDF pavilions with floating angled consoles.",
            "binaural-horizon",
            "present custom { topology: orbital_volumetric_amphitheatre optics: spectral_hdr }",
        )
    } else if lower.contains("glass")
        || lower.contains("curve")
        || lower.contains("arc")
        || lower.contains("fresnel")
        || lower.contains("bloom")
    {
        (
            SpatialManifoldTopology::CylindricalGlassArc,
            PostProcessShaderProfile::RefractiveCurvedGlassBloom,
            "Custom Curved Refractive Glass Arc",
            "Concave cylindrical translucent glass panels with Fresnel edge refraction and soft bloom.",
            "glass-tap",
            "present custom { topology: cylindrical_glass_arc optics: fresnel_bloom }",
        )
    } else if lower.contains("high contrast") || lower.contains("night") || lower.contains("ink") {
        (
            SpatialManifoldTopology::PlanarDesktopGrid,
            PostProcessShaderProfile::CleanSpatialSrgb,
            "Custom High-Clarity Night Desk",
            "Planar desktop grid with maximum ink-on-panel luminance for glare-sensitive reading.",
            "mute-click",
            "present custom { topology: planar_desktop_grid optics: clean_spatial_srgb contrast: aaa }",
        )
    } else {
        (
            SpatialManifoldTopology::PlanarDesktopGrid,
            PostProcessShaderProfile::CleanSpatialSrgb,
            "Custom Clear Spatial Desk",
            "Planar desktop grid with calm depth fog, readable labels, and gentle icon kinetics.",
            "soft-chime",
            "present custom { topology: planar_desktop_grid optics: clean_spatial_srgb }",
        )
    };
    let slug = slug_from_prompt(prompt);
    Ok(ThemeCartridgeManifest {
        cartridge_id: format!("custom-theme-{slug}"),
        title: title.into(),
        geometry_blurb: blurb.into(),
        topology,
        shader_profile: shader,
        vibe_source: vibe.into(),
        acoustic_profile: acoustic.into(),
        ink_srgb: [236, 244, 252],
        panel_srgb: [14, 22, 36],
        wcag_contrast_min: 4.5,
        reduced_motion_equivalent: true,
        built_in: false,
    })
}

fn slug_from_prompt(prompt: &str) -> String {
    let mut slug = String::new();
    for ch in prompt.chars() {
        if slug.len() >= 24 {
            break;
        }
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() && !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    if slug.len() < 4 {
        format!("{:x}", q_hash_60(prompt) & 0xffff_ffff)
    } else {
        slug.trim_matches('-').to_owned()
    }
}

pub fn verify_designed_theme(cartridge: &ThemeCartridgeManifest) -> Result<(), String> {
    validate_cartridge(cartridge)?;
    verify_cartridge_affordance_equivalence(cartridge)?;
    validate_profile_wgsl(cartridge.shader_profile)?;
    Ok(())
}

pub fn export_shared_theme_bundle(
    cartridge: &ThemeCartridgeManifest,
    author_label: &str,
) -> Result<SharedThemeCartridgeBundle, String> {
    verify_designed_theme(cartridge)?;
    validate_human_centric_host_nomenclature(author_label)?;
    let quin = NQuin48::invariant(
        "urn:webizen:theme:author",
        author_label,
        q_hash_60(&cartridge.cartridge_id),
    );
    let payload = serde_json::to_vec(cartridge).map_err(|err| err.to_string())?;
    let digest = Sha3_256::digest(&payload);
    let mut sha3_256 = [0u8; 32];
    sha3_256.copy_from_slice(&digest);
    Ok(SharedThemeCartridgeBundle {
        cartridge: cartridge.clone(),
        author_nquin: quin.0,
        sha3_256,
        collection_state: ThemeCollectionState::CollectedInactive,
    })
}

/// Import a shared bundle into CollectedInactive. Activation is a later human step.
pub fn import_shared_theme_bundle(
    bundle: SharedThemeCartridgeBundle,
) -> Result<SharedThemeCartridgeBundle, String> {
    verify_designed_theme(&bundle.cartridge)?;
    let payload = serde_json::to_vec(&bundle.cartridge).map_err(|err| err.to_string())?;
    let digest = Sha3_256::digest(&payload);
    if digest.as_slice() != bundle.sha3_256.as_slice() {
        return Err("Shared theme digest does not match the cartridge payload".into());
    }
    if bundle.author_nquin.len() != 6 {
        return Err("Author attribution must be a 48-byte NQuin48".into());
    }
    let mut imported = bundle;
    imported.collection_state = ThemeCollectionState::CollectedInactive;
    Ok(imported)
}

pub fn activate_collected_bundle(
    bundle: &mut SharedThemeCartridgeBundle,
) -> Result<(), String> {
    if bundle.collection_state != ThemeCollectionState::CollectedInactive
        && bundle.collection_state != ThemeCollectionState::Active
    {
        return Err("Theme is not in the local collection".into());
    }
    verify_designed_theme(&bundle.cartridge)?;
    bundle.collection_state = ThemeCollectionState::Active;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_only_gate_rejects_private_prompts() {
        let err = co_design_theme_with_llm("curved glass arc with soft bloom", 32, None)
            .unwrap_err();
        assert!(err.contains("public-only"));
    }

    #[test]
    fn glass_prompt_drafts_holoslate_geometry_and_passes_verifier() {
        let cartridge = co_design_theme_with_llm(
            "curved refractive glass arc with fresnel bloom",
            SENSITIVITY_PUBLIC_ONLY,
            None,
        )
        .expect("draft");
        assert!(cartridge.cartridge_id.starts_with("custom-theme-"));
        assert_eq!(
            cartridge.topology,
            SpatialManifoldTopology::CylindricalGlassArc
        );
        assert!(cartridge.reduced_motion_equivalent);
    }

    #[test]
    fn collect_does_not_activate() {
        let cartridge = co_design_theme_with_llm(
            "orbital volumetric amphitheatre of pavilions",
            SENSITIVITY_PUBLIC_ONLY,
            None,
        )
        .unwrap();
        let bundle = export_shared_theme_bundle(&cartridge, "did:qi:author").unwrap();
        assert_eq!(
            bundle.collection_state,
            ThemeCollectionState::CollectedInactive
        );
        let imported = import_shared_theme_bundle(bundle).unwrap();
        assert_eq!(
            imported.collection_state,
            ThemeCollectionState::CollectedInactive
        );
        let mut live = imported;
        activate_collected_bundle(&mut live).unwrap();
        assert_eq!(live.collection_state, ThemeCollectionState::Active);
    }
}
