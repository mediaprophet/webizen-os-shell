// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Local HOLO specimen inspection for the compact Webizen and canonical QualiaDB profiles.
//!
//! Compact files carry projection declarations. MID2 files use QualiaDB's
//! typed payload codecs to verify authored HOLO, MSTR, AREF, IOVR and INST records.

use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use qualia_core_db::{
    container_10d as ten,
    q42::m64_matter::M64HoloRecord,
    render::skeletal::{valid_motor, Motor},
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
const MAX_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
const COMPACT_PROFILE: &str = "webizen-compact-10d-v2";
const CANONICAL_PROFILE: &str = "qualiadb-mid2-10d-v2";

pub const DEFAULT_HOLO_SPECIMENS: [&str; 4] = [
    "chora-spatiotemporal-commons",
    "anatomy-cardiovascular-holo",
    "ecopunk-riparian-biosphere",
    "poet-hyperbolic-mobius-v2",
];

/// Synth-authored workspace specimens bundled under `assets/synth/`.
/// Kept separate from `DEFAULT_HOLO_SPECIMENS` so compact rehearsal tests stay at four.
pub const UX_HOLO_SPECIMENS: [&str; 4] = [
    "sanctuary-guardian-crest",
    "wellfair-vitals-dyad",
    "commons-solar-microgrid",
    "holoslate-desk-totem",
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

/// Shell filter names for the seven level ordinals stored in canonical MSTR rows.
/// Each authored row retains its original level ordinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiScaleStratumLayer {
    L0Continuum,
    L1RigidArticulatedPgaXpbd,
    L2SoftBodyFluid,
    L3MolecularChemical,
    L4CellularBiological,
    L5EcologicalGeomorphic,
    L6QuantumPhysics,
}

impl MultiScaleStratumLayer {
    pub const ALL: [Self; 7] = [
        Self::L0Continuum,
        Self::L1RigidArticulatedPgaXpbd,
        Self::L2SoftBodyFluid,
        Self::L3MolecularChemical,
        Self::L4CellularBiological,
        Self::L5EcologicalGeomorphic,
        Self::L6QuantumPhysics,
    ];

    pub const fn code(self) -> &'static str {
        ["L0", "L1", "L2", "L3", "L4", "L5", "L6"][self as usize]
    }

    pub const fn title(self) -> &'static str {
        [
            "L0 · Continuum",
            "L1 · Rigid / Articulated PGA / XPBD / SKL2",
            "L2 · Soft-Body / Fluid",
            "L3 · Molecular / Chemical",
            "L4 · Cellular / Biological",
            "L5 · Ecological / Geomorphic",
            "L6 · Quantum Physics (Dynamics, Physics & Quantum Chemistry)",
        ][self as usize]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstituentLeafRef {
    pub aref_slot: u16,
    pub child_uri: String,
    pub stratum: MultiScaleStratumLayer,
    pub motor_translation: [f32; 3],
    pub iovr_override_summary: String,
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
    #[serde(default)]
    pub container_profile: String,
    #[serde(default)]
    pub active_strata: Vec<MultiScaleStratumLayer>,
    #[serde(default)]
    pub constituent_leaves: Vec<ConstituentLeafRef>,
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
    if bytes.starts_with(&ten::MAGIC_10D) {
        return decode_canonical_qualiadb_10d_container(specimen_id, bytes);
    }
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
    let rehearsal = DEFAULT_HOLO_SPECIMENS
        .iter()
        .position(|&candidate| candidate == specimen_id)
        .filter(|&index| encode_holo_10d_container(&default_specimen(index)).as_slice() == bytes)
        .map(compact_scene_rehearsal);
    let (active_strata, constituent_leaves) = rehearsal.unwrap_or_default();

    Ok(VerifiedHolo10dSpecimen {
        specimen_id: specimen_id.into(),
        title: specimen_title(specimen_id),
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
        container_profile: COMPACT_PROFILE.to_owned(),
        active_strata,
        constituent_leaves,
    })
}

pub fn decode_canonical_qualiadb_10d_container(
    specimen_id: &str,
    bytes: &[u8],
) -> Result<VerifiedHolo10dSpecimen, String> {
    let specimen_id = specimen_slug(specimen_id)?;
    if bytes.len() > MAX_CANONICAL_BYTES {
        return Err("The canonical .10d file exceeds the 16 MiB limit.".into());
    }
    let envelope = ten::Container10dHeader::parse(bytes).map_err(|error| error.to_string())?;
    if envelope.version != ten::HEADER_VERSION
        || envelope.header_crc32c != ten::compute_whole_file_crc32c(bytes)
    {
        return Err("The canonical .10d v2 version or whole-file checksum did not verify.".into());
    }
    let mut fields = [ten::TypedFieldRecord::default(); ten::MANIFOLD_IDENTITY_V2_MAX_FIELDS];
    let mut scratch = fields;
    let resolver = ten::TypedPayloadResolver::new(bytes, &[], &mut scratch, &mut fields)
        .map_err(|error| error.to_string())?;
    if !resolver.has_manifest() {
        return Err("The canonical .10d file requires a MID2 manifest.".into());
    }

    let payload = one_field(&resolver, ten::FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE)?
        .ok_or("The MID2 manifest has no HOLO (0x0306) field.")?;
    let records = record_slots(
        payload,
        ten::HOLOGRAPHIC_PHASE_HEADER_SIZE,
        ten::EULERIAN_WAVEFIELD_RECORD_SIZE,
        ten::MAX_HOLO_SEGMENTS + ten::MAX_EULERIAN_WAVEFIELDS,
        "HOLO",
    )?;
    let mut segments = vec![M64HoloRecord::default(); records.min(ten::MAX_HOLO_SEGMENTS)];
    let mut wavefields = vec![
        ten::EulerianWavefieldRecord {
            stream_velocity_m_s: [0.0; 3],
            vorticity_curl_rad_s: 0.0,
            bed_elevation_m: 0.0,
            water_depth_m: 0.0,
            fluid_density_kg_m3: 0.0,
            drag_coefficient_cd: 0.0,
            mode_amplitudes_f16: [0; 4],
            mode_wavenumbers_f16: [0; 4],
            mode_frequencies_f16: [0; 4],
            mode_directions_q8: [0; 4],
            record_crc32c: 0,
        };
        records.min(ten::MAX_EULERIAN_WAVEFIELDS)
    ];
    let (header, segment_count, wavefield_count) =
        ten::decode_holographic_phase_section_into(payload, &mut segments, &mut wavefields)
            .map_err(|error| format!("Could not verify canonical HOLO records: {error}"))?;
    if header.max_harmonic_order > MAX_HARMONIC_ORDER
        || !(1..=HOLO_MAX_RAYMARCH_STEPS).contains(&u32::from(header.max_raymarch_steps))
        || !header
            .proxy_aabb_min
            .into_iter()
            .zip(header.proxy_aabb_max)
            .all(|(min, max)| min.is_finite() && max.is_finite() && min <= max)
        || !header.sdf_smooth_union_radius.is_finite()
        || header.sdf_smooth_union_radius < 0.0
        || !(0.0..=1.0).contains(&header.reference_sigma)
    {
        return Err(
            "The canonical HOLO bounds, blend parameters or projection limits are invalid.".into(),
        );
    }

    let strata = read_strata(&resolver)?;
    let constituent_leaves = read_constituent_leaves(&resolver, &strata)?;
    let sha3_256_hex = format!("{:x}", Sha3_256::digest(bytes));
    let quin = NQuin48::from_parts(
        header.nquin_subject_hash,
        q_hash_60("urn:webizen:specimen:verified-mid2-holo"),
        q_hash_60(&sha3_256_hex),
        u64::from(ten::FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE),
        u64::from(envelope.version),
    );
    Ok(VerifiedHolo10dSpecimen {
        specimen_id: specimen_id.to_owned(),
        title: specimen_title(specimen_id),
        source_path: None,
        field_kind: ten::FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE,
        version: envelope.version,
        holo_segment_count: segment_count as u16,
        wavefield_count: wavefield_count as u16,
        max_harmonic_order: header.max_harmonic_order,
        max_raymarch_steps: header.max_raymarch_steps,
        proxy_aabb_min: header.proxy_aabb_min,
        proxy_aabb_max: header.proxy_aabb_max,
        sdf_smooth_union_radius: header.sdf_smooth_union_radius,
        reference_sigma: header.reference_sigma,
        default_shader_mode: HoloShaderVisualizationMode::PhaseSpaceWavefield,
        nquin_subject_hash: header.nquin_subject_hash,
        sealed_nquin_parity: quin.parity(),
        sha3_256_hex,
        container_profile: CANONICAL_PROFILE.to_owned(),
        active_strata: strata.active,
        constituent_leaves,
    })
}

/// Encode admitted HOLO and other authored payloads through the canonical MID2 writer.
/// On success only the returned prefix of the caller's output is part of the file.
pub fn encode_canonical_qualiadb_10d_container(
    identity: &ten::ManifoldIdentityV2,
    fields: &[ten::TypedFieldRecord],
    sections: &[ten::SectionInput<'_>],
    out: &mut [u8],
) -> Result<usize, String> {
    if sections.len() >= ten::MAX_SECTION_COUNT as usize {
        return Err("The .10d scene has too many section declarations.".into());
    }
    let manifest_len =
        ten::manifold_identity_v2_encoded_len(fields.len()).map_err(|error| error.to_string())?;
    let mut manifest = vec![0; manifest_len];
    let mut section_scratch = sections.to_vec();
    section_scratch.push(ten::SectionInput {
        section_type: ten::SectionType::ManifoldIdentityV2,
        alignment_tier: ten::AlignmentTier::Page,
        stride: 0,
        element_count: 0,
        payload: &[],
    });
    let capacity = out.len().min(MAX_CANONICAL_BYTES);
    let size = ten::encode_container_with_manifold_identity(
        &ten::Container10dHeader::proposed(),
        sections,
        identity,
        fields,
        &mut manifest,
        &mut section_scratch,
        &mut out[..capacity],
    )
    .map_err(|error| format!("Could not encode the canonical MID2 scene: {error:?}"))?;
    ten::seal_whole_file_crc32c(&mut out[..size]);
    decode_canonical_qualiadb_10d_container("scene", &out[..size])?;
    Ok(size)
}

fn one_field<'a>(
    resolver: &ten::TypedPayloadResolver<'a, '_>,
    kind: u16,
) -> Result<Option<&'a [u8]>, String> {
    let mut fields = resolver.payloads_of_kind(kind);
    let first = fields
        .next()
        .transpose()
        .map_err(|error| error.to_string())?;
    if fields.next().is_some() {
        return Err(format!("Ambiguous canonical .10d field 0x{kind:04x}."));
    }
    Ok(first.map(|field| field.bytes))
}

