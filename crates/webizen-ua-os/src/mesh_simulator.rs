// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use crate::deb_builder::{build_deb_from_tree, generate_apt_repository_indexes};
use crate::host_daemon_runtime::{
    execute_hostd_command, handle_peer_p64_mcp_json_rpc_line, HostDaemonCommand, HostDaemonConfig,
    HEARTBEAT_FILE,
};
use crate::host_stewardship::{
    generate_tri_target_debian_suite, ActorJuralPersonalityKind, CareAlertTriggerKind,
    DispatchedCareAndGuardianAlert, HostNodeLocusKind, HostedProjectOwnershipStage,
    HostedProjectWorkload, IncorporatedEntityPricingTier, NaturalHostStewardshipFabric,
    SharedLivingAndCareTopologyKind, HOST_STEWARDSHIP_STORE_REL_PATH,
};
use crate::updater::{verify_quorum_update_manifest, QuorumUpdateProof, UpdateManifest};
use crate::{
    q_hash_60, trigger_one_tap_principal_ok_checkin, IdentifierSet, LoopbackHost,
    NativeEtalonStatus, TrustRoots, SENSITIVITY_PUBLIC_ONLY, SENSITIVITY_SANCTUARY_LOCAL_MAX,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha3::{Digest, Sha3_256};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use vibescript_abi::NQuin48;

const DESKTOP: &str = "did:qi:steward:desktop-principal";
const MOBILE: &str = "did:qi:steward:mobile-avf-pocket";
const AGENT: &str = "did:qi:agent:mesh-fixture";
const HOME: &str = "node:mesh-fixture-home";
const CAMPER: &str = "node:mesh-fixture-camper";
const DOMAIN: &str = "mesh-fixture.invalid";
const PACKAGE: &str = "webizen-hostd_0.1.1_amd64.deb";
const VERSION: &str = "0.1.1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MeshStageStatus {
    Verified,
    Simulated,
    Blocked,
    Failed,
}

