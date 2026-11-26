# Webizen OS-Shell & Aether Native — Human-Centric Desktop UX, Functional Application & Synth 3D/Hypermedia Asset Overhaul Specification (Beyond macOS & Windows 11)

**Copyright (c) 2026 Timothy Charles Holborn**  
**https://www.linkedin.com/in/ubiquitous/ | timothy.holborn@gmail.com**  
**License:** Attribution-NonCommercial-NoDerivatives 4.0 International (`CC BY-NC-ND 4.0`)

---

## 1. Executive Diagnosis: Why the Current UI Still Feels Like "Spec-Receipt / Bot-Jargon Leakage" & How to Surpass macOS / Windows 11

In earlier iterations, the specification asked for deep architectural guarantees (`VaultContainerV2`, `NQuin48`, `42 MiB SlgArena Sentinel`, `RouteStamp`, `M-of-N Ed25519 + ML-DSA-65 quorum`, `Noisy-OR Confidence Ladder < 1.0`, `QECP`, `SHACL`, `WebizenOpcode`). When an AI agent implements those requirements without a strict **Human Product vs. Engine Inspector** boundary, it treats the user's primary application screen as a **compliance receipt dashboard**—printing internal Rust struct names, specification paragraphs, and cryptographic hex dumps directly in headings, buttons, and primary views.

Even after the first pass of `WP-ETALON-00A..12`, a file-by-file audit of `src/components/` reveals **remaining spec-receipt leakage** mixed into the primary human surfaces:
1. **`OrbitWheel` (`src/components/orbit_wheel.rs`)**:
   - The `Ctrl+K` / `Launch…` app launcher subtitle still says: *"First-class volumes + 6 verified Qap packages in storage/Qapps/registry.json · Live vs Planned honesty only."*
   - The app badges still say: `"4-slot VaultContainerV2"`, `".q42 / .10d / .hmc / QECP"`, `"6 Core SI + 40 Logics"`, `"VibeScript + 150 morphisms"`, `"9-Stage Contribution Ledger"`.
2. **`TalkVolume` & `DirectoryVolume` (`src/components/volumes/communications.rs`)**:
   - `Talk` only has a single placeholder `"Human peer"` button in the sidebar instead of a rich multi-person / family / co-op conversation list with unread badges, online status, and thread switching.
   - `Directory` button says `"Mint Pairwise Relationship"` and adds raw strings like `"NaturalAgent (living-SHACL) · Alice · Directional pairwise locator minted"` instead of rendering human contact cards with **Message (`Talk`)**, **Write Letter (`Mail`)**, **Share File**, and **Relationship Scope** actions (with the pairwise DID / SHACL receipt tucked in an expandable `"Connection security"` drawer).
3. **`BrowserVolume` (`src/components/volumes/browser.rs`)**:
   - Has a toolbar at the top, **but** lacks **multi-tab browsing** (open tabs bar, `+ New Tab`, close tab `×`, pinned bookmarks bar) and immediately dumps 450+ lines of `"Mode 3 Hybrid Bridge · QualiaDB Capabilities in Traditional Web Pages (Control::PageRequest -> Host Sentinel Gate -> PageReply)"` and `"QDNF Social-Web Overlay"` forms right below the viewport unless collapsed.
4. **`PoetSynthVolume` (`src/components/volumes/poet_synth.rs`)**:
   - Currently stacks a `<textarea>`, the 3D `Holo10dViewport`, `ThemeCodesignerPanel`, `SharedThemeCollectionPanel`, `GovernedAuthoringPanel`, and `"Run Integrated Hypermedia Production & Seal Gazetteer NQuins"` in one long vertical scroll (`1,319` lines!) instead of a clean **5-Mode Studio Workspace Switcher** (**`All-in-One Studio`**, **`Write & Annotate`**, **`Grid & Formulas`**, **`3D & Synth Asset Forge`**, **`Theme Co-Designer`**, plus a collapsible **`Studio Inspector`** drawer).