fn record_slots(
    payload: &[u8],
    header_size: usize,
    record_size: usize,
    max: usize,
    label: &str,
) -> Result<usize, String> {
    payload
        .len()
        .checked_sub(header_size)
        .filter(|length| length % record_size == 0)
        .map(|length| length / record_size)
        .filter(|&count| count <= max)
        .ok_or_else(|| format!("The canonical {label} payload has an invalid extent."))
}

#[derive(Default)]
struct SceneStrata {
    active: Vec<MultiScaleStratumLayer>,
    by_entity: BTreeMap<u64, Option<MultiScaleStratumLayer>>,
    by_taxon: BTreeMap<u64, Option<MultiScaleStratumLayer>>,
}

fn empty_stratum_record() -> ten::NestedStratumNodeRecord {
    ten::NestedStratumNodeRecord {
        host_anchor: Default::default(),
        parent_node_index: ten::STRATUM_NO_PARENT,
        stratum_level: 0,
        constituent_kind: ten::StratumConstituentKind::HolobiontOrSystemRoot as u8,
        interaction_regime: ten::StratumInteractionRegime::StructuralConstituent as u8,
        embodiment_state: ten::EmbodimentResolutionState::ComputationalOnly as u8,
        quantum_basis_and_xc: 0,
        characteristic_scale_log10_m_q12: 0,
        taxon_or_ontology_iri_hash: 0,
        target_archetype_or_qpu_nquin_hash: 0,
        density_or_abundance_per_m3: 0.0,
        metabolic_flux_or_hartree_energy: 0.0,
        coupled_metric_a_f16: 0,
        coupled_metric_b_f16: 0,
        logic_invariants: Default::default(),
        belnap_and_spin_u16: 0,
        orbital_or_coupling_flags: 0,
        record_crc32c: 0,
    }
}

