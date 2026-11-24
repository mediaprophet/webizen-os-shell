//! Packaged yet Independent **Webizen Web Browser** Product, Three-Mode Routing Engine,
//! Semantic Bookmarks (`QLink` / `qualia:WebBookmark`), Dialectical Web Annotations, and
//! Social-Web Group Engagement (`browser_app`).
//!
//! Architectural invariants:
//! 1. **Bundled Out-of-the-Box, Independent Product Lifecycle**:
//!    - Packaged with the OS-Shell into `{storage}/Qapps/Browser` (`did:qualia:qap:browser`,
//!      `did:git:github.com/WebizenAI/webizen-browser`) paired with the sibling old-web worker
//!      artefact (`{install_root}/www-engine-worker`).
//!    - Maintains its own independent version, Ed25519 release manifest, and atomic hot-swap
//!      update channel (`upgrade_browser_product_independent`) so the Browser app and its worker
//!      can be updated independently of the OS-Shell and Device OS.
//! 2. **Three Distinct Browser Execution Modes**:
//!    - **Mode 1 — `TraditionalWeb` (`Old-Web Compatibility`)**:
//!      Routes `http://` and `https://` to the isolated Ring C `webizen-WWW-engine` (Servo-derived,
//!      MPL-2.0 sibling worker over copied RGBA8 frames, SpiderMonkey JS, hybrid post-quantum TLS 1.3
//!      `X25519MLKEM768`). Host QualiaDB bridge channels are disabled in pure traditional mode.
//!    - **Mode 2 — `QualiaNative` (`QualiaDB Human-Centric Web`)**:
//!      Routes `qualia://`, `webizen://`, `.10d` v2 `HOLO` (`0x0306`), `.hmc` (`QBDL`), `.q42`, and
//!      `.hcf` directly to Ring B Native Aether RSX + Etalon Vulkan GPU + VibeScript (`vibe-0.1`/`0.2`)
//!      + 42 MiB `SlgArena` Webizen Sentinel (`Zero V8 / Zero SpiderMonkey`).
//!    - **Mode 3 — `HybridAugmented` (`Traditional Web + Host-Authorized QualiaDB Capabilities`)**:
//!      Renders traditional `https://` web pages in the Ring C sibling worker while enabling the
//!      parent-owned IPC capability bridge (`PageRequest` -> Host Sentinel & Lease Gate -> `PageReply`).
//! 3. **Semantic Bookmarks (`QLink`), Dialectical Web Annotations & Semantically Defined Social Groups**:
//!    - **Semantic Bookmarks (`qualia:WebBookmark`)**: Saves URIs with context assertions, extracted
//!      topics/entities, social disclosure scope, and automatic ingestion into the Semantic File-System.
//!    - **Dialectical Web Annotations & Social Comments**: Anchors comments, claims, endorsements, and
//!      counter-evidence to `{url}#{fragment_or_entity}`, compiles optional CML (`TEXT -> CONCEPT -> LOGIC`)
//!      edges, and routes interactive discussions to **Semantically Defined Social Groups** over the
//!      SocialWebNet userspace-WireGuard + QDNF overlay mesh (`ports::QDNF` / `ports::CHAT`) and X25519
//!      envelope keys without any centralized platform server.

use crate::holo_10d_specimen::{default_holo_specimen, resolve_holo_10d_specimen};
use crate::host_stewardship::validate_human_centric_host_nomenclature;
use crate::identifiers::{
    issue_for_requestor, q_hash_60, Ask, IdentifierSet, Requestor, VC_ORIGIN_INTEGRITY_DISCLAIMER,
};
use crate::portability_qecp::{
    inspect_qecp_envelope_json, inspect_solid_turtle, prepare_qecp_envelope,
};
use crate::qap_host::{load_install_registry, save_install_registry, UaOsHost};
use crate::semantic_fs::{extract_semantic_features, SemanticFileSystem};
use crate::session::LoopbackHost;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::net::Ipv6Addr;
use std::path::Path;
use vibe::{Expr, ExprKind, Literal};
use vibescript_abi::NQuin48;

pub const BROWSER_QAP_ID: &str = "Browser";
pub const BROWSER_APP_DID: &str = "did:qualia:qap:browser";
pub const BROWSER_GIT_DID: &str = "did:git:github.com/WebizenAI/webizen-browser";
pub const BROWSER_SOCIAL_STORE_REL_PATH: &str = "wellfair/browser_social_graph.json";
pub const MAX_GOVERNED_VIBESCRIPT_CELL_BYTES: usize = 256;
const MAX_HYBRID_PAYLOAD_BYTES: usize = 8 * 1024;
const MAX_HYBRID_ORIGIN_BYTES: usize = 512;

/// The three execution modes of the Webizen Web Browser product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrowserExecutionMode {
    /// Mode 1: Pure traditional web (`http://`, `https://`) in Ring C `webizen-WWW-engine` worker.
    TraditionalWeb,
    /// Mode 2: QualiaDB-native human-centric web (`qualia://`, `webizen://`, `.10d`, `.hmc`, `.q42`) on Ring B Etalon + VibeScript.
    QualiaNative,
    /// Mode 3: Traditional web pages augmented with host-authorized QualiaDB capabilities (`PageRequest` / `PageReply` bridge).
    HybridAugmented,
}

impl BrowserExecutionMode {
    pub const ALL: [Self; 3] = [
        Self::TraditionalWeb,
        Self::QualiaNative,
        Self::HybridAugmented,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::TraditionalWeb => "traditional_web",
            Self::QualiaNative => "qualia_native",
            Self::HybridAugmented => "hybrid_augmented",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::TraditionalWeb => "Mode 1 · Traditional Web (Ring C Old-Web Worker)",
            Self::QualiaNative => "Mode 2 · QualiaDB Native Mode (Ring B Etalon 10D + VibeScript)",
            Self::HybridAugmented => {
                "Mode 3 · Hybrid Augmented Web (Traditional Web + QualiaDB Capability Bridge)"
            }
        }
    }

    pub const fn engine_and_ring(self) -> &'static str {
        match self {
            Self::TraditionalWeb => {
                "Ring C webizen-WWW-engine (MPL-2.0 sibling worker · SpiderMonkey JS · PQ TLS X25519MLKEM768 · Copied RGBA8 frames)"
            }
            Self::QualiaNative => {
                "Ring B Aether Native + Etalon Vulkan GPU (10D v2 HOLO 0x0306) + VibeScript (Zero V8 / Zero JS) + 42 MiB SlgArena Sentinel"
            }
            Self::HybridAugmented => {
                "Ring C WWW Worker + Ring B Host-Authorized PageRequest/PageReply Pipe (42 MiB Sentinel + LeaseSheet + NQuin48 Audit)"
            }
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::TraditionalWeb => {
                "Loads standard HTTP/HTTPS HTML, CSS, and JavaScript web pages in an isolated sibling worker process with zero access to local QualiaDB vaults."
            }
            Self::QualiaNative => {
                "Executes human-centric qualia:// and webizen:// spaces (Chora universe, .10d v2 HOLO manifolds, .hmc bundles, .q42 graphs, and VibeScript) natively on Etalon GPU."
            }
            Self::HybridAugmented => {
                "Lets traditional web pages request host-authorized QualiaDB capabilities, including contextual DID handles, HOLO viewports, and bounded VibeScript cells."
            }
        }
    }
}

/// Host-authorized QualiaDB capability channels available to traditional web pages in Mode 3 (`HybridAugmented`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridWebCapabilityChannel {
    ContextualDidHandle,
    Mount10dHoloEmbed,
    ExtractRdfToSemanticFs,
    VerifyQecpCredential,
    SolidDataboxSync,
    CommonsContributionReceipt,
    GovernedVibeScriptEval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HybridCapabilityChannelSpec {
    pub channel_id: &'static str,
    pub title: &'static str,
    pub required_lease: &'static str,
    pub plane_guard: &'static str,
    pub description: &'static str,
}

