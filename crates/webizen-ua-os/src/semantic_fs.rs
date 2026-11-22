//! Semantic File-System & Persistent Hypermedia Library (`semantic_fs`).
//!
//! Brings QualiaDB's canonical `"context is the asset"` semantic file-system
//! (`qualia-core-db::hypermedia` + `qualia-client-core::wellfair::hypermedia_store`)
//! directly into the User-Agent OS runtime (`webizen-ua-os`):
//!
//! 1. **Graph of Edges, Never a Folder Tree**:
//!    - Every asset is packaged as a semantic container (`urn:qualia:hypermedia:Container`)
//!      binding its primary asset (`hasPrimary`), bundled related assets (`bundledWith` +
//!      `AssetRole::{Primary, Source, Derivation, Lod, Analysis, Provenance, Related}`),
//!      W3C PROV lineage (`prov:wasDerivedFrom`), provenance (`hasProvenance`), and
//!      computed analytics (`analysisTarget` / `analysisResult` / `analysisMethod`) as
//!      48-byte parity-checked `NQuin48` records in the 60-bit FNV-1a subject space (`q_hash_60`).
//! 2. **Entity, Feature & Depiction Extraction (What's *Inside* the Asset)**:
//!    - Ingest processors extract internal entities, features, and depictions
//!      (`urn:qualia:hypermedia:depicts` — e.g. anatomical structures inside a `.10d` v2
//!      `HOLO` container, objects/species depicted in an image, C2PA `QECP` credentials
//!      inside a PDF, or `TEXT -> CONCEPT -> LOGIC` CML concepts & deontic norms inside
//!      prose/legislation), alongside spatiotemporal facets (`occurredAt` timeline +
//!      `atPlace` / `lat,lon` map coordinates).
//! 3. **Ontologically Defined Purpose-Shaped Sections & Orthogonal Sensitivity**:
//!    - Routes assets into 7 purpose-shaped sections (`Secret`, `Wellfair`, `Personal`,
//!      `Work`, `Tools`, `Software`, `Commons`).
//!    - High sensitivity (`restricted` / `classified` / `sanctuary`) always forces `Secret`
//!      and fail-closed blocks `CommonsVisibility::{Peers, Commons}` sharing.
//! 4. **Multi-Dimensional Faceted Search (`AND` across dimensions, `OR` within dimension)**:
//!    - Filters and tallies across `section`, free-text, `depicts` (entities/features inside
//!      assets), `topics`, `purposes`, `projects`, `media_types`, and `place`, with
//!      deterministic sorting (`Newest`, `Oldest`, `TitleAsc`, `TitleDesc`, `MediaType`).

use crate::identifiers::q_hash_60;
use crate::poet_synth::LocalLlmModelEntry;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use vibescript_abi::NQuin48;

pub const HYPERMEDIA_LIBRARY_REL_PATH: &str = "wellfair/hypermedia_library.json";

/// Opt-in Semantic File-System Toolbox IDs — structured after Poet's `ToolChest` / `ShedToolbox`
/// pattern (`qualia-core-db::poet_host::invoke::shed_registry`), where users elect which specialized
/// facet extractors, sub-entity traversers, and analysis toolchains are active in their workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticToolboxId {
    /// 10D v2 HOLO Assembly (`AREF`/`IOVR`/`INST`), 7-Layer `L0`–`L6` `MSTR` Strata & Harmonic LODs.
    AssemblyAndStrata10D,
    /// Audio `.qac`, `SonicToken`, MIDI 2.0 UMP, 10D Holosonic HOA & Bioacoustics Extractor.
    HolosonicAndBioacoustics,
    /// Temporal Timeline Window (`occurredAt` / Allen Intervals) & GeoSPARQL Hilbert `.m64` Map.
    SpatiotemporalAndGeodesy,
    /// Anatomical Cavity SDF, Intrinsic Radial Tissue Depth, 4-Regime Pathology & DICOM LPS Registration.
    BiomedicalAndDicomAtlas,
    /// CML (`TEXT -> CONCEPT -> LOGIC`), Deontic/Hohfeld Norms, COF HTML+RDFa Segments & C2PA `QECP` Verifier.
    CmlDeonticAndQecp,
    /// 9-Stage Contribution Lifecycle, Source-Asset Licensing, Guardianship Watch-Flags & W3C Solid Off-Ramp.
    PermissiveCommonsAndLifecycle,
}

/// Static specification of an opt-in Semantic File-System Toolbox (aligned with Poet's `SHED_TOOLBOXES`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticToolboxSpec {
    pub id: SemanticToolboxId,
    pub slug: &'static str,
    pub title: &'static str,
    pub poet_shed_family: &'static str,
    pub default_enabled: bool,
    pub unlocked_facets: &'static [&'static str],
    pub toolchain_ops: &'static [&'static str],
    pub description: &'static str,
}

impl SemanticToolboxId {
    pub const ALL: [Self; 6] = [
        Self::AssemblyAndStrata10D,
        Self::HolosonicAndBioacoustics,
        Self::SpatiotemporalAndGeodesy,
        Self::BiomedicalAndDicomAtlas,
        Self::CmlDeonticAndQecp,
        Self::PermissiveCommonsAndLifecycle,
    ];

