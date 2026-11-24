// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Sanctuary policy metadata and local onboarding rehearsals.
//!
//! The shell does not link the QualiaDB vault client. This module never opens a
//! vault, distributes recovery shares, contacts a peer, or asserts attestation.

use crate::host_stewardship::{
    validate_human_centric_host_nomenclature, NaturalHostStewardshipFabric,
};
use crate::identifiers::{q_hash_60, IdentifierSet};
use crate::llm_management::{
    SENSITIVITY_COMMONS_GROUP, SENSITIVITY_PUBLIC_ONLY, SENSITIVITY_SANCTUARY_LOCAL_MAX,
};
use crate::native_gpu::NativeEtalonStatus;
use sha3::{Digest, Sha3_256};
use std::net::{Ipv4Addr, Ipv6Addr};
use vibescript_abi::NQuin48;

const RECOVERY_FIXTURE: &[u8; 32] = b"webizen-public-shamir-fixture-v1";
const REHEARSAL_THRESHOLD: u8 = 2;
const REHEARSAL_HOLDERS: [(&str, &str); 4] = [
    ("Home Server", "home-server"),
    ("12V Camper Rig", "camper-rig"),
    ("Mobile AVF Pocket Co-Signer", "pocket-cosigner"),
    ("Family or Housemate Guardian", "household-guardian"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SanctuaryVaultSlotSpec {
    pub slot_index: u8,
    pub label: &'static str,
    pub encryption_profile: &'static str,
    pub coercion_posture: &'static str,
    pub max_sensitivity: u8,
    /// This catalog does not read encrypted records. Zero is a local preview count.
    pub active_record_count: usize,
    pub sealed_nquin_parity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShamirGuardianShareHolder {
    pub share_index: u8,
    pub holder_label: String,
    pub holder_did: String,
    pub locus_kind: String,
    pub share_commitment_hex: String,
    /// Fixture material is present locally. This does not attest a remote holder.
    pub verified_present: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PocketCosignerPairingState {
    pub mobile_node_id: String,
    pub mobile_did: String,
    pub wireguard_endpoint: String,
    pub vsock_cid: u32,
    pub pkvm_attestation_status: String,
    pub paired: bool,
    /// Reserved for a verified remote signature receipt.
    pub last_cosign_receipt_hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SanctuaryOnboardingState {
    pub digital_birth_did: String,
    pub vault_slots: [SanctuaryVaultSlotSpec; 4],
    pub shamir_threshold_m: u8,
    pub shamir_total_n: u8,
    pub shamir_shares: Vec<ShamirGuardianShareHolder>,
    pub last_reconstruction_test: String,
    pub pocket_cosigner: PocketCosignerPairingState,
}

impl Default for SanctuaryOnboardingState {
    fn default() -> Self {
        let shamir_shares: Vec<_> = REHEARSAL_HOLDERS
            .iter()
            .zip(1u8..)
            .map(|(&(label, locus), share_index)| ShamirGuardianShareHolder {
                share_index,
                holder_label: label.to_owned(),
                holder_did: format!("did:webizen:rehearsal:{locus}"),
                locus_kind: locus.to_owned(),
                share_commitment_hex: share_commitment(
                    share_index,
                    &fixture_share(share_index, REHEARSAL_THRESHOLD),
                ),
                verified_present: true,
            })
            .collect();
        Self {
            digital_birth_did: String::new(),
            vault_slots: [
                vault_slot(
                    0,
                    "Public Commons",
                    "Public disclosure policy",
                    "Only material approved for the Commons.",
                    SENSITIVITY_PUBLIC_ONLY,
                ),
                vault_slot(
                    1,
                    "Contextual Social",
                    "Context-bound disclosure policy",
                    "Scoped to purpose and audience.",
                    SENSITIVITY_COMMONS_GROUP,
                ),
                vault_slot(
                    2,
                    "Plausible-Deniability Decoy",
                    "Target profile: Argon2id and AES-256-GCM",
                    "Independent decoy access. Zero Nuke-PIN policy.",
                    SENSITIVITY_PUBLIC_ONLY,
                ),
                vault_slot(
                    3,
                    "Sanctuary100 Personal Keep",
                    "Target profile: Argon2id and AES-256-GCM",
                    "Local-only policy with scoped access.",
                    SENSITIVITY_SANCTUARY_LOCAL_MAX,
                ),
            ],
            shamir_threshold_m: REHEARSAL_THRESHOLD,
            shamir_total_n: REHEARSAL_HOLDERS.len() as u8,
            shamir_shares,
            last_reconstruction_test: String::new(),
            pocket_cosigner: PocketCosignerPairingState {
                vsock_cid: 2,
                pkvm_attestation_status: "Pocket pKVM attestation pending.".to_owned(),
                ..PocketCosignerPairingState::default()
            },
        }
    }
}

pub fn build_sanctuary_onboarding_state(
    ids: &IdentifierSet,
    fabric: &NaturalHostStewardshipFabric,
    gpu: &NativeEtalonStatus,
) -> SanctuaryOnboardingState {
    let mut state = SanctuaryOnboardingState::default();
    if let Some(binding) = ids
        .bindings
        .iter()
        .find(|binding| binding.role == "digital-birth-record" && binding.attested)
    {
        state.digital_birth_did = binding.uri.clone();
    }

    if !fabric.sanctuary_keep_local_only {
        state.vault_slots[3] = vault_slot(
            3,
            "Sanctuary100 Personal Keep",
            "Target profile: Argon2id and AES-256-GCM",
            "Local-only policy is not confirmed in the current Stewardship fabric.",
            SENSITIVITY_SANCTUARY_LOCAL_MAX,
        );
    }

    if gpu.may_advertise_hardware_attested_vault {
        state.pocket_cosigner.pkvm_attestation_status =
            "Local Anvil gate passed. Pocket pKVM attestation pending.".into();
    }
    state
}

fn vault_slot(
    slot_index: u8,
    label: &'static str,
    encryption_profile: &'static str,
    coercion_posture: &'static str,
    max_sensitivity: u8,
) -> SanctuaryVaultSlotSpec {
    let subject = format!("urn:webizen:sanctuary:onboarding:slot:{slot_index}");
    let policy = format!("{label}|{encryption_profile}|{coercion_posture}|{max_sensitivity}");
    let quin = seal(
        &subject,
        "https://webizen.org/q42#sanctuarySlotPolicy",
        q_hash_60(&policy),
    );
    SanctuaryVaultSlotSpec {
        slot_index,
        label,
        encryption_profile,
        coercion_posture,
        max_sensitivity,
        active_record_count: 0,
        sealed_nquin_parity: quin.parity(),
    }
}

fn seal(subject: &str, predicate: &str, metadata: u64) -> NQuin48 {
    let quin = NQuin48::invariant(subject, predicate, metadata);
    debug_assert!(quin.verify_parity());
    quin
}

/// Reconstruct a public fixture using distinct GF(256) shares and verify its commitment.
pub fn simulate_shamir_recovery_ceremony(
    state: &mut SanctuaryOnboardingState,
    participating_share_indices: &[u8],
) -> Result<String, String> {
    let threshold = state.shamir_threshold_m;
    let total = state.shamir_total_n;
    if !(2..=total).contains(&threshold)
        || usize::from(total) != REHEARSAL_HOLDERS.len()
        || state.shamir_shares.len() != usize::from(total)
    {
        return Err("The rehearsal threshold or roster is invalid.".into());
    }
    if participating_share_indices.len() < usize::from(threshold) {
        return Err(format!(
            "Select at least {threshold} distinct rehearsal shares."
        ));
    }
    if participating_share_indices.len() > usize::from(total) {
        return Err("The selection exceeds the rehearsal roster.".into());
    }

    let mut indices = participating_share_indices.to_vec();
    indices.sort_unstable();
    if indices.iter().any(|index| !(1..=total).contains(index))
        || indices.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(
            "Each selected share must have a distinct index in the rehearsal roster.".into(),
        );
    }

    let mut shares = Vec::with_capacity(indices.len());
    for &index in &indices {
        let mut holders = state
            .shamir_shares
            .iter()
            .filter(|holder| holder.share_index == index);
        let holder = holders
            .next()
            .filter(|holder| holder.verified_present)
            .ok_or_else(|| format!("Rehearsal share {index} is unavailable."))?;
        if holders.next().is_some() {
            return Err(format!("Rehearsal share {index} has ambiguous holders."));
        }
        let data = fixture_share(index, threshold);
        if share_commitment(index, &data) != holder.share_commitment_hex {
            return Err(format!(
                "Rehearsal share {index} failed commitment verification."
            ));
        }
        shares.push((index, data));
    }

    let reconstructed = interpolate_at_zero(&shares);
    if reconstructed != *RECOVERY_FIXTURE {
        return Err("The reconstructed fixture failed verification.".into());
    }

    let mut transcript = Sha3_256::new();
    transcript.update(b"webizen.sanctuary.shamir.rehearsal.v1");
    transcript.update([threshold, total]);
    transcript.update(&indices);
    transcript.update(reconstructed);
    let digest = hex(&transcript.finalize());
    let quin = seal(
        &format!("urn:webizen:sanctuary:rehearsal:{digest}"),
        "https://webizen.org/q42#publicFixtureRecovered",
        q_hash_60(&digest),
    );
    let receipt = format!(
        "Rehearsal reconstructed the public fixture with {} distinct shares. SHA3-256 {digest}. NQuin48 parity {:#018x}.",
        shares.len(),
        quin.parity(),
    );
    state.last_reconstruction_test.clone_from(&receipt);
    Ok(receipt)
}

fn fixture_share(index: u8, threshold: u8) -> [u8; 32] {
    let mut share = *RECOVERY_FIXTURE;
    let mut power = index;
    for order in 1..threshold {
        let mut hash = Sha3_256::new();
        hash.update(b"webizen.sanctuary.public-fixture.coefficient.v1");
        hash.update([threshold, order]);
        let coefficient = hash.finalize();
        for (value, factor) in share.iter_mut().zip(coefficient) {
            *value ^= gf_multiply(power, factor);
        }
        power = gf_multiply(power, index);
    }
    share
}

fn share_commitment(index: u8, share: &[u8; 32]) -> String {
    let mut hash = Sha3_256::new();
    hash.update(b"webizen.sanctuary.public-fixture.share.v1");
    hash.update([index]);
    hash.update(share);
    hex(&hash.finalize())
}

// The caller verifies that share indices are nonzero and distinct.
pub(crate) fn interpolate_at_zero(shares: &[(u8, [u8; 32])]) -> [u8; 32] {
    let mut output = [0; 32];
    for (index, data) in shares {
        let mut weight = 1;
        for (other, _) in shares {
            if index != other {
                let ratio = gf_multiply(*other, gf_inverse(*index ^ *other));
                weight = gf_multiply(weight, ratio);
            }
        }
        for (value, byte) in output.iter_mut().zip(data) {
            *value ^= gf_multiply(weight, *byte);
        }
    }
    output
}

pub(crate) fn gf_multiply(mut left: u8, mut right: u8) -> u8 {
    let mut result = 0;
    for _ in 0..8 {
        if right & 1 != 0 {
            result ^= left;
        }
        left = (left << 1) ^ if left & 0x80 != 0 { 0x1b } else { 0 };
        right >>= 1;
    }
    result
}

fn gf_inverse(value: u8) -> u8 {
    let squared = gf_multiply(value, value);
    let mut result = squared;
    let mut power = squared;
    for _ in 0..6 {
        power = gf_multiply(power, power);
        result = gf_multiply(result, power);
    }
    result
}

/// Validate the endpoint and seal a local pairing transcript for human review.
/// Real pairing requires an authenticated challenge and a verified AVF attestation response.
pub fn pair_and_test_pocket_cosigner(
    state: &mut SanctuaryOnboardingState,
    mobile_did: &str,
    wg_endpoint: &str,
) -> Result<String, String> {
    if state.pocket_cosigner.paired {
        return Err(
            "Disconnect the existing pocket pairing before rehearsing a replacement.".into(),
        );
    }
    let did = mobile_did.trim();
    validate_human_centric_host_nomenclature(did)?;
    validate_mobile_did(did)?;
    if did == state.digital_birth_did {
        return Err("Use a contextual pairing DID. Keep the Digital Birth Record private.".into());
    }
    let endpoint = wg_endpoint.trim();
    validate_human_centric_host_nomenclature(endpoint)?;
    let endpoint = normalize_endpoint(endpoint)?;

    let mut hash = Sha3_256::new();
    hash.update(b"webizen.sanctuary.pocket-pairing.rehearsal.v1\0");
    hash.update(did.as_bytes());
    hash.update(b"\0");
    hash.update(endpoint.as_bytes());
    let digest = hex(&hash.finalize());
    let quin = seal(
        &format!("urn:webizen:sanctuary:pocket-pairing:rehearsal:{digest}"),
        "https://webizen.org/q42#pocketPairingRehearsal",
        q_hash_60(&digest),
    );
    state.pocket_cosigner.mobile_node_id = format!("rehearsal:{}", &digest[..16]);
    state.pocket_cosigner.mobile_did = did.to_owned();
    state.pocket_cosigner.wireguard_endpoint = endpoint;
    Ok(format!(
        "Pairing rehearsal recorded. SHA3-256 {digest}. NQuin48 parity {:#018x}. Pocket authentication and pKVM attestation pending.",
        quin.parity(),
    ))
}

fn validate_mobile_did(did: &str) -> Result<(), String> {
    let Some((method, identifier)) = did.strip_prefix("did:").and_then(|did| did.split_once(':'))
    else {
        return Err("Enter a scoped mobile DID.".into());
    };
    let method_valid = !method.is_empty()
        && method
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    let identifier_valid = identifier.split(':').all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
    });
    if did.len() > 256 || !method_valid || !identifier_valid {
        return Err("Enter a scoped mobile DID using plain DID characters.".into());
    }
    Ok(())
}

fn normalize_endpoint(endpoint: &str) -> Result<String, String> {
    let (host, port) = endpoint
        .rsplit_once(':')
        .ok_or("Enter a WireGuard endpoint as host:port or [IPv6]:port.")?;
    if !port.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("The WireGuard port must be between 1 and 65535.".into());
    }
    let port = port
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or("The WireGuard port must be between 1 and 65535.")?;
    if let Some(ipv6) = host
        .strip_prefix('[')
        .and_then(|host| host.strip_suffix(']'))
    {
        let address = ipv6
            .parse::<Ipv6Addr>()
            .map_err(|_| "Invalid IPv6 address.")?;
        return Ok(format!("[{address}]:{port}"));
    }
    if host.len() > 253
        || !host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
    {
        return Err("Enter a DNS name, IPv4 address, or bracketed IPv6 address.".into());
    }
    if host
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'.')
        && host.contains('.')
    {
        let address = host
            .parse::<Ipv4Addr>()
            .map_err(|_| "Invalid IPv4 address.")?;
        return Ok(format!("{address}:{port}"));
    }
    Ok(format!("{}:{port}", host.to_ascii_lowercase()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ============================================================================
// PROGRESSIVE DESKTOP & MOBILE ONBOARDING CONFIDENCE LADDER &
// MULTI-ACCOUNT / MULTI-USER SHARED-MACHINE SEAT ASSIGNMENT
// ============================================================================

/// Progressive confidence tier computed from QualiaDB's noisy-OR Enumerated Identity formula
/// (`C_agg = 1.0 - Π_i (1.0 - c_i) < 1.0`, preserving the out-of-band remainder and rejecting `>= 1.0`
/// as `DefinitiveCollapse`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachinePrincipalConfidenceTier {
    /// `< 2` distinct attested identifiers or `C_agg < 0.70` — unverified/ephemeral guest session.
    Tier0UnverifiedGuest,
    /// `>= 2` attested identifiers & `0.70 <= C_agg < 0.88` — single-apparatus local Sanctuary unlocked
    /// (Anvil `.np64` hardware quote + local Argon2id Sanctuary Vault lane).
    Tier1LocalApparatusBound,
    /// `>= 3` attested identifiers & `0.88 <= C_agg < 0.96` — cross-apparatus Desktop + Mobile AVF
    /// (`pKVM` EL2 Pocket Co-Signer) paired & co-attested.
    Tier2DesktopAndMobilePaired,
    /// `>= 4` attested identifiers & `0.96 <= C_agg < 1.0` — high-assurance multi-factor principal
    /// (Desktop Anvil `.np64` + Mobile AVF `pKVM` + Prior-History/QDP Domain/Shamir Guardian + Liveness/Fiduciary VC).
    Tier3HighAssuranceMultiFactorPrincipal,
    /// Rejected fail-closed: an identifier claimed `confidence >= 1.0` (`DefinitiveCollapse`).
    RejectedDefinitiveCollapse,
}

impl MachinePrincipalConfidenceTier {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tier0UnverifiedGuest => "Tier 0 · Unverified / Ephemeral Guest (< 0.70)",
            Self::Tier1LocalApparatusBound => "Tier 1 · Single-Apparatus Local Sanctuary (0.70..0.88)",
            Self::Tier2DesktopAndMobilePaired => {
                "Tier 2 · Desktop + Mobile AVF Pocket Co-Signer Paired (0.88..0.96)"
            }
            Self::Tier3HighAssuranceMultiFactorPrincipal => {
                "Tier 3 · High-Assurance Multi-Factor Principal (0.96..<1.0)"
            }
            Self::RejectedDefinitiveCollapse => {
                "REJECTED · DefinitiveCollapse (confidence >= 1.0 forbidden)"
            }
        }
    }
}