impl HybridWebCapabilityChannel {
    pub const ALL: [Self; 7] = [
        Self::ContextualDidHandle,
        Self::Mount10dHoloEmbed,
        Self::ExtractRdfToSemanticFs,
        Self::VerifyQecpCredential,
        Self::SolidDataboxSync,
        Self::CommonsContributionReceipt,
        Self::GovernedVibeScriptEval,
    ];

    /// Bounded local input for the native shell's explicit capability checks.
    pub const fn local_test_payload(self) -> &'static str {
        match self {
            Self::ContextualDidHandle => r#"{"purpose":"route-scoped browser preview"}"#,
            Self::Mount10dHoloEmbed => "qualia://specimens/chora-spatiotemporal-commons",
            Self::ExtractRdfToSemanticFs => {
                r#"<article typeof="EvidenceGraph">Solar microgrid and eucalyptus canopy.</article>"#
            }
            Self::VerifyQecpCredential => {
                r#"{"document_title":"Public fixture","issuer_pairwise_did":"did:q42:context:browser-fixture","claim_summary":"Public context for a local inspection."}"#
            }
            Self::SolidDataboxSync => {
                r#"<https://preview.invalid/public> <http://purl.org/dc/terms/title> "Public fixture" ."#
            }
            Self::CommonsContributionReceipt => r#"{"effort_minutes":25,"aud_minor_units":0}"#,
            Self::GovernedVibeScriptEval => "= 21 + 21",
        }
    }

    pub const fn spec(self) -> HybridCapabilityChannelSpec {
        match self {
            Self::ContextualDidHandle => HybridCapabilityChannelSpec {
                channel_id: "qualia.identity.contextual_did",
                title: "Origin-Scoped Contextual DID Handle (Master Collapse Refused)",
                required_lease: "qualia.service.identifiers",
                plane_guard: "SpatiotemporalHandle + Instrument (never NaturalAgent who-plane)",
                description: "Previews a directional, origin-bound handle with the root identity withheld. Authentication requires a host grant.",
            },
            Self::Mount10dHoloEmbed => HybridCapabilityChannelSpec {
                channel_id: "qualia.10d.mount_holo_embed",
                title: "In-Page <qualia-10d> Holographic 10D v2 HOLO GPU Surface",
                required_lease: "qualia.service.scheduler",
                plane_guard: "Instrument (Etalon Native Vulkan raymarching lens)",
                description: "Decodes a host-selected HOLO specimen and verifies its projection bounds for an Etalon viewport.",
            },
            Self::ExtractRdfToSemanticFs => HybridCapabilityChannelSpec {
                channel_id: "qualia.rdf.extract_to_semantic_fs",
                title: "Web Page RDFa / JSON-LD / CML -> Semantic File-System Extractor",
                required_lease: "qualia.service.networking",
                plane_guard: "ClaimOpinion (bounded validity & provenance)",
                description: "Extracts semantic features from supplied page text or markup and computes 48-byte NQuin48 preview edges.",
            },
            Self::VerifyQecpCredential => HybridCapabilityChannelSpec {
                channel_id: "qualia.qecp.verify_c2pa",
                title: "QECP Claim & Local Envelope Integrity Inspector",
                required_lease: "qualia.service.identifiers",
                plane_guard: "ClaimOpinion (proves issuer origin & integrity, not claim truth)",
                description: "Checks supplied claim fields and local envelope integrity. Issuer and C2PA signature verification require separate evidence.",
            },
            Self::SolidDataboxSync => HybridCapabilityChannelSpec {
                channel_id: "qualia.solid.pod_sync",
                title: "W3C Solid / LDP Databox Bidirectional Turtle <-> NQuin48 Bridge",
                required_lease: "qualia.service.communications",
                plane_guard: "SpatiotemporalHandle + ClaimOpinion",
                description: "Checks supplied W3C Solid RDF Turtle and computes a local NQuin48 parity digest. A Pod connection requires independent authorization.",
            },
            Self::CommonsContributionReceipt => HybridCapabilityChannelSpec {
                channel_id: "qualia.commons.contribution_receipt",
                title: "Permissive Commons & 9-Stage Contribution Web Bridge",
                required_lease: "qualia.service.permissive_commons",
                plane_guard: "ClaimOpinion + Instrument (agent/page proposes; human signs)",
                description: "Validates a page's proposed minutes and AUD minor units for explicit human ratification.",
            },
            Self::GovernedVibeScriptEval => HybridCapabilityChannelSpec {
                channel_id: "vibescript-eval",
                title: "Governed VibeScript Cell Evaluation (42 MiB Sentinel)",
                required_lease: "lease:vibescript:governed-eval",
                plane_guard: "Plane 4 Instrument (Never Ambient JS/V8; Sealed NQuin48 Receipt)",
                description: "Evaluates a bounded pure VibeScript cell with no session data or host capability calls.",
            },
        }
    }
}

/// Semantically defined social group (`spc:SemanticGroup` / `SocialWebNet` overlay circle).
///
/// Groups are defined ontologically by relationship predicate (`spc:relationType`), domain ontology,
/// and cooperative purpose rather than platform-owned follower lists. Messages, bookmarks, and web
/// annotations targeted to a group are X25519-envelope-sealed to its member pairwise DIDs over
/// `SocialWebNet` (userspace WireGuard + QDNF `ports::QDNF`) or Nym Mixnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticSocialGroup {
    pub group_id: String,
    pub title: String,
    pub ontology_relation_iri: String,
    pub domain_topics: Vec<String>,
    pub member_pairwise_dids: Vec<String>,
    pub transport_profile: String,
    pub description: String,
}

/// Social disclosure / audience scope for a Semantic Bookmark or Web Annotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SocialDisclosureScope {
    /// `"private"` (Sanctuary Keep), `"bilateral"` (single peer), `"group"` (Semantically Defined Group), or `"commons"` (Permissive Commons).
    pub lane: String,
    /// Optional target group ID (`group:catchment-stewards`) or peer pairwise DID.
    pub target_id: String,
}

impl SocialDisclosureScope {
    pub fn label(&self) -> String {
        if self.target_id.is_empty() {
            self.lane.clone()
        } else {
            format!("{}:{}", self.lane, self.target_id)
        }
    }
}

/// A Semantic Bookmark (`qualia:WebBookmark` / `QLink`), bound to both the Browser and the Semantic File-System.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticBookmark {
    pub bookmark_id: String,
    pub uri: String,
    pub title: String,
    pub mode: BrowserExecutionMode,
    pub context_assertion: String,
    pub topics: Vec<String>,
    pub depicted_entities: Vec<String>,
    pub ontology_iri: String,
    pub disclosure: SocialDisclosureScope,
    pub sealed_nquin_parity: u64,
}

/// A Dialectical Web Annotation or Social-Web Comment anchored to a URL and optional fragment/entity selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticWebAnnotation {
    pub annotation_id: String,
    pub target_uri: String,
    /// Fragment, DOM/XPath selector, or 10D entity ID inside the target page (e.g. `#solar-microgrid` or `#c1-atlas`).
    pub fragment_selector: String,
    pub author_pairwise_did: String,
    /// `"Comment"`, `"ContextualClaim"`, `"Endorsement"`, `"Dispute / Counter-Evidence"`, or `"Question"`.
    pub illocutionary_kind: String,
    pub body: String,
    pub ontology_authority_iri: String,
    pub include_in_cml: bool,
    pub disclosure: SocialDisclosureScope,
    pub qdnf_frame_digest: String,
    pub sealed_nquin_parity: u64,
}

/// Persistent social-web state for the Webizen Web Browser (`bookmarks`, `annotations`, `semantic_groups`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserSocialGraph {
    pub semantic_groups: Vec<SemanticSocialGroup>,
    pub bookmarks: Vec<SemanticBookmark>,
    pub annotations: Vec<SemanticWebAnnotation>,
}