fn read_strata(resolver: &ten::TypedPayloadResolver<'_, '_>) -> Result<SceneStrata, String> {
    let mut strata = SceneStrata::default();
    let Some(payload) = one_field(resolver, ten::FIELD_KIND_MULTISCALE_STRATA)? else {
        return Ok(strata);
    };
    let count = record_slots(
        payload,
        ten::MULTISCALE_STRATA_HEADER_SIZE,
        ten::NESTED_STRATUM_NODE_SIZE,
        ten::MAX_NESTED_STRATUM_NODES,
        "MSTR",
    )?;
    let mut records = vec![empty_stratum_record(); count];
    let (_, count) = ten::decode_multiscale_strata_section_into(payload, &mut records)
        .map_err(|error| format!("Could not verify canonical MSTR records: {error:?}"))?;
    let mut active = [false; 7];
    for node in &records[..count] {
        let index = usize::from(node.stratum_level);
        let level = MultiScaleStratumLayer::ALL[index];
        active[index] = true;
        for (bindings, hash) in [
            (&mut strata.by_entity, node.host_anchor.entity_hash),
            (&mut strata.by_taxon, node.taxon_or_ontology_iri_hash),
        ] {
            if hash != 0 {
                bindings
                    .entry(hash)
                    .and_modify(|known| {
                        if *known != Some(level) {
                            *known = None;
                        }
                    })
                    .or_insert(Some(level));
            }
        }
    }
    strata.active = MultiScaleStratumLayer::ALL
        .into_iter()
        .zip(active)
        .filter_map(|(level, present)| present.then_some(level))
        .collect();
    Ok(strata)
}

