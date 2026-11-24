use aether::refract::{
    self, AssertionEvidenceLevel, AssertionKind, BitemporalStamp, ChallengeState,
    ContextualAssertionView, ContributionStage, ContributionStageView, DimensionOutcome,
    FairnessReviewView, InstrumentReadiness, Lens, QuinSlot, QuinWindow, QuinWindowLens,
};
use aether::refract_bridge::RefractStore;
use std::sync::OnceLock;
use webizen_ua_os::{
    build_desktop_appliance_profile, build_sanctuary_onboarding_state,
    core_semantic_instruments_catalog, inspect_browser_product, load_workspace_snapshot,
    poet_analyze_and_seal_gazetteer, poet_eval_native_cell, poet_open_lexicon_pack,
    probe_mobile_avf_sanctuary, probe_only, q_hash_60, sync_fleet_from_fabric,
    AuthoringWorkflowSnapshot, BrowserExecutionMode, BrowserProductStatus, BrowserSocialGraph,
    CapabilityLeaseScope, ChoraWorkerSupervisorSnapshot, DesktopApplianceProfile, DeviceSession,
    GovernedVibeActionReport, HoloShaderVisualizationMode, HostDaemonStatusSnapshot,
    HybridBridgeReceipt, HypermediaProduceReceipt, IdentifierFabricPlane, LaunchPlan,
    LlmAndAgentManagementRegistry, LocalLlmModelEntry, LocalModelReadinessReceipt,
    MobileAvfSanctuaryProfile, NativeGpuAdapterInfo, NaturalHostStewardshipFabric,
    OsControlCenterState, OsEnvironment, PersistedWorkspaceSnapshot, PoetCellEntry,
    PoetGazetteerResult, QappRegistryEntry, QecpCredentialEnvelope, SanctuaryOnboardingState,
    SemanticFacetFilter, SemanticFileSystem, SemanticToolboxId, ShamirQ42jBackupArchive,
    ShellRendererHealthSummary, Sku, SolidTurtleSyncReceipt, ThreeNodeMeshSimulationReceipt,
    UaOsHost,
};

static SESSION: OnceLock<DeviceSession> = OnceLock::new();

/// Build the Guest/Appliance session once at process start (main calls this).
pub fn init_session(session: DeviceSession) -> &'static DeviceSession {
    let _ = SESSION.set(session);
    SESSION.get().expect("session set")
}

pub fn session() -> Option<&'static DeviceSession> {
    SESSION.get()
}

/// First-class OS-Shell volumes (no life-domain top chrome; apps are apps).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeStage {
    None,
    Talk,
    Mail,
    Directory,
    Browser,
    Keep,
    Library,
    Instruments,
    Poet,
    Wellfair,
    Wallet,
    Projects,
    Anatomy,
    Settings,
    Admin,
}

/// 10 Canonical Settings & Environment Setup sections (aligned with `webizen-studio/src/components/settings/types.rs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsSection {
    #[default]
    Health,
    Data,
    Models,
    People,
    Privacy,
    Appearance,
    Device,
    Backup,
    Services,
    Technical,
}

pub const ALL_SETTINGS_SECTIONS: [SettingsSection; 10] = [
    SettingsSection::Health,
    SettingsSection::Data,
    SettingsSection::Models,
    SettingsSection::People,
    SettingsSection::Privacy,
    SettingsSection::Appearance,
    SettingsSection::Device,
    SettingsSection::Backup,
    SettingsSection::Services,
    SettingsSection::Technical,
];

impl SettingsSection {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Health => "1. Setup health",
            Self::Data => "2. Data & memory",
            Self::Models => "3. AI instruments",
            Self::People => "4. People & reachability",
            Self::Privacy => "5. Privacy & sanctuary",
            Self::Appearance => "6. Appearance & access",
            Self::Device => "7. Person & devices",
            Self::Backup => "8. Backup & recovery",
            Self::Services => "9. Services & updates",
            Self::Technical => "10. All technical settings",
        }
    }

    pub const fn search_terms(self) -> &'static str {
        match self {
            Self::Health => "setup ready attention repair status sentinel foundations renderer GPU backend",
            Self::Data => "data storage path quota memory q42 10d hmc m64 emf qac p64",
            Self::Models => "ai model llm gguf p64 etalon inference readiness probe",
            Self::People => "people network reachability qdnf wireguard nym mesh mail solid",
            Self::Privacy => {
                "privacy sanctuary vault container argon2id duress anvil deadman incapacity"
            }
            Self::Appearance => "appearance theme colour contrast accessibility reduced motion workspace snapshot save",
            Self::Device => {
                "person device apparatus fleet four plane naturalagent claim handle instrument"
            }
            Self::Backup => "backup recovery restore export import q42j qnf solid off-ramp",
            Self::Services => "services daemon updates logs ports qaps dual-sku debian trixie jobs capability lease notifications",
            Self::Technical => {
                "advanced technical raw configuration providers shacl semantic instruments renderer GPU fallback"
            }
        }
    }
}

/// Stage presentation geometry controlled by the Poet 8-Sector Radial Command Orb (`POET-SPEC-011`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StageChromeMode {
    #[default]
    DefaultDepth,
    FullStage,
    FloatLayer,
}

const STEWARDSHIP_SLOT_START: usize = 4;
const INSTRUMENT_SLOT_START: usize = 10;
const SEEDED_QUIN_SLOTS: usize = 16;

/// Owner-thread UI projections of session data and local semantic review drafts.
#[derive(Clone, Debug)]
pub struct WebizenRefractModel {
    pub quin_window: QuinWindow<64>,
    pub instrument_readiness: [InstrumentReadiness; 6],
    pub wellfair_review: FairnessReviewView,
    pub assertion: ContextualAssertionView,
    pub assertion_evidence: Option<AssertionEvidenceLevel>,
    pub contribution: ContributionStageView,
}

