// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::{
    admin::{build_appliance_packages, ApplianceProfilesPanel, HostDaemonPanel},
    llm_agents::LlmAndAgentManagementPanel,
    theme_studio::ThemeStudio,
};
use crate::state::{session, SettingsSection, ShellState, ALL_SETTINGS_SECTIONS};
use aether::{aether_core::try_consume_context, prelude::*};
use webizen_ua_os::{
    export_semantic_entry_to_solid_turtle, export_shamir_q42j_backup,
    generate_dual_sku_delivery_artifacts, issue_for_requestor, load_shamir_q42j_backup,
    qap_lease_status, run_local_model_readiness_probe, run_updater, save_workspace_snapshot,
    verify_and_restore_shamir_q42j_backup, AdoptedQualiaFoundation, Ask, CapabilityLeaseScope,
    CommonsShareScope, OsJobInput, OsJobKind, OsJobRunner, OsJobStatus, QapLeaseStatus, Requestor,
    UpdateAction, CO_ATTESTATION_BUNDLE_FORMULA, DID_Q42_QRC_LABEL, PLANE_CLAIM_OPINION_TITLE,
    PLANE_INSTRUMENT_TITLE, PLANE_NATURAL_AGENT_TITLE, PLANE_SPATIOTEMPORAL_HANDLE_TITLE,
    VC_ORIGIN_INTEGRITY_DISCLAIMER,
};

/// Canonical settings and environment setup shell.
#[component]
pub fn SettingsVolume(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let active_sec = s.settings_section;
    let query = s.settings_search.clone();
    let query_lower = query.trim().to_ascii_lowercase();
    drop(s);

    let visible_sections: Vec<SettingsSection> = ALL_SETTINGS_SECTIONS
        .iter()
        .copied()
        .filter(|sec| {
            query_lower.is_empty()
                || sec.label().to_ascii_lowercase().contains(&query_lower)
                || sec.search_terms().contains(&query_lower)
        })
        .collect();

    rsx! {
        div {
            class: "settings-shell",
            aside {
                class: "settings-sidebar",
                aria_label: "Settings categories",
                h4 { "Your Webizen" }
                input {
                    class: "settings-search",
                    r#type: "search",
                    value: "{query}",
                    placeholder: "Find a setting…",
                    aria_label: "Find a setting",
                    oninput: move |evt| state.write().settings_search = evt.value(),
                }
                for sec in visible_sections {
                    {
                        let is_sel = active_sec == sec;
                        let tab_class = if is_sel { "settings-tab active" } else { "settings-tab" };
                        let label = sec.label();
                        rsx! {
                            button {
                                key: "{label}",
                                r#type: "button",
                                class: "{tab_class}",
                                onclick: move |_| state.write().settings_section = sec,
                                span { "{label}" }
                            }
                        }
                    }
                }
            }
            div {
                class: "settings-content",
                SettingsSectionBody { section: active_sec, state }
            }
        }
    }
}

