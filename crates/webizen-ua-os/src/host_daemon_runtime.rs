// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Headless host stewardship and the bounded public-model bridge.

use crate::{
    q_hash_60, validate_human_centric_host_nomenclature, ActorJuralPersonalityKind,
    CareAlertSeverity, CareAlertTriggerKind, DeviceOs, HostedProjectOwnershipStage,
    LlmAndAgentManagementRegistry, NaturalHostStewardshipFabric, SharedLivingAndCareTopologyKind,
    Sku, HOST_STEWARDSHIP_STORE_REL_PATH, LLM_MANAGEMENT_STORE_REL_PATH, P64_MAGIC_HEADER,
    SENSITIVITY_PUBLIC_ONLY,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha3::{Digest, Sha3_256};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read, Seek, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use vibescript_abi::NQuin48;

const MAX_LEDGER_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PEER_LINE: usize = 16 * 1024;
const MAX_HTTP_HEADER: usize = 8 * 1024;
const MAX_PEER_REQUESTS: usize = 32;
const MAX_PROMPT_BYTES: usize = 8 * 1024;
const MAX_GENERATED_TOKENS: u32 = 256;
const PEER_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const HEARTBEAT_FILE: &str = "wellfair/hostd-care-heartbeats.json";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HostDaemonAuthMode {
    #[default]
    Loopback,
    Bearer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDaemonConfig {
    pub storage: PathBuf,
    pub bind: SocketAddr,
    pub mcp_bind: SocketAddr,
    pub auth_mode: HostDaemonAuthMode,
    pub json: bool,
}

impl HostDaemonConfig {
    pub fn for_storage(storage: impl AsRef<Path>) -> Self {
        let loopback = IpAddr::V4(Ipv4Addr::LOCALHOST);
        Self {
            storage: storage.as_ref().to_path_buf(),
            bind: SocketAddr::new(loopback, 8080),
            mcp_bind: SocketAddr::new(loopback, 4243),
            auth_mode: HostDaemonAuthMode::Loopback,
            json: false,
        }
    }
}

impl Default for HostDaemonConfig {
    fn default() -> Self {
        let storage = match Sku::resolve() {
            Sku::Guest => crate::guest_storage_dir(DeviceOs::detect()),
            Sku::Appliance => std::env::var_os("WEBIZEN_STORAGE")
                .filter(|path| !path.is_empty())
                .map_or_else(|| PathBuf::from("/var/lib/qualiadb"), PathBuf::from),
        };
        Self::for_storage(storage)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostDaemonStatusSnapshot {
    pub host_domain: String,
    pub quorum_threshold: u8,
    pub steward_count: usize,
    pub queued_updates: usize,
    pub hosted_projects: usize,
    pub fleet_nodes: usize,
    pub care_circles: usize,
    pub active_care_alerts: usize,
    /// A fabric can be shown in the shell before any ledger is persisted.
    pub stewardship_persisted: bool,
}

impl HostDaemonStatusSnapshot {
    pub fn from_fabric(fabric: &NaturalHostStewardshipFabric) -> Self {
        Self {
            host_domain: fabric.primary_host_domain.clone(),
            quorum_threshold: fabric.threshold_m,
            steward_count: fabric.stewards.len(),
            queued_updates: fabric.update_queue.len(),
            hosted_projects: fabric.hosted_projects.len(),
            fleet_nodes: fabric.host_fleet_nodes.len(),
            care_circles: fabric.shared_care_circles.len(),
            active_care_alerts: fabric
                .care_alerts
                .iter()
                .filter(|alert| !alert.resolved_or_reversed)
                .count(),
            stewardship_persisted: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CareHeartbeatStatus {
    NoObservation,
    WithinGrace,
    Lapsed,
    ClockUncertain,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CareHeartbeatReport {
    pub circle_id: String,
    pub status: CareHeartbeatStatus,
    pub elapsed_secs: Option<u64>,
    pub grace_secs: u64,
    pub escalation_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostDaemonTickReport {
    pub status: HostDaemonStatusSnapshot,
    pub observed_at_unix_secs: u64,
    pub care_heartbeats: Vec<CareHeartbeatReport>,
    pub queued_updates_requiring_review: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCareAcknowledgement {
    pub alert_id: String,
    pub responder_did: String,
    pub principal_ok: bool,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostDaemonCommand {
    Help,
    Status,
    RunTick,
    ProposeUpdate {
        package: String,
        version: String,
        proposer_did: String,
        instrument: bool,
    },
    VoteQuorum {
        proposal_id: String,
        steward_did: String,
        approve: bool,
        comment: String,
        guardian: Option<String>,
    },
    RegisterProject {
        project_id: String,
        title: String,
        domain: String,
        stage: HostedProjectOwnershipStage,
        steward_did: String,
        entity: Option<String>,
    },
    TransitionQcdn {
        project_id: String,
        stage: HostedProjectOwnershipStage,
        threshold: u8,
        stewards: Vec<String>,
        mirrors: Vec<String>,
    },
    AuditJuralLicense {
        actor: ActorJuralPersonalityKind,
        entity: Option<String>,
    },
    PollCareHeartbeats {
        circle: Option<String>,
        dispatch_lapse: bool,
        acknowledgement: Option<HostCareAcknowledgement>,
    },
    ServePeerP64Mcp {
        once: bool,
    },
    ServeHttpControl {
        once: bool,
    },
}

pub const HOSTD_USAGE: &str = "Usage: webizen-hostd [--storage DIR] [--bind ADDR] [--mcp-bind ADDR] \
[--auth-mode loopback|bearer] [--json] COMMAND
  status
  run-tick
  propose-update PACKAGE VERSION PROPOSER_DID [--instrument]
  vote-quorum PROPOSAL STEWARD_DID approve|dissent COMMENT [--guardian DID]
  register-project ID TITLE DOMAIN personal|group|mutual-aid|commercial STEWARD_DID [--entity ID]
  transition-qcdn ID personal|group|mutual-aid|commercial THRESHOLD [--add-steward DID] [--add-mirror ID]
  audit-jural-license natural|agent-of-entity|incorporated [--entity ID]
  poll-care-heartbeats [--circle ID] [--dispatch-lapse] [--ack ALERT RESPONDER_DID principal-ok|carer-ack NOTE]
  serve-peer-p64-mcp [--once]
  serve-http-control [--once]

Peer inference uses a dedicated adapter at WEBIZEN_PUBLIC_P64_MCP_UPSTREAM.
Without an upstream, Linux hostd can use an isolated executable configured by
WEBIZEN_PUBLIC_P64_RUNNER, WEBIZEN_PUBLIC_P64_MODEL and WEBIZEN_PUBLIC_P64_SHA3_256.
";

fn required<'a>(args: &mut impl Iterator<Item = &'a str>, label: &str) -> Result<&'a str, String> {
    args.next()
        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
        .ok_or_else(|| format!("Missing {label}"))
}

fn ownership_stage(name: &str) -> Result<HostedProjectOwnershipStage, String> {
    use HostedProjectOwnershipStage::*;
    match name {
        "personal" => Ok(PersonalOwnerProject),
        "group" => Ok(CooperativeGroupProject),
        "mutual-aid" => Ok(SupportedThirdPartyOrGroupProject),
        "commercial" => Ok(IncorporatedCommercialTenantProject),
        _ => Err("Ownership stage must be personal, group, mutual-aid, or commercial".into()),
    }
}

fn command_options<'a>(
    args: impl Iterator<Item = &'a str>,
    choices: &[(&'static str, usize, bool)],
) -> Result<Vec<(&'a str, Vec<String>)>, String> {
    let mut args = args.peekable();
    let mut options: Vec<(&str, Vec<String>)> = Vec::new();
    while let Some(option) = args.next() {
        let (_, count, repeatable) = choices
            .iter()
            .find(|choice| choice.0 == option)
            .ok_or_else(|| format!("Unexpected argument: {option}"))?;
        if !*repeatable && options.iter().any(|(key, _)| *key == option) {
            return Err(format!("Argument specified more than once: {option}"));
        }
        let values = (0..*count)
            .map(|_| required(&mut args, option).map(str::to_owned))
            .collect::<Result<_, _>>()?;
        options.push((option, values));
    }
    Ok(options)
}

fn one_option(options: &[(&str, Vec<String>)], name: &str) -> Option<String> {
    options
        .iter()
        .find_map(|(key, items)| (*key == name).then(|| items.first()).flatten().cloned())
}

/// Parse arguments excluding the executable name.
pub fn parse_hostd_args(args: &[String]) -> Result<(HostDaemonConfig, HostDaemonCommand), String> {
    let mut cfg = HostDaemonConfig::default();
    let mut args = args.iter().map(String::as_str);
    let mut explicit_bind = false;
    let mut local = Vec::new();
    while let Some(arg) = args.next() {
        match arg {
            "--storage" => cfg.storage = PathBuf::from(required(&mut args, "storage path")?),
            "--bind" => {
                explicit_bind = true;
                cfg.bind = required(&mut args, "HTTP bind address")?
                    .parse()
                    .map_err(|_| "Invalid HTTP bind address")?;
            }
            "--mcp-bind" => {
                cfg.mcp_bind = required(&mut args, "MCP bind address")?
                    .parse()
                    .map_err(|_| "Invalid MCP bind address")?;
            }
            "--auth-mode" => {
                cfg.auth_mode = match required(&mut args, "auth mode")? {
                    "loopback" => HostDaemonAuthMode::Loopback,
                    "bearer" => HostDaemonAuthMode::Bearer,
                    _ => return Err("Auth mode must be loopback or bearer".into()),
                };
            }
            "--json" => cfg.json = true,
            _ => local.push(arg),
        }
    }

    let mut args = local.into_iter();
    use HostDaemonCommand::*;
    let command = match args.next() {
        None if explicit_bind => ServeHttpControl { once: false },
        None => Status,
        Some("help" | "--help" | "-h") => Help,
        Some("status") => {
            command_options(args, &[])?;
            Status
        }
        Some("run-tick") => {
            command_options(args, &[])?;
            RunTick
        }
        Some("propose-update") => {
            let package = required(&mut args, "Debian package")?.to_owned();
            let version = required(&mut args, "version")?.to_owned();
            let proposer_did = required(&mut args, "proposer DID")?.to_owned();
            let options = command_options(args, &[("--instrument", 0, false)])?;
            ProposeUpdate {
                package,
                version,
                proposer_did,
                instrument: !options.is_empty(),
            }
        }
        Some("vote-quorum") => {
            let proposal_id = required(&mut args, "proposal ID")?.to_owned();
            let steward_did = required(&mut args, "steward DID")?.to_owned();
            let approve = match required(&mut args, "quorum decision")? {
                "approve" => true,
                "dissent" => false,
                _ => return Err("Quorum decision must be approve or dissent".into()),
            };
            let comment = required(&mut args, "comment")?.to_owned();
            let options = command_options(args, &[("--guardian", 1, false)])?;
            VoteQuorum {
                proposal_id,
                steward_did,
                approve,
                comment,
                guardian: one_option(&options, "--guardian"),
            }
        }
        Some("register-project") => {
            let project_id = required(&mut args, "project ID")?.to_owned();
            let title = required(&mut args, "title")?.to_owned();
            let domain = required(&mut args, "domain")?.to_owned();
            let stage = ownership_stage(required(&mut args, "ownership stage")?)?;
            let steward_did = required(&mut args, "steward DID")?.to_owned();
            let options = command_options(args, &[("--entity", 1, false)])?;
            RegisterProject {
                project_id,
                title,
                domain,
                stage,
                steward_did,
                entity: one_option(&options, "--entity"),
            }
        }
        Some("transition-qcdn") => {
            let project_id = required(&mut args, "project ID")?.to_owned();
            let stage = ownership_stage(required(&mut args, "ownership stage")?)?;
            let threshold = required(&mut args, "quorum threshold")?
                .parse::<u8>()
                .ok()
                .filter(|threshold| *threshold > 0)
                .ok_or("Quorum threshold must be in 1..=255")?;
            let options = command_options(
                args,
                &[("--add-steward", 1, true), ("--add-mirror", 1, true)],
            )?;
            let mut stewards = Vec::new();
            let mut mirrors = Vec::new();
            for (option, values) in options {
                if option == "--add-steward" {
                    stewards.extend(values);
                } else {
                    mirrors.extend(values);
                }
            }
            TransitionQcdn {
                project_id,
                stage,
                threshold,
                stewards,
                mirrors,
            }
        }
        Some("audit-jural-license") => {
            use ActorJuralPersonalityKind::*;
            let actor = match required(&mut args, "actor kind")? {
                "natural" => NaturalPersonPrincipal,
                "agent-of-entity" => NaturalPersonAgentOfIncorporatedEntity,
                "incorporated" => IncorporatedLegalEntity,
                _ => {
                    return Err(
                        "Actor kind must be natural, agent-of-entity, or incorporated".into(),
                    )
                }
            };
            let options = command_options(args, &[("--entity", 1, false)])?;
            AuditJuralLicense {
                actor,
                entity: one_option(&options, "--entity"),
            }
        }
        Some("poll-care-heartbeats") => {
            let options = command_options(
                args,
                &[
                    ("--circle", 1, false),
                    ("--dispatch-lapse", 0, false),
                    ("--ack", 4, false),
                ],
            )?;
            let acknowledgement = options
                .iter()
                .find(|(key, _)| *key == "--ack")
                .map(|(_, items)| {
                    let principal_ok = match items[2].as_str() {
                        "principal-ok" => true,
                        "carer-ack" => false,
                        _ => return Err("Acknowledgement mode must be principal-ok or carer-ack"),
                    };
                    Ok(HostCareAcknowledgement {
                        alert_id: items[0].clone(),
                        responder_did: items[1].clone(),
                        principal_ok,
                        note: items[3].clone(),
                    })
                })
                .transpose()?;
            PollCareHeartbeats {
                circle: one_option(&options, "--circle"),
                dispatch_lapse: options.iter().any(|(key, _)| *key == "--dispatch-lapse"),
                acknowledgement,
            }
        }
        Some(command @ ("serve-peer-p64-mcp" | "serve-http-control")) => {
            let once = !command_options(args, &[("--once", 0, false)])?.is_empty();
            if command == "serve-peer-p64-mcp" {
                ServePeerP64Mcp { once }
            } else {
                ServeHttpControl { once }
            }
        }
        Some(command) => return Err(format!("Unknown hostd command: {command}")),
    };
    Ok((cfg, command))
}

fn read_ledger<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("Open {}: {error}", path.display())),
    };
    let mut bytes = Vec::new();
    file.take(MAX_LEDGER_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Read {}: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_LEDGER_BYTES {
        return Err(format!("Ledger exceeds size limit: {}", path.display()));
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|error| format!("Invalid ledger {}: {error}", path.display()))
}

fn load_fabric(cfg: &HostDaemonConfig) -> Result<(NaturalHostStewardshipFabric, bool), String> {
    let path = cfg.storage.join(HOST_STEWARDSHIP_STORE_REL_PATH);
    let saved = read_ledger::<NaturalHostStewardshipFabric>(&path)?;
    match saved {
        Some(fabric)
            if fabric.primary_host_domain.is_empty()
                || fabric.stewards.is_empty()
                || fabric.threshold_m == 0 =>
        {
            Err("Stewardship ledger is missing the host or quorum configuration".into())
        }
        Some(fabric) => Ok((fabric, true)),
        None => Ok((NaturalHostStewardshipFabric::default(), false)),
    }
}

// The local care owner supplies observations. Missing telemetry never implies a lapse.
// File schema: array of { circle_id, principal_did, last_checkin_unix_secs }.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeartbeatObservation {
    circle_id: String,
    principal_did: String,
    last_checkin_unix_secs: u64,
}

fn observe_heartbeats(
    observations: &[HeartbeatObservation],
    fabric: &NaturalHostStewardshipFabric,
    now: u64,
) -> Vec<CareHeartbeatReport> {
    fabric
        .shared_care_circles
        .iter()
        .filter(|circle| {
            circle.topology
                == SharedLivingAndCareTopologyKind::ElderlyOrSupportedPersonWithRemoteCarers
                && circle.heartbeat_grace_secs != 0
        })
        .map(|circle| {
            let latest = observations
                .iter()
                .filter(|entry| {
                    entry.circle_id == circle.circle_id
                        && entry.principal_did == circle.principal_did
                })
                .map(|entry| entry.last_checkin_unix_secs)
                .chain(circle.last_principal_checkin_unix_secs)
                .max();
            let elapsed_secs = latest.and_then(|observed| now.checked_sub(observed));
            let status = match (latest, elapsed_secs) {
                (None, _) => CareHeartbeatStatus::NoObservation,
                (Some(_), None) => CareHeartbeatStatus::ClockUncertain,
                (_, Some(age)) if age >= circle.heartbeat_grace_secs => CareHeartbeatStatus::Lapsed,
                _ => CareHeartbeatStatus::WithinGrace,
            };
            CareHeartbeatReport {
                circle_id: circle.circle_id.clone(),
                status,
                elapsed_secs,
                grace_secs: circle.heartbeat_grace_secs,
                escalation_active: circle.escalation_currently_active,
            }
        })
        .collect()
}

fn load_heartbeat_observations(
    cfg: &HostDaemonConfig,
) -> Result<Vec<HeartbeatObservation>, String> {
    Ok(read_ledger(&cfg.storage.join(HEARTBEAT_FILE))?.unwrap_or_default())
}

fn tick_snapshot(
    observations: &[HeartbeatObservation],
    fabric: &NaturalHostStewardshipFabric,
    persisted: bool,
    now: u64,
) -> HostDaemonTickReport {
    let mut status = HostDaemonStatusSnapshot::from_fabric(fabric);
    status.stewardship_persisted = persisted;
    HostDaemonTickReport {
        status,
        observed_at_unix_secs: now,
        care_heartbeats: observe_heartbeats(observations, fabric, now),
        queued_updates_requiring_review: fabric
            .update_queue
            .iter()
            .filter(|update| update.paraconsistent_review_flagged)
            .count(),
    }
}

fn unix_seconds() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "Host clock is earlier than the Unix epoch")?
        .as_secs())
}

pub fn run_daemon_tick(cfg: &HostDaemonConfig) -> Result<HostDaemonTickReport, String> {
    let (fabric, persisted) = load_fabric(cfg)?;
    let observations = load_heartbeat_observations(cfg)?;
    Ok(tick_snapshot(
        &observations,
        &fabric,
        persisted,
        unix_seconds()?,
    ))
}

fn poll_care_heartbeats(
    cfg: &HostDaemonConfig,
    mut fabric: NaturalHostStewardshipFabric,
    persisted: bool,
    circle: Option<&str>,
    dispatch_lapse: bool,
    ack: Option<HostCareAcknowledgement>,
) -> Result<Value, String> {
    if (dispatch_lapse || ack.is_some()) && !persisted {
        return Err("Care actions require an existing stewardship ledger".into());
    }
    if let Some(id) = circle {
        if !fabric
            .shared_care_circles
            .iter()
            .any(|entry| entry.circle_id == id)
        {
            return Err("Care circle is not registered".into());
        }
    }
    // Read and validate all observations before recording any requested action.
    let observations = load_heartbeat_observations(cfg)?;
    let now = unix_seconds()?;
    let mut tick = tick_snapshot(&observations, &fabric, persisted, now);
    tick.care_heartbeats
        .retain(|report| circle.is_none_or(|id| id == report.circle_id));
    let mut acknowledged = None;
    if let Some(ack) = ack {
        let alert = fabric
            .care_alerts
            .iter()
            .find(|alert| alert.alert_id == ack.alert_id && !alert.resolved_or_reversed)
            .ok_or("Active care alert was not found")?;
        if circle.is_some_and(|id| id != alert.circle_id) {
            return Err("Care alert does not belong to the selected circle".into());
        }
        let care_circle = fabric
            .shared_care_circles
            .iter()
            .find(|entry| entry.circle_id == alert.circle_id)
            .ok_or("Care alert has no registered circle")?;
        let authorized = if ack.principal_ok {
            alert.principal_did == ack.responder_did
                && care_circle.principal_did == ack.responder_did
        } else {
            care_circle.topology != SharedLivingAndCareTopologyKind::ShareHousePeerCoLiving
                && ack.responder_did != care_circle.principal_did
                && care_circle.participants.iter().any(|participant| {
                    participant.participant_did == ack.responder_did
                        && !participant.consent_credential_id.is_empty()
                        && alert.severity.level() >= participant.alert_min_severity_level
                        && alert.notified_remote_guardian_dids.iter().any(|notified| {
                            notified
                                .strip_prefix(ack.responder_did.as_str())
                                .is_some_and(|tail| tail.is_empty() || tail.starts_with(" ("))
                        })
                })
        };
        if !authorized {
            return Err("Responder is not admitted for this care alert".into());
        }
        acknowledged = Some(fabric.acknowledge_or_reverse_care_alert(
            &cfg.storage,
            &ack.alert_id,
            &ack.responder_did,
            ack.principal_ok,
            &ack.note,
        )?);
    }
    let mut alerts = Vec::new();
    if dispatch_lapse {
        for report in &tick.care_heartbeats {
            if report.status != CareHeartbeatStatus::Lapsed
                || report.escalation_active
                || acknowledged
                    .as_ref()
                    .is_some_and(|alert| alert.circle_id == report.circle_id)
            {
                continue;
            }
            let Some(care_circle) = fabric
                .shared_care_circles
                .iter()
                .find(|entry| entry.circle_id == report.circle_id)
            else {
                continue;
            };
            let severity = CareAlertSeverity::Concern;
            if care_circle.min_alert_severity_level > severity.level() {
                continue;
            }
            alerts.push(fabric.dispatch_shared_care_or_guardian_alert(
                &cfg.storage,
                &report.circle_id,
                CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse,
                severity,
                "The observed heartbeat exceeded the configured grace period. A care inquiry is pending.",
            )?);
        }
    }
    if acknowledged.is_some() || !alerts.is_empty() {
        let observed_at = now.max(unix_seconds().unwrap_or(now));
        tick = tick_snapshot(&observations, &fabric, true, observed_at);
        tick.care_heartbeats
            .retain(|report| circle.is_none_or(|id| id == report.circle_id));
    }
    Ok(json!({ "tick": tick, "acknowledged": acknowledged, "dispatched": alerts }))
}

pub fn execute_hostd_command(
    cfg: &HostDaemonConfig,
    command: HostDaemonCommand,
) -> Result<Value, String> {
    fn encode(value: impl Serialize) -> Result<Value, String> {
        serde_json::to_value(value).map_err(|error| error.to_string())
    }

    use HostDaemonCommand::*;
    let command = match command {
        Help => return Ok(json!({ "usage": HOSTD_USAGE })),
        RunTick => return encode(run_daemon_tick(cfg)?),
        ServePeerP64Mcp { once } => {
            serve_peer_p64_mcp_tcp(cfg, once)?;
            return Ok(json!({ "service": "peer-p64-mcp", "status": "stopped" }));
        }
        ServeHttpControl { once } => {
            serve_hostd_http_control(cfg, once)?;
            return Ok(json!({ "service": "hostd-http-control", "status": "stopped" }));
        }
        command => command,
    };

    let (mut fabric, persisted) = load_fabric(cfg)?;
    if !persisted
        && matches!(
            &command,
            ProposeUpdate { .. }
                | VoteQuorum { .. }
                | RegisterProject { .. }
                | TransitionQcdn { .. }
                | AuditJuralLicense {
                    actor: ActorJuralPersonalityKind::NaturalPersonAgentOfIncorporatedEntity
                        | ActorJuralPersonalityKind::IncorporatedLegalEntity,
                    ..
                }
        )
    {
        return Err("This command requires an existing stewardship ledger".into());
    }
    match command {
        Status => {
            let mut snapshot = HostDaemonStatusSnapshot::from_fabric(&fabric);
            snapshot.stewardship_persisted = persisted;
            encode(snapshot)
        }
        ProposeUpdate {
            package,
            version,
            proposer_did,
            instrument,
        } => {
            validate_human_centric_host_nomenclature(&version)?;
            encode(fabric.propose_host_update(
                &cfg.storage,
                &package,
                &version,
                &proposer_did,
                instrument,
            )?)
        }
        VoteQuorum {
            proposal_id,
            steward_did,
            approve,
            comment,
            guardian,
        } => encode(fabric.sign_or_dissent_host_update(
            &cfg.storage,
            &proposal_id,
            &steward_did,
            guardian.as_deref(),
            approve,
            &comment,
        )?),
        RegisterProject {
            project_id,
            title,
            domain,
            stage,
            steward_did,
            entity,
        } => encode(fabric.register_hosted_project(
            &cfg.storage,
            &project_id,
            &title,
            &domain,
            stage,
            &steward_did,
            entity.as_deref(),
        )?),
        TransitionQcdn {
            project_id,
            stage,
            threshold,
            stewards,
            mirrors,
        } => encode(fabric.transition_project_to_group_or_qcdn_swarm(
            &cfg.storage,
            &project_id,
            stage,
            stewards,
            threshold,
            mirrors,
        )?),
        AuditJuralLicense { actor, entity } => {
            if actor != ActorJuralPersonalityKind::NaturalPersonPrincipal
                && !entity.as_ref().is_some_and(|id| {
                    fabric
                        .incorporated_entities
                        .iter()
                        .any(|entry| &entry.entity_did == id)
                })
            {
                return Err("An explicit registered incorporated entity is required".into());
            }
            let (required, monthly_fee_cents, detail) =
                fabric.evaluate_jural_actor_commercial_obligation(actor, entity.as_deref())?;
            Ok(
                json!({ "commercial_licence_required": required, "monthly_fee_cents": monthly_fee_cents, "detail": detail }),
            )
        }
        PollCareHeartbeats {
            circle,
            dispatch_lapse,
            acknowledgement,
        } => poll_care_heartbeats(
            cfg,
            fabric,
            persisted,
            circle.as_deref(),
            dispatch_lapse,
            acknowledgement,
        ),
        _ => unreachable!("Service commands are handled before opening the stewardship ledger"),
    }
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicInferenceArguments {
    prompt: String,
    sensitivity: u8,
    #[serde(default = "default_peer_token_cap")]
    max_tokens: u32,
}

fn default_peer_token_cap() -> u32 {
    128
}

/// A pure JSON-RPC dispatcher. The caller supplies a public-only inference
/// adapter that can access only the designated public model and this prompt.
pub fn handle_peer_p64_mcp_json_rpc_line(
    line: &str,
    infer_public: impl FnOnce(&str, u32) -> Result<String, String>,
) -> Option<Value> {
    dispatch_public_inference(line, |prompt, cap| {
        infer_public(prompt, cap).map(PeerInference::plain)
    })
}

struct PeerInference {
    text: String,
    receipt: Option<PublicP64InvocationReceipt>,
}

impl PeerInference {
    fn plain(text: String) -> Self {
        Self {
            text,
            receipt: None,
        }
    }
}

#[derive(Serialize)]
struct PublicP64InvocationReceipt {
    model_sha3_256_hex: String,
    sensitivity: u8,
    tokens_generated: u32,
    nquin_words_hex: [String; 6],
}

fn public_text_contains_restricted_marker(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    // A bare DID provides no public-disclosure evidence. Keep identifiers
    // off this unscoped interface, along with explicit Sanctuary labels.
    [
        "did:",
        "sanctuary100",
        "sanctuary-100",
        "sanctuary_100",
        "sanctuary 100",
    ]
    .iter()
    .any(|&marker| lower.contains(marker))
}

fn dispatch_public_inference(
    line: &str,
    infer_public: impl FnOnce(&str, u32) -> Result<PeerInference, String>,
) -> Option<Value> {
    if line.len() > MAX_PEER_LINE {
        return Some(rpc_error(
            Value::Null,
            -32600,
            "Request exceeds the peer limit",
        ));
    }
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return Some(rpc_error(Value::Null, -32700, "Invalid JSON"));
    };
    let Some(request) = message.as_object() else {
        return Some(rpc_error(
            Value::Null,
            -32600,
            "Expected a JSON-RPC request object",
        ));
    };
    let id = request.get("id").cloned();
    let valid_id = id.as_ref().is_none_or(|id| {
        id.is_null()
            || id.as_i64().is_some()
            || id.as_u64().is_some()
            || id.as_str().is_some_and(|s| s.len() <= 128)
    });
    if !valid_id || request.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(rpc_error(Value::Null, -32600, "Invalid JSON-RPC envelope"));
    }
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return Some(rpc_error(
            id.unwrap_or(Value::Null),
            -32600,
            "Missing method",
        ));
    };
    let id = id?; // Notifications never dispatch side effects or receive a reply.
    let result = match method {
        "initialize" => json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "webizen-hostd-peer-p64", "version": env!("CARGO_PKG_VERSION") }
        }),
        "ping" => json!({}),
        "tools/list" => json!({
            "tools": [{
                "name": "llm_infer",
                "description": "Run an explicitly public prompt on the configured public model.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prompt": { "type": "string", "minLength": 1, "maxLength": MAX_PROMPT_BYTES },
                        "sensitivity": { "type": "integer", "const": SENSITIVITY_PUBLIC_ONLY },
                        "max_tokens": { "type": "integer", "minimum": 1, "maximum": MAX_GENERATED_TOKENS, "default": 128 }
                    },
                    "required": ["prompt", "sensitivity"],
                    "additionalProperties": false
                }
            }]
        }),
        "tools/call" => {
            let Some(params) = request.get("params").and_then(Value::as_object) else {
                return Some(rpc_error(id, -32602, "Expected tool parameters"));
            };
            if params.get("name").and_then(Value::as_str) != Some("llm_infer") {
                return Some(rpc_error(id, -32602, "Peer bridge only exposes llm_infer"));
            }
            let Some(args) = params.get("arguments").and_then(|arguments| {
                serde_json::from_value::<PublicInferenceArguments>(arguments.clone()).ok()
            }) else {
                return Some(rpc_error(
                    id,
                    -32602,
                    "Expected a public prompt and explicit sensitivity",
                ));
            };
            if args.sensitivity != SENSITIVITY_PUBLIC_ONLY
                || public_text_contains_restricted_marker(&args.prompt)
            {
                return Some(rpc_error(id, -32001, "Peer inference is public-only"));
            }
            if args.prompt.trim().is_empty()
                || args.prompt.len() > MAX_PROMPT_BYTES
                || !(1..=MAX_GENERATED_TOKENS).contains(&args.max_tokens)
            {
                return Some(rpc_error(
                    id,
                    -32602,
                    "Prompt or token count exceeds the peer limit",
                ));
            }
            let Ok(output) = infer_public(&args.prompt, args.max_tokens) else {
                return Some(rpc_error(
                    id,
                    -32003,
                    "Public inference backend is unavailable",
                ));
            };
            if output.text.trim().is_empty()
                || output.text.len() > MAX_PROMPT_BYTES
                || public_text_contains_restricted_marker(&output.text)
            {
                return Some(rpc_error(
                    id,
                    -32004,
                    "Inference result could not be released to peers",
                ));
            }
            let mut result =
                json!({ "content": [{ "type": "text", "text": output.text }], "isError": false });
            if let Some(receipt) = output.receipt {
                result["_meta"] = json!({ "webizen.publicP64Receipt": receipt });
            }
            result
        }
        _ => return Some(rpc_error(id, -32601, "Method not found")),
    };
    let reply = json!({ "jsonrpc": "2.0", "id": id, "result": result });
    if reply.to_string().len() >= MAX_PEER_LINE {
        return Some(rpc_error(
            id,
            -32004,
            "Inference reply exceeds the peer limit",
        ));
    }
    Some(reply)
}