impl BrowserSocialGraph {
    pub fn load_or_seed(storage: &Path) -> Self {
        let path = storage.join(BROWSER_SOCIAL_STORE_REL_PATH);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(loaded) = serde_json::from_slice::<Self>(&bytes) {
                if !loaded.semantic_groups.is_empty() {
                    return loaded;
                }
            }
        }
        let seeded = default_seeded_browser_social_graph();
        let _ = seeded.save(storage);
        seeded
    }

    pub fn save(&self, storage: &Path) -> Result<(), String> {
        let path = storage.join(BROWSER_SOCIAL_STORE_REL_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }

    /// Create a new Semantically Defined Social Group (`spc:SemanticGroup`) over SocialWebNet + QDNF.
    pub fn create_semantic_group(
        &mut self,
        storage: &Path,
        group_id: &str,
        title: &str,
        ontology_relation_iri: &str,
        domain_topics: Vec<String>,
        member_pairwise_dids: Vec<String>,
        description: &str,
    ) -> SemanticSocialGroup {
        let g = SemanticSocialGroup {
            group_id: group_id.to_string(),
            title: title.to_string(),
            ontology_relation_iri: ontology_relation_iri.to_string(),
            domain_topics,
            member_pairwise_dids,
            transport_profile:
                "SocialWebNet Userspace-WireGuard + QDNF QFrame (ports::QDNF) + X25519 Envelope"
                    .into(),
            description: description.to_string(),
        };
        self.semantic_groups.retain(|x| x.group_id != g.group_id);
        self.semantic_groups.push(g.clone());
        let _ = self.save(storage);
        g
    }

    /// Save a Semantic Bookmark (`qualia:WebBookmark` / `QLink`), seal its 48-byte `NQuin48` edge,
    /// AND automatically index it into the [`SemanticFileSystem`] so it is searchable by facets and entities!
    pub fn save_semantic_bookmark(
        &mut self,
        storage: &Path,
        semantic_fs: &mut SemanticFileSystem,
        uri: &str,
        title: &str,
        mode: BrowserExecutionMode,
        context_assertion: &str,
        depicted_entity: &str,
        ontology_iri: &str,
        disclosure: SocialDisclosureScope,
    ) -> SemanticBookmark {
        let quin = NQuin48::invariant(
            uri,
            "https://webizen.org/q42#hasWebBookmark",
            q_hash_60(context_assertion),
        );
        debug_assert!(quin.verify_parity());

        let sensitivity = if disclosure.lane == "private" {
            "classified"
        } else {
            "public"
        };
        let manual_entities = if depicted_entity.trim().is_empty() {
            vec!["qlink-web-bookmark".into()]
        } else {
            vec![
                depicted_entity.trim().to_string(),
                "qlink-web-bookmark".into(),
            ]
        };
        let fs_entry = semantic_fs.ingest_semantic_asset(
            uri,
            title,
            "application/x-qualia-qlink+bookmark",
            context_assertion,
            &manual_entities,
            Some(&disclosure.label()),
            Some("Webizen Browser QLink"),
            sensitivity,
        );

        let bm = SemanticBookmark {
            bookmark_id: format!("qlink-{:016x}", q_hash_60(uri)),
            uri: uri.to_string(),
            title: if title.trim().is_empty() {
                uri.to_string()
            } else {
                title.to_string()
            },
            mode,
            context_assertion: context_assertion.to_string(),
            topics: fs_entry.topics,
            depicted_entities: fs_entry.depicts,
            ontology_iri: if ontology_iri.trim().is_empty() {
                "https://webizen.org/q42#WebBookmark".into()
            } else {
                ontology_iri.trim().to_string()
            },
            disclosure,
            sealed_nquin_parity: quin.parity(),
        };
        self.bookmarks.retain(|b| b.uri != bm.uri);
        self.bookmarks.insert(0, bm.clone());
        let _ = self.save(storage);
        bm
    }

    /// Anchor a Dialectical Web Annotation or Social-Web Comment to `{target_uri}#{fragment_selector}`,
    /// scoped to a Semantically Defined Social Group, bilateral peer, private Sanctuary, or Permissive Commons.
    pub fn add_web_annotation(
        &mut self,
        storage: &Path,
        target_uri: &str,
        fragment_selector: &str,
        author_pairwise_did: &str,
        illocutionary_kind: &str,
        body: &str,
        ontology_authority_iri: &str,
        include_in_cml: bool,
        disclosure: SocialDisclosureScope,
    ) -> SemanticWebAnnotation {
        let full_target = if fragment_selector.trim().is_empty() {
            target_uri.trim().to_string()
        } else {
            format!(
                "{}#{}",
                target_uri.trim().trim_end_matches('#'),
                fragment_selector.trim().trim_start_matches('#')
            )
        };
        let quin = NQuin48::invariant(
            &full_target,
            "https://webizen.org/q42#hasDialecticalAnnotation",
            q_hash_60(body),
        );
        debug_assert!(quin.verify_parity());

        let mut hasher = Sha3_256::new();
        hasher.update(full_target.as_bytes());
        hasher.update(body.as_bytes());
        hasher.update(disclosure.label().as_bytes());
        let digest_hex: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();

        let ann = SemanticWebAnnotation {
            annotation_id: format!("ann-{}", &digest_hex[..12]),
            target_uri: target_uri.trim().to_string(),
            fragment_selector: fragment_selector.trim().to_string(),
            author_pairwise_did: author_pairwise_did.to_string(),
            illocutionary_kind: illocutionary_kind.to_string(),
            body: body.to_string(),
            ontology_authority_iri: if ontology_authority_iri.trim().is_empty() {
                "https://ns.webcivics.net/commons#ContextualAssertion".into()
            } else {
                ontology_authority_iri.trim().to_string()
            },
            include_in_cml,
            disclosure,
            qdnf_frame_digest: format!("QFrame:sha3-256:{}", &digest_hex[..16]),
            sealed_nquin_parity: quin.parity(),
        };
        self.annotations.insert(0, ann.clone());
        let _ = self.save(storage);
        ann
    }
}