fn read_constituent_leaves(
    resolver: &ten::TypedPayloadResolver<'_, '_>,
    strata: &SceneStrata,
) -> Result<Vec<ConstituentLeafRef>, String> {
    let references = resolver
        .payloads_of_kind(ten::FIELD_KIND_ARCHETYPE_REF)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let has_overrides = resolver
        .payloads_of_kind(ten::FIELD_KIND_INSTANCE_OVERRIDES)
        .next()
        .is_some();
    if has_overrides {
        resolver
            .validate_instance_overrides()
            .map_err(|error| error.to_string())?;
    }
    let mut instances = Vec::new();
    let mut coordinates = None;
    if let Some(payload) = one_field(resolver, ten::FIELD_KIND_INSTANCE_BATCH)? {
        let count = record_slots(
            payload,
            ten::INSTANCE_BATCH_HEADER_SIZE,
            ten::INSTANCE_BATCH_RECORD_SIZE,
            ten::MAX_INSTANCE_BATCH_RECORDS,
            "INST",
        )?;
        instances.resize(count, ten::InstanceBatchRecord::default());
        ten::decode_instance_batch_into(payload, &mut instances)
            .map_err(|error| format!("Could not verify canonical INST records: {error:?}"))?;
        if let Some(nodes) = resolver.section_payload(ten::SectionType::Tensor10DNodes) {
            if !nodes.as_ptr().cast::<ten::NodeMiniHeader>().is_aligned() {
                return Err("The authored Tensor10D node section is not aligned.".into());
            }
            let (header, length) = ten::parse_node_header(nodes)
                .map_err(|error| format!("Invalid INST node coordinates: {error:?}"))?;
            if header.node_count as usize != instances.len() || length != nodes.len() {
                return Err("The INST batch does not match the authored Tensor10D nodes.".into());
            }
            coordinates = Some(nodes);
        }
    }

    let mut leaves = Vec::with_capacity(references.len());
    let mut coefficients = [ten::SsmInstanceCoeff::default(); ten::MAX_INSTANCE_SSM_COEFFS];
    for (slot, field) in references.iter().enumerate() {
        let reference = ten::decode_archetype_ref_record(field.bytes)
            .map_err(|error| format!("Could not verify canonical AREF slot {slot}: {error:?}"))?;
        let algorithm = match ten::ArchetypeDigestAlg::from_u8(reference.digest_alg) {
            Some(ten::ArchetypeDigestAlg::Sha256) => "sha256",
            Some(ten::ArchetypeDigestAlg::Blake3_256) => "blake3",
            None => return Err("The AREF digest algorithm is unavailable.".into()),
        };
        // A standalone envelope stores content digests and qualified identifiers.
        // Bundle paths are resolved by the separate HMC admission boundary.
        let digest: String = reference
            .archetype_digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let child_uri = format!("urn:webizen:10d:{algorithm}:{digest}");

        let instance = (instances.len() == references.len()).then(|| &instances[slot]);
        let stratum = instance
            .and_then(|record| strata.by_entity.get(&record.entity_id_hash))
            .or_else(|| strata.by_taxon.get(&reference.taxon_or_class_iri_hash))
            .copied()
            .flatten();
        let bound = if has_overrides {
            resolver
                .instance_override_into(slot, &mut coefficients)
                .map_err(|error| error.to_string())?
        } else {
            None
        };
        let mut details = match bound {
            Some(override_data) => format!(
                "IOVR: age {} d, scale {}, {} shape coefficients",
                override_data.header.age_days,
                override_data.header.uniform_scale,
                override_data.coefficient_count
            ),
            None => String::from("No IOVR override"),
        };
        if instance.is_some() {
            details.push_str(" · ordered INST binding");
        } else if !instances.is_empty() {
            details.push_str(&format!(
                " · INST batch: {} instances; no per-slot binding",
                instances.len()
            ));
        }
        if stratum.is_none() {
            details.push_str(" · no matching MSTR level; filed under L0");
        }
        let motor_translation = if let Some(override_data) = bound {
            motor_translation(override_data.header.local_pga_motor)?
        } else if let Some(nodes) = coordinates.filter(|_| instance.is_some()) {
            let node = ten::read_node(nodes, slot)
                .map_err(|error| format!("Could not read INST node coordinates: {error:?}"))?;
            let translation = [node.x, node.y, node.z];
            if !translation.into_iter().all(f32::is_finite) {
                return Err("The INST translation contains a non-finite value.".into());
            }
            translation
        } else {
            details.push_str(" · no authored translation; origin placeholder");
            [0.0; 3]
        };
        leaves.push(ConstituentLeafRef {
            aref_slot: slot as u16,
            child_uri,
            stratum: stratum.unwrap_or(MultiScaleStratumLayer::L0Continuum),
            motor_translation,
            iovr_override_summary: details,
        });
    }
    Ok(leaves)
}

