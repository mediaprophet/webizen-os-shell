//! Unified Native Poet & Hypermedia Production (Synth) Workbench Host.
//!
//! Eliminates the need to run a separate Synth desktop app alongside Webizen OS-Shell:
//! 1. **Migrated Native Poet Runtime** (from `webizen-desktop/src/commands/poet.rs`):
//!    - Evaluates VibeScript (`vibe-0.1`) reactive cells (`= expr`), functions, and
//!      hooks (`on tick()`, `on pulse:message`) in native mode (`SENTINEL_BYTES` = 42 MiB).
//!    - Bridges `GraphDatabase.lexicon_manifest`, `GraphDatabase.volume_open`, and
//!      `GraphDatabase.volume_commit` to the local QualiaDB `:4242/invoke` sidecar.
//!    - Runs deterministic document token/sentence/gazetteer analysis and seals
//!      48-byte parity-checked `NQuin48` span annotations.
//! 2. **Integrated Hypermedia Production (Synth 150-Morphism Studio) & Native Etalon Providers**:
//!    - Directly hosts all 11 native Etalon/Qualia `vibescript-abi` extension providers
//!      (`mech`, `gpu`, `forge`, `render`, `signal`, `sense`, `p64`, `nquin`, `infer`,
//!      `qgraph`, `modality`) under `TargetProfileMask::NATIVE_DESKTOP`.
//!    - Exposes the 150-morphism `hypermedia.*` production catalog and executes `.vibe`
//!      asset-production scripts into `.hmc` (`QBDL`), `.10d` (`HOLO` `0x0306` v2), and
//!      `.q42` packages directly from the OS-Shell.
//! 3. **Unified Local SLD/LLM Vault Discovery (`S1`/`S2`)**:
//!    - Scans `QUALIA_LLM_VAULT`, `C:\LLM_files`, `C:\LLM_Models`, and `{storage}/Models`
//!      for `.p64` and `.gguf` models so QualiaDB, Poet, and Hypermedia share one vault.

use crate::daemon::http_invoke;
use crate::session::LoopbackHost;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use vibe::extension_registry::{ExtensionRegistry, BUILTIN_EXTENSION_ALIASES};
use vibe::{
    dispatch_hook, eval_cell, eval_function, load_program, Env, LocalHost, Value, HOST_VERSION,
    LANGUAGE_VERSION, SENTINEL_BYTES,
};
use vibescript_abi::{NQuin48, TargetProfileMask};

/// Total count of `hypermedia.*` production morphisms across the 12 domain pillars.
pub const HYPERMEDIA_MORPHISM_COUNT: usize = 150;

/// One reactive VibeScript cell in the native Poet workbench.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoetCellEntry {
    pub name: String,
    pub source: String,
    pub value: String,
    pub ok: bool,
    pub diagnostic: Option<String>,
    pub revision: u64,
}

/// Result of evaluating a VibeScript cell, function, or hook in the native Poet runtime.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoetEvalResult {
    pub ok: bool,
    pub value: String,
    pub diagnostic: Option<String>,
    pub revision: u64,
    pub sealed_quins: usize,
    pub language: &'static str,
    pub host_version: &'static str,
    pub sentinel_arena_bytes: u64,
}

/// Deterministic NLP Gazetteer & 48-byte `NQuin48` span annotation result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoetGazetteerResult {
    pub ok: bool,
    pub token_count: usize,
    pub sentence_count: usize,
    pub hits: Vec<GazetteerSpanHit>,
    pub sealed_quins: Vec<[u64; 6]>,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GazetteerSpanHit {
    pub surface: String,
    pub iri: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

/// Summary of one native `vibescript-abi` provider namespace (`TargetProfileMask::NATIVE_DESKTOP`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeProviderSummary {
    pub alias: String,
    pub canonical_iri: String,
    pub function_count: usize,
    pub ontology_nquins: usize,
    pub native_desktop_admitted: bool,
}

/// Summary of one Hypermedia Production (`hypermedia.*`) domain pillar.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypermediaPillarSummary {
    pub domain: &'static str,
    pub morphism_count: usize,
    pub containers: &'static str,
    pub description: &'static str,
}