impl WebizenRefractModel {
    /// Project the existing session without changing any database or authority record.
    pub fn from_session(sess: &DeviceSession) -> Self {
        let mut model = Self::placeholder();
        for (index, plane) in IdentifierFabricPlane::ALL.into_iter().enumerate() {
            // Bind only matching Identifier Fabric planes. The current device
            // catalogue also contains OS agent and hardware hints that describe tools.
            let canonical = sess
                .identifiers
                .bindings
                .iter()
                .zip(&sess.identifiers.quins)
                .find_map(|(binding, quin)| {
                    let binding_plane = match binding.plane {
                        "who" | "NaturalAgent" => IdentifierFabricPlane::NaturalAgent,
                        "claim" | "ClaimOpinion" => IdentifierFabricPlane::ClaimOpinion,
                        "handle" | "locus" | "SpatiotemporalHandle" => {
                            IdentifierFabricPlane::SpatiotemporalHandle
                        }
                        "tool" | "instrument" | "hardware" | "agent" | "Instrument" => {
                            IdentifierFabricPlane::Instrument
                        }
                        _ => return None,
                    };
                    (binding_plane == plane && quin.verify_parity()).then_some(quin)
                });
            if let Some(quin) = canonical {
                // The view uses Refract's salted parity. Preserve the canonical
                // payload words and leave the original record and seal untouched.
                model.quin_window.slots[index] = QuinSlot::new(
                    quin.subject(),
                    quin.predicate(),
                    quin.object(),
                    quin.context(),
                    quin.metadata(),
                );
            }
        }

        let stewardship = stewardship_slots(&sess.host_stewardship);
        model.quin_window.slots[STEWARDSHIP_SLOT_START..INSTRUMENT_SLOT_START]
            .copy_from_slice(&stewardship);
        if !sess.browser_social.annotations.is_empty() {
            model.assertion = ContextualAssertionView {
                summary: "This session contains an annotation. Its statement kind and evidence are reviewed separately.",
                ..model.assertion
            };
            model.assertion_evidence = Some(AssertionEvidenceLevel::Contextual);
        }
        model
    }

    /// An empty session projection with reserved, parity-valid slot ranges.
    pub fn placeholder() -> Self {
        let mut quin_window = QuinWindow::new();
        let mut seeded = [QuinSlot::default(); SEEDED_QUIN_SLOTS];
        for (slot, plane) in seeded.iter_mut().zip(IdentifierFabricPlane::ALL) {
            *slot = QuinSlot::new(
                q_hash_60(plane.code()),
                q_hash_60("urn:webizen:inspector:identifier-fabric-plane"),
                0,
                0,
                0,
            );
        }
        for (slot, instrument) in seeded[INSTRUMENT_SLOT_START..]
            .iter_mut()
            .zip(core_semantic_instruments_catalog())
        {
            *slot = QuinSlot::new(
                q_hash_60(instrument.id),
                q_hash_60("urn:webizen:inspector:semantic-instrument-manifest"),
                q_hash_60(instrument.evaluator),
                q_hash_60(instrument.plane_scope),
                0,
            );
        }
        for mut slot in seeded {
            slot.recompute_parity();
            let _ = quin_window.insert(slot, BitemporalStamp::default());
        }
        Self {
            quin_window,
            // Inspectable manifests alone do not establish local collection or activation.
            instrument_readiness: [InstrumentReadiness::Uncollected; 6],
            wellfair_review: FairnessReviewView {
                outcomes: [DimensionOutcome::Held; 6],
                remedy_notes: ["Awaiting evidence for this dimension."; 6],
            },
            assertion: ContextualAssertionView {
                kind: AssertionKind::Observation,
                challenge: ChallengeState::Unchallenged,
                independent_source_count: 0,
                summary: "No contextual assertion is selected.",
            },
            assertion_evidence: None,
            contribution: ContributionStageView {
                current_stage: ContributionStage::RequestAndAgreement,
                target_threshold_cents: 0,
                pool_funded_cents: 0,
                contributor_settled_cents: 0,
                waived_obligation_cents: 0,
                unresolved_challenges: 0,
                human_transition_approved: false,
            },
        }
    }
}

/// Stable field projection shared by the slot inspector and reconciliation helpers.
pub fn quin_window_lens() -> impl Lens<WebizenRefractModel, Target = QuinWindow<64>> {
    refract::lens!(WebizenRefractModel => 0: quin_window)
}

pub fn instrument_readiness_lens(
) -> impl Lens<WebizenRefractModel, Target = [InstrumentReadiness; 6]> {
    refract::lens!(WebizenRefractModel => 1: instrument_readiness)
}

pub fn wellfair_review_lens() -> impl Lens<WebizenRefractModel, Target = FairnessReviewView> {
    refract::lens!(WebizenRefractModel => 2: wellfair_review)
}

pub fn assertion_lens() -> impl Lens<WebizenRefractModel, Target = ContextualAssertionView> {
    refract::lens!(WebizenRefractModel => 3: assertion)
}

pub fn assertion_evidence_lens(
) -> impl Lens<WebizenRefractModel, Target = Option<AssertionEvidenceLevel>> {
    refract::lens!(WebizenRefractModel => 4: assertion_evidence)
}

pub fn contribution_lens() -> impl Lens<WebizenRefractModel, Target = ContributionStageView> {
    refract::lens!(WebizenRefractModel => 5: contribution)
}

/// Edit one draft payload word and update its derived parity in one notification.
/// Column five is derived. Out-of-range coordinates leave the draft unchanged.
pub fn update_refract_quin_word(
    store: &RefractStore<WebizenRefractModel>,
    slot_idx: usize,
    field_idx: u8,
    value: u64,
) {
    if slot_idx >= 64 || field_idx >= 5 {
        return;
    }
    store.write(quin_window_lens().slot(slot_idx), |slot| {
        let word = match field_idx {
            0 => &mut slot.subject,
            1 => &mut slot.predicate,
            2 => &mut slot.object,
            3 => &mut slot.context,
            _ => &mut slot.metadata,
        };
        *word = value;
        slot.recompute_parity();
    });
}

/// Refresh the six stewardship projections after a fabric mutation.
/// Semantic review drafts and other inspector slots stay in the same store.
pub fn refresh_refract_host_records(
    store: &RefractStore<WebizenRefractModel>,
    fabric: &NaturalHostStewardshipFabric,
) {
    for (offset, slot) in stewardship_slots(fabric).into_iter().enumerate() {
        let index = STEWARDSHIP_SLOT_START + offset;
        if store.read_quin_slot(quin_window_lens(), index) != slot {
            store.write(quin_window_lens().slot(index), |current| *current = slot);
        }
    }
}