    pub const fn spec(self) -> SemanticToolboxSpec {
        match self {
            Self::AssemblyAndStrata10D => SemanticToolboxSpec {
                id: Self::AssemblyAndStrata10D,
                slug: "assembly-10d-strata",
                title: "10D v2 HOLO Assembly (AREF/IOVR/INST) & L0-L6 Strata",
                poet_shed_family: "ComputationalGeometry + GeometricAlgebra",
                default_enabled: true,
                unlocked_facets: &[
                    "aref:leaf-archetype",
                    "iovr:pga-motor-override",
                    "mstr:L0-L6-strata",
                    "holo:harmonic-lod-k0..K",
                ],
                toolchain_ops: &[
                    "hypermedia.locus.compile_10d_holo",
                    "ComputationalGeometry.svo_raycast",
                    "GeometricAlgebra.apply_rotor",
                ],
                description: "Traverses two-tier .10d v2 HOLO assemblies (AREF/IOVR/INST), 7-layer L0-L6 MSTR hierarchy, and scale-space harmonic LODs (k=0..K).",
            },
            Self::HolosonicAndBioacoustics => SemanticToolboxSpec {
                id: Self::HolosonicAndBioacoustics,
                slug: "holosonic-bioacoustics",
                title: "Holosonic .qac, SonicToken, MIR & Bioacoustics",
                poet_shed_family: "StudioAndHypermedia (AudioStudio)",
                default_enabled: false,
                unlocked_facets: &[
                    "qac:sonic-token-ump",
                    "qac:hoa-acn-sn3d",
                    "mir:yin-crepe-pitch-hpcp",
                    "bioacoustics:species-call-stft",
                ],
                toolchain_ops: &[
                    "hypermedia.acoustics.solve_holosonic_field",
                    "Timeline.master_lufs",
                    "Audio.sonic_token_bridge",
                ],
                description: "Extracts SonicToken articulatory/MIDI 2.0 UMP frames, 10D Holosonic Ambisonics, YIN/CREPE pitch & HPCP key, and bioacoustic call spectra.",
            },
            Self::SpatiotemporalAndGeodesy => SemanticToolboxSpec {
                id: Self::SpatiotemporalAndGeodesy,
                slug: "spatiotemporal-geodesy",
                title: "Temporal Timeline Slider, Allen Intervals & GeoSPARQL Map",
                poet_shed_family: "Manifolds::Temporal + Manifolds::Spatial",
                default_enabled: true,
                unlocked_facets: &[
                    "time:occurredAt-window",
                    "time:allen-interval-relation",
                    "geo:cb-usri-16b-coord",
                    "geo:m64-grm3-hilbert-strata",
                ],
                toolchain_ops: &[
                    "Poet.timeline_scrub",
                    "hypermedia.geodesy.hilbert_strata",
                    "GraphDatabase.geosparql_within",
                ],
                description: "Enables temporal window filtering (occurredAt / Allen Intervals) and GeoSPARQL / 16-byte CB-USRI / .m64 GRM3 Hilbert map discovery.",
            },
            Self::BiomedicalAndDicomAtlas => SemanticToolboxSpec {
                id: Self::BiomedicalAndDicomAtlas,
                slug: "biomedical-dicom-atlas",
                title: "Anatomical Cavity SDF, Tissue Depth & DICOM LPS Registration",
                poet_shed_family: "ScienceChemistryClinical (health/anatomy)",
                default_enabled: false,
                unlocked_facets: &[
                    "anatomy:cavity-sdf-containment",
                    "anatomy:radial-tissue-depth-d0..1",
                    "pathology:4-regime-doid-hp",
                    "dicom:lps-cross-frame-cert",
                ],
                toolchain_ops: &[
                    "synth_validate_model",
                    "Medical.register_patient_to_anatomy",
                    "ClinicalRisk.comorbidity_eval",
                ],
                description: "Inspects FMA 5.0.0/Uberon cavity SDF containment, radial tissue depth d in [0,1], 4-regime pathology, and DICOM Part-10 LPS registration.",
            },
            Self::CmlDeonticAndQecp => SemanticToolboxSpec {
                id: Self::CmlDeonticAndQecp,
                slug: "cml-deontic-qecp",
                title: "CML (TEXT->CONCEPT->LOGIC), Hohfeld Jural Norms & QECP Verifier",
                poet_shed_family: "LogicAndSemantics (logic-sentinel)",
                default_enabled: true,
                unlocked_facets: &[
                    "cml:text-concept-logic",
                    "deontic:hohfeld-jural-norm",
                    "cof:html-rdfa-token-segments",
                    "qecp:c2pa-jumbf-mldsa65-vc",
                ],
                toolchain_ops: &[
                    "NLP.gazetteer_seal_nquin48",
                    "Deontic.evaluate",
                    "SHACL.validate_qecp_jumbf",
                ],
                description: "Extracts CML concepts, deontic/Hohfeld legal norms, token-bounded COF HTML+RDFa segments, and C2PA JUMBF QECP embedded PDF credentials.",
            },
            Self::PermissiveCommonsAndLifecycle => SemanticToolboxSpec {
                id: Self::PermissiveCommonsAndLifecycle,
                slug: "permissive-commons-lifecycle",
                title: "9-Stage Contribution Lifecycle, Guardianship Flags & Solid Off-Ramp",
                poet_shed_family: "EconomicsAndGovernance (rights/commons)",
                default_enabled: false,
                unlocked_facets: &[
                    "cop:nine-stage-obligation",
                    "license:source-asset-licensing-n3",
                    "guardian:watch-flag-severity",
                    "solid:turtle-nquin48-offramp",
                ],
                toolchain_ops: &[
                    "Poet.toolchain_execute(coop_contribution_log)",
                    "SemanticInstrument.licence_transition_readiness",
                    "SolidBridge.export_turtle_pod",
                ],
                description: "Tracks 9-stage contribution obligations, source-asset-licensing.n3 compliance, bilateral guardianship watch-flags, and W3C Solid Pod export.",
            },
        }
    }
}

/// Recommend opt-in Semantic Toolboxes for a given asset entry (mirroring Poet's `shed_registry::recommend_for_context`).
pub fn recommend_toolboxes_for_entry(entry: &SemanticAssetEntry) -> Vec<SemanticToolboxId> {
    let mut recs = Vec::new();
    let hay = format!(
        "{} {} {} {}",
        entry.asset_uri,
        entry.media_type,
        entry.topics.join(" "),
        entry.depicts.join(" ")
    )
    .to_ascii_lowercase();

    if hay.contains("10d") || hay.contains("holo") || hay.contains("hmc") || hay.contains("aref") {
        recs.push(SemanticToolboxId::AssemblyAndStrata10D);
    }
    if hay.contains("qac") || hay.contains("audio") || hay.contains("sonic") || hay.contains("canopy") {
        recs.push(SemanticToolboxId::HolosonicAndBioacoustics);
    }
    if entry.place.is_some() || entry.occurred_at.is_some() || hay.contains("geodesy") {
        recs.push(SemanticToolboxId::SpatiotemporalAndGeodesy);
    }
    if hay.contains("anatomy")
        || hay.contains("cervical")
        || hay.contains("dyad")
        || hay.contains("dicom")
        || hay.contains("hepatocyte")
    {
        recs.push(SemanticToolboxId::BiomedicalAndDicomAtlas);
    }
    if hay.contains("qecp")
        || hay.contains("pdf")
        || hay.contains("instrument")
        || entry.cml_concept_count > 0
    {
        recs.push(SemanticToolboxId::CmlDeonticAndQecp);
    }
    if hay.contains("commons")
        || hay.contains("rights")
        || hay.contains("poiesis")
        || hay.contains("intenteco")
        || entry.is_secret()
    {
        recs.push(SemanticToolboxId::PermissiveCommonsAndLifecycle);
    }
    if recs.is_empty() {
        recs.push(SemanticToolboxId::CmlDeonticAndQecp);
    }
    recs
}

/// Purpose-shaped section in the Semantic File-System (never a folder).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSection {
    All,
    Secret,
    Wellfair,
    Personal,
    Work,
    Tools,
    Software,
    Commons,
}

impl SemanticSection {
    pub const ALL: [Self; 8] = [
        Self::All,
        Self::Secret,
        Self::Wellfair,
        Self::Personal,
        Self::Work,
        Self::Tools,
        Self::Software,
        Self::Commons,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Secret => "secret",
            Self::Wellfair => "wellfair",
            Self::Personal => "personal",
            Self::Work => "work",
            Self::Tools => "tools",
            Self::Software => "software",
            Self::Commons => "commons",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All Resources",
            Self::Secret => "Secret (Sanctuary)",
            Self::Wellfair => "WellFair & Care",
            Self::Personal => "Personal Shelf",
            Self::Work => "Work & Projects",
            Self::Tools => "Tools & Telemetry",
            Self::Software => "Software & Models",
            Self::Commons => "Permissive Commons",
        }
    }

    pub const fn blurb(self) -> &'static str {
        match self {
            Self::All => "Every ontologically indexed resource on this apparatus.",
            Self::Secret => "Sanctuary & high-sensitivity records — never eligible for Commons or Peer sharing.",
            Self::Wellfair => "Health, 3D anatomy, clinical, and welfare records (routes to Secret when classified).",
            Self::Personal => "Private life admin, notes, and unshared research.",
            Self::Work => "Cooperative project labour, engineering digital twins, statutes, and legislation.",
            Self::Tools => "Agent/tool execution receipts, logs, and technical diagnostics.",
            Self::Software => "Qaps, .p64/.gguf AI instruments, Collectable Semantic Instruments, and packages.",
            Self::Commons => "Permissive Commons share surface — visible to bilateral peers and micro-commons.",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "secret" | "sanctuary" | "private" | "classified" | "restricted" => Self::Secret,
            "wellfair" | "health" | "welfare" | "care" => Self::Wellfair,
            "personal" | "home" => Self::Personal,
            "work" | "project" | "coop" => Self::Work,
            "tools" | "tool" | "logs" | "telemetry" | "ops" => Self::Tools,
            "software" | "qapp" | "qapps" | "model" | "models" => Self::Software,
            "commons" | "public" | "share" | "permissive" => Self::Commons,
            _ => Self::All,
        }
    }
}

/// Role of a bundled asset inside a `HypermediaContainer` (`urn:qualia:hypermedia:hasRole`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticAssetRole {
    Primary,
    Source,
    Derivation,
    Lod,
    Analysis,
    Provenance,
    Related,
}