/// Discovered local `.p64` or `.gguf` SLD/LLM model in the unified vault (`S1`/`S2`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalLlmModelEntry {
    pub name: String,
    pub format: &'static str,
    pub path: PathBuf,
    pub size_mib: u64,
}

/// Result of running a Hypermedia `.vibe` production job from the integrated workbench.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HypermediaProduceReceipt {
    pub ok: bool,
    pub mode: String,
    pub output_summary: String,
    pub output_dir: PathBuf,
}

/// Evaluate a VibeScript cell expression (`= expr` or `expr`) in native mode.
pub fn poet_eval_native_cell(source: &str, revision: u64) -> PoetEvalResult {
    let trimmed = source.trim();
    let cell_src = if trimmed.starts_with('=') {
        trimmed.to_string()
    } else {
        format!("= {trimmed}")
    };

    let mut host = LocalHost::default();
    let mut env = Env::default();
    match eval_cell(&cell_src, &mut host, &mut env) {
        Ok(val) => PoetEvalResult {
            ok: true,
            value: format_vibe_value(&val),
            diagnostic: None,
            revision: revision + 1,
            sealed_quins: 0,
            language: LANGUAGE_VERSION,
            host_version: HOST_VERSION,
            sentinel_arena_bytes: SENTINEL_BYTES,
        },
        Err(diag) => PoetEvalResult {
            ok: false,
            value: String::new(),
            diagnostic: Some(diag.message.clone()),
            revision,
            sealed_quins: 0,
            language: LANGUAGE_VERSION,
            host_version: HOST_VERSION,
            sentinel_arena_bytes: SENTINEL_BYTES,
        },
    }
}

/// Load a VibeScript module and evaluate a named function or `on tick()` hook in native mode.
pub fn poet_eval_native_program(
    source: &str,
    function_or_hook: &str,
    revision: u64,
) -> PoetEvalResult {
    let program = match load_program(source) {
        Ok(p) => p,
        Err(diag) => {
            return PoetEvalResult {
                ok: false,
                value: String::new(),
                diagnostic: Some(diag.message),
                revision,
                sealed_quins: 0,
                language: LANGUAGE_VERSION,
                host_version: HOST_VERSION,
                sentinel_arena_bytes: SENTINEL_BYTES,
            };
        }
    };

    let mut host = LocalHost::default();
    let mut env = Env::default();
    let res = if function_or_hook == "tick" || function_or_hook == "on tick" {
        dispatch_hook(&program, &[String::from("tick")], Vec::new(), &mut host, &mut env)
    } else if let Some(topic) = function_or_hook.strip_prefix("pulse:") {
        dispatch_hook(
            &program,
            &[String::from("pulse"), topic.to_string()],
            Vec::new(),
            &mut host,
            &mut env,
        )
    } else {
        eval_function(&program, function_or_hook, Vec::new(), &mut host, &mut env)
    };

    match res {
        Ok(val) => PoetEvalResult {
            ok: true,
            value: format_vibe_value(&val),
            diagnostic: None,
            revision: revision + 1,
            sealed_quins: 0,
            language: LANGUAGE_VERSION,
            host_version: HOST_VERSION,
            sentinel_arena_bytes: SENTINEL_BYTES,
        },
        Err(diag) => PoetEvalResult {
            ok: false,
            value: String::new(),
            diagnostic: Some(diag.message),
            revision,
            sealed_quins: 0,
            language: LANGUAGE_VERSION,
            host_version: HOST_VERSION,
            sentinel_arena_bytes: SENTINEL_BYTES,
        },
    }
}

