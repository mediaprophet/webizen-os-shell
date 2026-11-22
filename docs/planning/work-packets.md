# Webizen OS-Shell & User-Agent OS Work Packets

The **product** is **Webizen OS-Shell** (Talk, Mail, Directory, Library, Settings, Poet & Hypermedia Synth Workbench, Wallet, WellFair, Sanctuary, Browser, Qaps, Tray) running on the **Aether native** toolkit and **Etalon Native Desktop GPU** (`TargetProfileMask::NATIVE_DESKTOP`, Vulkan / DX12).

- **`qualia-cli` (`127.0.0.1:4242`)** is the local `.q42` graph, Sanctuary vault, **42 MiB `SlgArena` Webizen Sentinel** (`SLG_ARENA_SIZE = 42 * 1024 * 1024`, 917,504 48-byte `NQuin` slots), and governed SLD/LLM inference sidecar.
- **Native Poet & Hypermedia Production (Synth) Workbench** is integrated directly into `webizen-ua-os` and `webizen-os-shell` (`vibe-0.1`, `vibescript-abi`, `etalon-vibe` 11 native providers, and the 150-morphism `hypermedia.*` production pipeline) so the user never needs both Synth and Webizen OS-Shell running.
- **Enumerated Identity (`KD-20`)** extends QualiaDB's `validate_enumerated_identity` (`q42:EnumeratedIdentityShape`, noisy-OR `< 1.0`, `DefinitiveCollapse` rejection, 48-byte `NQuin48` parity-verified records).

| ID | Packet | Status |
|---|---|---|
| **WP-C1** | Aether native window (`HaloBar` / `OrbitWheel` / `StageHost`) | **Done (`49fe719`)** |
| **WP-C0** | `webizen-ua-os` `UaOsHost`, sidecar `:4242` supervision & `/invoke` bridge | **Done** |
| **WP-GPU** | Native Etalon GPU (`etalon::Instance` Vulkan/DX12) + Anvil `NodeProfile64` + 11 `NATIVE_DESKTOP` Vibe providers | **Done** |
| **WP-ID** | QualiaDB `validate_enumerated_identity` extension (noisy-OR `< 1.0`, `NQuin48` parity, contextual Sanctuary issuance) | **Done** |
| **WP-C2/D0–D4** | Atomic Qap installer (`.qpkg` ZIP, Directory, `yaml-ld-q42`), `LaunchPlan` (no `APP_STATE`), `LeaseSheet`, bundled `Anatomy` & `Poet` seeding | **Done** |
| **WP-POET-SYNTH** | Migrated Native Poet (`vibe-0.1` reactive cells, lexicon pack, NLP gazetteer `NQuin48` sealer, unified `C:\LLM_files` vault) + 150-morphism Hypermedia Production workbench | **Done** |
| **WP-C7** | Synth-style Ed25519 self-updater (`webizen-os-shell update check\|verify\|upgrade\|apply`, `.new` staging) | **Done** |
| **WP-C6/D5/F4/A7** | Dual-SKU Delivery generator (`webizen-os-shell package-dual-sku`): SKU A Guest Binary Installer + SKU B Debian 13 Trixie `/opt/webizen` Overlay (`qualiadb.service`, `webizen-hostd.service`, `webizen-os-shell.service`, `webizen.target`, three-ring `LICENSE-BOUNDARY.md`) | **Done** |
| **WP-T1** | Guest host-OS system tray (`tray-icon`) | **Done** |
| **WP-E0/E3** | Ring C `www-engine-worker` (`chora-worker`) MPL-2.0 sibling process + copied-frame browser stage | **Next Wave** |