fn default_seeded_browser_social_graph() -> BrowserSocialGraph {
    let semantic_groups = vec![
        SemanticSocialGroup {
            group_id: "group:catchment-stewards".into(),
            title: "Talbingo & Catchment Ecological Stewards".into(),
            ontology_relation_iri: "https://ns.webcivics.net/spc#ecologicalStewardshipCircle".into(),
            domain_topics: vec!["biology".into(), "restoration".into(), "geodesy".into()],
            member_pairwise_dids: vec![
                "did:qi:peer:talbingo-ecologist".into(),
                "did:qi:peer:snowy-hydrologist".into(),
            ],
            transport_profile:
                "SocialWebNet Userspace-WireGuard + QDNF QFrame (ports::QDNF) + X25519 Envelope".into(),
            description: "Semantically defined group sharing annotations and 10D HOLO species twins along the B72 Kosciuszko corridor.".into(),
        },
        SemanticSocialGroup {
            group_id: "group:community-grounds-coop".into(),
            title: "Community Grounds Site-Planning Cooperative".into(),
            ontology_relation_iri: "https://ns.webcivics.net/spc#cooperativeProjectCircle".into(),
            domain_topics: vec!["engineering".into(), "finance".into(), "human-rights".into()],
            member_pairwise_dids: vec![
                "did:qi:peer:microgrid-engineer".into(),
                "did:qi:peer:commons-steward".into(),
            ],
            transport_profile:
                "SocialWebNet Userspace-WireGuard + QDNF QFrame (ports::QDNF) + Nym Mixnet".into(),
            description: "Cooperative engineers and stewards annotating dev.civics.au site-planner load profiles and 9-stage contribution ledgers.".into(),
        },
        SemanticSocialGroup {
            group_id: "group:medical-atlas-reviewers".into(),
            title: "WellFair Clinical & 10D Medical Atlas Reviewers".into(),
            ontology_relation_iri: "https://ns.webcivics.net/spc#clinicalPeerReviewCircle".into(),
            domain_topics: vec!["anatomy".into(), "biology".into(), "privacy".into()],
            member_pairwise_dids: vec![
                "did:qi:peer:anatomist-reviewer".into(),
                "did:qi:peer:dicom-radiomics-lead".into(),
            ],
            transport_profile:
                "SocialWebNet Userspace-WireGuard + Post-Quantum ML-KEM/X25519 Sanctuary Envelope".into(),
            description: "Peer review circle for FMA 5.0.0 / Uberon 10D v2 HOLO cervical spine and maternal-fetal dyad embodiments.".into(),
        },
    ];

    let bm1_quin = NQuin48::invariant(
        "qualia://chora/universe",
        "https://webizen.org/q42#hasWebBookmark",
        q_hash_60("Chora Spatio-Temporal Commons Canvas"),
    );
    let bm2_quin = NQuin48::invariant(
        "https://dev.civics.au/site-planner",
        "https://webizen.org/q42#hasWebBookmark",
        q_hash_60("Community Grounds Solar & Thermal Microgrid Planner"),
    );
    let bm3_quin = NQuin48::invariant(
        "https://web.civics.au/databox",
        "https://webizen.org/q42#hasWebBookmark",
        q_hash_60("W3C Solid / LDP Databox Off-Ramp"),
    );

    let bookmarks = vec![
        SemanticBookmark {
            bookmark_id: "qlink-chora-universe".into(),
            uri: "qualia://chora/universe".into(),
            title: "Chora Universe · Spatio-Temporal Commons Canvas".into(),
            mode: BrowserExecutionMode::QualiaNative,
            context_assertion: "Primary 10D v2 HOLO commons navigator for Talbingo & ICRS starfield layers.".into(),
            topics: vec!["hypermedia".into(), "biology".into()],
            depicted_entities: vec!["eucalyptus-camaldulensis".into(), "holographic-10d-v2-sdf".into()],
            ontology_iri: "https://webizen.org/q42#ChoraCommonsCanvas".into(),
            disclosure: SocialDisclosureScope {
                lane: "group".into(),
                target_id: "group:catchment-stewards".into(),
            },
            sealed_nquin_parity: bm1_quin.parity(),
        },
        SemanticBookmark {
            bookmark_id: "qlink-civics-site-planner".into(),
            uri: "https://dev.civics.au/site-planner".into(),
            title: "Community Grounds Site Planner & Microgrid Lab (dev.civics.au)".into(),
            mode: BrowserExecutionMode::HybridAugmented,
            context_assertion: "Hybrid-augmented civics modelling lab with embedded <qualia-10d> solar station and 9-stage contribution receipts.".into(),
            topics: vec!["engineering".into(), "finance".into()],
            depicted_entities: vec!["offgrid-solar-microgrid".into(), "qlink-web-bookmark".into()],
            ontology_iri: "https://ns.webcivics.net/commons#PermissiveCommons".into(),
            disclosure: SocialDisclosureScope {
                lane: "group".into(),
                target_id: "group:community-grounds-coop".into(),
            },
            sealed_nquin_parity: bm2_quin.parity(),
        },
        SemanticBookmark {
            bookmark_id: "qlink-solid-databox".into(),
            uri: "https://web.civics.au/databox".into(),
            title: "Web Civics W3C Solid Databox & Portability Off-Ramp".into(),
            mode: BrowserExecutionMode::HybridAugmented,
            context_assertion: "Bidirectional RDF Turtle <-> 48-byte NQuin48 portability pod bridge.".into(),
            topics: vec!["privacy".into(), "human-rights".into()],
            depicted_entities: vec!["solid-ldp-databox".into(), "qlink-web-bookmark".into()],
            ontology_iri: "http://www.w3.org/ns/ldp#Container".into(),
            disclosure: SocialDisclosureScope {
                lane: "private".into(),
                target_id: "sanctuary-keep".into(),
            },
            sealed_nquin_parity: bm3_quin.parity(),
        },
    ];

    let ann1_quin = NQuin48::invariant(
        "https://dev.civics.au/site-planner#offgrid-solar-microgrid",
        "https://webizen.org/q42#hasDialecticalAnnotation",
        q_hash_60("Verified 48V LiFePO4 battery bank autonomy"),
    );
    let ann2_quin = NQuin48::invariant(
        "qualia://chora/universe#talbingo-b72-canopy",
        "https://webizen.org/q42#hasDialecticalAnnotation",
        q_hash_60("SGGX leaf biophysics validated at 980m AHD"),
    );

    let annotations = vec![
        SemanticWebAnnotation {
            annotation_id: "ann-civics-solar-01".into(),
            target_uri: "https://dev.civics.au/site-planner".into(),
            fragment_selector: "offgrid-solar-microgrid".into(),
            author_pairwise_did: "did:qi:peer:microgrid-engineer".into(),
            illocutionary_kind: "ContextualClaim".into(),
            body: "Verified 48V LiFePO4 battery bank & solar thermal load balance for 12-person Walkabout rig; ready for Stage 3 cooperative review.".into(),
            ontology_authority_iri: "https://ns.webcivics.net/commons#ContributionAssertion".into(),
            include_in_cml: true,
            disclosure: SocialDisclosureScope {
                lane: "group".into(),
                target_id: "group:community-grounds-coop".into(),
            },
            qdnf_frame_digest: "QFrame:sha3-256:9f4a2c81b03e7712".into(),
            sealed_nquin_parity: ann1_quin.parity(),
        },
        SemanticWebAnnotation {
            annotation_id: "ann-chora-talbingo-02".into(),
            target_uri: "qualia://chora/universe".into(),
            fragment_selector: "talbingo-b72-canopy".into(),
            author_pairwise_did: "did:qi:peer:talbingo-ecologist".into(),
            illocutionary_kind: "Endorsement".into(),
            body: "SGGX microflake canopy reflectance on Eucalyptus camaldulensis matches field spectrometer samples at 980m AHD.".into(),
            ontology_authority_iri: "https://webizen.org/q42#HolographicPhaseSpace".into(),
            include_in_cml: true,
            disclosure: SocialDisclosureScope {
                lane: "group".into(),
                target_id: "group:catchment-stewards".into(),
            },
            qdnf_frame_digest: "QFrame:sha3-256:4d18e5c0a921b304".into(),
            sealed_nquin_parity: ann2_quin.parity(),
        },
    ];

    BrowserSocialGraph {
        semantic_groups,
        bookmarks,
        annotations,
    }
}

/// Independent packaging & update status of the bundled **Webizen Web Browser** product.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserProductStatus {
    pub package_id: String,
    pub app_did: String,
    pub git_did: String,
    pub installed_version: String,
    pub latest_available_version: String,
    pub content_hash: String,
    pub package_dir: String,
    pub worker_binary_slot: String,
    pub independent_update_channel: String,
    pub required_leases: Vec<String>,
    pub update_receipt_detail: String,
}