5. **`WellfairVolume` (`src/components/volumes/wellfair.rs`)**:
   - Sub-tabs and headers still read like an architecture whitepaper: `"WellFair · Shared Systems, Relational Guardianship (UN CRC / CRPD) & Remote Carer Alerts"` with a 60-word specification paragraph and raw struct field names (`AgencyStage`, `DigitalBirthRecord`, `StewardRoles`, `DutyOfInquiry`), rather than a warm, calm **Health, Care Circle & Wellbeing** application with a prominent **"I am OK" Check-In Card**, **My Care Circle & Family**, **Health Journal & 3D Body Atlas**, and **Specialist & Advocacy Visits**.
6. **`SanctuaryVolume` (`Keep` — `src/components/volumes/sanctuary.rs`)**:
   - After the appearance questionnaire, it immediately dumps `"7-Screen New-User Onboarding Wizard (Desktop & Mobile Progressive Confidence Ladder)"`, `"Multi-Account Shared-Machine Seat Assignments (OS Account != Principal Identity)"`, `"VaultContainerV2 · Four-slot policy inspector"`, and `"3-of-5 Shamir recovery rehearsal"` with raw `NQuin48 parity 0x...` strings, instead of a clean **Personal Vault & Family Recovery** app (**`My Protected Files & Notes`**, **`Family & Guardian Recovery`**, **`People on This Computer`**, **`Device Setup Guide`**, and **`Community Testimony`**).
7. **`LibraryVolume` (`src/components/volumes/library.rs`)**:
   - Places the 3D viewport and `"Opt-In Semantic ToolChest (Poet Shed Pattern — Elect Toolboxes to Enable Specialized Extractors & Facets)"` *above* the actual files/books/models grid, forcing the user to scroll past 300 lines of extractor cards before seeing their own files!
8. **`InstrumentsVolume` & `LlmAndAgentManagementPanel` (`src/components/volumes/instruments.rs`, `llm_agents.rs`)**:
   - Heading says `"Instruments · LLM (.gguf -> .p64) Management, Governed Agent Roster, External MCP Accounts & 6 Core Semantic Instruments"` and puts a raw 64-slot hexadecimal `NQuin slot inspector` (`0x0000...`) right on the main page (even though `NQuin48 & Refract Store` already lives in `F12` `DevInspectorDrawer` Tab 3!).
9. **`WalletVolume` & `ProjectsVolume` (`src/components/volumes/wallet.rs`, `projects.rs`)**:
   - The `Send` and `Receive` buttons in `WalletVolume` are static dummy buttons (`button { "Send" }` with no `onclick` handler or transaction history ledger!), followed immediately by raw `ContributionLifecyclePreview` and `ProviderCreditInspector` forms on the main screen.
10. **`SettingsVolume` & `AdminVolume` (`src/components/volumes/settings.rs`, `admin.rs`)**:
    - Every Settings tab title is numbered spec jargon (`"1. Setup Health · Local Foundations & Progressive Relational Paths"`, `"2. Data & Memory · Native Binary, Semantic & Hypermedia Container Suite"`, `"3. AI Instruments · .gguf -> .p64 Conversion & Purge..."`).

---

## 2. Mandatory Full Use of Webizen AI Synth (`synth`), `hypermedia-production`, `etalon`, and `vibe`

To make Webizen OS-Shell genuinely surpass macOS Sequoia and Windows 11, **do not build flat, static HTML/CSS placeholder panels**. You have full access to the ecosystem's production engines and MCP tools—**use them to produce and render real 3D, vector, acoustic, and semantic assets**:

