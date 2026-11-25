# VibeScript Semantic Theming Engine & Full Etalon 3D / Hypermedia Upgrade Plan (`WP-ETALON-01..12`)

**Copyright (c) 2026 Timothy Charles Holborn — CC BY-NC-ND 4.0**  
**Target Repositories:**
- `C:\github\Aether` (`mediaprophet/Aether`)
- `C:\github\webizenai\webizen-os-shell` (`WebizenAI/webizen-os-shell`)
- `C:\github\etalon` (`mediaprophet/etalon`)
- `C:\github\hypermedia-production` (`mediaprophet/hypermedia-production`)
- `C:\github\synth` (`mediaprophet/synth`)

---

## 0. The Core Product Rule: "Build the Application, Not an Engine-Diagnostics Readout"

1. **Never Mistake Specification Telemetry for the User Interface:** Dumping internal struct fields, DIDs, SHA3 hashes, and "Simulate Receipt" buttons onto the primary user screen as `<dl class="kv">` tables produces an engineering diagnostic readout, not a product.
2. **Apps Must Be Real Working Workspaces (90% Canvas / Content, 0% Jargon):**
   - Clicking **Browser** must open an actual browser toolbar + full-stage viewport embedding `ChoraFrameWidget` (`chora-worker` for `https://`) or the full-bleed 3D `Holo10dViewport` (`qualia://`), with a clear status state if `chora-worker` is not yet built/connected.
   - Clicking **Poet**, **Talk**, **Mail**, **Library**, **Keep**, **WellFair**, or **Wallet** must open the functional human workspace first, using plain human language.
3. **Move Architecture Telemetry into the `F12` / `Inspect` Developer Drawer (`WP-ETALON-00A..00B`):** Keep all `NQuin48` parity proofs, `RouteStamp` counters, DID bindings, and SHACL receipts accessible inside a collapsible `F12` Developer & Continuity Inspector drawer.

---

## 1. Architectural Thesis: Why VibeScript Semantic Backend + Spatial Theme Cartridges Replaces CSS

In traditional HTML/CSS toolkits, a "theme" is merely a set of color/font tokens applied to a static 2D DOM box tree (`<div>`, `<button>`). CSS variables can change colors, but they cannot transform a 2D sidebar into:
- **Minority Report (`urn:qualia:theme:gestural-holoslate`)**: Curved cylindrical translucent glass strata suspended in 3D depth with Fresnel refraction, bloom, ray-cast spatial hand/pointer picking, and XPBD spring-throw momentum where panels glide along a 3D cylindrical arc.
- **Hackers (`urn:qualia:theme:gibson-cyberspace-1995`)**: A 3D neon wireframe data-city (`HolographicNodeGpu` skyscrapers whose heights, street grids, and pulsing data buses are projected directly from `SemanticFileSystem` entries, `QuinWindow<64>` parity slots, and network daemon telemetry) with CRT phosphor bloom, scanline WGSL post-processing (`etalon-wgsl-forge`), and fly-through orbital camera navigation.
- **Ready Player One / Spatial OASIS (`urn:qualia:theme:oasis-commons-10d`)**: A full 6-DoF / orbital 10D v2 `HOLO` (`0x0306`) spatial amphitheatre where each OS volume is an architectural `.10d` / `.hmc` pavilion (`AREF`/`IOVR`/`INST` two-tier assembly) with diegetic 3D holographic consoles (`hypermedia.layout.project_diegetic_surface`), skeletal `SKL2` avatars/guides, and binaural HRTF spatial audio cues (`etalon-signal` / `SIGNAL_PROVIDER`).

### The 3-Tier Semantic Presentation Architecture

1. **Tier 1 — Semantic Domain & Intent Backend (`SemanticSceneGraph` & `SemanticIntentBus`)**:
   - Publishes a pure **Semantic Scene & Affordance Graph** rooted in `RefractStore<WebizenRefractModel>` and `ShellState` (`SemanticUiNode` records with stable 64-bit `semantic_id: u64`, Four-Plane classification, epistemic status, 10D coordinates `coords_10d: [f32; 10]`, and `SemanticIntent` affordances).
