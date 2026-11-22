// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Desktop workspace presets and a read-only fleet projection for the native shell.

use crate::{HostNodeLocusKind, NaturalHostStewardshipFabric, OsEnvironment, Sku};

/// A startup workspace available on both the desktop appliance and guest shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopWorkspacePreset {
    SpatialOrbit,
    AuthoringPoetSynth,
    DualWebBrowser,
    CarerStewardConsole,
    QuietSanctuary,
}

impl DesktopWorkspacePreset {
    pub const ALL: [Self; 5] = [
        Self::SpatialOrbit,
        Self::AuthoringPoetSynth,
        Self::DualWebBrowser,
        Self::CarerStewardConsole,
        Self::QuietSanctuary,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::SpatialOrbit => "spatial-orbit",
            Self::AuthoringPoetSynth => "authoring-poet-synth",
            Self::DualWebBrowser => "dual-web-browser",
            Self::CarerStewardConsole => "carer-steward-console",
            Self::QuietSanctuary => "quiet-sanctuary",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::SpatialOrbit => "Spatial Orbit",
            Self::AuthoringPoetSynth => "Poet & Synth Authoring",
            Self::DualWebBrowser => "Dual-Web Browser",
            Self::CarerStewardConsole => "Carer & Steward Console",
            Self::QuietSanctuary => "Quiet Sanctuary",
        }
    }

    pub const fn default_stage_slug(self) -> &'static str {
        match self {
            Self::SpatialOrbit => "none",
            Self::AuthoringPoetSynth => "poet",
            Self::DualWebBrowser => "browser",
            Self::CarerStewardConsole => "wellfair",
            Self::QuietSanctuary => "keep",
        }
    }
}

/// A registered host's advertised capabilities and accountable workload counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocusFleetNodeSummary {
    pub node_id: String,
    pub label: String,
    pub locus: HostNodeLocusKind,
    pub qdnf_ula_ipv6: String,
    pub endpoint_hint: String,
    pub runtime_operator_count: usize,
    pub hosted_project_count: usize,
    pub standalone_without_online_vps: bool,
    pub holds_local_sanctuary_keep: bool,
    pub qcdn_seeder_tier: String,
}

/// A local projection of the registered fabric. It records no liveness claims.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MultiLocusFleetControllerState {
    pub nodes: Vec<LocusFleetNodeSummary>,
    pub hosted_project_count: usize,
    pub standalone_node_count: usize,
    pub has_online_vps: bool,
    pub steward_count: usize,
    pub quorum_threshold_m: u8,
    pub pending_update_count: usize,
    pub open_care_alert_count: usize,
}

/// Stable menu data that a native shell can resolve into a workspace action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrayMenuBindingSpec {
    pub menu_id: &'static str,
    pub label: &'static str,
    pub stage_slug: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesktopApplianceProfile {
    pub sku: Sku,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub workspace_presets: [DesktopWorkspacePreset; 5],
    pub active_workspace: DesktopWorkspacePreset,
    pub fleet: MultiLocusFleetControllerState,
    pub tray_bindings: Vec<TrayMenuBindingSpec>,
}

/// Construct the profile from detected Device OS facts and the existing fabric.
pub fn build_desktop_appliance_profile(
    sku: Sku,
    env: &OsEnvironment,
    fabric: &NaturalHostStewardshipFabric,
) -> DesktopApplianceProfile {
    let workspace_presets = DesktopWorkspacePreset::ALL;
    let tray_bindings = workspace_presets
        .iter()
        .copied()
        .map(|preset| TrayMenuBindingSpec {
            menu_id: preset.code(),
            label: preset.title(),
            stage_slug: preset.default_stage_slug(),
        })
        .collect();
    let mut profile = DesktopApplianceProfile {
        sku,
        hostname: env.hostname.clone(),
        os: env.os.clone(),
        arch: env.arch.clone(),
        workspace_presets,
        active_workspace: DesktopWorkspacePreset::SpatialOrbit,
        fleet: MultiLocusFleetControllerState::default(),
        tray_bindings,
    };
    sync_fleet_from_fabric(&mut profile, fabric);
    profile
}