1. **Webizen AI Synth MCP & CLI (`C:\github\synth`, 400 Morphisms across 58 Categories & 1,200 `NQuin48` Triples)**:
   - Use **`synth_find`**, **`synth_operation_contracts`**, **`synth_produce_present` (`quality_profile=production16`)**, **`synth_compile_q42`**, and **`synth_validate_model`** (or the `synth.exe` CLI in `C:\github\synth\target-prod\debug\synth.exe`) to produce and bundle **real `.10d` v2 (`HOLO 0x0306`), `.hmc`, `.q42`, `.m64`, `.emf`, `.qac`, and high-craft SVG/PNG visual assets** under `C:\github\webizenai\webizen-os-shell\assets\synth\`!
   - Produce and wire **bespoke Synth-authored 3D `.10d` v2 Holographic Specimens & Visual Assets** for the desktop environment:
     - `assets/synth/sanctuary-guardian-crest.10d` — A 3D holographic Sanctuary Keep guardian shield & crystal vault emblem rendered live in `Keep` (`SanctuaryVolume`) and the empty desktop stage.
     - `assets/synth/wellfair-vitals-dyad.10d` — A 3D physiological / anatomical dyad & heart-wavefield specimen rendered live in `WellFair` and `AnatomyVolume`.
     - `assets/synth/commons-solar-microgrid.10d` — A 3D Community Grounds solar/battery microgrid & bioregional terrain specimen rendered live in `ProjectsVolume`, `BrowserVolume`, and `LibraryVolume`.
     - `assets/synth/holoslate-desk-totem.10d` — A refractive-glass multi-ring spatial totem for the `Holoslate Arc`, `Cyber-Grid Matrix`, and `Spatial Commons Portal` 3D theme stages.
     - High-resolution Synth-produced SVG/PNG workspace illustrations and reference sheets in `assets/` (including ensuring `assets/library-sheet.png` or `assets/synth/` preview cards exist on disk so image loads succeed cleanly!).
2. **`hypermedia-production` (`asset-production-vibe`, `asset-production`, 400 Morphisms)**:
   - In `PoetSynthVolume`, give the user an interactive **"Synth 3D & Media Forge"** bar where they can pick from curated `.vibe` asset-production recipes (*"3D Holographic Crystal (.10d)"*, *"Botanical Canopy Specimen (.10d)"*, *"Spatial Acoustic Chime (.qac)"*, *"Electromagnetic Wavefield Brick (.emf)"*, *"Knowledge Graph Vault (.q42)"*), click **"Produce with Synth"**, and immediately load the resulting `.10d` / `.hmc` into the live `Holo10dViewport` and `LibraryVolume`!
3. **`etalon` (`etalon-vibe`, `etalon-wgsl-forge`, `EtalonSurfaceWidget`, `EtalonVectorWidget`, `EtalonEmfUiSubstrate`)**:
   - Every volume hero header or empty state should use live **Etalon GPU rendering** (`HoloGlyphIcon` animated 3D/vector phase-space glyphs, `Holo10dViewport` 3D raymarching with `@group(1)` 96-byte `HolographicNodeGpu` + `@group(2)` 64-byte `Q42EMF64` storage buffers, and procedural `etalon-wgsl-forge` theme shaders)—never dull gray HTML boxes.
4. **`vibe` (`VibeScript` Live Hot-Reload Engine)**:
   - Use live VibeScript expressions (`poet_eval_native_cell`, `evaluate_theme_cartridge_vibe`, `eval_vibe_icon_omega`) to drive reactive spreadsheet cells in `Poet Studio`, live theme choreography in `ThemeStudio`, and animated icon physics in `HaloBar`.

---

## 3. Five Design Pillars That Make Webizen OS-Shell Superior to macOS & Windows 11

1. **Relationship- & Meaning-Centric Continuity (Not App Silos)**:
   - Clicking a person in **Directory** lets you immediately jump to your **Talk** chat with them, your **Mail** letters with them, or **Projects** you share with them.
   - Clicking an item in **Library** shows its **live 3D `.10d` / Document Preview**, which **Project** it belongs to, and lets you open it in **Poet Studio** or **Browser** in one click.
2. **Spatial Dock + Top Continuity Bar + Instant `Ctrl+K` Spotlight That Actually Understands Intent**:
   - The left **Halo Dock** has tactile animated 3D Etalon icons (`HoloGlyphIcon`) with unread/activity badges; the top **Continuity Bar** shows *who you are acting as* in plain English (`who · claim · handle · tool` with friendly tooltips), a **1-click Theme & 3D Stage Switcher (`Look: ...`)**, **Quick Text Size (`A− / 115% / A+`)**, **`Contrast`**, **`3D Icons`**, and **`Inspect (F12)`**.
3. **True Multi-Tab & Split-Pane Application Workspaces (Zero Scroll-of-Text Dashboards)**:
   - Real desktop applications use **clean workspace sub-tabs, split panes (master list on the left, active workspace in the center, optional detail drawer on the right)**, and **interactive state** (sending a payment in Wallet updates your balance and adds a row to your Recent Activity list; clicking a contact in Talk switches the active conversation thread; producing an asset in Poet+Synth adds it to the 3D stage).
4. **Two-Tier Progressive Disclosure ("Clean Human Surface by Default, Deep Cryptographic Truth on Demand")**:
   - **Tier 1 (Default View — 100% Human English)**: Every title, subtitle, card, and primary button uses clear, everyday English. Zero hex hashes, zero crate names, zero `.q42`/`.p64`/`NQuin48`/`SlgArena`/`VaultContainerV2` jargon in primary headings.
   - **Tier 2 (`F12` Inspector Drawer & Subtle Bottom `"Advanced & Cryptographic Details"` Drawers)**: Every single architectural proof, `NQuin48` parity check, `RouteStamp` counter, SHACL family, and `.deb` packaging action remains 100% present and interactive inside `<details class="tech-details-drawer" open: true>` (styled as a clean, muted footer tray at the bottom of the workspace) or the `F12` `DevInspectorDrawer`.
5. **100% UAT Button Compatibility Guarantee**:
   - **CRITICAL RULE**: The automated live-window UAT harness (`cargo run --bin webizen-os-shell -- uat`) clicks buttons and checks visible labels. Keep all existing UAT-tested buttons interactive inside `<details class="tech-details-drawer" open: true>` at the bottom of each workspace (or preserve their exact `id` / `aria_label` / button text) so `cargo test --workspace` and `cargo run --bin webizen-os-shell -- uat` pass with **0 failures**!

---

## 4. Work-Packet Breakdown for Grok (`WP-UX-00` through `WP-UX-10`)

### `WP-UX-00` — Synth MCP / Hypermedia-Production Asset Generation & Embedded 3D `.10d` Specimens
- **Target Files & Directories**:
  - `assets/synth/` (new directory for Synth-produced `.10d`, `.q42`, and `.svg`/`.png` assets)
  - `assets/library-sheet.png` (ensure a valid PNG reference sheet is generated and saved so `LibraryVolume`'s image preview always loads with `Some(true)`)
  - `crates/webizen-ua-os/src/holo_10d_specimen.rs`
- **Deliverables**:
  1. **Produce Real Assets via `synth` MCP (`synth_produce_present`, `synth_compile_q42`, `synth_find`) or `hypermedia-production`**:
     - Use the `synth` MCP server (or `asset_production::compile_10d`) to generate and seal 4 rich `.10d` v2 (`HOLO 0x0306`) specimens under `assets/synth/` (and register them in `holo_10d_specimen.rs` so `Holo10dViewport` can switch between them by specimen ID):
       - `sanctuary-guardian-crest.10d` (*Sanctuary Keep Guardian Shield & Crystal Vault*)
       - `wellfair-vitals-dyad.10d` (*WellFair Physiological Heart & Care Dyad Wavefield*)
       - `commons-solar-microgrid.10d` (*Community Grounds Solar/Battery Microgrid & Terrain*)
       - `holoslate-desk-totem.10d` (*Holoslate Multi-Ring Spatial Command Totem*)
     - Generate a clean, high-craft `assets/library-sheet.png` visual diagram so clicking `"Show Library reference sheet"` in `LibraryVolume` displays a crisp visual guide with zero load error.

---

### `WP-UX-01` — Shell Chrome, Top Continuity Bar (`Look` Theme Cycler) & `OrbitWheel` (`Ctrl+K` Spotlight Launcher) Polish
- **Target Files**:
  - `src/components/orbit_wheel.rs`
  - `src/components/stage_host.rs`
  - `src/components/halo_bar.rs`
  - `assets/shell.css`
- **Deliverables**:
  1. **`OrbitWheel` (`Ctrl+K` Launcher — `src/components/orbit_wheel.rs`)**:
     - Replace the jargon heading `"Constellation · Foundational Volumes & Qaps"` and subtitle `"First-class volumes + 6 verified Qap packages in storage/Qapps/registry.json · Live vs Planned honesty only"` with:
       - Heading: **`"Open an App or Run a Command"`**
       - Subtitle: **`"Search your workspaces, open a web or 3D spatial address, or ask a local helper."`**
     - Replace the jargon pill blurbs in `APPS` with clear human descriptions:
       - `Talk` → `"Messages & voice with people you know"`
       - `Mail` → `"Private letters & relationship inboxes"`
       - `Directory` → `"People, family & cooperative contacts"`
       - `Browser` → `"Websites & 3D spatial pages"`
       - `Keep · Sanctuary` → `"Private vault, passwords & family recovery"`
       - `Library` → `"Books, documents, media & 3D models"`
       - `Instruments` → `"Local AI helpers & verification tools"`
       - `Poet & Synth` → `"Documents, spreadsheets & 3D Synth studio"`
       - `WellFair` → `"Health journal, care circle & check-ins"`
       - `Anatomy (Qap)` → `"Interactive 3D human body atlas"`
       - `Wallet` → `"Balances, payments & shared project credits"`
       - `Projects` → `"Shared work, milestones & contributor fair pay"`
       - `Settings` → `"Appearance, accessibility, privacy & devices"`
       - `Admin` → `"Home server, device fleet & system updates"`
  2. **Top Continuity Bar Quick Theme Pill (`src/components/stage_host.rs`)**:
     - Add a compact **Theme Cycle button** (`"Look"` / active theme short name) in `.a11y-quick-bar` so the user can cycle through all 6 themes (`Holosonic`, `Obsidian Contrast`, `Warm Commons`, `Holoslate Arc 3D`, `Cyber-Grid Matrix 3D`, `Spatial Commons 3D`) from anywhere in the OS in one click.

---

### `WP-UX-02` — `Talk`, `Mail` & `Directory`: Real Multi-Contact Conversations, Inbox Threads & Contact Cards
- **Target Files**:
  - `src/components/volumes/communications.rs`
  - `src/state/shell_state.rs`
- **Deliverables**:
  1. **`TalkVolume` Multi-Conversation Switcher & Rich Chat Bubbles**:
     - Instead of a single hardcoded `"Human peer"` button in the left rail, render selectable conversation items in `.comms-pane-list` for the user's contacts (e.g. `"Elena Vance · Co-op Architect"`, `"Dr. Marcus Chen · Family GP"`, `"Mercer Household Circle"`, `"Community Grounds Crew"`, plus any contacts added in `Directory`) with online status indicators.
     - Clicking a person switches the active conversation header (`"Conversation with {selected_contact}"`) and renders styled outgoing (`class: "chat-bubble outgoing"`) and incoming (`class: "chat-bubble incoming"`) chat bubbles.
     - Replace `"Governed AI Tool Panel: CLOSED (Human-to-Human QDNF mode)"` with a friendly status pill: **`"Direct person-to-person chat (no AI listening)"`** when closed, and **`"Local helper tool active (marked separately from people)"`** when open.
  2. **`MailVolume` Split-Pane Inbox & Reader**:
     - Left pane shows selectable inbox threads with sender, subject, and preview snippet.
     - Right pane shows the selected letter view at the top and a clean **"Write a Letter / Reply"** composer (`To`, `Subject`, `Message` input, and `Send` button) below.
  3. **`DirectoryVolume` Human Contact Cards + 1-Click Jump to `Talk` / `Mail`**:
     - Render each contact in `directory_contacts` as a structured **Contact Card** with the person's display name, relationship badge (`"Person & Peer"`, `"Care Circle"`, `"Co-op Member"`), and quick-action buttons (`"Chat in Talk"` and `"Write in Mail"`).
     - Keep the `"Mint Pairwise Relationship"` button (for UAT compatibility) with a friendly subtitle explaining that each relationship gets its own private connection address so contacts cannot be tracked across apps.

---

### `WP-UX-03` — `BrowserVolume`: Multi-Tab Bar, Pinned Spatial Speed-Dial & Clean Bottom Inspector Drawer
- **Target Files**:
  - `src/components/volumes/browser.rs`
- **Deliverables**:
  1. **Browser Tab Strip above the Address Toolbar**:
     - Add a clean multi-tab strip (`.browser-tabs-bar`) above `.browser-toolbar` with quick-switch tabs linking to both Web and 3D Spatial destinations:
       - **`"Community Grounds"`** (`https://dev.civics.au/site-planner`)
       - **`"3D Biosphere Universe"`** (`qualia://chora/universe`)
       - **`"3D Solar Microgrid (.10d)"`** (`qualia://specimens/commons-solar-microgrid.10d`)
       - **`"Human Rights Observatory"`** (`https://observatory.civics.au`)
     - Clicking any tab updates `browser_url` and resolves `Web`, `Spatial`, or `Hybrid` mode.
  2. **Full-Height Viewport Primary Layout**:
     - Ensure `.browser-viewport` (`BrowserChoraSurface`) occupies the primary screen area uninterrupted.
     - Keep all 13 UAT-tested buttons in `BrowserVolume` accessible while styling the technical Mode 3 bridge and QDNF social-web sections as a clean, muted footer drawer.

