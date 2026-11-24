// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! WellFair Human-Rights Casework, Role-Qualified Semantic Instruments, Bio2RDF/Ontology -> `.q42`
//! Pipeline (with Permissive Commons Q-CDN distribution), CML (`ns.webcivics.net`) Legal & Human-Rights
//! Evidence Correlator, and Strong-Authenticated Online Specialist Portal (`wellfair_specialist_portal`).
//!
//! Architectural invariants:
//! 1. **Role-Qualified Clinical, Diagnostic & Wellbeing Semantic Instruments**:
//!    - Models clinical, psychological, neurodevelopmental, and functional determinations (such as
//!      **ADHD**, **PTSD**, trauma impact, functional capacity, or required accommodations) as
//!      hybrid **Verifiable Credentials (`VC = origin/integrity, never universal truth`) + Symbolic AI
//!      Inference Rules (`N3Logic` / `SHACL` -> 48-byte `NQuin48`)**.
//!    - Explicitly distinguishes [`InstrumentAuthorRole`]:
//!      - `PrincipalSelfAuthored`: Created or self-administered by the natural-person subject
//!        (lived-experience observations, accommodation requirements, symptom timelines).
//!      - `QualifiedSpecialistAttested`: Signed by an identified specialist (`ClinicalPsychologist`,
//!        `GeneralPractitioner`, `HumanRightsAdvocate`, `HousingSpecialist`).
//!      - `CollaborativeCoAttested`: Jointly signed by the principal and specialist (`CoAttestationBundle`).
//!      - `ContestedOrDisputedRecord`: Holds an external claim challenged by the principal with
//!        counter-evidence, preventing symbolic AI inference from treating disputed claims as fact.
//! 2. **Biomedical (`Bio2RDF`, `DOID`, `HP`) & Legal Ontology -> `.q42` Compilation + Permissive Commons Q-CDN**:
//!    - Pulls and compiles standard RDF/Turtle/N3 ontologies into `.q42` (`Q42\0` v4) containers with
//!      parity-checked 48-byte `NQuin48` records.
//!    - Supports local compilation on Desktop/Mobile AVF or elastic offload to `webizen-hostd` /
//!      `Synth Colab Ultra Burst`, and publishes **public reference `.q42` ontologies** onto the
//!      **Permissive Commons Q-CDN Swarm** (`< 10 KiB` `SignedRootIndex`) while keeping personal casework
//!      strictly private.
//! 3. **CML (`ns.webcivics.net`) + Legal & Human-Rights Evidence Correlator**:
//!    - Processes emails, PDFs (`QECP` C2PA JUMBF), agency letters, and field notes through CML
//!      (`TEXT -> CONCEPT -> LOGIC`) and correlates them with active Semantic Instruments and
//!      international/domestic human-rights & statutory duties (`UDHR`, `ICESCR`, `ICCPR`, `UN CRPD`,
//!      procedural fairness, duty of care, duty of inquiry).
//! 4. **Strong-Authenticated Online Specialist Permissive-Disclosure Portal (`webizen-hostd`)**:
//!    - Derives domain-scoped casework bundles (`Advocacy`, `Housing`, `Psychology`, `GeneralPractice`,
//!      `MultiDisciplinaryCoordination`) gated by Pairwise Specialist DID + Ed25519 challenge-response
//!      (or local deterministic rehearsal signature) + time-bounded `ConsentCredential` lease, with
//!      Six-Vector (`Who/When/Why/What/Where/Cost`) audit logging and one-click principal revocation.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use vibescript_abi::NQuin48;

use crate::{
    host_stewardship::validate_human_centric_host_nomenclature,
    identifiers::{q_hash_60, CO_ATTESTATION_BUNDLE_FORMULA, VC_ORIGIN_INTEGRITY_DISCLAIMER},
    os_orchestrator::ComputeExecutionTarget,
};

pub const WELLFAIR_PORTAL_STORE_REL_PATH: &str = "wellfair/specialist_portal_and_instruments.json";
const Q42_V4_MAGIC: &[u8; 8] = b"Q42\0\x04\x00\x00\x00";

/// The 5 specialist practice domains supported by the WellFair Permissive-Disclosure Portal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistDomainScope {
    LegalAndHumanRightsAdvocacy,
    HousingAndEnvironmentalSafety,
    PsychologyAndTraumaInformedSupport,
    GeneralPracticeAndClinicalHealth,
    MultiDisciplinaryCaseCoordination,
}

impl SpecialistDomainScope {
    pub const ALL: [Self; 5] = [
        Self::LegalAndHumanRightsAdvocacy,
        Self::HousingAndEnvironmentalSafety,
        Self::PsychologyAndTraumaInformedSupport,
        Self::GeneralPracticeAndClinicalHealth,
        Self::MultiDisciplinaryCaseCoordination,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::LegalAndHumanRightsAdvocacy => "advocacy_human_rights",
            Self::HousingAndEnvironmentalSafety => "housing_environmental_safety",
            Self::PsychologyAndTraumaInformedSupport => "psychology_trauma_support",
            Self::GeneralPracticeAndClinicalHealth => "general_practice_clinical",
            Self::MultiDisciplinaryCaseCoordination => "multidisciplinary_coordination",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::LegalAndHumanRightsAdvocacy => {
                "1. Legal & Human-Rights Advocacy (Conduct Records, QECP Exhibits & Statutory Duties)"
            }
            Self::HousingAndEnvironmentalSafety => {
                "2. Housing & Environmental Safety (Tenancy, Habitability & Spatiotemporal Continuity)"
            }
            Self::PsychologyAndTraumaInformedSupport => {
                "3. Psychology & Trauma-Informed Support (PTSD / ADHD Assessments & Accommodations)"
            }
            Self::GeneralPracticeAndClinicalHealth => {
                "4. General Practice & Clinical Health (Bio2RDF / DOID / HP & 3D Anatomy Atlas)"
            }
            Self::MultiDisciplinaryCaseCoordination => {
                "5. Multi-Disciplinary Case Coordination (Principal-Curated Cross-Specialist Brief)"
            }
        }
    }
}

/// Explicit authorship & epistemic standing of a WellFair Semantic Instrument.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentAuthorRole {
    /// Authored or self-administered by the natural-person subject (`NaturalAgent`).
    PrincipalSelfAuthored,
    /// Attested and signed by a qualified specialist (`Psychologist`, `GP`, `Advocate`, `Housing`).
    QualifiedSpecialistAttested,
    /// Jointly co-signed by both the principal and the specialist (`CoAttestationBundle`).
    CollaborativeCoAttested,
    /// External claim or historical diagnosis contested by the principal with counter-evidence.
    ContestedOrDisputedRecord,
}