#[component]
fn SettingsSectionBody(section: SettingsSection, mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let sku = s.sku_label.clone();
    let os_user = s.os_username.clone();
    let storage = s.storage_display.clone();
    let daemon = s.daemon_label.clone();
    let host_link = s.host_link_label.clone();
    let gpu = s.gpu_label.clone();
    let attested = s.may_advertise_attested_vault;
    let fabric = s.fabric_did.clone();
    let validation = s.validation_summary.clone();
    let ids = s.identifier_lines.clone();
    let adapter_lines: Vec<String> = s
        .gpu_adapters
        .iter()
        .map(|a| {
            format!(
                "{} · {} ({}) · PCI {:04x}:{:04x} · driver {} ({})",
                a.name, a.backend, a.device_type, a.vendor_id, a.device_id, a.driver, a.driver_info
            )
        })
        .collect();
    let qap_lines: Vec<String> = s
        .installed_qaps
        .iter()
        .map(|q| {
            let hash_short = &q.content_hash[..16.min(q.content_hash.len())];
            format!(
                "{} · v{} · sha3={} · revoked={}",
                q.package_id, q.active_version, hash_short, q.revoked
            )
        })
        .collect();
    let llm_models = s.llm_models.clone();
    let selected_model = s.selected_model_name.clone();
    let active_model = s.active_model_name.clone();
    let probe_receipt = s.model_probe_receipt.clone();
    let issued = if s.sanctum_issued.is_empty() {
        "None issued this session".to_string()
    } else {
        s.sanctum_issued.clone()
    };
    let updater_status = s.updater_status.clone();
    let delivery_status = s.delivery_status.clone();
    let host_activity = s.host_stewardship_toast.clone();
    let theme_preset = s.theme_preset;
    let reduced_motion = s.reduced_motion;
    drop(s);

    match section {
        SettingsSection::Health => {
            let model_status = if active_model.is_empty() {
                "Review · Choose & test a local .p64 or .gguf instrument in tab 3".to_string()
            } else {
                format!("Ready · Active local instrument: {active_model}")
            };
            rsx! {
                div {
                    header {
                        class: "settings-head",
                        h3 { "Setup health" }
                        p { "A living scorecard for this computer: files, graphics, local helpers, and network paths when you connect with people you know." }
                    }
                    div {
                        class: "plane-grid",
                        div { class: "plane-card", h5 { "Data Home [Ready]" } p { "{storage} · Private files, library items, and 3D models on this computer" } }
                        div { class: "plane-card", h5 { "Native Etalon GPU" } p { "{gpu} · TargetProfileMask::NATIVE_DESKTOP · 11 providers" } }
                        div { class: "plane-card", h5 { "Webizen Sentinel [Ready]" } p { "42 MiB Zero-Heap SlgArena · 917,504 48-byte NQuin48 slots" } }
                        div { class: "plane-card", h5 { "Four-Plane Fabric [Ready]" } p { "{validation}" } }
                        div { class: "plane-card", h5 { "Local AI Instrument" } p { "{model_status}" } }
                        div { class: "plane-card", h5 { "QualiaDB Sidecar (:4242)" } p { "{daemon}" } }
                    }
                    RendererHealthPanel { state }
                }
            }
        }
        SettingsSection::Data => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "Data & files" }
                    p { "Private records, library files, and shared community material each live in their own place under your data home." }
                }
                dl {
                    class: "kv",
                    dt { "Data Home Root" }
                    dd { class: "path", "{storage}" }
                    dt { "42 MiB Prolog Sentinel" }
                    dd { "Strictly governs .q42 symbolic reasoning hot-paths (FixedArena zero-heap evaluation)" }
                    dt { "Production16 Budget" }
                    dd { "128 MiB per .10d ring · 2 GiB aggregate resources · 2,304 MiB .hmc package budget" }
                }
                section {
                    class: "id-list",
                    h4 { "Canonical Container Strata Under {storage}" }
                    ul {
                        li { ".q42 (Q42\\0 v3/v4) — 48-byte NQuin records, Q42LEX lexicon, BIDX indexes, 40 KiB LZ4 QualiaSuperBlocks" }
                        li { ".10d (10d\\0 v2) — Mandatory HOLO (0x0306) phase-space SDFs, 7-layer L0-L6 MSTR hierarchy, AREF/IOVR/INST" }
                        li { ".hmc (QBDL v1) — Page-aligned transparent hypermedia bundle with appended CBOR index & SHA-256 digests" }
                        li { ".m64 (Codec 3 BLAKE3) · .emf (Q42EMF64 v2) · .qac (QUALIA-FIELD-INDEX-v1) · .p64 (v4 16 KiB AI weights)" }
                        li { "QECP (.pdf) — C2PA JUMBF-in-PDF carrying CBOR-LD W3C VC and 48-byte NQuin + ML-DSA-65 native credentials" }
                    }
                }
            }
        },
        SettingsSection::Models => {
            let model_count = llm_models.len();
            rsx! {
                div {
                    header {
                        class: "settings-head",
                        h3 { "AI helpers" }
                        p { "Models are instruments under your control, never people. Convert discovered .gguf models into 16 KiB page-aligned .p64 v4 vaults and remove the legacy .gguf files; call models as Governed Agents across the environment; and manage External LLM Platform Accounts, Deontic Egress Rules, and MCP Interfaces." }
                    }
                    LlmAndAgentManagementPanel { state }
                    p { class: "muted", "Raw Vault Scan: Discovered {model_count} files across QUALIA_LLM_VAULT, C:\\LLM_files, C:\\LLM_Models, and {storage}\\Models." }
                    if let Some(rec) = probe_receipt {
                        {
                            let summary = rec.summary;
                            rsx! {
                                p { class: "path", "{summary}" }
                            }
                        }
                    }
                    section {
                        class: "id-list",
                        h4 { "Low-Level Header Magic & Sentinel Readiness Probe" }
                        ul {
                            for m in llm_models.iter().take(12) {
                                {
                                    let m_clone = m.clone();
                                    let m_name = m.name.clone();
                                    let is_active = active_model == m.name;
                                    let status_tag = if is_active { "ACTIVE" } else { "AVAILABLE" };
                                    let row_text = format!(
                                        "[{}] {} · {} · {} MiB ({})",
                                        status_tag,
                                        m.name,
                                        m.format,
                                        m.size_mib,
                                        m.path.display()
                                    );
                                    rsx! {
                                        li {
                                            key: "{m_name}",
                                            div {
                                                "{row_text}"
                                                div {
                                                    button {
                                                        r#type: "button",
                                                        class: "sanctum-issue",
                                                        onclick: move |_| {
                                                            let receipt = run_local_model_readiness_probe(&m_clone);
                                                            let mut s = state.write();
                                                            s.selected_model_name = m_clone.name.clone();
                                                            s.active_model_name = m_clone.name.clone();
                                                            s.model_probe_receipt = Some(receipt);
                                                        },
                                                        "Run Low-Level Magic & Sentinel Probe"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    p { class: "muted", "Currently selected: {selected_model}" }
                }
            }
        }
        SettingsSection::People => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "People & network" }
                    p { "Separates the Who-Plane (NaturalAgent) from the Handle-Plane (DNI / DNS / IP), with post-quantum hybrid transport and W3C Solid Pod portability." }
                }
                section {
                    class: "id-list",
                    h4 { "Active Transport, Privacy Routing & Portability Stack" }
                    ul {
                        li { "QDNF (QRoute / QResolve / QPolicy): Ed25519 -> ULA IPv6 fd00::/8 with deontic policy labels" }
                        li { "O(1) NQuin DNS Cache Ring & Native Linux eBPF Per-Sandbox Egress Firewall" }
                        li { "Multi-Transport P2P: Userspace WireGuard, CSCP over HTTP/2 & WSS, Nym Mixnet, and Acoustic/BLE SonicToken Mesh" }
                        li { "Post-Quantum Hybrid Crypto: ML-KEM (Kyber) + X25519 key exchange and dual Ed25519 + ML-DSA-65 signatures" }
                        li { "W3C Solid / LDP Off-Ramp (web.civics.au / Solid-CSS-Databox): Bidirectional Turtle/RDF <-> 48-byte NQuin bridge" }
                    }
                }
                SettingsSolidExport { state }
            }
        },
        SettingsSection::Privacy => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "Privacy & sanctuary" }
                    p { "Constant-shape 4-layer CBOR vault (sanctuary_vault.cbor: Real, Decoy, and Reserved slots) with Argon2id (64 MiB, 3 passes) + AES-256-GCM, one-way key hierarchy (Real -> Decoy), and tamper-evident Sanctuary Audit DAG." }
                }
                dl {
                    class: "kv",
                    dt { "Anvil NP64 Hardware Gate" }
                    dd {
                        if attested {
                            "OPEN (IOMMU + non-zero GPU deviceUUID + Tier <= 1 verified)"
                        } else {
                            "CLOSED (OS-user isolation; does not advertise hardware-attested vault on this host)"
                        }
                    }
                    dt { "Last Contextual DID" }
                    dd { class: "path", "{issued}" }
                }
                button {
                    r#type: "button",
                    class: "sanctum-issue",
                    onclick: move |_| {
                        if let Some(sess) = session() {
                            let reply = issue_for_requestor(
                                &sess.identifiers,
                                &Requestor {
                                    ask: Ask::Master,
                                    requestor: "settings-privacy".into(),
                                    context: "sanctuary-audit".into(),
                                },
                            );
                            state.write().sanctum_issued = format!(
                                "{} · master refused={} · ctx={} · NQuin48 parity={:#018x}",
                                reply.issued_did,
                                reply.refused_master,
                                reply.bound_context,
                                reply.quin_parity
                            );
                        }
                    },
                    "Test Contextual Sanctuary DID Issuance (Refuse Master Ask)"
                }
                section {
                    class: "id-list",
                    h4 { "ADR 0011 Anti-Erasure, Anti-Retaliation & Incapacity Protections" }
                    ul {
                        li { "D1/D2 Encrypted Commons Anti-Erasure & Crypto-Shredding Revocation (ConsentCredential key destruction)" }
                        li { "D3 Checks on Statutory/Court Authority (requires participating party instigation + multi-signature threshold)" }
                        li { "D4/D5 Durable ConductRecord & Staff-Level Retaliatory Leak Attribution (TransparencyCc / trace_leak)" }
                        li { "D6/D7 Reversible Quorum Dead-Man Switch & Anti-Discrediting Incapacity Switch (TransparencyInvocation)" }
                    }
                }
            }
        },
        SettingsSection::Appearance => {
            let (ui_scale_percent, animate_icons, icon_vibe_formula) = {
                let shell = state.read();
                (
                    shell.ui_scale_percent,
                    shell.animate_icons,
                    shell.icon_vibe_formula.clone(),
                )
            };
            let omega = crate::components::eval_vibe_icon_omega(&icon_vibe_formula);
            let motion_label = if reduced_motion {
                "Reduced Motion: ON (3D transforms & icon animations paused)"
            } else if animate_icons {
                "Reduced Motion: OFF · Etalon 3D/Vector Icons: LIVE"
            } else {
                "Reduced Motion: OFF · Etalon 3D/Vector Icons: STATIC"
            };
            rsx! {
                div {
                    header {
                        class: "settings-head",
                        h3 { "Appearance & Access" }
                        p { "Choose a look, set text size, and answer a short comfort questionnaire. Suggested looks stay on this device until you turn one on." }
                    }
                    section {
                        class: "id-list",
                        h4 { "Text & Interface Scale (Live Native Viewport Zoom: {ui_scale_percent}%)" }
                        p { class: "muted", "Scales all typography, controls, and spatial cards across the entire native window via Blitz Viewport::zoom and Parley font re-shaping." }
                        div {
                            class: "input-row",
                            button {
                                r#type: "button",
                                class: "sanctum-issue",
                                onclick: move |_| {
                                    let next = state.read().ui_scale_percent.saturating_sub(10).max(85);
                                    state.write().ui_scale_percent = next;
                                },
                                "A− Zoom Out (−10%)"
                            }
                            button {
                                r#type: "button",
                                class: if ui_scale_percent == 100 { "sanctum-issue" } else { "ribbon-btn" },
                                onclick: move |_| state.write().ui_scale_percent = 100,
                                "100% (Compact)"
                            }
                            button {
                                r#type: "button",
                                class: if ui_scale_percent == 115 { "sanctum-issue" } else { "ribbon-btn" },
                                onclick: move |_| state.write().ui_scale_percent = 115,
                                "115% (Default Comfortable)"
                            }
                            button {
                                r#type: "button",
                                class: if ui_scale_percent == 125 { "sanctum-issue" } else { "ribbon-btn" },
                                onclick: move |_| state.write().ui_scale_percent = 125,
                                "125% (Large)"
                            }
                            button {
                                r#type: "button",
                                class: if ui_scale_percent == 140 { "sanctum-issue" } else { "ribbon-btn" },
                                onclick: move |_| state.write().ui_scale_percent = 140,
                                "140% (Extra Large)"
                            }
                            button {
                                r#type: "button",
                                class: if ui_scale_percent == 160 { "sanctum-issue" } else { "ribbon-btn" },
                                onclick: move |_| state.write().ui_scale_percent = 160,
                                "160% (Max Accessibility)"
                            }
                            button {
                                r#type: "button",
                                class: "sanctum-issue",
                                onclick: move |_| {
                                    let next = (state.read().ui_scale_percent + 10).min(175);
                                    state.write().ui_scale_percent = next;
                                },
                                "A+ Zoom In (+10%)"
                            }
                        }
                    }
                    section {
                        class: "id-list",
                        h4 { "Etalon 3D / Vector Phase-Space Icons & VibeScript Kinetics (ω = {omega:.2} rad/s)" }
                        p { class: "muted", "Icons render directly into the shared Etalon GPU scene via aether_native::EtalonVectorWidget and etalon_vibe (oracle_project_10d_to_3d, oracle_spring_damper_force, oracle_verlet_step, oracle_sh_irradiance_probe)." }
                        div {
                            class: "input-row",
                            button {
                                r#type: "button",
                                class: "sanctum-issue",
                                onclick: move |_| {
                                    let mut s = state.write();
                                    s.animate_icons = !s.animate_icons;
                                    if s.animate_icons {
                                        s.reduced_motion = false;
                                    }
                                },
                                if animate_icons && !reduced_motion {
                                    "Pause Animated 3D Icons"
                                } else {
                                    "Enable Animated 3D Icons"
                                }
                            }
                            button {
                                r#type: "button",
                                class: "sanctum-issue",
                                onclick: move |_| {
                                    let cur = state.read().reduced_motion;
                                    state.write().reduced_motion = !cur;
                                },
                                "Toggle Reduced Motion"
                            }
                        }
                        div {
                            class: "input-row",
                            label {
                                "VibeScript Icon Kinetic Frequency Cell (rad/s)"
                                input {
                                    class: "input-field",
                                    value: "{icon_vibe_formula}",
                                    oninput: move |evt| state.write().icon_vibe_formula = evt.value(),
                                }
                            }
                            button {
                                r#type: "button",
                                class: "ribbon-btn",
                                onclick: move |_| state.write().icon_vibe_formula = "= 0.85 * 1.0".into(),
                                "Gentle (= 0.85 * 1.0)"
                            }
                            button {
                                r#type: "button",
                                class: "ribbon-btn",
                                onclick: move |_| state.write().icon_vibe_formula = "= 1.25 * 1.6".into(),
                                "Default (= 1.25 * 1.6)"
                            }
                            button {
                                r#type: "button",
                                class: "ribbon-btn",
                                onclick: move |_| state.write().icon_vibe_formula = "= 2.0 * 1.75".into(),
                                "Harmonic Burst (= 2.0 * 1.75)"
                            }
                        }
                    }
                    ThemeStudio { state }
                    p { class: "muted", "Active look: {theme_preset} · UI scale: {ui_scale_percent}% · {motion_label}" }
                    WorkspaceSave { state }
                }
            }
        }
        SettingsSection::Device => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "This computer" }
                    p { "Identifiers are never human identity. Living beings, claims, spacetime/route coordinates, and cryptographic instruments remain strictly unmerged across all four planes." }
                }
                ApplianceProfilesPanel { state }
                div {
                    class: "plane-grid",
                    div {
                        class: "plane-card",
                        h5 { "Plane 1: {PLANE_NATURAL_AGENT_TITLE}" }
                        p { "Framed strictly as living-SHACL (D_living), disjoint from owl:Thing. Never labeled Thing, Object, or DID. OS account ({os_user}) is not principal identity." }
                    }
                    div {
                        class: "plane-card",
                        h5 { "Plane 2: {PLANE_CLAIM_OPINION_TITLE}" }
                        p { "Bounded validity window. CoAttestationBundle = \"{CO_ATTESTATION_BUNDLE_FORMULA}\". VC: \"{VC_ORIGIN_INTEGRITY_DISCLAIMER}\"." }
                    }
                    div {
                        class: "plane-card",
                        h5 { "Plane 3: {PLANE_SPATIOTEMPORAL_HANDLE_TITLE}" }
                        p { "Ephemeral DNI route epochs, 16-byte CB-USRI cosmic/geospatial coordinates, and did:q42 = {DID_Q42_QRC_LABEL} ({fabric})." }
                    }
                    div {
                        class: "plane-card",
                        h5 { "Plane 4: {PLANE_INSTRUMENT_TITLE}" }
                        p { "Scoped cryptographic tools: did:qi, directional pairwise locators (A->B != B->A), WalletShape, MachineDeviceId, SensorId, and BiometricFamily vs BiometricInstance." }
                    }
                }
                section {
                    class: "id-list",
                    h4 { "QualiaDB Enumerated Identity State ({validation})" }
                    ul {
                        for line in ids {
                            li { "{line}" }
                        }
                    }
                }
            }
        },
        SettingsSection::Backup => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "Backup & recovery" }
                    p { "Recover encrypted public metadata and export eligible Library records as local Turtle files." }
                }
                section {
                    class: "id-list",
                    h4 { "Recovery & Portability Mechanisms" }
                    ul {
                        li { ".q42j Client-Encrypted Journals: Shamir secret-shared backups & Two-Loop Permissive Commons value-flow logs" }
                        li { ".qnf (QNFNET01): Immutable network evidence containers with post-quantum ML-DSA/SLH-DSA proofs" }
                        li { "Person Transfer Bundle (qualia.person.transfer.v1): Re-bind principal stewardship across your apparatus fleet" }
                        li { "W3C Solid / LDP Databox export: portable RDF Turtle for supported Library metadata" }
                    }
                }
                BackupRecoveryPanel { state }
                SettingsSolidExport { state }
            }
        },
        SettingsSection::Services => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "Services & updates" }
                    p { "Manages the local QualiaDB sidecar, atomic Qap installation, Synth-style Ed25519 self-updates (C7), and Dual-SKU packaging (SKU A Guest Installer + SKU B Debian 13 Trixie Overlay)." }
                }
                dl {
                    class: "kv",
                    dt { "Active SKU" }
                    dd { "{sku}" }
                    dt { "QualiaDB Sidecar" }
                    dd { "{daemon}" }
                    dt { "Host Link" }
                    dd { "{host_link}" }
                }
                BranchingOrchestratorPanel { state }
                OsJobQueue { state }
                QapLeaseMatrix { state }
                HostDaemonPanel { state }
                if !host_activity.is_empty() {
                    p { class: "path", role: "status", "{host_activity}" }
                }
                div {
                    button {
                        r#type: "button",
                        onclick: move |_| build_appliance_packages(state),
                        "Build Desktop, Mobile AVF and Host Debian Archives"
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                let rec = run_updater(&sess.host, sess.sku, UpdateAction::Check, None);
                                state.write().updater_status = rec.detail;
                            }
                        },
                        "Check Ed25519 Self-Updater (C7)"
                    }
                    button {
                        r#type: "button",
                        class: "sanctum-issue",
                        onclick: move |_| {
                            if let Some(sess) = session() {
                                match generate_dual_sku_delivery_artifacts(&sess.host, &sess.gpu) {
                                    Ok(rec) => {
                                        state.write().delivery_status = format!(
                                            "Generated SKU A Guest Bundle ({}) + SKU B Debian 13 Trixie Overlay ({} · units: {}) + Three-Ring LICENSE-BOUNDARY ({})",
                                            rec.guest_manifest_path.display(),
                                            rec.debian_overlay_dir.display(),
                                            rec.systemd_units.join(", "),
                                            rec.license_boundary_path.display()
                                        );
                                    }
                                    Err(e) => {
                                        state.write().delivery_status = format!("Delivery generation error: {e}");
                                    }
                                }
                            }
                        },
                        "Generate Dual-SKU Delivery (Guest Installer + Debian Trixie Overlay)"
                    }
                }
                if !updater_status.is_empty() {
                    p { class: "path", "Updater (C7): {updater_status}" }
                }
                if !delivery_status.is_empty() {
                    p { class: "path", "Dual-SKU Packaging: {delivery_status}" }
                }
                section {
                    class: "id-list",
                    h4 { "Installed Qaps in storage/Qapps/registry.json ({qap_lines.len()})" }
                    ul {
                        for q_line in qap_lines {
                            li { "{q_line}" }
                        }
                    }
                }
            }
        },
        SettingsSection::Technical => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "Technical details" }
                    p { "Low-level physical GPU adapter diagnostics, Anvil NodeProfile64 specialization hints, and 11 native vibescript-abi extension providers." }
                }
                RendererHealthPanel { state }
                section {
                    class: "id-list",
                    h4 { "Physical Etalon Native GPU Adapters ({adapter_lines.len()})" }
                    ul {
                        for a_line in adapter_lines {
                            li { "{a_line}" }
                        }
                    }
                }
            }
        },
    }
}