impl SemanticAssetRole {
    pub const fn class_iri(self) -> &'static str {
        match self {
            Self::Primary => "urn:qualia:hypermedia:role:primary",
            Self::Source => "urn:qualia:hypermedia:role:source",
            Self::Derivation => "urn:qualia:hypermedia:role:derivation",
            Self::Lod => "urn:qualia:hypermedia:role:lod",
            Self::Analysis => "urn:qualia:hypermedia:role:analysis",
            Self::Provenance => "urn:qualia:hypermedia:role:provenance",
            Self::Related => "urn:qualia:hypermedia:role:related",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Source => "Source",
            Self::Derivation => "Derivation",
            Self::Lod => "Harmonic LOD",
            Self::Analysis => "Analysis",
            Self::Provenance => "Provenance",
            Self::Related => "Related",
        }
    }
}

/// How far a resource may travel on social / Permissive Commons layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommonsShareScope {
    #[default]
    None,
    Peers,
    Commons,
}

impl CommonsShareScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Peers => "peers",
            Self::Commons => "commons",
        }
    }
}

/// Bundled sub-asset or derivation edge inside a semantic container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundledSubAsset {
    pub uri: String,
    pub media_type: String,
    pub role: SemanticAssetRole,
    pub summary: String,
}

/// Analytics or feature-extraction record bound back to the asset it is *about* (`analysisTarget`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetAnalyticsAttachment {
    pub id: String,
    pub method: String,
    pub summary: String,
}

/// One ontologically indexed resource in the Semantic File-System.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticAssetEntry {
    pub asset_uri: String,
    /// 60-bit FNV-1a subject hash (`q_hash_60(asset_uri)`) joining across `.10d` / `.q42` / `.hmc`.
    pub primary_subject: u64,
    pub title: String,
    pub media_type: String,
    /// Derived & person-authored topic facets (`urn:qualia:hypermedia:topic`).
    pub topics: Vec<String>,
    /// Entities, structures, species, parts, or features depicted/contained *inside* the asset (`urn:qualia:hypermedia:depicts`).
    pub depicts: Vec<String>,
    /// Cooperative projects this asset belongs to (`urn:qualia:hypermedia:inProject`).
    pub projects: Vec<String>,
    /// Purposes served (`urn:qualia:hypermedia:purpose` — e.g. `tax-return`, `medical-atlas`, `species-twin`).
    pub purposes: Vec<String>,
    /// Source asset URIs this resource was derived from (`prov:wasDerivedFrom`).
    pub derived_from: Vec<String>,
    /// Bundled sub-assets (Source, Derivation, Harmonic LOD, Analysis, Provenance).
    pub bundled_assets: Vec<BundledSubAsset>,
    /// Analytics computed *about* this asset (`analysisTarget`).
    pub analytics: Vec<AssetAnalyticsAttachment>,
    /// Optional geographic place label & coordinates (`atPlace`, `atLat`, `atLon`).
    pub place: Option<String>,
    pub lat: Option<f32>,
    pub lon: Option<f32>,
    /// Optional event timestamp (unix seconds, `occurredAt`).
    pub occurred_at: Option<i64>,
    pub ingested_unix: u64,
    pub excerpt: String,
    /// `public` | `restricted` | `classified` — high sensitivity forces `Secret` section.
    pub sensitivity: String,
    pub section: String,
    pub commons_visibility: CommonsShareScope,
    /// Context Markup Language (`TEXT -> CONCEPT -> LOGIC`) signal tags.
    pub cml_signals: Vec<String>,
    pub cml_concept_count: u32,
    /// Token-bounded COF HTML+RDFa segment count.
    pub cof_segment_count: u32,
    /// Parity-checked 48-byte `NQuin48` edge count sealed for this container.
    pub sealed_nquin_count: usize,
    /// XOR parity accumulator of the container's `NQuin48` graph edges.
    pub graph_parity_digest: u64,
}

impl SemanticAssetEntry {
    pub fn is_secret(&self) -> bool {
        self.section == SemanticSection::Secret.as_str()
            || matches!(
                self.sensitivity.trim().to_ascii_lowercase().as_str(),
                "restricted" | "classified" | "sanctuary" | "secret"
            )
    }

    pub fn recompute_section_and_quin_edges(&mut self) {
        let sens = self.sensitivity.trim().to_ascii_lowercase();
        if sens == "restricted" || sens == "classified" || sens == "sanctuary" || sens == "secret" {
            self.sensitivity = "classified".into();
            self.section = SemanticSection::Secret.as_str().into();
            self.commons_visibility = CommonsShareScope::None;
        } else if self.commons_visibility == CommonsShareScope::Commons {
            self.section = SemanticSection::Commons.as_str().into();
        } else if self.section.trim().is_empty() || self.section == "all" {
            self.section = infer_section(&self.purposes, &self.projects).as_str().into();
        }

        let quins = build_entry_nquins(self);
        self.sealed_nquin_count = quins.len();
        self.graph_parity_digest = quins.iter().fold(0u64, |acc, q| acc ^ q.parity());
    }
}

fn infer_section(purposes: &[String], projects: &[String]) -> SemanticSection {
    let blob = purposes
        .iter()
        .chain(projects.iter())
        .map(|s| s.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    if blob.contains("health")
        || blob.contains("wellfair")
        || blob.contains("anatomy")
        || blob.contains("clinical")
        || blob.contains("medical")
    {
        SemanticSection::Wellfair
    } else if blob.contains("qapp")
        || blob.contains("model")
        || blob.contains("software")
        || blob.contains("semantic-instrument")
    {
        SemanticSection::Software
    } else if blob.contains("log") || blob.contains("telemetry") || blob.contains("diagnostic") {
        SemanticSection::Tools
    } else if !projects.is_empty()
        || blob.contains("work")
        || blob.contains("legislation")
        || blob.contains("statute")
        || blob.contains("engineering")
    {
        SemanticSection::Work
    } else {
        SemanticSection::Personal
    }
}

/// Seal the complete relationship, depiction, facet, lineage, and analytics edge-graph of an entry as 48-byte `NQuin48`s.
pub fn build_entry_nquins(entry: &SemanticAssetEntry) -> Vec<NQuin48> {
    let mut quins = Vec::new();
    let uri = &entry.asset_uri;
    quins.push(NQuin48::invariant(
        uri,
        "urn:qualia:hypermedia:hasPrimary",
        entry.primary_subject,
    ));
    quins.push(NQuin48::invariant(
        uri,
        "urn:qualia:hypermedia:mediaType",
        q_hash_60(&entry.media_type),
    ));
    for topic in &entry.topics {
        quins.push(NQuin48::invariant(
            uri,
            "urn:qualia:hypermedia:topic",
            q_hash_60(topic),
        ));
    }
    for entity in &entry.depicts {
        quins.push(NQuin48::invariant(
            uri,
            "urn:qualia:hypermedia:depicts",
            q_hash_60(entity),
        ));
    }
    for proj in &entry.projects {
        quins.push(NQuin48::invariant(
            uri,
            "urn:qualia:hypermedia:inProject",
            q_hash_60(proj),
        ));
    }
    for purp in &entry.purposes {
        quins.push(NQuin48::invariant(
            uri,
            "urn:qualia:hypermedia:purpose",
            q_hash_60(purp),
        ));
    }
    for src in &entry.derived_from {
        quins.push(NQuin48::invariant(
            uri,
            "http://www.w3.org/ns/prov#wasDerivedFrom",
            q_hash_60(src),
        ));
    }
    for sub in &entry.bundled_assets {
        quins.push(NQuin48::invariant(
            uri,
            "urn:qualia:hypermedia:bundledWith",
            q_hash_60(&sub.uri),
        ));
        quins.push(NQuin48::invariant(
            &sub.uri,
            "urn:qualia:hypermedia:hasRole",
            q_hash_60(sub.role.class_iri()),
        ));
    }
    for an in &entry.analytics {
        quins.push(NQuin48::invariant(
            &an.id,
            "urn:qualia:hypermedia:analysisTarget",
            entry.primary_subject,
        ));
        quins.push(NQuin48::invariant(
            uri,
            "urn:qualia:hypermedia:analysisResult",
            q_hash_60(&an.id),
        ));
    }
    if let Some(place) = &entry.place {
        quins.push(NQuin48::invariant(
            uri,
            "urn:qualia:hypermedia:atPlace",
            q_hash_60(place),
        ));
    }
    if let Some(ts) = entry.occurred_at {
        quins.push(NQuin48::invariant(
            uri,
            "urn:qualia:hypermedia:occurredAt",
            ts as u64,
        ));
    }
    debug_assert!(quins.iter().all(|q| q.verify_parity()));
    quins
}

/// Multi-dimensional facet query (`AND` across dimensions, `OR` within a dimension).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticFacetFilter {
    pub section: String,
    pub text_query: String,
    pub entity_depicts: Option<String>,
    pub topic: Option<String>,
    pub purpose: Option<String>,
    pub project: Option<String>,
    pub place: Option<String>,
    /// Optional temporal timeline lower bound (`occurredAt >= min_occurred_unix`), unlocked by `SpatiotemporalAndGeodesy`.
    pub min_occurred_unix: Option<i64>,
}