impl InstrumentAuthorRole {
    pub const ALL: [Self; 4] = [
        Self::PrincipalSelfAuthored,
        Self::QualifiedSpecialistAttested,
        Self::CollaborativeCoAttested,
        Self::ContestedOrDisputedRecord,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::PrincipalSelfAuthored => "principal_self_authored",
            Self::QualifiedSpecialistAttested => "specialist_attested",
            Self::CollaborativeCoAttested => "collaborative_co_attested",
            Self::ContestedOrDisputedRecord => "contested_or_disputed",
        }
    }

    pub const fn badge(self) -> &'static str {
        match self {
            Self::PrincipalSelfAuthored => {
                "PRINCIPAL SELF-AUTHORED (Subject Lived-Experience & Accommodation Instrument)"
            }
            Self::QualifiedSpecialistAttested => {
                "SPECIALIST ATTESTED (Qualified Clinician / Professional Verifiable Credential)"
            }
            Self::CollaborativeCoAttested => {
                "CO-ATTESTED BUNDLE (Principal + Specialist Joint Determination)"
            }
            Self::ContestedOrDisputedRecord => {
                "CONTESTED / DISPUTED RECORD (Quarantined from Unqualified Symbolic Inference)"
            }
        }
    }

    /// Whether symbolic AI inference is permitted to treat this instrument as an active positive rule.
    pub const fn admitted_for_positive_symbolic_inference(self) -> bool {
        !matches!(self, Self::ContestedOrDisputedRecord)
    }
}

/// A Role-Qualified Clinical, Diagnostic, Legal, or Wellbeing Semantic Instrument.
///
/// Combines a Verifiable Credential envelope (`VC = origin/integrity`) with formal ontology
/// bindings (`Bio2RDF`, `DOID`, `HP`, `ICD-11`, `UN CRPD`) and machine-evaluatable symbolic AI
/// inference & accommodation rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleQualifiedSemanticInstrument {
    pub instrument_id: String,
    pub title: String,
    pub domain: SpecialistDomainScope,
    pub author_role: InstrumentAuthorRole,
    pub author_did: String,
    pub author_qualification_or_context: String,
    pub subject_principal_did: String,
    /// Standardized ontology concept IRIs (`bio2rdf:...`, `OBO:DOID_...`, `OBO:HP_...`, `icd11:...`, `crpd:...`).
    pub ontology_concept_iris: Vec<String>,
    /// Clinical / diagnostic / functional determination summary (e.g., `"ADHD & Complex PTSD determination"`).
    pub determination_summary: String,
    /// Explicit symbolic AI inference & deontic accommodation rule (`N3Logic` / `SHACL` -> `NQuin48`).
    pub symbolic_inference_rule: String,
    /// Required human-rights & procedural accommodations derived by the symbolic engine.
    pub derived_accommodations: Vec<String>,
    pub vc_origin_integrity_note: String,
    pub sealed_nquin_parity: u64,
}

/// A Biomedical (`Bio2RDF`, `DOID`, `HP`) or Legal/Human-Rights (`ns.webcivics.net`) Ontology
/// compiled into a `.q42` (`Q42\0` v4) binary container and shared on the Permissive Commons Q-CDN.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledOntologyQ42Package {
    pub ontology_id: String,
    pub title: String,
    pub source_namespace_iri: String,
    pub domain_family: String,
    pub execution_target_used: ComputeExecutionTarget,
    pub compiled_q42_path: PathBuf,
    pub triple_nquin_count: usize,
    pub q42_byte_len: usize,
    pub sha3_256_hex: String,
    pub shared_on_permissive_commons_qcdn: bool,
    pub qcdn_signed_root_merkle_hex: String,
    pub sample_concepts: Vec<String>,
    pub sealed_nquin_parity: u64,
}

/// Primary evidence artifact (Email, PDF with QECP C2PA JUMBF, Agency/Government Letter, Clinical Note)
/// processed through the CML (`ns.webcivics.net`) + Legal & Human-Rights Correlator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmlEvidenceCorrelationRecord {
    pub evidence_id: String,
    pub source_kind: String,
    pub source_title_or_subject: String,
    pub source_uri_or_hash: String,
    pub occurred_date_label: String,
    pub domain: SpecialistDomainScope,
    pub extracted_cml_concepts: Vec<String>,
    /// Correlated Role-Qualified Semantic Instrument IDs (e.g., linking an agency letter to PTSD/ADHD accommodations).
    pub correlated_instrument_ids: Vec<String>,
    /// Applicable international human-rights & domestic statutory instruments (`UN CRPD Art. 13/28`, `ICESCR Art. 11/12`, `DutyOfInquiry`).
    pub applicable_legal_and_human_rights_norms: Vec<String>,
    /// Symbolic AI evaluation outcome (`"Reasonable Accommodation Required"`, `"Duty of Inquiry Breach Flagged"`, etc.).
    pub symbolic_evaluation_summary: String,
    pub sealed_nquin_parity: u64,
}

/// Time-bounded, domain-scoped Permissive-Disclosure Lease granting an authenticated specialist
/// online access to a derived casework bundle on `webizen-hostd`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecialistDisclosureLease {
    pub lease_id: String,
    pub specialist_display_name: String,
    pub specialist_discipline: String,
    pub specialist_pairwise_did: String,
    /// Optional hex-encoded Ed25519 public key for live cryptographic signature verification on `webizen-hostd`.
    pub specialist_ed25519_pubkey_hex: String,
    /// Bearer / challenge token hash (`Sha3_256(access_token)`).
    pub access_token_sha3_hex: String,
    pub allowed_domains: Vec<SpecialistDomainScope>,
    pub valid_from_unix: u64,
    pub valid_until_unix: u64,
    pub allow_specialist_instrument_submission: bool,
    pub mobile_pocket_cosigned: bool,
    pub revoked_or_shredded: bool,
    pub revocation_reason: Option<String>,
    pub consent_credential_id: String,
    pub sealed_nquin_parity: u64,
}

/// Six-Vector Transparency Audit Log entry (`Who`, `When`, `Why`, `What`, `Where`, `Cost`)
/// recorded whenever a specialist accesses the online portal or submits an attestation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SixVectorPortalAuditEntry {
    pub audit_id: String,
    pub lease_id: String,
    /// Vector 1: Who (`specialist_pairwise_did` + discipline)
    pub who_specialist_did: String,
    /// Vector 2: When (UNIX timestamp)
    pub when_unix_secs: u64,
    /// Vector 3: Why (declared purpose / consultation context)
    pub why_purpose: String,
    /// Vector 4: What (accessed domain scopes, instruments, and CML evidence IDs)
    pub what_resources_accessed: String,
    /// Vector 5: Where (`webizen-hostd` endpoint / locus)
    pub where_host_locus: String,
    /// Vector 6: Cost (`"Human-Rights Permissive No-Charge"` or bounded consultation receipt)
    pub cost_classification: String,
    pub access_permitted: bool,
    pub sealed_nquin_parity: u64,
}

/// Derived, domain-scoped casework bundle served by `webizen-hostd` to an authenticated specialist
/// (contains zero raw Sanctuary keys and excludes any domain not authorized by the lease).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedSpecialistPortalBundle {
    pub lease_id: String,
    pub principal_did: String,
    pub specialist_pairwise_did: String,
    pub authorized_domains: Vec<SpecialistDomainScope>,
    pub visible_instruments: Vec<RoleQualifiedSemanticInstrument>,
    pub visible_cml_evidence: Vec<CmlEvidenceCorrelationRecord>,
    pub mounted_public_ontologies: Vec<String>,
    pub co_attestation_formula: String,
    pub vc_disclaimer: String,
    pub audit_receipt: SixVectorPortalAuditEntry,
    pub sealed_nquin_parity: u64,
}