/// Role and persona assignment for a distinct account running on the same physical Desktop or Mobile machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineLocalAccountAssignmentKind {
    /// Primary natural-person steward's personal Sanctuary account (`PersonalSanctuaryPersona`).
    PrimaryNaturalPersonSanctuary { kinship_or_household_role: String },
    /// Separate professional / fiduciary duty-of-care persona account for the same or another
    /// natural person on the machine (e.g., Doctor/AHPRA clinician, Legal Counsel, Parliamentary/Civic office).
    ProfessionalFiduciaryPersonaAccount {
        fiduciary_sector: String,
        qualification_credential_id: String,
        jurisdiction_iso: String,
    },
    /// Adult co-steward or share-house peer account on a shared computer (isolated 42 MiB `SlgArena`
    /// and Argon2id vault; zero cross-account inspection).
    CoStewardOrHousematePeerAccount { relationship_label: String },
    /// Child or supported dependent account on a family desktop/tablet, linked to parental guardian
    /// co-signers and school federation bridges under UN CRC / UN CRPD.
    ChildOrSupportedDependentAccount {
        agency_stage: String,
        guardian_dids: Vec<String>,
        school_bridge_subdomain: Option<String>,
    },
    /// Governed service or delegated AI agent runner account — strictly bound to an identified
    /// `NaturalPerson` principal (`principal_natural_person_did`) and barred from casting human steward votes.
    DelegatedServiceAgentRunnerAccount {
        principal_natural_person_did: String,
        delegation_agreement_id: String,
    },
}