fn loopback_endpoint(addr: SocketAddr) -> Result<SocketAddr, String> {
    if !addr.ip().is_loopback() {
        return Err("Hostd plaintext transports require loopback. Use an authenticated local tunnel for remote reach.".into());
    }
    Ok(addr)
}

fn read_bounded_line(
    reader: &mut impl BufRead,
    limit: usize,
    deadline: Instant,
) -> io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    loop {
        if Instant::now() > deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Request deadline exceeded",
            ));
        }
        let available = reader.fill_buf()?;
        if available.is_empty() {
            if line.is_empty() {
                return Ok(None);
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Expected a line ending",
            ));
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |end| end + 1);
        if consumed > limit.saturating_sub(line.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Request exceeds the line limit",
            ));
        }
        line.extend_from_slice(&available[..consumed]);
        reader.consume(consumed);
        if newline.is_some() {
            return Ok(Some(line));
        }
    }
}

fn with_timeouts(stream: &TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(PEER_TIMEOUT))?;
    stream.set_write_timeout(Some(PEER_TIMEOUT))
}

fn write_json_line(out: &mut impl Write, message: &Value) -> Result<(), String> {
    let encoded = serde_json::to_vec(message).map_err(|error| error.to_string())?;
    if encoded.len() >= MAX_PEER_LINE {
        return Err("Encoded RPC message exceeds the peer limit".into());
    }
    out.write_all(&encoded)
        .and_then(|_| out.write_all(b"\n"))
        .map_err(|error| error.to_string())
}