fn stewardship_slots(fabric: &NaturalHostStewardshipFabric) -> [QuinSlot; 6] {
    let mut slots = [QuinSlot::new(0, 0, 0, 0, 0); 6];
    let context = q_hash_60(&fabric.primary_host_domain);
    for (slot, node) in slots[..2].iter_mut().zip(&fabric.host_fleet_nodes) {
        *slot = QuinSlot::new(
            q_hash_60(&node.node_id),
            q_hash_60("urn:webizen:inspector:host-runtime-locus"),
            q_hash_60(node.locus.code()),
            context,
            u64::from(node.standalone_without_online_vps),
        );
    }
    for (slot, project) in slots[2..4].iter_mut().zip(&fabric.hosted_projects) {
        *slot = QuinSlot::new(
            q_hash_60(&project.project_id),
            q_hash_60("urn:webizen:inspector:project-stewardship"),
            q_hash_60(project.ownership_stage.title()),
            context,
            u64::from(project.project_quorum_threshold_m),
        );
    }
    for (slot, circle) in slots[4..].iter_mut().zip(&fabric.shared_care_circles) {
        *slot = QuinSlot::new(
            q_hash_60(&circle.circle_id),
            q_hash_60("urn:webizen:inspector:care-circle"),
            q_hash_60(&circle.host_node_id),
            context,
            u64::from(circle.escalation_currently_active),
        );
    }
    slots
}

#[derive(Clone, Debug)]
pub struct ShellState {
    pub current_stage: VolumeStage,
    pub active_plane: IdentifierFabricPlane,
    pub settings_section: SettingsSection,
    pub settings_search: String,
    pub orb_open: bool,
    pub inspect_continuity_open: bool,
    pub stage_chrome_mode: StageChromeMode,
    pub orb_toast: String,
    pub theme_preset: &'static str,
    pub reduced_motion: bool,
    pub p2p_peers_count: usize,
    pub nym_mixnet_active: bool,
    pub spatial_mode: bool,
    pub launcher_open: bool,
    pub fabric_did: String,
    pub gpu_label: String,
    pub gpu_adapters: Vec<NativeGpuAdapterInfo>,
    pub native_vibe_providers: usize,
    pub nquin_parity_verified: bool,
    pub may_advertise_attested_vault: bool,
    pub sku_label: String,
    pub os_username: String,
    pub storage_display: String,
    pub daemon_label: String,
    pub host_link_label: String,
    pub validation_summary: String,
    pub identifier_lines: Vec<String>,
    pub installed_qaps: Vec<QappRegistryEntry>,
    pub active_launch_plan: Option<LaunchPlan>,
    pub llm_models: Vec<LocalLlmModelEntry>,
    pub selected_model_name: String,
    pub active_model_name: String,
    pub model_probe_receipt: Option<LocalModelReadinessReceipt>,
    pub poet_cells: Vec<PoetCellEntry>,
    pub poet_cell_input: String,
    pub poet_doc_input: String,
    pub poet_gazetteer: Option<PoetGazetteerResult>,
    pub poet_lexicon_status: String,
    pub hypermedia_script: String,
    pub hypermedia_receipt: Option<HypermediaProduceReceipt>,
    pub sanctum_issued: String,
    pub updater_status: String,
    pub delivery_status: String,
    pub talk_messages: Vec<String>,
    pub talk_input: String,
    pub talk_agent_tool_active: bool,
    pub mail_messages: Vec<String>,
    pub mail_to_locator: String,
    pub mail_subject: String,
    pub directory_contacts: Vec<String>,
    pub directory_new_name: String,
    pub browser_url: String,
    pub browser_mode: BrowserExecutionMode,
    pub browser_product: BrowserProductStatus,
    pub browser_bridge_receipts: Vec<HybridBridgeReceipt>,
    pub browser_history: Vec<String>,
    pub browser_social: BrowserSocialGraph,
    pub bookmark_title_input: String,
    pub bookmark_assertion_input: String,
    pub bookmark_entity_input: String,
    pub bookmark_group_target: String,
    pub annotation_fragment_input: String,
    pub annotation_kind_input: String,
    pub annotation_body_input: String,
    pub annotation_group_target: String,
    pub new_group_id_input: String,
    pub new_group_label_input: String,
    pub new_group_ontology_input: String,
    pub semantic_fs: SemanticFileSystem,
    pub library_filter: SemanticFacetFilter,
    pub selected_library_uri: String,
    pub ingest_uri: String,
    pub ingest_title: String,
    pub ingest_body: String,
    pub ingest_entity: String,
    pub ingest_sensitivity: String,
    pub host_stewardship: NaturalHostStewardshipFabric,
    pub mobile_avf: MobileAvfSanctuaryProfile,
    pub desktop_appliance: DesktopApplianceProfile,
    pub host_daemon_snapshot: HostDaemonStatusSnapshot,
    pub os_control_center: OsControlCenterState,
    pub os_control_center_error: Option<String>,
    pub os_job_runner_error: Option<String>,
    pub control_center_drawer_open: bool,
    pub chora_supervisor: ChoraWorkerSupervisorSnapshot,
    pub last_vibe_action_report: Option<GovernedVibeActionReport>,
    pub last_authoring_snapshot: Option<AuthoringWorkflowSnapshot>,
    pub last_qecp_envelope: Option<QecpCredentialEnvelope>,
    pub last_solid_receipt: Option<SolidTurtleSyncReceipt>,
    pub last_q42j_backup: Option<ShamirQ42jBackupArchive>,
    pub renderer_health: ShellRendererHealthSummary,
    pub sanctuary_onboarding: SanctuaryOnboardingState,
    pub mesh_simulation: Option<Result<ThreeNodeMeshSimulationReceipt, String>>,
    pub omni_intent_draft: String,
    pub omni_intent_status: String,
    pub staged_gguf_source: Option<(std::path::PathBuf, u64)>,
    pub aux_orb_window_open: bool,
    pub aux_alert_modal_open: bool,
    pub aux_palette_window_open: bool,
    pub viewport_widget_id: usize,
    pub active_holo_specimen_id: String,
    pub holo_shader_mode: Option<HoloShaderVisualizationMode>,
    pub holo_wireframe_animating: bool,
    pub holo_scale_k: u8,
    pub shell_viewport_size: (f64, f64),
    pub beforeinput_guard_notice: String,
    pub clipboard_toast: String,
    pub selection_range_label: String,
    pub host_proposal_deb_input: String,
    pub host_proposal_version_input: String,
    pub host_vote_steward_input: String,
    pub host_vote_comment_input: String,
    pub host_new_steward_did_input: String,
    pub host_new_steward_hint_input: String,
    pub host_new_steward_apparatus_input: String,
    pub host_stewardship_toast: String,
    pub llm_management: LlmAndAgentManagementRegistry,
    pub selected_agent_slug: String,
    pub agent_call_prompt_input: String,
    pub agent_call_volume_input: String,
    pub agent_call_sensitivity_input: u8,
    pub new_ext_account_label_input: String,
    pub new_ext_model_id_input: String,
    pub new_ext_mcp_url_input: String,
    pub new_ext_keyvault_ref_input: String,
    pub llm_management_toast: String,
}

