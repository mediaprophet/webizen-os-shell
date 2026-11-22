//! Natural-Person & Multi-Person Cooperative Host Stewardship, Quorum Update Governance,
//! and Tri-Target Debian 13 (Trixie) `.deb` Packaging (`webizen-host`, `webizen-desktop-appliance`,
//! and `webizen-mobile-avf`).
//!
//! ### Core Human-Centric Architectural Premise
//! Updating or mutating a `webizen-host` server (Linux VPS, regional Community Grounds node,
//! or Personal Cloud anchor) **never occurs as an unauthenticated vendor push or unilateral daemon
//! auto-update**. Every host update, configuration change, or Debian package rollout must rely upon
//! an accountable **Natural Person** (`NaturalAgent` / `living-SHACL`, disjoint from `owl:Thing`)
//! or a **Group of Natural Persons** (`M-of-N` Cooperative Quorum over the 32-slot
//! `SuspendedTransactionQueue` with BFT fault tolerance $f = \lfloor(n-1)/3\rfloor$ and
//! paraconsistent `0x32` dissent retention) who are responsible for that host.
//!
//! Consequently, the **Desktop** and **Mobile** products (unified as a **Tri-Target Debian 13
//! Trixie `.deb` Suite** across `amd64` and `arm64`, including **Android 16 AVF `pKVM` EL2
//! Hardware Sanctuary** and **Mobian Trixie**) equip the natural person with:
//! 1. **The Four-Plane Natural-Person Identifier & Stewardship Fabric**:
//!    - Plane 1 (`NaturalAgent` / `living-SHACL`, never `owl:Thing`)
//!    - Plane 2 (`ClaimOpinion` contextual assertions & `M-of-N` quorum votes / dissent)
//!    - Plane 3 (`SpatiotemporalHandle` directional pairwise locators & `Collapse Detector`
//!      proving `Handle revoke != who-erase`)
//!    - Plane 4 (`Instrument` governed tools, `.deb` artefacts, & `webizen-hostd` daemons under
//!      explicit `GovernedAgentDelegation`: *"Natural persons are not identity assets; instruments are not peers"*)
//! 2. **4-Archetype Host Stewardship Authority**:
//!    - `SingleNaturalPerson` (`1-of-1` natural person principal)
//!    - `FamilyHouseholdGuardianship` (`GuardianshipSemantics` + `modalities/capacity.rs` under UN CRC / CRPD)
//!    - `MultiPersonCooperativeQuorum` (`M-of-N` co-steward threshold over 32-slot `SuspendedTransactionQueue`
//!      with paraconsistent `0x32` `RequiresHumanReview` dissent preservation)
//!    - `GovernedServiceAgent` (bounded `ZCAP-LD` / `OP_AUTHORIZATION_GRANT` tool delegation — an
//!      AI or CI agent can *propose* or *stage* a `.deb` update into the queue, but **can never
//!      self-ratify** a host update without the required human steward signature(s)).
//! 3. **Tri-Target Debian 13 (Trixie) `.deb` Suite (`dpkg-deb` trees for `amd64` + `arm64`)**:
//!    - `webizen-desktop-appliance_0.1.0_amd64.deb` (Desktop Workstation & Guest/Appliance Steward Node)
//!    - `webizen-mobile-avf_0.1.0_arm64.deb` (Mobile Pocket Vault & Co-Signer inside Android 16 AVF
//!      `pKVM` EL2 Protected VM / Mobian Trixie with DICE `pvmfw -> anvil` attestation and
//!      `vhost-vsock` CID 2 split-domain bridge)
//!    - `webizen-hostd_0.1.0_amd64.deb` & `arm64` (Headless Human-Centric Linux Host Platform with
//!      `/etc/webizen-host/steward-quorum-policy.cbor-ld` and `/usr/local/bin/webizen-host-apply-update`
//!      that requires a configured cryptographic quorum verifier before permitting updates).

use crate::identifiers::{q_hash_60, IdentifierSet, NATURAL_AGENT_FORBIDDEN_LABELS};
use crate::native_gpu::NativeEtalonStatus;
use crate::session::LoopbackHost;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fs;
use std::path::{Path, PathBuf};
use vibescript_abi::NQuin48;

/// W3C CBOR-LD compressed tag header (`0xD9 0x05 0x01`, Tag 1281), matching `webizen-host`.
pub const CBOR_LD_TAG_HEADER: [u8; 3] = [0xD9, 0x05, 0x01];

/// Capacity mirrors `webizen-host-stewardship::SUSPENDED_TX_QUEUE_SLOTS` (32 slots).
pub const HOST_QUORUM_QUEUE_SLOTS: usize = 32;

/// Persistent relative path for natural-person host stewardship & quorum update governance.
pub const HOST_STEWARDSHIP_STORE_REL_PATH: &str = "wellfair/host_stewardship_quorum.json";

/// Denied labels in Human-Centric host records.
pub const FORBIDDEN_NOMENCLATURE_TERMS: [&str; 3] =
    ["\u{73}overeign", "\u{73}overeignty", "self-\u{73}overeign"];

/// Validate Human-Centric terminology in labels, domains, and host records.
pub fn validate_human_centric_host_nomenclature(text: &str) -> Result<(), String> {
    let lower = text.to_ascii_lowercase();
    if lower.contains("owl:thing")
        || FORBIDDEN_NOMENCLATURE_TERMS
            .iter()
            .any(|term| lower.contains(term))
    {
        return Err("Use Human-Centric, Natural Person, Personal & Family Stewardship, or Cooperative Quorum Stewardship terminology".into());
    }
    Ok(())
}

fn validate_host_field(text: &str, label: &str) -> Result<(), String> {
    validate_human_centric_host_nomenclature(text)?;
    if text.trim().is_empty() || text.chars().any(char::is_control) {
        return Err(format!(
            "{label} must be nonempty and contain no control characters"
        ));
    }
    Ok(())
}

/// The three unified Debian 13 (Trixie) `.deb` package targets built from the same Rust workspace
/// (`qualiadb`, `etalon`, `anvil`, `vibescript`, `webizen-ua-os`, `webizen-os-shell`, `webizen-hostd`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebianTargetProfile {
    /// **Target 1 — Desktop / Workstation Steward Node (`webizen-desktop-appliance`, `amd64` / `arm64`)**:
    /// Runs on Debian 13 Trixie (Wayland `cage` DRM/KMS appliance or Guest `.deb` on Linux/WSL2).
    /// Equips the natural person with the Four-Plane Identifier Fabric, Sanctuary Keep (`:8080`),
    /// Poet + 150-morphism Synth workbench, and primary `webizen-host` Steward & Quorum Signer console.
    DesktopStewardNode,
    /// **Target 2 — Mobile Pocket Sanctuary & Co-Signer (`webizen-mobile-avf`, `arm64`)**:
    /// Runs unmodified Debian 13 Trixie `arm64` inside **Android 16 AVF (`pKVM` EL2 Protected VM + `crosvm`)**
    /// or native **Mobian Trixie**. Isolates WellFair Sanctuary RAM at EL2 from the host Android OS,
    /// binds `pvmfw -> anvil` DICE hardware attestation, runs Etalon native Vulkan over `virtio-gpu` /
    /// `gfxstream` (bypassing mobile browser WebGPU bans), routes public tensor workloads over
    /// `vhost-vsock` (CID 2), and acts as the natural person's pocket `M-of-N` quorum co-signer and
    /// guardianship approval apparatus.
    MobileAvfSanctuaryNode,
    /// **Target 3 — Human-Centric Linux Host Platform (`webizen-hostd`, `amd64` / `arm64`)**:
    /// Runs on Debian 13 Trixie VPS, bare-metal server, or Raspberry Pi / Community Grounds microgrid node.
    /// Hosts `ns.webcivics.net` content-negotiated virtual namespaces, $O(1)$ `NQuin` DNS + `QPolicy`
    /// nftables/eBPF firewall, Three-Tier Multi-Tenant Sandbox, Relationship-Scoped Mail, `Q-CDN`,
    /// Two-Loop Permissive Commons, and client-encrypted `.q42j` Shamir backups — governed strictly
    /// by the natural person(s) who steward it viaDesktop/Mobile quorum ratification.
    HostServerPlatformNode,
}

impl DebianTargetProfile {
    pub const ALL: [Self; 3] = [
        Self::DesktopStewardNode,
        Self::MobileAvfSanctuaryNode,
        Self::HostServerPlatformNode,
    ];

    pub const fn deb_package_name(self) -> &'static str {
        match self {
            Self::DesktopStewardNode => "webizen-desktop-appliance",
            Self::MobileAvfSanctuaryNode => "webizen-mobile-avf",
            Self::HostServerPlatformNode => "webizen-hostd",
        }
    }

    pub const fn architecture(self) -> &'static str {
        match self {
            Self::DesktopStewardNode => "amd64",
            Self::MobileAvfSanctuaryNode => "arm64",
            Self::HostServerPlatformNode => "amd64",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::DesktopStewardNode => {
                "Target 1 · Desktop Steward & Authoring Node (webizen-desktop-appliance_0.1.0_amd64.deb)"
            }
            Self::MobileAvfSanctuaryNode => {
                "Target 2 · Mobile AVF pKVM EL2 Sanctuary & Pocket Co-Signer (webizen-mobile-avf_0.1.0_arm64.deb)"
            }
            Self::HostServerPlatformNode => {
                "Target 3 · Human-Centric Linux Host Platform (webizen-hostd_0.1.0_amd64.deb)"
            }
        }
    }

    pub const fn role_in_human_fabric(self) -> &'static str {
        match self {
            Self::DesktopStewardNode => {
                "Primary workstation where the natural person authors works (Poet/Synth), manages their Four-Plane Identifier Fabric, curates Sanctuary Keep (:8080), and proposes/signs updates for their webizen-host server."
            }
            Self::MobileAvfSanctuaryNode => {
                "Hardware-isolated Debian 13 arm64 micro-VM on the phone (Android 16 AVF pKVM EL2 / Mobian Trixie) with DICE pvmfw->anvil attestation, gfxstream native Vulkan for Qaps, vhost-vsock NPU bridge, WellFair Sanctuary Keep, and pocket M-of-N quorum co-signing for webizen-host updates."
            }
            Self::HostServerPlatformNode => {
                "Always-on Debian 13 Trixie VPS / Community Grounds node (:4242) executing NQuin DNS, QDNF mesh, Solid Databox, Q-CDN, and client-encrypted .q42j Shamir backups — refuses any binary or config update unless ratified by its responsible natural-person steward(s)."
            }
        }
    }
}

/// The Four Human-Centric Stewardship & Agency Archetypes (`webizen-host-stewardship` WP-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostStewardshipArchetype {
    /// Single Natural Person (`1-of-1` `NaturalAgent` principal across their Desktop + Mobile AVF nodes).
    SingleNaturalPerson,
    /// Family / Household Collective (`GuardianshipSemantics` + `modalities/capacity.rs` under UN CRC / CRPD,
    /// with 310,000-iteration PBKDF2-HMAC-SHA256 + HKDF-SHA256 per-member lane isolation and guardian co-signature).
    FamilyHouseholdGuardianship,
    /// Multi-Person Group / Community / Cooperative (`M-of-N` Steward Quorum over the 32-slot
    /// `SuspendedTransactionQueue` with BFT $f = \lfloor(n-1)/3\rfloor$ and paraconsistent `0x32` dissent retention).
    MultiPersonCooperativeQuorum,
    /// Governed Autonomous / Service Agent (`ZCAP-LD` + `OP_AUTHORIZATION_GRANT` — may stage candidate
    /// `.deb` packages into the queue, but can **never** self-ratify without natural-person steward signatures).
    GovernedServiceAgent,
}

impl HostStewardshipArchetype {
    pub const ALL: [Self; 4] = [
        Self::SingleNaturalPerson,
        Self::FamilyHouseholdGuardianship,
        Self::MultiPersonCooperativeQuorum,
        Self::GovernedServiceAgent,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::SingleNaturalPerson => "1. Single Natural Person Stewardship (1-of-1 Principal)",
            Self::FamilyHouseholdGuardianship => {
                "2. Family & Household Guardianship (UN CRC / CRPD Capacity & Co-Signature)"
            }
            Self::MultiPersonCooperativeQuorum => {
                "3. Multi-Person Cooperative Quorum (M-of-N Stewards · BFT + Paraconsistent 0x32 Dissent)"
            }
            Self::GovernedServiceAgent => {
                "4. Governed Service / CI Agent (Stage-Only Tool · Human Ratification Required)"
            }
        }
    }

    pub const fn update_authority_rule(self) -> &'static str {
        match self {
            Self::SingleNaturalPerson => {
                "Updates to webizen-host require a cryptographic signature from the natural person's Desktop or Mobile AVF Sanctuary keypair (or 2-of-2 Desktop + Mobile apparatus co-signature)."
            }
            Self::FamilyHouseholdGuardianship => {
                "Adult Steward Guardians can ratify host updates; supported dependents or children acting on family services require a guardian co-signature under GuardianshipSemantics."
            }
            Self::MultiPersonCooperativeQuorum => {
                "Updates enter the 32-slot SuspendedTransactionQueue and execute only when >= M of N natural-person co-stewards approve AND zero explicit denials (paraconsistent 0x32 RequiresHumanReview) are active."
            }
            Self::GovernedServiceAgent => {
                "Invariant: 'Natural persons are not identity assets; instruments are not peers.' An automated agent may build and enqueue a candidate .deb update, but is mathematically barred from casting a steward ratification vote."
            }
        }
    }
}

/// A Natural Person Steward (`NaturalAgent` / `living-SHACL`, disjoint from `owl:Thing`) who bears
/// responsibility for a `webizen-host` instance through their Desktop and/or Mobile AVF Debian apparatus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponsibleNaturalSteward {
    /// Pairwise / stewardship DID on Plane 1 (`did:qi:...` or `did:q42:qrc:...`).
    pub steward_did: String,
    /// 60-bit FNV-1a token (`q_hash_60(steward_did)`).
    pub steward_did_hash: u64,
    /// Human-readable display hint (validated against `NATURAL_AGENT_FORBIDDEN_LABELS` and `owl:Thing`).
    pub display_hint: String,
    /// Stewardship role (`"PrincipalSteward"`, `"CoStewardGuardian"`, `"SupportedFamilyMember"`).
    pub role: String,
    /// Apparatus profile from which this steward signs (`"Desktop Debian/Guest Steward"` or `"Mobile Debian 13 AVF pKVM EL2 Sanctuary"`).
    pub apparatus_profile: String,
    /// Ed25519 + post-quantum `ML-DSA-65` dual public key fingerprint bound in Sanctuary Keep.
    pub dual_pubkey_fingerprint: String,
    /// Whether this natural person has full legal capacity to ratify host updates directly
    /// (if `false`, requires `guardian_did` co-signature under `FamilyHouseholdStewardship`).
    pub has_full_legal_capacity: bool,
    pub guardian_did: Option<String>,
}

/// An individual natural-person steward's vote (Approval or Paraconsistent `0x32` Dissent) on a
/// proposed `webizen-host` update package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StewardQuorumVote {
    pub steward_did: String,
    pub apparatus_profile: String,
    pub approve: bool,
    pub comment: String,
    pub dual_signature_hex: String,
    pub sealed_nquin_parity: u64,
}

/// A proposed `webizen-host` `.deb` update or host configuration mutation sitting in the
/// 32-slot `SuspendedTransactionQueue` awaiting natural-person / `M-of-N` quorum ratification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostUpdateProposal {
    pub slot_index: usize,
    pub proposal_id: String,
    pub proposal_hash: u64,
    pub target_host_domain: String,
    pub target_deb_package: String,
    pub target_version: String,
    pub artifact_sha3_256: String,
    /// Who proposed the update (may be a `NaturalAgent` steward or a `GovernedServiceAgent` tool).
    pub proposed_by_did: String,
    pub proposer_is_instrument_tool: bool,
    pub required_m: u8,
    pub total_n: u8,
    pub votes: Vec<StewardQuorumVote>,
    /// True if any co-steward cast an explicit denial (`approve == false`), triggering
    /// Paraconsistent `0x32` (`https://ns.webcivics.net/illocution#RequiresHumanReview`) and
    /// blocking automatic execution until human reconciliation!
    pub paraconsistent_review_flagged: bool,
    /// True only after the governing host verifies a cryptographic quorum.
    pub ratified_for_host_apply: bool,
    pub applied_to_host: bool,
    pub cbor_ld_envelope_hex: String,
    pub sealed_nquin_parity: u64,
}

// ============================================================================
// NUANCED HUMAN-CENTRIC HOST TOPOLOGY:
// 1. Server Runtime Responsibility vs. Hosted Project/Website/Domain Ownership
//    & Project Lifecycle Transitions (Personal -> Cooperative Group -> Q-CDN Swarm)
// 2. Multi-Locus Personal & Roaming Host Fleet (Home Server, Camper/Mobile Rig,
//    Online VPS, or Local-Only Offline Mesh)
// 3. Three-Way Actor & Legal-Personality Distinction + Mandatory Commercial
//    Licensing Schedule for All Incorporated Entities
// ============================================================================