/// One isolated account seat assigned on a Desktop or Mobile machine.
///
/// Enforces the Four-Plane rule: **"OS account is never principal identity."**
/// Multiple accounts can share one OS user login (via separate Webizen profile seats & Argon2id vault paths)
/// or map across distinct OS user accounts (`timothy`, `elena`, `leo`) while keeping their 42 MiB `SlgArena`,
/// local loopback daemon port, and `VaultContainerV2` key material strictly isolated.
#[derive(Clone, Debug, PartialEq)]
pub struct MachineAccountSeatAssignment {
    pub seat_id: String,
    pub os_account_username: String,
    pub webizen_profile_slug: String,
    pub principal_did: String,
    pub display_name: String,
    pub assignment_kind: MachineLocalAccountAssignmentKind,
    pub isolated_vault_rel_path: String,
    pub dedicated_loopback_port: u16,
    pub isolated_slg_arena_mib: u16,
    pub paired_mobile_pocket_cosigner_did: Option<String>,
    pub identifier_bindings: Vec<crate::identifiers::IdentifierBinding>,
    pub noisy_or_aggregate_confidence: f32,
    pub confidence_tier: MachinePrincipalConfidenceTier,
    pub sealed_nquin_parity: u64,
}

/// Registry of all progressive onboarding sessions and multi-account seat assignments on a physical machine.
#[derive(Clone, Debug, PartialEq)]
pub struct MachineOnboardingAndAccountAssignmentRegistry {
    pub machine_hostname: String,
    pub machine_anvil_np64_did: String,
    pub active_seat_id: String,
    pub assigned_seats: Vec<MachineAccountSeatAssignment>,
}

/// Evaluates progressive identifier bindings for an onboarding account using QualiaDB's
/// `validate_enumerated_identity` noisy-OR accumulator (`< 1.0`) and classifies its confidence tier.
pub fn evaluate_onboarding_confidence_tier(
    bindings: &[crate::identifiers::IdentifierBinding],
) -> (f32, MachinePrincipalConfidenceTier) {
    match crate::identifiers::validate_enumerated_identity(bindings, 2, 2) {
        crate::identifiers::IdentityValidation::DefinitiveCollapse => {
            (0.0, MachinePrincipalConfidenceTier::RejectedDefinitiveCollapse)
        }
        crate::identifiers::IdentityValidation::Underdetermined { .. } => {
            (0.35, MachinePrincipalConfidenceTier::Tier0UnverifiedGuest)
        }
        crate::identifiers::IdentityValidation::Valid {
            attested,
            aggregate_confidence,
            ..
        } => {
            let tier = if attested >= 4 && aggregate_confidence >= 0.96 {
                MachinePrincipalConfidenceTier::Tier3HighAssuranceMultiFactorPrincipal
            } else if attested >= 3 && aggregate_confidence >= 0.88 {
                MachinePrincipalConfidenceTier::Tier2DesktopAndMobilePaired
            } else if attested >= 2 && aggregate_confidence >= 0.70 {
                MachinePrincipalConfidenceTier::Tier1LocalApparatusBound
            } else {
                MachinePrincipalConfidenceTier::Tier0UnverifiedGuest
            };
            (aggregate_confidence, tier)
        }
    }
}