impl Default for ShellState {
    fn default() -> Self {
        match session() {
            Some(s) => Self::from_session(s),
            None => Self::placeholder(),
        }
    }
}

impl ShellState {
    fn placeholder() -> Self {
        let host_stewardship = NaturalHostStewardshipFabric::default();
        let mut unresolved_fabric = host_stewardship.clone();
        unresolved_fabric.host_fleet_nodes.clear();
        unresolved_fabric.hosted_projects.clear();
        let unresolved_environment = OsEnvironment {
            hostname: String::new(),
            os: String::new(),
            arch: String::new(),
            username: String::new(),
            ram_bytes: None,
            cpu_count: None,
            cpu_brand: None,
        };
        Self {
            current_stage: VolumeStage::None,
            active_plane: IdentifierFabricPlane::NaturalAgent,
            settings_section: SettingsSection::Health,
            settings_search: String::new(),
            orb_open: false,
            inspect_continuity_open: false,
            stage_chrome_mode: StageChromeMode::DefaultDepth,
            orb_toast: String::new(),
            theme_preset: "theme-holosonic",
            reduced_motion: false,
            p2p_peers_count: 0,
            nym_mixnet_active: false,
            spatial_mode: true,
            launcher_open: false,
            fabric_did: String::new(),
            gpu_label: "Etalon Native GPU".into(),
            gpu_adapters: Vec::new(),
            native_vibe_providers: 11,
            nquin_parity_verified: true,
            may_advertise_attested_vault: false,
            sku_label: Sku::Guest.label().into(),
            os_username: "unknown".into(),
            storage_display: String::new(),
            daemon_label: "Planned — sidecar not probed".into(),
            host_link_label: "Unpaired".into(),
            validation_summary: String::new(),
            identifier_lines: Vec::new(),
            installed_qaps: Vec::new(),
            active_launch_plan: None,
            llm_models: Vec::new(),
            selected_model_name: String::new(),
            active_model_name: String::new(),
            model_probe_receipt: None,
            poet_cells: Vec::new(),
            poet_cell_input: "= 21 + 21".into(),
            poet_doc_input: String::new(),
            poet_gazetteer: None,
            poet_lexicon_status: String::new(),
            hypermedia_script: String::new(),
            hypermedia_receipt: None,
            sanctum_issued: String::new(),
            updater_status: String::new(),
            delivery_status: String::new(),
            talk_messages: Vec::new(),
            talk_input: String::new(),
            talk_agent_tool_active: false,
            mail_messages: Vec::new(),
            mail_to_locator: String::new(),
            mail_subject: String::new(),
            directory_contacts: Vec::new(),
            directory_new_name: String::new(),
            browser_url: "qualia://chora/universe".into(),
            browser_mode: BrowserExecutionMode::QualiaNative,
            browser_product: BrowserProductStatus {
                package_id: "Browser".into(),
                app_did: "did:qualia:qap:browser".into(),
                git_did: "did:git:github.com/WebizenAI/webizen-browser".into(),
                installed_version: "0.1.0".into(),
                latest_available_version: "0.1.1".into(),
                content_hash: "seeded".into(),
                package_dir: String::new(),
                worker_binary_slot: String::new(),
                independent_update_channel: "https://updates.webizen.org/qapps/browser/manifest.ed25519.json".into(),
                required_leases: Vec::new(),
                update_receipt_detail: "Bundled with OS-Shell; independent update channel ready".into(),
            },
            browser_bridge_receipts: Vec::new(),
            browser_history: Vec::new(),
            browser_social: BrowserSocialGraph::default(),
            bookmark_title_input: "Pinned Resource Bookmark (QLink)".into(),
            bookmark_assertion_input: "Contextual claim and Permalink reference for cooperative review.".into(),
            bookmark_entity_input: "solar-microgrid-sdf".into(),
            bookmark_group_target: "group:catchment-stewards".into(),
            annotation_fragment_input: "section-energy-storage".into(),
            annotation_kind_input: "ContextualClaim".into(),
            annotation_body_input: "Verified 60-cell battery bank topology against 10D v2 HOLO thermal envelope.".into(),
            annotation_group_target: "group:catchment-stewards".into(),
            new_group_id_input: "group:bioregional-energy-guild".into(),
            new_group_label_input: "Bioregional Energy & Microgrid Guild".into(),
            new_group_ontology_input: "urn:qualia:spc:relation:EnergyCommonsSteward".into(),
            semantic_fs: SemanticFileSystem::default(),
            library_filter: SemanticFacetFilter {
                section: "all".into(),
                ..Default::default()
            },
            selected_library_uri: String::new(),
            ingest_uri: "urn:qualia:asset:custom-10d-scene.10d".into(),
            ingest_title: "Custom 10D v2 HOLO Spatial Twin".into(),
            ingest_body: "Holographic 10D v2 SDF with eucalyptus canopy and solar microgrid telemetry.".into(),
            ingest_entity: "eucalyptus-canopy-sdf".into(),
            ingest_sensitivity: "public".into(),
            host_stewardship,
            mobile_avf: MobileAvfSanctuaryProfile::default(),
            desktop_appliance: build_desktop_appliance_profile(
                Sku::Guest, &unresolved_environment, &unresolved_fabric,
            ),
            host_daemon_snapshot: HostDaemonStatusSnapshot::default(),
            os_control_center: OsControlCenterState::default(),
            os_control_center_error: None,
            os_job_runner_error: None,
            control_center_drawer_open: false,
            chora_supervisor: ChoraWorkerSupervisorSnapshot::default(),
            last_vibe_action_report: None,
            last_authoring_snapshot: None,
            last_qecp_envelope: None,
            last_solid_receipt: None,
            last_q42j_backup: None,
            renderer_health: ShellRendererHealthSummary::default(),
            sanctuary_onboarding: SanctuaryOnboardingState::default(),
            mesh_simulation: None,
            omni_intent_draft: String::new(),
            omni_intent_status: String::new(),
            staged_gguf_source: None,
            aux_orb_window_open: false,
            aux_alert_modal_open: false,
            aux_palette_window_open: false,
            viewport_widget_id: 1,
            active_holo_specimen_id: String::new(),
            holo_shader_mode: None,
            holo_wireframe_animating: false,
            holo_scale_k: 2,
            shell_viewport_size: (0.0, 0.0),
            beforeinput_guard_notice: String::new(),
            clipboard_toast: String::new(),
            selection_range_label: String::new(),
            host_proposal_deb_input: "webizen-hostd_0.2.0_amd64.deb".into(),
            host_proposal_version_input: "0.2.0".into(),
            host_vote_steward_input: "did:qi:steward:mobile-avf-pocket".into(),
            host_vote_comment_input: "Verified NQuin DNS & QPolicy nftables from Android 16 AVF pKVM EL2 Pocket Sanctuary".into(),
            host_new_steward_did_input: "did:qi:steward:regional-coop-peer".into(),
            host_new_steward_hint_input: "Regional Cooperative Co-Steward (Living Person)".into(),
            host_new_steward_apparatus_input: "Mobile Debian 13 Trixie arm64 inside Android 16 AVF pKVM EL2 (webizen-mobile-avf.deb)".into(),
            host_stewardship_toast: String::new(),
            llm_management: LlmAndAgentManagementRegistry::default(),
            selected_agent_slug: "local-sanctuary-steward".into(),
            agent_call_prompt_input: "Analyse 10D v2 HOLO specimen & verify NQuin48 provenance for did:qi:alice".into(),
            agent_call_volume_input: "Poet".into(),
            agent_call_sensitivity_input: 0,
            new_ext_account_label_input: "Custom OpenRouter / Claude MCP Account".into(),
            new_ext_model_id_input: "anthropic/claude-4.6-sonnet".into(),
            new_ext_mcp_url_input: "https://openrouter.example/api/v1/mcp".into(),
            new_ext_keyvault_ref_input: "keyvault://sanctuary/external-llm/openrouter-work".into(),
            llm_management_toast: String::new(),
        }
    }