/// Complete persistent state for the WellFair Human-Rights Casework & Online Specialist Portal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WellfairSpecialistPortalState {
    pub principal_did: String,
    pub host_portal_endpoint: String,
    pub semantic_instruments: Vec<RoleQualifiedSemanticInstrument>,
    pub compiled_ontologies: Vec<CompiledOntologyQ42Package>,
    pub cml_evidence_records: Vec<CmlEvidenceCorrelationRecord>,
    pub specialist_leases: Vec<SpecialistDisclosureLease>,
    pub six_vector_audit_ledger: Vec<SixVectorPortalAuditEntry>,
    pub last_portal_notice: String,
    pub sealed_nquin_parity: u64,
}

impl Default for WellfairSpecialistPortalState {
    fn default() -> Self {
        let mut s = Self {
            principal_did: "did:q42:qrc:local-principal-steward".into(),
            host_portal_endpoint: "https://ns.webcivics.net/wellfair/portal".into(),
            semantic_instruments: default_seeded_instruments(),
            compiled_ontologies: Vec::new(),
            cml_evidence_records: default_seeded_cml_evidence(),
            specialist_leases: default_seeded_specialist_leases(),
            six_vector_audit_ledger: Vec::new(),
            last_portal_notice: "WellFair Specialist Portal & Role-Qualified Semantic Instruments ready (Local Sanctuary primary)".into(),
            sealed_nquin_parity: 0,
        };
        s.recompute_parity();
        s
    }
}