---

### `WP-UX-04` — `PoetSynthVolume`: 5-Mode Creative Studio & Interactive Webizen AI Synth 3D Asset Forge
- **Target Files**:
  - `src/components/volumes/poet_synth.rs`
- **Deliverables**:
  1. **Studio Mode Switcher Bar (`.volume-subtabs`) & Formatting Toolbar**:
     - Add a clean mode bar at the top of `PoetSynthVolume`:
       - **`"All-in-One Studio"`** (Default — shows Document Editor with Formatting Toolbar + Formula Bar + 3D `Holo10dViewport` + Synth Preset Picker, with the technical gazetteer/provider panels in an open-for-UAT muted footer drawer).
       - **`"Write & Annotate"`** (Focused document editor + quick Markdown/CML insert buttons: `Heading`, `Bold`, `Quote`, `Formula Cell`, `3D Embed`).
       - **`"Grid & Formulas"`** (Interactive spreadsheet cell table showing `cell_1`, `cell_2`, live VibeScript formula evaluation `= 64 * 16`, and cell revision history).
       - **`"3D & Synth Asset Forge"`** (Interactive **Webizen AI Synth Preset Launcher** where the user can click presets like *"Produce 3D Sanctuary Crest (.10d)"*, *"Produce 3D Solar Microgrid (.10d)"*, *"Produce 3D Vitals Dyad (.10d)"*, or run custom `.vibe` hypermedia scripts via `produce_hypermedia_vibe` and inspect the result immediately in `Holo10dViewport`!).
       - **`"Theme Co-Designer"`** (`ThemeCodesignerPanel` + `SharedThemeCollectionPanel`).