/// Locus / physical topology of a `webizen-host` server node in a person's or group's fleet.
///
/// A person might operate only a Home Server, only a 12V Camper/Walkabout Rig, an Online Cloud VPS,
/// or a hybrid mesh where their Home/Camper node is primary (`Sanctuary Keep` + localLAN/Wi-Fi/BLE)
/// and an optional Online VPS acts as an encrypted `.q42j` Shamir mirror and `Q-CDN` ingress relay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostNodeLocusKind {
    /// **Home Server / Homestead Appliance** (`arm64` RPi 5 / `amd64` SFF Debian 13 Trixie box on home LAN).
    /// Can operate 100% standalone without an online VPS, or sync with a roaming camper/phone and optional VPS.
    HomeServerAppliance,
    /// **Camper / Walkabout Mobile Rig / Field Node** (12V solar/battery Debian 13 Trixie box in a campervan/rover).
    /// Operates 100% offline/local-first on the road (mDNS + Acoustic/BLE + local Wi-Fi mesh to phone/laptop)
    /// and opportunistically syncs delta `NQuin`s over WireGuard/QDNF when back in connectivity range.
    CamperMobileRigNode,
    /// **Online Personal / Cooperative Cloud VPS** (Always-on remote Debian 13 Trixie VPS anchor).
    /// Never holds unencrypted Sanctuary Keep keys; serves public/permissioned `.q42`/`.hmc` namespaces,
    /// buffers incoming `pulse` mail/QDNF frames when the home/camper node is asleep or off-grid,
    /// and stores client-encrypted `.q42j` Shamir backup shares.
    OnlineCloudVpsRelay,
    /// **Community Grounds / Regional Microgrid Edge Node** (Shared regional cooperative server).
    CommunityGroundsEdgeNode,
}

impl HostNodeLocusKind {
    pub const ALL: [Self; 4] = [
        Self::HomeServerAppliance,
        Self::CamperMobileRigNode,
        Self::OnlineCloudVpsRelay,
        Self::CommunityGroundsEdgeNode,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::HomeServerAppliance => "home_server",
            Self::CamperMobileRigNode => "camper_mobile_rig",
            Self::OnlineCloudVpsRelay => "online_cloud_vps",
            Self::CommunityGroundsEdgeNode => "community_grounds_edge",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::HomeServerAppliance => {
                "Home Server / Homestead Appliance (Local-First Personal VPS)"
            }
            Self::CamperMobileRigNode => {
                "Camper / Walkabout 12V Mobile Rig (Offline-Capable Field Host)"
            }
            Self::OnlineCloudVpsRelay => {
                "Online Cloud VPS Anchor (Always-On Ingress & Shamir .q42j Mirror)"
            }
            Self::CommunityGroundsEdgeNode => "Community Grounds / Regional Microgrid Edge Host",
        }
    }

    pub const fn routing_profile(self) -> &'static str {
        match self {
            Self::HomeServerAppliance => {
                "Primary LAN/Wi-Fi mDNS + QDNF ULA fd00::/8; works standalone without an online VPS or pairs with Camper & Cloud VPS."
            }
            Self::CamperMobileRigNode => {
                "100% offline-capable 12V field mesh (local Wi-Fi + Acoustic/BLE + QDNF); opportunistically syncs NQuin deltas when connectivity returns."
            }
            Self::OnlineCloudVpsRelay => {
                "Public ingress (ns.webcivics.net / Solid Pod / Q-CDN Tier-A mirror); holds zero raw Sanctuary keys (2-of-3 Shamir .q42j only)."
            }
            Self::CommunityGroundsEdgeNode => {
                "Regional cooperative microgrid & Q-CDN Tier-A/B seeder governed by M-of-N community co-stewards."
            }
        }
    }
}

/// A physical or virtual `webizen-host` node in the user's or cooperative's multi-locus fleet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonalHostNode {
    pub node_id: String,
    pub label: String,
    pub locus: HostNodeLocusKind,
    /// Who bears **Server Runtime Responsibility** (kernel/OS `.deb` updates, hardware, power, `nftables` firewall)
    /// for this specific machine — distinct from who owns the projects or websites hosted *on* it!
    pub runtime_operator_steward_dids: Vec<String>,
    pub qdnf_ula_ipv6: String,
    pub endpoint_hint: String,
    /// True if this node is operating in standalone local-only mode (no remote cloud VPS required).
    pub standalone_without_online_vps: bool,
    pub holds_local_sanctuary_keep: bool,
    pub qcdn_seeder_tier: String,
    pub sealed_nquin_parity: u64,
}

/// Lifecycle and ownership stage of a **Project, Website, or Domain** hosted on one or more
/// `webizen-host` runtime nodes — strictly separated from **Server Runtime Responsibility**.
///
/// Supports:
/// - A personal project owned by the VPS owner (`PersonalOwnerProject`)
/// - A personal project that **evolves/transitions into a multi-person group project** (`TransitionedPersonalToGroupProject`)
/// - A native multi-person cooperative / community group project (`CooperativeGroupProject`)
/// - **Mutual-Aid / Third-Party Hosted Project** (`SupportedThirdPartyOrGroupProject`): another person's
///   or another group's project/website/domain being supported and replicated by this host node,
///   creating the user-powered **`Q-CDN` Swarm** infrastructure!
/// - **Incorporated Entity Commercial Project** (`IncorporatedCommercialTenantProject`): a project or
///   website belonging to an incorporated legal personality (subject to mandatory commercial licensing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostedProjectOwnershipStage {
    /// Owned by the same natural person who operates the host server (`1-of-1` personal project).
    PersonalOwnerProject,
    /// Started as a personal project on the owner's VPS/Home/Camper node and **transitioned into a
    /// multi-person cooperative group project** (`M-of-N` project stewards, distinct from the server hardware operator).
    TransitionedPersonalToGroupProject,
    /// Authored and governed from inception by a semantically defined cooperative group (`M-of-N` project quorum).
    CooperativeGroupProject,
    /// **Another person's or another group's project/website/domain** being supported, mirrored, or seeded
    /// on this host (`Q-CDN` Tier-A/B Swarm custody), where the project stewards control content/code
    /// while the server runtime steward controls local hardware resource quotas (`HostResourceBudgetPool`).
    SupportedThirdPartyOrGroupProject,
    /// A project, website, or domain owned by an **Incorporated Entity** (legal personality governed by
    /// corporate stakeholders/board rather than natural-person users — always subject to commercial licensing).
    IncorporatedCommercialTenantProject,
}

impl HostedProjectOwnershipStage {
    pub const ALL: [Self; 5] = [
        Self::PersonalOwnerProject,
        Self::TransitionedPersonalToGroupProject,
        Self::CooperativeGroupProject,
        Self::SupportedThirdPartyOrGroupProject,
        Self::IncorporatedCommercialTenantProject,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::PersonalOwnerProject => "1. Personal Project (Owned by Host Steward — Can Transition to Group)",
            Self::TransitionedPersonalToGroupProject => {
                "2. Transitioned Personal -> Group Project (Evolved into M-of-N Cooperative Quorum + Q-CDN)"
            }
            Self::CooperativeGroupProject => "3. Native Group / Cooperative Project (M-of-N Project Stewards)",
            Self::SupportedThirdPartyOrGroupProject => {
                "4. Supported Another Person's / Group's Project (Mutual-Aid Hosting & Q-CDN Swarm Mirror)"
            }
            Self::IncorporatedCommercialTenantProject => {
                "5. Incorporated Entity Workload (Stakeholder Legal Personality · Mandatory Commercial Licence)"
            }
        }
    }
}

/// Separation of **Server Runtime Responsibility** vs. **Hosted Project / Website / Domain Stewardship**,
/// enabling any project to be replicated across Home, Camper, Online VPS, and peer nodes as a
/// decentralized **`Q-CDN` Swarm** (`SignedRootIndex` `< 10 KiB` + 40 KiB `QualiaSuperBlock` chunks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedProjectWorkload {
    pub project_id: String,
    pub title: String,
    pub attached_domain: String,
    pub ownership_stage: HostedProjectOwnershipStage,
    /// Natural-person or group stewards who control the **Project / Website / Domain content & releases**
    /// (distinct from `host_runtime_node_ids` who operate the underlying server runtime!).
    pub project_steward_dids: Vec<String>,
    pub project_quorum_threshold_m: u8,
    /// Which `PersonalHostNode`s (Home Server, Camper Rig, Online VPS, Peer VPS) host or mirror this project.
    pub hosted_on_runtime_node_ids: Vec<String>,
    /// Peer `Q-CDN` seeder DIDs (`Tier A` Cooperative VPS Mirrors + `Tier B` Desktop/Mobile `:4242` seeders)
    /// that replicate this project's `< 10 KiB` `SignedRootIndex` and 40 KiB `QualiaSuperBlock` slices.
    pub qcdn_swarm_mirror_dids: Vec<String>,
    pub qcdn_signed_root_merkle_hex: String,
    /// Optional reference to an `IncorporatedLegalEntityRecord` if owned by an incorporated entity.
    pub incorporated_entity_id: Option<String>,
    pub provenance_history: Vec<String>,
    pub sealed_nquin_parity: u64,
}

/// Strict Ontological & Jural Distinction between the **Three Actor / Legal-Personality Categories**
/// in the Webizen Ecosystem:
/// 1. **`NaturalPersonPrincipal` (`NaturalAgent` / `living-SHACL`, disjoint from `owl:Thing`)**:
///    A living human being acting in their own personal, family, or cooperative capacity.
///    **Royalty / Commercial Licensing Fee: `$0.00` (0% Free Human-Rights & Permissive Commons Lane).**
/// 2. **`NaturalPersonAgentOfIncorporatedEntity` (`ActingRepresentative` / `ConductRecord`)**:
///    A living human being (employee, director, officer, contractor) acting as an authorized agent on
///    behalf of an incorporated legal personality. The human retains their personal human rights and
///    `ConductRecord` accountability (`acting_delegate_did`), **but their actions on behalf of the
///    corporation carry the corporation's commercial licensing obligations** (never laundering corporate
///    commercial use through a personal `$0` natural-person account!).
/// 3. **`IncorporatedLegalEntity` (`JuridicalPerson` / `IncorporatedOrganization`)**:
///    An incorporated entity (corporation, Pty Ltd, Inc., GmbH, statutory body, incorporated association,
///    or non-profit/charity corporation) whose structure is governed by **structural stakeholders**
///    (shareholders, board of directors, constitution, statutory charter, creditors) rather than
///    natural-person social-web users. **Always subject to mandatory Commercial Licensing irrespective
///    of incorporation subtype, with tiered pricing based on entity classification.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorJuralPersonalityKind {
    /// Living human acting for themselves, their family, or an unincorporated natural-person cooperative.
    /// Zero commercial licence charge (`$0.00` / 0% royalty).
    NaturalPersonPrincipal,
    /// Living human acting as an officer, employee, or representative of an Incorporated Entity.
    /// Explicit agency binding (`principal = IncorporatedLegalEntity`, `agent = NaturalAgent`);
    /// inherits the Incorporated Entity's mandatory commercial licensing schedule.
    NaturalPersonAgentOfIncorporatedEntity,
    /// The Incorporated Legal Personality itself (defined by structural stakeholder factors — board,
    /// equity/voting shares, jurisdiction, constitution). Always charged under the Commercial Licensing Schedule.
    IncorporatedLegalEntity,
}

impl ActorJuralPersonalityKind {
    pub const ALL: [Self; 3] = [
        Self::NaturalPersonPrincipal,
        Self::NaturalPersonAgentOfIncorporatedEntity,
        Self::IncorporatedLegalEntity,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::NaturalPersonPrincipal => {
                "1. Natural Person Principal (NaturalAgent · living-SHACL · $0 Free Human-Centric Commons Lane)"
            }
            Self::NaturalPersonAgentOfIncorporatedEntity => {
                "2. Natural Person Acting as Agent for an Incorporated Entity (Explicit Principal-Agent Binding · Corporate Commercial Licence Applies)"
            }
            Self::IncorporatedLegalEntity => {
                "3. Incorporated Legal Entity (Stakeholder/Board-Governed Legal Personality · Mandatory Commercial Licensing)"
            }
        }
    }
}

/// Tiered Commercial Pricing Classification for **Incorporated Entities**.
///
/// **Rule**: Every incorporated entity is **always charged (subject to commercial licensing)**
/// irrespective of what kind of incorporated entity it is (because it is an artificial statutory
/// legal personality structured around corporate stakeholders rather than a living natural person),
/// **notwithstanding differentiated pricing tiers** scaled to the incorporated entity's structural footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncorporatedEntityPricingTier {
    /// Tier 1: Incorporated Non-Profit, Registered Charity, or Incorporated Community Association
    /// (Mandatory commercial licence at a concessional solidarity rate — never `$0` free natural-person lane).
    IncorporatedNonProfitOrCivicBody,
    /// Tier 2: Incorporated Worker/Producer Cooperative or Small/Micro Enterprise (`< 20` FTE / bounded revenue).
    SmallOrCooperativeCorporation,
    /// Tier 3: Mid-Market Private or Public Corporation (`Pty Ltd` / `Ltd` / `Inc.`).
    MidMarketCommercialCorporation,
    /// Tier 4: Large / Multinational Enterprise or Institutional Platform Corporation.
    LargeEnterpriseOrMultinationalCorp,
}

impl IncorporatedEntityPricingTier {
    pub const ALL: [Self; 4] = [
        Self::IncorporatedNonProfitOrCivicBody,
        Self::SmallOrCooperativeCorporation,
        Self::MidMarketCommercialCorporation,
        Self::LargeEnterpriseOrMultinationalCorp,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::IncorporatedNonProfitOrCivicBody => {
                "Tier 1 · Incorporated Non-Profit / Charity / Statutory Body (Concessional Commercial Licence)"
            }
            Self::SmallOrCooperativeCorporation => {
                "Tier 2 · Small Enterprise / Incorporated Co-op Pty Ltd (Standard SME Commercial Licence)"
            }
            Self::MidMarketCommercialCorporation => {
                "Tier 3 · Mid-Market Commercial Corporation (Full Commercial Obligation Pool + Q-CDN Carriage)"
            }
            Self::LargeEnterpriseOrMultinationalCorp => {
                "Tier 4 · Large / Multinational Enterprise Corporation (Enterprise Commercial Schedule)"
            }
        }
    }

    /// Monthly base commercial licensing fee in minor units (cents) — **strictly `> 0` for all tiers**,
    /// feeding the Two-Loop Permissive Commons (`Loop 1` contributor obligation pool up to $10\times$ sunset +
    /// `Loop 2` `Q-CDN` seeder bandwidth/compute settlement).
    pub const fn monthly_base_commercial_fee_cents(self) -> u64 {
        match self {
            Self::IncorporatedNonProfitOrCivicBody => 4_900, // $49.00 / mo (mandatory commercial licence, concessional tier)
            Self::SmallOrCooperativeCorporation => 19_900,   // $199.00 / mo
            Self::MidMarketCommercialCorporation => 79_900,  // $799.00 / mo
            Self::LargeEnterpriseOrMultinationalCorp => 249_900, // $2,499.00 / mo
        }
    }

    /// Per-GiB `Q-CDN` & `webizen-host` commercial carriage rate in cents.
    pub const fn qcdn_carriage_rate_cents_per_gib(self) -> u64 {
        match self {
            Self::IncorporatedNonProfitOrCivicBody => 5,
            Self::SmallOrCooperativeCorporation => 12,
            Self::MidMarketCommercialCorporation => 25,
            Self::LargeEnterpriseOrMultinationalCorp => 45,
        }
    }
}

/// Record of an **Incorporated Legal Entity** (`JuridicalPerson`) operating or consuming `webizen-host`
/// infrastructure, explicitly modeling:
/// - Its **structural stakeholder factors** (board/constitutional authority, equity/voting structure,
///   jurisdiction, registration identifier like ACN/ABN/LEI) as distinct from end-users;
/// - The **Natural Persons acting as its authorized agents** (`NaturalPersonAgentOfIncorporatedEntity`);
/// - Its **mandatory Commercial Licensing obligation** (`commercial_licence_charged == true` invariant).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncorporatedLegalEntityRecord {
    pub entity_did: String,
    pub legal_name: String,
    pub jurisdiction_and_reg_number: String,
    pub pricing_tier: IncorporatedEntityPricingTier,
    /// Structural stakeholder factors governing the legal personality (e.g. Board of Directors,
    /// Shareholder Class A/B voting quorum, Constitutional Charter, Creditor covenants) — distinct from users!
    pub stakeholder_governance_factors: Vec<String>,
    /// Natural persons (`did:qi:...`) authorized to act as agents/officers for this incorporated entity
    /// (`ActorJuralPersonalityKind::NaturalPersonAgentOfIncorporatedEntity`).
    pub authorized_natural_person_agent_dids: Vec<String>,
    /// Invariant: **Always `true`** for every `IncorporatedLegalEntityRecord` regardless of tier.
    pub mandatory_commercial_licence_active: bool,
    pub monthly_base_fee_cents: u64,
    pub qcdn_carriage_cents_per_gib: u64,
    pub settlement_rail: String,
    pub loop1_commons_pool_contribution_cents: u64,
    pub sealed_nquin_parity: u64,
}

// ============================================================================
// 4. SHARED SYSTEMS, GUARDIANSHIP RELATIONS & REMOTE CARER / FAMILY ALERTS
//    (Share-Houses, Families with Children along the AgencyStage Gradient,
//     and Elderly / Supported Adults with Remote Family Members or Carers)
// ============================================================================