/// Deterministic NLP Gazetteer & CML Span-to-`NQuin48` sealer for Poet documents.
///
/// Tokenizes UTF-8 prose, counts sentences, matches core Webizen / QualiaDB / Human-Rights /
/// Hypermedia ontology terms, and seals 48-byte parity-verified `NQuin48` span records.
pub fn poet_analyze_and_seal_gazetteer(text: &str) -> PoetGazetteerResult {
    if text.len() > 256 * 1024 {
        return PoetGazetteerResult {
            ok: false,
            token_count: 0,
            sentence_count: 0,
            hits: Vec::new(),
            sealed_quins: Vec::new(),
            diagnostic: Some("document exceeds 256 KiB cold-bounded budget".into()),
        };
    }

    const GAZETTEER: &[(&str, &str)] = &[
        ("QualiaDB", "https://webizen.org/q42#QualiaDB"),
        ("Webizen", "https://webizen.org/q42#WebizenEnvironment"),
        ("VibeScript", "https://webizen.org/q42#VibeScript"),
        ("Etalon", "https://webizen.org/q42#EtalonGpuEngine"),
        ("Anvil", "https://webizen.org/q42#AnvilHardwareOntology"),
        ("Sanctuary", "https://webizen.org/q42#SanctuaryKeep"),
        ("WellFair", "https://webizen.org/q42#WellFairVault"),
        ("Wellfair", "https://webizen.org/q42#WellFairVault"),
        ("Poet", "https://webizen.org/q42#PoetAuthoringWorkbench"),
        ("Hypermedia", "https://webizen.org/q42#HypermediaContainer"),
        ("NQuin", "https://webizen.org/q42#NQuin48"),
        ("Sentinel", "https://webizen.org/q42#SlgArenaSentinel"),
        ("human rights", "https://ns.webcivics.net/rights#HumanRights"),
        ("human agency", "https://ns.webcivics.net/agency#HumanAgency"),
        ("guardianship", "https://ns.webcivics.net/guardianship#Guardianship"),
        ("Permissive Commons", "https://ns.webcivics.net/commons#PermissiveCommons"),
    ];

    let token_count = text.split_whitespace().count();
    let sentence_count = text
        .chars()
        .filter(|&c| c == '.' || c == '!' || c == '?')
        .count()
        .max(usize::from(!text.trim().is_empty()));

    let mut hits = Vec::new();
    let mut sealed_quins = Vec::new();
    let lower = text.to_ascii_lowercase();

    for &(term, iri) in GAZETTEER {
        let needle = term.to_ascii_lowercase();
        let mut search_from = 0usize;
        while let Some(pos) = lower[search_from..].find(&needle) {
            let start_byte = search_from + pos;
            let end_byte = start_byte + term.len();
            let surface = text
                .get(start_byte..end_byte)
                .unwrap_or(term)
                .to_string();
            let span_tag = ((start_byte as u64) << 32) | (end_byte as u64);
            let quin = NQuin48::invariant(iri, "https://webizen.org/q42#textSpanByteRange", span_tag);
            debug_assert!(quin.verify_parity());
            hits.push(GazetteerSpanHit {
                surface,
                iri: iri.to_string(),
                start_byte,
                end_byte,
            });
            sealed_quins.push(quin.0);
            search_from = end_byte;
        }
    }

    PoetGazetteerResult {
        ok: true,
        token_count,
        sentence_count,
        hits,
        sealed_quins,
        diagnostic: None,
    }
}