    pub fn from_session(s: &DeviceSession) -> Self {
        let daemon = probe_only(&s.host);
        let daemon_label = if daemon.reachable {
            format!("Live · Connected at {}", s.host.daemon_base)
        } else if daemon.binary.is_some() {
            format!("Live · Sidecar binary ready ({})", daemon.detail)
        } else {
            format!("Planned · Local sidecar ({})", daemon.detail)
        };

        let identifier_lines = s
            .identifiers
            .records
            .iter()
            .map(|r| format!("{} · {} · {}", r.plane, r.label, r.value))
            .collect();

        let active_launch_plan = s.host.launch("Anatomy", None).ok();

        let cell1 = poet_eval_native_cell("= 21 + 21", 0);
        let cell2 = poet_eval_native_cell("= 42 * 1024 * 1024", 1);
        let poet_cells = vec![
            PoetCellEntry {
                name: "answer_cell".into(),
                source: "= 21 + 21".into(),
                value: cell1.value,
                ok: cell1.ok,
                diagnostic: cell1.diagnostic,
                revision: cell1.revision,
            },
            PoetCellEntry {
                name: "sentinel_arena_bytes".into(),
                source: "= 42 * 1024 * 1024".into(),
                value: cell2.value,
                ok: cell2.ok,
                diagnostic: cell2.diagnostic,
                revision: cell2.revision,
            },
        ];

        let default_doc = "QualiaDB and VibeScript execute on Etalon native GPU to uphold human rights, human agency, guardianship, and Permissive Commons in Sanctuary and Poet.";
        let poet_gazetteer = Some(poet_analyze_and_seal_gazetteer(default_doc));
        let lex = poet_open_lexicon_pack(&s.host, "");
        let poet_lexicon_status = if lex.ok {
            lex.value
        } else {
            lex.diagnostic
                .unwrap_or_else(|| "Planned — lexicon pack".into())
        };

        let selected_model_name = s
            .llm_models
            .first()
            .map(|m| m.name.clone())
            .unwrap_or_default();
        let selected_library_uri = s
            .semantic_fs
            .entries
            .first()
            .map(|e| e.asset_uri.clone())
            .unwrap_or_default();

        let (os_control_center, os_control_center_error) =
            match OsControlCenterState::load_or_seed(&s.host.storage, &s.host_stewardship) {
                Ok(center) => (center, None),
                Err(error) => (OsControlCenterState::default(), Some(error)),
            };
        let mut shell = Self {
            current_stage: VolumeStage::None,
            active_plane: IdentifierFabricPlane::NaturalAgent,
            settings_section: SettingsSection::Health,
            settings_search: String::new(),
            orb_open: false,
            inspect_continuity_open: false,
            stage_chrome_mode: StageChromeMode::DefaultDepth,
            orb_toast: String::new(),
            theme_preset: "theme-holosonic",
            reduced_motion: false,
            p2p_peers_count: 0,
            nym_mixnet_active: false,
            spatial_mode: true,
            launcher_open: false,
            fabric_did: s.fabric_identifier().to_string(),
            gpu_label: s.gpu.summary_label.clone(),
            gpu_adapters: s.gpu.adapters.clone(),
            native_vibe_providers: s.gpu.native_vibe_provider_count,
            nquin_parity_verified: s.gpu.nquin_parity_verified,
            may_advertise_attested_vault: s.gpu.may_advertise_hardware_attested_vault,
            sku_label: s.sku.label().into(),
            os_username: s.os_username.clone(),
            storage_display: s.host.storage.display().to_string(),
            daemon_label,
            host_link_label: s.host_link.label().into(),
            validation_summary: s.identifiers.validation_summary.clone(),
            identifier_lines,
            installed_qaps: s.installed_qaps.clone(),
            active_launch_plan,
            llm_models: s.llm_models.clone(),
            selected_model_name: selected_model_name.clone(),
            active_model_name: selected_model_name,
            model_probe_receipt: None,
            poet_cells,
            poet_cell_input: "= 64 * 16".into(),
            poet_doc_input: default_doc.into(),
            poet_gazetteer,
            poet_lexicon_status,
            hypermedia_script: "= 150".into(),
            hypermedia_receipt: None,
            sanctum_issued: String::new(),
            updater_status: String::new(),
            delivery_status: String::new(),
            talk_messages: vec![
                "[NaturalAgent · Pairwise QDNF Channel] Ready for human-to-human messaging.".into(),
                "[Continuity Rule] Handle != human. Governed AI assistant sits on the Tool plane — never impersonates a peer.".into(),
            ],
            talk_input: String::new(),
            talk_agent_tool_active: false,
            mail_messages: vec![
                "Inbox [Pairwise Locator did:qi:mail:a->b] · Welcome to relationship-scoped mail (domain expiry never erases who).".into(),
                "Commons Notice [did:qi:mail:commons->steward] · 9-Stage Contribution Ledger & QECP PDF verification ready.".into(),
            ],
            mail_to_locator: "did:qi:locator:peer-steward".into(),
            mail_subject: "Cooperative project milestone & QECP receipt".into(),
            directory_contacts: vec![
                "NaturalAgent (living-SHACL) · Local Principal Steward · Pairwise DID bound".into(),
                "NaturalAgent (living-SHACL) · Cooperative Co-Steward · Bilateral Guardianship & Commons Peer".into(),
                "Organisational Peer (Separate Who-Kind) · Community Grounds Databox (dev.civics.au)".into(),
            ],
            directory_new_name: String::new(),
            browser_url: "qualia://chora/universe".into(),
            browser_mode: BrowserExecutionMode::QualiaNative,
            browser_product: inspect_browser_product(&s.host),
            browser_bridge_receipts: Vec::new(),
            browser_history: vec![
                "[Mode 2 · QualiaNative] qualia://chora/universe — Ring B Native 10D v2 HOLO Spatio-Temporal Commons Canvas".into(),
                "[Mode 3 · HybridAugmented] https://dev.civics.au — Ring C WWW Worker + Host-Authorized QualiaDB Capability Bridge (<qualia-10d>, Commons Receipts)".into(),
                "[Mode 3 · HybridAugmented] https://web.civics.au — Ring C WWW Worker + W3C Solid / LDP Databox Turtle <-> NQuin48 Bridge".into(),
                "[Mode 1 · TraditionalWeb] https://example.org — Ring C webizen-WWW-engine (MPL-2.0 sibling worker · PQ TLS X25519MLKEM768 · Zero Host Vault Access)".into(),
            ],
            browser_social: s.browser_social.clone(),
            bookmark_title_input: "Pinned Resource Bookmark (QLink)".into(),
            bookmark_assertion_input: "Contextual claim and Permalink reference for cooperative review.".into(),
            bookmark_entity_input: "solar-microgrid-sdf".into(),
            bookmark_group_target: "group:catchment-stewards".into(),
            annotation_fragment_input: "section-energy-storage".into(),
            annotation_kind_input: "ContextualClaim".into(),
            annotation_body_input: "Verified 60-cell battery bank topology against 10D v2 HOLO thermal envelope.".into(),
            annotation_group_target: "group:catchment-stewards".into(),
            new_group_id_input: "group:bioregional-energy-guild".into(),
            new_group_label_input: "Bioregional Energy & Microgrid Guild".into(),
            new_group_ontology_input: "urn:qualia:spc:relation:EnergyCommonsSteward".into(),
            semantic_fs: s.semantic_fs.clone(),
            library_filter: SemanticFacetFilter {
                section: "all".into(),
                ..Default::default()
            },
            selected_library_uri,
            ingest_uri: "urn:qualia:asset:field-survey-2026.hmc".into(),
            ingest_title: "Field Survey & 10D HOLO Specimen Bundle".into(),
            ingest_body: "Eucalyptus camaldulensis canopy specimen with solar microgrid sensor telemetry and C2PA QECP credential.".into(),
            ingest_entity: "field-specimen-holo-sdf".into(),
            ingest_sensitivity: "public".into(),
            host_stewardship: s.host_stewardship.clone(),
            mobile_avf: probe_mobile_avf_sanctuary(&s.gpu),
            desktop_appliance: build_desktop_appliance_profile(s.sku, &s.os_env, &s.host_stewardship),
            host_daemon_snapshot: HostDaemonStatusSnapshot::from_fabric(&s.host_stewardship),
            os_control_center,
            os_control_center_error,
            os_job_runner_error: None,
            control_center_drawer_open: false,
            chora_supervisor: ChoraWorkerSupervisorSnapshot::default(),
            last_vibe_action_report: None,
            last_authoring_snapshot: None,
            last_qecp_envelope: None,
            last_solid_receipt: None,
            last_q42j_backup: None,
            renderer_health: ShellRendererHealthSummary::default(),
            sanctuary_onboarding: build_sanctuary_onboarding_state(
                &s.identifiers, &s.host_stewardship, &s.gpu,
            ),
            mesh_simulation: None,
            omni_intent_draft: String::new(),
            omni_intent_status: String::new(),
            staged_gguf_source: None,
            aux_orb_window_open: false,
            aux_alert_modal_open: false,
            aux_palette_window_open: false,
            viewport_widget_id: 1,
            active_holo_specimen_id: String::new(),
            holo_shader_mode: None,
            holo_wireframe_animating: false,
            holo_scale_k: 2,
            shell_viewport_size: (0.0, 0.0),
            beforeinput_guard_notice: String::new(),
            clipboard_toast: String::new(),
            selection_range_label: String::new(),
            host_proposal_deb_input: "webizen-hostd_0.2.0_amd64.deb".into(),
            host_proposal_version_input: "0.2.0".into(),
            host_vote_steward_input: "did:qi:steward:mobile-avf-pocket".into(),
            host_vote_comment_input: "Verified NQuin DNS & QPolicy nftables from Android 16 AVF pKVM EL2 Pocket Sanctuary".into(),
            host_new_steward_did_input: "did:qi:steward:regional-coop-peer".into(),
            host_new_steward_hint_input: "Regional Cooperative Co-Steward (Living Person)".into(),
            host_new_steward_apparatus_input: "Mobile Debian 13 Trixie arm64 inside Android 16 AVF pKVM EL2 (webizen-mobile-avf.deb)".into(),
            host_stewardship_toast: String::new(),
            llm_management: s.llm_management.clone(),
            selected_agent_slug: "local-sanctuary-steward".into(),
            agent_call_prompt_input: "Analyse 10D v2 HOLO specimen & verify NQuin48 provenance for did:qi:alice".into(),
            agent_call_volume_input: "Poet".into(),
            agent_call_sensitivity_input: 0,
            new_ext_account_label_input: "Custom OpenRouter / Claude MCP Account".into(),
            new_ext_model_id_input: "anthropic/claude-4.6-sonnet".into(),
            new_ext_mcp_url_input: "https://openrouter.example/api/v1/mcp".into(),
            new_ext_keyvault_ref_input: "keyvault://sanctuary/external-llm/openrouter-work".into(),
            llm_management_toast: String::new(),
        };
        if let Some(snapshot) = load_workspace_snapshot(&s.host.storage) {
            shell.apply_workspace_snapshot(&snapshot);
            if shell.current_stage == VolumeStage::Poet {
                shell.active_launch_plan = s.host.launch("Poet", None).ok();
            }
        }
        shell
    }

