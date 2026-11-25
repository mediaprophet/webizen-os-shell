# Master Grok Execution Specification: Product-First App Host, VibeScript Semantic Theming Engine & Full Etalon 3D GPU Upgrade (`WP-ETALON-00A` .. `WP-ETALON-12`)

**Copyright (c) 2026 Timothy Charles Holborn — CC BY-NC-ND 4.0**  
**Target Repositories:**
- `C:\github\webizenai\webizen-os-shell` (`WebizenAI/webizen-os-shell`)
- `C:\github\Aether` (`mediaprophet/Aether`)
- `C:\github\etalon` (`mediaprophet/etalon`)
- `C:\github\hypermedia-production` (`mediaprophet/hypermedia-production`)
- `C:\github\synth` (`mediaprophet/synth`)

---

## 0. Mandatory Governing Directives for Grok (Read First Before Writing Any Code)

### Directive 1 — Product-First UI & Zero-Jargon Main Chrome ("Build the Application, Not an Engine-Diagnostics Readout")
1. **Never Mistake Specification Telemetry for the User Interface:**
   - Do **not** render internal struct dumps, `<dl class="kv">` key-value tables, raw DIDs, SHA3 hashes, `NQuin48` hex words, `RouteStamp` counters, or specification buzzwords (`MPL-2.0 sibling worker`, `living-SHACL`, `Argon2id`, `VaultContainerV2`, `SlgArena`, `B1b`) on the primary user-facing surface of any application volume or on the Home screen.
2. **90% Functional Workspace Canvas, Plain Everyday Human Language:**
   - Every volume in `webizen-os-shell` must open directly into **the functional human tool** occupying 85–90% of the stage area, using clear, natural, everyday English labels:
     - **Browser**: A real web & spatial browser toolbar (`← Back`, `→ Forward`, `↻ Reload`, URL/Search bar, `Bookmarks` popover, `Mode: Web / Spatial / Hybrid` selector) above a **full-window viewport** that embeds the live `ChoraFrameWidget` (`chora-worker` for `https://`) or the full-stage 3D Etalon viewport (`Holo10dViewport` for `qualia://`). If `chora-worker.exe` is not yet compiled or running, display a clean, honest in-viewport launcher state (*"Web Engine worker (`chora-worker`) is not running — click to probe or launch"*) rather than displaying a wall of diagnostic tables.
     - **Talk & Mail**: Real 2-pane communication workspaces (contact/thread list on the left, active conversation/message reader and compose bar on the right).
     - **Poet Studio**: Real rich document editor, spreadsheet cell grid, and interactive 3D asset studio canvas.
     - **Library**: Visual media, document, and 3D model card grid with an integrated preview pane.
     - **Keep (Personal Vault)**: Clean vault unlock status, protected files/credentials browser, and guided family/guardian recovery wizard.
     - **WellFair & Anatomy**: Interactive 3D anatomical atlas viewport, personal health journal, and care-circle check-in controls.
     - **Wallet & Projects**: Clean balance cards, send/receive payment form, and visual project milestone tracker.
3. **Move All Architecture Telemetry & Debug Proofs into the `F12` / `🔍 Inspect` Developer Drawer (`src/components/dev_inspector.rs`):**
   - Preserve 100% of the existing cryptographic proofs, 4-plane identifier bindings, `NQuin48` parity inspectors, capability lease matrices, and worker telemetry—but house them inside a collapsible **Developer & Continuity Inspector Drawer** toggled by pressing **`F12`** or clicking **`🔍 Inspect`**, hidden by default during normal use.