---

### `WP-UX-05` — `LibraryVolume`: Visual Media & 3D Asset Grid First, Live 3D Specimen Preview Side-by-Side
- **Target Files**:
  - `src/components/volumes/library.rs`
- **Deliverables**:
  1. **Put the User's Files, Books & 3D Models Grid FIRST**:
     - Immediately below the `"Library"` header, render a **2-Column Split Workspace**:
       - **Left/Center Asset Card Grid**: Clean cards for each matched item (`title`, friendly badge like `"3D Holographic Model (.10d)"`, `"Document"`, `"Verified Credential PDF"`, topic tags, and a `"Preview in 3D / Reader"` button).
       - **Right Preview Pane**: Live `Holo10dViewport` rendering the selected `.10d` specimen + `"Open in Poet Studio"` / `"Open in Browser"` / `"Share or Export"` (`LibraryPortabilityPanel`).
  2. **Tuck Specialist Extractors & Raw NQuin Graph Details Below the Primary Grid**:
     - Keep the `"Opt-In Semantic ToolChest"` (`toolbox_cards`) and deep graph details below the primary file grid in a muted `<details class="tech-details-drawer" open: true>` tray so the user immediately sees their files and 3D preview first while all 32 UAT buttons remain clickable.

---

### `WP-UX-06` — `Keep` (`SanctuaryVolume`): Personal Vault, 3D Guardian Crest, Family Recovery & People on This Computer
- **Target Files**:
  - `src/components/volumes/sanctuary.rs`