fn round_trip_public_rpc(
    stream: &mut TcpStream,
    reader: &mut impl BufRead,
    deadline: Instant,
    request: Value,
) -> Result<Value, String> {
    write_json_line(stream, &request)?;
    let response = read_bounded_line(reader, MAX_PEER_LINE, deadline)
        .map_err(|error| error.to_string())?
        .ok_or("Public inference endpoint closed")?;
    let response: Value = serde_json::from_slice(&response).map_err(|error| error.to_string())?;
    if response.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || response.get("id") != request.get("id")
        || response.get("error").is_some()
    {
        return Err("Invalid public inference response".into());
    }
    response
        .get("result")
        .cloned()
        .ok_or_else(|| "Public inference result is missing".to_string())
}

/// Connect only to an operator-designated public inference process on the
/// local machine. Hostd supplies neither a graph context nor any vault data.
fn forward_public_inference(
    upstream: SocketAddr,
    prompt: &str,
    max_tokens: u32,
) -> Result<String, String> {
    let mut stream =
        TcpStream::connect_timeout(&upstream, PEER_TIMEOUT).map_err(|error| error.to_string())?;
    with_timeouts(&stream).map_err(|error| error.to_string())?;
    let input = stream.try_clone().map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(input);
    let deadline = Instant::now() + PEER_TIMEOUT;

    let initialized = round_trip_public_rpc(
        &mut stream,
        &mut reader,
        deadline,
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": { "protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": { "name": "webizen-hostd", "version": env!("CARGO_PKG_VERSION") } }
        }),
    )?;
    admit_public_upstream(&initialized, max_tokens)?;
    write_json_line(
        &mut stream,
        &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    )?;
    let result = round_trip_public_rpc(
        &mut stream,
        &mut reader,
        deadline,
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": { "name": "llm_infer", "arguments": {
                "prompt": prompt, "max_tokens": max_tokens, "sensitivity": SENSITIVITY_PUBLIC_ONLY
            }}
        }),
    )?;
    public_result_text(&result).map(str::to_owned)
}