/// Builds a seeded multi-account Desktop & Mobile Onboarding Registry demonstrating:
/// 1. How a person onboarding on Desktop + Mobile develops progressive identifiers (Anvil `.np64` hardware,
///    Argon2id Sanctuary Vault lane, Mobile AVF `pKVM` Pocket Co-Signer, QDP Domain + Corroborated Prior History,
///    and Fiduciary AHPRA/Legal credentials) to reach **Tier 3 High-Assurance Multi-Factor Confidence (`~0.994 < 1.0`)**,
/// 2. How multiple accounts on the **same physical machine** (Father's Personal Sanctuary, Mother's Clinical
///    Fiduciary Persona, Child's Guardian-Linked Student Seat, and a Delegated Agent Runner) are assigned
///    isolated vault paths, dedicated ports (`8080..8083`), and 42 MiB `SlgArena` ceilings without ever
///    collapsing OS login names into human identity.
pub fn build_seeded_machine_onboarding_and_account_assignments(
    hostname: &str,
    anvil_np64_did: &str,
) -> MachineOnboardingAndAccountAssignmentRegistry {
    use crate::identifiers::{CryptoScheme, IdentifierBinding};

    let make_seat = |seat_id: &str,
                     os_user: &str,
                     profile_slug: &str,
                     principal_did: &str,
                     display_name: &str,
                     assignment_kind: MachineLocalAccountAssignmentKind,
                     port: u16,
                     mobile_did: Option<&str>,
                     bindings: Vec<IdentifierBinding>|
     -> MachineAccountSeatAssignment {
        let (conf, tier) = evaluate_onboarding_confidence_tier(&bindings);
        let quin = seal(
            &format!("urn:webizen:onboarding:seat:{seat_id}"),
            "https://webizen.org/q42#machineAccountSeatAssigned",
            q_hash_60(&format!("{principal_did}|{profile_slug}|{port}|{conf:.4}")),
        );
        MachineAccountSeatAssignment {
            seat_id: seat_id.to_string(),
            os_account_username: os_user.to_string(),
            webizen_profile_slug: profile_slug.to_string(),
            principal_did: principal_did.to_string(),
            display_name: display_name.to_string(),
            assignment_kind,
            isolated_vault_rel_path: format!("sanctuary/seats/{profile_slug}/sanctuary_vault.cbor"),
            dedicated_loopback_port: port,
            isolated_slg_arena_mib: 42,
            paired_mobile_pocket_cosigner_did: mobile_did.map(str::to_string),
            identifier_bindings: bindings,
            noisy_or_aggregate_confidence: conf,
            confidence_tier: tier,
            sealed_nquin_parity: quin.parity(),
        }
    };

    // Seat 1: Alex Mercer (Primary Personal Sanctuary — 5 distinct attested identifiers across Desktop + Mobile AVF)
    let alex_bindings = vec![
        IdentifierBinding {
            identifier: q_hash_60(anvil_np64_did),
            uri: anvil_np64_did.to_string(),
            plane: "instrument",
            role: "desktop-anvil-np64-tpm",
            scheme: CryptoScheme::Ed25519,
            attested: true,
            confidence: 0.62,
        },
        IdentifierBinding {
            identifier: q_hash_60("did:q42:locus:alex-sanctuary-argon2id"),
            uri: "did:q42:locus:alex-sanctuary-argon2id".to_string(),
            plane: "instrument",
            role: "argon2id-vault-lane-verifier",
            scheme: CryptoScheme::Blake3Commitment,
            attested: true,
            confidence: 0.72,
        },
        IdentifierBinding {
            identifier: q_hash_60("did:webizen:mobile-avf:alex-pixel9-pkvm"),
            uri: "did:webizen:mobile-avf:alex-pixel9-pkvm".to_string(),
            plane: "instrument",
            role: "mobile-avf-pkvm-el2-pocket-cosigner",
            scheme: CryptoScheme::MlDsa65,
            attested: true,
            confidence: 0.78,
        },
        IdentifierBinding {
            identifier: q_hash_60("qdp://alex.mercer-homestead.au#prior-history-corroborated"),
            uri: "qdp://alex.mercer-homestead.au#prior-history-corroborated".to_string(),
            plane: "claim",
            role: "qdp-domain-and-corroborated-prior-history",
            scheme: CryptoScheme::Ed25519,
            attested: true,
            confidence: 0.70,
        },
        IdentifierBinding {
            identifier: q_hash_60("urn:webizen:shamir:alex-2of4-household-quorum"),
            uri: "urn:webizen:shamir:alex-2of4-household-quorum".to_string(),
            plane: "claim",
            role: "shamir-guardian-social-co-attestation",
            scheme: CryptoScheme::MlDsa65,
            attested: true,
            confidence: 0.65,
        },
    ];

    // Seat 2: Dr. Elena Rostova-Mercer — Professional Fiduciary Clinical Account on the same workstation
    // (shares or switches OS user, but runs in a strictly isolated fiduciary vault & port :8081)
    let elena_fiduciary_bindings = vec![
        IdentifierBinding {
            identifier: q_hash_60(anvil_np64_did),
            uri: anvil_np64_did.to_string(),
            plane: "instrument",
            role: "desktop-anvil-np64-tpm",
            scheme: CryptoScheme::Ed25519,
            attested: true,
            confidence: 0.62,
        },
        IdentifierBinding {
            identifier: q_hash_60("did:q42:locus:elena-clinical-fiduciary-argon2id"),
            uri: "did:q42:locus:elena-clinical-fiduciary-argon2id".to_string(),
            plane: "instrument",
            role: "argon2id-fiduciary-vault-lane",
            scheme: CryptoScheme::Blake3Commitment,
            attested: true,
            confidence: 0.74,
        },
        IdentifierBinding {
            identifier: q_hash_60("did:webizen:mobile-avf:elena-pocket-cosigner"),
            uri: "did:webizen:mobile-avf:elena-pocket-cosigner".to_string(),
            plane: "instrument",
            role: "mobile-avf-pkvm-el2-pocket-cosigner",
            scheme: CryptoScheme::MlDsa65,
            attested: true,
            confidence: 0.80,
        },
        IdentifierBinding {
            identifier: q_hash_60("urn:ahpra:credential:MED0001849201"),
            uri: "urn:ahpra:credential:MED0001849201".to_string(),
            plane: "claim",
            role: "fiduciary-qualification-ahpra-specialist",
            scheme: CryptoScheme::MlDsa65,
            attested: true,
            confidence: 0.76,
        },
    ];

    // Seat 3: Leo Mercer (Child / Year 6 Student Seat on the same family desktop — Tier 2 paired with parental guardian quorum)
    let leo_child_bindings = vec![
        IdentifierBinding {
            identifier: q_hash_60(anvil_np64_did),
            uri: anvil_np64_did.to_string(),
            plane: "instrument",
            role: "desktop-anvil-np64-tpm",
            scheme: CryptoScheme::Ed25519,
            attested: true,
            confidence: 0.60,
        },
        IdentifierBinding {
            identifier: q_hash_60("did:q42:locus:leo-student-sanctuary-lane"),
            uri: "did:q42:locus:leo-student-sanctuary-lane".to_string(),
            plane: "instrument",
            role: "argon2id-child-vault-lane",
            scheme: CryptoScheme::Blake3Commitment,
            attested: true,
            confidence: 0.68,
        },
        IdentifierBinding {
            identifier: q_hash_60("urn:webizen:guardianship:leo-parental-co-attestation"),
            uri: "urn:webizen:guardianship:leo-parental-co-attestation".to_string(),
            plane: "claim",
            role: "parental-guardian-cosigner-quorum",
            scheme: CryptoScheme::MlDsa65,
            attested: true,
            confidence: 0.75,
        },
    ];

    // Seat 4: Delegated AI / Service Agent Runner Account on the machine (explicitly bound to Alex Mercer's NaturalPerson DID)
    let agent_runner_bindings = vec![
        IdentifierBinding {
            identifier: q_hash_60(anvil_np64_did),
            uri: anvil_np64_did.to_string(),
            plane: "instrument",
            role: "desktop-anvil-np64-tpm",
            scheme: CryptoScheme::Ed25519,
            attested: true,
            confidence: 0.55,
        },
        IdentifierBinding {
            identifier: q_hash_60("did:qualia:subagent:alex-mercer:forge-assistant"),
            uri: "did:qualia:subagent:alex-mercer:forge-assistant".to_string(),
            plane: "instrument",
            role: "delegated-subagent-zcap-instrument",
            scheme: CryptoScheme::Ed25519,
            attested: true,
            confidence: 0.65,
        },
    ];

    let seats = vec![
        make_seat(
            "MACHINE-SEAT-01-ALEX-PERSONAL",
            "alex",
            "alex-personal-sanctuary",
            "did:webizen:person:alex-mercer",
            "Alex Mercer (Primary Personal Sanctuary & Homestead Steward)",
            MachineLocalAccountAssignmentKind::PrimaryNaturalPersonSanctuary {
                kinship_or_household_role: "Father & Primary Workstation Steward".to_string(),
            },
            8080,
            Some("did:webizen:mobile-avf:alex-pixel9-pkvm"),
            alex_bindings,
        ),
        make_seat(
            "MACHINE-SEAT-02-ELENA-FIDUCIARY",
            "elena",
            "elena-clinical-fiduciary",
            "did:webizen:person:dr-elena-rostova",
            "Dr. Elena Rostova-Mercer (AHPRA Clinical Psychiatrist Fiduciary Account)",
            MachineLocalAccountAssignmentKind::ProfessionalFiduciaryPersonaAccount {
                fiduciary_sector: "HealthcareAndMedical".to_string(),
                qualification_credential_id: "AHPRA-MED0001849201".to_string(),
                jurisdiction_iso: "AU-VIC".to_string(),
            },
            8081,
            Some("did:webizen:mobile-avf:elena-pocket-cosigner"),
            elena_fiduciary_bindings,
        ),
        make_seat(
            "MACHINE-SEAT-03-LEO-STUDENT",
            "leo",
            "leo-student-commons",
            "did:webizen:person:leo-mercer",
            "Leo Mercer (Child · Year 6 Student Seat with Guardian Co-Signers)",
            MachineLocalAccountAssignmentKind::ChildOrSupportedDependentAccount {
                agency_stage: "AgencyStage::Child (UN CRC Evolving Capacity)".to_string(),
                guardian_dids: vec![
                    "did:webizen:person:alex-mercer".to_string(),
                    "did:webizen:person:dr-elena-rostova".to_string(),
                ],
                school_bridge_subdomain: Some("school.leo.mercer-homestead.au".to_string()),
            },
            8082,
            Some("did:webizen:mobile-avf:alex-pixel9-pkvm"),
            leo_child_bindings,
        ),
        make_seat(
            "MACHINE-SEAT-04-DELEGATED-AGENT",
            "alex",
            "alex-delegated-forge-agent",
            "did:qualia:subagent:alex-mercer:forge-assistant",
            "Alex's Delegated CAD & Ledger Sub-Agent (Non-Voting Instrument Seat)",
            MachineLocalAccountAssignmentKind::DelegatedServiceAgentRunnerAccount {
                principal_natural_person_did: "did:webizen:person:alex-mercer".to_string(),
                delegation_agreement_id: "AGR-AGENT-DELEGATION-ALEX".to_string(),
            },
            8083,
            None,
            agent_runner_bindings,
        ),
    ];

    MachineOnboardingAndAccountAssignmentRegistry {
        machine_hostname: hostname.to_string(),
        machine_anvil_np64_did: anvil_np64_did.to_string(),
        active_seat_id: "MACHINE-SEAT-01-ALEX-PERSONAL".to_string(),
        assigned_seats: seats,
    }
}