/// Refresh fleet facts from the local fabric and retain the selected workspace.
pub fn sync_fleet_from_fabric(
    profile: &mut DesktopApplianceProfile,
    fabric: &NaturalHostStewardshipFabric,
) {
    let nodes: Vec<_> = fabric
        .host_fleet_nodes
        .iter()
        .map(|node| LocusFleetNodeSummary {
            node_id: node.node_id.clone(),
            label: node.label.clone(),
            locus: node.locus,
            qdnf_ula_ipv6: node.qdnf_ula_ipv6.clone(),
            endpoint_hint: node.endpoint_hint.clone(),
            runtime_operator_count: node.runtime_operator_steward_dids.len(),
            hosted_project_count: fabric
                .hosted_projects
                .iter()
                .filter(|project| project.hosted_on_runtime_node_ids.contains(&node.node_id))
                .count(),
            standalone_without_online_vps: node.standalone_without_online_vps,
            holds_local_sanctuary_keep: node.holds_local_sanctuary_keep,
            qcdn_seeder_tier: node.qcdn_seeder_tier.clone(),
        })
        .collect();
    profile.fleet = MultiLocusFleetControllerState {
        hosted_project_count: fabric.hosted_projects.len(),
        standalone_node_count: nodes
            .iter()
            .filter(|node| node.standalone_without_online_vps)
            .count(),
        has_online_vps: nodes
            .iter()
            .any(|node| node.locus == HostNodeLocusKind::OnlineCloudVpsRelay),
        nodes,
        steward_count: fabric.stewards.len(),
        quorum_threshold_m: fabric.threshold_m,
        pending_update_count: fabric
            .update_queue
            .iter()
            .filter(|proposal| !proposal.ratified_for_host_apply)
            .count(),
        open_care_alert_count: fabric
            .care_alerts
            .iter()
            .filter(|alert| !alert.resolved_or_reversed)
            .count(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_multi_locus_fleet_and_workspace_presets() {
        let mut fabric = NaturalHostStewardshipFabric::default();
        let env = OsEnvironment {
            hostname: "desk".into(),
            os: "linux".into(),
            arch: "x86_64".into(),
            username: "tester".into(),
            ram_bytes: None,
            cpu_count: None,
            cpu_brand: None,
        };

        let mut profile = build_desktop_appliance_profile(Sku::Appliance, &env, &fabric);
        assert_eq!(profile.sku, Sku::Appliance);
        assert_eq!(profile.hostname, env.hostname);
        assert_eq!(profile.workspace_presets, DesktopWorkspacePreset::ALL);
        assert_eq!(
            profile.tray_bindings.len(),
            DesktopWorkspacePreset::ALL.len()
        );
        assert_eq!(profile.tray_bindings[2].stage_slug, "browser");
        assert_eq!(profile.fleet.nodes.len(), fabric.host_fleet_nodes.len());
        assert!(profile.fleet.has_online_vps);
        assert!(profile.fleet.standalone_node_count > 0);
        for node in &profile.fleet.nodes {
            let workloads = fabric
                .hosted_projects
                .iter()
                .filter(|project| project.hosted_on_runtime_node_ids.contains(&node.node_id))
                .count();
            assert_eq!(node.hosted_project_count, workloads);
        }

        profile.active_workspace = DesktopWorkspacePreset::QuietSanctuary;
        fabric
            .host_fleet_nodes
            .retain(|node| node.locus != HostNodeLocusKind::OnlineCloudVpsRelay);
        sync_fleet_from_fabric(&mut profile, &fabric);
        assert!(!profile.fleet.has_online_vps);
        assert_eq!(
            profile.active_workspace,
            DesktopWorkspacePreset::QuietSanctuary
        );
    }
}