impl MeshStageStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Verified => "Verified in the local fixture",
            Self::Simulated => "Fixture simulation passed",
            Self::Blocked => "Blocked without trust evidence",
            Self::Failed => "Fixture check failed",
        }
    }

    pub const fn passed(self) -> bool {
        matches!(self, Self::Verified | Self::Simulated)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MeshSimulationStageReceipt {
    pub stage_index: u8,
    pub title: String,
    pub status: MeshStageStatus,
    pub evidence: String,
    pub sealed_nquin_parity: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ThreeNodeMeshSimulationReceipt {
    pub stages: Vec<MeshSimulationStageReceipt>,
    pub apt_repo_inrelease_path: String,
    pub total_nquin_seals: usize,
    pub all_stages_passed: bool,
}

type StageCheck = Result<(MeshStageStatus, String), String>;

fn unix_seconds() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .map_err(|_| "Mesh simulation requires a clock after the Unix epoch".into())
}

fn allocate_fixture(storage: &Path) -> Result<PathBuf, String> {
    static NEXT_RUN: AtomicU64 = AtomicU64::new(0);
    fs::create_dir_all(storage).map_err(|error| error.to_string())?;
    let storage = storage.canonicalize().map_err(|error| error.to_string())?;
    let fixtures = storage.join("mesh-simulations");
    match fs::create_dir(&fixtures) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.to_string()),
    }
    let root = fixtures.canonicalize().map_err(|error| error.to_string())?;
    if root != fixtures {
        return Err("The mesh fixture directory must remain in the selected storage".into());
    }
    let timestamp = unix_seconds()?;
    for _ in 0..16 {
        let sequence = NEXT_RUN.fetch_add(1, Ordering::Relaxed);
        let folder = root.join(format!("run-{timestamp}-{}-{sequence}", std::process::id()));
        match fs::create_dir(&folder) {
            Ok(()) => return Ok(folder),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Could not allocate an unused mesh fixture directory".into())
}

fn new_fixture(storage: &Path) -> Result<NaturalHostStewardshipFabric, String> {
    let mut fabric = NaturalHostStewardshipFabric::default();
    fabric.primary_host_domain = DOMAIN.into();
    fabric.host_endpoint = "http://127.0.0.1:4242".into();
    fabric.attached_handles = vec![(DOMAIN.into(), "Fixture".into())];
    fabric
        .stewards
        .retain(|steward| matches!(steward.steward_did.as_str(), DESKTOP | MOBILE));
    if fabric.stewards.len() != 2 {
        return Err("Desktop and Pocket Co-Signer fixture stewards are required".into());
    }
    fabric.threshold_m = 2;
    fabric.update_queue.clear();
    fabric.hosted_projects.clear();
    fabric.host_fleet_nodes.clear();
    fabric.incorporated_entities.clear();
    fabric.care_alerts.clear();
    fabric.shared_care_circles.retain(|circle| {
        circle.topology == SharedLivingAndCareTopologyKind::ElderlyOrSupportedPersonWithRemoteCarers
    });
    let circle = fabric
        .shared_care_circles
        .first_mut()
        .ok_or("The supported-living care fixture has no circle")?;
    circle.host_node_id = HOME.into();
    circle.escalation_currently_active = false;
    circle.last_principal_checkin_unix_secs = None;
    fabric.register_host_fleet_node(
        storage,
        HOME,
        "Desktop Appliance and Home Sanctuary anchor",
        HostNodeLocusKind::HomeServerAppliance,
        vec![DESKTOP.into()],
        "http://127.0.0.1:4242",
        true,
    )?;
    fabric.register_host_fleet_node(
        storage,
        CAMPER,
        "Camper webizen-hostd relay",
        HostNodeLocusKind::CamperMobileRigNode,
        vec![DESKTOP.into(), MOBILE.into()],
        "http://127.0.0.1:4244",
        true,
    )?;
    fabric.save(storage)?;
    Ok(fabric)
}

fn read_fixture(storage: &Path) -> Result<NaturalHostStewardshipFabric, String> {
    let bytes = fs::read(storage.join(HOST_STEWARDSHIP_STORE_REL_PATH))
        .map_err(|error| error.to_string())?;
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

fn seal_stage(index: u8, title: &str, check: StageCheck) -> MeshSimulationStageReceipt {
    let (status, evidence) = match check {
        Ok(outcome) => outcome,
        Err(error) => (MeshStageStatus::Failed, error),
    };
    let evidence_key = format!("{}|{evidence}", status.label());
    let quin = NQuin48::invariant(
        &format!("urn:webizen:mesh-simulation:stage:{index}"),
        "https://ns.webcivics.net/stewardship#observedFixtureStage",
        q_hash_60(&evidence_key),
    );
    debug_assert!(quin.verify_parity());
    MeshSimulationStageReceipt {
        stage_index: index,
        title: title.into(),
        status,
        evidence,
        sealed_nquin_parity: quin.parity(),
    }
}

fn quorum_stage(
    storage: &Path,
    mut fabric: NaturalHostStewardshipFabric,
    gpu: &NativeEtalonStatus,
) -> StageCheck {
    let proposal = fabric.propose_host_update(storage, PACKAGE, VERSION, AGENT, true)?;
    if fabric
        .sign_or_dissent_host_update(
            storage,
            &proposal.proposal_id,
            AGENT,
            None,
            true,
            "Automated fixture approval",
        )
        .is_ok()
    {
        return Err("The instrument unexpectedly recorded a steward approval".into());
    }
    let mut voted = proposal.clone();
    for did in [DESKTOP, MOBILE] {
        voted = fabric.sign_or_dissent_host_update(
            storage,
            &proposal.proposal_id,
            did,
            None,
            true,
            "Isolated fixture review",
        )?;
    }
    if voted.required_m != 2
        || voted.total_n != 2
        || voted.votes.len() != 2
        || voted.votes.iter().any(|vote| !vote.approve)
        || voted.ratified_for_host_apply
        || voted.applied_to_host
    {
        return Err(
            "The recorded natural-person claims did not match the expected fixture quorum".into(),
        );
    }

    let host = LoopbackHost {
        daemon_base: "http://127.0.0.1:4242".into(),
        port: 4242,
        storage: storage.to_path_buf(),
        install_root: storage.to_path_buf(),
        trust: TrustRoots::for_storage(storage),
    };
    let mut fixture_gpu = gpu.clone();
    fixture_gpu.may_advertise_hardware_attested_vault = false;
    let suite = generate_tri_target_debian_suite(&host, &fixture_gpu, &fabric)?;

    // This candidate changes Debian metadata for the fixture; the existing
    // staged 0.1.0 archive remains intact in the repository.
    let control_path = suite.host_deb_tree.join("DEBIAN/control");
    let control = fs::read_to_string(&control_path).map_err(|error| error.to_string())?;
    if !control.lines().any(|line| line == "Version: 0.1.0") {
        return Err("The host package fixture has an unexpected starting version".into());
    }
    let candidate_control = control.replacen("Version: 0.1.0\n", "Version: 0.1.1\n", 1);
    fs::write(&control_path, candidate_control).map_err(|error| error.to_string())?;
    let candidate = build_deb_from_tree(&suite.host_deb_tree, &suite.suite_root_dir.join(PACKAGE));
    fs::write(control_path, control).map_err(|error| error.to_string())?;
    let candidate = candidate?;
    if candidate.version != VERSION {
        return Err("The built host fixture did not contain the proposed version".into());
    }
    let index = fabric
        .update_queue
        .iter()
        .position(|entry| entry.proposal_id == proposal.proposal_id)
        .ok_or("The fixture proposal disappeared before digest verification")?;
    fabric.update_queue[index].artifact_sha3_256 = candidate.sha3_256_hex.clone();
    fabric.save(storage)?;

    let manifest = UpdateManifest {
        version: VERSION.into(),
        sku: "appliance".into(),
        artifact_name: PACKAGE.into(),
        sha3_256_hex: candidate.sha3_256_hex.clone(),
        signature_hex: String::new(),
        pubkey_hex: String::new(),
        quorum: Some(QuorumUpdateProof {
            proposal_id: proposal.proposal_id,
            approvals: Vec::new(),
        }),
    };
    let Err(verification_error) = verify_quorum_update_manifest(&manifest, &fabric) else {
        return Err(
            "Two unsigned fixture votes unexpectedly passed cryptographic verification".into(),
        );
    };
    if !verification_error.contains("2 distinct steward approvals") {
        return Err(format!("The quorum verifier rejected the fixture for an unexpected reason: {verification_error}"));
    }

    let mut packages = suite.built_deb_packages;
    packages.push(candidate);
    let steward_dids = vec![DESKTOP.to_owned(), MOBILE.to_owned()];
    let apt = generate_apt_repository_indexes(
        &suite.suite_root_dir,
        &packages,
        DOMAIN,
        fabric.threshold_m,
        &steward_dids,
    )?;
    if apt.signed {
        return Err("Unexpected signing evidence appeared in the isolated mesh fixture".into());
    }

    Ok((
        MeshStageStatus::Blocked,
        format!(
            "The instrument vote was denied and two local approvals were recorded for {PACKAGE}. The artifact and APT indexes were staged at {}. Fixture signing keys were absent, so the update remains unratified.",
            apt.repo_root.display()
        ),
    ))
}

fn project_stage(cfg: &HostDaemonConfig) -> StageCheck {
    let project_id = "project:mesh-fixture";
    execute_hostd_command(
        cfg,
        HostDaemonCommand::RegisterProject {
            project_id: project_id.into(),
            title: "Loopback fixture project".into(),
            domain: DOMAIN.into(),
            stage: HostedProjectOwnershipStage::PersonalOwnerProject,
            steward_did: DESKTOP.into(),
            entity: None,
        },
    )?;
    let transitioned = execute_hostd_command(
        cfg,
        HostDaemonCommand::TransitionQcdn {
            project_id: project_id.into(),
            stage: HostedProjectOwnershipStage::TransitionedPersonalToGroupProject,
            threshold: 2,
            stewards: vec![MOBILE.into()],
            mirrors: vec![HOME.into(), CAMPER.into()],
        },
    )?;
    let project: HostedProjectWorkload =
        serde_json::from_value(transitioned).map_err(|error| error.to_string())?;
    if project.ownership_stage != HostedProjectOwnershipStage::TransitionedPersonalToGroupProject
        || project.project_quorum_threshold_m != 2
        || ![HOME, CAMPER].iter().all(|node| {
            project
                .qcdn_swarm_mirror_dids
                .iter()
                .any(|mirror| mirror.as_str() == *node)
        })
    {
        return Err("The project transition did not preserve the two mirror assignments".into());
    }

    // Write and verify identical payloads at both designated fixture loci.
    let payload = serde_json::to_vec(&project).map_err(|error| error.to_string())?;
    let digest = Sha3_256::digest(&payload);
    for target in ["home-mirror", "camper-mirror"] {
        let directory = cfg.storage.join(target);
        fs::create_dir(&directory).map_err(|error| error.to_string())?;
        let path = directory.join("project.json");
        fs::write(&path, &payload).map_err(|error| error.to_string())?;
        let copied = fs::read(path).map_err(|error| error.to_string())?;
        if Sha3_256::digest(&copied) != digest {
            return Err("The fixture mirror digest differed from the project payload".into());
        }
    }
    Ok((
        MeshStageStatus::Simulated,
        format!(
            "The project moved from personal to cooperative ownership. Its serialized state matched on both fixture mirror directories (SHA3-256 {digest:x})."
        ),
    ))
}

fn licensing_stage(cfg: &HostDaemonConfig) -> StageCheck {
    let mut fabric = read_fixture(&cfg.storage)?;
    let entity = "did:qi:corp:mesh-fixture-cooperative";
    let registered = fabric.register_incorporated_entity_commercial_licence(
        &cfg.storage,
        entity,
        "Mesh Fixture Cooperative",
        "Fixture register",
        IncorporatedEntityPricingTier::SmallOrCooperativeCorporation,
        vec!["Fixture cooperative governance".into()],
        vec![DESKTOP.into()],
        "Fixture valuation in cents",
    )?;
    let audit = |actor, entity: Option<&str>| {
        execute_hostd_command(
            cfg,
            HostDaemonCommand::AuditJuralLicense {
                actor,
                entity: entity.map(str::to_owned),
            },
        )
    };
    let principal = audit(ActorJuralPersonalityKind::NaturalPersonPrincipal, None)?;
    if principal["commercial_licence_required"] != false
        || principal["monthly_fee_cents"].as_u64() != Some(0)
    {
        return Err("The personal-use lane did not remain free of commercial fees".into());
    }

    for actor in [
        ActorJuralPersonalityKind::NaturalPersonAgentOfIncorporatedEntity,
        ActorJuralPersonalityKind::IncorporatedLegalEntity,
    ] {
        if audit(actor, None).is_ok() {
            return Err("The commercial gate admitted an actor without an entity".into());
        }
        let commercial = audit(actor, Some(entity))?;
        if registered.monthly_base_fee_cents == 0
            || commercial["commercial_licence_required"] != true
            || commercial["monthly_fee_cents"].as_u64() != Some(registered.monthly_base_fee_cents)
        {
            return Err("A commercial actor did not incur the fixture entity's licence fee".into());
        }
    }
    Ok((
        MeshStageStatus::Verified,
        format!(
            "The natural-person principal owed 0 cents per month. The incorporated entity and its representative each mapped to the entity's {}-cent monthly licence.",
            registered.monthly_base_fee_cents,
        ),
    ))
}

fn care_stage(cfg: &HostDaemonConfig) -> StageCheck {
    let fabric = read_fixture(&cfg.storage)?;
    let circle = fabric
        .shared_care_circles
        .first()
        .ok_or("The supported-living fixture has no care circle")?;
    let circle_id = circle.circle_id.clone();
    let principal = circle.principal_did.clone();
    let observation_age = circle
        .heartbeat_grace_secs
        .checked_add(1)
        .ok_or("The fixture heartbeat duration overflowed")?;
    let observed = unix_seconds()?
        .checked_sub(observation_age)
        .ok_or("The fixture clock cannot represent a stale heartbeat")?;
    let observation = serde_json::to_vec(&json!([{
        "circle_id": circle_id,
        "principal_did": principal,
        "last_checkin_unix_secs": observed,
    }]))
    .map_err(|error| error.to_string())?;
    fs::write(cfg.storage.join(HEARTBEAT_FILE), observation).map_err(|error| error.to_string())?;

    let poll = |dispatch_lapse| {
        execute_hostd_command(
            cfg,
            HostDaemonCommand::PollCareHeartbeats {
                circle: Some(circle_id.clone()),
                dispatch_lapse,
                acknowledgement: None,
            },
        )
    };
    let lapsed = poll(true)?;
    let dispatched: Vec<DispatchedCareAndGuardianAlert> =
        serde_json::from_value(lapsed["dispatched"].clone()).map_err(|error| error.to_string())?;
    let [dispatch] = dispatched.as_slice() else {
        return Err("A stale fixture heartbeat did not dispatch exactly one care alert".into());
    };
    if dispatch.trigger_kind != CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse {
        return Err("The simulated heartbeat dispatched an unrelated alert".into());
    }

    let mut saved = read_fixture(&cfg.storage)?;
    if trigger_one_tap_principal_ok_checkin(
        &mut saved,
        &cfg.storage,
        &dispatch.alert_id,
        AGENT,
        Some("Fixture instrument attempt"),
    )
    .is_ok()
    {
        return Err("An unrelated fixture instrument acknowledged the care alert".into());
    }
    let checkin = trigger_one_tap_principal_ok_checkin(
        &mut saved,
        &cfg.storage,
        &dispatch.alert_id,
        &principal,
        Some("Isolated principal check-in"),
    )?;
    if !checkin.escalation_reversed
        || poll(true)?["dispatched"]
            .as_array()
            .is_none_or(|alerts| !alerts.is_empty())
    {
        return Err(
            "The principal check-in failed to clear or suppress the fixture escalation".into(),
        );
    }

    Ok((
        MeshStageStatus::Verified,
        format!(
            "A stale heartbeat raised a local fixture alert. Only its principal cleared it, and the next poll did not reopen it. Care receipt parity {:016x}.",
            checkin.sealed_nquin_parity
        ),
    ))
}

fn peer_stage() -> StageCheck {
    let request = |id, prompt, sensitivity| {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": "llm_infer",
                "arguments": { "prompt": prompt, "sensitivity": sensitivity, "max_tokens": 16 },
            },
        })
        .to_string()
    };
    let mut attempts = 0;
    for (id, prompt, sensitivity) in [
        (
            1,
            "Sanctuary100 synthetic fixture marker",
            SENSITIVITY_SANCTUARY_LOCAL_MAX,
        ),
        (
            2,
            "Sanctuary100 fixture with an incorrect public label",
            SENSITIVITY_PUBLIC_ONLY,
        ),
    ] {
        let protected =
            handle_peer_p64_mcp_json_rpc_line(&request(id, prompt, sensitivity), |_, _| {
                attempts += 1;
                Ok("Protected response would breach the fixture boundary".into())
            });
        if attempts != 0
            || protected
                .as_ref()
                .and_then(|reply| reply["error"]["code"].as_i64())
                != Some(-32001)
        {
            return Err(
                "The public-only dispatcher presented protected fixture data to its backend".into(),
            );
        }
    }
    let public_prompt = "Public fixture prompt";
    let public = handle_peer_p64_mcp_json_rpc_line(
        &request(3, public_prompt, SENSITIVITY_PUBLIC_ONLY),
        |prompt, tokens| {
            attempts += 1;
            if prompt != public_prompt || tokens != 16 {
                return Err("Unexpected fixture prompt or token budget".into());
            }
            Ok("Synthetic loopback backend receipt".into())
        },
    );
    if attempts != 1
        || public
            .as_ref()
            .and_then(|reply| reply["result"]["content"][0]["text"].as_str())
            != Some("Synthetic loopback backend receipt")
    {
        return Err("The public fixture request did not produce the expected response".into());
    }
    Ok((
        MeshStageStatus::Simulated,
        "Public JSON-RPC reached the injected test backend. The protected request was rejected before inference. Live .p64 inference was outside this fixture.".into(),
    ))
}