### Directive 2 — 100% Original IP-Clean Spatial Design Archetypes (Zero Third-Party Film / Franchise References)
- All visual themes, shader profiles, struct/enum identifiers, `.vibe` scripts, and UI labels must be **original creative works** defined strictly by their **geometric, optical, kinetic, and acoustic properties**:
  - **Archetype A (`theme-holoslate` — *Curved Refractive Glass Arc & Gestural Depth Strata*)**: Concave 3D cylindrical translucent glass panels suspended in depth with Fresnel edge refraction, soft bloom, and spring-damper inertial throw physics.
  - **Archetype B (`theme-cyber-grid` — *3D Neon Wireframe Telemetry Grid & Phosphor Matrix*)**: 3D isometric vector grid where semantic vaults, volumes, and data records rise as glowing translucent wireframe geometric towers with packet light-lanes, fly-through camera docking, and optional phosphor scanline filtering.
  - **Archetype C (`theme-spatial-commons` — *360° Orbital 10D Volumetric Amphitheatre & Diegetic Consoles*)**: Full 360-degree orbital 10D v2 `HOLO` (`0x0306`) spatial commons horizon where volumes are architectural 3D SDF pavilions (`AREF`/`IOVR`/`INST`) with floating angled diegetic 3D consoles and binaural HRTF audio cues.
- **Never** use copyrighted film titles, book titles, character names, or franchise trademarks in any code identifier, comment, UI label, or commit message.

### Directive 3 — Human-Rights Vocabulary & Licensing Headers
- Every new or modified file must begin with:
  ```rust
  // Copyright (c) 2026 Timothy Charles Holborn.
  // SPDX-License-Identifier: CC-BY-NC-ND-4.0
  ```
- **Forbidden Terms:** Never use the words `"sovereign"`, `"sovereignty"`, or `"self-sovereign"` anywhere in code, comments, or UI strings (`validate_human_centric_host_nomenclature` enforces this fail-closed). Always use human-rights agreement vocabulary: **human rights**, **human dignity**, **human agency**, **natural person**, **moral and material interests of the author**, **sanctuary / selfhood**, and **Permissive Commons**.
- Use **civics** for people-led cooperative work and **civic** for government/parliamentary activity.

### Directive 4 — 42 MiB Sentinel & Zero-Heap Hot Paths
- All VibeScript evaluations (`vibe`, `poet_eval_native_cell`), `refract` lens updates, and per-frame rendering/kinetic callbacks (`etalon-vibe`) must execute within the 42 MiB `SlgArena` Sentinel without unbounded per-frame heap allocations.

### Directive 5 — EMF-First (`Q42EMF64`), 10D HOLO & Zero-Heap `UiLayoutTree` Primary Rendering (Demoting CSS to Fallback)
- Push UI layout, surface optics, focus/salience fields, and wave-coupled animations down the stack into `etalon-render::emf` (`EmfPipeline`, 64-byte `EmfFieldCell` with 10D manifold coordinates + 3 Stokes polarization parameters + CIE 1931 XYZ 6-band spectral color), `etalon-render::volumetric` (`VolumetricPipeline`, 96-byte `HolographicNodeGpu`), and `etalon-render::hud::UiLayoutTree` (`MAX_UI_NODES = 256` zero-heap flat-array constraint solver + `HudVertex` generator).
- Demote CSS (`assets/shell.css`) to a minimal text-shaping (Parley), IME input, AccessKit screen-reader tree, and CPU/Software fallback layer (`RendererFallbackPolicy::CpuOnFailure`).

---

## 1. Work-Packet Register (`WP-ETALON-00A` .. `WP-ETALON-12`)

### Phase 0 — Product-First UI Transformation, Jargon Purge & Real App / Chora Hosting
- **`WP-ETALON-00A` — Jargon Purge, Plain-Language App Launcher & `F12` Developer Diagnostics Drawer**:
  - Target files: `src/components/dev_inspector.rs` (new), `src/components/stage_host.rs`, `src/components/halo_bar.rs`, `src/state/shell_state.rs`, `src/app.rs`, `assets/shell.css`.
  - Add `dev_inspector_open: bool` (toggled via `F12` / `Ctrl+Shift+I` / `🔍 Inspect`) and `DevInspectorDrawer` with 4 tabs (`Continuity & Four Planes`, `GPU & Etalon Renderer`, `NQuin48 & Refract Store`, `Workers & Capability Leases`).
  - Replace all specification jargon on `EmptyManifold` home cards, top ribbon, and footer with clean, plain-English consumer labels.