    /// Persist only the volume and presentation preferences. Live authority and drafts stay in memory.
    pub fn to_workspace_snapshot(&self) -> PersistedWorkspaceSnapshot {
        PersistedWorkspaceSnapshot {
            active_stage_slug: self.current_stage.workspace_slug().into(),
            active_plane_code: self.active_plane.code().into(),
            stage_chrome_mode: match self.stage_chrome_mode {
                StageChromeMode::DefaultDepth => "default-depth",
                StageChromeMode::FullStage => "full-stage",
                StageChromeMode::FloatLayer => "float-layer",
            }
            .into(),
            theme_preset: self.theme_preset.into(),
            reduced_motion: self.reduced_motion,
            inspect_continuity_open: self.inspect_continuity_open,
            active_holo_specimen_id: self.active_holo_specimen_id.clone(),
            holo_scale_k: self.holo_scale_k.min(4),
            enabled_toolbox_slugs: self
                .semantic_fs
                .enabled_toolboxes
                .iter()
                .map(|id| id.spec().slug.to_owned())
                .collect(),
            browser_url: self.browser_url.clone(),
            browser_mode_code: self.browser_mode.code().into(),
        }
    }

    pub fn apply_workspace_snapshot(&mut self, snap: &PersistedWorkspaceSnapshot) {
        if let Some((stage, _)) = VolumeStage::WORKSPACE_SLUGS
            .iter()
            .find(|(_, slug)| *slug == snap.active_stage_slug)
        {
            self.current_stage = *stage;
        }
        self.active_plane = IdentifierFabricPlane::ALL
            .into_iter()
            .find(|plane| plane.code() == snap.active_plane_code)
            .unwrap_or_else(|| self.current_stage.primary_plane());
        self.stage_chrome_mode = match snap.stage_chrome_mode.as_str() {
            "full-stage" => StageChromeMode::FullStage,
            "float-layer" => StageChromeMode::FloatLayer,
            _ => StageChromeMode::DefaultDepth,
        };
        self.theme_preset = match snap.theme_preset.as_str() {
            "theme-obsidian" => "theme-obsidian",
            "theme-commons" => "theme-commons",
            _ => "theme-holosonic",
        };
        self.reduced_motion = snap.reduced_motion;
        self.inspect_continuity_open = snap.inspect_continuity_open;
        self.active_holo_specimen_id
            .clone_from(&snap.active_holo_specimen_id);
        self.holo_scale_k = snap.holo_scale_k.min(4);
        self.semantic_fs.enabled_toolboxes = SemanticToolboxId::ALL
            .into_iter()
            .filter(|id| {
                snap.enabled_toolbox_slugs
                    .iter()
                    .any(|slug| slug == id.spec().slug)
            })
            .collect();
        self.browser_url.clone_from(&snap.browser_url);
        self.browser_mode = BrowserExecutionMode::ALL
            .into_iter()
            .find(|mode| mode.code() == snap.browser_mode_code)
            .unwrap_or(BrowserExecutionMode::QualiaNative);
    }

