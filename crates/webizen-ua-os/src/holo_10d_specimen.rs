// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Webizen's compact .10d v2 specimen profile for the native shader studio.
//!
//! This profile stores a HOLO projection header rather than QualiaDB's section
//! table and segment bodies. Its checksums validate the declared projection
//! inputs. Imported QualiaDB scene containers require the canonical resolver.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use vibescript_abi::NQuin48;

use crate::identifiers::q_hash_60;

pub const FIELD_KIND_HOLO_PHASE_SPACE: u16 = 0x0306;
pub const HOLO_HEADER_BYTES: usize = 64;
pub const HOLO_MAX_RAYMARCH_STEPS: u32 = 32;

const CONTAINER_MAGIC: &[u8; 8] = b"10Dv2\0\0\0";
const SECTION_MAGIC: &[u8; 4] = b"HOLO";
const VERSION: u16 = 2;
const SECTION_OFFSET: usize = 10;
const CHECKSUM_OFFSET: usize = SECTION_OFFSET + 60;
const CONTAINER_BYTES: usize = SECTION_OFFSET + HOLO_HEADER_BYTES;
const MAX_HARMONIC_ORDER: u8 = 12;
const MAX_HOLO_SEGMENTS: u16 = 4096;
const MAX_WAVEFIELDS: u16 = 1024;
const SPECIMEN_DIRECTORY: &str = "library/specimens-10d";