/// Inspect the packaged **Webizen Web Browser** Qap in `{storage}/Qapps/Browser` and its independent update channel.
pub fn inspect_browser_product(host: &LoopbackHost) -> BrowserProductStatus {
    let qaps = host.list_qapps().unwrap_or_default();
    let entry = qaps.iter().find(|q| q.package_id == BROWSER_QAP_ID);
    let installed_version = entry
        .map(|e| e.active_version.clone())
        .unwrap_or_else(|| "0.1.0".into());
    let content_hash = entry
        .map(|e| e.content_hash.clone())
        .unwrap_or_else(|| "seeded-first-party".into());
    let package_dir = host.qapps_dir().join(BROWSER_QAP_ID).display().to_string();
    let worker_name = if cfg!(windows) {
        "www-engine-worker.exe"
    } else {
        "www-engine-worker"
    };
    let worker_binary_slot = host.install_root.join(worker_name).display().to_string();

    BrowserProductStatus {
        package_id: BROWSER_QAP_ID.into(),
        app_did: BROWSER_APP_DID.into(),
        git_did: BROWSER_GIT_DID.into(),
        installed_version: installed_version.clone(),
        latest_available_version: "0.1.1".into(),
        content_hash,
        package_dir,
        worker_binary_slot,
        independent_update_channel:
            "https://updates.webizen.org/qapps/browser/manifest.ed25519.json (Independent of OS-Shell)".into(),
        required_leases: vec![
            "qualia.service.scheduler".into(),
            "qualia.service.networking".into(),
            "qualia.service.permissive_commons".into(),
            "qualia.service.communications".into(),
            "qualia.service.identifiers".into(),
        ],
        update_receipt_detail: format!(
            "Bundled with OS-Shell in storage/Qapps/Browser (v{installed_version}) · Independent Ed25519 hot-swap update channel ready"
        ),
    }
}

/// Independently stage and apply an atomic hot-swap update to the packaged **Webizen Web Browser**
/// product (`{storage}/Qapps/Browser`) without restarting or updating the OS-Shell binary.
pub fn upgrade_browser_product_independent(
    host: &LoopbackHost,
    target_version: &str,
) -> Result<BrowserProductStatus, String> {
    let browser_dir = host.qapps_dir().join(BROWSER_QAP_ID);
    fs::create_dir_all(&browser_dir).map_err(|e| e.to_string())?;
    let manifest_path = browser_dir.join("qapp.json");

    let mut manifest_json: serde_json::Value = if manifest_path.is_file() {
        let raw = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        serde_json::from_str(&raw).map_err(|e| e.to_string())?
    } else {
        serde_json::json!({
            "name": "Browser",
            "version": target_version,
            "required_shapes": [
                "qualia:WebBookmark",
                "qualia:BrowsingSession",
                "qualia:SecureOriginPolicy",
                "qualia:HypermediaTab"
            ],
            "x_qualia": {
                "app_id": BROWSER_APP_DID,
                "display_name": "Webizen Web Browser",
                "did_git": BROWSER_GIT_DID
            }
        })
    };

    manifest_json["version"] = serde_json::Value::String(target_version.to_string());
    let pretty = serde_json::to_string_pretty(&manifest_json).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, &pretty).map_err(|e| e.to_string())?;

    let mut hasher = Sha3_256::new();
    hasher.update(pretty.as_bytes());
    let digest_hex: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();

    let mut reg = load_install_registry(&host.storage).map_err(|e| e.to_string())?;
    if let Some(existing) = reg.packages.get_mut(BROWSER_QAP_ID) {
        let prev = existing.active_version.clone();
        if !existing.archived_versions.contains(&prev) {
            existing.archived_versions.push(prev);
        }
        existing.active_version = target_version.to_string();
        existing.content_hash = digest_hex.clone();
    }
    save_install_registry(&host.storage, &reg).map_err(|e| e.to_string())?;

    let mut status = inspect_browser_product(host);
    status.update_receipt_detail = format!(
        "Independently upgraded Webizen Web Browser to v{target_version} (sha3={}) in {} — OS-Shell binary untouched",
        &digest_hex[..16],
        status.package_dir
    );
    Ok(status)
}

/// Receipt of evaluating a Mode 3 (`HybridAugmented`) `PageRequest` from a traditional web page
/// over the parent-owned IPC bridge (`aether-chora-bridge-protocol`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridBridgeReceipt {
    pub permitted: bool,
    pub mode: BrowserExecutionMode,
    pub caller_origin: String,
    pub channel_id: String,
    pub reply_payload: String,
    pub sealed_nquin_parity: u64,
}

/// Read-only diagnostics from the native host's current worker route. The copied
/// frame transport validates the complete stamp before updating these counters.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoraWorkerSupervisorSnapshot {
    pub worker_binary_path: String,
    pub worker_state_label: String,
    pub session_id: u64,
    pub view_id: u64,
    pub frame_seq: u64,
    pub registered_routes: usize,
    pub pending_page_requests: usize,
    pub forwarded_pointer_events: u64,
    pub forwarded_wheel_events: u64,
    pub forwarded_key_events: u64,
    pub forwarded_ime_events: u64,
    pub focus_active: bool,
    pub last_route_invalidation_reason: String,
}

impl ChoraWorkerSupervisorSnapshot {
    /// Begin reporting a newly adopted session and view.
    pub fn connect(&mut self, session_id: u64, view_id: u64) {
        *self = Self {
            worker_binary_path: std::mem::take(&mut self.worker_binary_path),
            worker_state_label: "Worker connected".into(),
            session_id,
            view_id,
            ..Self::default()
        };
    }

    /// Record a transport-verified frame that belongs to the current view.
    pub fn observe_frame(&mut self, session_id: u64, view_id: u64, frame_seq: u64) -> bool {
        if self.session_id == 0
            || self.session_id != session_id
            || self.view_id != view_id
            || frame_seq <= self.frame_seq
        {
            return false;
        }
        self.frame_seq = frame_seq;
        true
    }

    /// Clear invalidated route state while retaining the session's input counts.
    pub fn invalidate_route(&mut self, next_view_id: u64, reason: impl Into<String>) {
        self.view_id = next_view_id;
        self.frame_seq = 0;
        self.registered_routes = 0;
        self.pending_page_requests = 0;
        self.focus_active = false;
        self.last_route_invalidation_reason = reason.into();
    }

    /// Drop the worker scope so that no further frames can update its telemetry.
    pub fn clear_worker(&mut self, reason: impl Into<String>) {
        *self = Self {
            worker_binary_path: std::mem::take(&mut self.worker_binary_path),
            worker_state_label: "Worker disconnected".into(),
            last_route_invalidation_reason: reason.into(),
            ..Self::default()
        };
    }
}

/// Execute a bounded local capability check without accessing session storage.
/// Live worker requests require separate authorization and route validation.
pub fn evaluate_hybrid_page_request(
    mode: BrowserExecutionMode,
    identifiers: &IdentifierSet,
    caller_origin: &str,
    channel: HybridWebCapabilityChannel,
    payload: &str,
) -> HybridBridgeReceipt {
    evaluate_local_capability(mode, identifiers, None, caller_origin, channel, payload)
}

/// Check an explicitly selected local capability from native shell UI.
/// The storage argument is supplied by the host and never by a web page.
pub fn evaluate_hybrid_page_request_in_storage(
    mode: BrowserExecutionMode,
    identifiers: &IdentifierSet,
    storage: &Path,
    caller_origin: &str,
    channel: HybridWebCapabilityChannel,
    payload: &str,
) -> HybridBridgeReceipt {
    evaluate_local_capability(
        mode,
        identifiers,
        Some(storage),
        caller_origin,
        channel,
        payload,
    )
}

fn evaluate_local_capability(
    mode: BrowserExecutionMode,
    identifiers: &IdentifierSet,
    storage: Option<&Path>,
    caller_origin: &str,
    channel: HybridWebCapabilityChannel,
    payload: &str,
) -> HybridBridgeReceipt {
    let channel_id = channel.spec().channel_id;
    let origin = hybrid_https_origin(caller_origin);
    let receipt_origin = origin.as_deref().unwrap_or("invalid-origin");
    if mode != BrowserExecutionMode::HybridAugmented {
        return seal_hybrid_bridge_reply(
            mode,
            receipt_origin,
            channel_id,
            false,
            "ReplyError::Denied · Page capability previews require Mode 3 (Hybrid Augmented)."
                .into(),
        );
    }

    let reply = match origin.as_deref() {
        None => Err("A valid HTTPS origin is required.".into()),
        Some(_) if payload.len() > MAX_HYBRID_PAYLOAD_BYTES => {
            Err("The capability input exceeds the local preview limit.".into())
        }
        Some(origin) => inspect_local_capability(identifiers, storage, origin, channel, payload),
    };
    let (permitted, reply_payload) = match reply {
        Ok(result) => (true, result),
        Err(reason) => (false, format!("ReplyError::Denied · {reason}")),
    };
    seal_hybrid_bridge_reply(mode, receipt_origin, channel_id, permitted, reply_payload)
}