    /// Read the active installation and latest persisted restrictive policy.
    pub fn blocks_live_qap_lease(&self, package_id: &str, scope: CapabilityLeaseScope) -> bool {
        self.os_control_center_error.is_some()
            || session().is_none_or(|session| {
                webizen_ua_os::require_qap_capability(&session.host.storage, package_id, scope)
                    .is_err()
            })
    }

    /// Rebuild the local fleet and daemon summaries from the latest stewardship records.
    pub fn refresh_host_profiles(&mut self) {
        sync_fleet_from_fabric(&mut self.desktop_appliance, &self.host_stewardship);
        self.host_daemon_snapshot = HostDaemonStatusSnapshot::from_fabric(&self.host_stewardship);
    }
}

impl VolumeStage {
    const WORKSPACE_SLUGS: [(Self, &'static str); 15] = [
        (Self::None, "none"),
        (Self::Talk, "talk"),
        (Self::Mail, "mail"),
        (Self::Directory, "directory"),
        (Self::Browser, "browser"),
        (Self::Keep, "keep"),
        (Self::Library, "library"),
        (Self::Instruments, "instruments"),
        (Self::Poet, "poet"),
        (Self::Wellfair, "wellfair"),
        (Self::Wallet, "wallet"),
        (Self::Projects, "projects"),
        (Self::Anatomy, "anatomy"),
        (Self::Settings, "settings"),
        (Self::Admin, "admin"),
    ];

    fn workspace_slug(self) -> &'static str {
        Self::WORKSPACE_SLUGS
            .into_iter()
            .find(|(stage, _)| *stage == self)
            .map(|(_, slug)| slug)
            .expect("every shell volume has a workspace slug")
    }