/// Exercise the mesh lifecycle on dedicated fixture storage with a synthetic public
/// backend. This does not connect to peers, apply system updates, or access live care data.
pub fn run_three_node_mesh_simulation(
    storage: &Path,
    ids: &IdentifierSet,
    gpu: &NativeEtalonStatus,
) -> Result<ThreeNodeMeshSimulationReceipt, String> {
    if ids.quins.iter().any(|quin| !quin.verify_parity()) {
        return Err("The session identifier projection contains a broken parity seal".into());
    }
    let fixture = allocate_fixture(storage)?;
    let fabric = new_fixture(&fixture)?;
    let cfg = HostDaemonConfig::for_storage(&fixture);
    let quorum = quorum_stage(&fixture, fabric, gpu);
    let stages = vec![
        seal_stage(1, "Natural-person quorum and Debian update", quorum),
        seal_stage(
            2,
            "Cooperative project and Q-CDN mirrors",
            project_stage(&cfg),
        ),
        seal_stage(3, "Three-way jural licensing gate", licensing_stage(&cfg)),
        seal_stage(
            4,
            "Care heartbeat and Pocket Co-Signer check-in",
            care_stage(&cfg),
        ),
        seal_stage(5, "Public-only peer MCP admission", peer_stage()),
    ];
    Ok(ThreeNodeMeshSimulationReceipt {
        total_nquin_seals: stages.len(),
        all_stages_passed: stages.iter().all(|stage| stage.status.passed()),
        stages,
        apt_repo_inrelease_path: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IdentityValidation;

    #[test]
    fn executes_five_stage_three_node_mesh_lifecycle_end_to_end() {
        let test_storage = allocate_fixture(&std::env::temp_dir()).unwrap();
        let ids = IdentifierSet {
            fabric_did: "did:qi:private:real-session-fixture".into(),
            records: Vec::new(),
            bindings: Vec::new(),
            validation: IdentityValidation::Underdetermined {
                distinct: 0,
                attested: 0,
            },
            quins: Vec::new(),
            turtle_path: PathBuf::new(),
            validation_summary: String::new(),
        };
        let gpu = NativeEtalonStatus {
            native_gpu_active: false,
            primary_adapter: None,
            adapters: Vec::new(),
            anvil_profile: anvil_core::NodeProfile64::empty(),
            specializer_hints: anvil_gpu::hints_for_pci(0, 0),
            native_vibe_provider_count: 0,
            nquin_parity_verified: false,
            may_advertise_hardware_attested_vault: false,
            summary_label: "Software fixture".into(),
        };

        let receipt = run_three_node_mesh_simulation(&test_storage, &ids, &gpu).unwrap();
        assert_eq!(receipt.total_nquin_seals, 5);
        assert_eq!(
            receipt
                .stages
                .iter()
                .map(|stage| stage.status)
                .collect::<Vec<_>>(),
            [
                MeshStageStatus::Blocked,
                MeshStageStatus::Simulated,
                MeshStageStatus::Verified,
                MeshStageStatus::Verified,
                MeshStageStatus::Simulated,
            ]
        );
        assert!(!receipt.all_stages_passed);
        assert!(receipt.apt_repo_inrelease_path.is_empty());
        let mut runs = fs::read_dir(test_storage.join("mesh-simulations")).unwrap();
        let run = runs.next().expect("one isolated run").unwrap().path();
        assert!(runs.next().is_none());
        let apt = run.join("delivery/tri-target-debian-trixie/dists/trixie");
        let amd64_packages = fs::read_to_string(apt.join("main/binary-amd64/Packages")).unwrap();
        assert!(amd64_packages.contains("Package: webizen-hostd\nVersion: 0.1.0"));
        assert!(amd64_packages.contains("Package: webizen-hostd\nVersion: 0.1.1"));
        let arm64_packages = fs::read_to_string(apt.join("main/binary-arm64/Packages")).unwrap();
        assert!(arm64_packages.contains("Package: webizen-mobile-avf"));
        assert!(!apt.join("InRelease").exists());
        assert!(!test_storage.join(HOST_STEWARDSHIP_STORE_REL_PATH).exists());
        assert!(receipt
            .stages
            .iter()
            .all(|stage| !stage.evidence.contains(&ids.fabric_did)));
        let _ = fs::remove_dir_all(test_storage);
    }
}