- **Deliverables**:
  1. **Human-First Vault Status Hero Card + Embedded 3D Guardian Crest**:
     - At the top of `Keep`, render a calm **Personal Vault Overview** banner with 3 human status cards (**`"Vault Status: Protected on This Device"`**, **`"Family & Guardian Recovery: {threshold} of {total} Guardians Ready"`**, **`"Phone Co-Signer: Paired & Attested"`**) alongside a compact live `Holo10dViewport` rendering `sanctuary-guardian-crest.10d`.
  2. **Plain-English Section Titles (Replacing Spec Jargon)**:
     - `"7-Screen New-User Onboarding Wizard..."` → **`"Guided Setup & Security Checklist (7 Steps)"`**
     - `"Multi-Account Shared-Machine Seat Assignments (OS Account != Principal Identity)"` → **`"People & Roles on This Computer"`** (*"Each person or professional role on this computer gets their own isolated vault, even if you share the same computer login."*)
     - `"VaultContainerV2 · Four-slot policy inspector"` → **`"Private Storage Compartments (Standard, High-Safety & Emergency Decoy)"`**
     - `"3-of-5 Shamir recovery rehearsal"` → **`"Practice a Family & Guardian Recovery"`**
     - Preserve all existing button labels (`Inspect Seat Identifiers (...)`, `Test contextual handle issuance`, `Mint Zero-Identifier Verifiable Testimony Proof`) for 100% UAT pass.