impl WellfairSpecialistPortalState {
    pub fn load_or_seed(storage: &Path, principal_did: &str) -> Self {
        let path = storage.join(WELLFAIR_PORTAL_STORE_REL_PATH);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(mut loaded) = serde_json::from_slice::<Self>(&bytes) {
                if !loaded.semantic_instruments.is_empty() {
                    if !principal_did.trim().is_empty() {
                        loaded.principal_did = principal_did.to_owned();
                    }
                    if loaded.compiled_ontologies.is_empty() {
                        let _ = loaded.seed_default_ontologies(storage);
                    }
                    loaded.recompute_parity();
                    return loaded;
                }
            }
        }
        let mut seeded = Self::default();
        if !principal_did.trim().is_empty() {
            seeded.principal_did = principal_did.to_owned();
        }
        let _ = seeded.seed_default_ontologies(storage);
        seeded.recompute_parity();
        let _ = seeded.save(storage);
        seeded
    }

    pub fn save(&self, storage: &Path) -> Result<(), String> {
        let path = storage.join(WELLFAIR_PORTAL_STORE_REL_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
        fs::rename(&tmp, &path).map_err(|e| e.to_string())
    }

    fn recompute_parity(&mut self) {
        let mut acc = q_hash_60(&self.principal_did);
        for inst in &self.semantic_instruments {
            acc ^= inst.sealed_nquin_parity;
        }
        for ont in &self.compiled_ontologies {
            acc ^= ont.sealed_nquin_parity;
        }
        for ev in &self.cml_evidence_records {
            acc ^= ev.sealed_nquin_parity;
        }
        for lease in &self.specialist_leases {
            acc ^= lease.sealed_nquin_parity;
        }
        self.sealed_nquin_parity = acc;
    }

    /// Create or register a Role-Qualified Semantic Instrument (either `PrincipalSelfAuthored` by
    /// the subject, `QualifiedSpecialistAttested` by a psychologist/GP/advocate/housing specialist,
    /// `CollaborativeCoAttested`, or `ContestedOrDisputedRecord`).
    #[allow(clippy::too_many_arguments)]
    pub fn author_or_register_semantic_instrument(
        &mut self,
        storage: &Path,
        title: &str,
        domain: SpecialistDomainScope,
        author_role: InstrumentAuthorRole,
        author_did: &str,
        author_qualification_or_context: &str,
        ontology_concept_iris: Vec<String>,
        determination_summary: &str,
        symbolic_inference_rule: &str,
        derived_accommodations: Vec<String>,
    ) -> Result<RoleQualifiedSemanticInstrument, String> {
        validate_human_centric_host_nomenclature(title)?;
        validate_human_centric_host_nomenclature(determination_summary)?;
        if title.trim().is_empty() || determination_summary.trim().is_empty() {
            return Err("Instrument title and determination summary are required.".into());
        }
        if !author_did.trim().starts_with("did:") {
            return Err("Author DID must be a valid DID (e.g. did:qi:... or did:q42:...).".into());
        }

        let seq = self.semantic_instruments.len() + 1;
        let instrument_id = format!("si-wellfair:{}:{seq}", domain.code());
        let sealed_nquin_parity = NQuin48::from_parts(
            q_hash_60(&instrument_id),
            q_hash_60(author_role.code()),
            q_hash_60(author_did),
            q_hash_60(&self.principal_did),
            q_hash_60(determination_summary),
        )
        .parity();

        let inst = RoleQualifiedSemanticInstrument {
            instrument_id,
            title: title.trim().to_owned(),
            domain,
            author_role,
            author_did: author_did.trim().to_owned(),
            author_qualification_or_context: author_qualification_or_context.trim().to_owned(),
            subject_principal_did: self.principal_did.clone(),
            ontology_concept_iris,
            determination_summary: determination_summary.trim().to_owned(),
            symbolic_inference_rule: symbolic_inference_rule.trim().to_owned(),
            derived_accommodations,
            vc_origin_integrity_note: format!(
                "{} · Role: {}",
                VC_ORIGIN_INTEGRITY_DISCLAIMER,
                author_role.badge()
            ),
            sealed_nquin_parity,
        };
        self.semantic_instruments.insert(0, inst.clone());
        self.recompute_parity();
        self.last_portal_notice = format!(
            "Sealed Semantic Instrument '{}' [{}] (NQuin48 parity {:#018x})",
            inst.title,
            inst.author_role.badge(),
            inst.sealed_nquin_parity
        );
        self.save(storage)?;
        Ok(inst)
    }

    /// Pull/ingest a Biomedical (`Bio2RDF`, `DOID`, `HP`) or Legal/Human-Rights (`ns.webcivics.net`)
    /// ontology source (Turtle/N3 triples), compile it into a binary `.q42` (`Q42\0` v4) container
    /// with 48-byte `NQuin48` records, and optionally publish the public ontology to the
    /// **Permissive Commons Q-CDN Swarm**.
    pub fn compile_ontology_to_q42_and_share_commons(
        &mut self,
        storage: &Path,
        ontology_id: &str,
        title: &str,
        source_namespace_iri: &str,
        domain_family: &str,
        rdf_turtle_or_concepts: &str,
        execution_target_used: ComputeExecutionTarget,
        share_on_permissive_commons_qcdn: bool,
    ) -> Result<CompiledOntologyQ42Package, String> {
        validate_human_centric_host_nomenclature(title)?;
        validate_human_centric_host_nomenclature(rdf_turtle_or_concepts)?;
        let slug: String = ontology_id
            .trim()
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        if slug.is_empty() {
            return Err("Ontology ID must not be empty.".into());
        }

        // Parse lines/triples from the supplied Turtle/N3 or concept definitions into 48-byte NQuin48 records.
        let mut quins = Vec::new();
        let mut sample_concepts = Vec::new();
        let ctx_hash = q_hash_60(source_namespace_iri);
        for (idx, raw_line) in rdf_turtle_or_concepts
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("@prefix"))
            .enumerate()
        {
            if sample_concepts.len() < 6 {
                sample_concepts.push(raw_line.to_owned());
            }
            let quin = NQuin48::from_parts(
                q_hash_60(&format!("{source_namespace_iri}#{idx}")),
                q_hash_60("https://webizen.org/q42#ontologyAxiom"),
                q_hash_60(raw_line),
                ctx_hash,
                (idx + 1) as u64,
            );
            quins.push(quin);
        }
        if quins.is_empty() {
            let fallback = NQuin48::from_parts(
                q_hash_60(source_namespace_iri),
                q_hash_60("https://webizen.org/q42#ontologyRoot"),
                q_hash_60(title),
                ctx_hash,
                1,
            );
            quins.push(fallback);
            sample_concepts.push(format!("{source_namespace_iri} ({title})"));
        }

        // Encode deterministic `.q42` v4 container (`Q42\0\x04\0\0\0` + u64 count + 48-byte NQuin48 array).
        let mut q42_bytes = Vec::with_capacity(16 + quins.len() * 48);
        q42_bytes.extend_from_slice(Q42_V4_MAGIC);
        q42_bytes.extend_from_slice(&(quins.len() as u64).to_le_bytes());
        let mut parity_acc = 0u64;
        for q in &quins {
            for word in [
                q.subject(),
                q.predicate(),
                q.object(),
                q.context(),
                q.metadata(),
                q.parity(),
            ] {
                q42_bytes.extend_from_slice(&word.to_le_bytes());
            }
            parity_acc ^= q.parity();
        }

        let sha3_256_hex = format!("{:x}", Sha3_256::digest(&q42_bytes));
        let qcdn_signed_root_merkle_hex = if share_on_permissive_commons_qcdn {
            let mut m = Sha3_256::new();
            m.update(b"qcdn-permissive-commons-ontology-v1\0");
            m.update(source_namespace_iri.as_bytes());
            m.update(&q42_bytes);
            format!("qcdn:merkle:{:x}", m.finalize())[..44].to_owned()
        } else {
            "local-only (not shared on Q-CDN)".into()
        };

        let out_dir = storage.join("library").join("ontologies-q42");
        fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
        let compiled_q42_path = out_dir.join(format!("{slug}.q42"));
        fs::write(&compiled_q42_path, &q42_bytes).map_err(|e| e.to_string())?;

        let pkg = CompiledOntologyQ42Package {
            ontology_id: slug.clone(),
            title: title.trim().to_owned(),
            source_namespace_iri: source_namespace_iri.trim().to_owned(),
            domain_family: domain_family.trim().to_owned(),
            execution_target_used,
            compiled_q42_path,
            triple_nquin_count: quins.len(),
            q42_byte_len: q42_bytes.len(),
            sha3_256_hex,
            shared_on_permissive_commons_qcdn: share_on_permissive_commons_qcdn,
            qcdn_signed_root_merkle_hex,
            sample_concepts,
            sealed_nquin_parity: parity_acc,
        };
        self.compiled_ontologies.retain(|o| o.ontology_id != slug);
        self.compiled_ontologies.insert(0, pkg.clone());
        self.recompute_parity();
        self.last_portal_notice = format!(
            "Compiled ontology '{}' -> {} ({} NQuin48 triples · {} bytes · Q-CDN shared={})",
            pkg.title,
            pkg.compiled_q42_path.display(),
            pkg.triple_nquin_count,
            pkg.q42_byte_len,
            pkg.shared_on_permissive_commons_qcdn
        );
        self.save(storage)?;
        Ok(pkg)
    }

    /// Process primary evidence (Email, QECP PDF, Agency Letter, Clinical Note) through the
    /// CML (`ns.webcivics.net`) + Legal & Human-Rights Correlator, linking it against active
    /// Role-Qualified Semantic Instruments (excluding `ContestedOrDisputedRecord` from positive inference).
    pub fn correlate_cml_evidence_with_instruments_and_rights(
        &mut self,
        storage: &Path,
        source_kind: &str,
        source_title_or_subject: &str,
        source_uri_or_hash: &str,
        occurred_date_label: &str,
        domain: SpecialistDomainScope,
        evidence_text: &str,
    ) -> Result<CmlEvidenceCorrelationRecord, String> {
        validate_human_centric_host_nomenclature(source_title_or_subject)?;
        validate_human_centric_host_nomenclature(evidence_text)?;
        if source_title_or_subject.trim().is_empty() || evidence_text.trim().is_empty() {
            return Err("Evidence title/subject and content text are required.".into());
        }

        let lower = evidence_text.to_ascii_lowercase();
        let mut concepts = Vec::new();
        let mut norms = Vec::new();

        for (kw, concept, norm) in [
            (
                "adhd",
                "cml:neurodevelopmental:ADHD_ExecutiveFunction",
                "UN CRPD Art. 5 & Art. 24/27 (Reasonable Accommodation for Neurodivergent Cognition)",
            ),
            (
                "ptsd",
                "cml:trauma:PTSD_TraumaResponse",
                "UN CRPD Art. 13 (Access to Justice & Trauma-Informed Procedural Accommodations) + ICESCR Art. 12",
            ),
            (
                "trauma",
                "cml:trauma:TraumaInformedSafeguard",
                "UN CRPD Art. 16 (Freedom from Exploitation, Violence & Abuse) + Duty of Care",
            ),
            (
                "housing",
                "cml:welfare:HousingSecurityAndHabitability",
                "ICESCR Art. 11(1) (Right to Adequate Housing) + Residential Tenancy Habitability Duty",
            ),
            (
                "evict",
                "cml:welfare:TenancyPreservationAlert",
                "UN Human Rights Committee General Comment 7 (Protection Against Forced Eviction)",
            ),
            (
                "letter",
                "cml:jural:AdministrativeCorrespondence",
                "Administrative Law Duty of Procedural Fairness & Duty of Inquiry (Diligent vs Negligent)",
            ),
            (
                "email",
                "cml:provenance:WrittenNoticeEvidence",
                "QECP Verifiable Correspondence Provenance & ConductRecord Audit Trail",
            ),
            (
                "pdf",
                "cml:provenance:QecpDocumentExhibit",
                "C2PA JUMBF + 48-Byte NQuin48 Document Exhibit Integrity",
            ),
            (
                "medication",
                "cml:clinical:PharmacotherapyContinuity",
                "Bio2RDF / DrugBank Clinical Continuity & General Practice Duty of Care",
            ),
        ] {
            if lower.contains(kw) {
                concepts.push(concept.to_owned());
                if !norms.iter().any(|n| n == norm) {
                    norms.push(norm.to_owned());
                }
            }
        }
        if concepts.is_empty() {
            concepts.push("cml:civics:PrimaryEvidenceStatement".into());
        }
        if norms.is_empty() {
            norms.push("UDHR Art. 1, 12 & 22 (Human Dignity, Privacy & Social Security Foundation)".into());
        }

        // Correlate with admitted positive Semantic Instruments in the same domain or MultiDisciplinary scope.
        let mut correlated_instrument_ids = Vec::new();
        let mut accommodation_notes = Vec::new();
        for inst in &self.semantic_instruments {
            if inst.author_role.admitted_for_positive_symbolic_inference()
                && (inst.domain == domain
                    || inst.domain == SpecialistDomainScope::PsychologyAndTraumaInformedSupport
                    || domain == SpecialistDomainScope::MultiDisciplinaryCaseCoordination)
            {
                correlated_instrument_ids.push(inst.instrument_id.clone());
                for acc in &inst.derived_accommodations {
                    if !accommodation_notes.contains(acc) {
                        accommodation_notes.push(acc.clone());
                    }
                }
            }
        }

        let symbolic_evaluation_summary = format!(
            "Extracted {} CML concept(s) across {} human-rights/statutory norm(s); correlated with {} admitted Semantic Instrument(s) (Contested/Disputed claims excluded). Required accommodations/actions: [{}]",
            concepts.len(),
            norms.len(),
            correlated_instrument_ids.len(),
            if accommodation_notes.is_empty() {
                "Preserve evidentiary chain & require diligent Duty of Inquiry".to_owned()
            } else {
                accommodation_notes.join("; ")
            }
        );

        let seq = self.cml_evidence_records.len() + 1;
        let evidence_id = format!("cml-ev:{}:{seq}", domain.code());
        let sealed_nquin_parity = NQuin48::from_parts(
            q_hash_60(&evidence_id),
            q_hash_60("https://ns.webcivics.net/cml#correlatedEvidence"),
            q_hash_60(source_title_or_subject),
            q_hash_60(domain.code()),
            q_hash_60(&symbolic_evaluation_summary),
        )
        .parity();

        let rec = CmlEvidenceCorrelationRecord {
            evidence_id,
            source_kind: source_kind.trim().to_owned(),
            source_title_or_subject: source_title_or_subject.trim().to_owned(),
            source_uri_or_hash: source_uri_or_hash.trim().to_owned(),
            occurred_date_label: occurred_date_label.trim().to_owned(),
            domain,
            extracted_cml_concepts: concepts,
            correlated_instrument_ids,
            applicable_legal_and_human_rights_norms: norms,
            symbolic_evaluation_summary,
            sealed_nquin_parity,
        };
        self.cml_evidence_records.insert(0, rec.clone());
        self.recompute_parity();
        self.last_portal_notice = format!(
            "Correlated CML evidence '{}' ({}) across {} human-rights/legal norms",
            rec.source_title_or_subject,
            rec.source_kind,
            rec.applicable_legal_and_human_rights_norms.len()
        );
        self.save(storage)?;
        Ok(rec)
    }

    /// Issue a strong-authenticated, time-bounded, domain-scoped Permissive-Disclosure Lease
    /// for a specialist (Advocate, Housing Specialist, Psychologist, GP) to access derived
    /// casework on `webizen-hostd`.
    #[allow(clippy::too_many_arguments)]
    pub fn grant_specialist_disclosure_lease(
        &mut self,
        storage: &Path,
        specialist_display_name: &str,
        specialist_discipline: &str,
        specialist_pairwise_did: &str,
        specialist_ed25519_pubkey_hex: &str,
        raw_access_token: &str,
        allowed_domains: Vec<SpecialistDomainScope>,
        valid_from_unix: u64,
        valid_until_unix: u64,
        allow_specialist_instrument_submission: bool,
        mobile_pocket_cosigned: bool,
    ) -> Result<SpecialistDisclosureLease, String> {
        validate_human_centric_host_nomenclature(specialist_display_name)?;
        validate_human_centric_host_nomenclature(specialist_discipline)?;
        if specialist_display_name.trim().is_empty()
            || !specialist_pairwise_did.trim().starts_with("did:")
        {
            return Err("Specialist name and pairwise DID (did:...) are required.".into());
        }
        if raw_access_token.trim().len() < 12 {
            return Err("Strong specialist access token must be at least 12 characters.".into());
        }
        if allowed_domains.is_empty() {
            return Err("Grant at least one SpecialistDomainScope to the lease.".into());
        }
        if valid_until_unix <= valid_from_unix {
            return Err("Lease expiry must be after valid_from_unix.".into());
        }

        let seq = self.specialist_leases.len() + 1;
        let lease_id = format!("lease:specialist:{seq}");
        let access_token_sha3_hex = format!("{:x}", Sha3_256::digest(raw_access_token.trim().as_bytes()));
        let consent_credential_id = format!("urn:qualia:consent:wellfair-specialist:{seq}");
        let sealed_nquin_parity = NQuin48::from_parts(
            q_hash_60(&lease_id),
            q_hash_60(specialist_pairwise_did),
            q_hash_60(&access_token_sha3_hex),
            valid_from_unix,
            valid_until_unix,
        )
        .parity();

        let lease = SpecialistDisclosureLease {
            lease_id,
            specialist_display_name: specialist_display_name.trim().to_owned(),
            specialist_discipline: specialist_discipline.trim().to_owned(),
            specialist_pairwise_did: specialist_pairwise_did.trim().to_owned(),
            specialist_ed25519_pubkey_hex: specialist_ed25519_pubkey_hex.trim().to_owned(),
            access_token_sha3_hex,
            allowed_domains,
            valid_from_unix,
            valid_until_unix,
            allow_specialist_instrument_submission,
            mobile_pocket_cosigned,
            revoked_or_shredded: false,
            revocation_reason: None,
            consent_credential_id,
            sealed_nquin_parity,
        };
        self.specialist_leases.insert(0, lease.clone());
        self.recompute_parity();
        self.last_portal_notice = format!(
            "Granted strong-auth Specialist Lease '{}' to {} ({})",
            lease.lease_id, lease.specialist_display_name, lease.specialist_discipline
        );
        self.save(storage)?;
        Ok(lease)
    }

    /// One-click revocation / crypto-shredding (`ADR 0011 D1/D2`) of a single specialist lease,
    /// or emergency freeze of all active online leases from Desktop or Mobile AVF.
    pub fn revoke_specialist_lease(
        &mut self,
        storage: &Path,
        lease_id_or_all: &str,
        reason: &str,
    ) -> Result<usize, String> {
        let mut count = 0usize;
        for lease in &mut self.specialist_leases {
            if (lease_id_or_all == "*" || lease.lease_id == lease_id_or_all)
                && !lease.revoked_or_shredded
            {
                lease.revoked_or_shredded = true;
                lease.access_token_sha3_hex = "0".repeat(64);
                lease.revocation_reason = Some(reason.trim().to_owned());
                count += 1;
            }
        }
        if count == 0 {
            return Err("No active specialist lease matched the revocation request.".into());
        }
        self.recompute_parity();
        self.last_portal_notice = format!(
            "Revoked & crypto-shredded {count} specialist disclosure lease(s): {reason}"
        );
        self.save(storage)?;
        Ok(count)
    }

    /// Authenticate a specialist request on `webizen-hostd` (verifying Lease ID, Pairwise DID,
    /// strong access token hash, time window, and non-revoked status), record a **Six-Vector
    /// Transparency Audit Entry**, and derive the strictly domain-scoped casework bundle.
    pub fn verify_and_derive_specialist_portal_bundle(
        &mut self,
        storage: &Path,
        lease_id: &str,
        specialist_pairwise_did: &str,
        raw_access_token: &str,
        declared_purpose: &str,
        now_unix_secs: u64,
    ) -> Result<DerivedSpecialistPortalBundle, String> {
        let token_hash = format!("{:x}", Sha3_256::digest(raw_access_token.trim().as_bytes()));
        let lease = self
            .specialist_leases
            .iter()
            .find(|l| l.lease_id == lease_id)
            .cloned()
            .ok_or_else(|| "Unknown specialist disclosure lease.".to_owned())?;

        let valid_time = now_unix_secs >= lease.valid_from_unix && now_unix_secs < lease.valid_until_unix;
        let permitted = !lease.revoked_or_shredded
            && valid_time
            && lease.specialist_pairwise_did == specialist_pairwise_did.trim()
            && lease.access_token_sha3_hex == token_hash;

        let audit_seq = self.six_vector_audit_ledger.len() + 1;
        let audit_id = format!("audit:6vec:{audit_seq}");

        if !permitted {
            let reason = if lease.revoked_or_shredded {
                "DENIED: Lease revoked & crypto-shredded by principal"
            } else if !valid_time {
                "DENIED: Lease outside authorized time window"
            } else {
                "DENIED: Specialist DID or strong authentication token mismatch"
            };
            let denied_entry = SixVectorPortalAuditEntry {
                audit_id,
                lease_id: lease.lease_id.clone(),
                who_specialist_did: specialist_pairwise_did.trim().to_owned(),
                when_unix_secs: now_unix_secs,
                why_purpose: declared_purpose.trim().to_owned(),
                what_resources_accessed: reason.into(),
                where_host_locus: self.host_portal_endpoint.clone(),
                cost_classification: "Human-Rights Permissive Portal (Denied Access)".into(),
                access_permitted: false,
                sealed_nquin_parity: NQuin48::from_parts(
                    q_hash_60(&lease.lease_id),
                    q_hash_60(specialist_pairwise_did),
                    now_unix_secs,
                    0,
                    0,
                )
                .parity(),
            };
            self.six_vector_audit_ledger.insert(0, denied_entry);
            let _ = self.save(storage);
            return Err(reason.into());
        }

        let includes_all = lease
            .allowed_domains
            .contains(&SpecialistDomainScope::MultiDisciplinaryCaseCoordination);

        let visible_instruments: Vec<RoleQualifiedSemanticInstrument> = self
            .semantic_instruments
            .iter()
            .filter(|i| includes_all || lease.allowed_domains.contains(&i.domain))
            .cloned()
            .collect();

        let visible_cml_evidence: Vec<CmlEvidenceCorrelationRecord> = self
            .cml_evidence_records
            .iter()
            .filter(|e| includes_all || lease.allowed_domains.contains(&e.domain))
            .cloned()
            .collect();

        let mounted_public_ontologies: Vec<String> = self
            .compiled_ontologies
            .iter()
            .map(|o| format!("{} ({} · {})", o.title, o.source_namespace_iri, o.qcdn_signed_root_merkle_hex))
            .collect();

        let what_summary = format!(
            "Authorized domains: [{}] · {} Semantic Instrument(s) · {} CML Evidence record(s)",
            lease
                .allowed_domains
                .iter()
                .map(|d| d.code())
                .collect::<Vec<_>>()
                .join(", "),
            visible_instruments.len(),
            visible_cml_evidence.len()
        );

        let audit_parity = NQuin48::from_parts(
            q_hash_60(&audit_id),
            q_hash_60(&lease.lease_id),
            q_hash_60(&lease.specialist_pairwise_did),
            now_unix_secs,
            q_hash_60(&what_summary),
        )
        .parity();

        let audit_receipt = SixVectorPortalAuditEntry {
            audit_id,
            lease_id: lease.lease_id.clone(),
            who_specialist_did: format!(
                "{} ({} · {})",
                lease.specialist_pairwise_did,
                lease.specialist_display_name,
                lease.specialist_discipline
            ),
            when_unix_secs: now_unix_secs,
            why_purpose: declared_purpose.trim().to_owned(),
            what_resources_accessed: what_summary,
            where_host_locus: self.host_portal_endpoint.clone(),
            cost_classification: "Human-Rights Permissive Portal (No-Charge Specialist Review)".into(),
            access_permitted: true,
            sealed_nquin_parity: audit_parity,
        };
        self.six_vector_audit_ledger.insert(0, audit_receipt.clone());
        self.last_portal_notice = format!(
            "Served derived Specialist Portal bundle to {} ({} instruments, {} CML evidence records · 6-Vector audit logged)",
            lease.specialist_display_name,
            visible_instruments.len(),
            visible_cml_evidence.len()
        );
        self.save(storage)?;

        Ok(DerivedSpecialistPortalBundle {
            lease_id: lease.lease_id,
            principal_did: self.principal_did.clone(),
            specialist_pairwise_did: lease.specialist_pairwise_did,
            authorized_domains: lease.allowed_domains,
            visible_instruments,
            visible_cml_evidence,
            mounted_public_ontologies,
            co_attestation_formula: CO_ATTESTATION_BUNDLE_FORMULA.to_owned(),
            vc_disclaimer: VC_ORIGIN_INTEGRITY_DISCLAIMER.to_owned(),
            audit_receipt,
            sealed_nquin_parity: audit_parity,
        })
    }

    fn seed_default_ontologies(&mut self, storage: &Path) -> Result<(), String> {
        for (id, title, iri, family, ttl, target) in [
            (
                "bio2rdf-clinical-neuro-pharma",
                "Bio2RDF Clinical Neurobiology & Pharmacology Reference Slice",
                "https://bio2rdf.org/clinical_neuro_pharma",
                "Bio2RDF / DOID / HP / DrugBank",
                "bio2rdf:DOID_1094 rdfs:label \"Attention Deficit Hyperactivity Disorder (ADHD)\" .\nbio2rdf:DOID_2055 rdfs:label \"Post-Traumatic Stress Disorder (PTSD)\" .\nbio2rdf:HP_0007018 rdfs:label \"Attention deficit / executive function variance\" .\nbio2rdf:HP_0033676 rdfs:label \"Trauma-triggered autonomic hyperarousal\" .",
                ComputeExecutionTarget::SynthColabUltraBurst,
            ),
            (
                "webcivics-cml-human-rights-law",
                "WebCivics CML Jural, Statutory & International Human-Rights Ontology",
                "https://ns.webcivics.net/ontologies/cml-human-rights-v1",
                "CML / UDHR / ICESCR / UN CRPD / Administrative Law",
                "cml:CRPD_Art13 rdfs:label \"Access to Justice & Procedural Accommodations\" .\ncml:ICESCR_Art11 rdfs:label \"Right to Adequate & Safe Housing\" .\ncml:ICESCR_Art12 rdfs:label \"Right to Highest Attainable Physical & Mental Health\" .\ncml:DutyOfInquiry rdfs:label \"Diligent vs Negligent Administrative Duty of Inquiry\" .",
                ComputeExecutionTarget::StewardedWebizenHost,
            ),
        ] {
            let _ = self.compile_ontology_to_q42_and_share_commons(
                storage,
                id,
                title,
                iri,
                family,
                ttl,
                target,
                true,
            )?;
        }
        Ok(())
    }
}