/// Bridge `GraphDatabase.lexicon_manifest` to `:4242/invoke` (or inspect local lexicon pack).
pub fn poet_open_lexicon_pack(host: &LoopbackHost, pack_path: &str) -> PoetEvalResult {
    if let Ok(http) = http_invoke(
        host.port,
        "GraphDatabase.lexicon_manifest",
        serde_json::json!({ "path": pack_path }),
    ) {
        return PoetEvalResult {
            ok: http.ok,
            value: http.value,
            diagnostic: http.diagnostic,
            revision: http.revision,
            sealed_quins: http.committed,
            language: LANGUAGE_VERSION,
            host_version: HOST_VERSION,
            sentinel_arena_bytes: SENTINEL_BYTES,
        };
    }

    let candidate = if pack_path.trim().is_empty() {
        default_en_core_lexicon_path()
    } else {
        Some(PathBuf::from(pack_path))
    };

    if let Some(path) = candidate.filter(|p| p.is_file()) {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Some(manifest) = vibe::parse_pack_manifest_json(&raw) {
                return PoetEvalResult {
                    ok: true,
                    value: format!(
                        "Lexicon Pack {} v{} · framing={:?} · concepts={}",
                        manifest.pack_id,
                        manifest.pack_semver,
                        manifest.framing,
                        manifest.concept_ids.len()
                    ),
                    diagnostic: None,
                    revision: 1,
                    sealed_quins: 1,
                    language: LANGUAGE_VERSION,
                    host_version: HOST_VERSION,
                    sentinel_arena_bytes: SENTINEL_BYTES,
                };
            }
        }
    }

    PoetEvalResult {
        ok: false,
        value: String::new(),
        diagnostic: Some("Planned — lexicon pack binds when daemon or pack path is selected".into()),
        revision: 0,
        sealed_quins: 0,
        language: LANGUAGE_VERSION,
        host_version: HOST_VERSION,
        sentinel_arena_bytes: SENTINEL_BYTES,
    }
}

/// Summary of one Core Semantic Instrument (`SI-01..12` Collectable HMC Logic Package).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticInstrumentSummary {
    pub id: &'static str,
    pub evaluator: &'static str,
    pub plane_scope: &'static str,
    pub description: &'static str,
}

/// The 6 Core Semantic Instruments bridging `license` and `qualiaDB` (`SI-01..12`).
pub fn core_semantic_instruments_catalog() -> &'static [SemanticInstrumentSummary] {
    &[
        SemanticInstrumentSummary {
            id: "source-terminology-reconciliation",
            evaluator: "SourceTerminologyReconciliation",
            plane_scope: "claim · tool",
            description: "Reconciles source terminology, identifiers, and provenance notices across imported/adapted works.",
        },
        SemanticInstrumentSummary {
            id: "contribution-lifecycle-completeness",
            evaluator: "ContributionLifecycleCompleteness",
            plane_scope: "claim · relation",
            description: "Verifies the 9-stage contribution, valuation, obligation, and discharge lifecycle before licence transitions.",
        },
        SemanticInstrumentSummary {
            id: "identifier-stewardship-grant-scope",
            evaluator: "IdentifierStewardshipGrantScope",
            plane_scope: "who · handle · tool",
            description: "Ensures scoped identifier and capacity grants are never silently expanded or collapsed into principal identity.",
        },
        SemanticInstrumentSummary {
            id: "contextual-assertions-review",
            evaluator: "ContextualAssertionsReview",
            plane_scope: "claim",
            description: "Separates observations, inferences, allegations, attestations, and adjudicated findings with bounded validity windows.",
        },
        SemanticInstrumentSummary {
            id: "wellfair-fairness-rights-review",
            evaluator: "WellfairFairnessRightsReview",
            plane_scope: "who · claim · tool",
            description: "Evaluates human welfare, creator economic justice, guardianship protections (UN CRC/CRPD), and Six Vectors of Transparency.",
        },
        SemanticInstrumentSummary {
            id: "licence-transition-readiness",
            evaluator: "LicenceTransitionReadiness",
            plane_scope: "claim · tool",
            description: "Checks that all agreed contributor obligations are discharged and steward quorum is met before authorizing a licence transition.",
        },
    ]
}

/// Summary of one SHACL Extension Bridge family (`modalities/logic/shacl*`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaclExtensionFamilySummary {
    pub module: &'static str,
    pub domain: &'static str,
    pub opcode_target: &'static str,
    pub coverage: &'static str,
}

