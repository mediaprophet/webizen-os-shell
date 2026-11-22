// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Mobile AVF observations, local resource policy, and principal check-in.

use crate::{
    ManagedModelResidency, NativeEtalonStatus, NaturalHostStewardshipFabric,
    SENSITIVITY_PUBLIC_ONLY, SENSITIVITY_SANCTUARY_LOCAL_MAX,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

/// Observed guest isolation capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvfIsolationState {
    ProtectedPkvmEl2Active,
    GuestDevSimulation,
    UnsupportedHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VsockTierIsolation {
    Tier1El2SanctuaryLocal,
    Tier2BlindedPublicNpuRpc,
}

/// Endpoint routing policy for a guest with a kernel vsock device.
/// These descriptors contain no listener or channel credentials.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VsockChannelDescriptor {
    pub cid: u32,
    pub port: u32,
    pub tier: VsockTierIsolation,
    pub transport_label: String,
    pub max_sensitivity_allowed: u8,
    pub zero_copy_ahb_gfxstream: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvfSanctuaryProbe {
    pub isolation_state: AvfIsolationState,
    pub pkvm_el2_detected: bool,
    pub pvmfw_dice_present: bool,
    pub anvil_np64_attested: bool,
    pub virtio_gpu_venus_active: bool,
    pub volatile_scrub_on_sleep: bool,
    pub vsock_channels: Vec<VsockChannelDescriptor>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThermalPressureLevel {
    Nominal,
    Fair,
    Serious,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct P64ThermalBatteryGovernor {
    pub thermal_level: ThermalPressureLevel,
    pub battery_percent: u8,
    pub on_external_power: bool,
    pub active_p64_kv_cache_mib: u32,
    pub max_kv_cache_cap_mib: u32,
    pub recommended_residency: ManagedModelResidency,
    pub allow_new_streaming_vram: bool,
    pub scrub_evicted_on_critical: bool,
}

impl P64ThermalBatteryGovernor {
    /// Evaluate the cache cap using measured pressure and available power.
    /// A caller with a live model registry can supply measured active cache usage.
    pub fn evaluate_governor(
        thermal: ThermalPressureLevel,
        battery_percent: u8,
        on_external_power: bool,
    ) -> Self {
        let battery_percent = battery_percent.min(100);
        let thermal_cap: u32 = match thermal {
            ThermalPressureLevel::Nominal => 1024,
            ThermalPressureLevel::Fair => 512,
            ThermalPressureLevel::Serious => 256,
            ThermalPressureLevel::Critical => 128,
        };
        let battery_cap: u32 = match (on_external_power, battery_percent) {
            (true, _) => 1024,
            (false, 0..=15) => 128,
            (false, 16..=30) => 256,
            (false, 31..=59) => 512,
            (false, _) => 768,
        };
        let recommended_residency = match (thermal, on_external_power, battery_percent) {
            (ThermalPressureLevel::Nominal, true, _) => ManagedModelResidency::PinnedInVram,
            (ThermalPressureLevel::Fair, true, _)
            | (ThermalPressureLevel::Nominal | ThermalPressureLevel::Fair, false, 31..=100) => {
                ManagedModelResidency::KeepWarm
            }
            _ => ManagedModelResidency::OnDemand,
        };
        Self {
            thermal_level: thermal,
            battery_percent,
            on_external_power,
            active_p64_kv_cache_mib: 0,
            max_kv_cache_cap_mib: thermal_cap.min(battery_cap),
            recommended_residency,
            allow_new_streaming_vram: thermal <= ThermalPressureLevel::Fair
                && (on_external_power || battery_percent > 30),
            scrub_evicted_on_critical: thermal == ThermalPressureLevel::Critical,
        }
    }
}

/// Paint and input geometry for the 8-Sector Command Orb.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThumbArcOrbSector {
    pub index: u8,
    pub label: &'static str,
    pub glyph: &'static str,
    pub angle_deg_tenths: i16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThumbArcOrbGeometry {
    pub right_handed: bool,
    pub arc_start_deg: i16,
    pub arc_sweep_deg: i16,
    pub radius_dp: u16,
    pub target_hit_dp: u16,
    pub sectors: [ThumbArcOrbSector; 8],
}

impl ThumbArcOrbGeometry {
    /// Mirror the same command order around the vertical axis for either thumb.
    pub fn compute_thumb_arc(right_handed: bool, viewport_width_dp: u32) -> Self {
        const ACTIONS: [(&str, &str); 8] = [
            ("Close", "✕"),
            ("Soften", "—"),
            ("Full", "▢"),
            ("Float", "◎"),
            ("Keep", "📁"),
            ("Share", "↗"),
            ("Inspect", "🔍"),
            ("More", "⋯"),
        ];
        let (arc_start_deg, arc_sweep_deg) = if right_handed {
            (-168, 156)
        } else {
            (-12, -156)
        };
        let sectors = std::array::from_fn(|index| {
            let (label, glyph) = ACTIONS[index];
            ThumbArcOrbSector {
                index: index as u8,
                label,
                glyph,
                angle_deg_tenths: arc_start_deg * 10 + arc_sweep_deg * 10 * index as i16 / 7,
            }
        });
        Self {
            right_handed,
            arc_start_deg,
            arc_sweep_deg,
            radius_dp: (viewport_width_dp.saturating_sub(24) / 2).clamp(1, 132) as u16,
            target_hit_dp: 44,
            sectors,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileOneTapCheckInReceipt {
    pub circle_id: String,
    pub alert_id: String,
    pub principal_did: String,
    pub escalation_reversed: bool,
    pub duty_of_inquiry_status: String,
    pub sealed_nquin_parity: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MobileAvfSanctuaryProfile {
    pub probe: AvfSanctuaryProbe,
    pub governor: P64ThermalBatteryGovernor,
    pub orb_geometry: ThumbArcOrbGeometry,
    pub last_ok_checkin: Option<MobileOneTapCheckInReceipt>,
}

impl Default for MobileAvfSanctuaryProfile {
    fn default() -> Self {
        Self {
            probe: AvfEvidence::default().into_probe(false, None),
            governor: P64ThermalBatteryGovernor::evaluate_governor(
                ThermalPressureLevel::Fair,
                0,
                false,
            ),
            orb_geometry: ThumbArcOrbGeometry::compute_thumb_arc(true, 360),
            last_ok_checkin: None,
        }
    }
}

#[derive(Default)]
struct AvfEvidence {
    avf_guest: bool,
    strict_boot: bool,
    dice_handover: bool,
    vsock_available: bool,
    venus_active: bool,
}

impl AvfEvidence {
    fn into_probe(self, np64_allows_attestation: bool, battery: Option<u8>) -> AvfSanctuaryProbe {
        let pkvm_el2_detected = self.strict_boot && self.dice_handover;
        let anvil_np64_attested = pkvm_el2_detected && np64_allows_attestation;
        let (isolation_state, label) = if pkvm_el2_detected {
            (
                AvfIsolationState::ProtectedPkvmEl2Active,
                "Protected AVF guest boot markers detected",
            )
        } else if self.avf_guest {
            (
                AvfIsolationState::GuestDevSimulation,
                "AVF guest detected without protected boot evidence",
            )
        } else {
            (
                AvfIsolationState::UnsupportedHost,
                "No local AVF guest evidence",
            )
        };
        let mut vsock_channels = Vec::new();
        if self.vsock_available {
            if anvil_np64_attested {
                vsock_channels.push(VsockChannelDescriptor {
                    cid: 1,
                    port: 4242,
                    tier: VsockTierIsolation::Tier1El2SanctuaryLocal,
                    transport_label: "Guest-local Sanctuary (vsock CID_LOCAL)".into(),
                    max_sensitivity_allowed: SENSITIVITY_SANCTUARY_LOCAL_MAX,
                    zero_copy_ahb_gfxstream: false,
                });
            }
            vsock_channels.push(VsockChannelDescriptor {
                cid: 2,
                port: 4243,
                tier: VsockTierIsolation::Tier2BlindedPublicNpuRpc,
                transport_label: "Host NPU RPC (public workloads only)".into(),
                max_sensitivity_allowed: SENSITIVITY_PUBLIC_ONLY,
                zero_copy_ahb_gfxstream: false,
            });
        }
        let attestation = if anvil_np64_attested {
            "Anvil vault attestation gate satisfied"
        } else {
            "Anvil vault attestation unavailable"
        };
        let mut summary = format!("{label} · {attestation}");
        if battery.is_none() {
            summary.push_str(" · Battery telemetry unavailable");
        }
        AvfSanctuaryProbe {
            isolation_state,
            pkvm_el2_detected,
            pvmfw_dice_present: self.dice_handover,
            anvil_np64_attested,
            virtio_gpu_venus_active: self.venus_active,
            // Admission of a protected guest does not observe a sleep-time scrub.
            volatile_scrub_on_sleep: false,
            vsock_channels,
            summary,
        }
    }
}

/// Probe local protected boot markers. Host configuration and environment hints
/// do not establish the kernel-protected guest state.
pub fn probe_mobile_avf_sanctuary(gpu: &NativeEtalonStatus) -> MobileAvfSanctuaryProfile {
    let mut evidence = AvfEvidence::default();
    if cfg!(all(
        target_arch = "aarch64",
        any(target_os = "linux", target_os = "android")
    )) {
        // Android AVF exports the strict-boot flag on protected guests. The
        // pvmfw DICE handover is a reserved-memory node with this compatible
        // string. Probe metadata only: the DICE handover contains secrets.
        let device_tree = Path::new("/sys/firmware/devicetree/base");
        evidence.strict_boot = device_tree.join("chosen/avf,strict-boot").is_file();
        evidence.dice_handover =
            directory_entries(&device_tree.join("reserved-memory")).any(|entry| {
                read_small(&entry.path().join("compatible")).is_some_and(|data| {
                    data.split(|b| *b == 0)
                        .any(|tag| tag == b"google,open-dice")
                })
            });
        evidence.avf_guest = evidence.strict_boot
            || evidence.dice_handover
            || device_tree.join("avf/untrusted/instance-id").is_file();
        evidence.vsock_available = evidence.avf_guest
            && (Path::new("/dev/vsock").exists() || Path::new("/sys/class/misc/vsock").exists());
        evidence.venus_active = evidence.avf_guest
            && directory_entries(Path::new("/sys/bus/virtio/drivers/virtio_gpu"))
                .any(|entry| entry.file_name().to_string_lossy().starts_with("virtio"))
            && gpu.adapters.iter().any(|adapter| {
                adapter.backend.eq_ignore_ascii_case("vulkan")
                    && [&adapter.name, &adapter.driver, &adapter.driver_info]
                        .iter()
                        .any(|value| value.to_ascii_lowercase().contains("venus"))
            });
    }
    let (battery, on_external_power) = probe_power_supply();
    MobileAvfSanctuaryProfile {
        probe: evidence.into_probe(gpu.may_advertise_hardware_attested_vault, battery),
        governor: P64ThermalBatteryGovernor::evaluate_governor(
            probe_thermal_pressure(),
            battery.unwrap_or(0),
            on_external_power,
        ),
        orb_geometry: ThumbArcOrbGeometry::compute_thumb_arc(true, 360),
        last_ok_checkin: None,
    }
}

fn directory_entries(path: &Path) -> impl Iterator<Item = fs::DirEntry> {
    fs::read_dir(path).into_iter().flatten().take(64).flatten()
}

fn read_small(path: &Path) -> Option<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)
        .ok()?
        .take(4097)
        .read_to_end(&mut data)
        .ok()?;
    (data.len() <= 4096).then_some(data)
}

fn read_fact(path: &Path) -> Option<String> {
    String::from_utf8(read_small(path)?)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn probe_power_supply() -> (Option<u8>, bool) {
    let mut battery: Option<u8> = None;
    let mut external_power = false;
    if cfg!(any(target_os = "linux", target_os = "android")) {
        for supply in directory_entries(Path::new("/sys/class/power_supply")) {
            let path = supply.path();
            match read_fact(&path.join("type")).as_deref() {
                Some("Battery") => {
                    if let Some(capacity) = read_fact(&path.join("capacity"))
                        .and_then(|value| value.parse::<u8>().ok())
                        .filter(|percent| *percent <= 100)
                    {
                        battery = Some(battery.map_or(capacity, |current| current.min(capacity)));
                    }
                }
                Some("Mains" | "USB" | "USB_C" | "USB_PD" | "USB_PD_DRP" | "Wireless") => {
                    external_power |= read_fact(&path.join("online")).as_deref() == Some("1");
                }
                _ => {}
            }
        }
    }
    (battery, external_power)
}

fn probe_thermal_pressure() -> ThermalPressureLevel {
    let mut highest = None;
    if cfg!(any(target_os = "linux", target_os = "android")) {
        for zone in directory_entries(Path::new("/sys/class/thermal")).filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("thermal_zone")
        }) {
            let path = zone.path();
            let Some(temperature) =
                read_fact(&path.join("temp")).and_then(|value| value.parse::<i32>().ok())
            else {
                continue;
            };
            for trip in 0..32 {
                let pressure =
                    match read_fact(&path.join(format!("trip_point_{trip}_type"))).as_deref() {
                        Some("critical") => ThermalPressureLevel::Critical,
                        Some("hot") => ThermalPressureLevel::Serious,
                        Some("active" | "passive") => ThermalPressureLevel::Fair,
                        _ => continue,
                    };
                if let Some(limit) = read_fact(&path.join(format!("trip_point_{trip}_temp")))
                    .and_then(|value| value.parse::<i32>().ok())
                {
                    let level = if temperature >= limit {
                        pressure
                    } else {
                        ThermalPressureLevel::Nominal
                    };
                    highest =
                        Some(highest.map_or(level, |prior: ThermalPressureLevel| prior.max(level)));
                }
            }
        }
    }
    highest.unwrap_or(ThermalPressureLevel::Fair)
}

/// Reverse the named escalation for its own principal and return the ledger receipt.
pub fn trigger_one_tap_principal_ok_checkin(
    fabric: &mut NaturalHostStewardshipFabric,
    storage: &Path,
    alert_id: &str,
    principal_did: &str,
    note: Option<&str>,
) -> Result<MobileOneTapCheckInReceipt, String> {
    let was_escalated = {
        let alert = fabric
            .care_alerts
            .iter()
            .find(|alert| alert.alert_id == alert_id.trim())
            .ok_or_else(|| "Care alert is unavailable".to_string())?;
        let circle = fabric
            .shared_care_circles
            .iter()
            .find(|circle| circle.circle_id == alert.circle_id)
            .ok_or_else(|| "Care circle is unavailable".to_string())?;
        let principal = principal_did.trim();
        if principal.is_empty()
            || principal != circle.principal_did
            || principal != alert.principal_did
        {
            return Err("Only the care circle principal can confirm this check-in".into());
        }
        if alert.resolved_or_reversed {
            return Err("This care alert is already resolved".into());
        }
        circle.escalation_currently_active
    };
    let alert = fabric.acknowledge_or_reverse_care_alert(
        storage,
        alert_id,
        principal_did,
        true,
        note.map(str::trim)
            .filter(|note| !note.is_empty())
            .unwrap_or("I am OK"),
    )?;
    let still_escalated = fabric
        .shared_care_circles
        .iter()
        .any(|circle| circle.circle_id == alert.circle_id && circle.escalation_currently_active);
    Ok(MobileOneTapCheckInReceipt {
        circle_id: alert.circle_id,
        alert_id: alert.alert_id,
        principal_did: alert.principal_did,
        escalation_reversed: was_escalated && !still_escalated && alert.resolved_or_reversed,
        duty_of_inquiry_status: alert.duty_of_inquiry_status,
        sealed_nquin_parity: alert.sealed_nquin_parity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CareAlertSeverity, CareAlertTriggerKind, HOST_STEWARDSHIP_STORE_REL_PATH};

    #[test]
    fn governor_scales_kv_cache_and_one_tap_checkin_reverses_care_escalation() {
        let nominal =
            P64ThermalBatteryGovernor::evaluate_governor(ThermalPressureLevel::Nominal, 90, true);
        let critical =
            P64ThermalBatteryGovernor::evaluate_governor(ThermalPressureLevel::Critical, 90, true);
        assert!(nominal.max_kv_cache_cap_mib > critical.max_kv_cache_cap_mib);
        assert_eq!(critical.max_kv_cache_cap_mib, 128);
        assert_eq!(
            critical.recommended_residency,
            ManagedModelResidency::OnDemand
        );
        assert!(!critical.allow_new_streaming_vram);
        assert!(critical.scrub_evicted_on_critical);

        let output = std::env::temp_dir().join(format!(
            "webizen-mobile-checkin-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut fabric = NaturalHostStewardshipFabric::default();
        let circle_id = "circle:elderly-home-remote-carers";
        let alert = fabric
            .dispatch_shared_care_or_guardian_alert(
                &output,
                circle_id,
                CareAlertTriggerKind::ElderlyLivenessHeartbeatLapse,
                CareAlertSeverity::Urgent,
                "A scheduled check-in elapsed",
            )
            .unwrap();

        assert!(trigger_one_tap_principal_ok_checkin(
            &mut fabric,
            &output,
            &alert.alert_id,
            "did:qi:unrelated",
            None,
        )
        .is_err());
        assert!(
            fabric
                .shared_care_circles
                .iter()
                .find(|circle| circle.circle_id == circle_id)
                .unwrap()
                .escalation_currently_active
        );

        let receipt = trigger_one_tap_principal_ok_checkin(
            &mut fabric,
            &output,
            &alert.alert_id,
            &alert.principal_did,
            None,
        )
        .unwrap();
        assert!(receipt.escalation_reversed);
        assert_eq!(receipt.circle_id, circle_id);
        assert_eq!(receipt.principal_did, alert.principal_did);
        assert!(receipt.duty_of_inquiry_status.contains("I am OK"));
        assert_ne!(receipt.sealed_nquin_parity, 0);
        let saved: NaturalHostStewardshipFabric = serde_json::from_slice(
            &fs::read(output.join(HOST_STEWARDSHIP_STORE_REL_PATH)).unwrap(),
        )
        .unwrap();
        assert!(
            !saved
                .shared_care_circles
                .iter()
                .find(|circle| circle.circle_id == circle_id)
                .unwrap()
                .escalation_currently_active
        );
        assert!(
            saved
                .care_alerts
                .iter()
                .find(|item| item.alert_id == receipt.alert_id)
                .unwrap()
                .resolved_or_reversed
        );
        fs::remove_dir_all(output).unwrap();
    }

    #[test]
    fn mobile_attestation_requires_protected_boot_and_the_anvil_gate() {
        let weak = AvfEvidence {
            avf_guest: true,
            vsock_available: true,
            ..AvfEvidence::default()
        }
        .into_probe(true, None);
        assert_eq!(weak.isolation_state, AvfIsolationState::GuestDevSimulation);
        assert!(!weak.anvil_np64_attested);
        assert!(weak
            .vsock_channels
            .iter()
            .all(|channel| channel.max_sensitivity_allowed == SENSITIVITY_PUBLIC_ONLY));

        for np64_allowed in [false, true] {
            let protected = AvfEvidence {
                strict_boot: true,
                dice_handover: true,
                vsock_available: true,
                ..AvfEvidence::default()
            }
            .into_probe(np64_allowed, Some(80));
            assert_eq!(
                protected.isolation_state,
                AvfIsolationState::ProtectedPkvmEl2Active
            );
            assert_eq!(protected.anvil_np64_attested, np64_allowed);
            assert_eq!(
                protected
                    .vsock_channels
                    .iter()
                    .any(|channel| channel.tier == VsockTierIsolation::Tier1El2SanctuaryLocal),
                np64_allowed
            );
        }
    }

    #[test]
    fn thumb_arc_mirrors_coordinates_and_keeps_command_order() {
        let right = ThumbArcOrbGeometry::compute_thumb_arc(true, 360);
        let left = ThumbArcOrbGeometry::compute_thumb_arc(false, 360);
        assert_eq!(right.target_hit_dp, 44);
        assert_eq!(right.radius_dp, left.radius_dp);
        for (r, l) in right.sectors.iter().zip(&left.sectors) {
            assert_eq!(r.label, l.label);
            assert_eq!(r.angle_deg_tenths + l.angle_deg_tenths, -1800);
        }
    }
}