/// Ordered 7-Screen Interactive New-User Onboarding Wizard for Desktop & Mobile users.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewUserOnboardingScreenStep {
    /// Screen 1: Hardware Apparatus & Substrate Probe (Anvil `.np64` GPU/TPM/NPU quote + Windows/WSL2 Kernel Bridge vs. Debian 13 Trixie Primary OS).
    Screen1HardwareAnvilAndWsl2Probe,
    /// Screen 2: Four-Plane Identifier Fabric & Digital Birth / Principal Anchor (`NaturalAgent` living-SHACL, strictly disjoint from `owl:Thing`, `"OS account != identity"`).
    Screen2FourPlaneNaturalPersonAnchor,
    /// Screen 3: 4-Slot `VaultContainerV2` Sanctuary Keep & Anti-Coercion Setup (Argon2id 64 MiB Real PIN vs. Plausible-Deniability Duress Decoy PIN; zero Nuke-PIN).
    Screen3SanctuaryDualPinAndVaultSetup,
    /// Screen 4: Mobile AVF (`pKVM` EL2 `arm64`) Pocket Co-Signer Pairing & Step-Up Permission Elevator (`WireGuard` + `vhost-vsock` CID 2 + camera rPPG/PAD liveness option).
    Screen4MobileAvfPocketCosignerPairing,
    /// Screen 5: Multi-Account Shared-Machine Seat Assignment & Fiduciary/Family Personas (Personal Sanctuary `:8080`, Professional Fiduciary Clinical/Legal `:8081`, Child Student `:8082`, Delegated Agent `:8083`).
    Screen5MultiAccountSharedMachineSeats,
    /// Screen 6: Always-On Persistence Mesh, Family/Village Homestead VPS & Australian ABN / Cost-Recovery Setup (Mac Mini `:8080` / Cloud VPS `:4242` store-and-forward + ATO Mod-89 ABN or Statement by Supplier + `M-of-N` Shamir `.q42j` guardians).
    Screen6PersistenceMeshVpsAndShamirGuardians,
    /// Screen 7: Enumerated-Identity Confidence Readiness Summary & Activation Receipt (verifying Noisy-OR $C_{\text{agg}} < 1.0$ across Tiers 1–3 and sealing the 48-byte `NQuin48` onboarding receipt).
    Screen7ConfidenceSummaryAndActivation,
}

impl NewUserOnboardingScreenStep {
    pub const ALL: [Self; 7] = [
        Self::Screen1HardwareAnvilAndWsl2Probe,
        Self::Screen2FourPlaneNaturalPersonAnchor,
        Self::Screen3SanctuaryDualPinAndVaultSetup,
        Self::Screen4MobileAvfPocketCosignerPairing,
        Self::Screen5MultiAccountSharedMachineSeats,
        Self::Screen6PersistenceMeshVpsAndShamirGuardians,
        Self::Screen7ConfidenceSummaryAndActivation,
    ];

    pub const fn step_number(self) -> u8 {
        match self {
            Self::Screen1HardwareAnvilAndWsl2Probe => 1,
            Self::Screen2FourPlaneNaturalPersonAnchor => 2,
            Self::Screen3SanctuaryDualPinAndVaultSetup => 3,
            Self::Screen4MobileAvfPocketCosignerPairing => 4,
            Self::Screen5MultiAccountSharedMachineSeats => 5,
            Self::Screen6PersistenceMeshVpsAndShamirGuardians => 6,
            Self::Screen7ConfidenceSummaryAndActivation => 7,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Screen1HardwareAnvilAndWsl2Probe => {
                "Screen 1 · Apparatus Hardware (Anvil .np64) & WSL2 / Debian Primary OS Substrate"
            }
            Self::Screen2FourPlaneNaturalPersonAnchor => {
                "Screen 2 · Four-Plane Natural-Person Principal Anchor (Living-SHACL · OS Account != Identity)"
            }
            Self::Screen3SanctuaryDualPinAndVaultSetup => {
                "Screen 3 · 4-Slot VaultContainerV2 & Dual-PIN Anti-Coercion Sanctuary Keep"
            }
            Self::Screen4MobileAvfPocketCosignerPairing => {
                "Screen 4 · Mobile AVF (pKVM EL2) Pocket Co-Signer & Permission Elevator Pairing"
            }
            Self::Screen5MultiAccountSharedMachineSeats => {
                "Screen 5 · Multi-Account Machine Seat Assignments (Personal, Fiduciary, Child & Delegated Agent)"
            }
            Self::Screen6PersistenceMeshVpsAndShamirGuardians => {
                "Screen 6 · Always-On Persistence Mesh (Mac Mini / VPS), Australian ABN & M-of-N Shamir Guardians"
            }
            Self::Screen7ConfidenceSummaryAndActivation => {
                "Screen 7 · Progressive Noisy-OR Confidence Ladder (< 1.0) & Sanctuary Activation Receipt"
            }
        }
    }

    pub const fn objective_summary(self) -> &'static str {
        match self {
            Self::Screen1HardwareAnvilAndWsl2Probe => {
                "Attests local GPU/TPM/IOMMU hardware via 64-byte Anvil NodeProfile64 (.np64) and detects whether to bridge Linux eBPF/nftables/ext4 via WSL2 or run as Debian 13 Trixie Primary OS."
            }
            Self::Screen2FourPlaneNaturalPersonAnchor => {
                "Establishes the natural person on Plane 1 (NaturalAgent, disjoint from owl:Thing) and separates Who-Plane identity from Handle-Plane domains and underlying Windows/Linux user accounts."
            }
            Self::Screen3SanctuaryDualPinAndVaultSetup => {
                "Initializes the constant-shape 4-slot CBOR vault (Real Sanctuary Keep + Plausible-Deniability Duress Decoy lane) with memory-hard Argon2id (64 MiB) and zero destructive Nuke-PIN."
            }
            Self::Screen4MobileAvfPocketCosignerPairing => {
                "Pairs the user's phone install (Android 16 AVF pKVM EL2 / Mobian arm64) over WireGuard + vhost-vsock CID 2 so the phone can elevate permissions while the laptop is asleep."
            }
            Self::Screen5MultiAccountSharedMachineSeats => {
                "Assigns isolated loopback ports (:8080..:8083), 42 MiB SlgArena ceilings, and separate Argon2id vaults for Personal, Professional Fiduciary (Doctor/Legal/Civic), Child Student, and Delegated Agent accounts on this machine."
            }
            Self::Screen6PersistenceMeshVpsAndShamirGuardians => {
                "Links the always-on persistence layer (Mac Mini Home Server :8080 / Cloud VPS :4242 with ATO Mod-89 ABN or cost-recovery statement) and distributes 2-of-4 GF(256) Shamir .q42j recovery shares."
            }
            Self::Screen7ConfidenceSummaryAndActivation => {
                "Evaluates all developed identifier bindings through QualiaDB's noisy-OR accumulator (C_agg < 1.0, rejecting DefinitiveCollapse >= 1.0) to unlock Tier 3 High-Assurance operation."
            }
        }
    }

    pub const fn identifier_developed_on_screen(self) -> &'static str {
        match self {
            Self::Screen1HardwareAnvilAndWsl2Probe => {
                "Plane 4 Instrument: desktop-anvil-np64-tpm (Ed25519, c = 0.62)"
            }
            Self::Screen2FourPlaneNaturalPersonAnchor => {
                "Plane 1 NaturalAgent: contextual pairwise root anchor (Master collapse refused)"
            }
            Self::Screen3SanctuaryDualPinAndVaultSetup => {
                "Plane 4 Instrument: argon2id-vault-lane-verifier (BLAKE3-Commitment, c = 0.72 -> Tier 1 LocalApparatusBound 0.894)"
            }
            Self::Screen4MobileAvfPocketCosignerPairing => {
                "Plane 4 Instrument: mobile-avf-pkvm-el2-pocket-cosigner (ML-DSA-65, c = 0.78 -> Tier 2 DesktopAndMobilePaired 0.977)"
            }
            Self::Screen5MultiAccountSharedMachineSeats => {
                "Plane 2 Claim: fiduciary-qualification-ahpra-specialist / parental-guardian-cosigner-quorum (ML-DSA-65, c = 0.75..0.76)"
            }
            Self::Screen6PersistenceMeshVpsAndShamirGuardians => {
                "Plane 2 Claim: qdp-domain-and-corroborated-prior-history + shamir-2of4-quorum (Ed25519 + ML-DSA-65, c = 0.65..0.70)"
            }
            Self::Screen7ConfidenceSummaryAndActivation => {
                "Aggregate Enumerated Identity State: 5 attested bindings -> Noisy-OR C_agg = 0.997 (< 1.0 remainder preserved -> Tier 3 HighAssuranceMultiFactorPrincipal)"
            }
        }
    }
}

// ============================================================================
// CIVICS.AU — AUSTRALIAN HUMAN-RIGHTS & CIVICS STATISTICAL OBSERVATORY,
// ZERO-IDENTIFIER ("ZERO-WHO-LEAK") VERIFIABLE TESTIMONY & PARLIAMENTARY ENGAGEMENT
// ============================================================================

/// Australian Federal, State, and Territory jurisdictional strata hosted on `civics.au`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AustralianCivicsJurisdiction {
    CommonwealthOfAustraliaFederal,
    Victoria,
    NewSouthWales,
    Queensland,
    AustralianCapitalTerritory,
    WesternAustralia,
    SouthAustralia,
    Tasmania,
    NorthernTerritory,
}

impl AustralianCivicsJurisdiction {
    pub const fn iso_code(self) -> &'static str {
        match self {
            Self::CommonwealthOfAustraliaFederal => "AU-CTH",
            Self::Victoria => "AU-VIC",
            Self::NewSouthWales => "AU-NSW",
            Self::Queensland => "AU-QLD",
            Self::AustralianCapitalTerritory => "AU-ACT",
            Self::WesternAustralia => "AU-WA",
            Self::SouthAustralia => "AU-SA",
            Self::Tasmania => "AU-TAS",
            Self::NorthernTerritory => "AU-NT",
        }
    }
}