- **`WP-ETALON-00B` — Real App Workspace Hosting (`ChoraFrameWidget` Browser + Workspace-First Volumes)**:
  - Target files: `src/components/volumes/browser.rs`, `src/components/volumes/communications.rs`, `src/components/volumes/poet_synth.rs`, `src/components/volumes/library.rs`, `src/components/volumes/wallet.rs`, `src/components/volumes/projects.rs`.
  - Convert `BrowserVolume` into a compact top browser toolbar (`←`, `→`, `↻`, URL/Search bar, `Bookmarks` drawer toggle, `Engine Status` drawer toggle) above a **full-window viewport (85%+ height)** mounting `Holo10dViewport` (for `qualia://` / `.10d` / `.hmc`) or `ChoraFrameWidget` / honest worker-launcher card (for `https://`).
  - Lead all volumes immediately with their primary interactive workspace canvas, moving `<dl class="kv">` diagnostic tables into `DevInspectorDrawer` or collapsed `<details class="tech-details-drawer">` sections.

### Phase 1 — Headless Semantic UI Scene-Graph & VibeScript Theme Cartridge Engine
- **`WP-ETALON-01` — Semantic UI Scene-Graph & Typed `SemanticIntentBus` Projection**:
  - Target files: `crates/webizen-ua-os/src/semantic_scene_graph.rs` (new), `crates/webizen-ua-os/src/lib.rs`, `src/state/shell_state.rs`.
  - Implement `SemanticNodeKind`, `SemanticIntent`, `SemanticUiNode` (with `semantic_id: u64`, `coords_10d: [f32; 10]`, `primary_intent`), `ShellState::project_semantic_scene_graph()`, and `ShellState::apply_semantic_intent()`.
- **`WP-ETALON-02` — VibeScript Theme Cartridge Engine (`.vibe` + `ThemeCartridgeRegistry` + Affordance Equivalence Gate)**:
  - Target files: `crates/webizen-ua-os/src/theme_cartridge.rs` (new), `C:\github\Aether\packages\qualia-host\src\theme_projection.rs` (new).
  - Implement `SpatialManifoldTopology` (`PlanarDesktopGrid`, `CylindricalGlassArc`, `IsometricWireframeCityGrid`, `OrbitalVolumetricAmphitheatre10d`), `PostProcessShaderProfile` (`CleanSpatialSrgb`, `RefractiveCurvedGlassBloom`, `WireframePhosphorScanline`, `VolumetricSpectralHdr`), 6 built-in IP-clean cartridges (`theme-holosonic`, `theme-obsidian`, `theme-commons`, `theme-holoslate`, `theme-cyber-grid`, `theme-spatial-commons`), `project_semantic_scene_with_cartridge()`, and `verify_cartridge_affordance_equivalence()`.
- **`WP-ETALON-03` — Live VibeScript Theme Studio & Cartridge Switcher**:
  - Target files: `src/components/stage_host.rs`, `src/components/volumes/settings.rs`, `src/components/volumes/poet_synth.rs`, `crates/webizen-ua-os/src/workspace_persistence.rs`.
  - Wire one-click switching across all 6 Theme Cartridges and custom cartridges (`custom-theme-*`), live `.vibe` projection script editing, and snapshot persistence.
