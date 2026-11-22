//! Shared User-Agent OS Runtime (`webizen-ua-os`) for Guest (app on an existing
//! Windows / macOS / Linux Device OS) and Appliance (overlay on unmodified Debian 13 Trixie).
//!
//! Responsibilities:
//! - **Native Etalon GPU & Anvil Hardware Probe (`native_gpu`)**: Directly initializes
//!   `etalon::Instance` in native Vulkan / DX12 / Metal mode, binds Anvil `NodeProfile64`
//!   and `SpecializerHints`, and verifies all 11 native `vibescript-abi` providers under
//!   `TargetProfileMask::NATIVE_DESKTOP`.
//! - **Canonical `UaOsHost` & Qap Installer/Launcher (`qap_host`)**: Atomic Qap installation
//!   (`install_package_atomic`), `.qpkg` ZIP extraction, `yaml-ld-q42` synthetic manifest
//!   generation, fail-closed Sentinel `LeaseSheet`, and `LaunchPlan` without `APP_STATE`.
//! - **QualiaDB Enumerated Identity & Sidecar Supervision (`identifiers`, `daemon`)**:
//!   Extends QualiaDB's `validate_enumerated_identity` (noisy-OR `< 1.0`, `DefinitiveCollapse`
//!   protection), 48-byte `NQuin48` parity-checked records, and `:4242/invoke` HTTP bridge.
//! - **Unified Native Poet & Hypermedia Production Workbench (`poet_synth`)**:
//!   Hosts native VibeScript 0.1 reactive cells, functions, hooks, NLP gazetteer `NQuin48`
//!   sealing, unified SLD/LLM vault discovery (`C:\LLM_files`), and the 150-morphism
//!   Hypermedia Production pipeline so the user does not need a separate Synth app running.
//! - **Dual-SKU Delivery & Synth-Style Ed25519 Updater (`updater`)**:
//!   Implements `webizen-os-shell update check|verify|upgrade|apply` (`C7`) and emits both
//!   SKU A (Guest binary installer) and SKU B (Debian 13 Trixie `/opt/webizen` overlay)
//!   artefacts (`C6`, `A7`, `D5`, `F4`).

mod browser_app;
mod daemon;
pub mod deb_builder;
pub mod desktop_appliance;
pub mod holo_10d_specimen;
pub mod host_daemon_runtime;
mod host_stewardship;
mod identifiers;
pub mod intent_router;
mod llm_management;
pub mod mesh_simulator;
pub mod mobile_avf;
mod native_gpu;
mod os_env;
mod poet_synth;
mod qap_host;
pub mod sanctuary_onboarding;
mod semantic_fs;
mod session;
mod updater;