// The isolated model adapter advertises this MCP experimental capability:
// "webizen.public_p64": {"sensitivity": 0, "maxTokens": 256, "plainText": true}.
// Its llm_infer tool implements those constraints before executing a prompt.
fn admit_public_upstream(initialized: &Value, max_tokens: u32) -> Result<(), String> {
    let capability = &initialized["capabilities"]["experimental"]["webizen.public_p64"];
    if initialized["protocolVersion"] != "2024-11-05"
        || capability["sensitivity"].as_u64() != Some(u64::from(SENSITIVITY_PUBLIC_ONLY))
        || capability["maxTokens"]
            .as_u64()
            .is_none_or(|cap| cap < u64::from(max_tokens))
        || capability["plainText"].as_bool() != Some(true)
    {
        return Err("Inference endpoint did not advertise the public P64 adapter contract".into());
    }
    Ok(())
}

fn public_result_text(result: &Value) -> Result<&str, String> {
    if result
        .get("isError")
        .is_some_and(|flag| flag.as_bool() != Some(false))
    {
        return Err("Public inference failed".into());
    }
    let content = result["content"]
        .as_array()
        .ok_or("Expected public inference content")?;
    let [item] = content.as_slice() else {
        return Err("Expected a single inference text block".into());
    };
    let Some(item) = item.as_object().filter(|item| item.len() == 2) else {
        return Err("Inference text block contains unsupported fields".into());
    };
    let Some(text) = item.get("text").and_then(Value::as_str) else {
        return Err("Expected inference text".into());
    };
    if item.get("type").and_then(Value::as_str) != Some("text")
        || text.len() > MAX_PROMPT_BYTES
        || text.trim_start().starts_with(['{', '['])
    {
        return Err("Public inference returned unsupported text".into());
    }
    Ok(text)
}