- **`WP-ETALON-03A` — Interactive Personal Preference, Sensory & Accessibility Questionnaire + Tailored Theme Option Recommender**:
  - Target files: `crates/webizen-ua-os/src/theme_discovery_questionnaire.rs` (new), `src/components/volumes/settings.rs`, `src/components/volumes/sanctuary.rs`.
  - Ask the user friendly plain-English questions about reading scale (`100%..165%`), contrast/glare comfort, color-vision profile (`Standard`, `Deuteranopia/Protanopia`, `Tritanopia`, `Monochrome`), vestibular/motion comfort (`Full 3D Kinetics`, `Gentle Damped`, `Strict Zero-Motion Static`), workspace density/style, and acoustic feedback.
  - Implement `recommend_environment_options(&UserEnvironmentQuestionnaire) -> [RecommendedEnvironmentOption; 3]` returning 3 tailored, ranked environment options (`Primary Tailored Match`, `High-Clarity Comfort Variant`, `Spatial Exploration Variant`) with instant one-click preview and adoption. Keep answers strictly local (`SENSITIVITY_PRIVATE`).
- **`WP-ETALON-03B` — Conversational LLM Theme Co-Designer, Fail-Closed WCAG/Affordance Verifier & Permissive Commons `.hmc` Theme Sharing**:
  - Target files: `crates/webizen-ua-os/src/llm_theme_codesigner.rs` (new), `src/components/volumes/settings.rs`, `src/components/volumes/poet_synth.rs`.
  - Allow the user to work conversationally with a local `.p64` or governed external MCP LLM (`SENSITIVITY_PUBLIC_ONLY = 0` gate enforced) to generate or refine a custom `ThemeCartridgeManifest` + `.vibe` projection script from natural-language prompts.
  - Gate every LLM-designed theme through a 3-stage fail-closed verifier: (1) human-rights nomenclature & IP-cleanliness check, (2) WCAG luminance contrast auto-repair ($\ge 4.5:1$ AA / $7.0:1$ AAA), and (3) `verify_cartridge_affordance_equivalence` + `etalon_wgsl_forge::validate_wgsl`.
  - Support exporting/sharing custom themes as signed **Permissive Commons** `.hmc` / `.q42` Theme Cartridge bundles (`SharedThemeCartridgeBundle` with 48-byte `NQuin48` author attribution and SHA3-256 digest) and importing shared themes into `CollectedInactive` state (`collect != activate`) for inspection before activation.

### Phase 2 — Full Etalon 3D GPU Pipeline Integration in `Aether`
- **`WP-ETALON-04` — Link `etalon-render` & Bind `.10d` `HOLO` (`0x0306`) Storage Buffers (`HolographicNodeGpu`)**:
  - Target files: `C:\github\Aether\packages\qualia-host\src\holographic_viewport.rs`, `src/components/viewports.rs`.
  - Convert decoded `M64HoloRecord` and `EulerianWavefieldRecord` slices into 96-byte `std430` `HolographicNodeGpu` storage buffers (`@group(1) @binding(0) var<storage, read> nodes: array<HolographicNodeGpu>`) and raymarch up to 64 3D SDF primitives per pass.
- **`WP-ETALON-04A` — EMF-First (`Q42EMF64` / `EmfPipeline`) + Zero-Heap `UiLayoutTree` Primary GPU UI Substrate (Demoting CSS to Fallback)**:
  - Target files: `C:\github\Aether\packages\native\src\emf_hud_substrate.rs` (new), `crates/webizen-ua-os/src/emf_ui_field.rs` (new), `src/components/viewports.rs`, `assets/shell.css`.
  - Implement `EtalonEmfUiSubstrate` combining `etalon_render::hud::UiLayoutTree` (`MAX_UI_NODES = 256` zero-heap flat-array solver + `HudVertex` generator), 64-byte `EmfFieldCell` wavefield grids (`attention_depth`, `epistemic_weight`, `topological_spin`, `manifold_curvature`, and Stokes polarization `stokes_q/u/v` rendered via `EmfPipeline` / `EMF_VOLUMETRIC_WGSL` with CIE 1931 XYZ 6-band spectral conversion), and 96-byte `HolographicNodeGpu` SDF nodes.
  - Strip decorative CSS bloat from `assets/shell.css`, demoting CSS to text-shaping, IME input, AccessKit semantics, and `RendererFallbackPolicy::CpuOnFailure` software fallback.