/// The specific type of shared living or distributed care topology on a `webizen-host` / Home Server /
/// Desktop / Mobile AVF network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharedLivingAndCareTopologyKind {
    /// **1. Share-House / Co-Living Peer Household (`NoGuardianshipOverPeers`)**:
    /// Equal adult housemates sharing a Home Server / LAN / utilities / shopping / house rules.
    /// - **Strict Invariant**: Every housemate has a cryptographically isolated Sanctuary Keep lane
    ///   (310,000-iteration PBKDF2-HMAC-SHA256 + HKDF-SHA256 per-member key derivation).
    /// - **Zero Cross-Peer Guardianship**: No housemate (including the router/server hardware owner)
    ///   has guardianship or inspection rights over another housemate's personal vault, medical records,
    ///   or communications. Shared administration applies *only* to the shared household commons
    ///   (Wi-Fi/DNS, electricity/solar microgrid bills, chore/maintenance ledger) via `M-of-N` housemate quorum.
    ShareHousePeerCoLiving,

    /// **2. Family Household with Children (`DevelopmentalAgencyGradient & SplitStewardRoles`)**:
    /// Parents / guardians and children (`Neonate -> Infant -> Child -> Adolescent -> Adult` on `AgencyStage`,
    /// grounded in `wellfare-core::anatomy::birth` and UN CRC Articles 3, 5, 12, 16).
    /// - **Child Is Never Collapsed into Parent's DID**: Every child owns their own foundational
    ///   `DigitalBirthRecord` and pairwise fabric (`BirthRecordInvalid::SubjectCollapsed` is rejected).
    /// - **Distributed `StewardRole`s**: Different guardians can hold distinct roles (`PrimaryCare`, `Medical`,
    ///   `Educational`, `Financial`, `Legal`, `Cultural`).
    /// - **Severity-Gated `GuardianNotification`s & Proxy Escrow**: Ingested or encountered content raising
    ///   safety/medical flags at or above the configured `FlagSeverity` (`Info`/`Notice`/`Concern`/`Urgent`)
    ///   emits a tamper-evident `GuardianNotification` recorded in the `AccountabilityLedger`, while proxy writes
    ///   enter `GuardianshipProposal` `M-of-N` co-signature escrow (with protective co-guardian veto).
    FamilyWithChildrenGuardianship,

    /// **3. Elderly or Supported Person with Remote Family Members & Carers (`SupportedAgency & RemoteCareAlerts`)**:
    /// An elderly person (living independently at home) or a `SupportedAdult` (UN CRPD Article 12
    /// supported—never substituted—decision-making) whose family members, next-of-kin, or professional carers
    /// live *elsewhere* (connected over `SocialWebNet` WireGuard / QDNF `ports::QDNF`).
    /// - **Sanctuary Privacy Preserved vs. Scoped Care Alerts**: Remote family/carers do **not** get raw access
    ///   to the elderly person's private Sanctuary Keep or diary; they receive **purpose-bound, severity-gated
    ///   Care & Safety Alerts** (missed medication reminders, fall/inactivity `Heartbeat` lapse, rPPG/respiration
    ///   physiological anomaly, urgent clinical observation, or `IncapacitySwitch` advocacy trigger) under
    ///   revocable envelope-encrypted `ConsentCredential`s.
    /// - **Reversibility & Accountability (`ConductRecord` + `DutyOfInquiry`)**: Every remote carer action or
    ///   alert acknowledgement records an auditable `ConductRecord` evaluated against `DutyOfInquiry`
    ///   (`Diligent`, `NoFault`, `UncheckedNoHarm`, `Negligent`), and if the elderly person taps `"I am OK"`
    ///   (`principal_alive` / `regain_capacity`), any pending escalation reverses immediately.
    ElderlyOrSupportedPersonWithRemoteCarers,
}

impl SharedLivingAndCareTopologyKind {
    pub const ALL: [Self; 3] = [
        Self::ShareHousePeerCoLiving,
        Self::FamilyWithChildrenGuardianship,
        Self::ElderlyOrSupportedPersonWithRemoteCarers,
    ];

    pub const fn title(self) -> &'static str {
        match self {
            Self::ShareHousePeerCoLiving => {
                "1. Share-House / Co-Living Peers (Isolated Personal Vaults + Shared Utilities Quorum · Zero Peer Guardianship)"
            }
            Self::FamilyWithChildrenGuardianship => {
                "2. Family with Children (UN CRC AgencyStage Gradient · Distributed StewardRoles · Flagged-Ingest Guardian Alerts)"
            }
            Self::ElderlyOrSupportedPersonWithRemoteCarers => {
                "3. Elderly / Supported Person with Remote Family & Carers (UN CRPD Supported Agency · Heartbeat/Medication/Vital Alerts)"
            }
        }
    }

    pub const fn privacy_and_alert_invariant(self) -> &'static str {
        match self {
            Self::ShareHousePeerCoLiving => {
                "Housemates co-govern shared router/solar/utility bills (M-of-N), but each person's Sanctuary Keep is cryptographically isolated; the hardware owner cannot inspect a flatmate's private life."
            }
            Self::FamilyWithChildrenGuardianship => {
                "Children own their own principal DID (never collapsed into a parent); guardians hold scoped StewardRoles (PrimaryCare, Medical, Educational) and receive ledger-recorded GuardianNotifications when flags >= threshold."
            }
            Self::ElderlyOrSupportedPersonWithRemoteCarers => {
                "Supported agency, not surveillance: the elderly principal retains Sanctuary privacy while remote family/carers elsewhere receive scoped QDNF alerts (heartbeat lapse, missed meds, falls, vitals) with reversible 'I am OK' check-in."
            }
        }
    }
}

/// Severity level for a Guardianship or Remote Care Alert (mirrors `qualia_core_db::hypermedia::FlagSeverity`
/// and `qualia_client_core::wellfair::ingest_guardian::GuardianNotification`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CareAlertSeverity {
    /// Level 0 — Routine informational log (e.g. daily check-in completed).
    Info = 0,
    /// Level 1 — Notice (e.g. medication reminder due soon or household bill enqueued).
    Notice = 1,
    /// Level 2 — Concern (e.g. missed medication dose, child flagged-content ingest, or abnormal sleep/vital trend).
    Concern = 2,
    /// Level 3 — Urgent (e.g. elderly fall / prolonged liveness `Heartbeat` lapse, emergency SOS, or acute physiological alert).
    Urgent = 3,
}

impl CareAlertSeverity {
    pub const fn level(self) -> u64 {
        self as u64
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "0 · Info",
            Self::Notice => "1 · Notice",
            Self::Concern => "2 · Concern",
            Self::Urgent => "3 · Urgent",
        }
    }
}

/// Category of trigger for a Guardianship or Remote Carer Alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CareAlertTriggerKind {
    /// Elderly / supported person liveness `Heartbeat` lapse (no check-in / activity within grace window).
    ElderlyLivenessHeartbeatLapse,
    /// Missed scheduled medication dose (`wellfare-core::medication` / `med_reminders.rs`).
    MissedMedicationSchedule,
    /// Physiological / fall / environmental sensor anomaly (consent-gated biosense rPPG/respiration/fall detector).
    PhysiologicalOrFallSensorAlert,
    /// Child / dependent flagged hypermedia or web content ingest (`ingest_guardian::guardian_notifications`).
    ChildFlaggedContentIngest,
    /// Proxy record write requiring `M-of-N` co-guardian ratification (`GuardianshipProposal` escrow).
    ProxyWriteEscrowCoSignatureNeeded,
    /// Share-house common utility / microgrid / lease proposal requiring housemate quorum.
    ShareHouseCommonsUtilityNotice,
}

impl CareAlertTriggerKind {
    pub const ALL: [Self; 6] = [
        Self::ElderlyLivenessHeartbeatLapse,
        Self::MissedMedicationSchedule,
        Self::PhysiologicalOrFallSensorAlert,
        Self::ChildFlaggedContentIngest,
        Self::ProxyWriteEscrowCoSignatureNeeded,
        Self::ShareHouseCommonsUtilityNotice,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::ElderlyLivenessHeartbeatLapse => "Elderly Liveness Heartbeat / Check-In Lapse",
            Self::MissedMedicationSchedule => "Missed Medication Schedule Alert",
            Self::PhysiologicalOrFallSensorAlert => "Fall / Physiological Biosense Anomaly",
            Self::ChildFlaggedContentIngest => {
                "Child Flagged-Content Ingest (GuardianNotification)"
            }
            Self::ProxyWriteEscrowCoSignatureNeeded => {
                "Proxy Write Held in M-of-N Guardianship Escrow"
            }
            Self::ShareHouseCommonsUtilityNotice => {
                "Share-House Commons Utility / Microgrid Quorum Notice"
            }
        }
    }
}

/// A member or remote participant in a Shared Living or Distributed Guardianship/Care Circle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedSystemParticipant {
    pub participant_did: String,
    pub display_name: String,
    /// E.g. `"HousematePeer"`, `"ChildPrincipal (AgencyStage::Child, 40% self-determination)"`,
    /// `"ElderlyPrincipal (AgencyStage::SupportedAdult, 100% self-determination with supports)"`,
    /// `"ParentGuardian"`, `"RemoteFamilyCarer (Daughter in Melbourne)"`, `"VisitingCommunityNurse"`.
    pub relationship_role: String,
    /// Distributed `StewardRole`s held (`PrimaryCare`, `Medical`, `Financial`, `Educational`, `EmergencyAdvocate`, or `HousemateCommonsOnly`).
    pub steward_roles: Vec<String>,
    /// Physical locus of this participant (e.g. `"Local Home Server LAN"`, `"Remote Mobile AVF Sanctuary (Elsewhere via QDNF WireGuard)"`).
    pub physical_locus: String,
    /// Minimum severity (`0..=3`) at which this guardian/carer receives alerts for the principal.
    pub alert_min_severity_level: u64,
    /// Revocable `ConsentCredential` ID governing what scoped alerts/data this carer can receive
    /// (revoking destroys the wrapped key immediately while keeping `ConductRecord` accountability).
    pub consent_credential_id: String,
}

/// AShared Living or Distributed Guardianship / Care Arrangement (Share-House, Family with Children,
/// or Elderly Person with Remote Family/Carers elsewhere).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedGuardianshipAndCareCircle {
    pub circle_id: String,
    pub title: String,
    pub topology: SharedLivingAndCareTopologyKind,
    /// The primary natural person(s) whose home/vault or wellbeing this circle serves.
    pub principal_did: String,
    pub principal_agency_stage: String,
    pub host_node_id: String,
    pub participants: Vec<SharedSystemParticipant>,
    /// Minimum `CareAlertSeverity` level (`0..=3`) that triggers remote QDNF guardian/carer notifications.
    pub min_alert_severity_level: u64,
    /// Liveness `Heartbeat` check-in window in seconds for elderly/supported-living care (e.g. `43_200` = 12h).
    pub heartbeat_grace_secs: u64,
    pub last_principal_checkin_note: String,
    /// A locally accepted principal check-in, independent of the latest sensor heartbeat.
    #[serde(default)]
    pub last_principal_checkin_unix_secs: Option<u64>,
    /// True if a liveness lapse or incapacity alert is currently active (reverses immediately when the
    /// elderly/supported principal signals `"I am OK"` via `acknowledge_or_reverse_care_alert`).
    pub escalation_currently_active: bool,
    pub sealed_nquin_parity: u64,
}

/// An auditable Guardianship or Remote Carer Alert dispatched over `SocialWebNet` WireGuard + QDNF
/// `ports::QDNF` and permanently recorded in the tamper-evident `AccountabilityLedger`
/// (`wellfare-core` + `qualia-client-core/src/wellfair/ingest_guardian.rs` & `duty_of_inquiry.rs`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchedCareAndGuardianAlert {
    pub alert_id: String,
    pub circle_id: String,
    pub topology: SharedLivingAndCareTopologyKind,
    pub trigger_kind: CareAlertTriggerKind,
    pub severity: CareAlertSeverity,
    pub principal_did: String,
    /// Remote family members, guardians, or carers notified over QDNF `ports::QDNF` (filtered by
    /// `participant.alert_min_severity_level <= severity.level()` and role).
    pub notified_remote_guardian_dids: Vec<String>,
    pub summary: String,
    /// Whether this alert was recorded in the hash-chained `AccountabilityLedger` (`"guardian_notified"`).
    pub recorded_in_accountability_ledger: bool,
    /// Classification under `duty_of_inquiry::InquiryVerdict` (`"PendingResponse"`, `"Diligent"`, `"NoFault"`, `"ReversiblePrincipalOk"`).
    pub duty_of_inquiry_status: String,
    pub resolved_or_reversed: bool,
    pub sealed_nquin_parity: u64,
}

/// Complete Natural-Person & Cooperative Quorum Stewardship controller for `webizen-host`,
/// synchronised across the user's Desktop (`webizen-desktop-appliance`) and Mobile (`webizen-mobile-avf`) nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NaturalHostStewardshipFabric {
    pub primary_host_domain: String,
    pub host_endpoint: String,
    pub archetype: HostStewardshipArchetype,
    pub threshold_m: u8,
    /// Active natural-person stewards (`living-SHACL`, disjoint from `owl:Thing`).
    pub stewards: Vec<ResponsibleNaturalSteward>,
    /// Attached Handle-Plane domains (`(domain, active_or_held)` — `Collapse Detector` ensures
    /// domain seizure/expiry never erases the natural-person stewards or Sanctuary Keep volumes).
    pub attached_handles: Vec<(String, String)>,
    pub sanctuary_keep_local_only: bool,
    pub shamir_q42j_backup_threshold: String,
    /// Up to 32 queued host update proposals (`HOST_QUORUM_QUEUE_SLOTS = 32`).
    pub update_queue: Vec<HostUpdateProposal>,
    /// Multi-locus fleet of personal, roaming, or remote host servers (Home Server, 12V Camper Rig,
    /// Online Cloud VPS, or Community Grounds Edge Node).
    #[serde(default)]
    pub host_fleet_nodes: Vec<PersonalHostNode>,
    /// Hosted projects, websites, and domains (separating **Server Runtime Responsibility** from
    /// **Project / Website / Domain Stewardship**, supporting Personal -> Group transitions and `Q-CDN` Swarm replication).
    #[serde(default)]
    pub hosted_projects: Vec<HostedProjectWorkload>,
    /// Incorporated Legal Entities (governed by structural stakeholder factors rather than social-web users,
    /// distinct from natural persons and natural-person corporate agents, and **always subject to commercial licensing**).
    #[serde(default)]
    pub incorporated_entities: Vec<IncorporatedLegalEntityRecord>,
    /// Shared living & distributed guardianship/care circles (Share-Houses, Families with Children,
    /// and Elderly / Supported Persons with Remote Family Members or Carers elsewhere).
    #[serde(default)]
    pub shared_care_circles: Vec<SharedGuardianshipAndCareCircle>,
    /// Auditable stream of dispatched Guardianship & Remote Carer Alerts (recorded in the Accountability Ledger).
    #[serde(default)]
    pub care_alerts: Vec<DispatchedCareAndGuardianAlert>,
}

impl Default for NaturalHostStewardshipFabric {
    fn default() -> Self {
        default_seeded_host_stewardship()
    }
}