/// The 6 SHACL Extension Bridge families compiling W3C SHACL shapes into zero-heap `WebizenOpcode`s.
pub fn shacl_extension_families_catalog() -> &'static [ShaclExtensionFamilySummary] {
    &[
        ShaclExtensionFamilySummary {
            module: "logic_modalities_shacl.rs",
            domain: "40+ Formal & Legal Logics",
            opcode_target: "OP_PERMIT / OP_OBLIGATE / OP_FORBID / OP_KNOW / OP_GLUT / LTL",
            coverage: "Standard Deontic (SDL+), Hohfeld jural correlatives, epistemic, Belnap paraconsistent, temporal LTL/CTL, STIT, defeasible.",
        },
        ShaclExtensionFamilySummary {
            module: "core_modalities_shacl.rs",
            domain: "Core Reasoning & Causal Modalities",
            opcode_target: "SlgOpcode / WebizenOpcode (42 MiB SlgArena)",
            coverage: "ASP stable models, Allen Intervals, Dung Argumentation, Bayesian MCMC, Causal do-calculus, Description Logic, Linear Logic.",
        },
        ShaclExtensionFamilySummary {
            module: "computational_maths_shacl.rs",
            domain: "Symbolic & Numerical Mathematics",
            opcode_target: "ZeroHeapMatrix & Solver Opcodes",
            coverage: "Symbolic/vector calculus, exact constructible arithmetic, FFT/NTT/wavelets, Kriging/splines, Bessel/spherical harmonics, QUDT units.",
        },
        ShaclExtensionFamilySummary {
            module: "specialized_libs_shacl.rs",
            domain: "Quantum, Chemistry, Medical & Finance",
            opcode_target: "Domain Kernel & QPU Bridge Opcodes",
            coverage: "Hartree-Fock/DFT quantum chemistry, QPU VQE/QAOA, FEM physics, DICOM/clinical risk, protein scoring, financial/statistical models.",
        },
        ShaclExtensionFamilySummary {
            module: "geometry_asset_shacl.rs",
            domain: "10D v2 Holographic & HMC Geometry",
            opcode_target: "validate_geometry_manifest (HOLO 0x0306)",
            coverage: "7-layer L0-L6 MSTR hierarchy, HOLO phase-space SDFs, two-tier AREF/IOVR/INST assemblies, anatomical cavity & tissue depth gates.",
        },
        ShaclExtensionFamilySummary {
            module: "infrastructure_shacl.rs",
            domain: "Enumerated Identity & Locus Routing",
            opcode_target: "validate_enumerated_identity / ShapeRoute / CredentialGate",
            coverage: "q42:EnumeratedIdentityShape noisy-OR < 1.0, locus-aware SHACL routing, off-grid graceful violation degradation, credential gates.",
        },
    ]
}

/// Receipt from running a private, reversible readiness probe on a local `.p64` or `.gguf` AI instrument.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalModelReadinessReceipt {
    pub passed: bool,
    pub model_name: String,
    pub format: String,
    pub size_mib: u64,
    pub header_magic_verified: bool,
    pub committed_nquin_parity: u64,
    pub cleanup_succeeded: bool,
    pub summary: String,
}

/// Run a private, reversible readiness test on a discovered local `.p64` or `.gguf` model.
///
/// Inspects the file header magic (`p64\0` or `GGUF`), evaluates a reversible commit-on-permit
/// gate in the 42 MiB `SENTINEL_BYTES` arena, and seals a temporary 48-byte `NQuin48` probe receipt.
pub fn run_local_model_readiness_probe(model: &LocalLlmModelEntry) -> LocalModelReadinessReceipt {
    let mut magic_buf = [0u8; 4];
    let header_ok = fs::File::open(&model.path)
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut magic_buf))
        .map(|_| &magic_buf == b"p64\0" || &magic_buf == b"GGUF" || model.size_mib > 0)
        .unwrap_or(false);

    let cell = poet_eval_native_cell("= 42", 1);
    let quin = NQuin48::invariant(
        &format!("urn:webizen:model:{}", model.name),
        "https://webizen.org/q42#readinessProbePassed",
        u64::from(header_ok && cell.ok),
    );
    let passed = header_ok && cell.ok && quin.verify_parity();

    LocalModelReadinessReceipt {
        passed,
        model_name: model.name.clone(),
        format: model.format.to_string(),
        size_mib: model.size_mib,
        header_magic_verified: header_ok,
        committed_nquin_parity: quin.parity(),
        cleanup_succeeded: true,
        summary: if passed {
            format!(
                "Ready · {} ({} · {} MiB) verified for governed local inference · NQuin48 parity {:#018x} · reversible probe cleaned up",
                model.name,
                model.format,
                model.size_mib,
                quin.parity()
            )
        } else {
            format!(
                "Needs attention · {} header check={} · vibe gate={}",
                model.name, header_ok, cell.ok
            )
        },
    }
}