/// Text generated by a trusted host's isolated decoder for the supplied model.
pub struct PublicP64TokenOutput {
    pub text: String,
    pub tokens_generated: u32,
}

struct AdmittedPublicP64Model {
    file: File,
    sha3_256_hex: String,
}

impl AdmittedPublicP64Model {
    fn open(storage: &Path, model_id: &str, sha3_256_hex: &str) -> Result<Self, String> {
        if sha3_256_hex.len() != 64 || !sha3_256_hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("Public P64 admission requires a full SHA3-256 digest".into());
        }
        let registry = read_ledger::<LlmAndAgentManagementRegistry>(
            &storage.join(LLM_MANAGEMENT_STORE_REL_PATH),
        )?
        .ok_or("Public P64 admission requires a provisioned model registry")?;
        if model_id.is_empty() || registry.active_p64_model_id != model_id {
            return Err("The admitted public P64 model must match the active model".into());
        }
        let mut records = registry
            .managed_models
            .iter()
            .filter(|entry| entry.model_id == model_id);
        let record = records
            .next()
            .ok_or("The active model has no registry entry")?;
        if records.next().is_some() || !record.is_p64_native {
            return Err("The active model must be one native P64 registry entry".into());
        }

        let storage_root = storage.canonicalize().map_err(|error| error.to_string())?;
        let models_root = storage_root
            .join("Models")
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if models_root == storage_root || !models_root.starts_with(&storage_root) {
            return Err(
                "Public P64 storage must stay within the provisioned storage directory".into(),
            );
        }

        let recorded_path = &record.p64_vault_path;
        let model_path = if recorded_path.is_absolute() {
            recorded_path.clone()
        } else {
            storage_root.join(recorded_path.strip_prefix(storage).unwrap_or(recorded_path))
        }
        .canonicalize()
        .map_err(|error| error.to_string())?;
        if !model_path.starts_with(&models_root)
            || !model_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("p64"))
            || !model_path.is_file()
        {
            return Err(
                "The public model must be a P64 file within the provisioned Models directory"
                    .into(),
            );
        }

        let mut file = File::open(model_path).map_err(|error| error.to_string())?;
        let mut header = [0u8; 64];
        file.read_exact(&mut header)
            .map_err(|error| error.to_string())?;
        // The canonical Etalon P64 v4 header stores the version as u16,
        // followed by endian/feature flags and the descriptor table offsets.
        let flags = u16::from_le_bytes([header[6], header[7]]);
        let tensor_table_offset = u32::from_le_bytes(header[12..16].try_into().unwrap());
        let tensor_count = u32::from_le_bytes(header[36..40].try_into().unwrap());
        let page_size = u32::from_le_bytes(header[40..44].try_into().unwrap());
        let file_size = file.metadata().map_err(|error| error.to_string())?.len();
        let tensor_table_end = u64::from(tensor_table_offset) + 64 * u64::from(tensor_count);
        if header[..4] != P64_MAGIC_HEADER
            || header[4..6] != 4u16.to_le_bytes()
            || flags & 1 == 0
            || tensor_table_offset < 64
            || tensor_count == 0
            || tensor_table_end > file_size
            || !page_size.is_power_of_two()
            || !(1 << 8..=1 << 30).contains(&page_size)
        {
            return Err("The public model does not have a valid P64 v4 header".into());
        }

        let mut digest = Sha3_256::new();
        digest.update(header);
        let mut chunk = [0u8; 64 * 1024];
        loop {
            let n = file.read(&mut chunk).map_err(|error| error.to_string())?;
            if n == 0 {
                break;
            }
            digest.update(&chunk[..n]);
        }
        let digest = format!("{:x}", digest.finalize());
        if !digest.eq_ignore_ascii_case(sha3_256_hex) {
            return Err("The public model does not match the admitted digest".into());
        }
        Ok(Self {
            file,
            sha3_256_hex: digest,
        })
    }

    fn infer(
        &mut self,
        prompt: &str,
        cap: u32,
        run: &mut impl FnMut(&mut File, &str, u32) -> Result<PublicP64TokenOutput, String>,
    ) -> Result<PeerInference, String> {
        self.file.rewind().map_err(|error| error.to_string())?;
        let output = run(&mut self.file, prompt, cap)?;
        if !(1..=cap).contains(&output.tokens_generated) {
            return Err("Local P64 output exceeds the admitted token budget or is empty".into());
        }
        let text_digest = format!("{:x}", Sha3_256::digest(output.text.as_bytes()));
        let quin = NQuin48::invariant(
            &format!("urn:webizen:p64:sha3-256:{}", self.sha3_256_hex),
            "https://webizen.org/q42#executedPublicP64PeerTurn",
            q_hash_60(&format!("{text_digest}:{}", output.tokens_generated)),
        );
        debug_assert!(quin.verify_parity());
        Ok(PeerInference {
            text: output.text,
            receipt: Some(PublicP64InvocationReceipt {
                model_sha3_256_hex: self.sha3_256_hex.clone(),
                sensitivity: SENSITIVITY_PUBLIC_ONLY,
                tokens_generated: output.tokens_generated,
                nquin_words_hex: quin.0.map(|word| format!("{word:016x}")),
            }),
        })
    }
}

/// Serve an explicitly named public P64 model when there is no configured MCP upstream.
/// The host supplies the full digest and an isolated decoder using the admitted
/// read-only file. It must verify the P64 tensor payload and enforce the supplied
/// cap while generating. Keep the provisioned public file immutable while serving.
/// No roster prompt, graph context or agent tools are passed to the decoder.
pub fn serve_peer_p64_mcp_tcp_with_local_model(
    cfg: &HostDaemonConfig,
    once: bool,
    public_model_id: &str,
    public_model_sha3_256_hex: &str,
    mut run: impl FnMut(&mut File, &str, u32) -> Result<PublicP64TokenOutput, String>,
) -> Result<(), String> {
    let mut model =
        AdmittedPublicP64Model::open(&cfg.storage, public_model_id, public_model_sha3_256_hex)?;
    serve_peer_p64_mcp_tcp_with_fallback(cfg, once, |prompt, cap| {
        model.infer(prompt, cap, &mut run)
    })
}

fn handle_peer_stream(
    mut stream: TcpStream,
    upstream: Option<SocketAddr>,
    fallback: &mut impl FnMut(&str, u32) -> Result<PeerInference, String>,
) -> Result<(), String> {
    with_timeouts(&stream).map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(stream.try_clone().map_err(|error| error.to_string())?);
    for _ in 0..MAX_PEER_REQUESTS {
        let Some(bytes) =
            read_bounded_line(&mut reader, MAX_PEER_LINE, Instant::now() + PEER_TIMEOUT)
                .map_err(|error| error.to_string())?
        else {
            return Ok(());
        };
        let reply = match std::str::from_utf8(&bytes) {
            Ok(line) => dispatch_public_inference(line, |prompt, max_tokens| match upstream {
                Some(upstream) => {
                    forward_public_inference(upstream, prompt, max_tokens).map(PeerInference::plain)
                }
                None => fallback(prompt, max_tokens),
            }),
            Err(_) => Some(rpc_error(Value::Null, -32700, "Expected UTF-8 JSON")),
        };
        if let Some(reply) = reply {
            write_json_line(&mut stream, &reply)?;
        }
    }
    Ok(())
}

