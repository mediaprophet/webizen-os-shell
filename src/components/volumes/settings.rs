// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use super::{
    admin::{build_appliance_packages, ApplianceProfilesPanel, HostDaemonPanel},
    llm_agents::LlmAndAgentManagementPanel,
};
use crate::state::{session, SettingsSection, ShellState, ALL_SETTINGS_SECTIONS};
use aether::prelude::*;
use webizen_ua_os::{
    generate_dual_sku_delivery_artifacts, issue_for_requestor, run_local_model_readiness_probe,
    run_updater, Ask, Requestor, UpdateAction, CO_ATTESTATION_BUNDLE_FORMULA, DID_Q42_QRC_LABEL,
    PLANE_CLAIM_OPINION_TITLE, PLANE_INSTRUMENT_TITLE, PLANE_NATURAL_AGENT_TITLE,
    PLANE_SPATIOTEMPORAL_HANDLE_TITLE, VC_ORIGIN_INTEGRITY_DISCLAIMER,
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
                        h3 { "1. Setup Health · Local Foundations & Progressive Relational Paths" }
                        p { "Living health scorecard: local foundations first (Data home, Native Etalon GPU, 42 MiB Sentinel, Four-Plane Identifier Fabric, Local AI Instrument), then progressive relational and network paths when you connect peers." }
                    }
                    div {
                        class: "plane-grid",
                        div { class: "plane-card", h5 { "Data Home [Ready]" } p { "{storage} · Unified .q42 v3/v4 + .10d v2 HOLO + VaultContainerV2" } }
                        div { class: "plane-card", h5 { "Native Etalon GPU [Ready]" } p { "{gpu} · TargetProfileMask::NATIVE_DESKTOP · 11 providers" } }
                        div { class: "plane-card", h5 { "Webizen Sentinel [Ready]" } p { "42 MiB Zero-Heap SlgArena · 917,504 48-byte NQuin48 slots" } }
                        div { class: "plane-card", h5 { "Four-Plane Fabric [Ready]" } p { "{validation}" } }
                        div { class: "plane-card", h5 { "Local AI Instrument" } p { "{model_status}" } }
                        div { class: "plane-card", h5 { "QualiaDB Sidecar (:4242)" } p { "{daemon}" } }
                    }
                }
            }
        }
        SettingsSection::Data => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "2. Data & Memory · Native Binary, Semantic & Hypermedia Container Suite" }
                    p { "Webizen separates private Sanctuary records, lived-memory library assets, and Permissive Commons material into cache- and page-aligned binary containers under your data home." }
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
                        h3 { "3. AI Instruments · .gguf -> .p64 Conversion & Purge, Governed Agent Roster & External LLM MCP Accounts" }
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
                    h3 { "4. People & Reachability · Socially Defined Networking (QDNF) & Solid Off-Ramp" }
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
            }
        },
        SettingsSection::Privacy => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "5. Privacy & Sanctuary · Dual-Lane VaultContainerV2 & Anti-Coercion Safeguards" }
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
            let motion_label = if reduced_motion {
                "Reduced Motion: ON (3D transforms flattened)"
            } else {
                "Reduced Motion: OFF (Spatial depth & float active)"
            };
            rsx! {
                div {
                    header {
                        class: "settings-head",
                        h3 { "6. Appearance & Access · Synth Theme Scope & WCAG AA Accessibility" }
                        p { "Accessible presets generated with Synth hypermedia.layout.bind_theme_scope. Keyboard reachability, labelled controls, and reduced-motion support are first-class." }
                    }
                    div {
                        button {
                            r#type: "button",
                            class: "sanctum-issue",
                            onclick: move |_| state.write().theme_preset = "theme-holosonic",
                            "Preset 1: Holosonic Dark (Default Spatial)"
                        }
                        button {
                            r#type: "button",
                            class: "sanctum-issue",
                            onclick: move |_| state.write().theme_preset = "theme-obsidian",
                            "Preset 2: High-Contrast Obsidian (WCAG AAA)"
                        }
                        button {
                            r#type: "button",
                            class: "sanctum-issue",
                            onclick: move |_| state.write().theme_preset = "theme-commons",
                            "Preset 3: Permissive Commons Teal"
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
                    p { class: "muted", "Active Theme Preset: {theme_preset} · {motion_label}" }
                }
            }
        }
        SettingsSection::Device => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "7. Person & Devices · Four-Plane Identifier Fabric (Jury-Safe Enumeration)" }
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
                    h3 { "8. Backup & Recovery · Shamir .q42j Journals, .qnf Evidence & Solid Portability" }
                    p { "Client-encrypted backups and standards-compliant W3C Solid off-ramp ensure you are never locked into a single device or binary runtime." }
                }
                section {
                    class: "id-list",
                    h4 { "Recovery & Portability Mechanisms" }
                    ul {
                        li { ".q42j Client-Encrypted Journals: Shamir secret-shared backups & Two-Loop Permissive Commons value-flow logs" }
                        li { ".qnf (QNFNET01): Immutable network evidence containers with post-quantum ML-DSA/SLH-DSA proofs" }
                        li { "Person Transfer Bundle (qualia.person.transfer.v1): Re-bind principal stewardship across your apparatus fleet" }
                        li { "W3C Solid / LDP Databox Export (web.civics.au): Lossless 48-byte NQuin <-> RDF Turtle translation boundary" }
                    }
                }
            }
        },
        SettingsSection::Services => rsx! {
            div {
                header {
                    class: "settings-head",
                    h3 { "9. Services & Updates · Qap Registry, Ed25519 Updater & Dual-SKU Delivery" }
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
                    h3 { "10. All Technical Settings · Native Etalon Adapters & VibeScript ABI" }
                    p { "Low-level physical GPU adapter diagnostics, Anvil NodeProfile64 specialization hints, and 11 native vibescript-abi extension providers." }
                }
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