- **`WP-ETALON-05` — Bijective 3D GPU Object Picking (`etalon-render::picking`) in `aether-native`**:
  - Target files: `C:\github\Aether\packages\native\src\viewport.rs`, `src/components/spatial_stage_3d.rs` (new).
  - Add `pick_semantic_id(local_x, local_y, width, height) -> Option<u64>` to `EtalonSurfaceDelegate` and `EtalonVectorDelegate` using `etalon_render::picking::{encode_mesh_slot, resolve_pick}` and ray-intersection so clicking any 3D primitive dispatches its `SemanticIntent`.
- **`WP-ETALON-06` — Runtime Shader Synthesis & Optical Post-Processing via `etalon-wgsl-forge`**:
  - Target files: `crates/webizen-ua-os/src/forge_theme_shaders.rs` (new).
  - Synthesize and validate (`etalon_wgsl_forge::validate_wgsl`) the WGSL shaders for `RefractiveCurvedGlassBloom`, `WireframePhosphorScanline`, and `VolumetricSpectralHdr`.

### Phase 3 — The Three Interactive 3D Spatial Theme Stages
- **`WP-ETALON-07` — Spatial Theme Stage 1: Curved Refractive Glass Arc (`theme-holoslate`)**:
  - Target files: `src/components/themes/holoslate_arc_stage.rs` (new).
  - Concave 3D cylindrical translucent glass carousel with symplectic Verlet drag/scroll inertia (`oracle_spring_damper_force`, `oracle_verlet_step`), multi-depth frosted glass strata, and Fresnel edge glow.
- **`WP-ETALON-08` — Spatial Theme Stage 2: 3D Neon Wireframe Telemetry City (`theme-cyber-grid`)**:
  - Target files: `src/components/themes/cyber_grid_stage.rs` (new).
  - 3D isometric wireframe data-city where volumes, semantic file vaults, and `QuinWindow<64>` slots rise as translucent 3D geometric towers with packet light-lanes, fly-through camera docking (`oracle_skeletal_lerp_joint`), and phosphor scanline HUD framing.
- **`WP-ETALON-09` — Spatial Theme Stage 3: 360° Orbital 10D Volumetric Amphitheatre (`theme-spatial-commons`)**:
  - Target files: `src/components/themes/spatial_commons_stage.rs` (new).
  - 360-degree orbital ring of 3D SDF architectural pavilions raymarched on Etalon GPU with floating angled diegetic 3D holographic consoles (`hypermedia.layout.project_diegetic_surface`).

### Phase 4 — Spatial Acoustics, Synth `.hmc` Cartridge Packaging & Multi-Theme UAT Gate
- **`WP-ETALON-10` — Zero-Heap Procedural Acoustic & Binaural UI Feedback (`etalon-vibe` `SIGNAL_PROVIDER`)**:
  - Target files: `crates/webizen-ua-os/src/theme_acoustics.rs` (new).
  - Deterministic zero-heap binaural audio frames (`oracle_biquad_df1_step`, `oracle_render_binaural_frame`, `oracle_sonify_emf_envelope`) for each spatial theme archetype.
- **`WP-ETALON-11` — Synth 400-Morphism `.hmc` Theme Cartridge Exporter in `Poet Studio`**:
  - Target files: `crates/webizen-ua-os/src/poet_synth.rs`, `src/components/volumes/poet_synth.rs`.
  - Package custom `.vibe` theme cartridges into parity-verified `.hmc` bundles under the **Permissive Commons**.
- **`WP-ETALON-12` — Multi-Theme UAT Qualification & Affordance Equivalence Gate (`webizen-os-shell uat --all-themes`)**:
  - Target files: `src/uat.rs`.
  - Verify all 27 scopes, all 4 `F12` `DevInspectorDrawer` tabs, and all 6 Theme Cartridges (including 3D picked nodes) with `0 crashes`.