impl SemanticFacetFilter {
    pub fn matches(&self, e: &SemanticAssetEntry) -> bool {
        let sec = self.section.trim();
        if !sec.is_empty() && sec != "all" && !e.section.eq_ignore_ascii_case(sec) {
            return false;
        }
        if let Some(min_ts) = self.min_occurred_unix {
            if e.occurred_at.unwrap_or(0) < min_ts {
                return false;
            }
        }
        if let Some(dep) = &self.entity_depicts {
            let d = dep.trim().to_ascii_lowercase();
            if !d.is_empty() && !e.depicts.iter().any(|x| x.to_ascii_lowercase().contains(&d)) {
                return false;
            }
        }
        if let Some(top) = &self.topic {
            let t = top.trim().to_ascii_lowercase();
            if !t.is_empty() && !e.topics.iter().any(|x| x.to_ascii_lowercase() == t) {
                return false;
            }
        }
        if let Some(purp) = &self.purpose {
            let p = purp.trim().to_ascii_lowercase();
            if !p.is_empty() && !e.purposes.iter().any(|x| x.to_ascii_lowercase() == p) {
                return false;
            }
        }
        if let Some(proj) = &self.project {
            let pr = proj.trim().to_ascii_lowercase();
            if !pr.is_empty() && !e.projects.iter().any(|x| x.to_ascii_lowercase() == pr) {
                return false;
            }
        }
        if let Some(pl) = &self.place {
            let pl_low = pl.trim().to_ascii_lowercase();
            if !pl_low.is_empty()
                && !e
                    .place
                    .as_ref()
                    .map(|x| x.to_ascii_lowercase().contains(&pl_low))
                    .unwrap_or(false)
            {
                return false;
            }
        }
        let q = self.text_query.trim().to_ascii_lowercase();
        if !q.is_empty() {
            let hit = e.asset_uri.to_ascii_lowercase().contains(&q)
                || e.title.to_ascii_lowercase().contains(&q)
                || e.excerpt.to_ascii_lowercase().contains(&q)
                || e.media_type.to_ascii_lowercase().contains(&q)
                || e.topics.iter().any(|x| x.to_ascii_lowercase().contains(&q))
                || e.depicts.iter().any(|x| x.to_ascii_lowercase().contains(&q))
                || e.projects.iter().any(|x| x.to_ascii_lowercase().contains(&q))
                || e.purposes.iter().any(|x| x.to_ascii_lowercase().contains(&q))
                || e.cml_signals
                    .iter()
                    .any(|x| x.to_ascii_lowercase().contains(&q))
                || e.place
                    .as_ref()
                    .map(|x| x.to_ascii_lowercase().contains(&q))
                    .unwrap_or(false);
            if !hit {
                return false;
            }
        }
        true
    }
}

/// Dynamic post-filter facet tallies for the Semantic File-System UI.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticFacetTallies {
    pub matching_entries: usize,
    pub total_quins: usize,
    pub sections: BTreeMap<String, usize>,
    pub depicts_entities: BTreeMap<String, usize>,
    pub topics: BTreeMap<String, usize>,
    pub purposes: BTreeMap<String, usize>,
    pub projects: BTreeMap<String, usize>,
    pub places: BTreeMap<String, usize>,
}

/// Complete Semantic File-System catalog state, including Poet-style opt-in `enabled_toolboxes`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SemanticFileSystem {
    pub store_path: PathBuf,
    pub entries: Vec<SemanticAssetEntry>,
    pub enabled_toolboxes: Vec<SemanticToolboxId>,
}

impl SemanticFileSystem {
    /// Load or seed the persistent Semantic File-System at `{storage}/wellfair/hypermedia_library.json`.
    pub fn load_or_seed(storage: &Path, discovered_models: &[LocalLlmModelEntry]) -> Self {
        let store_path = storage.join(HYPERMEDIA_LIBRARY_REL_PATH);
        if let Some(parent) = store_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut entries: Vec<SemanticAssetEntry> = fs::read(&store_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();

        if entries.is_empty() {
            entries = default_seeded_semantic_entries(discovered_models);
            for e in &mut entries {
                e.recompute_section_and_quin_edges();
            }
            let _ = save_entries_atomic(&store_path, &entries);
        } else {
            for e in &mut entries {
                e.recompute_section_and_quin_edges();
            }
        }

        let enabled_toolboxes = SemanticToolboxId::ALL
            .iter()
            .copied()
            .filter(|id| id.spec().default_enabled)
            .collect();

        Self {
            store_path,
            entries,
            enabled_toolboxes,
        }
    }

    pub fn is_toolbox_enabled(&self, id: SemanticToolboxId) -> bool {
        self.enabled_toolboxes.contains(&id)
    }

    /// Elect to enable or disable a Semantic Toolbox in the workspace.
    pub fn toggle_toolbox(&mut self, id: SemanticToolboxId) -> bool {
        if let Some(pos) = self.enabled_toolboxes.iter().position(|&x| x == id) {
            self.enabled_toolboxes.remove(pos);
            false
        } else {
            self.enabled_toolboxes.push(id);
            true
        }
    }

    /// Execute all currently enabled Semantic Toolboxes against a target container (`asset_uri`),
    /// enriching its `depicts` sub-entities, `cml_signals`, `bundled_assets`, `analytics`, and
    /// resealing its 48-byte `NQuin48` edge-graph.
    pub fn execute_enabled_toolboxes_on_entry(
        &mut self,
        asset_uri: &str,
    ) -> Option<SemanticAssetEntry> {
        let active_boxes = self.enabled_toolboxes.clone();
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.asset_uri == asset_uri)?;

        for tb in active_boxes {
            let spec = tb.spec();
            for &facet in spec.unlocked_facets {
                if !entry.depicts.iter().any(|d| d == facet) {
                    entry.depicts.push(facet.to_string());
                }
            }
            let sig = format!("toolbox:{}", spec.slug);
            if !entry.cml_signals.iter().any(|s| s == &sig) {
                entry.cml_signals.push(sig);
            }
            let an_id = format!("urn:qualia:toolbox-run:{}:{}", spec.slug, entry.primary_subject & 0xFFFF);
            if !entry.analytics.iter().any(|a| a.id == an_id) {
                entry.analytics.push(AssetAnalyticsAttachment {
                    id: an_id,
                    method: spec.toolchain_ops.join(" -> "),
                    summary: format!(
                        "Poet ToolChest [{}] unlocked facets: {}",
                        spec.title,
                        spec.unlocked_facets.join(", ")
                    ),
                });
            }
        }
        entry.recompute_section_and_quin_edges();
        let updated = entry.clone();
        let _ = save_entries_atomic(&self.store_path, &self.entries);
        Some(updated)
    }