fn motor_translation(components: [f32; 8]) -> Result<[f32; 3], String> {
    let [s, e23, e31, e12, e01, e02, e03, e0123] = components.map(f64::from);
    let motor = Motor {
        r_w: s,
        r_x: e23,
        r_y: e31,
        r_z: e12,
        d_w: e0123,
        d_x: -e01,
        d_y: -e02,
        d_z: -e03,
    };
    if !valid_motor(&motor) {
        return Err("IOVR placement requires a unit PGA motor.".into());
    }
    let translation = motor.transform_point([0.0; 3]).map(|value| value as f32);
    if translation.into_iter().all(f32::is_finite) {
        Ok(translation)
    } else {
        Err("The IOVR translation exceeds finite workspace coordinates.".into())
    }
}

fn specimen_title(specimen_id: &str) -> String {
    if let Some(index) = DEFAULT_HOLO_SPECIMENS
        .iter()
        .position(|&candidate| candidate == specimen_id)
    {
        return DEFAULT_TITLES[index].to_owned();
    }
    if let Some(index) = UX_HOLO_SPECIMENS
        .iter()
        .position(|&candidate| candidate == specimen_id)
    {
        return UX_TITLES[index].to_owned();
    }
    specimen_id.to_owned()
}

fn compact_scene_rehearsal(index: usize) -> (Vec<MultiScaleStratumLayer>, Vec<ConstituentLeafRef>) {
    use MultiScaleStratumLayer::*;
    let examples: &[(&str, MultiScaleStratumLayer, [f32; 3])] = match index {
        0 => &[
            ("landform", L0Continuum, [0.0, 0.0, 0.0]),
            ("walkway", L1RigidArticulatedPgaXpbd, [0.25, 0.12, 0.0]),
            ("catchment", L5EcologicalGeomorphic, [-0.3, -0.1, 0.15]),
        ],
        1 => &[
            ("vessel-wall", L0Continuum, [0.0, 0.0, 0.0]),
            ("cardiomyocyte", L4CellularBiological, [0.1, 0.25, 0.0]),
        ],
        2 => &[
            ("riverbank", L5EcologicalGeomorphic, [-0.35, 0.1, 0.0]),
            ("flow", L2SoftBodyFluid, [0.0, 0.0, 0.1]),
        ],
        _ => &[
            ("mobius-surface", L0Continuum, [0.0, 0.0, 0.0]),
            ("phase-feature", L6QuantumPhysics, [0.1, 0.0, 0.2]),
        ],
    };
    let active = MultiScaleStratumLayer::ALL
        .into_iter()
        .filter(|level| examples.iter().any(|(_, stratum, _)| stratum == level))
        .collect();
    let leaves = examples
        .iter()
        .enumerate()
        .map(|(slot, &(name, stratum, translation))| ConstituentLeafRef {
            aref_slot: slot as u16,
            child_uri: format!(
                "urn:webizen:holo-rehearsal:{}:{name}",
                DEFAULT_HOLO_SPECIMENS[index]
            ),
            stratum,
            motor_translation: translation,
            iovr_override_summary: "Local Rehearsal · illustrative AREF, IOVR and INST placement"
                .into(),
        })
        .collect();
    (active, leaves)
}