/// Core Human-Rights & Civics Issue Domains tracked on `civics.au`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HumanRightsCivicsIssueDomain {
    HousingSecurityAndHomelessness,
    HealthcareAndContestedDiagnosisAutonomy,
    SocialSecurityNdisAndAutomatedWelfareFairness,
    ChildEducationAndAgedCareGuardianship,
    WhistleblowerAndDutyOfInquiryAccountability,
    EconomicDignityAndCooperativeLivelihood,
}

impl HumanRightsCivicsIssueDomain {
    pub const fn code(self) -> &'static str {
        match self {
            Self::HousingSecurityAndHomelessness => "HR-CIVICS-HOUSING-ICESCR11",
            Self::HealthcareAndContestedDiagnosisAutonomy => "HR-CIVICS-HEALTH-CRPD12",
            Self::SocialSecurityNdisAndAutomatedWelfareFairness => "HR-CIVICS-WELFARE-ICESCR9",
            Self::ChildEducationAndAgedCareGuardianship => "HR-CIVICS-GUARDIANSHIP-CRC-CRPD",
            Self::WhistleblowerAndDutyOfInquiryAccountability => "HR-CIVICS-INQUIRY-ICCPR19",
            Self::EconomicDignityAndCooperativeLivelihood => "HR-CIVICS-LIVELIHOOD-UDHR27",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::HousingSecurityAndHomelessness => {
                "Housing Adequacy, Rental Safety & Homelessness Prevention (ICESCR Art. 11)"
            }
            Self::HealthcareAndContestedDiagnosisAutonomy => {
                "Healthcare Access, Contested Diagnoses & Mental-Health Autonomy (ICESCR Art. 12 / CRPD Art. 12)"
            }
            Self::SocialSecurityNdisAndAutomatedWelfareFairness => {
                "Social Security, NDIS & Automated Welfare Decision Fairness (ICESCR Art. 9 / CRPD Art. 28)"
            }
            Self::ChildEducationAndAgedCareGuardianship => {
                "Child Protection, School Equity & Aged-Care Supported Agency (UN CRC / UN CRPD)"
            }
            Self::WhistleblowerAndDutyOfInquiryAccountability => {
                "Duty of Inquiry, Public-Interest Disclosure & Anti-Retaliation (ICCPR Art. 19 / PID Act)"
            }
            Self::EconomicDignityAndCooperativeLivelihood => {
                "Economic Dignity, Creator Moral Interests & Village/Cooperative Livelihood (UDHR Art. 23 & 27)"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroIdentityVerifiableTruthfulnessProof {
    pub epoch_issue_nullifier_hex: String,
    pub apparatus_confidence_tier_verified_permille: u16,
    pub electorate_and_jurisdiction_membership_zk_commitment_hex: String,
    pub corroborating_evidence_act_anchor_commitment_hex: String,
    pub corroborated_primary_artifact_count: u8,
    pub statutory_truthfulness_undertaking_hash_hex: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnonymisedVerifiableConstituentTestimony {
    pub testimony_id: String,
    pub jurisdiction: AustralianCivicsJurisdiction,
    pub federal_electorate_division: String,
    pub state_or_lga_region: String,
    pub issue_domain: HumanRightsCivicsIssueDomain,
    pub impact_severity_score: u8,
    pub systemic_delay_or_unresolved_days: u32,
    pub deidentified_lived_experience_summary: String,
    pub proposed_policy_or_remedy_view: String,
    pub truthfulness_proof: ZeroIdentityVerifiableTruthfulnessProof,
    pub sealed_nquin_parity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CivicsAuAnonymisedStatisticalSeries {
    pub series_id: String,
    pub hosted_endpoint_uri: String,
    pub jurisdiction: AustralianCivicsJurisdiction,
    pub electorate_or_region_scope: String,
    pub issue_domain: HumanRightsCivicsIssueDomain,
    pub verified_unique_constituent_count: u32,
    pub k_anonymity_threshold_satisfied: bool,
    pub published_mean_severity_score: Option<u8>,
    pub published_median_unresolved_days: Option<u32>,
    pub corroborated_evidence_act_backing_pct: Option<u8>,
    pub top_constituent_remedy_consensus: String,
    pub sealed_nquin_parity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParliamentaryConstituentEngagementThread {
    pub engagement_id: String,
    pub civics_au_thread_uri: String,
    pub jurisdiction: AustralianCivicsJurisdiction,
    pub electorate_or_senate_portfolio: String,
    pub parliamentarian_fiduciary_did: String,
    pub parliamentarian_name_and_title: String,
    pub parliamentary_register_credential_id: String,
    pub issue_domain: HumanRightsCivicsIssueDomain,
    pub linked_statistical_series_id: String,
    pub parliamentary_response_and_action_commitment: String,
    pub hansard_or_committee_reference: String,
    pub constituent_nullifier_blind_reply_tokens: Vec<String>,
    pub duty_of_inquiry_standing: String,
    pub sealed_nquin_parity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CivicsAuHumanRightsObservatory {
    pub primary_domain: String,
    pub observatory_endpoints: Vec<String>,
    pub testimonies: Vec<AnonymisedVerifiableConstituentTestimony>,
    pub statistical_series: Vec<CivicsAuAnonymisedStatisticalSeries>,
    pub parliamentary_engagements: Vec<ParliamentaryConstituentEngagementThread>,
}

pub fn mint_zero_identity_verifiable_truthfulness_proof(
    local_sanctuary_secret_seed: &str,
    issue_code: &str,
    epoch_ym: &str,
    jurisdiction: AustralianCivicsJurisdiction,
    federal_electorate_division: &str,
    apparatus_confidence_permille: u16,
    corroborating_qar_and_evidence_act_digests: &[&str],
    deidentified_summary: &str,
) -> Result<ZeroIdentityVerifiableTruthfulnessProof, String> {
    validate_human_centric_host_nomenclature(deidentified_summary)?;
    if !(880..1000).contains(&apparatus_confidence_permille) {
        return Err(
            "Apparatus confidence must be in 880..999 permille (Tier 2/3 Desktop+Mobile paired, < 1.0 DefinitiveCollapse bound)"
                .into(),
        );
    }
    if corroborating_qar_and_evidence_act_digests.is_empty() {
        return Err(
            "Anti-False-Statement Gate: testimony must commit to at least one corroborating Sanctuary / Evidence Act (.qar / .q42) primary record digest"
                .into(),
        );
    }
    let lower = deidentified_summary.to_ascii_lowercase();
    if lower.contains("did:qi:") || lower.contains("did:webizen:person:") || lower.contains('@') {
        return Err(
            "Zero-Identifier Privacy Gate: de-identified testimony summary must not contain a personal DID or email address"
                .into(),
        );
    }

    let hash_hex32 = |label: &str| -> String {
        let mut h = Sha3_256::new();
        h.update(label.as_bytes());
        hex(&h.finalize()[..16])
    };

    let epoch_issue_nullifier_hex = hash_hex32(&format!(
        "civics.au:nullifier:v1|{local_sanctuary_secret_seed}|{issue_code}|{epoch_ym}"
    ));
    let electorate_and_jurisdiction_membership_zk_commitment_hex = hash_hex32(&format!(
        "civics.au:electorate-zk:v1|{local_sanctuary_secret_seed}|{}|{federal_electorate_division}",
        jurisdiction.iso_code()
    ));
    let corroborating_evidence_act_anchor_commitment_hex = hash_hex32(&format!(
        "civics.au:evidence-act-anchor:v1|{epoch_issue_nullifier_hex}|{}",
        corroborating_qar_and_evidence_act_digests.join("|")
    ));
    let statutory_truthfulness_undertaking_hash_hex = hash_hex32(&format!(
        "civics.au:statutory-truth-undertaking:v1|{epoch_issue_nullifier_hex}|{corroborating_evidence_act_anchor_commitment_hex}|{deidentified_summary}"
    ));

    Ok(ZeroIdentityVerifiableTruthfulnessProof {
        epoch_issue_nullifier_hex,
        apparatus_confidence_tier_verified_permille: apparatus_confidence_permille,
        electorate_and_jurisdiction_membership_zk_commitment_hex,
        corroborating_evidence_act_anchor_commitment_hex,
        corroborated_primary_artifact_count: corroborating_qar_and_evidence_act_digests.len() as u8,
        statutory_truthfulness_undertaking_hash_hex,
    })
}

pub fn build_seeded_civics_au_human_rights_observatory() -> CivicsAuHumanRightsObservatory {
    let p1 = mint_zero_identity_verifiable_truthfulness_proof(
        "sanctuary_secret_seed_constituent_alpha",
        HumanRightsCivicsIssueDomain::SocialSecurityNdisAndAutomatedWelfareFairness.code(),
        "2026-11",
        AustralianCivicsJurisdiction::Victoria,
        "Melbourne",
        997,
        &[
            "qar://evidence-act-s69/agency-determination-letter-2026-04.qar",
            "qar://evidence-act-s146/independent-specialist-functional-assessment.qar",
            "q42://conduct-ledger/duty-of-inquiry-omission-receipt-01",
        ],
        "Automated functional capacity algorithm reduced disability support plan by 44% without reading specialist occupational therapy report; internal review unacknowledged for 165 days.",
    )
    .expect("p1");

    let p2 = mint_zero_identity_verifiable_truthfulness_proof(
        "sanctuary_secret_seed_constituent_beta",
        HumanRightsCivicsIssueDomain::HealthcareAndContestedDiagnosisAutonomy.code(),
        "2026-11",
        AustralianCivicsJurisdiction::Victoria,
        "Melbourne",
        995,
        &[
            "qar://evidence-act-s48/contested-diagnosis-clinical-dossier.qar",
            "qar://evidence-act-s161/rppg-autonomic-pharmacokinetic-log.qar",
        ],
        "Institutional discharge summary carried an unverified historical diagnosis label despite two subsequent specialist clearances, blocking access to trauma-informed physical healthcare.",
    )
    .expect("p2");

    let p3 = mint_zero_identity_verifiable_truthfulness_proof(
        "sanctuary_secret_seed_constituent_gamma",
        HumanRightsCivicsIssueDomain::HousingSecurityAndHomelessness.code(),
        "2026-11",
        AustralianCivicsJurisdiction::CommonwealthOfAustraliaFederal,
        "Sydney",
        968,
        &[
            "qar://evidence-act-s146/environmental-hygrometer-mould-inspection.qar",
            "qar://evidence-act-s69/no-grounds-termination-notice-14d-after-repair-request.qar",
        ],
        "Received no-grounds termination notice 11 days after submitting certified indoor air-quality and structural dampness report requesting urgent roof flashing repair.",
    )
    .expect("p3");

    let q1 = seal(
        "urn:civics.au:testimony:TESTIMONY-AU-VIC-2026-001",
        "https://webizen.org/q42#verifiableAnonymisedTestimony",
        q_hash_60(&p1.epoch_issue_nullifier_hex),
    );
    let q2 = seal(
        "urn:civics.au:testimony:TESTIMONY-AU-VIC-2026-002",
        "https://webizen.org/q42#verifiableAnonymisedTestimony",
        q_hash_60(&p2.epoch_issue_nullifier_hex),
    );
    let q3 = seal(
        "urn:civics.au:testimony:TESTIMONY-AU-CTH-2026-003",
        "https://webizen.org/q42#verifiableAnonymisedTestimony",
        q_hash_60(&p3.epoch_issue_nullifier_hex),
    );

    let blind_1 = format!(
        "blind-receipt:{}:{}",
        &p1.epoch_issue_nullifier_hex[..12],
        &p1.corroborating_evidence_act_anchor_commitment_hex[..12]
    );
    let blind_2 = format!(
        "blind-receipt:{}:{}",
        &p2.epoch_issue_nullifier_hex[..12],
        &p2.corroborating_evidence_act_anchor_commitment_hex[..12]
    );

    let testimonies = vec![
        AnonymisedVerifiableConstituentTestimony {
            testimony_id: "TESTIMONY-AU-VIC-2026-001".into(),
            jurisdiction: AustralianCivicsJurisdiction::Victoria,
            federal_electorate_division: "Melbourne".into(),
            state_or_lga_region: "City of Melbourne / Inner North LGA".into(),
            issue_domain: HumanRightsCivicsIssueDomain::SocialSecurityNdisAndAutomatedWelfareFairness,
            impact_severity_score: 88,
            systemic_delay_or_unresolved_days: 165,
            deidentified_lived_experience_summary: "Automated functional capacity algorithm reduced disability support plan by 44% without reading specialist occupational therapy report; internal review unacknowledged for 165 days.".into(),
            proposed_policy_or_remedy_view: "Mandate human clinician Duty-of-Inquiry sign-off (Six Vectors of Transparency) before any automated algorithm can reduce an NDIS or DSP support plan.".into(),
            truthfulness_proof: p1,
            sealed_nquin_parity: q1.parity(),
        },
        AnonymisedVerifiableConstituentTestimony {
            testimony_id: "TESTIMONY-AU-VIC-2026-002".into(),
            jurisdiction: AustralianCivicsJurisdiction::Victoria,
            federal_electorate_division: "Melbourne".into(),
            state_or_lga_region: "City of Yarra / Melbourne Metro".into(),
            issue_domain: HumanRightsCivicsIssueDomain::HealthcareAndContestedDiagnosisAutonomy,
            impact_severity_score: 84,
            systemic_delay_or_unresolved_days: 210,
            deidentified_lived_experience_summary: "Institutional discharge summary carried an unverified historical diagnosis label despite two subsequent specialist clearances, blocking access to trauma-informed physical healthcare.".into(),
            proposed_policy_or_remedy_view: "Recognise patient-stewarded Contested Diagnosis & Corroborated Clinical Evidence dossiers under the Victorian Charter s 10 & 21 and Health Records Act.".into(),
            truthfulness_proof: p2,
            sealed_nquin_parity: q2.parity(),
        },
        AnonymisedVerifiableConstituentTestimony {
            testimony_id: "TESTIMONY-AU-CTH-2026-003".into(),
            jurisdiction: AustralianCivicsJurisdiction::CommonwealthOfAustraliaFederal,
            federal_electorate_division: "Sydney".into(),
            state_or_lga_region: "Greater Sydney Metro".into(),
            issue_domain: HumanRightsCivicsIssueDomain::HousingSecurityAndHomelessness,
            impact_severity_score: 91,
            systemic_delay_or_unresolved_days: 94,
            deidentified_lived_experience_summary: "Received no-grounds termination notice 11 days after submitting certified indoor air-quality and structural dampness report requesting urgent roof flashing repair.".into(),
            proposed_policy_or_remedy_view: "Enforce national minimum rental habitability standards and reverse-onus protection against retaliatory eviction within 12 months of a documented repair request.".into(),
            truthfulness_proof: p3,
            sealed_nquin_parity: q3.parity(),
        },
    ];

    let statistical_series = vec![
        CivicsAuAnonymisedStatisticalSeries {
            series_id: "STATS-AU-VIC-WELFARE-2026Q4".into(),
            hosted_endpoint_uri: "https://observatory.civics.au/stats/au-vic/melbourne/hr-civics-welfare-icescr9".into(),
            jurisdiction: AustralianCivicsJurisdiction::Victoria,
            electorate_or_region_scope: "Melbourne Federal Division & Metro Victoria".into(),
            issue_domain: HumanRightsCivicsIssueDomain::SocialSecurityNdisAndAutomatedWelfareFairness,
            verified_unique_constituent_count: 149,
            k_anonymity_threshold_satisfied: true,
            published_mean_severity_score: Some(88),
            published_median_unresolved_days: Some(165),
            corroborated_evidence_act_backing_pct: Some(100),
            top_constituent_remedy_consensus: "Mandate human clinician Duty-of-Inquiry sign-off (Six Vectors of Transparency) before any automated algorithm can reduce an NDIS or DSP support plan.".into(),
            sealed_nquin_parity: q1.parity(),
        },
        CivicsAuAnonymisedStatisticalSeries {
            series_id: "STATS-AU-VIC-HEALTH-2026Q4".into(),
            hosted_endpoint_uri: "https://observatory.civics.au/stats/au-vic/victoria-statewide/hr-civics-health-crpd12".into(),
            jurisdiction: AustralianCivicsJurisdiction::Victoria,
            electorate_or_region_scope: "Victoria State-Wide Charter s10/s21 Cohort".into(),
            issue_domain: HumanRightsCivicsIssueDomain::HealthcareAndContestedDiagnosisAutonomy,
            verified_unique_constituent_count: 93,
            k_anonymity_threshold_satisfied: true,
            published_mean_severity_score: Some(84),
            published_median_unresolved_days: Some(210),
            corroborated_evidence_act_backing_pct: Some(100),
            top_constituent_remedy_consensus: "Recognise patient-stewarded Contested Diagnosis & Corroborated Clinical Evidence dossiers under the Victorian Charter s 10 & 21 and Health Records Act.".into(),
            sealed_nquin_parity: q2.parity(),
        },
        CivicsAuAnonymisedStatisticalSeries {
            series_id: "STATS-AU-CTH-HOUSING-2026Q4".into(),
            hosted_endpoint_uri: "https://observatory.civics.au/stats/au-cth/national/hr-civics-housing-icescr11".into(),
            jurisdiction: AustralianCivicsJurisdiction::CommonwealthOfAustraliaFederal,
            electorate_or_region_scope: "Commonwealth National Housing & Rental Safety Cohort".into(),
            issue_domain: HumanRightsCivicsIssueDomain::HousingSecurityAndHomelessness,
            verified_unique_constituent_count: 413,
            k_anonymity_threshold_satisfied: true,
            published_mean_severity_score: Some(91),
            published_median_unresolved_days: Some(94),
            corroborated_evidence_act_backing_pct: Some(100),
            top_constituent_remedy_consensus: "Enforce national minimum rental habitability standards and reverse-onus protection against retaliatory eviction within 12 months of a documented repair request.".into(),
            sealed_nquin_parity: q3.parity(),
        },
    ];

    let parliamentary_engagements = vec![
        ParliamentaryConstituentEngagementThread {
            engagement_id: "PARL-ENGAGE-AU-VIC-MELB-01".into(),
            civics_au_thread_uri: "https://civics.au/parliamentary-engagement/parl-engage-au-vic-melb-01".into(),
            jurisdiction: AustralianCivicsJurisdiction::Victoria,
            electorate_or_senate_portfolio: "Federal Division of Melbourne / Victorian Charter Oversight".into(),
            parliamentarian_fiduciary_did: "did:webizen:person:hon-sarah-chen-mp".into(),
            parliamentarian_name_and_title: "Hon. Sarah Chen MP (Member for Melbourne · Parliamentary Human Rights & Social Security Committee)".into(),
            parliamentary_register_credential_id: "AU-PARL-VIC-MP-042".into(),
            issue_domain: HumanRightsCivicsIssueDomain::SocialSecurityNdisAndAutomatedWelfareFairness,
            linked_statistical_series_id: "STATS-AU-VIC-WELFARE-2026Q4".into(),
            parliamentary_response_and_action_commitment: "Tabled constituent statistical dossier STATS-AU-VIC-WELFARE-2026Q4 (149 verified constituents, 100% Evidence Act artifact-corroborated, median 165-day review delay) before the Joint Standing Committee on the NDIS and requested mandatory human Duty-of-Inquiry sign-off prior to any algorithmic plan reduction.".into(),
            hansard_or_committee_reference: "Commonwealth Hansard · House Committee on Social Security & NDIS · Ref QoN-2026-1142".into(),
            constituent_nullifier_blind_reply_tokens: vec![blind_1],
            duty_of_inquiry_standing: "Diligent (Six Vectors of Transparency logged in civics.au Accountability Ledger)".into(),
            sealed_nquin_parity: q1.parity(),
        },
        ParliamentaryConstituentEngagementThread {
            engagement_id: "PARL-ENGAGE-AU-VIC-HEALTH-02".into(),
            civics_au_thread_uri: "https://civics.au/parliamentary-engagement/parl-engage-au-vic-health-02".into(),
            jurisdiction: AustralianCivicsJurisdiction::Victoria,
            electorate_or_senate_portfolio: "Senate Community Affairs & PJCHR Human Rights Committee".into(),
            parliamentarian_fiduciary_did: "did:webizen:person:senator-david-wiradjuri".into(),
            parliamentarian_name_and_title: "Senator David Wiradjuri (Senator for Victoria · Parliamentary Joint Committee on Human Rights)".into(),
            parliamentary_register_credential_id: "AU-PARL-CTH-SEN-019".into(),
            issue_domain: HumanRightsCivicsIssueDomain::HealthcareAndContestedDiagnosisAutonomy,
            linked_statistical_series_id: "STATS-AU-VIC-HEALTH-2026Q4".into(),
            parliamentary_response_and_action_commitment: "Initiated legislative inquiry into patient-stewarded Contested Diagnosis & Corroborated Clinical Evidence rights under the Victorian Charter of Human Rights and Responsibilities Act 2006 (Vic) and national My Health Record interoperability rules.".into(),
            hansard_or_committee_reference: "Senate Community Affairs References Committee · PJCHR Inquiry Ref SEN-HR-2026-089".into(),
            constituent_nullifier_blind_reply_tokens: vec![blind_2],
            duty_of_inquiry_standing: "Diligent (Six Vectors of Transparency logged in civics.au Accountability Ledger)".into(),
            sealed_nquin_parity: q2.parity(),
        },
    ];

    CivicsAuHumanRightsObservatory {
        primary_domain: "civics.au".into(),
        observatory_endpoints: vec![
            "https://civics.au".into(),
            "https://observatory.civics.au".into(),
            "https://dev.civics.au".into(),
            "https://web.civics.au".into(),
        ],
        testimonies,
        statistical_series,
        parliamentary_engagements,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_shamir_m_of_n_recovery_and_pocket_cosigner_pairing() {
        let mut state = SanctuaryOnboardingState::default();
        for left in 1..=state.shamir_total_n {
            for right in left + 1..=state.shamir_total_n {
                let receipt =
                    simulate_shamir_recovery_ceremony(&mut state, &[left, right]).unwrap();
                assert!(receipt.contains("2 distinct shares"));
                assert_eq!(state.last_reconstruction_test, receipt);
            }
        }
        let first = simulate_shamir_recovery_ceremony(&mut state, &[1, 2, 4]).unwrap();
        let second = simulate_shamir_recovery_ceremony(&mut state, &[4, 2, 1]).unwrap();
        assert_eq!(first, second);

        for selection in [&[1][..], &[1, 1], &[0, 1], &[1, 5]] {
            assert!(simulate_shamir_recovery_ceremony(&mut state, selection).is_err());
        }
        state.shamir_shares[0].share_commitment_hex.clear();
        assert!(simulate_shamir_recovery_ceremony(&mut state, &[1, 2]).is_err());
        assert_eq!(state.last_reconstruction_test, second);

        let receipt = pair_and_test_pocket_cosigner(
            &mut state,
            "did:webizen:context:pocket",
            "[FD00::1]:51820",
        )
        .unwrap();
        assert!(receipt.contains("NQuin48 parity"));
        assert_eq!(state.pocket_cosigner.wireguard_endpoint, "[fd00::1]:51820");
        assert!(!state.pocket_cosigner.paired);
        assert!(state.pocket_cosigner.last_cosign_receipt_hex.is_empty());
        let valid_preview = state.pocket_cosigner.clone();
        for (did, endpoint) in [
            ("did:webizen:context:pocket/path", "peer.local:51820"),
            ("did:webizen:context:pocket", "https://peer.local:51820"),
            ("did:webizen:context:pocket", "peer.local:0"),
        ] {
            assert!(pair_and_test_pocket_cosigner(&mut state, did, endpoint).is_err());
            assert_eq!(state.pocket_cosigner, valid_preview);
        }
    }

    #[test]
    fn gf256_matches_reference_multiplication_and_has_nonzero_inverses() {
        assert_eq!(gf_multiply(0x57, 0x83), 0xc1);
        for factor in 1..=u8::MAX {
            assert_eq!(gf_multiply(factor, gf_inverse(factor)), 1);
        }
    }

    #[test]
    fn progressive_onboarding_confidence_ladder_and_multi_account_machine_assignments() {
        let reg = build_seeded_machine_onboarding_and_account_assignments(
            "mercer-workstation",
            "did:q42:anvil:10de-2684-99a1e4c2b7d01122",
        );
        assert_eq!(reg.assigned_seats.len(), 4);

        let alex = &reg.assigned_seats[0];
        assert_eq!(
            alex.confidence_tier,
            MachinePrincipalConfidenceTier::Tier3HighAssuranceMultiFactorPrincipal
        );
        assert!(alex.noisy_or_aggregate_confidence > 0.96 && alex.noisy_or_aggregate_confidence < 1.0);

        let elena = &reg.assigned_seats[1];
        assert_eq!(
            elena.confidence_tier,
            MachinePrincipalConfidenceTier::Tier3HighAssuranceMultiFactorPrincipal
        );
        assert_ne!(alex.isolated_vault_rel_path, elena.isolated_vault_rel_path);
        assert_ne!(alex.dedicated_loopback_port, elena.dedicated_loopback_port);

        let leo = &reg.assigned_seats[2];
        assert_eq!(
            leo.confidence_tier,
            MachinePrincipalConfidenceTier::Tier2DesktopAndMobilePaired
        );

        let agent = &reg.assigned_seats[3];
        assert_eq!(
            agent.confidence_tier,
            MachinePrincipalConfidenceTier::Tier1LocalApparatusBound
        );
        // Same OS user ("alex") can host both Alex's Personal Sanctuary seat and his Delegated Agent seat
        // without collapsing them into the same vault or port!
        assert_eq!(alex.os_account_username, agent.os_account_username);
        assert_ne!(alex.webizen_profile_slug, agent.webizen_profile_slug);
        assert_ne!(alex.dedicated_loopback_port, agent.dedicated_loopback_port);
    }
}