fn default_seeded_instruments() -> Vec<RoleQualifiedSemanticInstrument> {
    let items = [
        (
            "si-wellfair:psychology_trauma_support:1",
            "Psychologist Clinical Determination: ADHD & Complex PTSD Profile",
            SpecialistDomainScope::PsychologyAndTraumaInformedSupport,
            InstrumentAuthorRole::QualifiedSpecialistAttested,
            "did:qi:specialist:clinical-psychologist-au",
            "Registered Clinical Psychologist (Trauma & Neurodevelopmental Assessment)",
            vec![
                "https://bio2rdf.org/doid:1094 (ADHD)".into(),
                "https://bio2rdf.org/doid:2055 (PTSD)".into(),
                "https://purl.obolibrary.org/obo/HP_0007018".into(),
                "https://ns.webcivics.net/crpd#Article13_ProceduralAccommodation".into(),
            ],
            "Formal clinical determination confirming combined ADHD and PTSD following cumulative institutional/housing stressors; mandates written asynchronous communication, trauma-informed procedural pacing, and stable housing environment.",
            "IF Subject(?p) AND AttestedDiagnosis(?p, bio2rdf:DOID_1094) AND AttestedDiagnosis(?p, bio2rdf:DOID_2055) THEN RequireAccommodation(?p, crpd:WrittenAsyncCommunication) AND RequireAccommodation(?p, crpd:StableSafeHousingPriority)",
            vec![
                "Provide written asynchronous correspondence and advance notice for legal/agency deadlines".into(),
                "Prioritize safe, quiet, stable housing to prevent autonomic trauma exacerbation".into(),
                "Prohibit punitive administrative defaults caused by executive-function or trauma overload".into(),
            ],
        ),
        (
            "si-wellfair:advocacy_human_rights:2",
            "Principal Self-Authored Lived-Experience & Functional Impact Instrument",
            SpecialistDomainScope::LegalAndHumanRightsAdvocacy,
            InstrumentAuthorRole::PrincipalSelfAuthored,
            "did:q42:qrc:local-principal-steward",
            "Principal Natural Person (Subject Self-Authored Functional & Chronological Statement)",
            vec![
                "https://ns.webcivics.net/cml#SubjectLivedExperienceAssertion".into(),
                "https://ns.webcivics.net/udhr#Article22".into(),
            ],
            "Principal-authored record detailing chronological sequence of housing disruption, uncorrected agency records, and required accommodations for specialist coordination.",
            "IF SelfAuthoredAssertion(?p, ?claim) THEN BindEpistemicRole(?claim, role:PrincipalSelfAuthored) AND RequireDutyOfInquiryBeforeAdverseAction(?p, ?claim)",
            vec![
                "Cross-reference primary QECP PDF/email exhibits before relying on third-party summaries".into(),
            ],
        ),
        (
            "si-wellfair:general_practice_clinical:3",
            "Contested External Agency Summary (Quarantined from Positive Symbolic Inference)",
            SpecialistDomainScope::GeneralPracticeAndClinicalHealth,
            InstrumentAuthorRole::ContestedOrDisputedRecord,
            "did:qi:agency:unverified-external-note",
            "Disputed Uncorroborated Administrative Note (Challenged with QECP Counter-Evidence)",
            vec!["https://ns.webcivics.net/cml#DisputedAllegation".into()],
            "Incomplete administrative summary omitting documented PTSD/ADHD accommodations and housing safety evidence; quarantined by principal challenge.",
            "IF ContestedRecord(?r) THEN BlockPositiveInference(?r) AND FlagForAdvocateAndGpReview(?r)",
            vec![
                "Quarantined: Symbolic AI must not infer clinical or legal conclusions from this contested record".into(),
            ],
        ),
    ];

    items
        .into_iter()
        .map(
            |(
                id,
                title,
                domain,
                role,
                author_did,
                qual,
                iris,
                summary,
                rule,
                accommodations,
            )| {
                let parity = NQuin48::from_parts(
                    q_hash_60(id),
                    q_hash_60(role.code()),
                    q_hash_60(author_did),
                    q_hash_60("did:q42:qrc:local-principal-steward"),
                    q_hash_60(summary),
                )
                .parity();
                RoleQualifiedSemanticInstrument {
                    instrument_id: id.into(),
                    title: title.into(),
                    domain,
                    author_role: role,
                    author_did: author_did.into(),
                    author_qualification_or_context: qual.into(),
                    subject_principal_did: "did:q42:qrc:local-principal-steward".into(),
                    ontology_concept_iris: iris,
                    determination_summary: summary.into(),
                    symbolic_inference_rule: rule.into(),
                    derived_accommodations: accommodations,
                    vc_origin_integrity_note: format!(
                        "{} · Role: {}",
                        VC_ORIGIN_INTEGRITY_DISCLAIMER,
                        role.badge()
                    ),
                    sealed_nquin_parity: parity,
                }
            },
        )
        .collect()
}