    pub fn title(self) -> &'static str {
        match self {
            VolumeStage::None => "No volume open",
            VolumeStage::Talk => "Talk",
            VolumeStage::Mail => "Mail",
            VolumeStage::Directory => "Directory",
            VolumeStage::Browser => "Browser",
            VolumeStage::Keep => "Keep · Sanctuary Vault",
            VolumeStage::Library => "Library",
            VolumeStage::Instruments => "Instruments",
            VolumeStage::Poet => "Poet & Synth Workbench",
            VolumeStage::Wellfair => "WellFair & Anatomy",
            VolumeStage::Wallet => "Wallet & Permissive Commons",
            VolumeStage::Projects => "Projects · Cooperative Work-Graph",
            VolumeStage::Anatomy => "Anatomy (Qap · 10D v2 HOLO)",
            VolumeStage::Settings => "Settings",
            VolumeStage::Admin => "Admin · Operations & Fleet",
        }
    }

    pub fn primary_plane(self) -> IdentifierFabricPlane {
        match self {
            VolumeStage::None | VolumeStage::Directory => IdentifierFabricPlane::NaturalAgent,
            VolumeStage::Talk
            | VolumeStage::Library
            | VolumeStage::Wellfair
            | VolumeStage::Projects
            | VolumeStage::Anatomy => IdentifierFabricPlane::ClaimOpinion,
            VolumeStage::Mail | VolumeStage::Browser => IdentifierFabricPlane::SpatiotemporalHandle,
            VolumeStage::Keep
            | VolumeStage::Instruments
            | VolumeStage::Poet
            | VolumeStage::Wallet
            | VolumeStage::Settings
            | VolumeStage::Admin => IdentifierFabricPlane::Instrument,
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            VolumeStage::None => "",
            VolumeStage::Talk => "Human to human over QDNF. Chatbot is a governed tool on the ribbon — never the peer.",
            VolumeStage::Mail => "Relationship-scoped directional pairwise locators (A->B != B->A). Domain death never erases who.",
            VolumeStage::Directory => "People & Living Beings first (NaturalAgent / living-SHACL); organisations and tools stay distinct.",
            VolumeStage::Browser => "Dual-web reach: Ring B qualia://chora/universe + Ring C webizen-WWW-engine (MPL-2.0 sibling worker).",
            VolumeStage::Keep => {
                "Constant-shape 4-layer VaultContainerV2 (Real/Decoy/Reserved), Argon2id + AES-256-GCM, Audit DAG, and Anvil NP64 gate."
            }
            VolumeStage::Library => "Unified .q42, .10d v2 HOLO, .hmc, .m64, .emf, .qac, and QECP C2PA PDF lived-memory keep.",
            VolumeStage::Instruments => "6 Core Semantic Instruments (SI-01..12), 40+ formal logic systems, 6 SHACL families, and local AI models.",
            VolumeStage::Poet => {
                "Unified Native Poet & 150-Morphism Hypermedia Production (Synth) Workbench on Etalon Native Desktop."
            }
            VolumeStage::Wellfair => "Fiduciary health, 3D anatomy & maternal-fetal dyad, relational guardianship (UN CRC/CRPD), and 6 Vectors.",
            VolumeStage::Wallet => "Lightning (sats) · Nym Mixnet · eCash (XEC) · Tokens · Two-Loop Permissive Commons (no ETH).",
            VolumeStage::Projects => "Cooperative work-graph, add-wins effort/capital merge, and 9-stage contribution-to-licence lifecycle.",
            VolumeStage::Anatomy => "Installed Qap via install_package_atomic + LaunchPlan (NativeRsxCard · 10D v2 HOLO medical atlas).",
            VolumeStage::Settings => {
                "10-section environment setup, Four-Plane Identifier Fabric, local AI instruments, Sanctuary, and Dual-SKU delivery."
            }
            VolumeStage::Admin => "Local QualiaDB :4242 sidecar, 42 MiB SlgArena Sentinel, background jobs, logs, and Dual-SKU packaging.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_restores_only_preferences_and_leaves_live_state_untouched() {
        let mut original = ShellState::placeholder();
        original.current_stage = VolumeStage::Library;
        original.active_plane = IdentifierFabricPlane::ClaimOpinion;
        original.stage_chrome_mode = StageChromeMode::FloatLayer;
        original.theme_preset = "theme-commons";
        original.reduced_motion = true;
        original.active_holo_specimen_id = "qualia://specimens/example".into();
        original.holo_scale_k = 3;
        original.browser_url = "https://example.org/".into();
        original.browser_mode = BrowserExecutionMode::HybridAugmented;
        original.control_center_drawer_open = true;
        let snapshot = original.to_workspace_snapshot();
        let mut reopened = ShellState::placeholder();
        reopened.apply_workspace_snapshot(&snapshot);
        assert_eq!(reopened.to_workspace_snapshot(), snapshot);
        assert!(!reopened.control_center_drawer_open);
    }

    #[test]
    fn editing_a_quin_word_reseals_only_its_slot() {
        let store = RefractStore::new(WebizenRefractModel::placeholder());
        let neighbor = store.read_quin_slot(quin_window_lens(), 11);
        update_refract_quin_word(&store, 10, 2, 37);
        update_refract_quin_word(&store, 10, 5, 0);
        update_refract_quin_word(&store, 64, 0, 37);

        let edited = store.read_quin_slot(quin_window_lens(), 10);
        assert_eq!(edited.object, 37);
        assert!(edited.verify_parity());
        assert_eq!(store.read_quin_slot(quin_window_lens(), 11), neighbor);
    }

    #[test]
    fn refreshing_host_slots_preserves_instrument_and_contribution_drafts() {
        let store = RefractStore::new(WebizenRefractModel::placeholder());
        let original = store.read_quin_slot(quin_window_lens(), 10);
        store.write(contribution_lens(), |draft| draft.pool_funded_cents = 50);
        let fabric = NaturalHostStewardshipFabric::default();
        refresh_refract_host_records(&store, &fabric);

        let window = store.get_cloned(quin_window_lens());
        assert_eq!(window.active_len(), SEEDED_QUIN_SLOTS);
        assert!(window.verify_all_parity());
        assert_ne!(window.slots[STEWARDSHIP_SLOT_START].subject, 0);
        assert_eq!(window.slots[10], original);
        let review = store.get_cloned(contribution_lens());
        assert_eq!(review.pool_funded_cents, 50);
        assert_eq!(review.contributor_settled_cents, 0);
        assert!(!review.human_transition_approved);
    }
}
