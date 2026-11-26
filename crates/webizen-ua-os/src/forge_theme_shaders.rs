// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Runtime shader synthesis for spatial theme post-process profiles.
//! Sources are validated with `etalon_wgsl_forge::validate_wgsl`.

use crate::theme_cartridge::PostProcessShaderProfile;
use etalon_wgsl_forge::validate_wgsl;

pub const REFRACTIVE_CURVED_GLASS_BLOOM: &str = r#"
struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var out: VsOut;
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let p = positions[vertex_index];
    out.position = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_samp: sampler;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let bend = (uv.x - 0.5) * (uv.x - 0.5) * 0.18;
    let sample_uv = vec2<f32>(uv.x, clamp(uv.y + bend, 0.0, 1.0));
    let base = textureSample(source_tex, source_samp, sample_uv);
    let fresnel = pow(abs(uv.x - 0.5) * 2.0, 2.2);
    let bloom = vec3<f32>(0.55, 0.78, 1.0) * fresnel * 0.22;
    return vec4<f32>(base.rgb + bloom, 1.0);
}
"#;

pub const WIREFRAME_PHOSPHOR_SCANLINE: &str = r#"
struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var out: VsOut;
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let p = positions[vertex_index];
    out.position = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_samp: sampler;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let base = textureSample(source_tex, source_samp, uv);
    let scan = 0.82 + 0.18 * step(0.5, fract(uv.y * 240.0));
    let phosphor = vec3<f32>(0.35, 1.0, 0.62);
    let tinted = mix(base.rgb, base.rgb * phosphor, 0.35) * scan;
    return vec4<f32>(tinted, 1.0);
}
"#;

pub const VOLUMETRIC_SPECTRAL_HDR: &str = r#"
struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
    var out: VsOut;
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let p = positions[vertex_index];
    out.position = vec4<f32>(p, 0.0, 1.0);
    out.uv = p * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    return out;
}

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_samp: sampler;

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let base = textureSample(source_tex, source_samp, uv);
    let radial = length(uv - vec2<f32>(0.5, 0.5));
    let spectral = vec3<f32>(
        0.55 + 0.45 * cos(radial * 12.0),
        0.45 + 0.40 * cos(radial * 12.0 + 2.1),
        0.70 + 0.30 * cos(radial * 12.0 + 4.2),
    );
    let hdr = base.rgb * 1.15 + spectral * 0.08 * (1.0 - radial);
    return vec4<f32>(hdr, 1.0);
}
"#;

pub fn shader_source_for_profile(profile: PostProcessShaderProfile) -> &'static str {
    match profile {
        PostProcessShaderProfile::CleanSpatialSrgb => REFRACTIVE_CURVED_GLASS_BLOOM,
        PostProcessShaderProfile::RefractiveCurvedGlassBloom => REFRACTIVE_CURVED_GLASS_BLOOM,
        PostProcessShaderProfile::WireframePhosphorScanline => WIREFRAME_PHOSPHOR_SCANLINE,
        PostProcessShaderProfile::VolumetricSpectralHdr => VOLUMETRIC_SPECTRAL_HDR,
    }
}

pub fn validate_profile_wgsl(profile: PostProcessShaderProfile) -> Result<(), String> {
    let source = shader_source_for_profile(profile);
    validate_wgsl(source).map(|_| ()).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_theme_shaders_validate() {
        for profile in [
            PostProcessShaderProfile::RefractiveCurvedGlassBloom,
            PostProcessShaderProfile::WireframePhosphorScanline,
            PostProcessShaderProfile::VolumetricSpectralHdr,
        ] {
            validate_profile_wgsl(profile)
                .unwrap_or_else(|err| panic!("{profile:?}: {err}"));
        }
    }
}