fn default_seeded_cml_evidence() -> Vec<CmlEvidenceCorrelationRecord> {
    let summary = "Extracted 3 CML concept(s) across 3 human-rights/statutory norm(s); correlated with 2 admitted Semantic Instrument(s) (Contested/Disputed claims excluded). Required accommodations/actions: [Provide written asynchronous correspondence and advance notice for legal/agency deadlines; Prioritize safe, quiet, stable housing to prevent autonomic trauma exacerbation]";
    let parity = NQuin48::from_parts(
        q_hash_60("cml-ev:housing_environmental_safety:1"),
        q_hash_60("https://ns.webcivics.net/cml#correlatedEvidence"),
        q_hash_60("Housing & Clinical Accommodation Notice + QECP Correspondence Bundle"),
        q_hash_60(SpecialistDomainScope::HousingAndEnvironmentalSafety.code()),
        q_hash_60(summary),
    )
    .parity();
    vec![CmlEvidenceCorrelationRecord {
        evidence_id: "cml-ev:housing_environmental_safety:1".into(),
        source_kind: "QECP Verifiable PDF + Email Thread (.pdf / .eml)".into(),
        source_title_or_subject:
            "Housing & Clinical Accommodation Notice + QECP Correspondence Bundle".into(),
        source_uri_or_hash: "urn:qualia:qecp:housing-clinical-correspondence-2026.pdf".into(),
        occurred_date_label: "2026-11-20".into(),
        domain: SpecialistDomainScope::HousingAndEnvironmentalSafety,
        extracted_cml_concepts: vec![
            "cml:welfare:HousingSecurityAndHabitability".into(),
            "cml:neurodevelopmental:ADHD_ExecutiveFunction".into(),
            "cml:trauma:PTSD_TraumaResponse".into(),
        ],
        correlated_instrument_ids: vec![
            "si-wellfair:psychology_trauma_support:1".into(),
            "si-wellfair:advocacy_human_rights:2".into(),
        ],
        applicable_legal_and_human_rights_norms: vec![
            "ICESCR Art. 11(1) (Right to Adequate Housing) + Residential Tenancy Habitability Duty"
                .into(),
            "UN CRPD Art. 13 & Art. 28 (Procedural & Social Protection Accommodations)".into(),
            "Administrative Law Duty of Procedural Fairness & Diligent Duty of Inquiry".into(),
        ],
        symbolic_evaluation_summary: summary.into(),
        sealed_nquin_parity: parity,
    }]
}

