mod qualia_admission;

use self::qualia_admission::QualiaAdmissionPanel;
use crate::components::auxiliary_windows::{
    apply_shell_command, jump_to_stage, AuxShellCommand, AuxWindowBridge, CareResponseAuthority,
};
use crate::components::{HaloBar, OrbitWheel, StageHost};
use crate::state::{refresh_refract_host_records, session, ShellState, WebizenRefractModel};
use aether::{
    aether_core::{provide_context, try_consume_context},
    prelude::*,
};
use aether_native::winit::{
    event::{ElementState, WindowEvent},
    keyboard::{Key, ModifiersState},
};
use aether_native::{AetherNativeWindowRenderer, RendererFallbackPolicy};
use aether_qualia_host::{prelude::QualiaHostContext, NativeAuthoringBridge};
use serde_json::Value;
use std::{
    cell::{Cell, RefCell},
    fs::File,
    io::Read,
    path::PathBuf,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use webizen_ua_os::{
    validate_human_centric_host_nomenclature, verify_detached_ed25519, AdoptedQualiaFoundation,
    DispatchedCareAndGuardianAlert, OsJobRunner,
};

static C1_SMOKE_REDRAWS: AtomicUsize = AtomicUsize::new(0);
const SHELL_CSS: &str = include_str!("../assets/shell.css");
const CARE_GRANT_DOMAIN: &[u8] = b"webizen-care-response-authority-v1\0";
const CARE_GRANT_MAX_BYTES: usize = 16 * 1024;
const CARE_GRANT_MAX_LIFETIME: u64 = 8 * 60 * 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CareGrantRole {
    Principal,
    Carer,
}

#[derive(Clone, Debug)]
struct SessionCareGrant {
    circle_id: String,
    responder_did: String,
    role: CareGrantRole,
    not_before: u64,
    expires: u64,
}

impl SessionCareGrant {
    fn parse(claims: &str, fabric_did: &str, domain: &str, now: u64) -> Option<Self> {
        let fields: Value = serde_json::from_str(claims).ok()?;
        if fabric_did.is_empty()
            || domain.is_empty()
            || fields["fabric_did"].as_str()? != fabric_did
            || fields["host_domain"].as_str()? != domain
        {
            return None;
        }
        let clean_field = |name: &str| {
            let value = fields[name].as_str()?;
            (!value.is_empty()
                && value.len() <= 256
                && !value.chars().any(|c| c.is_whitespace() || c.is_control())
                && validate_human_centric_host_nomenclature(value).is_ok())
            .then_some(value.to_owned())
        };
        let responder_did = clean_field("responder_did")?;
        if !responder_did
            .strip_prefix("did:")
            .is_some_and(|suffix| suffix.split_once(':').is_some())
        {
            return None;
        }
        let grant = Self {
            circle_id: clean_field("circle_id")?,
            responder_did,
            role: match fields["role"].as_str()? {
                "principal" => CareGrantRole::Principal,
                "carer" => CareGrantRole::Carer,
                _ => return None,
            },
            not_before: fields["not_before"].as_u64()?,
            expires: fields["expires"].as_u64()?,
        };
        (grant.expires.checked_sub(grant.not_before)? <= CARE_GRANT_MAX_LIFETIME
            && grant.is_current(now))
        .then_some(grant)
    }

    fn is_current(&self, now: u64) -> bool {
        self.not_before <= now && now < self.expires
    }

    fn resolve(
        &self,
        alert: &DispatchedCareAndGuardianAlert,
        principal_ok: bool,
        now: u64,
    ) -> Result<String, String> {
        let matches_responder = match self.role {
            CareGrantRole::Principal => principal_ok && alert.principal_did == self.responder_did,
            CareGrantRole::Carer => {
                !principal_ok
                    && alert.principal_did != self.responder_did
                    && alert.notified_remote_guardian_dids.iter().any(|recipient| {
                        recipient.split_whitespace().next() == Some(self.responder_did.as_str())
                    })
            }
        };
        if self.is_current(now)
            && !alert.resolved_or_reversed
            && alert.circle_id == self.circle_id
            && matches_responder
        {
            Ok(self.responder_did.clone())
        } else {
            Err("The authenticated care grant does not cover this alert or response.".into())
        }
    }
}

fn unix_time() -> Option<u64> {
    Some(SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs())
}

fn session_care_authority() -> Option<CareResponseAuthority> {
    let session = session()?;
    // The host supplies the grant file and verification key independently. The envelope's
    // "claims" string is signed verbatim after the domain prefix, using "signature_hex".
    let trusted_key = std::env::var("WEBIZEN_CARE_AUTHORITY_PUBKEY").ok()?;
    let path = PathBuf::from(std::env::var_os("WEBIZEN_CARE_AUTHORITY_FILE")?);
    if !path.is_absolute() || !path.metadata().ok()?.is_file() {
        return None;
    }
    let file = File::open(path).ok()?;
    let mut raw_envelope = Vec::new();
    file.take(CARE_GRANT_MAX_BYTES as u64 + 1)
        .read_to_end(&mut raw_envelope)
        .ok()?;
    if raw_envelope.len() > CARE_GRANT_MAX_BYTES {
        return None;
    }
    let envelope: Value = serde_json::from_slice(&raw_envelope).ok()?;
    let claims = envelope["claims"].as_str()?;
    let signature = envelope["signature_hex"].as_str()?;
    let grant = SessionCareGrant::parse(
        claims,
        session.fabric_identifier(),
        &session.host_stewardship.primary_host_domain,
        unix_time()?,
    )?;
    let mut signed_bytes = Vec::with_capacity(CARE_GRANT_DOMAIN.len() + claims.len());
    signed_bytes.extend_from_slice(CARE_GRANT_DOMAIN);
    signed_bytes.extend_from_slice(claims.as_bytes());
    verify_detached_ed25519(&trusted_key, signature, &signed_bytes).ok()?;

    let circle = session
        .host_stewardship
        .shared_care_circles
        .iter()
        .find(|circle| circle.circle_id == grant.circle_id)?;
    let in_circle = match grant.role {
        CareGrantRole::Principal => circle.principal_did == grant.responder_did,
        CareGrantRole::Carer => {
            circle.principal_did != grant.responder_did
                && circle.participants.iter().any(|member| {
                    member.participant_did == grant.responder_did
                        && !member.consent_credential_id.trim().is_empty()
                })
        }
    };
    in_circle.then(|| CareResponseAuthority {
        resolve: Rc::new(move |alert, principal_ok| {
            let now = unix_time().ok_or("The device clock cannot validate the care grant.")?;
            grant.resolve(alert, principal_ok, now)
        }),
    })
}

fn arm_omni_shortcut(mut state: Signal<ShellState>) {
    let modifiers = use_hook(|| Rc::new(Cell::new(ModifiersState::empty())));
    aether_native::use_window_event(move |event, _| match event {
        WindowEvent::ModifiersChanged(change) => modifiers.set(change.state()),
        WindowEvent::Focused(false) => modifiers.set(ModifiersState::empty()),
        WindowEvent::KeyboardInput { event, .. }
            if event.state == ElementState::Pressed && !event.repeat =>
        {
            let held = modifiers.get();
            if held.control_key()
                && !held.alt_key()
                && matches!(&event.logical_key, Key::Character(key) if key.eq_ignore_ascii_case("k"))
                && !state.peek().launcher_open
            {
                state.write().launcher_open = true;
            }
        }
        _ => {}
    });
}

fn arm_window_commands(
    mut state: Signal<ShellState>,
    bridge: AuxWindowBridge,
    care_authority: Option<CareResponseAuthority>,
) {
    let window = aether_native::use_window();
    let app = aether_native::use_native_app();
    use_hook({
        let window = window.clone();
        let bridge = bridge.clone();
        move || {
            bridge.bind_main(window.clone());
            crate::tray::bind_window(&window);
        }
    });

    aether_native::use_window_event(move |event, event_loop| {
        if matches!(event, WindowEvent::CloseRequested) {
            // Retained overlays should not keep the shell alive after its main window closes.
            event_loop.exit();
            return;
        }

        if let Some(stage) = crate::tray::take_stage() {
            jump_to_stage(&mut state.write(), stage);
        }
        for kind in crate::tray::take_overlays() {
            bridge.open(&app, kind);
        }
        for command in bridge.drain_commands() {
            match command {
                AuxShellCommand::Open(kind) => bridge.open(&app, kind),
                AuxShellCommand::Dismiss(kind) => bridge.dismiss(&app, kind),
                command => {
                    apply_shell_command(&mut state.write(), command, care_authority.as_ref())
                }
            }
        }

        let (orb, alert, palette) = bridge.open_flags();
        let flags_changed = {
            let state = state.peek();
            (
                state.aux_orb_window_open,
                state.aux_alert_modal_open,
                state.aux_palette_window_open,
            ) != (orb, alert, palette)
        };
        if flags_changed {
            let mut state = state.write();
            state.aux_orb_window_open = orb;
            state.aux_alert_modal_open = alert;
            state.aux_palette_window_open = palette;
        }
        let state = state.peek();
        bridge.sync_from_state(&state, care_authority.is_some());
    });
}

/// Bounded WEBIZEN_C1_SMOKE=1 path: three native redraws then event-loop exit.
/// Hook order is fixed; the env flag only changes whether redraws request exit.
fn arm_c1_smoke() {
    let smoke = std::env::var("WEBIZEN_C1_SMOKE").ok().as_deref() == Some("1");
    let window = aether_native::use_window();
    if smoke {
        window.request_redraw();
    }
    aether_native::use_window_event(move |event, event_loop| {
        if !smoke {
            return;
        }
        if matches!(event, WindowEvent::RedrawRequested) {
            let frames = C1_SMOKE_REDRAWS.fetch_add(1, Ordering::SeqCst) + 1;
            if frames >= 3 {
                event_loop.exit();
            } else {
                window.request_redraw();
            }
        }
    });
}

fn arm_background_jobs(mut state: Signal<ShellState>) {
    let window = aether_native::use_window();
    let (runner, pending_refresh) = use_hook(move || {
        let pending_refresh = Arc::new(AtomicBool::new(true));
        let runner = session()
            .filter(|_| state.peek().os_control_center_error.is_none())
            .and_then(|session| {
                let changed = Arc::clone(&pending_refresh);
                let wake_window = window.clone();
                match OsJobRunner::new(session, move || {
                    changed.store(true, Ordering::Release);
                    wake_window.request_redraw();
                }) {
                    Ok(runner) => {
                        window.request_redraw();
                        Some(provide_context(runner))
                    }
                    Err(error) => {
                        state.write().os_job_runner_error = Some(error);
                        None
                    }
                }
            });
        (runner, pending_refresh)
    });

    aether_native::use_window_event(move |event, _| {
        if !matches!(event, WindowEvent::RedrawRequested)
            || !pending_refresh.swap(false, Ordering::AcqRel)
        {
            return;
        }
        let Some(runner) = runner.as_ref() else {
            return;
        };
        let mut current = state.peek().os_control_center.clone();
        match runner.refresh(&mut current) {
            Ok(changed) => {
                if changed || state.peek().os_job_runner_error.is_some() {
                    let mut shell = state.write();
                    current.sync_notifications_from_stewardship(&shell.host_stewardship);
                    shell.os_control_center = current;
                    shell.os_job_runner_error = None;
                }
            }
            Err(error) => {
                pending_refresh.store(true, Ordering::Release);
                let is_new = state.peek().os_job_runner_error.as_deref() != Some(error.as_str());
                if is_new {
                    state.write().os_job_runner_error = Some(error);
                }
            }
        }
    });
}

fn arm_renderer_health(mut state: Signal<ShellState>) {
    let renderer = try_consume_context::<AetherNativeWindowRenderer>();
    let window = aether_native::use_window();
    aether_native::use_window_event(move |event, _| {
        if !matches!(
            event,
            WindowEvent::RedrawRequested | WindowEvent::SurfaceResized(_)
        ) {
            return;
        }
        let Some(renderer) = renderer.as_ref() else {
            return;
        };
        let health = renderer.health();
        let physical_size = window.surface_size();
        let dimensions = (physical_size.width, physical_size.height);
        let fallback_engaged = renderer.fallback_engaged();
        let policy = match renderer.fallback_policy() {
            RendererFallbackPolicy::CpuOnFailure if cfg!(feature = "gpu-cpu-fallback") => {
                "CPU on GPU failure"
            }
            RendererFallbackPolicy::CpuOnFailure => "CPU fallback is disabled in this build",
            RendererFallbackPolicy::FailClosed => "Stop painting on renderer fault",
        };
        let backend = if fallback_engaged {
            format!("{} · software fallback", health.backend_name)
        } else if health.faulted {
            format!("{} · renderer fault", health.backend_name)
        } else if health.gpu_active {
            format!("{} · GPU active", health.backend_name)
        } else {
            health.backend_name.to_owned()
        };
        let changed = {
            let current = &state.peek().renderer_health;
            current.backend_label != backend
                || current.fallback_policy_label != policy
                || current.fallback_engaged != fallback_engaged
                || current.viewport_dimensions != dimensions
        };
        if changed {
            let mut shell = state.write();
            shell.renderer_health.backend_label = backend;
            shell.renderer_health.fallback_policy_label = policy.into();
            shell.renderer_health.fallback_engaged = fallback_engaged;
            shell.renderer_health.viewport_dimensions = dimensions;
        }
    });
}

#[component]
fn QualiaBackedStage(
    state: Signal<ShellState>,
    adopted: Signal<Option<AdoptedQualiaFoundation>>,
) -> Element {
    let scope = use_hook(|| {
        adopted.peek().clone().map(|foundation| {
            let mut context = QualiaHostContext::new(foundation.authority().clone());
            context.authoring = NativeAuthoringBridge::connect(
                context.authority.clone(),
                foundation.session().clone(),
            )
            .ok();
            (foundation, context)
        })
    });
    if let Some((foundation, context)) = scope {
        provide_context(foundation);
        provide_context(context);
    }
    rsx! { StageHost { state } }
}

#[component]
pub fn App() -> Element {
    let mut state = use_signal(ShellState::default);
    arm_c1_smoke();
    arm_omni_shortcut(state);
    arm_renderer_health(state);
    arm_background_jobs(state);
    let adopted = use_signal(|| None::<AdoptedQualiaFoundation>);
    let has_adopted_scope = adopted.read().is_some();
    let refract_store = use_refract_store(|| {
        session()
            .map(WebizenRefractModel::from_session)
            .unwrap_or_else(WebizenRefractModel::placeholder)
    });
    let bridge = use_context_provider(AuxWindowBridge::new);
    let care_authority = use_hook(|| {
        if let Some(authority) = try_consume_context::<CareResponseAuthority>() {
            return Some(authority);
        }
        session_care_authority().map(provide_context)
    });
    let previous_fabric = use_hook(|| Rc::new(RefCell::new(state.peek().host_stewardship.clone())));
    use_effect({
        let bridge = bridge.clone();
        let refract_store = refract_store.clone();
        let care_authority_available = care_authority.is_some();
        move || {
            let snapshot = state.read();
            bridge.sync_from_state(&snapshot, care_authority_available);
            let mut previous_fabric = previous_fabric.borrow_mut();
            if *previous_fabric != snapshot.host_stewardship {
                refresh_refract_host_records(&refract_store, &snapshot.host_stewardship);
                *previous_fabric = snapshot.host_stewardship.clone();
                drop(snapshot);
                state
                    .write()
                    .os_control_center
                    .sync_notifications_from_stewardship(&previous_fabric);
            }
        }
    });
    arm_window_commands(state, bridge, care_authority);
    let s = state.read();
    let status = if s.current_stage == crate::state::VolumeStage::None {
        format!("{} · {} · 42 MiB Sentinel", s.sku_label, s.gpu_label)
    } else {
        format!(
            "Volume · {} ({})",
            s.current_stage.title(),
            s.current_stage.primary_plane().code()
        )
    };
    let active = s.current_stage.title();
    let launcher_open = s.launcher_open;
    let root_class = if s.reduced_motion {
        format!("webizen-shell-root shell {} reduced-motion", s.theme_preset)
    } else {
        format!("webizen-shell-root shell {}", s.theme_preset)
    };

    rsx! {
        style { "{SHELL_CSS}" }
        div { class: "field", aria_hidden: "true" }
        div {
            class: "{root_class}",
            onresize: move |event| {
                if let Ok(size) = event.data().get_content_box_size() {
                    let dimensions = (size.width, size.height);
                    let changed = state.peek().shell_viewport_size != dimensions;
                    if changed { state.write().shell_viewport_size = dimensions; }
                }
            },

            HaloBar { state }

            if has_adopted_scope {
                QualiaBackedStage { state, adopted }
            } else {
                StageHost { state }
            }

            footer {
                class: "footer",
                role: "contentinfo",
                span { "handle ≠ human · agent = tool · VC = origin/integrity" }
                span { class: "grow" }
                span { id: "shell-status", "{status}" }
                button {
                    r#type: "button",
                    id: "open-launcher",
                    onclick: move |_| {
                        let open = state.read().launcher_open;
                        state.write().launcher_open = !open;
                    },
                    "Launch…"
                }
                span { id: "active-label", "{active}" }
                QualiaAdmissionPanel { adopted }
            }
        }
        if launcher_open {
            OrbitWheel { state }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use webizen_ua_os::{CareAlertSeverity, CareAlertTriggerKind, SharedLivingAndCareTopologyKind};

    #[test]
    fn session_care_grant_is_bound_to_role_fabric_host_circle_and_time() {
        let claims = serde_json::json!({
            "fabric_did": "did:qi:device",
            "host_domain": "care.example",
            "circle_id": "circle:home",
            "responder_did": "did:qi:principal",
            "role": "principal",
            "not_before": 10,
            "expires": 20
        });
        let encoded = claims.to_string();
        let grant = SessionCareGrant::parse(&encoded, "did:qi:device", "care.example", 11)
            .expect("authenticated claims from the owning session");
        assert!(SessionCareGrant::parse(&encoded, "did:qi:other", "care.example", 11).is_none());
        assert!(SessionCareGrant::parse(&encoded, "did:qi:device", "other.example", 11).is_none());
        assert!(SessionCareGrant::parse(&encoded, "did:qi:device", "care.example", 20).is_none());

        let mut alert = DispatchedCareAndGuardianAlert {
            alert_id: "alert:home".into(),
            circle_id: "circle:home".into(),
            topology: SharedLivingAndCareTopologyKind::ElderlyOrSupportedPersonWithRemoteCarers,
            trigger_kind: CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse,
            severity: CareAlertSeverity::Urgent,
            principal_did: "did:qi:principal".into(),
            notified_remote_guardian_dids: vec!["did:qi:carer (Home Carer)".into()],
            summary: String::new(),
            recorded_in_accountability_ledger: false,
            duty_of_inquiry_status: String::new(),
            resolved_or_reversed: false,
            sealed_nquin_parity: 0,
        };
        assert_eq!(
            grant.resolve(&alert, true, 11).as_deref(),
            Ok("did:qi:principal")
        );
        assert!(grant.resolve(&alert, false, 11).is_err());
        assert!(grant.resolve(&alert, true, 20).is_err());

        let carer = SessionCareGrant {
            role: CareGrantRole::Carer,
            responder_did: "did:qi:carer".into(),
            ..grant.clone()
        };
        assert_eq!(
            carer.resolve(&alert, false, 11).as_deref(),
            Ok("did:qi:carer")
        );
        assert!(carer.resolve(&alert, true, 11).is_err());
        alert.circle_id = "circle:somewhere-else".into();
        assert!(grant.resolve(&alert, true, 11).is_err());
    }
}