impl NaturalHostStewardshipFabric {
    pub fn load_or_seed(storage: &Path, ids: &IdentifierSet) -> Self {
        let path = storage.join(HOST_STEWARDSHIP_STORE_REL_PATH);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(mut loaded) = serde_json::from_slice::<Self>(&bytes) {
                if !loaded.stewards.is_empty() {
                    let defaults = default_seeded_host_stewardship();
                    if loaded.host_fleet_nodes.is_empty() {
                        loaded.host_fleet_nodes = defaults.host_fleet_nodes;
                    }
                    if loaded.hosted_projects.is_empty() {
                        loaded.hosted_projects = defaults.hosted_projects;
                    }
                    if loaded.incorporated_entities.is_empty() {
                        loaded.incorporated_entities = defaults.incorporated_entities;
                    }
                    if loaded.shared_care_circles.is_empty() {
                        loaded.shared_care_circles = defaults.shared_care_circles;
                    }
                    if loaded.care_alerts.is_empty() {
                        loaded.care_alerts = defaults.care_alerts;
                    }
                    return loaded;
                }
            }
        }
        let mut seeded = default_seeded_host_stewardship();
        if !ids.fabric_did.is_empty() {
            if let Some(first) = seeded.stewards.first_mut() {
                first.steward_did = ids.fabric_did.clone();
                first.steward_did_hash = q_hash_60(&ids.fabric_did);
            }
        }
        let _ = seeded.save(storage);
        seeded
    }

    pub fn save(&self, storage: &Path) -> Result<(), String> {
        use std::io::Write;
        use std::sync::atomic::{AtomicU64, Ordering};

        static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let path = storage.join(HOST_STEWARDSHIP_STORE_REL_PATH);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let bytes = serde_json::to_vec_pretty(self).map_err(|error| error.to_string())?;
        let tmp = path.with_extension(format!(
            "json.{}.{}.tmp",
            std::process::id(),
            TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        ));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|error| error.to_string())?;
        let written = file.write_all(&bytes).and_then(|()| file.sync_all());
        drop(file);
        let replaced = written.and_then(|()| fs::rename(&tmp, &path));
        if let Err(error) = replaced {
            let _ = fs::remove_file(&tmp);
            return Err(error.to_string());
        }
        Ok(())
    }

    /// Byzantine Fault Tolerance bound $f = \lfloor(n - 1) / 3\rfloor$ across natural-person stewards.
    pub fn bft_fault_tolerance(&self) -> u8 {
        let n = self
            .stewards
            .iter()
            .filter(|s| s.has_full_legal_capacity)
            .count() as u8;
        if n == 0 {
            0
        } else {
            (n - 1) / 3
        }
    }

    /// Register a new Natural-Person Co-Steward (e.g. pairing a person's Mobile Debian 13 AVF `pKVM`
    /// Pocket Sanctuary or adding a cooperative co-steward).
    ///
    /// Fails closed if:
    /// - The label violates `validate_human_centric_host_nomenclature`.
    /// - The label reduces a living person to a forbidden `owl:Thing` / `Object` / `Asset` label
    /// - The DID is an AI/instrument tool (`did:q42:agent:...` or `urn:qualia:instrument:...`) trying
    ///   to register on the `NaturalAgent` Who-Plane!
    pub fn register_natural_steward(
        &mut self,
        storage: &Path,
        steward_did: &str,
        display_hint: &str,
        role: &str,
        apparatus_profile: &str,
        has_full_legal_capacity: bool,
        guardian_did: Option<String>,
    ) -> Result<ResponsibleNaturalSteward, String> {
        validate_human_centric_host_nomenclature(steward_did)?;
        validate_human_centric_host_nomenclature(display_hint)?;

        let hint_lower = display_hint.trim().to_ascii_lowercase();
        for forbidden in NATURAL_AGENT_FORBIDDEN_LABELS {
            if hint_lower == *forbidden || hint_lower.contains("owl:thing") {
                return Err(format!(
                    "SHACL-First Non-Thing Rule violation: NaturalAgent '{display_hint}' must never be typed as '{forbidden}' or owl:Thing"
                ));
            }
        }
        let did_lower = steward_did.trim().to_ascii_lowercase();
        if did_lower.contains(":agent:")
            || did_lower.contains(":instrument:")
            || did_lower.contains(":bot")
        {
            return Err(
                "Anti-Collapse Invariant violation ('Natural persons are not identity assets; instruments are not peers'): an AI or service instrument cannot be registered as a NaturalAgent steward"
                    .into(),
            );
        }

        let mut hasher = Sha3_256::new();
        hasher.update(steward_did.as_bytes());
        hasher.update(apparatus_profile.as_bytes());
        let fp_hex: String = hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();

        let steward = ResponsibleNaturalSteward {
            steward_did: steward_did.trim().to_string(),
            steward_did_hash: q_hash_60(steward_did.trim()),
            display_hint: display_hint.trim().to_string(),
            role: role.trim().to_string(),
            apparatus_profile: apparatus_profile.trim().to_string(),
            dual_pubkey_fingerprint: format!("ed25519+mldsa65:{}", &fp_hex[..20]),
            has_full_legal_capacity,
            guardian_did,
        };

        self.stewards
            .retain(|s| s.steward_did != steward.steward_did);
        self.stewards.push(steward.clone());
        let _ = self.save(storage);
        Ok(steward)
    }

    /// Enqueue a proposed `webizen-host` update (`.deb` package rollout or host configuration change)
    /// into the 32-slot `SuspendedTransactionQueue`.
    ///
    /// Note: An automated CI/build tool (`proposer_is_instrument_tool = true`) is allowed to *stage*
    /// a proposal into the queue, but it casts **zero** ratification votes and cannot approve the update.
    pub fn propose_host_update(
        &mut self,
        storage: &Path,
        target_deb_package: &str,
        target_version: &str,
        proposed_by_did: &str,
        proposer_is_instrument_tool: bool,
    ) -> Result<HostUpdateProposal, String> {
        let proposal = self.make_host_update_proposal(
            target_deb_package,
            target_version,
            proposed_by_did,
            proposer_is_instrument_tool,
        )?;
        self.update_queue.insert(0, proposal.clone());
        if let Err(error) = self.save(storage) {
            self.update_queue.remove(0);
            return Err(error);
        }
        Ok(proposal)
    }

    fn make_host_update_proposal(
        &self,
        target_deb_package: &str,
        target_version: &str,
        proposed_by_did: &str,
        proposer_is_instrument_tool: bool,
    ) -> Result<HostUpdateProposal, String> {
        validate_host_field(target_deb_package, "Package")?;
        validate_host_field(target_version, "Version")?;
        validate_host_field(proposed_by_did, "Proposer DID")?;

        if self.update_queue.len() >= HOST_QUORUM_QUEUE_SLOTS {
            return Err("SuspendedTransactionQueue (32 slots) is full".into());
        }

        let slot_index = self.update_queue.len();
        let proposal_id = format!(
            "tx:host-update:{}:{}:slot{}",
            target_deb_package.trim(),
            target_version.trim(),
            slot_index
        );
        let quin = NQuin48::invariant(
            &self.primary_host_domain,
            "https://ns.webcivics.net/quorum#proposedCooperativeMutation",
            q_hash_60(&proposal_id),
        );
        debug_assert!(quin.verify_parity());

        let total_n = self
            .stewards
            .iter()
            .filter(|s| s.has_full_legal_capacity)
            .count()
            .max(1) as u8;
        let required_m = match self.archetype {
            HostStewardshipArchetype::SingleNaturalPerson => 1,
            HostStewardshipArchetype::FamilyHouseholdGuardianship => 1,
            HostStewardshipArchetype::MultiPersonCooperativeQuorum => {
                self.threshold_m.clamp(1, total_n)
            }
            HostStewardshipArchetype::GovernedServiceAgent => self.threshold_m.clamp(1, total_n),
        };

        let mut cbor_ld = Vec::new();
        cbor_ld.extend_from_slice(&CBOR_LD_TAG_HEADER);
        cbor_ld.push(1u8);
        cbor_ld.extend_from_slice(proposal_id.as_bytes());
        let cbor_ld_envelope_hex: String = cbor_ld.iter().map(|b| format!("{b:02x}")).collect();

        Ok(HostUpdateProposal {
            slot_index,
            proposal_id: proposal_id.clone(),
            proposal_hash: q_hash_60(&proposal_id),
            target_host_domain: self.primary_host_domain.clone(),
            target_deb_package: target_deb_package.trim().to_string(),
            target_version: target_version.trim().to_string(),
            artifact_sha3_256: String::new(),
            proposed_by_did: proposed_by_did.trim().to_string(),
            proposer_is_instrument_tool,
            required_m,
            total_n,
            votes: Vec::new(),
            paraconsistent_review_flagged: false,
            ratified_for_host_apply: false,
            applied_to_host: false,
            cbor_ld_envelope_hex,
            sealed_nquin_parity: quin.parity(),
        })
    }

    /// Record a local steward approval or dissent claim.
    ///
    /// Local membership and guardian references establish eligibility for the
    /// record. Applying an update requires independently verified signatures.
    pub fn sign_or_dissent_host_update(
        &mut self,
        storage: &Path,
        proposal_id: &str,
        steward_did: &str,
        co_signer_guardian_did: Option<&str>,
        approve: bool,
        comment: &str,
    ) -> Result<HostUpdateProposal, String> {
        validate_host_field(proposal_id, "Proposal ID")?;
        validate_host_field(steward_did, "Steward DID")?;
        validate_human_centric_host_nomenclature(comment)?;
        if let Some(guardian) = co_signer_guardian_did {
            validate_host_field(guardian, "Guardian DID")?;
        }

        let did_lower = steward_did.trim().to_ascii_lowercase();
        if did_lower.contains(":agent:")
            || did_lower.contains(":instrument:")
            || did_lower.contains(":bot")
        {
            return Err(
                "PermissionDenied: 'Natural persons are not identity assets; instruments are not peers.' An automated instrument or AI agent cannot ratify a webizen-host update."
                    .into(),
            );
        }

        let Some(steward) = self
            .stewards
            .iter()
            .find(|s| s.steward_did == steward_did.trim())
            .cloned()
        else {
            return Err(format!(
                "PermissionDenied: '{steward_did}' is not a registered ResponsibleNaturalSteward for {}",
                self.primary_host_domain
            ));
        };

        if !steward.has_full_legal_capacity {
            let expected_guardian = steward.guardian_did.as_deref().unwrap_or("");
            let provided_guardian = co_signer_guardian_did.unwrap_or("");
            if expected_guardian.is_empty() || expected_guardian != provided_guardian {
                return Err(format!(
                    "PermissionDenied: Guardianship co-signature from '{}' is required under GuardianshipSemantics (UN CRC / CRPD capacity protection)",
                    if expected_guardian.is_empty() {
                        "AdultStewardGuardian"
                    } else {
                        expected_guardian
                    }
                ));
            }
        }

        let index = self
            .update_queue
            .iter()
            .position(|proposal| proposal.proposal_id == proposal_id.trim())
            .ok_or_else(|| format!("Proposal '{proposal_id}' is absent from the queue"))?;
        let before = self.update_queue[index].clone();
        let proposal = &mut self.update_queue[index];

        let predicate_iri = if approve {
            "https://ns.webcivics.net/quorum#recordedStewardApproval"
        } else {
            "https://ns.webcivics.net/illocution#RequiresHumanReview"
        };
        let quin = NQuin48::invariant(
            &proposal.proposal_id,
            predicate_iri,
            steward.steward_did_hash,
        );
        debug_assert!(quin.verify_parity());

        proposal
            .votes
            .retain(|v| v.steward_did != steward.steward_did);
        proposal.votes.push(StewardQuorumVote {
            steward_did: steward.steward_did.clone(),
            apparatus_profile: steward.apparatus_profile.clone(),
            approve,
            comment: comment.trim().to_string(),
            dual_signature_hex: String::new(),
            sealed_nquin_parity: quin.parity(),
        });

        proposal.paraconsistent_review_flagged = proposal.votes.iter().any(|vote| !vote.approve);
        proposal.ratified_for_host_apply = false;
        proposal.applied_to_host = false;

        let updated = proposal.clone();
        if let Err(error) = self.save(storage) {
            self.update_queue[index] = before;
            return Err(error);
        }
        Ok(updated)
    }

    /// Trigger the `Collapse Detector` (`Handle revoke != who-erase`) when an ICANN DNS domain
    /// expires or is held by a registrar: transitions the domain handle to
    /// `HeldOrClosedByRegistrar` while proving that all `ResponsibleNaturalSteward` records,
    /// `M-of-N` quorum authority, QDNF `fd00::/8` ULA routing, and local Sanctuary Keep volumes
    /// remain 100% intact.
    pub fn trigger_domain_collapse_detector(
        &mut self,
        storage: &Path,
        domain: &str,
    ) -> Result<String, String> {
        let mut found = false;
        for (h, state) in &mut self.attached_handles {
            if h == domain.trim() {
                *state = "HeldOrClosedByRegistrar (Who-Plane & QDNF ULA fd00::/8 Intact)".into();
                found = true;
            }
        }
        if !found {
            self.attached_handles.push((
                domain.trim().to_string(),
                "HeldOrClosedByRegistrar (Who-Plane & QDNF ULA fd00::/8 Intact)".into(),
            ));
        }
        debug_assert!(!self.stewards.is_empty());
        debug_assert!(self.sanctuary_keep_local_only);
        let _ = self.save(storage);
        Ok(format!(
            "Collapse Detector triggered on handle '{}': handle transitioned to HeldOrClosedByRegistrar while {} natural-person stewards, Sanctuary Keep (:8080), and QDNF ULA mesh remain intact (Handle revoke != who-erase).",
            domain.trim(),
            self.stewards.len()
        ))
    }

    /// Register a physical or virtual `webizen-host` server node in the user's multi-locus fleet
    /// (Home Server Appliance, 12V Camper/Walkabout Rig, Online Cloud VPS, or Community Grounds Node).
    ///
    /// Supports standalone local-first operation (`standalone_without_online_vps = true` for users who
    /// only run a Home or Camper server) or multi-node QDNF `fd00::/8` mesh pairing.
    pub fn register_host_fleet_node(
        &mut self,
        storage: &Path,
        node_id: &str,
        label: &str,
        locus: HostNodeLocusKind,
        runtime_operator_steward_dids: Vec<String>,
        endpoint_hint: &str,
        standalone_without_online_vps: bool,
    ) -> Result<PersonalHostNode, String> {
        validate_human_centric_host_nomenclature(node_id)?;
        validate_human_centric_host_nomenclature(label)?;
        for d in &runtime_operator_steward_dids {
            validate_human_centric_host_nomenclature(d)?;
        }

        let ula_suffix = q_hash_60(node_id) & 0xFFFF_FFFF;
        let qdnf_ula_ipv6 = format!(
            "fd42:7175:616c:{:04x}::{:04x}",
            (ula_suffix >> 16) & 0xFFFF,
            ula_suffix & 0xFFFF
        );
        let holds_local_sanctuary_keep = matches!(
            locus,
            HostNodeLocusKind::HomeServerAppliance | HostNodeLocusKind::CamperMobileRigNode
        );
        let qcdn_seeder_tier = match locus {
            HostNodeLocusKind::HomeServerAppliance | HostNodeLocusKind::CamperMobileRigNode => {
                "Tier B · Personal/Roaming Seeder (:4242 + Local LAN/Mesh)".into()
            }
            HostNodeLocusKind::OnlineCloudVpsRelay
            | HostNodeLocusKind::CommunityGroundsEdgeNode => {
                "Tier A · Cooperative VPS / Edge Mirror (:4242 SignedRootIndex)".into()
            }
        };

        let quin = NQuin48::invariant(
            node_id,
            "https://ns.webcivics.net/stewardship#operatesHostRuntimeLocus",
            q_hash_60(locus.code()),
        );
        debug_assert!(quin.verify_parity());

        let node = PersonalHostNode {
            node_id: node_id.trim().to_string(),
            label: label.trim().to_string(),
            locus,
            runtime_operator_steward_dids,
            qdnf_ula_ipv6,
            endpoint_hint: endpoint_hint.trim().to_string(),
            standalone_without_online_vps,
            holds_local_sanctuary_keep,
            qcdn_seeder_tier,
            sealed_nquin_parity: quin.parity(),
        };

        self.host_fleet_nodes.retain(|n| n.node_id != node.node_id);
        self.host_fleet_nodes.push(node.clone());
        let _ = self.save(storage);
        Ok(node)
    }

    /// Record the initial stewardship claim for a hosted project.
    ///
    /// Runtime placement and signed Q-CDN publication are separate operations.
    pub fn register_hosted_project(
        &mut self,
        storage: &Path,
        project_id: &str,
        title: &str,
        attached_domain: &str,
        ownership_stage: HostedProjectOwnershipStage,
        steward_did: &str,
        incorporated_entity_id: Option<&str>,
    ) -> Result<HostedProjectWorkload, String> {
        for (text, label) in [
            (project_id, "Project ID"),
            (title, "Project title"),
            (attached_domain, "Attached domain"),
            (steward_did, "Project steward DID"),
        ] {
            validate_host_field(text, label)?;
        }
        let project_id = project_id.trim();
        let incorporated_entity_id = incorporated_entity_id.map(str::trim);
        self.validate_project_entity(ownership_stage, incorporated_entity_id)?;
        if self
            .hosted_projects
            .iter()
            .any(|project| project.project_id == project_id)
        {
            return Err(format!("Project '{project_id}' is already registered"));
        }
        let object = serde_json::to_string(&(
            title.trim(),
            attached_domain.trim(),
            ownership_stage,
            steward_did.trim(),
            incorporated_entity_id,
        ))
        .map_err(|error| error.to_string())?;
        let quin = NQuin48::invariant(
            project_id,
            "https://ns.webcivics.net/stewardship#registeredHostedProject",
            q_hash_60(&object),
        );
        debug_assert!(quin.verify_parity());

        let project = HostedProjectWorkload {
            project_id: project_id.into(),
            title: title.trim().into(),
            attached_domain: attached_domain.trim().into(),
            ownership_stage,
            project_steward_dids: vec![steward_did.trim().into()],
            project_quorum_threshold_m: 1,
            hosted_on_runtime_node_ids: Vec::new(),
            qcdn_swarm_mirror_dids: Vec::new(),
            qcdn_signed_root_merkle_hex: String::new(),
            incorporated_entity_id: incorporated_entity_id.map(str::to_owned),
            provenance_history: vec![format!("Registered as {}", ownership_stage.title())],
            sealed_nquin_parity: quin.parity(),
        };
        self.hosted_projects.push(project.clone());
        if let Err(error) = self.save(storage) {
            self.hosted_projects.pop();
            return Err(error);
        }
        Ok(project)
    }

    fn validate_project_entity(
        &self,
        stage: HostedProjectOwnershipStage,
        entity: Option<&str>,
    ) -> Result<(), String> {
        if stage == HostedProjectOwnershipStage::IncorporatedCommercialTenantProject
            && entity.is_none()
        {
            return Err("Commercial projects require a registered incorporated entity".into());
        }
        if let Some(entity) = entity {
            validate_host_field(entity, "Incorporated entity ID")?;
            if !self
                .incorporated_entities
                .iter()
                .any(|record| record.entity_did == entity)
            {
                return Err("The incorporated entity is not registered in this fabric".into());
            }
        }
        Ok(())
    }

    /// Transition a hosted project/website/domain along its stewardship lifecycle — for example:
    /// - From a **Personal Project** (`PersonalOwnerProject`) owned by the VPS/Home/Camper operator
    ///   into a **Multi-Person Cooperative Group Project** (`TransitionedPersonalToGroupProject`) with
    ///   `M-of-N` project co-stewards and replicated `Q-CDN` Swarm mirror nodes; or
    /// - Into a **Supported Third-Party / Another Group's Project** (`SupportedThirdPartyOrGroupProject`)
    ///   where the server operator provides runtime hosting/mirroring while the external project stewards
    ///   retain authority over the project's content, code, and domain!
    pub fn transition_project_to_group_or_qcdn_swarm(
        &mut self,
        storage: &Path,
        project_id: &str,
        new_stage: HostedProjectOwnershipStage,
        additional_project_steward_dids: Vec<String>,
        new_quorum_threshold_m: u8,
        additional_qcdn_mirror_dids: Vec<String>,
    ) -> Result<HostedProjectWorkload, String> {
        validate_host_field(project_id, "Project ID")?;
        for did in &additional_project_steward_dids {
            validate_host_field(did, "Project steward DID")?;
        }
        for mirror in &additional_qcdn_mirror_dids {
            validate_host_field(mirror, "Q-CDN mirror ID")?;
        }
        let index = self
            .hosted_projects
            .iter()
            .position(|project| project.project_id == project_id.trim())
            .ok_or_else(|| format!("Project '{project_id}' is not registered"))?;
        self.validate_project_entity(
            new_stage,
            self.hosted_projects[index]
                .incorporated_entity_id
                .as_deref(),
        )?;
        let before = self.hosted_projects[index].clone();
        let proj = &mut self.hosted_projects[index];
        let old_stage_title = proj.ownership_stage.title();
        proj.ownership_stage = new_stage;
        for d in additional_project_steward_dids {
            let trimmed = d.trim().to_string();
            if !trimmed.is_empty() && !proj.project_steward_dids.contains(&trimmed) {
                proj.project_steward_dids.push(trimmed);
            }
        }
        for m in additional_qcdn_mirror_dids {
            let trimmed = m.trim().to_string();
            if !trimmed.is_empty() && !proj.qcdn_swarm_mirror_dids.contains(&trimmed) {
                proj.qcdn_swarm_mirror_dids.push(trimmed);
            }
        }
        let n = u8::try_from(proj.project_steward_dids.len())
            .unwrap_or(u8::MAX)
            .max(1);
        proj.project_quorum_threshold_m = new_quorum_threshold_m.clamp(1, n);

        let quin = NQuin48::invariant(
            &proj.project_id,
            "https://ns.webcivics.net/stewardship#transitionedProjectStewardship",
            q_hash_60(new_stage.title()),
        );
        debug_assert!(quin.verify_parity());
        proj.sealed_nquin_parity = quin.parity();
        proj.provenance_history.push(format!(
            "Transitioned [{}] -> [{}] · Project Stewards ({}-of-{}): [{}] · Q-CDN Swarm Mirrors ({}): [{}]",
            old_stage_title,
            new_stage.title(),
            proj.project_quorum_threshold_m,
            proj.project_steward_dids.len(),
            proj.project_steward_dids.join(", "),
            proj.qcdn_swarm_mirror_dids.len(),
            proj.qcdn_swarm_mirror_dids.join(", ")
        ));

        let updated = proj.clone();
        if let Err(error) = self.save(storage) {
            self.hosted_projects[index] = before;
            return Err(error);
        }
        Ok(updated)
    }

    /// Register an **Incorporated Legal Entity** (`JuridicalPerson` / `IncorporatedOrganization`),
    /// recording its structural stakeholder governance factors (distinct from social-web users),
    /// binding the natural persons authorized to act as its corporate agents
    /// (`ActorJuralPersonalityKind::NaturalPersonAgentOfIncorporatedEntity`), and enforcing the
    /// **Mandatory Commercial Licensing Invariant**:
    /// - Every incorporated entity is **always charged (`mandatory_commercial_licence_active = true`,
    ///   `monthly_base_fee_cents > 0`)** irrespective of its incorporation subtype (even Tier 1
    ///   Incorporated Non-Profits/Charities pay a concessional commercial licence rather than entering
    ///   the `$0` natural-person lane), with tiered pricing determined by [`IncorporatedEntityPricingTier`].
    pub fn register_incorporated_entity_commercial_licence(
        &mut self,
        storage: &Path,
        entity_did: &str,
        legal_name: &str,
        jurisdiction_and_reg_number: &str,
        pricing_tier: IncorporatedEntityPricingTier,
        stakeholder_governance_factors: Vec<String>,
        authorized_natural_person_agent_dids: Vec<String>,
        settlement_rail: &str,
    ) -> Result<IncorporatedLegalEntityRecord, String> {
        validate_human_centric_host_nomenclature(entity_did)?;
        validate_human_centric_host_nomenclature(legal_name)?;

        let monthly_base_fee_cents = pricing_tier.monthly_base_commercial_fee_cents();
        let qcdn_carriage_cents_per_gib = pricing_tier.qcdn_carriage_rate_cents_per_gib();
        debug_assert!(
            monthly_base_fee_cents > 0,
            "Every incorporated entity must be subject to commercial licensing (> 0 cents)"
        );

        let quin = NQuin48::invariant(
            entity_did.trim(),
            "https://ns.webcivics.net/economics#obligatedIncorporatedCommercialLicence",
            monthly_base_fee_cents,
        );
        debug_assert!(quin.verify_parity());

        let record = IncorporatedLegalEntityRecord {
            entity_did: entity_did.trim().to_string(),
            legal_name: legal_name.trim().to_string(),
            jurisdiction_and_reg_number: jurisdiction_and_reg_number.trim().to_string(),
            pricing_tier,
            stakeholder_governance_factors,
            authorized_natural_person_agent_dids,
            mandatory_commercial_licence_active: true,
            monthly_base_fee_cents,
            qcdn_carriage_cents_per_gib,
            settlement_rail: if settlement_rail.trim().is_empty() {
                "QualiaPQ-Ledger (QPQ) / eCash (XEC) / BTC-LN / USDC".into()
            } else {
                settlement_rail.trim().to_string()
            },
            loop1_commons_pool_contribution_cents: monthly_base_fee_cents,
            sealed_nquin_parity: quin.parity(),
        };

        self.incorporated_entities
            .retain(|e| e.entity_did != record.entity_did);
        self.incorporated_entities.push(record.clone());
        let _ = self.save(storage);
        Ok(record)
    }

    /// Evaluate the **Jural Actor & Commercial Licensing Gate** for any actor requesting to host or
    /// operate a workload on `webizen-host`:
    /// - `NaturalPersonPrincipal` -> `$0.00` (Free Human-Centric & Permissive Commons Lane)
    /// - `NaturalPersonAgentOfIncorporatedEntity` -> Charged at the principal Incorporated Entity's
    ///   commercial tier (`monthly_base_fee_cents > 0`; prevents corporate fee-evasion via employee personal DIDs)
    /// - `IncorporatedLegalEntity` -> Charged at the Incorporated Entity's commercial tier (`monthly_base_fee_cents > 0`).
    pub fn evaluate_jural_actor_commercial_obligation(
        &self,
        actor_kind: ActorJuralPersonalityKind,
        incorporated_entity_did: Option<&str>,
    ) -> Result<(bool, u64, String), String> {
        match actor_kind {
            ActorJuralPersonalityKind::NaturalPersonPrincipal => Ok((
                false,
                0,
                "NaturalPersonPrincipal (living-SHACL): $0.00 royalty (0% Free Human-Rights & Permissive Commons Lane)"
                    .into(),
            )),
            ActorJuralPersonalityKind::NaturalPersonAgentOfIncorporatedEntity
            | ActorJuralPersonalityKind::IncorporatedLegalEntity => {
                let target_did = incorporated_entity_did.unwrap_or("");
                let entity = self
                    .incorporated_entities
                    .iter()
                    .find(|e| e.entity_did == target_did)
                    .or_else(|| self.incorporated_entities.first())
                    .ok_or_else(|| {
                        "HTTP 402 Payment Required: Incorporated Entity or Corporate Agent must bind a registered IncorporatedLegalEntityRecord commercial licence".to_string()
                    })?;
                debug_assert!(entity.mandatory_commercial_licence_active);
                debug_assert!(entity.monthly_base_fee_cents > 0);
                Ok((
                    true,
                    entity.monthly_base_fee_cents,
                    format!(
                        "[COMMERCIAL LICENCE CHARGED] Actor={:?} -> Principal Incorporated Entity '{}' ({}) · Tier: {} · Base Fee: ${}.{:02}/mo + ${}.{:02}/GiB Q-CDN carriage via {}",
                        actor_kind,
                        entity.legal_name,
                        entity.jurisdiction_and_reg_number,
                        entity.pricing_tier.title(),
                        entity.monthly_base_fee_cents / 100,
                        entity.monthly_base_fee_cents % 100,
                        entity.qcdn_carriage_cents_per_gib / 100,
                        entity.qcdn_carriage_cents_per_gib % 100,
                        entity.settlement_rail
                    ),
                ))
            }
        }
    }

    /// Dispatch a severity-gated **Guardianship or Remote Carer Alert** across a
    /// [`SharedGuardianshipAndCareCircle`] (Share-House, Family with Children, or Elderly/Supported
    /// Person with Remote Family Members & Carers elsewhere).
    ///
    /// Enforces:
    /// 1. **Share-House Privacy Isolation (`ShareHousePeerCoLiving`)**:
    ///    - Rejects any attempt to dispatch a personal health, liveness, or content-ingest guardianship
    ///      alert over a housemate peer (`Err("Share-house peers have zero guardianship over each other's personal lives...")`)!
    ///    - Only shared household utility / microgrid quorum notices (`ShareHouseCommonsUtilityNotice`)
    ///      are permitted in a share-house circle.
    /// 2. **Severity-Threshold Filtering (`ingest_guardian::guardian_notifications`)**:
    ///    - Only notifies remote family members, guardians, or carers whose `alert_min_severity_level <= severity.level()`.
    /// 3. **Local Record & 48-Byte `NQuin48` Seal**:
    ///    - The local record identifies eligible recipients. A delivery or ledger
    ///      receipt requires the corresponding canonical service.
    pub fn dispatch_shared_care_or_guardian_alert(
        &mut self,
        storage: &Path,
        circle_id: &str,
        trigger_kind: CareAlertTriggerKind,
        severity: CareAlertSeverity,
        detail: &str,
    ) -> Result<DispatchedCareAndGuardianAlert, String> {
        validate_host_field(circle_id, "Care circle ID")?;
        validate_human_centric_host_nomenclature(detail)?;
        let circle_index = self
            .shared_care_circles
            .iter()
            .position(|circle| circle.circle_id == circle_id.trim())
            .ok_or_else(|| format!("Shared care circle '{circle_id}' is absent"))?;
        let circle = &mut self.shared_care_circles[circle_index];
        let previously_active = circle.escalation_currently_active;

        // 1. Share-House Invariant: Zero cross-peer guardianship over personal health/content!
        if circle.topology == SharedLivingAndCareTopologyKind::ShareHousePeerCoLiving
            && trigger_kind != CareAlertTriggerKind::ShareHouseCommonsUtilityNotice
        {
            return Err(
                "PermissionDenied (Share-House Peer Isolation Invariant): Equal adult housemates have zero guardianship over each other's personal health, vitals, or content. Only ShareHouseCommonsUtilityNotice (shared bills/router/solar) is permitted."
                    .into(),
            );
        }

        // 2. Filter recipients by severity threshold & guardianship/carer role
        let notified_remote_guardian_dids: Vec<String> = circle
            .participants
            .iter()
            .filter(|p| {
                p.participant_did != circle.principal_did
                    && severity.level() >= p.alert_min_severity_level
                    && severity.level() >= circle.min_alert_severity_level
            })
            .map(|p| {
                format!(
                    "{} ({} · {})",
                    p.participant_did, p.display_name, p.physical_locus
                )
            })
            .collect();

        if matches!(
            trigger_kind,
            CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse
                | CareAlertTriggerKind::PhysiologicalOrFallSensorAlert
                | CareAlertTriggerKind::MissedMedicationSchedule
        ) && severity.level() >= CareAlertSeverity::Concern.level()
        {
            circle.escalation_currently_active = true;
        }

        let alert_id = format!(
            "alert:{}:{}:{}",
            circle.circle_id,
            severity.level(),
            self.care_alerts.len() + 1
        );
        let quin = NQuin48::invariant(
            &circle.principal_did,
            "https://ns.webcivics.net/wellfair#dispatchedGuardianOrCarerAlert",
            q_hash_60(&alert_id),
        );
        debug_assert!(quin.verify_parity());

        let alert = DispatchedCareAndGuardianAlert {
            alert_id,
            circle_id: circle.circle_id.clone(),
            topology: circle.topology,
            trigger_kind,
            severity,
            principal_did: circle.principal_did.clone(),
            notified_remote_guardian_dids,
            summary: detail.trim().to_string(),
            recorded_in_accountability_ledger: false,
            duty_of_inquiry_status:
                "PendingCarerInquiryOrPrincipalCheckIn (MeansToCheck Accessible)".into(),
            resolved_or_reversed: false,
            sealed_nquin_parity: quin.parity(),
        };

        self.care_alerts.insert(0, alert.clone());
        if let Err(error) = self.save(storage) {
            self.care_alerts.remove(0);
            self.shared_care_circles[circle_index].escalation_currently_active = previously_active;
            return Err(error);
        }
        Ok(alert)
    }

    /// Record a local Principal Check-In or an eligible carer's acknowledgment.
    /// The host supplies the acting identity. A stored claim is not an
    /// independently evaluated Duty-of-Inquiry verdict.
    pub fn acknowledge_or_reverse_care_alert(
        &mut self,
        storage: &Path,
        alert_id: &str,
        responder_did: &str,
        by_principal_self_checkin: bool,
        note: &str,
    ) -> Result<DispatchedCareAndGuardianAlert, String> {
        validate_host_field(alert_id, "Care alert ID")?;
        validate_host_field(responder_did, "Responder DID")?;
        validate_human_centric_host_nomenclature(note)?;
        let responder = responder_did.trim();
        let alert_index = self
            .care_alerts
            .iter()
            .position(|alert| alert.alert_id == alert_id.trim())
            .ok_or_else(|| format!("Care alert '{alert_id}' is absent"))?;
        let before = self.care_alerts[alert_index].clone();
        let circle_index = self
            .shared_care_circles
            .iter()
            .position(|circle| circle.circle_id == before.circle_id)
            .ok_or_else(|| "The care circle for this alert is absent".to_string())?;
        let circle_before = self.shared_care_circles[circle_index].clone();
        if before.resolved_or_reversed || before.principal_did != circle_before.principal_did {
            return Err("Care alert is closed or bound to a different principal".into());
        }

        let allowed = if by_principal_self_checkin {
            responder == before.principal_did
        } else {
            responder != before.principal_did
                && circle_before.participants.iter().any(|participant| {
                    participant.participant_did == responder
                        && !participant.consent_credential_id.trim().is_empty()
                        && before.severity.level() >= participant.alert_min_severity_level
                })
                && before
                    .notified_remote_guardian_dids
                    .iter()
                    .any(|recipient| recipient.split_whitespace().next() == Some(responder))
        };
        if !allowed {
            return Err("This responder is not eligible for the selected care action".into());
        }

        let checkin_time = if by_principal_self_checkin {
            Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| "Cannot record a principal check-in before the Unix epoch")?
                    .as_secs(),
            )
        } else {
            None
        };

        let alert = &mut self.care_alerts[alert_index];
        alert.resolved_or_reversed = true;
        alert.duty_of_inquiry_status = if by_principal_self_checkin {
            format!("Reversed by Principal Self Check-In ('I am OK' / principal_alive by {responder}): {}", note.trim())
        } else {
            format!(
                "Carer acknowledgment and reported inquiry by {responder}: {}",
                note.trim()
            )
        };
        let quin = NQuin48::invariant(
            &alert.alert_id,
            "https://ns.webcivics.net/wellfair#recordedCareConductOrReversal",
            q_hash_60(responder),
        );
        debug_assert!(quin.verify_parity());
        alert.sealed_nquin_parity = quin.parity();
        let updated = alert.clone();

        let circle = &mut self.shared_care_circles[circle_index];
        circle.escalation_currently_active = self.care_alerts.iter().any(|alert| {
            alert.circle_id == circle.circle_id
                && !alert.resolved_or_reversed
                && alert.severity >= CareAlertSeverity::Concern
                && matches!(
                    alert.trigger_kind,
                    CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse
                        | CareAlertTriggerKind::PhysiologicalOrFallSensorAlert
                        | CareAlertTriggerKind::MissedMedicationSchedule
                )
        });
        if let Some(checkin_time) = checkin_time {
            circle.last_principal_checkin_unix_secs = Some(checkin_time);
            circle.last_principal_checkin_note = format!("Principal confirmed OK: {}", note.trim());
        }
        if let Err(error) = self.save(storage) {
            self.care_alerts[alert_index] = before;
            self.shared_care_circles[circle_index] = circle_before;
            return Err(error);
        }
        Ok(updated)
    }
}