    /// Filter entries by multi-dimensional [`SemanticFacetFilter`] and compute live facet tallies.
    pub fn query(&self, filter: &SemanticFacetFilter) -> (Vec<SemanticAssetEntry>, SemanticFacetTallies) {
        let mut sections = BTreeMap::new();
        for e in &self.entries {
            *sections.entry(e.section.clone()).or_insert(0) += 1;
        }
        sections.insert("all".into(), self.entries.len());

        let matched: Vec<SemanticAssetEntry> = self
            .entries
            .iter()
            .filter(|e| filter.matches(e))
            .cloned()
            .collect();

        let mut depicts_entities = BTreeMap::new();
        let mut topics = BTreeMap::new();
        let mut purposes = BTreeMap::new();
        let mut projects = BTreeMap::new();
        let mut places = BTreeMap::new();
        let mut total_quins = 0usize;

        for e in &matched {
            total_quins += e.sealed_nquin_count;
            for d in &e.depicts {
                *depicts_entities.entry(d.clone()).or_insert(0) += 1;
            }
            for t in &e.topics {
                *topics.entry(t.clone()).or_insert(0) += 1;
            }
            for p in &e.purposes {
                *purposes.entry(p.clone()).or_insert(0) += 1;
            }
            for pr in &e.projects {
                *projects.entry(pr.clone()).or_insert(0) += 1;
            }
            if let Some(pl) = &e.place {
                *places.entry(pl.clone()).or_insert(0) += 1;
            }
        }

        let tallies = SemanticFacetTallies {
            matching_entries: matched.len(),
            total_quins,
            sections,
            depicts_entities,
            topics,
            purposes,
            projects,
            places,
        };
        (matched, tallies)
    }

    /// Ingest a new asset/document into the Semantic File-System, automatically extracting
    /// topics, depicted entities/features, CML signals, and 48-byte `NQuin48` container edges.
    pub fn ingest_semantic_asset(
        &mut self,
        uri: &str,
        title: &str,
        media_type: &str,
        body_or_caption: &str,
        manual_depicts: &[String],
        manual_project: Option<&str>,
        manual_place: Option<&str>,
        sensitivity: &str,
    ) -> SemanticAssetEntry {
        let (auto_topics, auto_depicts, cml_signals) =
            extract_semantic_features(uri, media_type, body_or_caption);

        let mut depicts = auto_depicts;
        for d in manual_depicts {
            let trimmed = d.trim();
            if !trimmed.is_empty() && !depicts.iter().any(|x| x.eq_ignore_ascii_case(trimmed)) {
                depicts.push(trimmed.to_string());
            }
        }

        let mut projects = Vec::new();
        if let Some(p) = manual_project.map(str::trim).filter(|s| !s.is_empty()) {
            projects.push(p.to_string());
        }

        let purposes = if media_type.contains("10d") || media_type.contains("hmc") {
            vec!["hypermedia-embodiment".into(), "spatial-twin".into()]
        } else if media_type.contains("pdf") {
            vec!["qecp-credential".into(), "evidence".into()]
        } else {
            vec!["research".into(), "knowledge-keep".into()]
        };

        let mut entry = SemanticAssetEntry {
            asset_uri: uri.to_string(),
            primary_subject: q_hash_60(uri),
            title: if title.trim().is_empty() {
                uri.to_string()
            } else {
                title.trim().to_string()
            },
            media_type: media_type.to_string(),
            topics: auto_topics,
            depicts,
            projects,
            purposes,
            derived_from: vec![format!("{uri}#source")],
            bundled_assets: vec![
                BundledSubAsset {
                    uri: format!("{uri}#primary"),
                    media_type: media_type.to_string(),
                    role: SemanticAssetRole::Primary,
                    summary: "Primary content-addressed asset payload".into(),
                },
                BundledSubAsset {
                    uri: format!("{uri}#text-derivation"),
                    media_type: "text/plain".into(),
                    role: SemanticAssetRole::Derivation,
                    summary: "Searchable text & CML concept derivation".into(),
                },
                BundledSubAsset {
                    uri: format!("{uri}#provenance"),
                    media_type: "application/cbor-ld".into(),
                    role: SemanticAssetRole::Provenance,
                    summary: "6-column NQuin48 CBLD provenance & licence seal".into(),
                },
            ],
            analytics: vec![AssetAnalyticsAttachment {
                id: format!("urn:qualia:analysis:{}", q_hash_60(uri) & 0xFFFF_FFFF),
                method: "qualia-core-db::nlp::gazetteer+cml_context".into(),
                summary: format!(
                    "Extracted {} CML signals; 48-byte NQuin48 edge parity verified",
                    cml_signals.len()
                ),
            }],
            place: manual_place
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            lat: None,
            lon: None,
            occurred_at: Some(1_763_770_000),
            ingested_unix: 1_763_770_000 + (self.entries.len() as u64),
            excerpt: body_or_caption.chars().take(180).collect(),
            sensitivity: sensitivity.to_string(),
            section: String::new(),
            commons_visibility: CommonsShareScope::None,
            cml_signals,
            cml_concept_count: 4,
            cof_segment_count: 2,
            sealed_nquin_count: 0,
            graph_parity_digest: 0,
        };
        entry.recompute_section_and_quin_edges();
        self.entries.retain(|e| e.asset_uri != entry.asset_uri);
        self.entries.insert(0, entry.clone());
        let _ = save_entries_atomic(&self.store_path, &self.entries);
        entry
    }
}