fn local_specimen_name(query: &str) -> Result<&str, String> {
    let query = query.strip_prefix("holo:").unwrap_or(query);
    if let Some(uri) = query
        .strip_prefix("qualia://")
        .or_else(|| query.strip_prefix("webizen://"))
    {
        let route = uri.split(['?', '#']).next().unwrap_or_default();
        return match route {
            "chora/universe" => Ok(DEFAULT_HOLO_SPECIMENS[0]),
            _ => specimen_slug(
                route
                    .strip_prefix("specimens/")
                    .ok_or("The native route has no registered shader-studio specimen.")?,
            ),
        };
    }
    Ok(query)
}

/// Verify the bytes of a known packaged rehearsal without consulting storage.
pub(crate) fn default_holo_specimen(query: &str) -> Option<VerifiedHolo10dSpecimen> {
    let id = specimen_slug(local_specimen_name(query.trim()).ok()?).ok()?;
    let index = DEFAULT_HOLO_SPECIMENS
        .iter()
        .position(|&known| known == id)?;
    decode_holo_10d_container(id, &encode_holo_10d_container(&default_specimen(index))).ok()
}

const DEFAULT_TITLES: [&str; 4] = [
    "Chora Spatio-Temporal Commons",
    "Anatomy Cardiovascular",
    "Ecopunk Riparian Biosphere",
    "Poet Hyperbolic Möbius v2",
];

const UX_TITLES: [&str; 4] = [
    "Sanctuary Keep Guardian Shield & Crystal Vault",
    "WellFair Physiological Heart & Care Dyad",
    "Community Grounds Solar Microgrid & Terrain",
    "Holoslate Multi-Ring Spatial Command Totem",
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
        container_profile: COMPACT_PROFILE.to_owned(),
        active_strata: Vec::new(),
        constituent_leaves: Vec::new(),
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
    let local_name = local_specimen_name(input)?;
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
    let ux_index = (!is_path)
        .then(|| UX_HOLO_SPECIMENS.iter().position(|&name| name == slug))
        .flatten();
    let directory = specimen_directory(storage, seed_index.is_some() || ux_index.is_some())?;
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
        None if ux_index.is_some() => seed_ux_specimen(&directory, slug),
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
        || !(CONTAINER_BYTES as u64..=MAX_CANONICAL_BYTES as u64).contains(&metadata.len())
    {
        return Err("The specimen needs a regular .10d file within the 16 MiB limit.".into());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(file)
        .map_err(|error| error.to_string())?
        .take((MAX_CANONICAL_BYTES + 1) as u64)
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

fn bundled_ux_specimen_path(slug: &str) -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/synth")
        .join(format!("{slug}.10d"));
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() && !is_reparse_point(&metadata) => Some(path),
        _ => None,
    }
}

fn ux_compact_specimen(index: usize) -> VerifiedHolo10dSpecimen {
    use HoloShaderVisualizationMode::*;
    let (extent, mode, wavefield_count, sigma) = [
        ([0.72, 0.92, 0.72], PhaseSpaceWavefield, 3, 0.52),
        ([0.58, 0.78, 0.58], AnatomicalRadialDepth, 2, 0.64),
        ([0.96, 0.42, 0.96], ScaleSpaceHarmonicSlice, 4, 0.70),
        ([0.68, 1.05, 0.68], HyperbolicMobiusV2, 2, 0.77),
    ][index];
    let id = UX_HOLO_SPECIMENS[index];
    VerifiedHolo10dSpecimen {
        specimen_id: id.into(),
        title: UX_TITLES[index].into(),
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
        container_profile: COMPACT_PROFILE.to_owned(),
        active_strata: Vec::new(),
        constituent_leaves: Vec::new(),
    }
}