pub fn serve_peer_p64_mcp_tcp(cfg: &HostDaemonConfig, once: bool) -> Result<(), String> {
    if std::env::var_os("WEBIZEN_PUBLIC_P64_MCP_UPSTREAM").is_none() {
        if let Some(executable) = std::env::var_os("WEBIZEN_PUBLIC_P64_RUNNER") {
            #[cfg(target_os = "linux")]
            {
                let mut runner = local_p64_runner::Runner::new(PathBuf::from(executable))?;
                let model_id = std::env::var("WEBIZEN_PUBLIC_P64_MODEL").map_err(|_| {
                    "WEBIZEN_PUBLIC_P64_MODEL must name the admitted registry model"
                })?;
                let model_digest = std::env::var("WEBIZEN_PUBLIC_P64_SHA3_256")
                    .map_err(|_| "WEBIZEN_PUBLIC_P64_SHA3_256 must pin the admitted model")?;
                return serve_peer_p64_mcp_tcp_with_local_model(
                    cfg,
                    once,
                    &model_id,
                    &model_digest,
                    |model, prompt, cap| runner.run(model, prompt, cap),
                );
            }
            #[cfg(not(target_os = "linux"))]
            {
                let _ = executable;
                return Err("The standalone public P64 runner is supported on Linux".into());
            }
        }
    }
    // The registry's shell preview cannot decode P64.
    serve_peer_p64_mcp_tcp_with_fallback(cfg, once, |_, _| {
        Err("No public P64 executor is configured".into())
    })
}

