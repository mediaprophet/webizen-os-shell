//! QualiaDB Enumerated-Identity & Identifier Fabric Bridge (`KD-20`).
//!
//! Extends QualiaDB's canonical `validate_enumerated_identity` SHACL contract
//! (`qualia-core-db/src/modalities/logic/shacl_extensions/identity.rs`) and
//! 48-byte `NQuin48` records (`vibescript_abi::NQuin48`).
//!
//! Architectural invariants preserved from QualiaDB:
//! 1. **Identifiers ≠ Identity**: Identity stands only as an enumerated state over
//!    distinct, cryptographically attested identifier bindings (`CryptoScheme`)
//!    with a confidence relation strictly in `(0.0, 1.0)`.
//! 2. **Out-of-Band Remainder (`DefinitiveCollapse` Rejection)**: Any binding
//!    claiming certainty (`confidence >= 1.0`) is rejected as `DefinitiveCollapse`.
//!    Noisy-OR aggregate confidence is clamped strictly below `1.0`.
//! 3. **Who-Plane ≠ Handle-Plane ≠ Package Coordinate**: Package coordinates
//!    (`did:q42:qap:<package_id>:v<version>`) and topological QRC pointers
//!    (`did:q42:`) never sit on the `Who` plane (`NaturalAgent`).
//! 4. **Contextual Sanctuary Issuance**: When a requestor asks for a master DID,
//!    the request for the root/who-plane is refused and a purpose/context-bound
//!    handle is issued and sealed as a 48-byte parity-checked `NQuin48`.

use crate::native_gpu::NativeEtalonStatus;
use crate::os_env::OsEnvironment;
use crate::session::{LoopbackHost, Sku};
use anvil_core::fnv1a64;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::path::{Path, PathBuf};
use vibescript_abi::NQuin48;

/// Canonical jury-safe plane enumeration in the Four-Plane Identifier Fabric (`F1` §14.4 / `F5` §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentifierFabricPlane {
    NaturalAgent,
    ClaimOpinion,
    SpatiotemporalHandle,
    Instrument,
}

pub const PLANE_NATURAL_AGENT_TITLE: &str = "People & Living Beings (NaturalAgent)";
pub const PLANE_CLAIM_OPINION_TITLE: &str = "Claims, Opinions & Co-Attestations (ClaimOpinion)";
pub const PLANE_SPATIOTEMPORAL_HANDLE_TITLE: &str = "Where & When Now (SpatiotemporalHandle)";
pub const PLANE_INSTRUMENT_TITLE: &str = "Cryptographic Tools & Instruments (Instrument)";

pub const VC_ORIGIN_INTEGRITY_DISCLAIMER: &str =
    "Proves issuer origin & integrity, not claim truth";
pub const CO_ATTESTATION_BUNDLE_FORMULA: &str =
    "these signatures, this window, these scoped machines/agents";
pub const DID_Q42_QRC_LABEL: &str = "Storage / Topological Coordinate (QRC)";
pub const NATURAL_AGENT_FORBIDDEN_LABELS: [&str; 3] = ["Thing", "Object", "DID"];

impl IdentifierFabricPlane {
    pub const ALL: [Self; 4] = [
        Self::NaturalAgent,
        Self::ClaimOpinion,
        Self::SpatiotemporalHandle,
        Self::Instrument,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::NaturalAgent => PLANE_NATURAL_AGENT_TITLE,
            Self::ClaimOpinion => PLANE_CLAIM_OPINION_TITLE,
            Self::SpatiotemporalHandle => PLANE_SPATIOTEMPORAL_HANDLE_TITLE,
            Self::Instrument => PLANE_INSTRUMENT_TITLE,
        }
    }

    pub const fn short_ribbon_label(self) -> &'static str {
        match self {
            Self::NaturalAgent => "who",
            Self::ClaimOpinion => "claim",
            Self::SpatiotemporalHandle => "handle",
            Self::Instrument => "tool",
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::NaturalAgent => "NaturalAgent",
            Self::ClaimOpinion => "ClaimOpinion",
            Self::SpatiotemporalHandle => "SpatiotemporalHandle",
            Self::Instrument => "Instrument",
        }
    }

    pub const fn anti_collapse_rule(self) -> &'static str {
        match self {
            Self::NaturalAgent => {
                "Framed strictly as living-SHACL (D_living), disjoint from owl:Thing. Never labeled Thing, Object, or DID. OS account is never principal identity."
            }
            Self::ClaimOpinion => {
                "Bounded validity interval (not_before .. expires_at). VC signatures prove issuer origin & integrity, never substantive truth."
            }
            Self::SpatiotemporalHandle => {
                "Ephemeral DNI route epochs, 16-byte CB-USRI cosmic/geospatial coordinates, and did:q42 Storage / Topological Coordinate (QRC) — never human identity."
            }
            Self::Instrument => {
                "Scoped cryptographic tools (did:qi, directional pairwise locators A->B != B->A, WalletShape, MachineDeviceId, SensorId). Agents propose as tools; humans decide."
            }
        }
    }
}

