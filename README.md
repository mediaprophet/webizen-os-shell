# Webizen OS-Shell (`webizen-os-shell` & `webizen-ua-os`)

**Copyright (c) 2026 Timothy Charles Holborn**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (`CC BY-NC-ND 4.0`)

The native **Webizen User-Agent OS Shell** built on **Aether Native** (`aether-native` + Blitz + **Etalon** native Vulkan / DX12 / Metal GPU), backed by **QualiaDB** (`.q42` graph, 42 MiB `SlgArena` Webizen Sentinel, `VaultContainerV2` Sanctuary crypto, and SLD/LLM inference bridge), and unified with **Native Poet** (`vibe-0.1`/`0.2`) and **Hypermedia Production (`Synth` 150 morphisms)**.

---

## Architecture & Overhauled Capabilities

1. **Synth + Aether Spatial UI Overhaul & Poet 8-Sector Command Orb Chrome (`src/components/*`, `assets/*.vibe`, `assets/shell.css`)**:
   - **No Classic OS Clone & No Traffic Lights**: Rejects 1990s–2010s OSX/Windows/Linux desktop chrome and life-domain top-bar IA. Every open volume uses the **Poet 8-Sector Command Orb (`◉`)** (`1. Close ✕`, `2. Soften —`, `3. Full ▢`, `4. Float ◎`, `5. Keep 📁`, `6. Share ↗`, `7. Inspect 🔍`, `8. More ···`).
   - **Top Continuity Ribbon (`who · claim · handle · tool`)**: Interactively inspects the Four-Plane Identifier Fabric and enforces anti-collapse rules (`handle ≠ human`, `agent = tool`, `VC signature = issuer origin & integrity, not truth`, `did:q42 = Storage / Topological Coordinate QRC`).
   - **Humans-First Spatial Halo Rail**: Separates **Humans** (`Talk · Mail · Directory`) from **Foundational Volumes** (`Browser · Keep · Library · Instruments · Poet+Synth · WellFair · Wallet · Settings · Admin`).
   - **Synth UI Sheets & Diegetic Projections**: Authored via [`assets/spatial-shell.vibe`](assets/spatial-shell.vibe) and [`assets/foundations-shell.vibe`](assets/foundations-shell.vibe) using `hypermedia.layout.bind_theme_scope`, `render_ui_sheet`, `project_ui_svg`, and `project_diegetic_surface` (`assets/shell-sheet.png`, `assets/shell-diegetic.png`, `assets/foundations-sheet.png`, `assets/foundations-diegetic.png`).

2. **10-Section Settings & Foundational Environment Setup (`SettingsVolume`)**:
   - **1. Setup Health**: Local foundations scorecard (Data home, Native Etalon GPU, 42 MiB `SlgArena` Sentinel, Four-Plane Identifier Fabric, Local AI Instrument) vs progressive relational paths.
   - **2. Data & Memory**: Unified `.q42` v3/v4, `.10d` v2 `HOLO` (`0x0306`), `.hmc` (`QBDL`), `.m64`, `.emf`, `.qac`, `.p64` v4, and **`QECP`** C2PA-embedded credential PDF strata.
   - **3. AI Instruments**: Unified local SLD/LLM vault scanner (`QUALIA_LLM_VAULT`, `C:\LLM_files`, `C:\LLM_Models`, `{storage}/Models`) with interactive model activation and a private, reversible readiness probe (`run_local_model_readiness_probe`).
   - **4. People & Reachability**: Socially Defined Networking (**QDNF**), $O(1)$ `NQuin` DNS ring, Userspace WireGuard, Nym Mixnet, Acoustic/BLE `SonicToken` mesh, and **W3C Solid / LDP Off-Ramp** (`web.civics.au`).
   - **5. Privacy & Sanctuary**: Constant-shape 4-layer `VaultContainerV2` (`Real` / `Decoy` / `Reserved`) with Argon2id (64 MiB) + AES-256-GCM, Sanctuary Audit DAG, Anvil `NP64` fail-closed attestation gate, Contextual DID issuance (refusing master ask), and ADR 0011 Dead-Man / Incapacity switches.
   - **6. Appearance & Access**: WCAG AA accessible theme presets (`Holosonic Dark`, `High-Contrast Obsidian`, `Permissive Commons Teal`) and reduced-motion mode.
   - **7. Person & Devices (Four-Plane Identifier Fabric)**: Jury-safe separation of `NaturalAgent` (`living-SHACL`, disjoint from `owl:Thing`), `ClaimOpinion` (`CoAttestationBundle`), `SpatiotemporalHandle` (`DNI` / `CB-USRI` / `did:q42` QRC), and `Instrument` (`did:qi`, pairwise locators, `WalletShape`, `SensorId`), paired with `validate_enumerated_identity` (noisy-OR `< 1.0`).
   - **8. Backup & Recovery**: `.q42j` Shamir backups, `.qnf` (`QNFNET01`) network evidence archives, person transfer bundles, and W3C Solid Pod portability.
   - **9. Services & Updates**: Local QualiaDB `:4242` sidecar supervision, Qap registry (`storage/Qapps/registry.json`), **Synth-style Ed25519 self-updater (`C7`)**, and **Dual-SKU delivery generator**.
   - **10. All Technical Settings**: Physical Etalon native GPU adapter diagnostics and 11 native `vibescript-abi` providers.

3. **Unified Native Poet & 150-Morphism Hypermedia Production (Synth) Workbench (`crates/webizen-ua-os/src/poet_synth.rs`)**:
   - Evaluates VibeScript (`vibe-0.1`/`0.2`) reactive cells, functions, and hooks in-process under the 42 MiB `SENTINEL_BYTES` budget (`Zero V8 / Zero JIT`), seals deterministic NLP Gazetteer 48-byte `NQuin48` span records, exposes the **6 Core Semantic Instruments (`SI-01..12`)** and **6 SHACL Extension Bridge families**, and runs `.vibe` 150-morphism `hypermedia.*` production jobs directly inside the OS-Shell.

4. **Dual-SKU Delivery & Synth-Style Ed25519 Self-Updater (`crates/webizen-ua-os/src/updater.rs`)**:
   - **SKU A (Guest Shell Binary Installer & Self-Updater)**: Runs on existing Windows, macOS, and Linux Device OS environments (`webizen-os-shell update check|verify|upgrade|apply`).
   - **SKU B (Linux-Backed User-Agent OS Appliance Overlay)**: Installs `/opt/webizen/` (`qualiadb.service`, `webizen-hostd.service`, `webizen-os-shell.service`, `webizen.target`) onto unmodified **Debian 13 Trixie** (`amd64` and `arm64`) while enforcing the Three-Ring Licence Fence (`LICENSE-BOUNDARY.md`).