fn save_entries_atomic(path: &Path, entries: &[SemanticAssetEntry]) -> std::io::Result<()> {
    let bytes = serde_json::to_vec_pretty(entries)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Deterministic feature, entity (`depicts`), topic, and CML signal extractor.
pub fn extract_semantic_features(
    uri: &str,
    media_type: &str,
    text: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let hay = format!("{uri} {media_type} {text}").to_ascii_lowercase();
    let mut topics = Vec::new();
    let mut depicts = Vec::new();
    let mut signals = Vec::new();

    const TOPIC_RULES: &[(&str, &[&str])] = &[
        ("anatomy", &["atlas", "axis", "cervical", "vertebra", "liver", "heart", "organ", "dicom", "dyad", "fma", "uberon"]),
        ("biology", &["eucalyptus", "wallabia", "koala", "wombat", "cell", "flora", "fauna", "canopy", "botany"]),
        ("engineering", &["rover", "solar", "microgrid", "narrowboat", "footbridge", "barn", "windpump", "10d", "holo"]),
        ("law", &["statute", "legislation", "contract", "clause", "jurisdiction", "court"]),
        ("finance", &["invoice", "receipt", "tax", "expense", "xec", "lightning", "obligation"]),
        ("privacy", &["sanctuary", "consent", "argon2id", "vault", "coercion", "erasure"]),
        ("human-rights", &["human rights", "udhr", "icescr", "crc", "crpd", "guardianship", "agency", "dignity"]),
        ("hypermedia", &["hmc", "10d", "q42", "m64", "emf", "qac", "p64", "qecp", "vibe"]),
    ];

    for (topic, words) in TOPIC_RULES {
        if words.iter().any(|w| hay.contains(w)) {
            topics.push((*topic).to_string());
        }
    }
    if topics.is_empty() {
        topics.push("knowledge".into());
    }

    const ENTITY_RULES: &[(&str, &[&str])] = &[
        ("cervical-c1-atlas", &["c1", "atlas", "cervical"]),
        ("cervical-c2-axis-dens", &["c2", "axis", "dens"]),
        ("maternal-fetal-dyad", &["dyad", "gestation", "fetal", "maternal"]),
        ("hepatocyte-liver-lobule", &["liver", "hepatocyte", "bile"]),
        ("eucalyptus-camaldulensis", &["eucalyptus", "red gum", "canopy"]),
        ("wallabia-bicolor", &["wallabia", "swamp wallaby"]),
        ("offgrid-solar-microgrid", &["solar", "battery", "microgrid"]),
        ("rover-field-utility-4wd", &["rover", "4wd", "talbingo"]),
        ("c2pa-qecp-verifiable-credential", &["qecp", "c2pa", "credential", "pathology", "letter"]),
        ("holographic-10d-v2-sdf", &["10d", "holo", "0x0306", "sdf"]),
    ];

    for (entity, triggers) in ENTITY_RULES {
        if triggers.iter().any(|w| hay.contains(w)) {
            depicts.push((*entity).to_string());
        }
    }
    if depicts.is_empty() {
        depicts.push("semantic-document-node".into());
    }

    if hay.contains("must") || hay.contains("shall") || hay.contains("obligation") {
        signals.push("deontic:obligation".into());
    }
    if hay.contains("consent") || hay.contains("sanctuary") || hay.contains("private") {
        signals.push("privacy:consent-gate".into());
    }
    if hay.contains("rights") || hay.contains("guardianship") || hay.contains("dignity") {
        signals.push("rights:human-principal".into());
    }
    if signals.is_empty() {
        signals.push("cml:Proposed".into());
    }

    (topics, depicts, signals)
}

fn default_seeded_semantic_entries(models: &[LocalLlmModelEntry]) -> Vec<SemanticAssetEntry> {
    let mut list = vec![
        SemanticAssetEntry {
            asset_uri: "urn:qualia:atlas:cervical-c1-c2.10d".into(),
            primary_subject: q_hash_60("urn:qualia:atlas:cervical-c1-c2.10d"),
            title: "Visible Human Cervical Spine (C1 Atlas & C2 Axis) · 10D v2 HOLO".into(),
            media_type: "model/qualia-10d+holo".into(),
            topics: vec!["anatomy".into(), "biology".into(), "hypermedia".into()],
            depicts: vec![
                "cervical-c1-atlas".into(),
                "cervical-c2-axis-dens".into(),
                "transverse-foramen".into(),
                "holographic-10d-v2-sdf".into(),
            ],
            projects: vec!["medical-atlas".into(), "wellfair-anatomy".into()],
            purposes: vec!["medical-atlas".into(), "anatomical-reference".into()],
            derived_from: vec!["urn:doi:10.57967/hf/10464#visible-human-ct".into()],
            bundled_assets: vec![
                BundledSubAsset {
                    uri: "urn:qualia:atlas:cervical-c1-c2.10d#holo-0x0306".into(),
                    media_type: "application/x-qualia-holo".into(),
                    role: SemanticAssetRole::Primary,
                    summary: "10D v2 HOLO (0x0306) continuous SDF + L0-L6 MSTR tissue strata".into(),
                },
                BundledSubAsset {
                    uri: "urn:doi:10.57967/hf/10464#visible-human-ct".into(),
                    media_type: "application/dicom".into(),
                    role: SemanticAssetRole::Source,
                    summary: "Aligned Visible Human CT musculoskeletal source geometry".into(),
                },
                BundledSubAsset {
                    uri: "urn:qualia:atlas:cervical-cutaways.html".into(),
                    media_type: "text/html".into(),
                    role: SemanticAssetRole::Analysis,
                    summary: "Multi-plane section cutaway topology verification (foramina & C2 dens)".into(),
                },
                BundledSubAsset {
                    uri: "urn:qualia:atlas:cervical-c1-c2.q42".into(),
                    media_type: "application/x-qualia-q42".into(),
                    role: SemanticAssetRole::Provenance,
                    summary: "FMA 5.0.0 & Uberon 6-column NQuin provenance graph".into(),
                },
            ],
            analytics: vec![AssetAnalyticsAttachment {
                id: "urn:qualia:analysis:cervical-topology-gate".into(),
                method: "compare_section_topology + synth_validate_model".into(),
                summary: "Verified vertebral & transverse foramina and C2 dens aperture continuity across male/female specimens".into(),
            }],
            place: Some("Canberra · ANU / Medical Atlas Vault".into()),
            lat: Some(-35.2809),
            lon: Some(149.1300),
            occurred_at: Some(1_763_500_000),
            ingested_unix: 1_763_500_000,
            excerpt: "Specimen-evidenced 10D v2 HOLO signed-distance fields & .m64 cortical/trabecular bone strata for adult male and female C1/C2 vertebrae.".into(),
            sensitivity: "public".into(),
            section: "wellfair".into(),
            commons_visibility: CommonsShareScope::Commons,
            cml_signals: vec!["anatomical:cavity-sdf-verified".into(), "fma:5.0.0".into()],
            cml_concept_count: 14,
            cof_segment_count: 1,
            sealed_nquin_count: 0,
            graph_parity_digest: 0,
        },
        SemanticAssetEntry {
            asset_uri: "urn:qualia:wellfair:human-welfare-dyad.hmc".into(),
            primary_subject: q_hash_60("urn:qualia:wellfair:human-welfare-dyad.hmc"),
            title: "WellFair Maternal-Fetal Dyad & 17-System Human Welfare Schematic (.hmc)".into(),
            media_type: "application/x-qualia-hmc".into(),
            topics: vec!["anatomy".into(), "human-rights".into(), "privacy".into()],
            depicts: vec![
                "maternal-fetal-dyad".into(),
                "gestational-weeks-10-22-34-38".into(),
                "digital-birth-record-schema".into(),
            ],
            projects: vec!["wellfair-core".into()],
            purposes: vec!["health-record".into(), "guardianship".into()],
            derived_from: vec!["urn:qualia:ontology:biomedical-fma-uberon.q42".into()],
            bundled_assets: vec![
                BundledSubAsset {
                    uri: "urn:qualia:wellfair:human-welfare-dyad.hmc#primary".into(),
                    media_type: "application/x-qualia-hmc".into(),
                    role: SemanticAssetRole::Primary,
                    summary: "QBDL v1 page-aligned bundle of adult & gestational .10d + .q42 volumes".into(),
                },
                BundledSubAsset {
                    uri: "urn:qualia:wellfair:dyad-burden-scorecard".into(),
                    media_type: "application/cbor-ld".into(),
                    role: SemanticAssetRole::Analysis,
                    summary: "Systemic burden accumulation & Progenitor vs gestational mother autonomy guard".into(),
                },
            ],
            analytics: vec![AssetAnalyticsAttachment {
                id: "urn:qualia:analysis:un-crc-crpd-dyad-guard".into(),
                method: "wellfare-core::anatomy::dyad + guardianship".into(),
                summary: "Enforces gestational mother bodily/data autonomy and plural Guardianship agency gradient".into(),
            }],
            place: Some("Melbourne · Sanctuary Health Keep".into()),
            lat: Some(-37.8136),
            lon: Some(144.9631),
            occurred_at: Some(1_763_550_000),
            ingested_unix: 1_763_550_000,
            excerpt: "Couples 3D systemic burden scorecards with relational guardianship (UN CRC / CRPD) and 48-byte NQuin epistemic record envelopes.".into(),
            sensitivity: "classified".into(),
            section: "secret".into(),
            commons_visibility: CommonsShareScope::None,
            cml_signals: vec![
                "privacy:sanctuary-classified".into(),
                "rights:un-crc-crpd".into(),
                "deontic:obligation".into(),
            ],
            cml_concept_count: 19,
            cof_segment_count: 3,
            sealed_nquin_count: 0,
            graph_parity_digest: 0,
        },
        SemanticAssetEntry {
            asset_uri: "urn:intenteco:species:eucalyptus-camaldulensis.hmc".into(),
            primary_subject: q_hash_60("urn:intenteco:species:eucalyptus-camaldulensis.hmc"),
            title: "River Red Gum (Eucalyptus camaldulensis) · 7-Layer Biosphere Digital Twin".into(),
            media_type: "application/x-qualia-hmc".into(),
            topics: vec!["biology".into(), "hypermedia".into()],
            depicts: vec![
                "eucalyptus-camaldulensis".into(),
                "sggx-microflake-canopy".into(),
                "holographic-10d-v2-sdf".into(),
            ],
            projects: vec!["ecopunk".into(), "talbingo-continuum".into()],
            purposes: vec!["species-twin".into(), "restoration".into()],
            derived_from: vec!["urn:intenteco:dossier:eucalyptus-camaldulensis.q42.yaml".into()],
            bundled_assets: vec![
                BundledSubAsset {
                    uri: "urn:intenteco:species:eucalyptus-camaldulensis.10d".into(),
                    media_type: "model/qualia-10d+holo".into(),
                    role: SemanticAssetRole::Primary,
                    summary: "L0-L6 multi-scale flora archetype with SGGX leaf biophysics".into(),
                },
                BundledSubAsset {
                    uri: "urn:intenteco:species:eucalyptus-camaldulensis-lod.10d".into(),
                    media_type: "model/qualia-10d".into(),
                    role: SemanticAssetRole::Lod,
                    summary: "Scale-space harmonic LODs k=0..K for film, game & cartoon profiles".into(),
                },
            ],
            analytics: vec![AssetAnalyticsAttachment {
                id: "urn:qualia:analysis:leaf-spectral-metamer".into(),
                method: "etalon-render::leaf_biophysics + metamer_fspm".into(),
                summary: "Spectral colour & metamer reflectance verified under fire-ecology and hydrology regimes".into(),
            }],
            place: Some("Talbingo · Kosciuszko National Park (B72 Corridor)".into()),
            lat: Some(-35.5833),
            lon: Some(148.3000),
            occurred_at: Some(1_763_600_000),
            ingested_unix: 1_763_600_000,
            excerpt: "SE Australian canopy digital twin governed under the Social Permissive Commons with L0-L6 MSTR strata.".into(),
            sensitivity: "public".into(),
            section: "commons".into(),
            commons_visibility: CommonsShareScope::Commons,
            cml_signals: vec!["commons:social-permissive-commons".into(), "dom-ax-1:biosphere".into()],
            cml_concept_count: 11,
            cof_segment_count: 2,
            sealed_nquin_count: 0,
            graph_parity_digest: 0,
        },
        SemanticAssetEntry {
            asset_uri: "urn:poiesis:artefact:offgrid-solar-battery-station.hmc".into(),
            primary_subject: q_hash_60("urn:poiesis:artefact:offgrid-solar-battery-station.hmc"),
            title: "Community Grounds Off-Grid Solar & Battery Microgrid Station (.hmc)".into(),
            media_type: "application/x-qualia-hmc".into(),
            topics: vec!["engineering".into(), "hypermedia".into(), "finance".into()],
            depicts: vec![
                "offgrid-solar-microgrid".into(),
                "aref-iovr-inst-assembly".into(),
                "holographic-10d-v2-sdf".into(),
            ],
            projects: vec!["civics-grounds".into(), "poiesis-made-world".into()],
            purposes: vec!["site-planning".into(), "cooperative-infrastructure".into()],
            derived_from: vec!["https://dev.civics.au/site-planner#microgrid-spec".into()],
            bundled_assets: vec![
                BundledSubAsset {
                    uri: "urn:poiesis:artefact:offgrid-solar-battery-station.10d".into(),
                    media_type: "model/qualia-10d+holo".into(),
                    role: SemanticAssetRole::Primary,
                    summary: "Two-tier AREF/IOVR/INST PGA assembly & EMF field brick sidecar".into(),
                },
                BundledSubAsset {
                    uri: "urn:poiesis:artefact:solar-load-balance.q42".into(),
                    media_type: "application/x-qualia-q42".into(),
                    role: SemanticAssetRole::Analysis,
                    summary: "9-stage cooperative contribution & load-profile sustainability ledger".into(),
                },
            ],
            analytics: vec![AssetAnalyticsAttachment {
                id: "urn:qualia:analysis:microgrid-load-fem".into(),
                method: "qualia-sim + etalon-sim::yee_fdtd".into(),
                summary: "Verified thermal/water/solar balance and Poynting flux envelope for Community Grounds rig".into(),
            }],
            place: Some("Sydney · Community Grounds Modelling Lab".into()),
            lat: Some(-33.8688),
            lon: Some(151.2093),
            occurred_at: Some(1_763_650_000),
            ingested_unix: 1_763_650_000,
            excerpt: "Made-world (qualia-poiesis) 7-layer digital twin coupled to dev.civics.au site-planner and Two-Loop Permissive Commons accounting.".into(),
            sensitivity: "public".into(),
            section: "work".into(),
            commons_visibility: CommonsShareScope::Peers,
            cml_signals: vec!["dom-ax-1:made-world".into(), "cop:nine-stage-lifecycle".into()],
            cml_concept_count: 16,
            cof_segment_count: 2,
            sealed_nquin_count: 0,
            graph_parity_digest: 0,
        },
        SemanticAssetEntry {
            asset_uri: "urn:qualia:qecp:2026-pathology-and-tax-receipt.pdf".into(),
            primary_subject: q_hash_60("urn:qualia:qecp:2026-pathology-and-tax-receipt.pdf"),
            title: "QECP C2PA-Embedded Verifiable Credential PDF (Clinical & Tax Evidence)".into(),
            media_type: "application/pdf+qecp".into(),
            topics: vec!["finance".into(), "law".into(), "privacy".into()],
            depicts: vec![
                "c2pa-qecp-verifiable-credential".into(),
                "hepatocyte-liver-lobule".into(),
                "mldsa65-nquin48-assertion".into(),
            ],
            projects: vec!["personal-records".into()],
            purposes: vec!["tax-return".into(), "expenses-claim".into(), "evidence".into()],
            derived_from: vec!["urn:qualia:issuer:au-pathology-and-ato-gateway".into()],
            bundled_assets: vec![
                BundledSubAsset {
                    uri: "urn:qualia:qecp:2026-pathology-and-tax-receipt.pdf#c2pa-jumbf".into(),
                    media_type: "application/cbor-ld".into(),
                    role: SemanticAssetRole::Provenance,
                    summary: "C2PA JUMBF manifest carrying qualia:credential (W3C VC) + qualia:native_credential (NQuin48 + ML-DSA-65)".into(),
                },
                BundledSubAsset {
                    uri: "urn:qualia:qecp:2026-pathology-and-tax-receipt.pdf#cof-seg-0".into(),
                    media_type: "text/html+rdfa".into(),
                    role: SemanticAssetRole::Derivation,
                    summary: "COF HTML+RDFa index segment extracted without OCR ambiguity".into(),
                },
            ],
            analytics: vec![AssetAnalyticsAttachment {
                id: "urn:qualia:analysis:qecp-signature-verify".into(),
                method: "qualia-core-db::sparql_mm::verify_qecp_jumbf".into(),
                summary: "Cryptographically bound to both PDF bytes and visible text-content hash (proves issuer origin & integrity)".into(),
            }],
            place: Some("Brisbane · Personal Sanctuary Archive".into()),
            lat: Some(-27.4705),
            lon: Some(153.0260),
            occurred_at: Some(1_763_700_000),
            ingested_unix: 1_763_700_000,
            excerpt: "Renders in any standard PDF viewer while allowing QualiaDB to extract structured RDF/NQuin credentials directly without OCR.".into(),
            sensitivity: "public".into(),
            section: "personal".into(),
            commons_visibility: CommonsShareScope::None,
            cml_signals: vec![
                "vc:origin-and-integrity-only".into(),
                "qecp:c2pa-jumbf-verified".into(),
            ],
            cml_concept_count: 9,
            cof_segment_count: 2,
            sealed_nquin_count: 0,
            graph_parity_digest: 0,
        },
        SemanticAssetEntry {
            asset_uri: "urn:qualia:instrument:wellfair-fairness-rights-review.hmc".into(),
            primary_subject: q_hash_60("urn:qualia:instrument:wellfair-fairness-rights-review.hmc"),
            title: "Semantic Instrument SI-05 · WellFair Fairness & Human-Rights Review (.hmc)".into(),
            media_type: "application/x-qualia-semantic-instrument".into(),
            topics: vec!["human-rights".into(), "law".into(), "hypermedia".into()],
            depicts: vec![
                "collectable-hmc-logic-package".into(),
                "six-vectors-of-transparency".into(),
                "10d-visual-badge-presentation".into(),
            ],
            projects: vec!["humanitarian-equity-license".into()],
            purposes: vec!["semantic-instrument".into(), "rights-review".into()],
            derived_from: vec!["urn:qualia:ontology:rights-instrument-model.ttl".into()],
            bundled_assets: vec![
                BundledSubAsset {
                    uri: "urn:qualia:instrument:wellfair-fairness-rights-review.hmc#definition.q42".into(),
                    media_type: "application/x-qualia-q42".into(),
                    role: SemanticAssetRole::Primary,
                    summary: "Instrument ontology + input-shapes.q42 + output-shapes.q42 + N3Logic rules".into(),
                },
                BundledSubAsset {
                    uri: "urn:qualia:instrument:wellfair-fairness-rights-review.hmc#badge.10d".into(),
                    media_type: "model/qualia-10d+holo".into(),
                    role: SemanticAssetRole::Related,
                    summary: "3D .10d BadgePresentation citing immutable InstrumentRelease".into(),
                },
            ],
            analytics: vec![AssetAnalyticsAttachment {
                id: "urn:qualia:analysis:si-05-preflight".into(),
                method: "semantic_instruments::evaluators_rights::WellfairFairnessRightsReview".into(),
                summary: "Tier-1 zero-heap SlgArena entry point ready; emits contextual 48-byte NQuin execution receipts".into(),
            }],
            place: Some("Adelaide · Cooperative Commons Registry".into()),
            lat: Some(-34.9285),
            lon: Some(138.6007),
            occurred_at: Some(1_763_720_000),
            ingested_unix: 1_763_720_000,
            excerpt: "Collectable HMC logic package evaluating human welfare, creator economic justice, guardianship protections, and Six Vectors of Transparency.".into(),
            sensitivity: "public".into(),
            section: "software".into(),
            commons_visibility: CommonsShareScope::Commons,
            cml_signals: vec!["si:HonestyPolicy".into(), "rights:udhr-art-27".into()],
            cml_concept_count: 12,
            cof_segment_count: 1,
            sealed_nquin_count: 0,
            graph_parity_digest: 0,
        },
    ];

    // Also index up to 4 discovered local .p64 / .gguf AI models into the Software section
    for (idx, m) in models.iter().take(4).enumerate() {
        let uri = format!("model://local/{}", m.name);
        list.push(SemanticAssetEntry {
            asset_uri: uri.clone(),
            primary_subject: q_hash_60(&uri),
            title: format!("Local AI Instrument · {} ({} MiB)", m.name, m.size_mib),
            media_type: "application/x-webizen-model".into(),
            topics: vec!["hypermedia".into()],
            depicts: vec![
                "governed-local-ai-instrument".into(),
                m.format.to_ascii_lowercase().replace(' ', "-"),
            ],
            projects: vec!["local-ai-vault".into()],
            purposes: vec!["model".into(), "software".into()],
            derived_from: vec![m.path.display().to_string()],
            bundled_assets: vec![BundledSubAsset {
                uri: m.path.display().to_string(),
                media_type: "application/octet-stream".into(),
                role: SemanticAssetRole::Primary,
                summary: format!("{} · {} MiB zero-copy mapped weights", m.format, m.size_mib),
            }],
            analytics: vec![AssetAnalyticsAttachment {
                id: format!("urn:qualia:analysis:model-slot-{idx}"),
                method: "etalon-model + qualia-inference-kernel".into(),
                summary: "Governed by 42 MiB SlgArena N3Logic commit-on-permit gate".into(),
            }],
            place: None,
            lat: None,
            lon: None,
            occurred_at: Some(1_763_740_000 + idx as i64),
            ingested_unix: 1_763_740_000 + idx as u64,
            excerpt: format!(
                "Local AI instrument at {} ({} · {} MiB). Models are tools on the Instrument plane, never people.",
                m.path.display(),
                m.format,
                m.size_mib
            ),
            sensitivity: "public".into(),
            section: "software".into(),
            commons_visibility: CommonsShareScope::None,
            cml_signals: vec!["plane:Instrument".into(), "n3:commit-on-permit".into()],
            cml_concept_count: 3,
            cof_segment_count: 0,
            sealed_nquin_count: 0,
            graph_parity_digest: 0,
        });
    }

    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_fs_seeds_and_queries_by_entity_depiction_topic_and_section() {
        let tmp = std::env::temp_dir().join(format!("webizen-semfs-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let mut sfs = SemanticFileSystem::load_or_seed(&tmp, &[]);
        assert!(sfs.entries.len() >= 6);

        // 1. Search by internal depicted entity/feature (`cervical-c1-atlas`)
        let (cervical_hits, tallies) = sfs.query(&SemanticFacetFilter {
            entity_depicts: Some("cervical-c1-atlas".into()),
            ..Default::default()
        });
        assert_eq!(cervical_hits.len(), 1);
        assert_eq!(
            cervical_hits[0].asset_uri,
            "urn:qualia:atlas:cervical-c1-c2.10d"
        );
        assert!(tallies.total_quins >= 10);

        // 2. Verify high-sensitivity items force `secret` section and block Commons sharing
        let ingested = sfs.ingest_semantic_asset(
            "urn:qualia:clinical:private-biopsy.qecp.pdf",
            "Private Liver Biopsy QECP Report",
            "application/pdf+qecp",
            "Hepatocyte and liver lobule histopathology under Sanctuary consent.",
            &["stained-biopsy-slide".into()],
            Some("wellfair-clinical"),
            Some("Melbourne"),
            "classified",
        );
        assert_eq!(ingested.section, "secret");
        assert_eq!(ingested.commons_visibility, CommonsShareScope::None);
        assert!(ingested
            .depicts
            .contains(&"hepatocyte-liver-lobule".to_string()));
        assert!(ingested
            .depicts
            .contains(&"stained-biopsy-slide".to_string()));
        assert!(ingested.sealed_nquin_count > 6);

        // 3. Verify Poet-style opt-in Semantic Toolbox toggling and execution on a container
        assert!(sfs.is_toolbox_enabled(SemanticToolboxId::AssemblyAndStrata10D));
        assert!(!sfs.is_toolbox_enabled(SemanticToolboxId::HolosonicAndBioacoustics));
        let now_on = sfs.toggle_toolbox(SemanticToolboxId::HolosonicAndBioacoustics);
        assert!(now_on);
        let enriched = sfs
            .execute_enabled_toolboxes_on_entry("urn:intenteco:species:eucalyptus-camaldulensis.hmc")
            .expect("entry exists");
        assert!(enriched
            .depicts
            .contains(&"qac:sonic-token-ump".to_string()));
        assert!(enriched
            .depicts
            .contains(&"mstr:L0-L6-strata".to_string()));

        let _ = fs::remove_dir_all(&tmp);
    }
}