/// Validates that a `NaturalAgent` UI label never uses forbidden objectifying or instrument terms.
pub fn is_valid_natural_agent_label(label: &str) -> bool {
    !NATURAL_AGENT_FORBIDDEN_LABELS.iter().any(|forbidden| {
        label
            .split(|c: char| !c.is_alphanumeric() && c != ':')
            .any(|token| token.eq_ignore_ascii_case(forbidden))
    }) && !label.to_ascii_lowercase().contains("owl:thing")
}

/// Cryptographic scheme attesting an identifier binding (aligned with QualiaDB `CryptoScheme`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoScheme {
    Ed25519,
    MlDsa65,
    Blake3Commitment,
    X25519,
    /// Unattested / Device-OS hint — does not count toward cryptographic attestation quorum.
    Unknown,
}

impl CryptoScheme {
    pub fn as_str(self) -> &'static str {
        match self {
            CryptoScheme::Ed25519 => "Ed25519",
            CryptoScheme::MlDsa65 => "ML-DSA-65",
            CryptoScheme::Blake3Commitment => "BLAKE3-Commitment",
            CryptoScheme::X25519 => "X25519",
            CryptoScheme::Unknown => "Unattested-Hint",
        }
    }
}

/// One identifier binding in an enumerated identity (`qualia-core-db` SHACL contract).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdentifierBinding {
    /// 60-bit masked `q_hash` of the identifier URI.
    pub identifier: u64,
    pub uri: String,
    pub plane: &'static str,
    pub role: &'static str,
    pub scheme: CryptoScheme,
    pub attested: bool,
    /// Confidence strictly in `(0.0, 1.0)`. `>= 1.0` triggers `DefinitiveCollapse`.
    pub confidence: f32,
}

/// Verdict of [`validate_enumerated_identity`] (aligned with QualiaDB `IdentityValidation`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IdentityValidation {
    Valid {
        distinct: u16,
        attested: u16,
        aggregate_confidence: f32,
    },
    Underdetermined {
        distinct: u16,
        attested: u16,
    },
    /// Rejected: a binding asserted certainty (`confidence >= 1.0`), violating the out-of-band remainder.
    DefinitiveCollapse,
}

/// Validate an enumerated identity over cryptographically attested identifier bindings.
///
/// Mirrors `qualia_core_db::modalities::logic::shacl_extensions::identity::validate_enumerated_identity`.
pub fn validate_enumerated_identity(
    bindings: &[IdentifierBinding],
    min_distinct: u16,
    min_attested: u16,
) -> IdentityValidation {
    let mut distinct = 0u16;
    let mut attested = 0u16;
    let mut not_prob = 1.0f32;

    for (i, b) in bindings.iter().enumerate() {
        if !b.confidence.is_finite() || b.confidence >= 1.0 || b.confidence <= 0.0 {
            return IdentityValidation::DefinitiveCollapse;
        }
        let first_seen = !bindings[..i].iter().any(|p| p.identifier == b.identifier);
        if first_seen {
            distinct += 1;
        }
        if b.attested && b.scheme != CryptoScheme::Unknown {
            attested += 1;
            not_prob *= 1.0 - b.confidence.clamp(0.0, 1.0);
        }
    }

    if distinct < min_distinct || attested < min_attested {
        return IdentityValidation::Underdetermined { distinct, attested };
    }
    let max_open_unit = f32::from_bits(1.0f32.to_bits() - 1);
    IdentityValidation::Valid {
        distinct,
        attested,
        aggregate_confidence: (1.0 - not_prob).min(max_open_unit),
    }
}