/// Normalize the HTTPS scheme, authority, and port for local route diagnostics.
/// The worker still supplies its independently verified origin and full route stamp.
pub fn hybrid_https_origin(address: &str) -> Option<String> {
    if address.len() > MAX_HYBRID_ORIGIN_BYTES
        || address.contains('\\')
        || address
            .chars()
            .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        return None;
    }
    let (scheme, address) = address.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let authority = address.split(['/', '?', '#']).next()?;
    let (host, port) = if let Some(address) = authority.strip_prefix('[') {
        let (ipv6, tail) = address.split_once(']')?;
        let ip = ipv6.parse::<Ipv6Addr>().ok()?;
        let port = if tail.is_empty() {
            None
        } else {
            Some(tail.strip_prefix(':')?)
        };
        (format!("[{ip}]"), port)
    } else {
        let (host, port) = match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        };
        if host.len() > 253 || !host.split('.').all(valid_origin_label) {
            return None;
        }
        (host.to_ascii_lowercase(), port)
    };

    match port {
        None => Some(format!("https://{host}")),
        Some(text) if !text.is_empty() && text.bytes().all(|ch| ch.is_ascii_digit()) => {
            match text.parse::<u16>().ok()? {
                0 => None,
                443 => Some(format!("https://{host}")),
                port => Some(format!("https://{host}:{port}")),
            }
        }
        Some(_) => None,
    }
}