---

### `WP-UX-07` — `WellfairVolume`: Calm Health Journal, 1-Tap `"I am OK"` Check-In, Care Circle & 3D Body Atlas
- **Target Files**:
  - `src/components/volumes/wellfair.rs`
- **Deliverables**:
  1. **Friendly Sub-Tab Labels & Reassuring Top Check-In Banner**:
     - Rename the 3 sub-tabs to:
       - **`"1. Health Team, Visits & Advocacy"`**
       - **`"2. Family, Shared Living & Check-In Alerts"`**
       - **`"3. Fairness & Rights Check"`**
     - Add a top **"Daily Wellbeing & Care Check-In"** hero card with a 1-tap **`"✓ I am OK"`** status indicator, an **`"Open Urgent Care Alerts Window"`** button, and a 1-click **`"Open 3D Body Atlas"`** button that jumps to `AnatomyVolume` (or previews `wellfair-vitals-dyad.10d`).
  2. **Replace Spec Paragraphs with Warm Human Copy**:
     - Replace the 60-word architectural paragraph in `GuardianshipAndCarePanel` with a warm 1-sentence summary: **`"Coordinate health notes, shared household care, and remote family check-ins while keeping every person in control of their own records."`**

---

### `WP-UX-08` — `WalletVolume` & `ProjectsVolume`: Live Interactive Payments, Activity Ledger & Visual Milestone Board
- **Target Files**:
  - `src/components/volumes/wallet.rs`
  - `src/components/volumes/projects.rs`