#[component]
pub(crate) fn WorkspaceSave(state: Signal<ShellState>) -> Element {
    let mut status = use_signal(String::new);
    rsx! {
        div {
            class: "workspace-save",
            button {
                r#type: "button",
                class: "sanctum-issue",
                onclick: move |_| {
                    let Some(sess) = session() else {
                        status.set("Open a local session to save a workspace.".into());
                        return;
                    };
                    let snapshot = state.peek().to_workspace_snapshot();
                    status.set(match save_workspace_snapshot(&sess.host.storage, &snapshot) {
                        Ok(path) => format!("Workspace saved to {}", path.display()),
                        Err(error) => format!("Workspace save failed: {error}"),
                    });
                },
                "Save Workspace Snapshot (shell_workspace.json)"
            }
            p { class: "path", role: "status", "{status}" }
        }
    }
}

#[component]
fn RendererHealthPanel(state: Signal<ShellState>) -> Element {
    let health = state.read().renderer_health.clone();
    let (width, height) = health.viewport_dimensions;
    rsx! {
        section {
            class: "id-list",
            h4 { "Native Renderer Health" }
            dl {
                class: "kv renderer-health",
                dt { "Backend" }
                dd { "{health.backend_label}" }
                dt { "Fallback policy" }
                dd { "{health.fallback_policy_label}" }
                dt { "Fallback engaged" }
                dd { if health.fallback_engaged { "Yes" } else { "No" } }
                dt { "HOLO surface" }
                dd { "{health.active_holo_surface_status}" }
                dt { "Viewport" }
                dd { "{width} × {height} px" }
            }
        }
    }
}