fn seed_ux_specimen(directory: &Path, slug: &str) -> Result<VerifiedHolo10dSpecimen, String> {
    let path = directory.join(format!("{slug}.10d"));
    match fs::symlink_metadata(&path) {
        Ok(_) => return read_specimen(slug, &path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    if let Some(bundled) = bundled_ux_specimen_path(slug) {
        if publish_specimen_bytes(&path, slug, &fs::read(&bundled).map_err(|error| error.to_string())?)
            .is_ok()
        {
            if let Ok(spec) = read_specimen(slug, &path) {
                return Ok(spec);
            }
            let _ = fs::remove_file(&path);
        }
    }
    let index = UX_HOLO_SPECIMENS
        .iter()
        .position(|&name| name == slug)
        .ok_or_else(|| "The workspace specimen is not registered.".to_string())?;
    publish_specimen_bytes(
        &path,
        slug,
        &encode_holo_10d_container(&ux_compact_specimen(index)),
    )?;
    read_specimen(slug, &path)
}

fn publish_specimen_bytes(path: &Path, slug: &str, bytes: &[u8]) -> Result<(), String> {
    static NEXT_UX_TEMP: AtomicU64 = AtomicU64::new(1);
    let directory = path.parent().ok_or("The specimen path is missing a directory.")?;
    let temp = directory.join(format!(
        ".{slug}-{}-{}.tmp",
        std::process::id(),
        NEXT_UX_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|error| error.to_string())?;
    let stored = file.write_all(bytes).and_then(|()| file.sync_all());
    drop(file);
    let published = stored.and_then(|()| match fs::hard_link(&temp, path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    });
    let cleaned = fs::remove_file(&temp);
    published.and(cleaned).map_err(|error| error.to_string())
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
    fn resolves_bundled_or_compact_ux_workspace_specimens() {
        let storage = std::env::temp_dir().join(format!(
            "webizen-ux-holo-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&storage).unwrap();
        for (index, id) in UX_HOLO_SPECIMENS.into_iter().enumerate() {
            let specimen = resolve_holo_10d_specimen(&storage, id).unwrap();
            assert_eq!(specimen.specimen_id, id);
            assert_eq!(specimen.title, UX_TITLES[index]);
            assert_eq!(specimen.field_kind, FIELD_KIND_HOLO_PHASE_SPACE);
            let again =
                resolve_holo_10d_specimen(&storage, &format!("qualia://specimens/{id}")).unwrap();
            assert_eq!(again.specimen_id, id);
            assert_eq!(again.title, UX_TITLES[index]);
        }
        fs::remove_dir_all(storage).unwrap();
    }

    #[test]
    fn decodes_canonical_qualiadb_mid2_10d_containers_and_preserves_l0_l6_strata() {
        let header = ten::HolographicPhaseHeader {
            magic: ten::HOLOGRAPHIC_PHASE_TAG,
            version: ten::HOLOGRAPHIC_PHASE_VERSION,
            flags: 0,
            holo_segment_count: 1,
            wavefield_count: 0,
            max_harmonic_order: 4,
            max_raymarch_steps: 32,
            primary_constituent_lut_idx: 0,
            proxy_lens_kind: 0,
            proxy_aabb_min: [-1.0, -2.0, -3.0],
            proxy_aabb_max: [1.0, 2.0, 3.0],
            sdf_smooth_union_radius: 0.1,
            reference_sigma: 0.5,
            nquin_subject_hash: 42,
            payload_crc32c: 0,
            header_crc32c: 0,
        };
        let mut holo = [0; 128];
        ten::encode_holographic_phase_section(header, &[M64HoloRecord::default()], &[], &mut holo)
            .unwrap();

        let mut strata = [empty_stratum_record(); 7];
        for (level, node) in strata.iter_mut().enumerate() {
            node.stratum_level = level as u8;
            node.host_anchor.entity_hash = 800 + level as u64;
            node.taxon_or_ontology_iri_hash = 900 + level as u64;
        }
        let mut mstr = vec![0; ten::multiscale_strata_encoded_len(strata.len()).unwrap()];
        ten::encode_multiscale_strata_section(&strata, 77, &mut mstr).unwrap();

        let reference = ten::ArchetypeRefRecord::new(
            ten::ArchetypeDigestAlg::Sha256,
            ten::ArchetypeTargetKind::Archetype10D,
            ten::OVERRIDE_FLAG_COORDS_10D,
            [0xab; 32],
            [0x21; 16],
            904,
            432,
        );
        let mut aref = [0; ten::ARCHETYPE_REF_RECORD_SIZE];
        ten::encode_archetype_ref_record(&reference, &mut aref).unwrap();
        let mut iovr = [0; ten::INSTANCE_OVERRIDE_HEADER_SIZE];
        ten::encode_instance_overrides(
            12.0,
            1.25,
            [1.0, 0.0, 0.0, 0.0, -0.5, -1.0, -1.5, 0.0],
            &[],
            &mut iovr,
        )
        .unwrap();

        let mut inst = [0; ten::INSTANCE_BATCH_HEADER_SIZE + ten::INSTANCE_BATCH_RECORD_SIZE];
        ten::encode_instance_batch(
            &[ten::InstanceBatchRecord {
                entity_id_hash: 804,
                ..Default::default()
            }],
            &mut inst,
        )
        .unwrap();

        let mut payload = Vec::new();
        let mut fields = Vec::new();
        for (kind, bytes) in [
            (ten::FIELD_KIND_HOLOGRAPHIC_PHASE_SPACE, holo.as_slice()),
            (ten::FIELD_KIND_MULTISCALE_STRATA, mstr.as_slice()),
            (ten::FIELD_KIND_ARCHETYPE_REF, aref.as_slice()),
            (ten::FIELD_KIND_INSTANCE_OVERRIDES, iovr.as_slice()),
            (ten::FIELD_KIND_INSTANCE_BATCH, inst.as_slice()),
        ] {
            fields.push(ten::TypedFieldRecord {
                field_id: fields.len() as u64 + 1,
                kind,
                payload_section_id: ten::SectionType::FieldSidecar as u32,
                payload_offset: payload.len() as u64,
                payload_length: bytes.len() as u64,
                ..Default::default()
            });
            payload.extend_from_slice(bytes);
        }
        let sections = [ten::SectionInput {
            section_type: ten::SectionType::FieldSidecar,
            alignment_tier: ten::AlignmentTier::Page,
            stride: 0,
            element_count: 0,
            payload: &payload,
        }];
        let identity = ten::ManifoldIdentityV2 {
            entity_id: ten::StableEntityId([0x42; 16]),
            profile_id: 1,
            domain_id: 1,
            coordinate_convention_id: 1,
            coordinates: [0.0; 10],
            external_digest: None,
        };
        let mut bytes = vec![0; 32 * 1024];
        let written =
            encode_canonical_qualiadb_10d_container(&identity, &fields, &sections, &mut bytes)
                .unwrap();
        bytes.truncate(written);
        let scene = decode_holo_10d_container("mid2-sample", &bytes).unwrap();
        assert_eq!(scene.container_profile, CANONICAL_PROFILE);
        assert_eq!(scene.active_strata, MultiScaleStratumLayer::ALL);
        assert_eq!(scene.proxy_aabb_max, [1.0, 2.0, 3.0]);
        assert_eq!(scene.holo_segment_count, 1);
        assert_eq!(scene.max_raymarch_steps, 32);
        assert_eq!(scene.constituent_leaves.len(), 1);
        let leaf = &scene.constituent_leaves[0];
        assert_eq!(leaf.aref_slot, 0);
        assert_eq!(
            leaf.child_uri,
            format!("urn:webizen:10d:sha256:{}", "ab".repeat(32))
        );
        assert_eq!(leaf.stratum, MultiScaleStratumLayer::L4CellularBiological);
        assert_eq!(leaf.motor_translation, [1.0, 2.0, 3.0]);
        assert!(leaf.iovr_override_summary.contains("ordered INST binding"));

        let storage = std::env::temp_dir().join(format!(
            "webizen-mid2-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let specimen_directory = specimen_directory(&storage, true).unwrap();
        fs::write(specimen_directory.join("mid2-sample.10d"), &bytes).unwrap();
        let on_disk =
            resolve_holo_10d_specimen(&storage, "qualia://specimens/mid2-sample").unwrap();
        assert_eq!(on_disk.sha3_256_hex, scene.sha3_256_hex);
        assert_eq!(on_disk.constituent_leaves, scene.constituent_leaves);
        fs::remove_dir_all(storage).unwrap();

        let end = bytes.len() - 1;
        bytes[end] ^= 1;
        assert!(decode_holo_10d_container("mid2-sample", &bytes).is_err());
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