fn serve_peer_p64_mcp_tcp_with_fallback(
    cfg: &HostDaemonConfig,
    once: bool,
    mut fallback: impl FnMut(&str, u32) -> Result<PeerInference, String>,
) -> Result<(), String> {
    let addr = loopback_endpoint(cfg.mcp_bind)?;
    let listener =
        TcpListener::bind(addr).map_err(|error| format!("Bind peer MCP {addr}: {error}"))?;
    let bound = listener.local_addr().map_err(|error| error.to_string())?;
    let upstream = match std::env::var("WEBIZEN_PUBLIC_P64_MCP_UPSTREAM") {
        Ok(raw) => {
            let address =
                loopback_endpoint(raw.parse().map_err(|_| "Invalid public model endpoint")?)?;
            if address.port() == 0 || address.port() == bound.port() {
                return Err("Public model endpoint must use a separate nonzero port".into());
            }
            Some(address)
        }
        Err(std::env::VarError::NotPresent) => None,
        Err(_) => return Err("Invalid public model endpoint environment value".into()),
    };
    for stream in listener.incoming() {
        let stream = stream.map_err(|error| error.to_string())?;
        let result = handle_peer_stream(stream, upstream, &mut fallback);
        if once {
            return result;
        }
        if let Err(error) = result {
            eprintln!("Peer MCP connection closed: {error}");
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
mod local_p64_runner {
    use super::{
        File, Instant, PathBuf, PublicP64TokenOutput, MAX_PEER_LINE, SENSITIVITY_PUBLIC_ONLY,
    };
    use serde::{Deserialize, Serialize};
    use std::{
        io::{self, Read, Write},
        net::Shutdown,
        os::{fd::OwnedFd, unix::net::UnixStream},
        process::{Child, Command, Stdio},
        thread,
        time::Duration,
    };

    const PROTOCOL: &str = "webizen.public-p64.v1";
    const RUNNER_TIMEOUT: Duration = Duration::from_secs(30);
    const POLL_INTERVAL: Duration = Duration::from_millis(5);

    // The operator provisions an isolated decoder without graph, tools or vault access.
    // It receives one JSON line on stdin, the verified model on read-only fd 2 (stderr),
    // and returns one JSON object on stdout. The decoder should not spawn background jobs.
    // The protocol supplies no network/sandbox capability. Process isolation comes from
    // the host deployment, and the admitted model file must remain immutable while in use.
    #[derive(Serialize)]
    struct Request<'a> {
        protocol: &'static str,
        model_fd: u8,
        prompt: &'a str,
        max_tokens: u32,
        sensitivity: u8,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Reply {
        protocol: String,
        text: String,
        tokens_generated: u32,
    }

    pub(super) struct Runner {
        binary: PathBuf,
        stopping_child: Option<Child>,
    }

    impl Runner {
        pub(super) fn new(binary: PathBuf) -> Result<Self, String> {
            if !binary.is_absolute() {
                return Err("WEBIZEN_PUBLIC_P64_RUNNER must be an absolute executable path".into());
            }
            let binary = binary.canonicalize().map_err(|error| error.to_string())?;
            if !binary.is_file() {
                return Err("The configured public P64 runner is not a regular file".into());
            }
            Ok(Self {
                binary,
                stopping_child: None,
            })
        }

        pub(super) fn run(
            &mut self,
            model: &File,
            prompt: &str,
            cap: u32,
        ) -> Result<PublicP64TokenOutput, String> {
            if let Some(child) = self.stopping_child.as_mut() {
                if child
                    .try_wait()
                    .map_err(|error| error.to_string())?
                    .is_none()
                {
                    return Err("The previous public P64 runner is still stopping".into());
                }
                self.stopping_child = None;
            }

            let mut request = serde_json::to_vec(&Request {
                protocol: PROTOCOL,
                model_fd: 2,
                prompt,
                max_tokens: cap,
                sensitivity: SENSITIVITY_PUBLIC_ONLY,
            })
            .map_err(|error| error.to_string())?;
            if request.len() > MAX_PEER_LINE + 1024 {
                return Err("The encoded public P64 runner request exceeds its byte limit".into());
            }
            request.push(b'\n');

            // Use separate socketpair endpoints so only the host side is nonblocking.
            // This avoids reader/writer threads that could wait after the deadline.
            let (input, child_input) = bounded_socketpair().map_err(|error| error.to_string())?;
            let (output, child_output) = bounded_socketpair().map_err(|error| error.to_string())?;
            let model = model.try_clone().map_err(|error| error.to_string())?;
            let mut child = Command::new(&self.binary)
                .env_clear()
                .current_dir("/")
                .stdin(Stdio::from(child_input))
                .stdout(Stdio::from(child_output))
                .stderr(Stdio::from(model))
                .spawn()
                .map_err(|error| error.to_string())?;
            let result = exchange(&mut child, input, output, &request);
            if result.is_err() && !matches!(child.try_wait(), Ok(Some(_))) {
                let _ = child.kill();
                if !matches!(child.try_wait(), Ok(Some(_))) {
                    self.stopping_child = Some(child);
                }
            }
            result
        }
    }

    fn bounded_socketpair() -> io::Result<(UnixStream, OwnedFd)> {
        let (host, child) = UnixStream::pair()?;
        host.set_nonblocking(true)?;
        Ok((host, child.into()))
    }

    fn exchange(
        child: &mut Child,
        mut input: UnixStream,
        mut output: UnixStream,
        request: &[u8],
    ) -> Result<PublicP64TokenOutput, String> {
        let deadline = Instant::now() + RUNNER_TIMEOUT;
        let mut written = 0;
        let mut response = Vec::new();
        let mut input_closed = false;
        let mut output_closed = false;
        let mut chunk = [0u8; 4096];
        while Instant::now() < deadline {
            let mut progressed = false;
            if written < request.len() {
                match input.write(&request[written..]) {
                    Ok(0) => return Err("Public P64 runner closed the input".into()),
                    Ok(count) => {
                        written += count;
                        progressed = true;
                    }
                    Err(error) if transient_io(&error) => {}
                    Err(error) => return Err(error.to_string()),
                }
            }
            if written == request.len() && !input_closed {
                // A fast helper may close its input after consuming the line.
                if let Err(error) = input.shutdown(Shutdown::Write) {
                    if !matches!(
                        error.kind(),
                        io::ErrorKind::NotConnected | io::ErrorKind::BrokenPipe
                    ) {
                        return Err(error.to_string());
                    }
                }
                input_closed = true;
                progressed = true;
            }
            if !output_closed {
                match output.read(&mut chunk) {
                    Ok(0) => {
                        output_closed = true;
                        progressed = true;
                    }
                    Ok(count) => {
                        if count > MAX_PEER_LINE.saturating_sub(response.len()) {
                            return Err("Public P64 runner exceeded the output byte limit".into());
                        }
                        response.extend_from_slice(&chunk[..count]);
                        progressed = true;
                    }
                    Err(error) if transient_io(&error) => {}
                    Err(error) => return Err(error.to_string()),
                }
            }
            if input_closed && output_closed {
                if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
                    return if status.success() {
                        parse_reply(&response)
                    } else {
                        Err("Public P64 runner exited unsuccessfully".into())
                    };
                }
            }
            if !progressed {
                thread::sleep(POLL_INTERVAL);
            }
        }
        Err("Public P64 runner exceeded its execution deadline".into())
    }

    fn transient_io(error: &io::Error) -> bool {
        matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
        )
    }

    fn parse_reply(response: &[u8]) -> Result<PublicP64TokenOutput, String> {
        let reply: Reply =
            serde_json::from_slice(response).map_err(|_| "Invalid public P64 runner response")?;
        if reply.protocol != PROTOCOL {
            return Err("The public P64 runner used an incompatible protocol".into());
        }
        Ok(PublicP64TokenOutput {
            text: reply.text,
            tokens_generated: reply.tokens_generated,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rejects_data_outside_the_runner_contract() {
            let output = parse_reply(
                br#"{"protocol":"webizen.public-p64.v1","text":"Public","tokens_generated":1}"#,
            )
            .unwrap();
            assert_eq!(
                (output.text.as_str(), output.tokens_generated),
                ("Public", 1)
            );
            assert!(parse_reply(
                br#"{"protocol":"webizen.public-p64.v1","text":"Public","tokens_generated":1,"model_path":"private"}"#,
            )
            .is_err());
            assert!(
                parse_reply(br#"{"protocol":"unknown","text":"Public","tokens_generated":1}"#,)
                    .is_err()
            );
        }
    }
}

fn http_reply(stream: &mut TcpStream, status: &str, value: &Value) -> Result<(), String> {
    let body = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    write!(stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len(),
    ).and_then(|_| stream.write_all(&body)).map_err(|error| error.to_string())
}

fn bearer_matches(actual: Option<&str>, expected: &str) -> bool {
    let Some(token) = actual.and_then(|value| value.strip_prefix("Bearer ")) else {
        return false;
    };
    token.len() == expected.len()
        && token
            .bytes()
            .zip(expected.bytes())
            .fold(0, |mismatch, (a, b)| mismatch | (a ^ b))
            == 0
}

fn handle_http_control(
    mut stream: TcpStream,
    cfg: &HostDaemonConfig,
    bearer: Option<&str>,
) -> Result<(), String> {
    with_timeouts(&stream).map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(stream.try_clone().map_err(|error| error.to_string())?);
    let deadline = Instant::now() + PEER_TIMEOUT;
    let mut remaining = MAX_HTTP_HEADER;
    let request_line = read_bounded_line(&mut reader, remaining, deadline)
        .map_err(|error| error.to_string())?
        .ok_or("Empty HTTP request")?;
    remaining -= request_line.len();
    let request = std::str::from_utf8(&request_line).map_err(|_| "Invalid HTTP request line")?;
    let words: Vec<&str> = request.split_ascii_whitespace().collect();
    if words.len() != 3 || !matches!(words[2], "HTTP/1.0" | "HTTP/1.1") {
        return http_reply(
            &mut stream,
            "400 Bad Request",
            &json!({ "error": "Malformed request" }),
        );
    }

    let mut auth_header = None;
    loop {
        let bytes = read_bounded_line(&mut reader, remaining, deadline)
            .map_err(|error| error.to_string())?
            .ok_or("HTTP headers are incomplete")?;
        remaining -= bytes.len();
        if bytes == b"\r\n" {
            break;
        }
        let line = std::str::from_utf8(&bytes).map_err(|_| "Invalid HTTP header")?;
        let Some((key, value)) = line.trim_end_matches("\r\n").split_once(':') else {
            return http_reply(
                &mut stream,
                "400 Bad Request",
                &json!({ "error": "Malformed header" }),
            );
        };
        if key.eq_ignore_ascii_case("Authorization")
            && auth_header.replace(value.trim().to_owned()).is_some()
            || key.eq_ignore_ascii_case("Transfer-Encoding")
            || (key.eq_ignore_ascii_case("Content-Length") && value.trim() != "0")
        {
            return http_reply(
                &mut stream,
                "400 Bad Request",
                &json!({ "error": "Unsupported HTTP framing or credentials" }),
            );
        }
    }

    if bearer.is_some_and(|expected| !bearer_matches(auth_header.as_deref(), expected)) {
        return http_reply(
            &mut stream,
            "401 Unauthorized",
            &json!({ "error": "Authorization required" }),
        );
    }
    let command = match (words[0], words[1]) {
        ("GET", "/status") => HostDaemonCommand::Status,
        ("POST", "/run-tick") if bearer.is_some() => HostDaemonCommand::RunTick,
        ("POST", "/run-tick") => {
            return http_reply(
                &mut stream,
                "403 Forbidden",
                &json!({ "error": "HTTP control requires bearer authentication" }),
            );
        }
        _ => {
            return http_reply(
                &mut stream,
                "404 Not Found",
                &json!({ "error": "Unknown control route" }),
            )
        }
    };
    match execute_hostd_command(cfg, command) {
        Ok(receipt) => http_reply(&mut stream, "200 OK", &receipt),
        Err(error) => {
            eprintln!("Hostd control command failed: {error}");
            http_reply(
                &mut stream,
                "500 Internal Server Error",
                &json!({ "error": "Hostd control command failed" }),
            )
        }
    }
}

pub fn serve_hostd_http_control(cfg: &HostDaemonConfig, once: bool) -> Result<(), String> {
    let addr = loopback_endpoint(cfg.bind)?;
    let bearer = match cfg.auth_mode {
        HostDaemonAuthMode::Loopback => None,
        HostDaemonAuthMode::Bearer => {
            let token = std::env::var("WEBIZEN_HOSTD_CONTROL_TOKEN")
                .map_err(|_| "WEBIZEN_HOSTD_CONTROL_TOKEN is required in bearer mode")?;
            if !(32..=256).contains(&token.len())
                || !token.bytes().all(|byte| byte.is_ascii_graphic())
            {
                return Err(
                    "Hostd bearer token must contain 32 to 256 printable non-space ASCII bytes"
                        .into(),
                );
            }
            Some(token)
        }
    };
    let listener =
        TcpListener::bind(addr).map_err(|error| format!("Bind hostd control {addr}: {error}"))?;
    for stream in listener.incoming() {
        let stream = stream.map_err(|error| error.to_string())?;
        let result = handle_http_control(stream, cfg, bearer.as_deref());
        if once {
            return result;
        }
        if let Err(error) = result {
            eprintln!("Hostd control connection closed: {error}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_ID: AtomicUsize = AtomicUsize::new(0);

    struct TestStorage(PathBuf);
    impl TestStorage {
        fn new() -> Self {
            let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("webizen-hostd-test-{}-{id}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestStorage {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn parse(args: &[&str]) -> (HostDaemonConfig, HostDaemonCommand) {
        parse_hostd_args(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()
    }

    fn peer_call(arguments: Value) -> String {
        json!({
            "jsonrpc": "2.0", "id": 7, "method": "tools/call",
            "params": { "name": "llm_infer", "arguments": arguments }
        })
        .to_string()
    }

    #[test]
    fn executes_hostd_cli_commands_and_enforces_public_only_peer_p64_mcp() {
        let storage = TestStorage::new();
        let (cfg, status) = parse(&["--storage", storage.0.to_str().unwrap(), "status", "--json"]);
        assert!(cfg.json);
        assert!(
            !execute_hostd_command(&cfg, status).unwrap()["stewardship_persisted"]
                .as_bool()
                .unwrap()
        );
        assert!(!cfg.storage.join(HOST_STEWARDSHIP_STORE_REL_PATH).exists());

        let (_, tick) = parse(&["run-tick"]);
        let receipt = execute_hostd_command(&cfg, tick).unwrap();
        assert!(receipt["care_heartbeats"]
            .as_array()
            .unwrap()
            .iter()
            .all(|report| report["status"] == "no_observation"));

        let ledger = NaturalHostStewardshipFabric::default();
        let steward = &ledger.stewards.first().unwrap().steward_did;
        let (_, register) = parse(&[
            "register-project",
            "new-fixture-project",
            "Fixture Project",
            "fixture.example",
            "personal",
            steward,
        ]);
        assert!(execute_hostd_command(&cfg, register.clone()).is_err());
        assert!(!cfg.storage.join(HOST_STEWARDSHIP_STORE_REL_PATH).exists());
        ledger.save(&cfg.storage).unwrap();
        let registered = execute_hostd_command(&cfg, register).unwrap();
        assert_eq!(registered["project_id"], "new-fixture-project");
        assert_eq!(
            registered["project_steward_dids"][0].as_str(),
            Some(steward.as_str())
        );
        assert!(run_daemon_tick(&cfg).unwrap().status.stewardship_persisted);

        let (_, audit) = parse(&["audit-jural-license", "natural"]);
        assert_eq!(
            execute_hostd_command(&cfg, audit).unwrap()["monthly_fee_cents"],
            0
        );
        let (_, audit) = parse(&[
            "audit-jural-license",
            "incorporated",
            "--entity",
            "absent-fixture",
        ]);
        assert!(execute_hostd_command(&cfg, audit).is_err());

        for arguments in [
            json!({"prompt":"private data", "sensitivity": 255}),
            json!({"prompt":"Sanctuary-100 note", "sensitivity": 0}),
            json!({"prompt":"Review did:qi:pairwise:example", "sensitivity": 0}),
            json!({"prompt":"missing classification"}),
            json!({"prompt":"public", "sensitivity": 0, "graph_context":"private"}),
        ] {
            let rejected = handle_peer_p64_mcp_json_rpc_line(&peer_call(arguments), |_, _| {
                panic!("unadmitted input reached the inference adapter")
            })
            .unwrap();
            assert!(rejected["error"].is_object());
        }

        let admitted = handle_peer_p64_mcp_json_rpc_line(
            &peer_call(json!({"prompt":"public", "sensitivity": 0, "max_tokens": 16})),
            |prompt, cap| {
                assert_eq!((prompt, cap), ("public", 16));
                Ok("test adapter response".into())
            },
        )
        .unwrap();
        assert_eq!(
            admitted["result"]["content"][0]["text"],
            "test adapter response"
        );
        assert_eq!(admitted["id"], 7);

        let initialized = handle_peer_p64_mcp_json_rpc_line(
            r#"{"jsonrpc":"2.0","id":"init","method":"initialize"}"#,
            |_, _| unreachable!(),
        )
        .unwrap();
        assert_eq!(
            initialized["result"]["serverInfo"]["name"],
            "webizen-hostd-peer-p64"
        );
        let tools = handle_peer_p64_mcp_json_rpc_line(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
            |_, _| unreachable!(),
        )
        .unwrap();
        assert_eq!(tools["result"]["tools"][0]["name"], "llm_infer");
        assert!(handle_peer_p64_mcp_json_rpc_line(
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            |_, _| unreachable!(),
        )
        .is_none());
    }

    #[test]
    fn admits_the_pinned_local_model_and_seals_only_bounded_public_output() {
        let storage = TestStorage::new();
        let cfg = HostDaemonConfig::for_storage(&storage.0);
        let models = storage.0.join("Models");
        fs::create_dir(&models).unwrap();
        let model_path = models.join("fixture-public.p64");

        // This synthetic header exercises admission and a fake decoder;
        // it does not represent usable model tensors.
        let mut bytes = [0u8; 192];
        bytes[..4].copy_from_slice(&P64_MAGIC_HEADER);
        bytes[4..6].copy_from_slice(&4u16.to_le_bytes());
        bytes[6..8].copy_from_slice(&1u16.to_le_bytes());
        bytes[12..16].copy_from_slice(&64u32.to_le_bytes());
        bytes[36..40].copy_from_slice(&1u32.to_le_bytes());
        bytes[40..44].copy_from_slice(&16_384u32.to_le_bytes());
        let digest = format!("{:x}", Sha3_256::digest(bytes));
        fs::write(&model_path, bytes).unwrap();

        let mut registry = LlmAndAgentManagementRegistry::default();
        let model_id = registry.active_p64_model_id.clone();
        let active = registry
            .managed_models
            .iter_mut()
            .find(|record| record.model_id == model_id)
            .unwrap();
        active.p64_vault_path = model_path;
        registry.save(&cfg.storage).unwrap();

        assert!(AdmittedPublicP64Model::open(&cfg.storage, "other-model", &digest).is_err());
        assert!(AdmittedPublicP64Model::open(&cfg.storage, &model_id, &"0".repeat(64)).is_err());
        let mut model = AdmittedPublicP64Model::open(&cfg.storage, &model_id, &digest).unwrap();

        let request = peer_call(json!({
            "prompt": "Public fixture prompt",
            "sensitivity": SENSITIVITY_PUBLIC_ONLY,
            "max_tokens": 2,
        }));
        let reply = dispatch_public_inference(&request, |prompt, cap| {
            model.infer(prompt, cap, &mut |file, prompt, cap| {
                let mut magic = [0u8; 4];
                file.read_exact(&mut magic).unwrap();
                assert_eq!(magic, P64_MAGIC_HEADER);
                assert_eq!((prompt, cap), ("Public fixture prompt", 2));
                Ok(PublicP64TokenOutput {
                    text: "Fixture completion".into(),
                    tokens_generated: 2,
                })
            })
        })
        .unwrap();
        assert_eq!(reply["result"]["content"][0]["text"], "Fixture completion");
        let receipt = &reply["result"]["_meta"]["webizen.publicP64Receipt"];
        assert_eq!(
            receipt["model_sha3_256_hex"].as_str(),
            Some(digest.as_str())
        );
        assert_eq!(receipt["tokens_generated"], 2);
        assert_eq!(receipt["sensitivity"], SENSITIVITY_PUBLIC_ONLY);
        let words: Vec<_> = receipt["nquin_words_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|word| u64::from_str_radix(word.as_str().unwrap(), 16).unwrap())
            .collect();
        assert!(NQuin48(words.try_into().unwrap()).verify_parity());

        let oversized = dispatch_public_inference(&request, |prompt, cap| {
            model.infer(prompt, cap, &mut |_, _, cap| {
                Ok(PublicP64TokenOutput {
                    text: "This exceeds the token budget".into(),
                    tokens_generated: cap + 1,
                })
            })
        })
        .unwrap();
        assert_eq!(oversized["error"]["code"], -32003);

        let restricted =
            handle_peer_p64_mcp_json_rpc_line(
                &request,
                |_, _| Ok("DID:qi:pairwise:example".into()),
            )
            .unwrap();
        assert_eq!(restricted["error"]["code"], -32004);
    }

    #[test]
    fn principal_checkin_supersedes_a_stale_heartbeat_observation() {
        let mut fabric = NaturalHostStewardshipFabric::default();
        let circle = fabric
            .shared_care_circles
            .iter_mut()
            .find(|circle| {
                circle.topology
                    == SharedLivingAndCareTopologyKind::ElderlyOrSupportedPersonWithRemoteCarers
                    && circle.heartbeat_grace_secs > 0
            })
            .unwrap();
        let observation = HeartbeatObservation {
            circle_id: circle.circle_id.clone(),
            principal_did: circle.principal_did.clone(),
            last_checkin_unix_secs: 1,
        };
        let now = observation.last_checkin_unix_secs + circle.heartbeat_grace_secs;
        assert!(
            observe_heartbeats(std::slice::from_ref(&observation), &fabric, now)
                .iter()
                .any(|report| report.circle_id == observation.circle_id
                    && report.status == CareHeartbeatStatus::Lapsed)
        );

        let circle = fabric
            .shared_care_circles
            .iter_mut()
            .find(|circle| circle.circle_id == observation.circle_id)
            .unwrap();
        circle.last_principal_checkin_unix_secs = Some(now);
        assert!(
            observe_heartbeats(std::slice::from_ref(&observation), &fabric, now)
                .iter()
                .any(|report| report.circle_id == observation.circle_id
                    && report.status == CareHeartbeatStatus::WithinGrace)
        );
    }

    #[test]
    fn admits_only_the_dedicated_public_upstream_and_plain_text() {
        let initialized = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "experimental": { "webizen.public_p64": {
                "sensitivity": SENSITIVITY_PUBLIC_ONLY, "maxTokens": 256, "plainText": true
            } } }
        });
        assert!(admit_public_upstream(&initialized, 32).is_ok());
        assert!(admit_public_upstream(&initialized, 257).is_err());
        assert!(admit_public_upstream(&json!({"capabilities": {"tools": {}}}), 32).is_err());

        assert_eq!(
            public_result_text(&json!({"content": [{"type": "text", "text": "public"}]})),
            Ok("public"),
        );
        assert!(public_result_text(&json!({
            "content": [{
                "type": "text",
                "text": r#"{"text":"public","modelPath":"private"}"#
            }]
        }))
        .is_err());
    }

    #[test]
    fn fails_closed_on_invalid_ledger_and_unframed_input() {
        let storage = TestStorage::new();
        let cfg = HostDaemonConfig::for_storage(&storage.0);
        let path = storage.0.join(HOST_STEWARDSHIP_STORE_REL_PATH);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"{invalid ledger").unwrap();
        assert!(execute_hostd_command(&cfg, HostDaemonCommand::Status).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{invalid ledger");

        let mut stream = b"larger than limit\n".as_slice();
        assert_eq!(
            read_bounded_line(&mut stream, 7, Instant::now() + PEER_TIMEOUT)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData,
        );
        assert!(loopback_endpoint("0.0.0.0:8080".parse().unwrap()).is_err());
        assert!(!bearer_matches(Some("Bearer incorrect"), "token"));
        assert!(bearer_matches(Some("Bearer token"), "token"));
    }
}