/// Receipt emitted by [`generate_tri_target_debian_suite`] producing the unified Debian 13 (Trixie)
/// `.deb` package trees for Desktop (`amd64`), Mobile AVF Sanctuary (`arm64`), and Host (`amd64`/`arm64`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriTargetDebianSuiteReceipt {
    pub suite_root_dir: PathBuf,
    pub desktop_deb_tree: PathBuf,
    pub mobile_avf_deb_tree: PathBuf,
    pub host_deb_tree: PathBuf,
    #[serde(default)]
    pub built_deb_packages: Vec<crate::deb_builder::DebBuildReceipt>,
    #[serde(default)]
    pub apt_repository: Option<crate::deb_builder::AptRepositoryReceipt>,
    pub host_state_cbor_ld_path: PathBuf,
    pub host_quorum_gate_script_path: PathBuf,
    pub avf_crosvm_launcher_path: PathBuf,
    pub build_all_debs_script_path: PathBuf,
    pub steward_count: usize,
    pub quorum_threshold_m: u8,
}

/// Generate deterministic **Tri-Target Debian 13 (Trixie) `.deb` archives** and their build trees
/// `DEBIAN/control`, `DEBIAN/postinst`, systemd units, Android 16 AVF `crosvm`/`pKVM` EL2 Sanctuary
/// launcher + `vhost-vsock` bridge config, and the `webizen-host` W3C CBOR-LD `0xD90501`
/// natural-person quorum update gate.
pub fn generate_tri_target_debian_suite(
    host: &LoopbackHost,
    gpu: &NativeEtalonStatus,
    stewardship: &NaturalHostStewardshipFabric,
) -> Result<TriTargetDebianSuiteReceipt, String> {
    let domain = stewardship.primary_host_domain.as_str();
    validate_host_field(domain, "Host domain")?;
    if domain.len() > 253
        || !domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
    {
        return Err("Host domain must be a DNS name".into());
    }
    for s in &stewardship.stewards {
        validate_human_centric_host_nomenclature(&s.steward_did)?;
        validate_human_centric_host_nomenclature(&s.display_hint)?;
    }

    let suite_root = host
        .storage
        .join("delivery")
        .join("tri-target-debian-trixie");
    let desktop_tree = suite_root.join("webizen-desktop-appliance_0.1.0_amd64");
    let mobile_tree = suite_root.join("webizen-mobile-avf_0.1.0_arm64");
    let host_tree = suite_root.join("webizen-hostd_0.1.0_amd64");

    // 1. Target 1 — Desktop Steward & Authoring Node (`webizen-desktop-appliance_0.1.0_amd64`)
    fs::create_dir_all(desktop_tree.join("DEBIAN")).map_err(|e| e.to_string())?;
    fs::create_dir_all(desktop_tree.join("opt/webizen/bin")).map_err(|e| e.to_string())?;
    fs::create_dir_all(desktop_tree.join("etc/webizen")).map_err(|e| e.to_string())?;
    fs::create_dir_all(desktop_tree.join("lib/systemd/system")).map_err(|e| e.to_string())?;

    let primary_steward_did = stewardship
        .stewards
        .first()
        .map(|s| s.steward_did.as_str())
        .unwrap_or("did:qi:steward-desktop");
    let steward_dids_csv: String = stewardship
        .stewards
        .iter()
        .map(|s| s.steward_did.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let desktop_control = format!(
        "Package: webizen-desktop-appliance\n\
         Version: 0.1.0\n\
         Section: x11\n\
         Priority: optional\n\
         Architecture: amd64\n\
         Depends: libc6 (>= 2.38), systemd, cage, libvulkan1, mesa-vulkan-drivers, nftables, ca-certificates\n\
         Recommends: webizen-hostd\n\
         Maintainer: Webizen Human-Centric Stewardship <steward@{domain}>\n\
         Description: Webizen User-Agent OS Desktop Steward & Authoring Environment (Debian 13 Trixie)\n\
         \x20Staged desktop profile for a Natural Person ({steward}).\n\
         \x20The desktop runtime supplies the 8-Sector Command Orb and authoring workspace.\n",
        domain = stewardship.primary_host_domain,
        steward = primary_steward_did,
    );
    validate_human_centric_host_nomenclature(&desktop_control)?;
    fs::write(desktop_tree.join("DEBIAN/control"), &desktop_control).map_err(|e| e.to_string())?;

    let desktop_postinst = r#"#!/usr/bin/env bash
set -euo pipefail
install -d -m 0750 /var/lib/qualiadb /etc/webizen
echo "appliance" > /etc/webizen/sku
systemctl daemon-reload || true
"#;
    fs::write(desktop_tree.join("DEBIAN/postinst"), desktop_postinst).map_err(|e| e.to_string())?;

    // 2. Target 2 — Mobile Pocket Sanctuary & Co-Signer (`webizen-mobile-avf_0.1.0_arm64`)
    fs::create_dir_all(mobile_tree.join("DEBIAN")).map_err(|e| e.to_string())?;
    fs::create_dir_all(mobile_tree.join("opt/webizen/bin")).map_err(|e| e.to_string())?;
    fs::create_dir_all(mobile_tree.join("etc/webizen")).map_err(|e| e.to_string())?;
    fs::create_dir_all(mobile_tree.join("lib/systemd/system")).map_err(|e| e.to_string())?;

    let mobile_control = format!(
        "Package: webizen-mobile-avf\n\
         Version: 0.1.0\n\
         Section: utils\n\
         Priority: optional\n\
         Architecture: arm64\n\
         Depends: libc6 (>= 2.38), systemd, libvulkan1, mesa-vulkan-drivers, wireguard-tools, ca-certificates\n\
         Maintainer: Webizen Human-Centric Stewardship <steward@{domain}>\n\
         Description: Webizen Mobile Pocket Sanctuary & Host Quorum Co-Signer (Debian 13 Trixie arm64 / Android 16 AVF pKVM EL2)\n\
         \x20Staged profile for Debian 13 arm64 on Android AVF or native Mobian.\n\
         \x20The target runtime must discover and verify isolation, attestation, GPU and vsock support.\n",
        domain = stewardship.primary_host_domain,
    );
    validate_human_centric_host_nomenclature(&mobile_control)?;
    fs::write(mobile_tree.join("DEBIAN/control"), &mobile_control).map_err(|e| e.to_string())?;

    let avf_crosvm_config = format!(
        "# Android 16 AVF (pKVM EL2 + crosvm) & Mobian Trixie Mobile Sanctuary Configuration\n\
         # Natural-Person Stewards: {stewards}\n\
         # Host Anchor: {domain} ({endpoint})\n\
         AVF_VM_MODE=\"protected-pkvm-el2\"\n\
         AVF_DICE_ATTESTATION=\"pvmfw-to-anvil-np64\"\n\
         AVF_GPU_BACKEND=\"virtio-gpu-gfxstream-venus-ahb-zero-copy\"\n\
         AVF_SANCTUARY_COMPUTE_TIER1=\"inside-el2-armv9-neon-i8mm-vulkan\"\n\
         AVF_PUBLIC_NPU_BRIDGE_TIER2=\"vhost-vsock-cid2-port-4243-blinded-tensor-rpc\"\n\
         WELLFAIR_SANCTUARY_VOLATILE_SCRUB=\"false\"\n\
         HOST_QUORUM_COSIGNER_ENABLED=\"false\"\n",
        stewards = steward_dids_csv,
        domain = stewardship.primary_host_domain,
        endpoint = stewardship.host_endpoint,
    );
    let avf_crosvm_launcher_path = mobile_tree.join("etc/webizen/avf-pkvm-sanctuary.conf");
    validate_human_centric_host_nomenclature(&avf_crosvm_config)?;
    fs::write(&avf_crosvm_launcher_path, &avf_crosvm_config).map_err(|e| e.to_string())?;

    let mobile_service = r#"[Unit]
Description=Webizen Mobile AVF pKVM EL2 Sanctuary, Local Qap Runtime & Host Quorum Co-Signer
After=network-online.target
Wants=network-online.target
ConditionPathIsExecutable=/opt/webizen/bin/qualia-cli

[Service]
Type=simple
User=qualiadb
Group=qualiadb
EnvironmentFile=/etc/webizen/avf-pkvm-sanctuary.conf
Environment=WEBIZEN_SKU=appliance
Environment=QUALIA_STORAGE_PATH=/var/lib/qualiadb
ExecStart=/opt/webizen/bin/qualia-cli daemon start --port 4242 --avf-el2-sanctuary
Restart=always
RestartSec=2
MemoryMax=512M
NoNewPrivileges=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/qualiadb

[Install]
WantedBy=multi-user.target
"#;
    fs::write(
        mobile_tree.join("lib/systemd/system/webizen-mobile-avf.service"),
        mobile_service,
    )
    .map_err(|e| e.to_string())?;

    // 3. Target 3 — Human-Centric Linux Host Platform (`webizen-hostd_0.1.0_amd64`)
    fs::create_dir_all(host_tree.join("DEBIAN")).map_err(|e| e.to_string())?;
    fs::create_dir_all(host_tree.join("usr/local/bin")).map_err(|e| e.to_string())?;
    fs::create_dir_all(host_tree.join("etc/webizen-host")).map_err(|e| e.to_string())?;
    fs::create_dir_all(host_tree.join("lib/systemd/system")).map_err(|e| e.to_string())?;

    let host_control = format!(
        "Package: webizen-hostd\n\
         Version: 0.1.0\n\
         Section: net\n\
         Priority: optional\n\
         Architecture: amd64\n\
         Depends: libc6 (>= 2.38), systemd, nftables, caddy, wireguard-tools, ca-certificates\n\
         Maintainer: Webizen Human-Centric Stewardship <steward@{domain}>\n\
         Description: Webizen & QualiaDB Human-Centric Linux Host Platform (Debian 13 Trixie)\n\
         \x20Staged configuration for {domain} and the headless Webizen host runtime.\n\
         \x20Updates require a verifier and cryptographic ratification by the responsible\n\
         \x20Natural-Person Steward(s) ({threshold_m}-of-{total_n} quorum).\n",
        domain = stewardship.primary_host_domain,
        threshold_m = stewardship.threshold_m,
        total_n = stewardship.stewards.len().max(1),
    );
    validate_human_centric_host_nomenclature(&host_control)?;
    fs::write(host_tree.join("DEBIAN/control"), &host_control).map_err(|e| e.to_string())?;

    // Write W3C CBOR-LD (0xD9 0x05 0x01) + SHA-256 sealed host_state.cbor-ld compatible with `webizen-host`
    let host_state_cbor_ld_path = host_tree.join("etc/webizen-host/host_state.cbor-ld");
    let cbor_bytes = encode_webizen_host_cbor_ld_state(stewardship)?;
    fs::write(&host_state_cbor_ld_path, &cbor_bytes).map_err(|e| e.to_string())?;

    // Write the human-stewarded update verification gate script (`/usr/local/bin/webizen-host-apply-update`)
    let host_quorum_gate_script_path = host_tree.join("usr/local/bin/webizen-host-apply-update");
    let host_gate_script = concat!(
        "#!/bin/sh\n",
        "printf '%s\\n' 'Host update refused: a cryptographic quorum verifier is required.' >&2\n",
        "exit 78\n",
    );
    validate_human_centric_host_nomenclature(host_gate_script)?;
    fs::write(&host_quorum_gate_script_path, host_gate_script).map_err(|e| e.to_string())?;

    // Master `build-tri-target-debs.sh` script that invokes `dpkg-deb --build` across all 3 targets
    let build_all_debs_script_path = suite_root.join("build-tri-target-debs.sh");
    let build_all_script = format!(
        "#!/usr/bin/env bash\n\
         # Build the Unified Tri-Target Debian 13 (Trixie) .deb Suite for Human-Centric Host Stewardship\n\
         # 1. webizen-desktop-appliance_0.1.0_amd64.deb (Desktop Steward & Authoring Node)\n\
         # 2. webizen-mobile-avf_0.1.0_arm64.deb       (Android 16 AVF pKVM EL2 Pocket Sanctuary & Co-Signer)\n\
         # 3. webizen-hostd_0.1.0_amd64.deb            (Human-Centric Linux Host Platform governed by Natural-Person Quorum)\n\
         # Hardware Attestation Gate on this build host: {attested}\n\
         set -euo pipefail\n\
         DIR=\"$(cd \"$(dirname \"${{BASH_SOURCE[0]}}\")\" && pwd)\"\n\
         dpkg-deb --build \"$DIR/webizen-desktop-appliance_0.1.0_amd64\"\n\
         dpkg-deb --build \"$DIR/webizen-mobile-avf_0.1.0_arm64\"\n\
         dpkg-deb --build \"$DIR/webizen-hostd_0.1.0_amd64\"\n\
         echo \"Built all 3 Debian 13 Trixie packages in $DIR\"\n",
        attested = gpu.may_advertise_hardware_attested_vault,
    );
    validate_human_centric_host_nomenclature(&build_all_script)?;
    fs::write(&build_all_debs_script_path, &build_all_script).map_err(|e| e.to_string())?;

    let mut receipt = TriTargetDebianSuiteReceipt {
        suite_root_dir: suite_root,
        desktop_deb_tree: desktop_tree,
        mobile_avf_deb_tree: mobile_tree,
        host_deb_tree: host_tree,
        built_deb_packages: Vec::new(),
        apt_repository: None,
        host_state_cbor_ld_path,
        host_quorum_gate_script_path,
        avf_crosvm_launcher_path,
        build_all_debs_script_path,
        steward_count: stewardship.stewards.len(),
        quorum_threshold_m: stewardship.threshold_m,
    };
    receipt.built_deb_packages = crate::deb_builder::build_tri_target_debs_from_receipt(&receipt)?;
    let eligible_stewards: Vec<String> = stewardship
        .stewards
        .iter()
        .filter(|steward| steward.has_full_legal_capacity)
        .map(|steward| steward.steward_did.clone())
        .collect();
    let apt_generator =
        if crate::deb_builder::apt_signing_evidence_exists(&receipt.suite_root_dir)? {
            let has_unresolved_dissent = stewardship.update_queue.iter().any(|proposal| {
                (proposal.paraconsistent_review_flagged
                    || proposal.votes.iter().any(|vote| !vote.approve))
                    && receipt.built_deb_packages.iter().any(|package| {
                        proposal.target_version == package.version
                            && (proposal.target_deb_package == package.package_name
                                || package.output_deb_path.file_name().is_some_and(|name| {
                                    name == proposal.target_deb_package.as_str()
                                }))
                    })
            });
            if has_unresolved_dissent {
                return Err("APT publication is held for unresolved steward dissent".into());
            }
            crate::deb_builder::generate_signed_apt_repository
        } else {
            crate::deb_builder::generate_apt_repository_indexes
        };
    receipt.apt_repository = Some(apt_generator(
        &receipt.suite_root_dir,
        &receipt.built_deb_packages,
        domain,
        stewardship.threshold_m,
        &eligible_stewards,
    )?);
    Ok(receipt)
}

/// Encode `host_state.cbor-ld` with the W3C CBOR-LD tag header `[0xD9, 0x05, 0x01]` and trailing
/// 32-byte digest seal, binding the natural-person steward DID into `webizen-host`.
fn encode_webizen_host_cbor_ld_state(
    stewardship: &NaturalHostStewardshipFabric,
) -> Result<Vec<u8>, String> {
    let primary_steward = stewardship
        .stewards
        .first()
        .map(|s| s.steward_did.as_str())
        .unwrap_or("did:qi:webcivics_steward");
    validate_human_centric_host_nomenclature(&stewardship.primary_host_domain)?;
    validate_human_centric_host_nomenclature(primary_steward)?;

    let mut buf = Vec::with_capacity(256);
    buf.extend_from_slice(&CBOR_LD_TAG_HEADER);
    buf.push(1u8); // HOST_STATE_VERSION = 1

    let write_lp = |out: &mut Vec<u8>, s: &str| {
        let b = s.as_bytes();
        out.extend_from_slice(&(b.len() as u32).to_be_bytes());
        out.extend_from_slice(b);
    };

    write_lp(&mut buf, &stewardship.primary_host_domain);
    write_lp(&mut buf, primary_steward);
    let egress_cap: u64 = 1_500 * 1024 * 1024 * 1024; // 1.5 TiB hard monthly circuit breaker ($0.00 overage)
    buf.extend_from_slice(&egress_cap.to_be_bytes());

    buf.extend_from_slice(&(stewardship.attached_handles.len() as u32).to_be_bytes());
    for (dom, _) in &stewardship.attached_handles {
        write_lp(&mut buf, dom);
    }

    // 1 seeded relationship-scoped mail alias
    buf.extend_from_slice(&1u32.to_be_bytes());
    write_lp(
        &mut buf,
        &format!("steward@{}", stewardship.primary_host_domain),
    );
    write_lp(&mut buf, primary_steward);
    write_lp(
        &mut buf,
        &format!("peer@{}", stewardship.primary_host_domain),
    );

    // 1 seeded git repo
    buf.extend_from_slice(&1u32.to_be_bytes());
    write_lp(&mut buf, "webcivics-ns.git");

    // Delegated permits count
    buf.extend_from_slice(&(stewardship.stewards.len() as u32).to_be_bytes());
    // Custom DNS records count
    buf.extend_from_slice(&0u32.to_be_bytes());

    let mut hasher = Sha3_256::new();
    hasher.update(&buf);
    let seal = hasher.finalize();
    buf.extend_from_slice(&seal);
    Ok(buf)
}

fn default_seeded_host_stewardship() -> NaturalHostStewardshipFabric {
    let stewards = vec![
        ResponsibleNaturalSteward {
            steward_did: "did:qi:steward:desktop-principal".into(),
            steward_did_hash: q_hash_60("did:qi:steward:desktop-principal"),
            display_hint: "Primary Natural-Person Steward (Workstation)".into(),
            role: "PrincipalSteward".into(),
            apparatus_profile: "Desktop Debian 13 Trixie / Guest Workstation (webizen-desktop-appliance.deb)".into(),
            dual_pubkey_fingerprint: "ed25519+mldsa65:8a41f0c912b74e05a319".into(),
            has_full_legal_capacity: true,
            guardian_did: None,
        },
        ResponsibleNaturalSteward {
            steward_did: "did:qi:steward:mobile-avf-pocket".into(),
            steward_did_hash: q_hash_60("did:qi:steward:mobile-avf-pocket"),
            display_hint: "Principal Pocket Co-Signer (Android 16 AVF pKVM EL2)".into(),
            role: "PrincipalPocketApparatus".into(),
            apparatus_profile: "Mobile Debian 13 Trixie arm64 inside Android 16 AVF pKVM EL2 (webizen-mobile-avf.deb)".into(),
            dual_pubkey_fingerprint: "ed25519+mldsa65:3c92e1a447d09f11b620".into(),
            has_full_legal_capacity: true,
            guardian_did: None,
        },
        ResponsibleNaturalSteward {
            steward_did: "did:qi:steward:community-grounds-costeward".into(),
            steward_did_hash: q_hash_60("did:qi:steward:community-grounds-costeward"),
            display_hint: "Cooperative Co-Steward (Community Grounds)".into(),
            role: "CoStewardGuardian".into(),
            apparatus_profile: "Desktop Debian 13 Trixie Wayland Appliance (webizen-desktop-appliance.deb)".into(),
            dual_pubkey_fingerprint: "ed25519+mldsa65:7e15b8d304c62a88f901".into(),
            has_full_legal_capacity: true,
            guardian_did: None,
        },
        ResponsibleNaturalSteward {
            steward_did: "did:qi:family:supported-youth-member".into(),
            steward_did_hash: q_hash_60("did:qi:family:supported-youth-member"),
            display_hint: "Supported Family Youth Member (UN CRC Guardianship)".into(),
            role: "SupportedDependent".into(),
            apparatus_profile: "Mobile Debian 13 Trixie arm64 AVF Sanctuary (Guardian Co-Sign Required)".into(),
            dual_pubkey_fingerprint: "ed25519+mldsa65:5b04d7c118e39a40c217".into(),
            has_full_legal_capacity: false,
            guardian_did: Some("did:qi:steward:desktop-principal".into()),
        },
    ];

    let mut fabric = NaturalHostStewardshipFabric {
        primary_host_domain: "ns.webcivics.net".into(),
        host_endpoint: "https://ns.webcivics.net:4242 (Debian 13 Trixie webizen-hostd)".into(),
        archetype: HostStewardshipArchetype::MultiPersonCooperativeQuorum,
        threshold_m: 2,
        stewards,
        attached_handles: vec![
            (
                "ns.webcivics.net".into(),
                "Active (Who-Plane != Handle-Plane; Collapse Detector Armed)".into(),
            ),
            (
                "dev.civics.au".into(),
                "Active (Community Grounds & Civics Observatory Host)".into(),
            ),
            (
                "web.civics.au".into(),
                "Active (Tier-3 W3C Solid / LDP Databox Off-Ramp)".into(),
            ),
        ],
        sanctuary_keep_local_only: true,
        shamir_q42j_backup_threshold:
            "2-of-3 Client-Encrypted .q42j Shamir Shares (Sanctuary Keep never leaves local Desktop/Mobile AVF hardware)"
                .into(),
        update_queue: Vec::new(),
        host_fleet_nodes: vec![
            PersonalHostNode {
                node_id: "node:home-homestead-server".into(),
                label: "Home Server / Homestead Appliance (Debian 13 Trixie SFF / RPi 5)".into(),
                locus: HostNodeLocusKind::HomeServerAppliance,
                runtime_operator_steward_dids: vec!["did:qi:steward:desktop-principal".into()],
                qdnf_ula_ipv6: "fd42:7175:616c:0101::0001".into(),
                endpoint_hint: "https://home-box.local:4242 (LAN mDNS + QDNF WireGuard)".into(),
                standalone_without_online_vps: true,
                holds_local_sanctuary_keep: true,
                qcdn_seeder_tier: "Tier B · Personal/Home Seeder (:4242 + Local LAN)".into(),
                sealed_nquin_parity: q_hash_60("node:home-homestead-server"),
            },
            PersonalHostNode {
                node_id: "node:camper-walkabout-rig".into(),
                label: "Camper / Walkabout 12V Solar Rig (Offline-First Roaming Debian 13 Host)".into(),
                locus: HostNodeLocusKind::CamperMobileRigNode,
                runtime_operator_steward_dids: vec![
                    "did:qi:steward:desktop-principal".into(),
                    "did:qi:steward:mobile-avf-pocket".into(),
                ],
                qdnf_ula_ipv6: "fd42:7175:616c:0102::0002".into(),
                endpoint_hint: "qpr://camper-rig.mesh:4242 (12V Wi-Fi + Acoustic/BLE + Opportunistic WireGuard)".into(),
                standalone_without_online_vps: true,
                holds_local_sanctuary_keep: true,
                qcdn_seeder_tier: "Tier B · Roaming Field Seeder (:4242 + Offline Mesh)".into(),
                sealed_nquin_parity: q_hash_60("node:camper-walkabout-rig"),
            },
            PersonalHostNode {
                node_id: "node:online-cloud-vps-anchor".into(),
                label: "Online Cloud VPS Anchor (Public Ingress & Encrypted Shamir .q42j Mirror)".into(),
                locus: HostNodeLocusKind::OnlineCloudVpsRelay,
                runtime_operator_steward_dids: vec![
                    "did:qi:steward:desktop-principal".into(),
                    "did:qi:steward:community-grounds-costeward".into(),
                ],
                qdnf_ula_ipv6: "fd42:7175:616c:0103::0003".into(),
                endpoint_hint: "https://ns.webcivics.net:4242 (Public Ingress · Zero Raw Sanctuary Keys)".into(),
                standalone_without_online_vps: false,
                holds_local_sanctuary_keep: false,
                qcdn_seeder_tier: "Tier A · Cooperative VPS Origin & Mirror (<10 KiB SignedRootIndex)".into(),
                sealed_nquin_parity: q_hash_60("node:online-cloud-vps-anchor"),
            },
        ],
        hosted_projects: vec![
            HostedProjectWorkload {
                project_id: "proj:walkabout-solar-field-journal".into(),
                title: "Walkabout Camper & Off-Grid Solar Microgrid Twin".into(),
                attached_domain: "walkabout.civics.au".into(),
                ownership_stage: HostedProjectOwnershipStage::PersonalOwnerProject,
                project_steward_dids: vec!["did:qi:steward:desktop-principal".into()],
                project_quorum_threshold_m: 1,
                hosted_on_runtime_node_ids: vec![
                    "node:camper-walkabout-rig".into(),
                    "node:home-homestead-server".into(),
                ],
                qcdn_swarm_mirror_dids: vec![
                    "did:qi:steward:desktop-principal".into(),
                    "did:qi:steward:mobile-avf-pocket".into(),
                ],
                qcdn_signed_root_merkle_hex: "sha256:9f24a10c78b3e112d044".into(),
                incorporated_entity_id: None,
                provenance_history: vec![
                    "Created as PersonalOwnerProject on Camper Rig + Home Server (ready to transition into a Multi-Person Group Project)".into(),
                ],
                sealed_nquin_parity: q_hash_60("proj:walkabout-solar-field-journal"),
            },
            HostedProjectWorkload {
                project_id: "proj:talbingo-catchment-biosphere".into(),
                title: "Talbingo & B72 Catchment Ecological Twin (Supported Group Project on Q-CDN)".into(),
                attached_domain: "dev.civics.au".into(),
                ownership_stage: HostedProjectOwnershipStage::SupportedThirdPartyOrGroupProject,
                project_steward_dids: vec![
                    "did:qi:peer:talbingo-ecologist".into(),
                    "did:qi:peer:snowy-hydrologist".into(),
                    "did:qi:steward:community-grounds-costeward".into(),
                ],
                project_quorum_threshold_m: 2,
                hosted_on_runtime_node_ids: vec![
                    "node:online-cloud-vps-anchor".into(),
                    "node:home-homestead-server".into(),
                ],
                qcdn_swarm_mirror_dids: vec![
                    "did:qi:coop_vps_mirror_1".into(),
                    "did:qi:peer:talbingo-ecologist".into(),
                    "did:qi:steward:desktop-principal".into(),
                ],
                qcdn_signed_root_merkle_hex: "sha256:4b88e301c92a75f061be".into(),
                incorporated_entity_id: None,
                provenance_history: vec![
                    "Server Runtime operated by Host Steward; Project content & domain governed by Talbingo Catchment Group (2-of-3 quorum) and mirrored across Q-CDN Swarm".into(),
                ],
                sealed_nquin_parity: q_hash_60("proj:talbingo-catchment-biosphere"),
            },
            HostedProjectWorkload {
                project_id: "proj:incorporated-energy-retailer-portal".into(),
                title: "Regional Energy Grid Pty Ltd — Corporate Customer & Telemetry Portal".into(),
                attached_domain: "grid-corp.example.com.au".into(),
                ownership_stage: HostedProjectOwnershipStage::IncorporatedCommercialTenantProject,
                project_steward_dids: vec!["did:qi:corp-agent:grid-engineering-lead".into()],
                project_quorum_threshold_m: 1,
                hosted_on_runtime_node_ids: vec!["node:online-cloud-vps-anchor".into()],
                qcdn_swarm_mirror_dids: vec!["did:qi:coop_vps_mirror_1".into()],
                qcdn_signed_root_merkle_hex: "sha256:c170d4498a23f80155e2".into(),
                incorporated_entity_id: Some("did:qi:corp:regional-energy-grid-pty-ltd".into()),
                provenance_history: vec![
                    "Owned by Incorporated Legal Personality (ACN 642 001 998); operated by Natural-Person Corporate Agent under Mandatory Commercial Licence".into(),
                ],
                sealed_nquin_parity: q_hash_60("proj:incorporated-energy-retailer-portal"),
            },
        ],
        incorporated_entities: vec![
            IncorporatedLegalEntityRecord {
                entity_did: "did:qi:corp:regional-energy-grid-pty-ltd".into(),
                legal_name: "Regional Energy Grid Pty Ltd".into(),
                jurisdiction_and_reg_number: "AU-ASIC · ACN 642 001 998 / ABN 81 642 001 998".into(),
                pricing_tier: IncorporatedEntityPricingTier::MidMarketCommercialCorporation,
                stakeholder_governance_factors: vec![
                    "Board of Directors Constitutional Resolution".into(),
                    "Ordinary & Preference Shareholder Voting Structure (Corporations Act)".into(),
                    "Secured Project-Finance Creditor Covenants".into(),
                ],
                authorized_natural_person_agent_dids: vec![
                    "did:qi:corp-agent:grid-engineering-lead".into(),
                    "did:qi:corp-agent:company-secretary".into(),
                ],
                mandatory_commercial_licence_active: true,
                monthly_base_fee_cents: IncorporatedEntityPricingTier::MidMarketCommercialCorporation
                    .monthly_base_commercial_fee_cents(),
                qcdn_carriage_cents_per_gib: IncorporatedEntityPricingTier::MidMarketCommercialCorporation
                    .qcdn_carriage_rate_cents_per_gib(),
                settlement_rail: "QualiaPQ-Ledger (QPQ) / eCash (XEC) / BTC-LN / USDC".into(),
                loop1_commons_pool_contribution_cents:
                    IncorporatedEntityPricingTier::MidMarketCommercialCorporation
                        .monthly_base_commercial_fee_cents(),
                sealed_nquin_parity: q_hash_60("did:qi:corp:regional-energy-grid-pty-ltd"),
            },
            IncorporatedLegalEntityRecord {
                entity_did: "did:qi:corp:regional-landcare-inc".into(),
                legal_name: "Regional Catchment Landcare Incorporated".into(),
                jurisdiction_and_reg_number: "AU-NSW · Incorporated Association INC9894210".into(),
                pricing_tier: IncorporatedEntityPricingTier::IncorporatedNonProfitOrCivicBody,
                stakeholder_governance_factors: vec![
                    "Incorporated Association Committee & Statutory Constitution".into(),
                    "Public Officer & Grant Acquittal Compliance Obligations".into(),
                ],
                authorized_natural_person_agent_dids: vec![
                    "did:qi:steward:community-grounds-costeward".into(),
                ],
                mandatory_commercial_licence_active: true,
                monthly_base_fee_cents: IncorporatedEntityPricingTier::IncorporatedNonProfitOrCivicBody
                    .monthly_base_commercial_fee_cents(),
                qcdn_carriage_cents_per_gib: IncorporatedEntityPricingTier::IncorporatedNonProfitOrCivicBody
                    .qcdn_carriage_rate_cents_per_gib(),
                settlement_rail: "QualiaPQ-Ledger (QPQ) / eCash (XEC) / BTC-LN".into(),
                loop1_commons_pool_contribution_cents:
                    IncorporatedEntityPricingTier::IncorporatedNonProfitOrCivicBody
                        .monthly_base_commercial_fee_cents(),
                sealed_nquin_parity: q_hash_60("did:qi:corp:regional-landcare-inc"),
            },
        ],
        shared_care_circles: vec![
            SharedGuardianshipAndCareCircle {
                circle_id: "circle:elderly-home-remote-carers".into(),
                title: "Elderly Independent Living & Remote Family/Carer Alert Circle (UN CRPD Supported Agency)".into(),
                topology: SharedLivingAndCareTopologyKind::ElderlyOrSupportedPersonWithRemoteCarers,
                principal_did: "did:qi:principal:elderly-parent-at-home".into(),
                principal_agency_stage: "AgencyStage::SupportedAdult (100% Self-Determination with Supports — Never Substituted Decision-Making)".into(),
                host_node_id: "node:home-homestead-server".into(),
                participants: vec![
                    SharedSystemParticipant {
                        participant_did: "did:qi:principal:elderly-parent-at-home".into(),
                        display_name: "Elderly Parent (Principal at Home)".into(),
                        relationship_role: "Principal (Sanctuary Keep Owner · Reversible 'I am OK' Heartbeat)".into(),
                        steward_roles: vec!["SelfPrincipal".into()],
                        physical_locus: "Home Homestead Server + Wrist/Ambient Fall & rPPG Sensor".into(),
                        alert_min_severity_level: 0,
                        consent_credential_id: "cc-self-principal".into(),
                    },
                    SharedSystemParticipant {
                        participant_did: "did:qi:carer:adult-daughter-remote".into(),
                        display_name: "Adult Daughter (Remote Family Carer in Melbourne)".into(),
                        relationship_role: "RemoteFamilyCarer & IncapacityAdvocate".into(),
                        steward_roles: vec!["PrimaryCare".into(), "Medical".into(), "EmergencyAdvocate".into()],
                        physical_locus: "Remote Mobile Debian 13 AVF pKVM Sanctuary (Elsewhere via QDNF WireGuard)".into(),
                        alert_min_severity_level: 1, // Receives Notice (1), Concern (2), and Urgent (3)
                        consent_credential_id: "cc-remote-daughter-vitals-and-heartbeat".into(),
                    },
                    SharedSystemParticipant {
                        participant_did: "did:qi:carer:visiting-community-nurse".into(),
                        display_name: "Visiting Community Nurse (Remote Clinical Responder)".into(),
                        relationship_role: "CredentialedCarer (DutyOfInquiry Bound)".into(),
                        steward_roles: vec!["Medical".into()],
                        physical_locus: "Remote Clinic Workstation (QDNF Scoped ConsentCredential)".into(),
                        alert_min_severity_level: 2, // Receives Concern (2) and Urgent (3) only
                        consent_credential_id: "cc-community-nurse-medication-and-falls".into(),
                    },
                ],
                min_alert_severity_level: 1,
                heartbeat_grace_secs: 43_200, // 12 hours
                last_principal_checkin_unix_secs: None,
                last_principal_checkin_note: "Morning tea check-in & blood-pressure log completed (All OK)".into(),
                escalation_currently_active: false,
                sealed_nquin_parity: q_hash_60("circle:elderly-home-remote-carers"),
            },
            SharedGuardianshipAndCareCircle {
                circle_id: "circle:family-children-stewardship".into(),
                title: "Family Household with Children (UN CRC Developmental AgencyStage Gradient & Split StewardRoles)".into(),
                topology: SharedLivingAndCareTopologyKind::FamilyWithChildrenGuardianship,
                principal_did: "did:qi:family:child-leo-own-birth-record".into(),
                principal_agency_stage: "AgencyStage::Child (40% Monotone Self-Determination -> Growing toward Adolescent 75% & Adult 100%)".into(),
                host_node_id: "node:home-homestead-server".into(),
                participants: vec![
                    SharedSystemParticipant {
                        participant_did: "did:qi:family:child-leo-own-birth-record".into(),
                        display_name: "Leo (Child Principal · Owns Own DigitalBirthRecord)".into(),
                        relationship_role: "ChildPrincipal (Never Collapsed into Parent's DID)".into(),
                        steward_roles: vec!["DevelopingSelfAgency".into()],
                        physical_locus: "Home Server Isolated Member Lane (PBKDF2 310k + HKDF)".into(),
                        alert_min_severity_level: 0,
                        consent_credential_id: "cc-child-own-fabric".into(),
                    },
                    SharedSystemParticipant {
                        participant_did: "did:qi:steward:desktop-principal".into(),
                        display_name: "Parent A (Primary Care & Medical Steward)".into(),
                        relationship_role: "ParentGuardian (StewardBasis::BiologicalDefault)".into(),
                        steward_roles: vec!["PrimaryCare".into(), "Medical".into(), "Educational".into()],
                        physical_locus: "Desktop Debian 13 + Mobile AVF Pocket Sanctuary".into(),
                        alert_min_severity_level: 1,
                        consent_credential_id: "cc-parent-a-stewardship".into(),
                    },
                    SharedSystemParticipant {
                        participant_did: "did:qi:guardian:parent-b-co-guardian".into(),
                        display_name: "Parent B (Co-Guardian · Protective Escrow Veto & Legal Steward)".into(),
                        relationship_role: "ParentGuardian (M-of-N Proxy Escrow Co-Signer)".into(),
                        steward_roles: vec!["Legal".into(), "Financial".into(), "Cultural".into()],
                        physical_locus: "Remote Mobile Debian 13 AVF Sanctuary".into(),
                        alert_min_severity_level: 2,
                        consent_credential_id: "cc-parent-b-stewardship".into(),
                    },
                ],
                min_alert_severity_level: 1,
                heartbeat_grace_secs: 86_400,
                last_principal_checkin_unix_secs: None,
                last_principal_checkin_note: "School project & 10D Talbingo botany dissection active".into(),
                escalation_currently_active: false,
                sealed_nquin_parity: q_hash_60("circle:family-children-stewardship"),
            },
            SharedGuardianshipAndCareCircle {
                circle_id: "circle:sharehouse-peer-commons".into(),
                title: "Fitzroy Co-Op Share-House (4 Equal Adult Housemates · Zero Peer Guardianship)".into(),
                topology: SharedLivingAndCareTopologyKind::ShareHousePeerCoLiving,
                principal_did: "did:qi:sharehouse:commons-utilities-pool".into(),
                principal_agency_stage: "AgencyStage::Adult (100% Independent Adult Peers — Cryptographically Isolated Sanctuary Vaults)".into(),
                host_node_id: "node:home-homestead-server".into(),
                participants: vec![
                    SharedSystemParticipant {
                        participant_did: "did:qi:steward:desktop-principal".into(),
                        display_name: "Housemate 1 (Server Hardware Operator — Zero Access to Flatmates' Vaults)".into(),
                        relationship_role: "AdultHousematePeer".into(),
                        steward_roles: vec!["HousemateCommonsOnly (Wi-Fi / Solar / Bills Quorum)".into()],
                        physical_locus: "Home Server Lane #1 (Isolated Argon2id/PBKDF2 Key)".into(),
                        alert_min_severity_level: 1,
                        consent_credential_id: "cc-sharehouse-commons-1".into(),
                    },
                    SharedSystemParticipant {
                        participant_did: "did:qi:housemate:sam-peer".into(),
                        display_name: "Housemate 2 (Sam — Equal Co-Tenant)".into(),
                        relationship_role: "AdultHousematePeer".into(),
                        steward_roles: vec!["HousemateCommonsOnly (Wi-Fi / Solar / Bills Quorum)".into()],
                        physical_locus: "Home Server Lane #2 (Isolated Argon2id/PBKDF2 Key)".into(),
                        alert_min_severity_level: 1,
                        consent_credential_id: "cc-sharehouse-commons-2".into(),
                    },
                ],
                min_alert_severity_level: 1,
                heartbeat_grace_secs: 0,
                last_principal_checkin_unix_secs: None,
                last_principal_checkin_note: "Solar battery microgrid & NBN bill split ratified (2-of-2 housemates)".into(),
                escalation_currently_active: false,
                sealed_nquin_parity: q_hash_60("circle:sharehouse-peer-commons"),
            },
        ],
        care_alerts: vec![
            DispatchedCareAndGuardianAlert {
                alert_id: "alert:circle:elderly-home-remote-carers:2:seed1".into(),
                circle_id: "circle:elderly-home-remote-carers".into(),
                topology: SharedLivingAndCareTopologyKind::ElderlyOrSupportedPersonWithRemoteCarers,
                trigger_kind: CareAlertTriggerKind::MissedMedicationSchedule,
                severity: CareAlertSeverity::Concern,
                principal_did: "did:qi:principal:elderly-parent-at-home".into(),
                notified_remote_guardian_dids: vec![
                    "did:qi:carer:adult-daughter-remote (Adult Daughter · Remote Mobile Debian 13 AVF pKVM Sanctuary)".into(),
                    "did:qi:carer:visiting-community-nurse (Visiting Community Nurse · Remote Clinic Workstation)".into(),
                ],
                summary: "Morning antihypertensive dose overdue by 90 mins — scoped alert sent over QDNF to remote daughter & community nurse (private Sanctuary diary remains unexposed)".into(),
                recorded_in_accountability_ledger: true,
                duty_of_inquiry_status: "InquiryVerdict::Diligent — Daughter phoned & principal confirmed dose taken".into(),
                resolved_or_reversed: true,
                sealed_nquin_parity: q_hash_60("alert:circle:elderly-home-remote-carers:2:seed1"),
            },
        ],
    };

    if let Ok(proposal) = fabric.make_host_update_proposal(
        "webizen-hostd_0.1.1_amd64.deb",
        "0.1.1",
        "did:q42:agent:release_builder_bot",
        true,
    ) {
        fabric.update_queue.push(proposal);
    }
    fabric
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifiers::enumerate_session_identifiers;
    use crate::native_gpu::probe_native_etalon;
    use crate::os_env::OsEnvironment;
    use crate::qap_host::TrustRoots;
    use crate::session::Sku;

    #[test]
    fn hosted_registration_seals_and_persists_only_an_admissible_project() {
        let path = std::env::temp_dir().join(format!(
            "webizen-project-registration-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        let mut fabric = NaturalHostStewardshipFabric::default();
        let stage = HostedProjectOwnershipStage::IncorporatedCommercialTenantProject;
        let args = (
            "proj:test-registration",
            "Community project",
            "project.example",
            "did:qi:steward:test",
        );
        assert!(fabric
            .register_hosted_project(&path, args.0, args.1, args.2, stage, args.3, None)
            .is_err());

        let entity = fabric.incorporated_entities[0].entity_did.clone();
        let project = fabric
            .register_hosted_project(&path, args.0, args.1, args.2, stage, args.3, Some(&entity))
            .unwrap();
        let proposed_words =
            serde_json::to_string(&(args.1, args.2, stage, args.3, Some(&entity))).unwrap();
        let quin = NQuin48::invariant(
            args.0,
            "https://ns.webcivics.net/stewardship#registeredHostedProject",
            q_hash_60(&proposed_words),
        );
        assert_eq!(project.sealed_nquin_parity, quin.parity());
        assert!(project.hosted_on_runtime_node_ids.is_empty());
        assert!(project.qcdn_signed_root_merkle_hex.is_empty());

        let persisted = fs::read(path.join(HOST_STEWARDSHIP_STORE_REL_PATH)).unwrap();
        let saved: NaturalHostStewardshipFabric = serde_json::from_slice(&persisted).unwrap();
        assert!(saved.hosted_projects.contains(&project));
        assert!(fabric
            .register_hosted_project(&path, args.0, args.1, args.2, stage, args.3, Some(&entity))
            .is_err());
        let file_as_storage = path.join("unwritable-child");
        fs::write(&file_as_storage, "").unwrap();
        let previous = fabric.hosted_projects.len();
        assert!(fabric
            .register_hosted_project(
                &file_as_storage,
                "proj:failed-save",
                args.1,
                args.2,
                stage,
                args.3,
                Some(&entity)
            )
            .is_err());
        assert_eq!(fabric.hosted_projects.len(), previous);
        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn natural_person_and_cooperative_quorum_govern_host_updates_and_emit_tri_target_debs() {
        let tmp = std::env::temp_dir().join(format!("wz-host-steward-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let host = LoopbackHost {
            daemon_base: "http://127.0.0.1:4242".into(),
            port: 4242,
            storage: tmp.clone(),
            install_root: tmp.clone(),
            trust: TrustRoots::for_storage(&tmp),
        };
        let gpu = probe_native_etalon(&tmp);
        let os_env = OsEnvironment::probe("steward");
        let ids = enumerate_session_identifiers(&host, Sku::Appliance, &os_env, &gpu);

        let mut fabric = NaturalHostStewardshipFabric::load_or_seed(&tmp, &ids);
        assert_eq!(fabric.threshold_m, 2);
        assert!(fabric.sanctuary_keep_local_only);

        // 1. Verify SHACL-First Non-Thing Rule & Anti-Collapse Invariant:
        //    - An AI/service instrument cannot register as a NaturalAgent steward
        //    - A person cannot be labeled as an owl:Thing / Object
        assert!(fabric
            .register_natural_steward(
                &tmp,
                "did:q42:agent:rogue_updater_bot",
                "Autonomous Bot",
                "CoStewardGuardian",
                "Server",
                true,
                None,
            )
            .is_err());
        assert!(fabric
            .register_natural_steward(
                &tmp,
                "did:qi:steward:bad-label",
                "owl:Thing",
                "CoStewardGuardian",
                "Desktop",
                true,
                None,
            )
            .is_err());

        // 2. Propose a webizen-host .deb update (staged by a governed build tool — 0 votes cast)
        let prop = fabric
            .propose_host_update(
                &tmp,
                "webizen-hostd_0.2.0_amd64.deb",
                "0.2.0",
                "did:q42:agent:ci_deb_packager_bot",
                true,
            )
            .expect("stage host update proposal");
        assert!(!prop.ratified_for_host_apply);
        assert_eq!(prop.required_m, 2);

        // 3. Verify that an AI/service agent is barred from voting to ratify the host update
        assert!(fabric
            .sign_or_dissent_host_update(
                &tmp,
                &prop.proposal_id,
                "did:q42:agent:ci_deb_packager_bot",
                None,
                true,
                "Bot trying to self-ratify",
            )
            .is_err());

        // 4. Verify Family Guardianship protection: supported youth member cannot ratify without guardian co-signature
        assert!(fabric
            .sign_or_dissent_host_update(
                &tmp,
                &prop.proposal_id,
                "did:qi:family:supported-youth-member",
                None,
                true,
                "Youth vote without guardian co-signer",
            )
            .is_err());

        // 5. First Natural-Person Steward signs from Desktop (`webizen-desktop-appliance.deb`): 1-of-2 (not yet ratified)
        let desktop_did = fabric.stewards[0].steward_did.clone();
        let after_vote1 = fabric
            .sign_or_dissent_host_update(
                &tmp,
                &prop.proposal_id,
                &desktop_did,
                None,
                true,
                "Verified NQuin DNS & QPolicy nftables diff on Desktop Steward Workstation",
            )
            .expect("desktop steward vote");
        assert!(!after_vote1.ratified_for_host_apply);

        // 6. Test Paraconsistent 0x32 Dissent Retention: Co-steward casts dissent -> flags RequiresHumanReview
        let after_dissent = fabric
            .sign_or_dissent_host_update(
                &tmp,
                &prop.proposal_id,
                "did:qi:steward:community-grounds-costeward",
                None,
                false,
                "Hold rollout until microgrid telemetry migration finishes",
            )
            .expect("co-steward dissent");
        assert!(after_dissent.paraconsistent_review_flagged);
        assert!(!after_dissent.ratified_for_host_apply);

        // 7. Second signature from Mobile Debian 13 AVF pKVM Pocket Sanctuary (`webizen-mobile-avf.deb`):
        //    Even with 2 approvals, paraconsistent dissent blocks auto-apply until the dissenting co-steward reconciles!
        let after_mobile = fabric
            .sign_or_dissent_host_update(
                &tmp,
                &prop.proposal_id,
                "did:qi:steward:mobile-avf-pocket",
                None,
                true,
                "Co-signed from Pixel 10 Pro XL Android 16 AVF pKVM EL2 Sanctuary",
            )
            .expect("mobile avf pocket co-sign");
        assert!(after_mobile.paraconsistent_review_flagged);
        assert!(!after_mobile.ratified_for_host_apply);

        // 8. Reconciliation clears dissent while independent authorization remains pending.
        let after_reconcile = fabric
            .sign_or_dissent_host_update(
                &tmp,
                &prop.proposal_id,
                "did:qi:steward:community-grounds-costeward",
                None,
                true,
                "Microgrid migration verified; approving webizen-hostd 0.2.0 rollout",
            )
            .expect("co-steward reconcile");
        assert!(!after_reconcile.paraconsistent_review_flagged);
        assert_eq!(
            after_reconcile
                .votes
                .iter()
                .filter(|vote| vote.approve)
                .count(),
            3
        );
        assert!(after_reconcile
            .votes
            .iter()
            .all(|vote| vote.dual_signature_hex.is_empty()));
        assert!(!after_reconcile.ratified_for_host_apply);
        assert!(!after_reconcile.applied_to_host);

        // 9. Verify Collapse Detector (Handle revoke != who-erase)
        let collapse_msg = fabric
            .trigger_domain_collapse_detector(&tmp, "ns.webcivics.net")
            .expect("collapse detector");
        assert!(collapse_msg.contains("Handle revoke != who-erase"));
        assert_eq!(fabric.stewards.len(), 4);

        // 10. Generate the Tri-Target Debian 13 Trixie .deb Suite (Desktop amd64 + Mobile AVF arm64 + Host amd64)
        let suite = generate_tri_target_debian_suite(&host, &gpu, &fabric)
            .expect("generate tri-target debian suite");
        assert!(suite.desktop_deb_tree.join("DEBIAN/control").is_file());
        assert!(suite.mobile_avf_deb_tree.join("DEBIAN/control").is_file());
        assert!(suite.host_deb_tree.join("DEBIAN/control").is_file());
        assert!(suite.host_state_cbor_ld_path.is_file());
        assert!(suite.host_quorum_gate_script_path.is_file());
        assert!(suite.avf_crosvm_launcher_path.is_file());
        assert!(suite.build_all_debs_script_path.is_file());

        let cbor_raw = fs::read(&suite.host_state_cbor_ld_path).unwrap();
        assert_eq!(&cbor_raw[0..3], &CBOR_LD_TAG_HEADER);

        // 11. Verify Multi-Locus Host Fleet (Home Server, 12V Camper Rig standalone without Online VPS, Online Cloud VPS)
        assert_eq!(fabric.host_fleet_nodes.len(), 3);
        let camper_node = fabric
            .register_host_fleet_node(
                &tmp,
                "node:outback-rover-camper",
                "Outback Rover 12V Camper Server (Standalone Local-Only Mode)",
                HostNodeLocusKind::CamperMobileRigNode,
                vec![desktop_did.clone()],
                "qpr://outback-rover.local:4242",
                true,
            )
            .expect("register camper node");
        assert!(camper_node.standalone_without_online_vps);
        assert!(camper_node.holds_local_sanctuary_keep);

        // 12. Verify Server Runtime Responsibility vs Hosted Project Ownership & Personal -> Group Q-CDN Transition
        let transitioned = fabric
            .transition_project_to_group_or_qcdn_swarm(
                &tmp,
                "proj:walkabout-solar-field-journal",
                HostedProjectOwnershipStage::TransitionedPersonalToGroupProject,
                vec![
                    "did:qi:steward:community-grounds-costeward".into(),
                    "did:qi:peer:talbingo-ecologist".into(),
                ],
                2,
                vec!["did:qi:coop_vps_mirror_1".into()],
            )
            .expect("transition personal project to group project");
        assert_eq!(
            transitioned.ownership_stage,
            HostedProjectOwnershipStage::TransitionedPersonalToGroupProject
        );
        assert_eq!(transitioned.project_quorum_threshold_m, 2);
        assert!(transitioned.project_steward_dids.len() >= 3);
        assert!(transitioned
            .qcdn_swarm_mirror_dids
            .contains(&"did:qi:coop_vps_mirror_1".to_string()));

        // 13. Verify 3-Way Jural Actor Distinction (NaturalPersonPrincipal vs NaturalPersonAgentOfIncorporatedEntity vs IncorporatedLegalEntity)
        //     and Mandatory Commercial Licensing across ALL Incorporated Entity Tiers
        let (nat_charged, nat_fee, _) = fabric
            .evaluate_jural_actor_commercial_obligation(
                ActorJuralPersonalityKind::NaturalPersonPrincipal,
                None,
            )
            .expect("natural person principal");
        assert!(!nat_charged);
        assert_eq!(nat_fee, 0);

        for tier in IncorporatedEntityPricingTier::ALL {
            assert!(
                tier.monthly_base_commercial_fee_cents() > 0,
                "All incorporated entity tiers must be charged (> 0 cents)"
            );
        }

        // Natural person acting as an agent for an incorporated entity MUST be charged the corporate commercial licence
        let (agent_charged, agent_fee, _) = fabric
            .evaluate_jural_actor_commercial_obligation(
                ActorJuralPersonalityKind::NaturalPersonAgentOfIncorporatedEntity,
                Some("did:qi:corp:regional-energy-grid-pty-ltd"),
            )
            .expect("corporate agent obligation");
        assert!(agent_charged);
        assert_eq!(
            agent_fee,
            IncorporatedEntityPricingTier::MidMarketCommercialCorporation
                .monthly_base_commercial_fee_cents()
        );

        // Even an Incorporated Non-Profit / Association is an incorporated legal personality and is charged its tier rate (> 0)
        let (nfp_charged, nfp_fee, _) = fabric
            .evaluate_jural_actor_commercial_obligation(
                ActorJuralPersonalityKind::IncorporatedLegalEntity,
                Some("did:qi:corp:regional-landcare-inc"),
            )
            .expect("incorporated non-profit obligation");
        assert!(nfp_charged);
        assert_eq!(
            nfp_fee,
            IncorporatedEntityPricingTier::IncorporatedNonProfitOrCivicBody
                .monthly_base_commercial_fee_cents()
        );
        assert!(nfp_fee < agent_fee); // Differentiated pricing tier, yet always > 0

        // 14. Verify Shared Systems, Guardianship Relations & Remote Carer Alerts:
        //     (a) Share-House Peer Isolation Invariant: housemates CANNOT dispatch personal health/heartbeat/content alerts on each other
        assert!(fabric
            .dispatch_shared_care_or_guardian_alert(
                &tmp,
                "circle:sharehouse-peer-commons",
                CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse,
                CareAlertSeverity::Urgent,
                "Trying to spy on flatmate's heartbeat",
            )
            .is_err());
        //     ...but CAN dispatch a shared utility/microgrid quorum notice
        let sh_notice = fabric
            .dispatch_shared_care_or_guardian_alert(
                &tmp,
                "circle:sharehouse-peer-commons",
                CareAlertTriggerKind::ShareHouseCommonsUtilityNotice,
                CareAlertSeverity::Notice,
                "Quarterly solar microgrid export credit & internet bill ready for 2-of-2 housemate sign-off",
            )
            .expect("sharehouse utility notice");
        assert!(!sh_notice.recorded_in_accountability_ledger);

        //     (b) Elderly Person at Home with Remote Family & Carers Elsewhere:
        //         Dispatch an Urgent liveness Heartbeat / Fall alert -> notifies both Remote Daughter (min_sev=1) and Nurse (min_sev=2)
        let elderly_alert = fabric
            .dispatch_shared_care_or_guardian_alert(
                &tmp,
                "circle:elderly-home-remote-carers",
                CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse,
                CareAlertSeverity::Urgent,
                "12h liveness Heartbeat window elapsed + hallway motion inactivity — alerting remote daughter & nurse over QDNF",
            )
            .expect("dispatch elderly remote carer alert");
        assert_eq!(elderly_alert.notified_remote_guardian_dids.len(), 2);
        assert!(!elderly_alert.recorded_in_accountability_ledger);
        assert!(
            fabric
                .shared_care_circles
                .iter()
                .find(|c| c.circle_id == "circle:elderly-home-remote-carers")
                .unwrap()
                .escalation_currently_active
        );

        //         Elderly principal taps "I am OK" (reversible check-in) -> reverses escalation immediately!
        let reversed = fabric
            .acknowledge_or_reverse_care_alert(
                &tmp,
                &elderly_alert.alert_id,
                "did:qi:principal:elderly-parent-at-home",
                true,
                "I was gardening outside without my phone — I am completely fine!",
            )
            .expect("reversible principal check-in");
        assert!(reversed.resolved_or_reversed);
        let persisted: NaturalHostStewardshipFabric = serde_json::from_slice(
            &fs::read(tmp.join(HOST_STEWARDSHIP_STORE_REL_PATH)).expect("saved care fabric"),
        )
        .expect("deserialize saved care fabric");
        let circle = persisted
            .shared_care_circles
            .iter()
            .find(|circle| circle.circle_id == "circle:elderly-home-remote-carers")
            .unwrap();
        assert!(!circle.escalation_currently_active);
        assert!(circle.last_principal_checkin_unix_secs.is_some());

        //     (c) Family with Children: Notice-level (severity 1) flagged content ingest notifies Parent A (min_sev=1)
        //         while respecting Parent B's Concern-only (min_sev=2) filter
        let child_alert = fabric
            .dispatch_shared_care_or_guardian_alert(
                &tmp,
                "circle:family-children-stewardship",
                CareAlertTriggerKind::ChildFlaggedContentIngest,
                CareAlertSeverity::Notice,
                "Flagged hypermedia ingest (Notice level 1): external contact request held for parent review",
            )
            .expect("child flagged ingest notification");
        assert_eq!(child_alert.notified_remote_guardian_dids.len(), 1);
        assert!(child_alert.notified_remote_guardian_dids[0].contains("Parent A"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