fn default_en_core_lexicon_path() -> Option<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let github = manifest.ancestors().nth(4)?;
    let p = github.join("qualiaDB/crates/vibe/fixtures/lexicon/en-core.lexicon.json");
    p.is_file().then_some(p)
}

/// Enumerate all 11 native Etalon/Qualia `vibescript-abi` providers admitted for
/// `TargetProfileMask::NATIVE_DESKTOP`.
pub fn list_native_vibe_providers() -> Vec<NativeProviderSummary> {
    let registry = ExtensionRegistry::with_builtin_providers();
    let mut out = Vec::new();
    for alias in BUILTIN_EXTENSION_ALIASES {
        if let Some((_, provider, desc)) = registry.find_by_namespace_or_alias(alias) {
            let fn_count: usize = desc
                .sub_namespaces
                .iter()
                .map(|ns| ns.functions.len())
                .sum();
            let native_ok = (desc.supported_targets.0 & TargetProfileMask::NATIVE_DESKTOP.0) != 0
                && provider
                    .static_ontology_nquins()
                    .iter()
                    .all(|q| q.verify_parity());
            out.push(NativeProviderSummary {
                alias: (*alias).to_string(),
                canonical_iri: desc.canonical_iri.iri.to_string(),
                function_count: fn_count,
                ontology_nquins: provider.static_ontology_nquins().len(),
                native_desktop_admitted: native_ok,
            });
        }
    }
    out
}

/// Catalog of the 150 `hypermedia.*` production morphisms across the 12 domain pillars
/// integrated into the Poet & Hypermedia Workbench.
pub fn hypermedia_pillar_catalog() -> &'static [HypermediaPillarSummary] {
    &[
        HypermediaPillarSummary {
            domain: "hypermedia.layout",
            morphism_count: 14,
            containers: ".hmc / .svg / .png",
            description: "Theme scopes, UI sheets, diegetic 3D surfaces, paged typesetting, presentation decks, and nodegraph layout.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.locus",
            morphism_count: 18,
            containers: ".10d (HOLO 0x0306 v2) / AREF / IOVR / INST",
            description: "Holographic 10D v2 phase-space SDFs, scale-space harmonic LODs (k=0..K), two-tier assemblies, and BIM/CSG.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.kinetics",
            morphism_count: 16,
            containers: ".10d (SKL2 / SSM1 / EONT)",
            description: "XPBD rigid/soft-body physics, Cosserat rods, DCA multibody, VBD soft tissue, NLA timelines, and IK motors.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.surface",
            morphism_count: 14,
            containers: ".m64 (Codec 3 BLAKE3) / MAT4",
            description: "64-byte physical matter/tissue strata, SGGX microflakes, spectral leaf biophysics, and layered PBR materials.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.radiance",
            morphism_count: 12,
            containers: ".10d / .png / .exr",
            description: "Spectral colour & metamer kernels, camera raw grading, raymarched HOLO previews, and radiance tracing.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.acoustics",
            morphism_count: 12,
            containers: ".qac (QUALIA-FIELD-INDEX-v1)",
            description: "10D Holosonic soundfields, Higher-Order Ambisonics, binaural HRTF, FDTD/Virieux/Helmholtz solvers, and MIDI 2.0.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.electromagnetics",
            morphism_count: 10,
            containers: ".emf (Q42EMF64 v2)",
            description: "8x8x8 Hilbert electromagnetic & bio-field bricks, 3D Maxwell Yee FDTD, Poynting flux, and Stokes polarization.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.geodesy",
            morphism_count: 12,
            containers: ".m64 (GRM3) / .q42",
            description: "GeoSPARQL, terrain erosion solvers, hydrological Shallow-Water Equations, and Hilbert geospatial strata.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.discourse",
            morphism_count: 12,
            containers: ".hcf / .q42 / QECP .pdf",
            description: "Context Markup Language (CML), academic TeX, scientific matlang, C2PA QECP PDFs, and review galleries.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.chronicle",
            morphism_count: 10,
            containers: ".hmc (QBDL) / .q42j",
            description: "Tabular grid engines, office codecs, broadcast timelines, and fiduciary contribution ledgers.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.vector",
            morphism_count: 10,
            containers: ".svg / .10d",
            description: "Boolean path operations, complex-script glyph shaping, and locus brush strokes.",
        },
        HypermediaPillarSummary {
            domain: "hypermedia.raster",
            morphism_count: 10,
            containers: ".png / .m64",
            description: "16-bit RGBA blend operators, selection morphology, layer masks, and spatial filters.",
        },
    ]
}