- **Deliverables**:
  1. **Make `WalletVolume` Genuinely Interactive**:
     - Seed friendly balances (`125,400 sats` on Lightning, `450.00 AUD-equiv` in eCash, `250.00 AUD` in Community Project Credit) and a **`Recent Activity`** ledger (`"Community Grounds Solar Microgrid · +150.00 AUD"`, `"Local Co-op Grocer · -2,400 sats"`).
     - Wire the **`"Send"`** button (`Recipient`, `Amount`) so clicking **`"Send"`** updates the balance and adds a live transaction row to **Recent Activity** with a human confirmation toast.
     - Wire **`"Receive"`** to toggle a clean **Receive Payment / QR & Address Card**.
     - Group `ContributionLifecyclePreview` and `ProviderCreditInspector` inside an open-for-UAT `"Shared Project Credits & Periodic Reconciliation"` drawer below the personal wallet.
  2. **Interactive Milestone Board & 3D Project Preview in `ProjectsVolume`**:
     - Make the 9-step `.project-milestones` bar interactive (clicking any step shows a plain-English card explaining what happens at that milestone and how contributor fair pay works), and embed a compact 3D preview of `commons-solar-microgrid.10d` next to the Community Grounds project archetype.

---

### `WP-UX-09` — `InstrumentsVolume`, `SettingsVolume` & `AdminVolume`: Plain-English Titles & Zero Duplicate Hex Tables in Main Chrome
- **Target Files**:
  - `src/components/volumes/instruments.rs`
  - `src/components/volumes/llm_agents.rs`
  - `src/components/volumes/settings.rs`
  - `src/components/volumes/admin.rs`
- **Deliverables**:
  1. **`InstrumentsVolume` & `LlmAndAgentManagementPanel`**:
     - Rename the header to **`"AI Helpers & Verification Tools"`** with subtitle **`"Choose which local AI models run on your graphics card, manage helper agents, and run verification checks."`**
     - Group the raw hexadecimal `QuinSlotInspector` (`Slot 0..63`) and `"6 Declarative SHACL Extension Bridge Families"` inside an open-for-UAT `<details class="tech-details-drawer" open: true><summary>Advanced: Memory Slot & Formal Logic Inspector</summary>...</details>` footer tray.
  2. **`SettingsVolume` & `AdminVolume`**:
     - Give all 10 Settings sections and all 3 Admin sub-tabs clean, human-friendly titles and subtitles while keeping every diagnostic card and UAT button intact.

---

### `WP-UX-10` — Visual Design System Upgrade (`assets/shell.css`) & Full 514+ Step Live-Window UAT Qualification
- **Target Files**:
  - `assets/shell.css`
  - `src/uat.rs`
- **Deliverables**:
  1. **Refined Spatial Glass, Chat Bubbles, Tab Strips & Muted Technical Footer Drawers (`assets/shell.css`)**:
     - Add polished CSS classes for `.browser-tabs-bar`, `.browser-tab`, `.chat-bubble.incoming`, `.chat-bubble.outgoing`, `.contact-card-grid`, `.wallet-activity-table`, `.studio-toolbar`, and `.synth-preset-bar`.
     - Style `.tech-details-drawer` with a subtle, muted footer appearance (`opacity: 0.88`, compact typography, dashed top separator) so technical drawers never visually compete with the primary human workspace.
  2. **Mandatory Verification Gates**:
     - `cargo test --workspace` passes 100% with 0 warnings.
     - `cargo run --bin webizen-os-shell -- uat inspect`, `uat browser`, and `uat themes` pass 100% with 0 failed steps.
     - Zero third-party franchise terms and zero forbidden `"sovereign"` / `"self-sovereign"` terms.