pub const DEFAULT_HOLO_SPECIMENS: [&str; 4] = [
    "chora-spatiotemporal-commons",
    "anatomy-cardiovascular-holo",
    "ecopunk-riparian-biosphere",
    "poet-hyperbolic-mobius-v2",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoloShaderVisualizationMode {
    PhaseSpaceWavefield,
    HyperbolicMobiusV2,
    AnatomicalRadialDepth,
    ScaleSpaceHarmonicSlice,
}

impl HoloShaderVisualizationMode {
    pub const ALL: [Self; 4] = [
        Self::PhaseSpaceWavefield,
        Self::HyperbolicMobiusV2,
        Self::AnatomicalRadialDepth,
        Self::ScaleSpaceHarmonicSlice,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::PhaseSpaceWavefield => "phase-space-wavefield",
            Self::HyperbolicMobiusV2 => "hyperbolic-mobius-v2",
            Self::AnatomicalRadialDepth => "anatomical-radial-depth",
            Self::ScaleSpaceHarmonicSlice => "scale-space-harmonic-slice",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::PhaseSpaceWavefield => "Phase-space wavefield",
            Self::HyperbolicMobiusV2 => "Hyperbolic Möbius v2",
            Self::AnatomicalRadialDepth => "Anatomical radial depth",
            Self::ScaleSpaceHarmonicSlice => "Scale-space harmonic slice",
        }
    }

    pub const fn shader_mode_index(self) -> u32 {
        match self {
            Self::PhaseSpaceWavefield => 0,
            Self::HyperbolicMobiusV2 => 1,
            Self::AnatomicalRadialDepth => 2,
            Self::ScaleSpaceHarmonicSlice => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VerifiedHolo10dSpecimen {
    pub specimen_id: String,
    pub title: String,
    pub source_path: Option<PathBuf>,
    pub field_kind: u16,
    pub version: u16,
    pub holo_segment_count: u16,
    pub wavefield_count: u16,
    pub max_harmonic_order: u8,
    pub max_raymarch_steps: u8,
    pub proxy_aabb_min: [f32; 3],
    pub proxy_aabb_max: [f32; 3],
    pub sdf_smooth_union_radius: f32,
    pub reference_sigma: f32,
    pub default_shader_mode: HoloShaderVisualizationMode,
    pub nquin_subject_hash: u64,
    pub sealed_nquin_parity: u64,
    pub sha3_256_hex: String,
}

pub fn encode_holo_10d_container(spec: &VerifiedHolo10dSpecimen) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(CONTAINER_BYTES);
    bytes.extend_from_slice(CONTAINER_MAGIC);
    bytes.extend_from_slice(&spec.field_kind.to_le_bytes());
    bytes.extend_from_slice(SECTION_MAGIC);
    bytes.extend_from_slice(&spec.version.to_le_bytes());
    bytes.extend_from_slice(&spec.holo_segment_count.to_le_bytes());
    bytes.extend_from_slice(&spec.wavefield_count.to_le_bytes());
    bytes.push(spec.max_harmonic_order);
    bytes.push(spec.max_raymarch_steps);
    for value in spec.proxy_aabb_min.into_iter().chain(spec.proxy_aabb_max) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&spec.sdf_smooth_union_radius.to_le_bytes());
    bytes.extend_from_slice(&spec.reference_sigma.to_le_bytes());
    bytes.extend_from_slice(&spec.nquin_subject_hash.to_le_bytes());
    bytes.push(spec.default_shader_mode.shader_mode_index() as u8);
    bytes.resize(CHECKSUM_OFFSET, 0);
    let tag = Sha3_256::digest(&bytes);
    bytes.extend_from_slice(&tag[..4]);
    bytes
}

pub fn decode_holo_10d_container(
    specimen_id: &str,
    bytes: &[u8],
) -> Result<VerifiedHolo10dSpecimen, String> {
    let specimen_id = specimen_slug(specimen_id)?;
    let bytes: &[u8; CONTAINER_BYTES] = bytes
        .try_into()
        .map_err(|_| "The specimen does not use the Webizen .10d v2 header profile.")?;
    if &bytes[..8] != CONTAINER_MAGIC || &bytes[SECTION_OFFSET..SECTION_OFFSET + 4] != SECTION_MAGIC
    {
        return Err("The Webizen .10d or HOLO signature is invalid.".into());
    }
    let field_kind = u16::from_le_bytes([bytes[8], bytes[9]]);
    let header = &bytes[SECTION_OFFSET..];
    let u16_at = |i| u16::from_le_bytes([header[i], header[i + 1]]);
    let f32_at = |i| f32::from_le_bytes([header[i], header[i + 1], header[i + 2], header[i + 3]]);
    let version = u16_at(4);
    if field_kind != FIELD_KIND_HOLO_PHASE_SPACE || version != VERSION {
        return Err("The Webizen .10d profile requires a v2 HOLO (0x0306) section.".into());
    }
    let tag = Sha3_256::digest(&bytes[..CHECKSUM_OFFSET]);
    if bytes[CHECKSUM_OFFSET..] != tag[..4] || header[53..60] != [0; 7] {
        return Err("The HOLO integrity tag or reserved header bytes are invalid.".into());
    }

    let holo_segment_count = u16_at(6);
    let wavefield_count = u16_at(8);
    let max_harmonic_order = header[10];
    let max_raymarch_steps = header[11];
    if holo_segment_count > MAX_HOLO_SEGMENTS
        || wavefield_count > MAX_WAVEFIELDS
        || max_harmonic_order > MAX_HARMONIC_ORDER
        || !(1..=HOLO_MAX_RAYMARCH_STEPS).contains(&u32::from(max_raymarch_steps))
    {
        return Err("The HOLO counts or projection budgets exceed the profile bounds.".into());
    }

    let proxy_aabb_min = [f32_at(12), f32_at(16), f32_at(20)];
    let proxy_aabb_max = [f32_at(24), f32_at(28), f32_at(32)];
    if !proxy_aabb_min
        .into_iter()
        .zip(proxy_aabb_max)
        .all(|(min, max)| min.is_finite() && max.is_finite() && min <= max)
    {
        return Err("The HOLO proxy bounding box is invalid.".into());
    }

    let sdf_smooth_union_radius = f32_at(36);
    let reference_sigma = f32_at(40);
    if !sdf_smooth_union_radius.is_finite()
        || sdf_smooth_union_radius < 0.0
        || !(0.0..=1.0).contains(&reference_sigma)
    {
        return Err("The HOLO blending radius or reference sigma is invalid.".into());
    }
    let nquin_subject_hash = u64::from_le_bytes(header[44..52].try_into().unwrap());
    let default_shader_mode = HoloShaderVisualizationMode::ALL
        .get(usize::from(header[52]))
        .copied()
        .ok_or("The HOLO default visualization mode is unknown.")?;
    let sha3_256_hex = format!("{:x}", Sha3_256::digest(bytes));
    let quin = NQuin48::from_parts(
        nquin_subject_hash,
        q_hash_60("urn:webizen:specimen:verified-holo-header"),
        q_hash_60(&sha3_256_hex),
        u64::from(field_kind),
        u64::from(version),
    );
    let title = DEFAULT_HOLO_SPECIMENS
        .iter()
        .position(|&candidate| candidate == specimen_id)
        .map_or_else(
            || specimen_id.to_owned(),
            |index| DEFAULT_TITLES[index].into(),
        );

    Ok(VerifiedHolo10dSpecimen {
        specimen_id: specimen_id.into(),
        title,
        source_path: None,
        field_kind,
        version,
        holo_segment_count,
        wavefield_count,
        max_harmonic_order,
        max_raymarch_steps,
        proxy_aabb_min,
        proxy_aabb_max,
        sdf_smooth_union_radius,
        reference_sigma,
        default_shader_mode,
        nquin_subject_hash,
        sealed_nquin_parity: quin.parity(),
        sha3_256_hex,
    })
}

const DEFAULT_TITLES: [&str; 4] = [
    "Chora Spatio-Temporal Commons",
    "Anatomy Cardiovascular",
    "Ecopunk Riparian Biosphere",
    "Poet Hyperbolic Möbius v2",
];

fn default_specimen(index: usize) -> VerifiedHolo10dSpecimen {
    use HoloShaderVisualizationMode::*;
    let (extent, mode, wavefield_count, sigma) = [
        ([0.95, 0.65, 0.85], PhaseSpaceWavefield, 4, 0.48),
        ([0.55, 0.88, 0.50], AnatomicalRadialDepth, 0, 0.60),
        ([0.90, 0.45, 0.90], ScaleSpaceHarmonicSlice, 3, 0.73),
        ([0.75, 0.75, 0.75], HyperbolicMobiusV2, 2, 0.81),
    ][index];
    let id = DEFAULT_HOLO_SPECIMENS[index];
    VerifiedHolo10dSpecimen {
        specimen_id: id.into(),
        title: DEFAULT_TITLES[index].into(),
        source_path: None,
        field_kind: FIELD_KIND_HOLO_PHASE_SPACE,
        version: VERSION,
        holo_segment_count: 12,
        wavefield_count,
        max_harmonic_order: 4,
        max_raymarch_steps: HOLO_MAX_RAYMARCH_STEPS as u8,
        proxy_aabb_min: extent.map(|value| -value),
        proxy_aabb_max: extent,
        sdf_smooth_union_radius: 0.08,
        reference_sigma: sigma,
        default_shader_mode: mode,
        nquin_subject_hash: q_hash_60(id),
        sealed_nquin_parity: 0,
        sha3_256_hex: String::new(),
    }
}

pub fn seed_default_holo_10d_specimens(
    storage: &Path,
) -> Result<Vec<VerifiedHolo10dSpecimen>, String> {
    let directory = specimen_directory(storage, true)?;
    (0..DEFAULT_HOLO_SPECIMENS.len())
        .map(|index| seed_default_specimen(&directory, index))
        .collect()
}

pub fn resolve_holo_10d_specimen(
    storage: &Path,
    query_or_uri: &str,
) -> Result<VerifiedHolo10dSpecimen, String> {
    let input = query_or_uri.trim();
    if input.len() > 4096 || input.chars().any(char::is_control) {
        return Err("The specimen address is invalid.".into());
    }
    let query = input.strip_prefix("holo:").unwrap_or(input);
    let local_name = if let Some(uri) = query
        .strip_prefix("qualia://")
        .or_else(|| query.strip_prefix("webizen://"))
    {
        let route = uri.split(['?', '#']).next().unwrap_or_default();
        match route {
            "chora/universe" => DEFAULT_HOLO_SPECIMENS[0],
            _ => specimen_slug(
                route
                    .strip_prefix("specimens/")
                    .ok_or("The native route has no registered shader-studio specimen.")?,
            )?,
        }
    } else {
        query
    };
    let path = Path::new(local_name);
    let is_path = path.is_absolute() || path.components().count() > 1;
    let file_name = if is_path {
        path.file_name()
            .and_then(|name| name.to_str())
            .ok_or("The specimen path must name a .10d file.")?
    } else {
        local_name
    };
    let slug = specimen_slug(file_name)?;
    let seed_index = (!is_path)
        .then(|| DEFAULT_HOLO_SPECIMENS.iter().position(|&name| name == slug))
        .flatten();
    let directory = specimen_directory(storage, seed_index.is_some())?;
    let file = directory.join(format!("{slug}.10d"));
    if is_path {
        if file_name != format!("{slug}.10d") {
            return Err("The specimen path must name a .10d file.".into());
        }
        let requested = if path.is_absolute() {
            path.to_owned()
        } else {
            storage.join(path)
        };
        let storage_directory = storage.join(SPECIMEN_DIRECTORY);
        let opened = requested
            .strip_prefix(&directory)
            .or_else(|_| requested.strip_prefix(&storage_directory));
        if !opened.is_ok_and(|relative| relative == Path::new(file_name)) {
            return Err(
                "Open specimen files from the shader studio's local specimen directory.".into(),
            );
        }
    }
    match seed_index {
        Some(index) => seed_default_specimen(&directory, index),
        None => read_specimen(slug, &file),
    }
}

pub fn proxy_corners_for_specimen(spec: &VerifiedHolo10dSpecimen) -> [[f32; 3]; 8] {
    let [x0, y0, z0] = spec.proxy_aabb_min;
    let [x1, y1, z1] = spec.proxy_aabb_max;
    [
        [x0, y0, z0],
        [x1, y0, z0],
        [x1, y1, z0],
        [x0, y1, z0],
        [x0, y0, z1],
        [x1, y0, z1],
        [x1, y1, z1],
        [x0, y1, z1],
    ]
}

fn specimen_slug(name: &str) -> Result<&str, String> {
    let slug = name.strip_suffix(".10d").unwrap_or(name);
    if slug.is_empty()
        || slug.len() > 96
        || !slug
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_'))
    {
        return Err("The specimen needs a local identifier of up to 96 letters, digits, hyphens, or underscores.".into());
    }
    Ok(slug)
}

fn specimen_directory(storage: &Path, create: bool) -> Result<PathBuf, String> {
    if create {
        fs::create_dir_all(storage).map_err(|error| error.to_string())?;
    }
    let mut directory = fs::canonicalize(storage).map_err(|error| error.to_string())?;
    for part in SPECIMEN_DIRECTORY.split('/') {
        directory.push(part);
        if create {
            match fs::create_dir(&directory) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        let metadata = fs::symlink_metadata(&directory).map_err(|error| error.to_string())?;
        if !metadata.is_dir() || is_reparse_point(&metadata) {
            return Err("The specimen directory must be a regular local directory.".into());
        }
    }
    Ok(directory)
}

fn read_specimen(slug: &str, file: &Path) -> Result<VerifiedHolo10dSpecimen, String> {
    let metadata = fs::symlink_metadata(file).map_err(|error| error.to_string())?;
    if !metadata.is_file()
        || is_reparse_point(&metadata)
        || metadata.len() != CONTAINER_BYTES as u64
    {
        return Err("The specimen needs a regular file using the Webizen .10d profile.".into());
    }
    let mut bytes = Vec::with_capacity(CONTAINER_BYTES);
    File::open(file)
        .map_err(|error| error.to_string())?
        .take((CONTAINER_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let mut spec = decode_holo_10d_container(slug, &bytes)?;
    spec.source_path = Some(file.to_owned());
    Ok(spec)
}

fn seed_default_specimen(
    directory: &Path,
    index: usize,
) -> Result<VerifiedHolo10dSpecimen, String> {
    let id = DEFAULT_HOLO_SPECIMENS[index];
    let path = directory.join(format!("{id}.10d"));
    match fs::symlink_metadata(&path) {
        Ok(_) => return read_specimen(id, &path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);
    let temp = directory.join(format!(
        ".{id}-{}-{}.tmp",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| error.to_string())?;
    let stored = file
        .write_all(&encode_holo_10d_container(&default_specimen(index)))
        .and_then(|()| file.sync_all());
    drop(file);
    let published = stored.and_then(|()| match fs::hard_link(&temp, &path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    });
    let cleaned = fs::remove_file(&temp);
    published.and(cleaned).map_err(|error| error.to_string())?;
    read_specimen(id, &path)
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn encodes_decodes_and_resolves_four_holo_10d_v2_specimens() {
        let storage = std::env::temp_dir().join(format!(
            "webizen-holo-specimen-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let specimens = seed_default_holo_10d_specimens(&storage).unwrap();
        assert_eq!(specimens.len(), 4);
        assert_eq!(
            specimens,
            seed_default_holo_10d_specimens(&storage).unwrap()
        );
        for (id, specimen) in DEFAULT_HOLO_SPECIMENS.into_iter().zip(specimens) {
            let file = specimen.source_path.as_ref().unwrap();
            let bytes = fs::read(file).unwrap();
            assert_eq!(&bytes[..8], CONTAINER_MAGIC);
            assert_eq!(&bytes[10..14], SECTION_MAGIC);
            assert_eq!(encode_holo_10d_container(&specimen), bytes);
            let decoded = decode_holo_10d_container(id, &bytes).unwrap();
            assert_eq!(decoded.sha3_256_hex.len(), 64);
            assert_eq!(decoded.sealed_nquin_parity, specimen.sealed_nquin_parity);
            assert_eq!(
                resolve_holo_10d_specimen(&storage, file.to_str().unwrap()).unwrap(),
                specimen
            );
            assert_eq!(
                resolve_holo_10d_specimen(&storage, &format!("qualia://specimens/{id}")).unwrap(),
                specimen
            );
            let corners = proxy_corners_for_specimen(&decoded);
            assert_eq!(corners[0], specimen.proxy_aabb_min);
            assert_eq!(corners[6], specimen.proxy_aabb_max);

            let mut corrupted = bytes.clone();
            corrupted[22] ^= 0x80;
            assert!(decode_holo_10d_container(id, &corrupted).is_err());
            assert!(decode_holo_10d_container(id, &bytes[..63]).is_err());
        }
        assert_eq!(
            resolve_holo_10d_specimen(&storage, "qualia://chora/universe")
                .unwrap()
                .specimen_id,
            DEFAULT_HOLO_SPECIMENS[0]
        );
        assert!(resolve_holo_10d_specimen(&storage, "qualia://specimens/../../escape").is_err());
        let foreign = storage.join("outside.10d");
        fs::write(&foreign, encode_holo_10d_container(&default_specimen(0))).unwrap();
        assert!(resolve_holo_10d_specimen(&storage, foreign.to_str().unwrap()).is_err());
        fs::remove_dir_all(storage).unwrap();
    }

    #[test]
    fn rejects_resealed_out_of_bounds_projection_inputs() {
        let mut specimen = default_specimen(0);
        specimen.max_raymarch_steps = 33;
        assert!(decode_holo_10d_container(
            &specimen.specimen_id,
            &encode_holo_10d_container(&specimen)
        )
        .is_err());
        specimen.max_raymarch_steps = 32;
        specimen.proxy_aabb_min[0] = f32::NAN;
        assert!(decode_holo_10d_container(
            &specimen.specimen_id,
            &encode_holo_10d_container(&specimen)
        )
        .is_err());
    }
}