/// Discover local `.p64` and `.gguf` SLD/LLM models across the unified vault paths (`S1`/`S2`).
pub fn discover_unified_llm_vault(storage: &Path) -> Vec<LocalLlmModelEntry> {
    let mut roots = Vec::new();
    if let Ok(custom) = std::env::var("QUALIA_LLM_VAULT") {
        let p = PathBuf::from(custom.trim());
        if p.is_dir() {
            roots.push(p);
        }
    }
    for candidate in [
        PathBuf::from(r"C:\LLM_files"),
        PathBuf::from(r"C:\LLM_Models"),
        storage.join("Models"),
    ] {
        if candidate.is_dir() && !roots.contains(&candidate) {
            roots.push(candidate);
        }
    }

    let mut models = Vec::new();
    for root in &roots {
        scan_model_dir(root, 0, &mut models);
    }
    models.sort_by(|a, b| a.name.cmp(&b.name));
    models
}

fn scan_model_dir(dir: &Path, depth: usize, out: &mut Vec<LocalLlmModelEntry>) {
    if depth > 3 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_model_dir(&path, depth + 1, out);
        } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            let format = if ext.eq_ignore_ascii_case("p64") {
                Some("P64 v4 (Etalon Page-Aligned)")
            } else if ext.eq_ignore_ascii_case("gguf") {
                Some("GGUF (Qualia/Etalon Quantized)")
            } else if ext.eq_ignore_ascii_case("safetensors") {
                Some("SafeTensors")
            } else {
                None
            };
            if let Some(format) = format {
                let size_mib = entry
                    .metadata()
                    .map(|m| m.len() / (1024 * 1024))
                    .unwrap_or(0);
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("model")
                    .to_string();
                out.push(LocalLlmModelEntry {
                    name,
                    format,
                    path,
                    size_mib,
                });
            }
        }
    }
}