pub(crate) fn control_center_storage(
    state: Signal<ShellState>,
) -> Result<&'static std::path::Path, String> {
    if let Some(error) = state.peek().os_control_center_error.as_ref() {
        return Err(format!("Control Center is read-only: {error}"));
    }
    session()
        .map(|sess| sess.host.storage.as_path())
        .ok_or_else(|| "Open a local session to update the Control Center.".into())
}

#[component]
fn OsJobQueue(mut state: Signal<ShellState>) -> Element {
    let runner = try_consume_context::<OsJobRunner>();
    let foundation = try_consume_context::<AdoptedQualiaFoundation>();
    let shell = state.read();
    let jobs = shell.os_control_center.jobs.clone();
    let load_error = shell.os_control_center_error.clone();
    let runner_error = shell.os_job_runner_error.clone();
    let gguf_source = shell
        .staged_gguf_source
        .as_ref()
        .map(|(path, _)| path.clone());
    let can_edit = load_error.is_none() && session().is_some();
    let can_run = can_edit && runner.is_some() && runner_error.is_none();
    drop(shell);

    let mut selected_kind = use_signal(|| OsJobKind::TriTargetDebianBuild);
    let mut vibe_source = use_signal(String::new);
    let mut qcdn_project = use_signal(String::new);
    let mut status = use_signal(String::new);
    let kind = selected_kind();
    let show_vibe_source = kind == OsJobKind::GovernedVibeScriptBatch
        || jobs
            .iter()
            .any(|job| job.kind == OsJobKind::GovernedVibeScriptBatch && job.status.can_start());
    let show_qcdn_project = kind == OsJobKind::QcdnSwarmMerkleSync
        || jobs
            .iter()
            .any(|job| job.kind == OsJobKind::QcdnSwarmMerkleSync && job.status.can_start());
    let show_gguf_source = kind == OsJobKind::GgufToP64Conversion
        || jobs
            .iter()
            .any(|job| job.kind == OsJobKind::GgufToP64Conversion && job.status.can_start());
    let has_vibe_admission = foundation
        .as_ref()
        .is_some_and(|scope| scope.graph_grant().is_some());
    let has_vibe_source = !vibe_source.read().trim().is_empty();
    let has_qcdn_project = !qcdn_project.read().trim().is_empty();
    let staged_gguf = gguf_source
        .as_ref()
        .and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned());

    rsx! {
        section {
            class: "id-list os-job-queue",
            h4 { "OS Job Queue" }
            if let Some(error) = load_error {
                p { class: "path", role: "status", "Control Center is read-only. {error}" }
            } else if let Some(error) = runner_error {
                p { class: "path", role: "status", "Background runner unavailable. {error}" }
            }
            p { class: "muted", "Queued jobs start when you select Run. You can leave Settings while a job runs." }
            div {
                class: "input-row",
                label {
                    "Job type "
                    select {
                        value: "{kind.code()}",
                        onchange: move |event| {
                            let code = event.value();
                            if let Some(kind) = OsJobKind::ALL.into_iter().find(|kind| kind.code() == code) {
                                selected_kind.set(kind);
                                status.set(String::new());
                            }
                        },
                        for kind in OsJobKind::ALL {
                            option { key: "{kind.code()}", value: "{kind.code()}", "{kind.label()}" }
                        }
                    }
                }
                button {
                    r#type: "button",
                    disabled: !can_edit,
                    onclick: move |_| {
                        let kind = selected_kind();
                        let result = control_center_storage(state).and_then(|storage| {
                            state.write().os_control_center.enqueue_os_job(
                                storage, kind, kind.label(), kind.default_volume_slug(),
                            )
                        });
                        status.set(match result {
                            Ok(job) => format!("{} queued.", job.title),
                            Err(error) => error,
                        });
                    },
                    "Queue job"
                }
            }
            if show_gguf_source {
                if let Some(name) = staged_gguf {
                    p { "GGUF source staged in Instruments · {name}" }
                } else {
                    p { class: "muted", "Stage a local .gguf file with the semantic command bar before running conversion." }
                }
            }
            if show_qcdn_project {
                label {
                    "Provisioned Q-CDN project ID"
                    input {
                        class: "input-field",
                        value: "{qcdn_project}",
                        maxlength: "128",
                        oninput: move |event| qcdn_project.set(event.value()),
                    }
                }
                p { class: "muted", "The host configuration supplies the source and destination for this project mirror." }
            }
            if show_vibe_source {
                if has_vibe_admission {
                    label {
                        "Admitted VibeScript source"
                        textarea {
                            value: "{vibe_source}",
                            maxlength: "65536",
                            rows: "5",
                            oninput: move |event| vibe_source.set(event.value()),
                        }
                    }
                    p { class: "muted", "Use the exact source admitted by your current QualiaDB session. This draft is kept in memory." }
                } else {
                    p { class: "muted", "Governed VibeScript jobs require an authenticated QualiaDB project session and signed graph scope." }
                }
            }
            if jobs.is_empty() {
                p { class: "muted", "The job queue is empty." }
            }
            ul {
                for job in jobs {
                    {
                        let ready = job.status.can_start();
                        let blocked = job.status == OsJobStatus::Blocked;
                        let running = job.status == OsJobStatus::Running;
                        let can_start = can_run && ready && match job.kind {
                            OsJobKind::GgufToP64Conversion => gguf_source.is_some(),
                            OsJobKind::QcdnSwarmMerkleSync => has_qcdn_project,
                            OsJobKind::GovernedVibeScriptBatch => has_vibe_admission && has_vibe_source,
                            _ => true,
                        };
                        let can_cancel = can_edit && (!running || runner.is_some());
                        let start_id = job.job_id.clone();
                        let cancel_id = job.job_id.clone();
                        let start_runner = runner.clone();
                        let cancel_runner = runner.clone();
                        let source = gguf_source.clone();
                        let adopted = foundation.clone();
                        let kind = job.kind;
                        let label = job.status.label();
                        rsx! {
                            li {
                                key: "{job.job_id}",
                                strong { "{job.title} · {label}" }
                                p { "{job.target_volume} · {job.detail}" }
                                if ready || running {
                                    div {
                                        class: "input-row",
                                        if ready {
                                            button {
                                                r#type: "button",
                                                disabled: !can_start,
                                                onclick: move |_| {
                                                    let result = (|| {
                                                        control_center_storage(state)?;
                                                        let runner = start_runner.as_ref().ok_or("Background runner unavailable.")?;
                                                        let input = match kind {
                                                            OsJobKind::GgufToP64Conversion => OsJobInput::Gguf {
                                                                source: source.clone().ok_or("Stage a local GGUF file first.")?,
                                                            },
                                                            OsJobKind::QcdnSwarmMerkleSync => {
                                                                let project = qcdn_project();
                                                                let project_id = project.trim();
                                                                if project_id.is_empty() {
                                                                    return Err("Enter the provisioned Q-CDN project ID.".into());
                                                                }
                                                                OsJobInput::QcdnMirror { project_id: project_id.to_owned() }
                                                            }
                                                            OsJobKind::GovernedVibeScriptBatch => {
                                                                let foundation = adopted.clone().ok_or("Authenticate the QualiaDB project first.")?;
                                                                let source = vibe_source();
                                                                if source.trim().is_empty() {
                                                                    return Err("Enter the exact admitted VibeScript source.".into());
                                                                }
                                                                OsJobInput::GovernedVibe { foundation, source }
                                                            }
                                                            _ => OsJobInput::Standard,
                                                        };
                                                        runner.start(&start_id, input)
                                                    })();
                                                    status.set(match result {
                                                        Ok(()) => "Job submitted to the background runner.".into(),
                                                        Err(error) => error,
                                                    });
                                                },
                                                if blocked { "Retry" } else { "Run" }
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            disabled: !can_cancel,
                                            onclick: move |_| {
                                                let result = control_center_storage(state).and_then(|storage| {
                                                    if let Some(runner) = cancel_runner.as_ref() {
                                                        runner.cancel(&cancel_id)
                                                    } else {
                                                        state.write().os_control_center.cancel_os_job(storage, &cancel_id).map(|_| ())
                                                    }
                                                });
                                                status.set(match result {
                                                    Ok(()) => "Cancellation requested.".into(),
                                                    Err(error) => error,
                                                });
                                            },
                                            if running { "Request cancellation" } else { "Cancel job" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { role: "status", "{status}" }
        }
    }
}

#[component]
fn QapLeaseMatrix(state: Signal<ShellState>) -> Element {
    let foundation = try_consume_context::<AdoptedQualiaFoundation>();
    let storage = session().map(|sess| sess.host.storage.as_path());
    let shell = state.read();
    let leases = shell.os_control_center.leases.clone();
    let can_edit = shell.os_control_center_error.is_none() && storage.is_some();
    let mut package_ids: Vec<_> = leases
        .iter()
        .map(|lease| lease.package_id.clone())
        .collect();
    package_ids.extend(
        shell
            .installed_qaps
            .iter()
            .filter(|qap| !qap.revoked)
            .map(|qap| qap.package_id.clone()),
    );
    package_ids.sort();
    package_ids.dedup();
    // A retained policy row must never revive a revoked package.
    package_ids.retain(|id| {
        !shell
            .installed_qaps
            .iter()
            .any(|qap| &qap.package_id == id && qap.revoked)
    });
    drop(shell);

    let mut draft = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut check_again = use_signal(|| false);
    let _ = check_again();
    let selected_qap = {
        let draft = draft.read();
        package_ids
            .iter()
            .find(|id| id.as_str() == draft.as_str())
            .or_else(|| package_ids.first())
            .cloned()
            .unwrap_or_default()
    };
    rsx! {
        section {
            class: "id-list qap-lease-matrix",
            h4 { "Per-Qap Capability Lease Matrix" }
            p { class: "muted", "A local deny blocks the capability. A local allow still requires a current canonical host grant." }
            if package_ids.is_empty() {
                p { "No Qaps are available for capability policy." }
            } else {
                div {
                    class: "input-row",
                    label {
                        "Qap "
                        select {
                            value: "{selected_qap}",
                            onchange: move |event| {
                                draft.set(event.value());
                                status.set(String::new());
                            },
                            for package_id in package_ids {
                                option { key: "{package_id}", value: "{package_id}", "{package_id}" }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        disabled: storage.is_none(),
                        onclick: move |_| {
                            let checked = check_again();
                            check_again.set(!checked);
                        },
                        "Recheck authority"
                    }
                }
                table {
                    aria_label: "Local preferences and effective Qap authority",
                    thead {
                        tr {
                            th { scope: "col", "Capability" }
                            th { scope: "col", "Local policy" }
                            th { scope: "col", "Effective authority" }
                            th { scope: "col", "Actions" }
                        }
                    }
                    tbody {
                        for scope in CapabilityLeaseScope::ALL {
                            {
                                let current = leases.iter().find(|lease| {
                                    lease.package_id == selected_qap && lease.scope == scope && lease.epoch_generation != 0
                                });
                                let locally_allowed = current.is_some_and(|lease| lease.granted);
                                let locally_denied = current.is_some_and(|lease| !lease.granted);
                                let local_label = match current {
                                    Some(lease) if lease.granted => "Allow with host grant",
                                    Some(_) => "Explicit deny",
                                    None => "No local preference",
                                };
                                let effective = storage.map_or(QapLeaseStatus::Unavailable, |storage| {
                                    qap_lease_status(storage, &selected_qap, scope, foundation.as_ref())
                                });
                                let allow_id = selected_qap.clone();
                                let deny_id = selected_qap.clone();
                                rsx! {
                                    tr {
                                        key: "{scope.code()}",
                                        th { scope: "row", "{scope.label()}" }
                                        td { "{local_label}" }
                                        td { "{effective.label()}" }
                                        td {
                                            div {
                                                class: "input-row",
                                                button {
                                                    r#type: "button",
                                                    aria_label: "Allow {scope.label()} with host authorization",
                                                    disabled: !can_edit || locally_allowed,
                                                    onclick: move |_| status.set(update_qap_preference(state, &allow_id, scope, true)),
                                                    "Allow with host grant"
                                                }
                                                button {
                                                    r#type: "button",
                                                    aria_label: "Deny {scope.label()}",
                                                    disabled: !can_edit || locally_denied,
                                                    onclick: move |_| status.set(update_qap_preference(state, &deny_id, scope, false)),
                                                    "Deny"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { role: "status", "{status}" }
        }
    }
}

fn update_qap_preference(
    mut state: Signal<ShellState>,
    package_id: &str,
    scope: CapabilityLeaseScope,
    allow: bool,
) -> String {
    let result = control_center_storage(state).and_then(|storage| {
        let mut s = state.write();
        let lease = s
            .os_control_center
            .set_qap_lease(storage, package_id, scope, allow)?;
        let cc = s.os_control_center.clone();
        s.orchestrator.rebalance_all_quotas(Some(&cc));
        let _ = s.orchestrator.save(storage);
        Ok(lease)
    });
    match result {
        Ok(lease) => format!("{} · {}", lease.scope.label(), lease.last_audit_note),
        Err(error) => error,
    }
}

#[component]
fn BackupRecoveryPanel(mut state: Signal<ShellState>) -> Element {
    let archive = state.read().last_q42j_backup.clone();
    let mut selected = use_signal(Vec::<u8>::new);
    let mut archive_id = use_signal(String::new);
    let mut status = use_signal(String::new);
    rsx! {
        section {
            class: "id-list portable-data-panel",
            h4 { "Shamir Journal Backup & Recovery" }
            p { class: "muted", "The archive contains encrypted public metadata. Keep staged shares with independent custodians." }
            button {
                r#type: "button",
                class: "sanctum-issue",
                onclick: move |_| {
                    let Some(sess) = session() else {
                        status.set("Open a local session to create a backup.".into());
                        return;
                    };
                    match export_shamir_q42j_backup(&sess.host.storage, 2, 3) {
                        Ok(archive) => {
                            archive_id.set(archive.archive_id.clone());
                            selected.set(Vec::new());
                            state.write().last_q42j_backup = Some(archive);
                            status.set("Encrypted archive and share files staged.".into());
                        }
                        Err(error) => status.set(error),
                    }
                },
                "Export 2-of-3 Shamir .q42j Backup"
            }
            div {
                class: "input-row",
                label {
                    "Saved backup ID "
                    input {
                        class: "input-field",
                        value: "{archive_id}",
                        maxlength: 32,
                        oninput: move |event| archive_id.set(event.value()),
                    }
                }
                button {
                    r#type: "button",
                    onclick: move |_| {
                        let Some(sess) = session() else {
                            status.set("Open a local session to load a backup.".into());
                            return;
                        };
                        let id = archive_id();
                        match load_shamir_q42j_backup(&sess.host.storage, id.trim()) {
                            Ok(archive) => {
                                selected.set(Vec::new());
                                state.write().last_q42j_backup = Some(archive);
                                status.set("Backup manifest loaded. Select the shares you can access.".into());
                            }
                            Err(error) => status.set(error),
                        }
                    },
                    "Load saved manifest"
                }
            }
            if let Some(archive) = archive {
                {
                    let threshold = archive.threshold_m;
                    let total = archive.total_n;
                    let archive_key = archive.archive_id.clone();
                    let included = archive.included_sections.join(", ");
                    let manifest = archive.share_files.first().and_then(|path| path.parent())
                        .map(|folder| folder.join("archive.json"));
                    let restore_ready = selected.read().len() >= usize::from(threshold);
                    let share_paths: Vec<_> = archive.share_files.iter().map(|path| path.display().to_string()).collect();
                    rsx! {
                        p { class: "path", "Backup {archive_key} · {threshold} of {total} shares · {archive.payload_bytes} bytes" }
                        p { "Included sections: {included}" }
                        if let Some(path) = manifest {
                            p { class: "path", "Manifest: {path.display()}" }
                        }
                        fieldset {
                            class: "q42j-share-picker",
                            legend { "Shares for recovery" }
                            for (offset, path) in share_paths.into_iter().enumerate() {
                                {
                                    let index = (offset + 1) as u8;
                                    let checked = selected.read().contains(&index);
                                    rsx! {
                                        label {
                                            key: "{index}",
                                            input {
                                                r#type: "checkbox",
                                                checked,
                                                onchange: move |event| {
                                                    let mut indices = selected.write();
                                                    if event.checked() {
                                                        if !indices.contains(&index) { indices.push(index); }
                                                    } else {
                                                        indices.retain(|chosen| *chosen != index);
                                                    }
                                                },
                                            }
                                            span { class: "path", "Share {index} · {path}" }
                                        }
                                    }
                                }
                            }
                        }
                        button {
                            r#type: "button",
                            disabled: !restore_ready,
                            onclick: move |_| {
                                let Some(sess) = session() else {
                                    status.set("Open a local session to recover a backup.".into());
                                    return;
                                };
                                let shares = selected();
                                status.set(match verify_and_restore_shamir_q42j_backup(&sess.host.storage, &archive, &shares) {
                                    Ok(receipt) => receipt,
                                    Err(error) => error,
                                });
                            },
                            "Verify & recover metadata to a review copy"
                        }
                    }
                }
            }
            p { class: "path", role: "status", "{status}" }
        }
    }
}

#[component]
fn SettingsSolidExport(mut state: Signal<ShellState>) -> Element {
    let shell = state.read();
    let selected_entry = shell
        .semantic_fs
        .entries
        .iter()
        .find(|entry| entry.asset_uri == shell.selected_library_uri)
        .cloned();
    let receipt = shell.last_solid_receipt.as_ref().map(|receipt| {
        (
            receipt.triple_count,
            receipt.sealed_nquin_parity,
            receipt.file_path.clone(),
        )
    });
    drop(shell);
    let selected_title = selected_entry
        .as_ref()
        .map(|entry| entry.title.as_str())
        .unwrap_or("Select a container in the Library")
        .to_owned();
    let eligible = selected_entry.as_ref().is_some_and(|entry| {
        !entry.is_secret()
            && entry.sensitivity.trim().eq_ignore_ascii_case("public")
            && entry.commons_visibility == CommonsShareScope::Commons
    });
    let mut target_endpoint = use_signal(String::new);
    let mut status = use_signal(String::new);
    rsx! {
        section {
            class: "id-list portable-data-panel",
            h4 { "Export Active Library Container to Solid Turtle (.ttl)" }
            p { "Library selection: {selected_title}" }
            p { class: "muted", "Exports eligible public Permissive Commons entries to a local Turtle file." }
            label {
                "HTTPS Pod endpoint label (optional) "
                input {
                    class: "input-field",
                    value: "{target_endpoint}",
                    maxlength: 2048,
                    oninput: move |event| target_endpoint.set(event.value()),
                }
            }
            button {
                r#type: "button",
                class: "sanctum-issue",
                disabled: !eligible,
                onclick: move |_| {
                    let Some(sess) = session() else {
                        status.set("Open a local session to export a Library entry.".into());
                        return;
                    };
                    let Some(entry) = selected_entry.as_ref() else {
                        status.set("Select a container in the Library.".into());
                        return;
                    };
                    let endpoint = target_endpoint();
                    match export_semantic_entry_to_solid_turtle(&sess.host.storage, entry, endpoint.trim()) {
                        Ok(receipt) => {
                            state.write().last_solid_receipt = Some(receipt);
                            status.set("Local Turtle export saved.".into());
                        }
                        Err(error) => status.set(error),
                    }
                },
                "Export local .ttl"
            }
            if let Some((triple_count, sealed_parity, file_path)) = receipt {
                {
                    let parity = format!("{sealed_parity:016x}");
                    rsx! {
                        p { "Last Turtle receipt: {triple_count} triples · NQuin48 parity {parity}" }
                        if let Some(path) = file_path {
                            p { class: "path", "{path.display()}" }
                        }
                    }
                }
            }
            p { role: "status", "{status}" }
        }
    }
}

#[component]
fn BranchingOrchestratorPanel(mut state: Signal<ShellState>) -> Element {
    let s = state.read();
    let orch = s.orchestrator.clone();
    drop(s);

    let tier_bps = orch.tier_budget_bps;
    let notice = orch.last_orchestrator_notice.clone();
    let branches = orch.branches.clone();
    let dispatches = orch.elastic_dispatches.clone();
    let parity = format!("{:#018x}", orch.sealed_nquin_parity);

    rsx! {
        section {
            class: "id-list",
            h4 { "Two-Level Branching OS & Per-App Thread Orchestrator (4 Environmental Tiers · {branches.len()} App Sub-Trees · NQuin48 {parity})" }
            p {
                class: "muted",
                "Level 1 (Root Environmental Tiers) arbitrates hardware budgets across Safety/Real-Time (Tier 0), Focused Foreground App (Tier 1), Orbit Resident Apps (Tier 2), and Elastic Bulk/Offload Compute (Tier 3). Level 2 (Per-App Sub-Trees) lets each App/Qap reprioritize or pause its own child threads (P0..P3) within its branch without ever starving Tier 0 safety tasks or stealing environmental quota from sibling apps."
            }
            p {
                "Root Tier Split: Tier 0 Safety = {f64::from(tier_bps[0]) / 100.0:.1}% · Tier 1 Foreground = {f64::from(tier_bps[1]) / 100.0:.1}% · Tier 2 Orbit Resident = {f64::from(tier_bps[2]) / 100.0:.1}% · Tier 3 Elastic Bulk = {f64::from(tier_bps[3]) / 100.0:.1}%"
            }
            if !notice.is_empty() {
                p { class: "path", role: "status", "{notice}" }
            }
            div {
                class: "plane-grid",
                for branch in branches {
                    {
                        let bid = branch.branch_id.clone();
                        let bid_focus = bid.clone();
                        let b_title = branch.app_or_volume_title.clone();
                        let b_tier = branch.assigned_tier.title();
                        let b_pct = f64::from(branch.allocated_env_budget_bps) / 100.0;
                        let b_weight = branch.branch_weight;
                        let can_focus = !matches!(
                            branch.assigned_tier,
                            webizen_ua_os::EnvironmentalTier::Tier0SafetyAndRealTime
                                | webizen_ua_os::EnvironmentalTier::Tier3EnvironmentalAndElasticCompute
                        );
                        let threads = branch.child_threads.clone();
                        rsx! {
                            div {
                                key: "{bid}",
                                class: "plane-card",
                                h5 { "{b_title} ({b_pct:.1}% OS Share · Weight {b_weight})" }
                                p { "Environmental Tier: {b_tier}" }
                                if can_focus {
                                    button {
                                        r#type: "button",
                                        class: "sanctum-issue",
                                        onclick: move |_| {
                                            let storage = session().map(|sess| sess.host.storage.as_path());
                                            let mut s = state.write();
                                            let cc = s.os_control_center.clone();
                                            let _ = s.orchestrator.focus_app_branch(
                                                storage,
                                                &bid_focus,
                                                Some(&cc),
                                            );
                                        },
                                        "Focus Branch into Tier 1 Foreground"
                                    }
                                }
                                ul {
                                    for t in threads {
                                        {
                                            let tid = t.thread_id.clone();
                                            let tid_boost = tid.clone();
                                            let tid_demote = tid.clone();
                                            let tid_pause = tid.clone();
                                            let bid_b = bid.clone();
                                            let bid_d = bid.clone();
                                            let bid_p = bid.clone();
                                            let t_title = t.title.clone();
                                            let t_prio = t.priority.label();
                                            let t_state = t.state.label();
                                            let t_share = f64::from(t.effective_global_share_bps) / 100.0;
                                            let t_kib = t.sentinel_arena_budget_kib;
                                            let t_target = t.execution_target.code();
                                            rsx! {
                                                li {
                                                    key: "{tid}",
                                                    div {
                                                        strong { "[{t_state}] [{t_prio}] {t_title} " }
                                                        span { "· Global Share: {t_share:.2}% · Arena: {t_kib} KiB · Locus: {t_target}" }
                                                    }
                                                    div {
                                                        class: "orb-sectors",
                                                        button {
                                                            r#type: "button",
                                                            class: "ribbon-btn",
                                                            onclick: move |_| {
                                                                let storage = session().map(|sess| sess.host.storage.as_path());
                                                                let mut s = state.write();
                                                                let cc = s.os_control_center.clone();
                                                                let _ = s.orchestrator.reprioritize_app_child_thread(
                                                                    storage,
                                                                    &bid_b,
                                                                    &tid_boost,
                                                                    true,
                                                                    Some(&cc),
                                                                );
                                                            },
                                                            "Boost Priority"
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "ribbon-btn",
                                                            onclick: move |_| {
                                                                let storage = session().map(|sess| sess.host.storage.as_path());
                                                                let mut s = state.write();
                                                                let cc = s.os_control_center.clone();
                                                                let _ = s.orchestrator.reprioritize_app_child_thread(
                                                                    storage,
                                                                    &bid_d,
                                                                    &tid_demote,
                                                                    false,
                                                                    Some(&cc),
                                                                );
                                                            },
                                                            "Demote Priority"
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "ribbon-btn",
                                                            onclick: move |_| {
                                                                let storage = session().map(|sess| sess.host.storage.as_path());
                                                                let mut s = state.write();
                                                                let cc = s.os_control_center.clone();
                                                                let _ = s.orchestrator.toggle_app_child_thread_pause(
                                                                    storage,
                                                                    &bid_p,
                                                                    &tid_pause,
                                                                    Some(&cc),
                                                                );
                                                            },
                                                            "Pause / Resume"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !dispatches.is_empty() {
                h5 { "Elastic Compute & High-RAM .q42 Ontology Offload Dispatches ({dispatches.len()})" }
                ul {
                    for d in dispatches.iter().take(6) {
                        li {
                            key: "{d.dispatch_id}",
                            strong { "{d.dispatch_id} · " }
                            span { "{d.status_summary} ({d.colab_routine_or_host_endpoint})" }
                        }
                    }
                }
            }
        }
    }
}