2. **Tier 2 — VibeScript Semantic Theme & Spatial Projection Engine (`ThemeCartridge` + `vibe` + `etalon-vibe`)**:
   - A **Theme** is a hot-swappable `.vibe` + `.hmc` cartridge evaluated inside the 42 MiB `SlgArena` Sentinel.
   - Maps `SemanticUiNode`s onto `SpatialManifoldTopology` (`PlanarDesktopGrid`, `CylindricalHoloslateArc`, `GibsonCyberspaceMatrix`, `OrbitalOasisAmphitheatre10d`), `HolographicNodeGpu` 3D instances, `PostProcessShaderProfile`, and `SIGNAL_PROVIDER` acoustic cues while enforcing `hypermedia.layout.audit_affordance_equivalence`.
3. **Tier 3 — Etalon Unified 3D + Diegetic Vector Compositor (`aether-native` + `etalon-render` + `etalon-wgsl-forge`)**:
   - Combines `etalon-render::VolumetricPipeline` (`.10d` `HOLO` `0x0306` storage buffers), `EmfPipeline`, `Vello` + `EtalonVectorWidget` diegetic quads, `etalon-wgsl-forge` runtime shaders, and `etalon-render::picking_readback` ($O(1)$ `R32Uint` 3D object picking).

---

## 2. Work-Packet Register (`WP-ETALON-00A` .. `WP-ETALON-12`)

- **`WP-ETALON-00A`**: Jargon Purge, Plain-Language App Launcher & `F12` Developer Diagnostics Drawer (`src/components/dev_inspector.rs`, `stage_host.rs`, `halo_bar.rs`).
- **`WP-ETALON-00B`**: Real App Workspace Hosting — Full-Viewport Browser (`ChoraFrameWidget` + full-bleed `Holo10dViewport`) & Workspace-First Volume Layouts (`browser.rs`, `poet_synth.rs`, `communications.rs`, `library.rs`).
- **`WP-ETALON-01`**: Semantic UI Scene-Graph (`SemanticSceneGraph`, `SemanticUiNode`) & Typed `SemanticIntentBus` Projection (`crates/webizen-ua-os/src/semantic_scene_graph.rs`).
- **`WP-ETALON-02`**: VibeScript Theme Cartridge Contract (`.vibe` + `N3`/`SHACL` + `ThemeCartridgeRegistry` with 6 built-in cartridges in `crates/webizen-ua-os/src/theme_cartridge.rs` & `aether-qualia-host`).
- **`WP-ETALON-03`**: Live VibeScript Theme Studio & Hot-Reload Inspector in `Settings -> 6. Appearance & Access` and `Poet & Synth`.
- **`WP-ETALON-04`**: Link `etalon-render` into `aether-native` & Bind `.10d` `HOLO` (`0x0306`) Storage Buffers (`M64HoloRecord` + `EulerianWavefieldRecord` -> `HolographicNodeGpu`).
- **`WP-ETALON-05`**: Bijective 3D GPU Object Picking (`etalon-render::picking` & `picking_readback`) in `aether-native`.
- **`WP-ETALON-06`**: Runtime Shader Synthesis & Post-Processing via `etalon-wgsl-forge` (`HoloslateRefractiveGlassBloom`, `GibsonPhosphorCrtScanline`, `OasisVolumetricSpectralHdr`).
- **`WP-ETALON-07`**: Flagship Cartridge 1 — *Minority Report* Gestural Holoslate (`src/components/themes/holoslate_stage.rs`, `assets/themes/holoslate.vibe`).
- **`WP-ETALON-08`**: Flagship Cartridge 2 — *Hackers* (1995) Gibson Cyberspace Matrix (`src/components/themes/gibson_cyberspace_stage.rs`, `assets/themes/gibson_cyberspace.vibe`).
- **`WP-ETALON-09`**: Flagship Cartridge 3 — *Ready Player One* Spatial OASIS 10D Amphitheatre (`src/components/themes/oasis_amphitheatre_stage.rs`, `assets/themes/oasis_10d.vibe`).
- **`WP-ETALON-10`**: Zero-Heap Procedural Acoustic & Binaural UI Feedback (`crates/webizen-ua-os/src/theme_acoustics.rs` via `etalon-vibe` `SIGNAL_PROVIDER`).
- **`WP-ETALON-11`**: Synth MCP 400-Morphism Theme Asset & `.hmc` Cartridge Compiler Bridge (`poet_synth.rs`).
- **`WP-ETALON-12`**: Multi-Theme UAT Qualification, Affordance Equivalence & Performance Benchmark Gate (`src/uat.rs` `--all-themes`).
