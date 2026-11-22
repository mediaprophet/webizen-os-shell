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
fn interpolate_at_zero(shares: &[(u8, [u8; 32])]) -> [u8; 32] {
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

fn gf_multiply(mut left: u8, mut right: u8) -> u8 {
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
}