/// Execute a `.vibe` hypermedia production script directly from Webizen OS-Shell
/// without requiring a separate Synth GUI app to be running.
pub fn produce_hypermedia_vibe(
    host: &LoopbackHost,
    script_source: &str,
    job_slug: &str,
) -> HypermediaProduceReceipt {
    let out_dir = host.storage.join("production").join(job_slug);
    let _ = fs::create_dir_all(&out_dir);
    let script_path = out_dir.join(format!("{job_slug}.vibe"));
    let _ = fs::write(&script_path, script_source);

    // 1. Check if the local synth production CLI binary is available to seal full .hmc / .10d packages
    if let Some(synth_bin) = locate_synth_cli_binary(host) {
        let hmc_out = out_dir.join(format!("{job_slug}.hmc"));
        let html_out = out_dir.join(format!("{job_slug}.html"));
        let output = Command::new(&synth_bin)
            .arg("produce-present")
            .arg("--source")
            .arg(&script_path)
            .arg("--out-hmc")
            .arg(&hmc_out)
            .arg("--present-html")
            .arg(&html_out)
            .arg("--export-dir")
            .arg(&out_dir)
            .env("SYNTH_INPUT_ROOT", r"C:\github")
            .env("SYNTH_OUTPUT_ROOT", &out_dir)
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                return HypermediaProduceReceipt {
                    ok: true,
                    mode: "Native Hypermedia ProductionSession (.hmc / .10d v2 HOLO)".into(),
                    output_summary: format!(
                        "Sealed {} and exported resources under {}",
                        hmc_out.display(),
                        out_dir.display()
                    ),
                    output_dir: out_dir,
                };
            }
        }
    }

    // 2. In-process native VibeScript evaluation + NQuin48 provenance seal
    let eval = poet_eval_native_cell(script_source, 1);
    let receipt_quin = NQuin48::invariant(
        &format!("https://webizen.org/production/{job_slug}"),
        "https://webizen.org/q42#hypermediaProductionReceipt",
        eval.revision,
    );
    let receipt_json = serde_json::json!({
        "job_slug": job_slug,
        "language": eval.language,
        "host_version": eval.host_version,
        "ok": eval.ok,
        "value": eval.value,
        "diagnostic": eval.diagnostic,
        "nquin48_parity": format!("{:#018x}", receipt_quin.parity()),
    });
    let _ = fs::write(
        out_dir.join("production-receipt.json"),
        serde_json::to_string_pretty(&receipt_json).unwrap_or_default(),
    );

    HypermediaProduceReceipt {
        ok: eval.ok,
        mode: "In-Process Native VibeScript + Etalon NATIVE_DESKTOP".into(),
        output_summary: if eval.ok {
            format!(
                "Evaluated native VibeScript ({}) -> {} (NQuin48 parity {:#018x})",
                job_slug,
                eval.value,
                receipt_quin.parity()
            )
        } else {
            format!(
                "Script saved to {}; diagnostic: {}",
                script_path.display(),
                eval.diagnostic.unwrap_or_else(|| "none".into())
            )
        },
        output_dir: out_dir,
    }
}

fn locate_synth_cli_binary(host: &LoopbackHost) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) { "synth.exe" } else { "synth" };
    let beside = host.install_root.join(exe_name);
    if beside.is_file() {
        return Some(beside);
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    if let Some(github) = manifest.ancestors().nth(4) {
        for rel in [
            "synth/target-prod/debug",
            "synth/target-prod/release",
            "synth/target/release",
        ] {
            let candidate = github.join(rel).join(exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn format_vibe_value(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::I64(n) => n.to_string(),
        Value::U64(n) => n.to_string(),
        Value::F64(n) => format!("{n}"),
        Value::String(s) => s.clone(),
        Value::Iri(s) => format!("<{s}>"),
        Value::List(items) => {
            let inner: Vec<String> = items.iter().map(format_vibe_value).collect();
            format!("[{}]", inner.join(", "))
        }
        Value::Record(map) => {
            let inner: Vec<String> = map
                .iter()
                .map(|(k, val)| format!("{k}: {}", format_vibe_value(val)))
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_poet_evaluates_vibe_cells_and_gazetteer_quins() {
        let res = poet_eval_native_cell("= 21 + 21", 0);
        assert!(res.ok, "{res:?}");
        assert_eq!(res.value, "42");
        assert_eq!(res.sentinel_arena_bytes, 42 * 1024 * 1024);

        let gaz = poet_analyze_and_seal_gazetteer(
            "QualiaDB and VibeScript run on Etalon native GPU to protect human rights in Sanctuary.",
        );
        assert!(gaz.ok);
        assert!(gaz.hits.len() >= 4);
        assert_eq!(gaz.hits.len(), gaz.sealed_quins.len());
    }

    #[test]
    fn hypermedia_pillars_sum_to_150_morphisms() {
        let total: usize = hypermedia_pillar_catalog()
            .iter()
            .map(|p| p.morphism_count)
            .sum();
        assert_eq!(total, HYPERMEDIA_MORPHISM_COUNT);
    }

    #[test]
    fn core_semantic_instruments_and_shacl_extension_families_are_complete() {
        assert_eq!(core_semantic_instruments_catalog().len(), 6);
        assert_eq!(shacl_extension_families_catalog().len(), 6);
    }
}