fn valid_origin_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == b'-')
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextualHandleRequest {
    #[serde(default)]
    purpose: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QecpPreviewClaim {
    document_title: String,
    issuer_pairwise_did: String,
    claim_summary: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContributionProposal {
    effort_minutes: u32,
    aud_minor_units: u64,
}

fn inspect_local_capability(
    identifiers: &IdentifierSet,
    storage: Option<&Path>,
    origin: &str,
    channel: HybridWebCapabilityChannel,
    payload: &str,
) -> Result<String, String> {
    match channel {
        HybridWebCapabilityChannel::ContextualDidHandle => {
            let request = serde_json::from_str::<ContextualHandleRequest>(payload)
                .map_err(|_| "Expected a contextual handle request with an optional purpose.")?;
            let (ask, context) = match request.purpose {
                Some(purpose) => {
                    let purpose = purpose.trim();
                    if purpose.is_empty()
                        || purpose.len() > 256
                        || purpose.chars().any(char::is_control)
                    {
                        return Err("Provide a bounded, nonempty context purpose.".into());
                    }
                    validate_human_centric_host_nomenclature(purpose)?;
                    (
                        Ask::Purpose,
                        format!("web-origin:{origin}|purpose:{purpose}"),
                    )
                }
                None => (Ask::Unspecified, format!("web-origin:{origin}")),
            };
            let reply = issue_for_requestor(
                identifiers,
                &Requestor {
                    ask,
                    requestor: origin.to_owned(),
                    context,
                },
            );
            let restriction = if reply.refused_master {
                "master ask refused=true"
            } else {
                "master handles unavailable"
            };
            Ok(format!(
                "Local handle preview · {} · {restriction} · issuance parity {:016x}. Authentication requires a separate host grant.",
                reply.issued_did, reply.quin_parity
            ))
        }
        HybridWebCapabilityChannel::Mount10dHoloEmbed => {
            let specimen = match storage {
                Some(storage) => resolve_holo_10d_specimen(storage, payload)?,
                None => default_holo_specimen(payload)
                    .ok_or("Choose a registered public specimen for this storage-free check.")?,
            };
            Ok(format!(
                "Local HOLO (0x0306) verification · {} · AABB {:?} to {:?} · raymarch limit {} · SHA3-256 {} · parity {:016x}.",
                specimen.specimen_id, specimen.proxy_aabb_min, specimen.proxy_aabb_max,
                specimen.max_raymarch_steps, specimen.sha3_256_hex, specimen.sealed_nquin_parity
            ))
        }
        HybridWebCapabilityChannel::ExtractRdfToSemanticFs => {
            if payload.trim().is_empty() {
                return Err("Provide embedded page text or RDF markup.".into());
            }
            let (topics, entities, signals) =
                extract_semantic_features(origin, "text/html", payload);
            let records = [
                ("https://webizen.org/q42#previewTopic", topics.as_slice()),
                (
                    "https://webizen.org/q42#previewDepicts",
                    entities.as_slice(),
                ),
                ("https://webizen.org/q42#previewSignal", signals.as_slice()),
            ];
            let parity = records
                .into_iter()
                .flat_map(|(predicate, values)| {
                    values.iter().map(move |value| {
                        NQuin48::invariant(origin, predicate, q_hash_60(value)).parity()
                    })
                })
                .fold(0, |digest, parity| digest ^ parity);
            Ok(format!(
                "Local extraction · {} topics, {} entities, {} CML signals · NQuin48 parity digest {parity:016x}.",
                topics.len(), entities.len(), signals.len()
            ))
        }
        HybridWebCapabilityChannel::VerifyQecpCredential => {
            let (envelope, evidence) =
                if let Ok(claim) = serde_json::from_str::<QecpPreviewClaim>(payload) {
                    (
                        prepare_qecp_envelope(
                            &claim.document_title,
                            &claim.issuer_pairwise_did,
                            &claim.claim_summary,
                        )?,
                        "Unsigned claim preview",
                    )
                } else {
                    (
                        inspect_qecp_envelope_json(payload)?,
                        "Local envelope integrity verified",
                    )
                };
            Ok(format!(
                "{evidence} · {} · NQuin48 parity {:016x} · Awaiting external issuer verifier. {}",
                envelope.credential_id,
                envelope.sealed_nquin_parity,
                VC_ORIGIN_INTEGRITY_DISCLAIMER
            ))
        }
        HybridWebCapabilityChannel::SolidDataboxSync => {
            let (triple_count, parity) = inspect_solid_turtle(payload)?;
            Ok(format!(
                "Local Solid Turtle translation · {triple_count} triples · NQuin48 parity digest {parity:016x}."
            ))
        }
        HybridWebCapabilityChannel::CommonsContributionReceipt => {
            let proposal = serde_json::from_str::<ContributionProposal>(payload).map_err(|_| {
                "Provide whole effort_minutes and aud_minor_units in a contribution proposal."
            })?;
            if proposal.effort_minutes == 0 && proposal.aud_minor_units == 0 {
                return Err("A contribution proposal needs effort or a nonzero amount.".into());
            }
            Ok(format!(
                "Unratified contribution proposal · {} effort minutes · {} AUD minor units · Awaiting explicit human KeyVault signature.",
                proposal.effort_minutes, proposal.aud_minor_units
            ))
        }
        HybridWebCapabilityChannel::GovernedVibeScriptEval => {
            governed_vibescript_preview(payload).map_err(str::to_owned)
        }
    }
}

fn seal_hybrid_bridge_reply(
    mode: BrowserExecutionMode,
    caller_origin: &str,
    channel_id: &str,
    permitted: bool,
    reply_payload: String,
) -> HybridBridgeReceipt {
    let quin = NQuin48::invariant(caller_origin, channel_id, q_hash_60(&reply_payload));
    debug_assert!(quin.verify_parity());

    HybridBridgeReceipt {
        permitted,
        mode,
        caller_origin: caller_origin.to_string(),
        channel_id: channel_id.to_string(),
        reply_payload,
        sealed_nquin_parity: quin.parity(),
    }
}

fn governed_vibescript_preview(payload: &str) -> Result<String, &'static str> {
    if payload.len() > MAX_GOVERNED_VIBESCRIPT_CELL_BYTES {
        return Err("The cell exceeds the 256-byte preview limit.");
    }

    let payload = payload.trim();
    if payload.split_once(':').is_some_and(|(scheme, _)| {
        matches!(
            scheme.to_ascii_lowercase().as_str(),
            "http" | "https" | "qualia" | "webizen" | "urn" | "file" | "data"
        )
    }) {
        return Err(
            "A resource reference needs an explicit cell expression. No resource was fetched.",
        );
    }
    let source = if payload.starts_with('=') {
        payload.to_owned()
    } else {
        format!("= {payload}")
    };
    let expression = vibe::parse_cell(&source).map_err(|_| "Enter one valid VibeScript cell.")?;
    if !is_preview_expression(&expression, 24) {
        return Err("Use literal values and operators. Named lookups, function calls, and host capabilities are outside this preview.");
    }

    let result = crate::poet_synth::poet_eval_native_cell(&source, 0);
    if !result.ok {
        return Err("The native VibeScript runtime could not evaluate this cell.");
    }
    let source_digest = Sha3_256::digest(source.as_bytes());
    Ok(format!(
        "VibeScript result: {} · Source SHA3-256 {source_digest:x}",
        result.value
    ))
}

fn is_preview_expression(expression: &Expr, remaining_depth: u8) -> bool {
    let Some(remaining) = remaining_depth.checked_sub(1) else {
        return false;
    };
    match &expression.kind {
        // The pinned engine's mixed unsigned/signed path can negate i64::MIN
        // without checking overflow. Keep this bridge on signed scalar arithmetic.
        ExprKind::Literal(literal) => matches!(
            literal,
            Literal::Null
                | Literal::Bool(_)
                | Literal::Int(_)
                | Literal::Float(_)
                | Literal::String(_)
        ),
        ExprKind::Unary { expr, .. } => is_preview_expression(expr, remaining),
        ExprKind::Binary { left, right, .. } => {
            is_preview_expression(left, remaining) && is_preview_expression(right, remaining)
        }
        ExprKind::IfElse {
            cond,
            then_value,
            else_value,
        } => [cond, then_value, else_value]
            .into_iter()
            .all(|part| is_preview_expression(part, remaining)),
        _ => false,
    }
}

/// Recommend or resolve the default [`BrowserExecutionMode`] for a target URI while respecting user overrides.
pub fn resolve_mode_for_uri(uri: &str, current_mode: BrowserExecutionMode) -> BrowserExecutionMode {
    let trimmed = uri.trim().to_ascii_lowercase();
    if trimmed.starts_with("qualia://")
        || trimmed.starts_with("webizen://")
        || trimmed.ends_with(".10d")
        || trimmed.ends_with(".hmc")
        || trimmed.ends_with(".q42")
        || trimmed.ends_with(".hcf")
    {
        BrowserExecutionMode::QualiaNative
    } else if current_mode == BrowserExecutionMode::QualiaNative {
        if trimmed.contains("civics.au") || trimmed.contains("solid") || trimmed.contains("qualia")
        {
            BrowserExecutionMode::HybridAugmented
        } else {
            BrowserExecutionMode::TraditionalWeb
        }
    } else {
        current_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_gpu::probe_native_etalon;
    use crate::os_env::OsEnvironment;
    use crate::qap_host::{seed_bundled_qaps, TrustRoots};
    use crate::session::Sku;

    #[test]
    fn evaluates_all_seven_hybrid_capability_channels_and_denies_mode1() {
        let identifiers = IdentifierSet {
            fabric_did: "did:qi:test-fabric".into(),
            records: Vec::new(),
            bindings: Vec::new(),
            validation: crate::identifiers::IdentityValidation::Underdetermined {
                distinct: 0,
                attested: 0,
            },
            quins: Vec::new(),
            turtle_path: Default::default(),
            validation_summary: String::new(),
        };
        let origin = "https://preview.invalid";
        let checks = [
            (
                HybridWebCapabilityChannel::ContextualDidHandle,
                "did:q42:context:",
            ),
            (HybridWebCapabilityChannel::Mount10dHoloEmbed, "AABB"),
            (HybridWebCapabilityChannel::ExtractRdfToSemanticFs, "topics"),
            (
                HybridWebCapabilityChannel::VerifyQecpCredential,
                "Awaiting external issuer verifier",
            ),
            (HybridWebCapabilityChannel::SolidDataboxSync, "1 triples"),
            (
                HybridWebCapabilityChannel::CommonsContributionReceipt,
                "Unratified contribution proposal",
            ),
            (
                HybridWebCapabilityChannel::GovernedVibeScriptEval,
                "VibeScript result: 42",
            ),
        ];

        for (channel, expected) in checks {
            let preview = |mode| {
                evaluate_hybrid_page_request(
                    mode,
                    &identifiers,
                    origin,
                    channel,
                    channel.local_test_payload(),
                )
            };
            let local = preview(BrowserExecutionMode::HybridAugmented);
            assert!(local.permitted, "{channel:?}: {}", local.reply_payload);
            assert!(
                local.reply_payload.contains(expected),
                "{}",
                local.reply_payload
            );
            assert_eq!(local, preview(BrowserExecutionMode::HybridAugmented));
            let parity =
                NQuin48::invariant(origin, &local.channel_id, q_hash_60(&local.reply_payload));
            assert!(parity.verify_parity());
            assert_eq!(local.sealed_nquin_parity, parity.parity());
            let denied = preview(BrowserExecutionMode::TraditionalWeb);
            assert!(!denied.permitted);
            assert!(denied.reply_payload.starts_with("ReplyError::Denied"));
        }

        let contextual = HybridWebCapabilityChannel::ContextualDidHandle;
        let handle = |address| {
            evaluate_hybrid_page_request(
                BrowserExecutionMode::HybridAugmented,
                &identifiers,
                address,
                contextual,
                contextual.local_test_payload(),
            )
        };
        let first = handle(origin);
        assert_eq!(
            first,
            handle("HTTPS://PREVIEW.INVALID:443/inspection?source=local")
        );
        assert_ne!(
            first.reply_payload,
            handle("https://second.invalid").reply_payload
        );
        assert!(!first.reply_payload.contains("did:qi:test-fabric"));
        assert!(!handle("https://preview.invalid@other.invalid").permitted);

        for (channel, invalid) in [
            (
                HybridWebCapabilityChannel::ContextualDidHandle,
                r#"{"purpose":" "}"#,
            ),
            (
                HybridWebCapabilityChannel::Mount10dHoloEmbed,
                "../private.10d",
            ),
            (HybridWebCapabilityChannel::ExtractRdfToSemanticFs, ""),
            (
                HybridWebCapabilityChannel::VerifyQecpCredential,
                r#"{"signed":true}"#,
            ),
            (
                HybridWebCapabilityChannel::SolidDataboxSync,
                "not a Turtle triple",
            ),
            (
                HybridWebCapabilityChannel::CommonsContributionReceipt,
                r#"{"effort_minutes":-1,"aud_minor_units":0}"#,
            ),
        ] {
            let rejected = evaluate_hybrid_page_request(
                BrowserExecutionMode::HybridAugmented,
                &identifiers,
                origin,
                channel,
                invalid,
            );
            assert!(
                !rejected.permitted,
                "{channel:?}: {}",
                rejected.reply_payload
            );
        }
    }

    #[test]
    fn tracks_chora_worker_routestamp_and_invalidation_lifecycle() {
        let mut supervisor = ChoraWorkerSupervisorSnapshot {
            worker_binary_path: "www-engine-worker".into(),
            ..ChoraWorkerSupervisorSnapshot::default()
        };
        assert!(!supervisor.observe_frame(15, 4, 1));

        supervisor.connect(15, 4);
        assert_eq!(supervisor.worker_binary_path, "www-engine-worker");
        assert!(supervisor.observe_frame(15, 4, 1));
        assert!(!supervisor.observe_frame(15, 4, 1));
        assert!(!supervisor.observe_frame(15, 3, 2));
        assert!(!supervisor.observe_frame(14, 4, 2));

        supervisor.registered_routes = 1;
        supervisor.pending_page_requests = 2;
        supervisor.forwarded_pointer_events = 3;
        supervisor.focus_active = true;
        supervisor.invalidate_route(5, "Manual route invalidation");
        assert_eq!(supervisor.frame_seq, 0);
        assert_eq!(supervisor.registered_routes, 0);
        assert_eq!(supervisor.pending_page_requests, 0);
        assert_eq!(supervisor.forwarded_pointer_events, 3);
        assert!(!supervisor.focus_active);
        assert!(!supervisor.observe_frame(15, 4, 2));
        assert!(supervisor.observe_frame(15, 5, 2));
        assert_eq!(
            supervisor.last_route_invalidation_reason,
            "Manual route invalidation"
        );

        supervisor.clear_worker("Worker exited");
        assert!(!supervisor.observe_frame(15, 5, 3));
        assert_eq!(supervisor.worker_binary_path, "www-engine-worker");
        assert_eq!(supervisor.last_route_invalidation_reason, "Worker exited");
        assert_eq!(supervisor.forwarded_pointer_events, 0);
    }

    #[test]
    fn governed_cell_preview_evaluates_only_bounded_pure_cells_in_hybrid_mode() {
        let ids = IdentifierSet {
            fabric_did: "did:qi:local-test".into(),
            records: Vec::new(),
            bindings: Vec::new(),
            validation: crate::identifiers::IdentityValidation::Underdetermined {
                distinct: 0,
                attested: 0,
            },
            quins: Vec::new(),
            turtle_path: Default::default(),
            validation_summary: String::new(),
        };
        let origin = "https://preview.invalid";
        let channel = HybridWebCapabilityChannel::GovernedVibeScriptEval;
        let preview =
            |mode, payload| evaluate_hybrid_page_request(mode, &ids, origin, channel, payload);
        let receipt = preview(BrowserExecutionMode::HybridAugmented, "= 21 + 21");
        assert!(receipt.permitted, "{}", receipt.reply_payload);
        assert_eq!(receipt.channel_id, "vibescript-eval");
        assert!(receipt.reply_payload.starts_with("VibeScript result: 42"));
        assert_eq!(
            receipt.sealed_nquin_parity,
            NQuin48::invariant(
                origin,
                &receipt.channel_id,
                q_hash_60(&receipt.reply_payload)
            )
            .parity()
        );
        assert_ne!(
            receipt.reply_payload,
            preview(BrowserExecutionMode::HybridAugmented, "= 40 + 2").reply_payload
        );

        let oversized = "1".repeat(MAX_GOVERNED_VIBESCRIPT_CELL_BYTES + 1);
        for input in [
            "= time.unix()",
            "= hidden_session",
            "= 2; 4",
            "= 0u64 + (-9223372036854775807 - 1)",
            oversized.as_str(),
        ] {
            assert!(!preview(BrowserExecutionMode::HybridAugmented, input).permitted);
        }
        let reference = preview(
            BrowserExecutionMode::HybridAugmented,
            "qualia://assets/test.10d",
        );
        assert!(!reference.permitted);
        assert!(reference.reply_payload.contains("No resource was fetched"));
        for mode in [
            BrowserExecutionMode::TraditionalWeb,
            BrowserExecutionMode::QualiaNative,
        ] {
            assert!(!preview(mode, "= 21 + 21").permitted);
        }
    }

    #[test]
    fn browser_is_packaged_independently_upgradable_and_supports_bookmarks_annotations_and_social_groups(
    ) {
        let tmp =
            std::env::temp_dir().join(format!("webizen-browser-app-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let host = LoopbackHost {
            daemon_base: "http://127.0.0.1:4242".into(),
            port: 4242,
            storage: tmp.clone(),
            install_root: tmp.join("bin"),
            trust: TrustRoots::for_storage(&tmp),
        };
        let _ = seed_bundled_qaps(&host);

        // 1. Verify Browser is packaged out-of-the-box in storage/Qapps/Browser
        let initial = inspect_browser_product(&host);
        assert_eq!(initial.package_id, "Browser");
        assert_eq!(initial.installed_version, "0.1.0");

        // 2. Verify independent hot-swap upgrade of Browser Qap without touching OS-Shell
        let upgraded = upgrade_browser_product_independent(&host, "0.1.1").expect("upgrade ok");
        assert_eq!(upgraded.installed_version, "0.1.1");

        // 3. Verify Mode 1 (TraditionalWeb) denies host QualiaDB bridge requests fail-closed
        let env = OsEnvironment::probe("tester");
        let gpu = probe_native_etalon(&tmp);
        let ids = crate::identifiers::enumerate_session_identifiers(&host, Sku::Guest, &env, &gpu);

        let denied = evaluate_hybrid_page_request(
            BrowserExecutionMode::TraditionalWeb,
            &ids,
            "https://example.com",
            HybridWebCapabilityChannel::ContextualDidHandle,
            "{}",
        );
        assert!(!denied.permitted);

        // 4. Verify Mode 3 (HybridAugmented) permits host-authorized capability requests and refuses master DID collapse
        let permitted = evaluate_hybrid_page_request(
            BrowserExecutionMode::HybridAugmented,
            &ids,
            "https://dev.civics.au",
            HybridWebCapabilityChannel::ContextualDidHandle,
            "{}",
        );
        assert!(permitted.permitted);
        assert!(permitted.reply_payload.contains("master ask refused=true"));

        // 5. Verify Semantic Bookmarks (QLink), Dialectical Web Annotations, and Semantically Defined Social Groups
        let mut sfs = SemanticFileSystem::load_or_seed(&tmp, &[]);
        let mut social = BrowserSocialGraph::load_or_seed(&tmp);
        assert!(social.semantic_groups.len() >= 3);
        assert!(social.bookmarks.len() >= 3);
        assert!(social.annotations.len() >= 2);

        let bm = social.save_semantic_bookmark(
            &tmp,
            &mut sfs,
            "https://observatory.civics.au/lga/snowy-valleys",
            "Snowy Valleys LGA Civics Evidence Graph",
            BrowserExecutionMode::HybridAugmented,
            "Regional housing, water, and solar microgrid dataset for Talbingo continuum.",
            "snowy-valleys-lga-dataset",
            "https://ns.webcivics.net/commons#EvidenceGraph",
            SocialDisclosureScope {
                lane: "group".into(),
                target_id: "group:catchment-stewards".into(),
            },
        );
        assert!(bm
            .depicted_entities
            .contains(&"snowy-valleys-lga-dataset".to_string()));
        assert!(sfs
            .entries
            .iter()
            .any(|e| e.asset_uri == "https://observatory.civics.au/lga/snowy-valleys"));

        let ann = social.add_web_annotation(
            &tmp,
            "https://observatory.civics.au/lga/snowy-valleys",
            "microgrid-capacity-table",
            "did:qi:peer:snowy-hydrologist",
            "ContextualClaim",
            "Hydrological pumped-storage buffer verified against 2026 seasonal inflow series.",
            "https://ns.webcivics.net/commons#ContextualAssertion",
            true,
            SocialDisclosureScope {
                lane: "group".into(),
                target_id: "group:catchment-stewards".into(),
            },
        );
        assert_eq!(ann.disclosure.target_id, "group:catchment-stewards");
        assert!(ann.qdnf_frame_digest.starts_with("QFrame:sha3-256:"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