pub use browser_app::{
    evaluate_hybrid_page_request, inspect_browser_product, resolve_mode_for_uri,
    upgrade_browser_product_independent, BrowserExecutionMode, BrowserProductStatus,
    BrowserSocialGraph, HybridBridgeReceipt, HybridCapabilityChannelSpec,
    HybridWebCapabilityChannel, SemanticBookmark, SemanticSocialGroup, SemanticWebAnnotation,
    SocialDisclosureScope, BROWSER_APP_DID, BROWSER_GIT_DID, BROWSER_QAP_ID,
    BROWSER_SOCIAL_STORE_REL_PATH, MAX_GOVERNED_VIBESCRIPT_CELL_BYTES,
};
pub use daemon::{
    ensure_daemon, http_invoke, ping_daemon, probe_only, DaemonError, DaemonHandle, DaemonStatus,
    HttpInvokeResult,
};
pub use deb_builder::{
    build_deb_from_tree, build_tri_target_debs_from_receipt, generate_apt_repository_indexes,
    generate_signed_apt_repository, verify_deb_ar_container, AptRepositoryReceipt, DebBuildReceipt,
};
pub use desktop_appliance::{
    build_desktop_appliance_profile, sync_fleet_from_fabric, DesktopApplianceProfile,
    DesktopWorkspacePreset, LocusFleetNodeSummary, MultiLocusFleetControllerState,
    TrayMenuBindingSpec,
};
pub use holo_10d_specimen::{
    decode_holo_10d_container, encode_holo_10d_container, proxy_corners_for_specimen,
    resolve_holo_10d_specimen, seed_default_holo_10d_specimens, HoloShaderVisualizationMode,
    VerifiedHolo10dSpecimen, DEFAULT_HOLO_SPECIMENS, FIELD_KIND_HOLO_PHASE_SPACE,
    HOLO_HEADER_BYTES, HOLO_MAX_RAYMARCH_STEPS,
};
pub use host_daemon_runtime::{
    execute_hostd_command, handle_peer_p64_mcp_json_rpc_line, parse_hostd_args, run_daemon_tick,
    serve_hostd_http_control, serve_peer_p64_mcp_tcp, serve_peer_p64_mcp_tcp_with_local_model,
    CareHeartbeatReport, CareHeartbeatStatus, HostCareAcknowledgement, HostDaemonAuthMode,
    HostDaemonCommand, HostDaemonConfig, HostDaemonStatusSnapshot, HostDaemonTickReport,
    PublicP64TokenOutput, HOSTD_USAGE,
};
pub use host_stewardship::{
    generate_tri_target_debian_suite, validate_human_centric_host_nomenclature,
    ActorJuralPersonalityKind, CareAlertSeverity, CareAlertTriggerKind, DebianTargetProfile,
    DispatchedCareAndGuardianAlert, HostNodeLocusKind, HostStewardshipArchetype,
    HostUpdateProposal, HostedProjectOwnershipStage, HostedProjectWorkload,
    IncorporatedEntityPricingTier, IncorporatedLegalEntityRecord, NaturalHostStewardshipFabric,
    PersonalHostNode, ResponsibleNaturalSteward, SharedGuardianshipAndCareCircle,
    SharedLivingAndCareTopologyKind, SharedSystemParticipant, StewardQuorumVote,
    TriTargetDebianSuiteReceipt, CBOR_LD_TAG_HEADER, HOST_QUORUM_QUEUE_SLOTS,
    HOST_STEWARDSHIP_STORE_REL_PATH,
};
pub use identifiers::{
    enumerate_session_identifiers, is_valid_natural_agent_label, issue_for_requestor, q_hash_60,
    validate_enumerated_identity, Ask, CryptoScheme, IdentifierBinding, IdentifierFabricPlane,
    IdentifierRecord, IdentifierSet, IdentityValidation, Requestor, SanctuaryReply,
    CO_ATTESTATION_BUNDLE_FORMULA, DID_Q42_QRC_LABEL, NATURAL_AGENT_FORBIDDEN_LABELS,
    PLANE_CLAIM_OPINION_TITLE, PLANE_INSTRUMENT_TITLE, PLANE_NATURAL_AGENT_TITLE,
    PLANE_SPATIOTEMPORAL_HANDLE_TITLE, VC_ORIGIN_INTEGRITY_DISCLAIMER,
};
pub use intent_router::{classify_semantic_intent, ClassifiedSemanticIntent, SemanticIntentAction};
pub use llm_management::{
    compile_principal_sub_agent_did, AgentInvocationTurnReceipt, EnvironmentAgentRosterEntry,
    ExternalLlmPlatformAccount, ExternalLlmProviderKind, ExternalRemoteConsentRule,
    GovernedAgentBackendBinding, LlmAndAgentManagementRegistry, ManagedModelLifecycleState,
    ManagedModelResidency, ManagedModelTask, ManagedP64ModelRecord, McpTransportConfig,
    P64ConversionAndPurgeReceipt, LLM_MANAGEMENT_STORE_REL_PATH, P64_MAGIC_HEADER,
    SENSITIVITY_COMMONS_GROUP, SENSITIVITY_PUBLIC_ONLY, SENSITIVITY_RESTRICTED,
    SENSITIVITY_SANCTUARY_LOCAL_MAX,
};
pub use mesh_simulator::{
    run_three_node_mesh_simulation, MeshSimulationStageReceipt, MeshStageStatus,
    ThreeNodeMeshSimulationReceipt,
};
pub use mobile_avf::{
    probe_mobile_avf_sanctuary, trigger_one_tap_principal_ok_checkin, AvfIsolationState,
    AvfSanctuaryProbe, MobileAvfSanctuaryProfile, MobileOneTapCheckInReceipt,
    P64ThermalBatteryGovernor, ThermalPressureLevel, ThumbArcOrbGeometry, ThumbArcOrbSector,
    VsockChannelDescriptor, VsockTierIsolation,
};
pub use native_gpu::{
    probe_native_etalon, probe_native_vibe_providers, NativeEtalonStatus, NativeGpuAdapterInfo,
    NODE_PROFILE_FILE_NAME,
};
pub use os_env::OsEnvironment;
pub use poet_synth::{
    core_semantic_instruments_catalog, discover_unified_llm_vault, hypermedia_pillar_catalog,
    list_native_vibe_providers, poet_analyze_and_seal_gazetteer, poet_eval_native_cell,
    poet_eval_native_program, poet_open_lexicon_pack, produce_hypermedia_vibe,
    run_local_model_readiness_probe, shacl_extension_families_catalog, GazetteerSpanHit,
    HypermediaPillarSummary, HypermediaProduceReceipt, LocalLlmModelEntry,
    LocalModelReadinessReceipt, NativeProviderSummary, PoetCellEntry, PoetEvalResult,
    PoetGazetteerResult, SemanticInstrumentSummary, ShaclExtensionFamilySummary,
    HYPERMEDIA_MORPHISM_COUNT,
};
pub use qap_host::{
    build_lease_sheet, install_from_source, install_package_atomic, load_install_registry,
    plan_qapp_launch, revoke_package, save_install_registry, seed_bundled_qaps,
    synthesize_qapp_json_from_yaml, unpack_qpkg_zip, validate_package_id, InstallPolicy,
    InstallSource, LaunchKind, LaunchPlan, LeaseFamily, LeaseSheet, QappHostExtension,
    QappInstallError, QappInstallRegistry, QappLaunchRequirement, QappPackageManifest,
    QappRegistryEntry, TrustRoots, UaOsHost, BUNDLED_FIRST_PARTY_QAPS, FIRST_PARTY_PUBKEY,
};
pub use sanctuary_onboarding::{
    build_sanctuary_onboarding_state, pair_and_test_pocket_cosigner,
    simulate_shamir_recovery_ceremony, PocketCosignerPairingState, SanctuaryOnboardingState,
    SanctuaryVaultSlotSpec, ShamirGuardianShareHolder,
};
pub use semantic_fs::{
    build_entry_nquins, extract_semantic_features, recommend_toolboxes_for_entry,
    AssetAnalyticsAttachment, BundledSubAsset, CommonsShareScope, SemanticAssetEntry,
    SemanticAssetRole, SemanticFacetFilter, SemanticFacetTallies, SemanticFileSystem,
    SemanticSection, SemanticToolboxId, SemanticToolboxSpec, HYPERMEDIA_LIBRARY_REL_PATH,
};
pub use session::{
    device_os_username, guest_storage_dir, DeviceOs, DeviceSession, HostLink, HostLinkStatus,
    LoopbackHost, Sku,
};
pub use updater::{
    generate_dual_sku_delivery_artifacts, quorum_update_signing_payload, run_updater,
    vendor_update_signing_payload, verify_detached_ed25519, verify_quorum_update_manifest,
    DualSkuPackagingReceipt, QuorumApproval, QuorumUpdateProof, UpdateAction, UpdateManifest,
    UpdateReceipt, FIRST_PARTY_UPDATER_PUBKEY_HEX,
};

pub const DEFAULT_DAEMON_PORT: u16 = 4242;
pub const PID_FILE_NAME: &str = "daemon-service.json";
pub const QAPPS_DIR: &str = "Qapps";
pub const QAPP_PACKAGE_MANIFEST: &str = "qapp.json";
pub const QAPP_REGISTRY_FILE: &str = "registry.json";