fn default_seeded_specialist_leases() -> Vec<SpecialistDisclosureLease> {
    let token_hash = format!(
        "{:x}",
        Sha3_256::digest(b"wellfair-specialist-demo-token-2026")
    );
    let parity = NQuin48::from_parts(
        q_hash_60("lease:specialist:1"),
        q_hash_60("did:qi:specialist:multidisciplinary-team"),
        q_hash_60(&token_hash),
        1_700_000_000,
        1_900_000_000,
    )
    .parity();
    vec![SpecialistDisclosureLease {
        lease_id: "lease:specialist:1".into(),
        specialist_display_name: "Multidisciplinary Advocacy, Housing, Psychology & GP Team".into(),
        specialist_discipline: "Advocacy · Housing · Clinical Psychology · General Practice".into(),
        specialist_pairwise_did: "did:qi:specialist:multidisciplinary-team".into(),
        specialist_ed25519_pubkey_hex: String::new(),
        access_token_sha3_hex: token_hash,
        allowed_domains: vec![
            SpecialistDomainScope::LegalAndHumanRightsAdvocacy,
            SpecialistDomainScope::HousingAndEnvironmentalSafety,
            SpecialistDomainScope::PsychologyAndTraumaInformedSupport,
            SpecialistDomainScope::GeneralPracticeAndClinicalHealth,
        ],
        valid_from_unix: 1_700_000_000,
        valid_until_unix: 1_900_000_000,
        allow_specialist_instrument_submission: true,
        mobile_pocket_cosigned: true,
        revoked_or_shredded: false,
        revocation_reason: None,
        consent_credential_id: "urn:qualia:consent:wellfair-specialist:1".into(),
        sealed_nquin_parity: parity,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_qualified_instruments_separate_principal_specialist_and_contested_and_compile_bio2rdf_q42() {
        let tmp = std::env::temp_dir().join(format!(
            "webizen-wellfair-portal-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let mut portal =
            WellfairSpecialistPortalState::load_or_seed(&tmp, "did:q42:qrc:timothy-principal");
        assert!(portal.compiled_ontologies.len() >= 2);
        assert!(portal
            .compiled_ontologies
            .iter()
            .all(|o| o.shared_on_permissive_commons_qcdn && o.compiled_q42_path.is_file()));

        // Register a GP clinical instrument and correlate an email/PDF evidence record.
        let gp_inst = portal
            .author_or_register_semantic_instrument(
                &tmp,
                "GP Care Plan & Medication Continuity Attestation",
                SpecialistDomainScope::GeneralPracticeAndClinicalHealth,
                InstrumentAuthorRole::QualifiedSpecialistAttested,
                "did:qi:specialist:gp-clinic",
                "General Practitioner (Primary Care)",
                vec!["https://bio2rdf.org/drugbank:clinical-care-plan".into()],
                "Confirms primary care treatment plan and supports housing stability accommodation.",
                "IF PrimaryCarePlan(?p) THEN SupportHousingContinuity(?p)",
                vec!["Maintain continuity of local GP & psychology care team".into()],
            )
            .unwrap();
        assert!(gp_inst
            .author_role
            .admitted_for_positive_symbolic_inference());

        let ev = portal
            .correlate_cml_evidence_with_instruments_and_rights(
                &tmp,
                "Email + PDF Attachment",
                "Request for ADHD/PTSD Procedural & Housing Accommodation",
                "urn:qualia:qecp:advocacy-housing-notice.pdf",
                "2026-11-24",
                SpecialistDomainScope::LegalAndHumanRightsAdvocacy,
                "Formal email and QECP PDF letter citing ADHD and PTSD clinical determinations and requesting safe housing habitability and written procedural fairness.",
            )
            .unwrap();
        // Contested instrument #3 must never be included in positive symbolic correlations.
        assert!(!ev
            .correlated_instrument_ids
            .iter()
            .any(|id| id == "si-wellfair:general_practice_clinical:3"));
        assert!(!ev.applicable_legal_and_human_rights_norms.is_empty());

        // Grant a Housing-only specialist lease and verify domain isolation + instant revocation.
        let lease = portal
            .grant_specialist_disclosure_lease(
                &tmp,
                "Regional Housing Advocate",
                "Housing & Tenancy Specialist",
                "did:qi:specialist:housing-advocate",
                "",
                "housing-strong-token-2026",
                vec![SpecialistDomainScope::HousingAndEnvironmentalSafety],
                1_700_000_000,
                1_900_000_000,
                true,
                true,
            )
            .unwrap();

        let bundle = portal
            .verify_and_derive_specialist_portal_bundle(
                &tmp,
                &lease.lease_id,
                "did:qi:specialist:housing-advocate",
                "housing-strong-token-2026",
                "Housing safety & tenancy review",
                1_750_000_000,
            )
            .unwrap();
        assert_eq!(
            bundle.authorized_domains,
            vec![SpecialistDomainScope::HousingAndEnvironmentalSafety]
        );
        assert!(bundle
            .visible_cml_evidence
            .iter()
            .all(|e| e.domain == SpecialistDomainScope::HousingAndEnvironmentalSafety));

        // Revoke the lease and confirm subsequent access is denied and logged in the 6-Vector ledger.
        portal
            .revoke_specialist_lease(&tmp, &lease.lease_id, "Consultation completed")
            .unwrap();
        assert!(portal
            .verify_and_derive_specialist_portal_bundle(
                &tmp,
                &lease.lease_id,
                "did:qi:specialist:housing-advocate",
                "housing-strong-token-2026",
                "Post-revocation attempt",
                1_750_000_100,
            )
            .is_err());
        assert!(!portal.six_vector_audit_ledger[0].access_permitted);

        let _ = fs::remove_dir_all(&tmp);
    }
}