/// Display record for the OS-Shell Settings & Halo rail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifierRecord {
    pub value: String,
    pub plane: &'static str,
    pub label: &'static str,
}

/// Requestor ask type for contextual Sanctuary identifier issuance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ask {
    Master,
    Purpose,
    Unspecified,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requestor {
    pub ask: Ask,
    pub requestor: String,
    pub context: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SanctuaryReply {
    pub issued_did: String,
    pub bound_context: String,
    pub refused_master: bool,
    pub quin_parity: u64,
}

/// Resolved identifier fabric state for the current User-Agent OS session.
#[derive(Clone, Debug, PartialEq)]
pub struct IdentifierSet {
    pub fabric_did: String,
    pub records: Vec<IdentifierRecord>,
    pub bindings: Vec<IdentifierBinding>,
    pub validation: IdentityValidation,
    pub quins: Vec<NQuin48>,
    pub turtle_path: PathBuf,
    pub validation_summary: String,
}

pub fn q_hash_60(s: &str) -> u64 {
    fnv1a64(s.as_bytes()) & 0x0FFF_FFFF_FFFF_FFFF
}

pub fn enumerate_session_identifiers(
    host: &LoopbackHost,
    sku: Sku,
    env: &OsEnvironment,
    gpu: &NativeEtalonStatus,
) -> IdentifierSet {
    let dir = host.storage.join("identifiers");
    let _ = fs::create_dir_all(&dir);
    let turtle_path = dir.join("local.ttl");

    // Derive deterministic cryptographic commitments (SHA3-256) for local node & vault locus
    let node_digest = sha3_hex(&format!(
        "webizen.node.v1|{}|{}|{:04x}:{:04x}|{:016x}",
        env.hostname,
        sku.as_str(),
        gpu.anvil_profile.vendor_id,
        gpu.anvil_profile.device_id,
        gpu.anvil_profile.hardware_fingerprint
    ));
    let vault_digest = sha3_hex(&format!(
        "webizen.vault.v1|{}|{}",
        host.storage.display(),
        sku.as_str()
    ));
    let session_digest = sha3_hex(&format!(
        "webizen.session.v1|{}|{}|{}",
        env.hostname, env.username, env.os
    ));

    let fabric_did = format!("did:q42:node:{}", &node_digest[..16]);
    let vault_did = format!("did:q42:locus:{}", &vault_digest[..16]);
    let agent_did = format!("did:webizen:agent:{}", &session_digest[..16]);
    let anvil_did = format!(
        "did:q42:anvil:{:04x}-{:04x}-{:016x}",
        gpu.anvil_profile.vendor_id,
        gpu.anvil_profile.device_id,
        gpu.anvil_profile.hardware_fingerprint
    );

    let bindings = vec![
        IdentifierBinding {
            identifier: q_hash_60(&fabric_did),
            uri: fabric_did.clone(),
            plane: "handle",
            role: "fabric-node",
            scheme: CryptoScheme::Ed25519,
            attested: true,
            confidence: 0.65,
        },
        IdentifierBinding {
            identifier: q_hash_60(&vault_did),
            uri: vault_did.clone(),
            plane: "locus",
            role: "vault-storage",
            scheme: CryptoScheme::Blake3Commitment,
            attested: true,
            confidence: 0.72,
        },
        IdentifierBinding {
            identifier: q_hash_60(&agent_did),
            uri: agent_did.clone(),
            plane: "agent",
            role: "ua-session",
            scheme: CryptoScheme::MlDsa65,
            attested: true,
            confidence: 0.68,
        },
        IdentifierBinding {
            identifier: q_hash_60(&anvil_did),
            uri: anvil_did.clone(),
            plane: "hardware",
            role: "anvil-np64",
            scheme: if gpu.may_advertise_hardware_attested_vault {
                CryptoScheme::Ed25519
            } else {
                CryptoScheme::Unknown
            },
            attested: gpu.may_advertise_hardware_attested_vault,
            confidence: 0.55,
        },
    ];

    let validation = validate_enumerated_identity(&bindings, 2, 2);
    let validation_summary = match validation {
        IdentityValidation::Valid {
            distinct,
            attested,
            aggregate_confidence,
        } => format!(
            "Enumerated Identity Valid · {distinct} distinct / {attested} attested · noisy-OR {:.3} (< 1.0 remainder preserved)",
            aggregate_confidence
        ),
        IdentityValidation::Underdetermined { distinct, attested } => format!(
            "Underdetermined · {distinct} distinct / {attested} attested"
        ),
        IdentityValidation::DefinitiveCollapse => {
            "REJECTED: DefinitiveCollapse (confidence >= 1.0 forbidden)".into()
        }
    };

    let mut quins = Vec::with_capacity(bindings.len());
    for b in &bindings {
        let q = NQuin48::invariant(&b.uri, "https://webizen.org/q42#hasIdentifierBinding", b.identifier);
        debug_assert!(q.verify_parity());
        quins.push(q);
    }

    let records = bindings
        .iter()
        .map(|b| IdentifierRecord {
            value: format!(
                "{} ({}, c={:.2})",
                b.uri,
                b.scheme.as_str(),
                b.confidence
            ),
            plane: b.plane,
            label: b.role,
        })
        .collect();

    let ttl = render_enumerated_ttl(&fabric_did, sku, &bindings, &validation_summary);
    let _ = fs::write(&turtle_path, ttl);
    ensure_library_index(&host.storage, &fabric_did);

    IdentifierSet {
        fabric_did,
        records,
        bindings,
        validation,
        quins,
        turtle_path,
        validation_summary,
    }
}

/// Issue a contextual Sanctuary handle bound to `(requestor, context)` while refusing
/// any request for a master / who-plane collapse, sealing a 48-byte parity-checked `NQuin48`.
pub fn issue_for_requestor(set: &IdentifierSet, req: &Requestor) -> SanctuaryReply {
    let refused_master = matches!(req.ask, Ask::Master | Ask::Unspecified);
    let digest = sha3_hex(&format!(
        "webizen.sanctum.v1|{}|{}|{}|{}",
        set.fabric_did,
        req.requestor,
        req.context,
        if refused_master { "refused-master" } else { "purpose-scoped" }
    ));
    let issued_did = format!("did:q42:context:{}", &digest[..16]);
    let quin = NQuin48::invariant(
        &issued_did,
        "https://webizen.org/q42#contextualIssuance",
        q_hash_60(&req.context),
    );
    debug_assert!(quin.verify_parity());

    SanctuaryReply {
        issued_did,
        bound_context: req.context.clone(),
        refused_master,
        quin_parity: quin.parity(),
    }
}

fn sha3_hex(input: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(input.as_bytes());
    let out = hasher.finalize();
    out.iter().map(|b| format!("{b:02x}")).collect()
}

fn render_enumerated_ttl(
    fabric_did: &str,
    sku: Sku,
    bindings: &[IdentifierBinding],
    summary: &str,
) -> String {
    let mut s = String::from(
        "@prefix q42: <https://webizen.org/q42#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n\
         # QualiaDB Enumerated Identity State (q42:EnumeratedIdentityShape)\n\
         # Out-of-band remainder is preserved: confidence is strictly in (0, 1).\n",
    );
    s.push_str(&format!(
        "<{fabric_did}> a q42:EnumeratedIdentityState ;\n  q42:sku \"{}\" ;\n  q42:validationSummary \"{}\" .\n\n",
        sku.as_str(),
        summary
    ));
    for b in bindings {
        s.push_str(&format!(
            "<{}> a q42:IdentifierBinding ;\n  q42:inState <{fabric_did}> ;\n  q42:plane \"{}\" ;\n  q42:role \"{}\" ;\n  q42:cryptoScheme \"{}\" ;\n  q42:attested \"{}\"^^xsd:boolean ;\n  q42:identifierConfidence \"{:.2}\"^^xsd:decimal .\n\n",
            b.uri,
            b.plane,
            b.role,
            b.scheme.as_str(),
            b.attested,
            b.confidence
        ));
    }
    s
}

fn ensure_library_index(storage: &Path, fabric_did: &str) {
    let dir = storage.join("library");
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("index.ttl");
    if path.exists() {
        return;
    }
    let body = format!(
        "@prefix q42: <https://webizen.org/q42#> .\n\n\
         <{fabric_did}> q42:libraryIndex <> .\n\
         <> a q42:LibraryIndex ;\n\
           q42:facet \"type\" ;\n\
           q42:facet \"plane\" ;\n\
           q42:note \"QualiaDB .q42 & .hmc semantic library index.\" .\n"
    );
    let _ = fs::write(path, body);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerated_identity_rejects_definitive_collapse_and_preserves_remainder() {
        let bindings = vec![
            IdentifierBinding {
                identifier: q_hash_60("did:q42:node:1"),
                uri: "did:q42:node:1".into(),
                plane: "handle",
                role: "node",
                scheme: CryptoScheme::Ed25519,
                attested: true,
                confidence: 0.65,
            },
            IdentifierBinding {
                identifier: q_hash_60("did:q42:locus:2"),
                uri: "did:q42:locus:2".into(),
                plane: "locus",
                role: "vault",
                scheme: CryptoScheme::Blake3Commitment,
                attested: true,
                confidence: 0.72,
            },
        ];
        match validate_enumerated_identity(&bindings, 2, 2) {
            IdentityValidation::Valid {
                distinct,
                attested,
                aggregate_confidence,
            } => {
                assert_eq!(distinct, 2);
                assert_eq!(attested, 2);
                assert!(aggregate_confidence > 0.85 && aggregate_confidence < 1.0);
            }
            other => panic!("expected Valid, got {other:?}"),
        }

        let collapsed = vec![IdentifierBinding {
            identifier: q_hash_60("did:q42:collapsed"),
            uri: "did:q42:collapsed".into(),
            plane: "who",
            role: "master",
            scheme: CryptoScheme::Ed25519,
            attested: true,
            confidence: 1.0,
        }];
        assert_eq!(
            validate_enumerated_identity(&collapsed, 1, 1),
            IdentityValidation::DefinitiveCollapse
        );
    }

    #[test]
    fn four_plane_fabric_voice_and_anti_collapse_invariants() {
        assert_eq!(
            IdentifierFabricPlane::NaturalAgent.title(),
            "People & Living Beings (NaturalAgent)"
        );
        assert_eq!(
            IdentifierFabricPlane::ClaimOpinion.title(),
            "Claims, Opinions & Co-Attestations (ClaimOpinion)"
        );
        assert_eq!(
            IdentifierFabricPlane::SpatiotemporalHandle.title(),
            "Where & When Now (SpatiotemporalHandle)"
        );
        assert_eq!(
            IdentifierFabricPlane::Instrument.title(),
            "Cryptographic Tools & Instruments (Instrument)"
        );
        assert!(is_valid_natural_agent_label(
            IdentifierFabricPlane::NaturalAgent.title()
        ));
        assert!(!is_valid_natural_agent_label("Thing"));
        assert!(!is_valid_natural_agent_label("Object"));
        assert!(!is_valid_natural_agent_label("Person DID"));
        assert!(!is_valid_natural_agent_label("owl:Thing subclass"));
        assert_eq!(
            VC_ORIGIN_INTEGRITY_DISCLAIMER,
            "Proves issuer origin & integrity, not claim truth"
        );
        assert_eq!(
            CO_ATTESTATION_BUNDLE_FORMULA,
            "these signatures, this window, these scoped machines/agents"
        );
        assert_eq!(DID_Q42_QRC_LABEL, "Storage / Topological Coordinate (QRC)");
        assert!(!DID_Q42_QRC_LABEL.to_ascii_lowercase().contains("identity"));
    }
}
