# Do first: Aether slice, then Guest chrome. Anvil is a parallel track.

**Date:** 2026-11-20  
**Status:** Planning  
**Parent design:** [00-dual-sku-design.md](00-dual-sku-design.md) (revision 3g)

This note answers: *you mentioned blockers (Anvil, Aether) — what needs to be done, and should that be first?*

**Yes for Aether. No for Anvil, as a Guest-chrome gate.**

---

## Verdict

| Item | Blocks first visible product (C1)? | Do first? |
|---|---|---|
| **Aether B1a** — native window/lifecycle **without** Chora, committed or sliced so OS-shell can path-pin it | **Yes.** C1 cannot paint HaloBar/OrbitWheel until this exists. | **Yes. This is the critical path.** |
| Aether full native production (WP00–WP17, pixels, device-loss, frozen v4, WDAC, Vibe eval, Chora frames) | **No.** Phase 1 Anatomy is a native-RSX card; `Document::eval` may stay `NoOpDocument`. | Do **not** wait for the 51-job overhaul. |
| **Anvil** live host profile / `anvil-scraper` NodeProfile64 | **No.** F4 overlay may ship without Anvil. | Parallel track. **Required before any “hardware-attested vault” claim.** |
| QualiaDB dirty / behind origin | **No** for C1 (C1 paints with no Qualia). **Yes** for C0/C2 install. | After B1a starts; D0/D4 can overlap C1. |
| Engine worker leak (`Aether/packages/chora-worker` path-deps `servo` under CC) | **No** for C1. **Yes** for the first worker artefact (E0). | After B1a; E0 builds the worker from the MPL engine tree. |
| Debian Trixie overlay, WSL, Poet | **No.** | After C3 (Poet after Guest can load apps). |

First visible product, from the design: Windows Guest that paints halo/orbit (**C1**), supervises `{install}/qualia-cli daemon :4242` (**C0**), installs Anatomy into `{storage}/Qapps`, opens a native-RSX card “installed; 3D view later” (**C2/C3**).

---

## Sequence (do this order)

```text
Wave 0 — hygiene (parallel, days)
  A1 OS-shell vocab
  A2 Tauri vocab (freeze features)
  A3 relative Qualia pins; drop unused OS-shell Qualia crates
  A4 Aether AGENTS.md line 78
  A5 record Aether HEAD d5ccce3 + dirty hashes + optional .cargo/config.toml [patch]
  A7 LICENSE-BOUNDARY (does not block C1; needed before D5/C6)
  G0 Synth LICENSE/Cargo.toml → CC BY-NC-ND 4.0 (does not block C1)

Wave 1 — the actual blocker (must finish before C1)
  B1a  Commit or slice Aether native lifecycle WITHOUT Chora
       packages/native/** minus chora feature, packages/native-dom/**
       Acceptance: a window opens, redraws, tears down on Windows.
       Do not add chora_frame.rs. Do not claim pixels. Do not wait for frozen v4.

Wave 1 parallel (does not gate C1)
  Anvil live host profile (see below)
  D0 LaunchPlan library (no APP_STATE)
  D4 Anatomy seed install path
  F1 webizen-host StrictRetail (VPS; not Guest)

Wave 2 — first visible product
  C1  webizen-os-shell --features native paints HaloBar/OrbitWheel (depends B1a only)
  C0  LoopbackHost / qualia-cli sidecar (depends A3, D0)
  C5  window close/resize, no tray

Wave 3 — installable Qap
  C2 + C3 + D4  Anatomy native-RSX card
  C7 updater
```

If B1a cannot yield a window that C1 accepts, **then** use the time-boxed **C1-desktop** off-ramp (Aether-desktop/Wry at committed `d5ccce3`). That off-ramp is not the default and does not reintroduce Tauri.

---

## Aether: what is wrong, and what “done enough” means

### What is true on this machine (2026-11-20)

- HEAD `d5ccce3` (2026-11-06) is the published crate rename. **GitHub `d5ccce3` is not native-ready.**
- Working tree is **dirty** (~40 tracked files) plus untracked native/Chora/host work. `packages/native/src/chora_frame.rs` is **untracked**.
- Workspace **excludes** `qualia-host`, `chora-client`, `chora-worker`.
- Native `Document::eval` is still `NoOpDocument`.
- Tracker: `Aether/notes/architecture/14-IMPLEMENTATION-TRACKER.md` and packets `notes/implementation/WORK-PACKETS-2026-11-15.md` (WP00–WP17).
- Windows Application Control (WDAC) blocked frozen-v4 `getrandom` 0.3.4 build scripts. That blocks **frozen canonical host**, not a live-sources B1a window.
- Blitz Stylo/ICU vs Chora SpiderMonkey cannot share a process. Copied-frame worker is already the design (KD-6). Do not “fix” that in-process for C1.

### What C1 actually needs from Aether (B1a)

A **path-pinnable** native backend that can:

1. Open a window (winit/Blitz).
2. Run the RSX scheduler (signals, launch, assets/CSS).
3. Paint **some** native frames of HaloBar + OrbitWheel.
4. Close without hanging.

It does **not** need: Chora pixels, JS/Vibe eval, tray, frozen v4, WDAC-cleared getrandom, device-loss, installed artefacts, EMF/QAC, or WP13 signed end-to-end.

### Concrete B1a work

1. **A5 first:** write a pin file listing `d5ccce3` plus `git hash-object` of dirty tracked paths. Optional `.cargo/config.toml` `[patch]` recipe. No `WEBIZEN_QUALIA_ROOT` Cargo interpolation.
2. **Slice** `packages/native` / `packages/native-dom` so OS-shell can depend on them **without** enabling a `chora` feature and without requiring excluded crates.
3. **Prefer a real commit** of that slice over an eternal dirty pin. Do not commit unless Timothy asks; the planning default is “prefer B1a commit over eternal dirt”.
4. Leave `chora_frame.rs` untracked until B1b.
5. Keep Synth on the committed desktop pin `d5ccce3` (G4) until it opts into the B1a rev.

### Do not treat as C1 blockers

- WP04 canonical host / signed native data lifecycle
- WP06–WP07 Chora worker / frame identity
- WP11–WP12 EMF/QAC
- WP14 GPU failure matrix
- WP16 distribution/release qualification

Those stay on the Aether overhaul tracker. This programme consumes a **slice**.

---

## Anvil: required for attested vaults, not for a desktop window

### What is true

- Tree: `C:\github\anvil` (`anvil-core`, `anvil-gpu`, `anvil-npu`, `anvil-wasm`, `tools/anvil-scraper`).
- Scraper compiles `schemas/hardware_ontology.yaml` into `assets/anvil_ontology.q42` from seeded SKUs. **N0/N1 (local, uncommitted):** `NodeProfile64` is a 64-byte `NP64` artefact in `anvil-core`. `anvil-scraper probe` writes `assets/anvil_node_profile.np64` from this box (CPUID, display PCI, IOMMU table presence). Live `*.np64` is gitignored.
- **N2 (local, uncommitted):** `qualia-core-db` path-deps `anvil-core` `vault-security`. `identity::anvil_policy` reads `anvil_node_profile.np64`; missing/invalid file is `AttestationClaimError` (fail closed). Advertising requires IOMMU + GPU UUID + tier ≤ 1. This box’s N1 profile does not advertise (no IOMMU).
- **N3 (local, uncommitted):** Etalon path-deps `anvil-gpu`. HAL reads `VkPhysicalDeviceIDProperties::deviceUUID`. `GpuAdapterCaps::anvil_workgroup_size` calls `hints_for_pci`. Qualia `persist_gpu_uuid` overlays the UUID onto the existing np64 (zeros do not wipe a previous bind). `shared_gpu` does not call `anvil_bind` against git pin `a8f12b3`. DXGI LUID is not this slot.
- **N4 (local commits):** Anvil `4d8e5be`, Qualia `eb9070914`, host `4e631d8`, Etalon N3 `6df7e05`. F4 `GET /sanctuary/status` fail-closes. Host vibe locked 0.0.41 via vibe-script → Qualia `vibescript-abi` patch. This box still must not advertise (no IOMMU).
- **C1 (verified on this box):** `webizen-os-shell` path-pins Aether B1a native (CPU softbuffer + vendor Blitz patch). HaloBar/OrbitWheel paint. StageHost stays a placeholder. Window title is **Webizen OS-Shell**. `WEBIZEN_C1_SMOKE=1` printed “native window opened, HaloBar/OrbitWheel painted, process exited.” Qualia is not linked.
- QualiaDB Sanctuary policy consumes Anvil facts (IOMMU, GPU UUID, sealed anchor, `attestation_tier<=1`) **and** HumanAgencyConsent. N2 added the `anvil-core` `vault-security` path dep.
- Etalon is the intended GPU consumer of `anvil-gpu`.

### What must exist before any attested-vault advertising

1. **Live scrape** of *this* box (not only seeded SKUs): adapter UUID, identifier-tier slots, IOMMU/virt flags Anvil already models.
2. A **stable host profile artefact** (design name: NodeProfile64 — define the layout in Anvil, 64-byte or documented size, written beside the ontology SuperBlock). `anvil-scraper` (or a `probe` subcommand) emits it.
3. QualiaDB `probe_host` reads that profile; Sanctuary fail-closed if it is missing when attestation is claimed.
4. Etalon `bind_gpu_device_uuid` against the same facts.
5. Overlay/VPS copy: **do not** put “hardware-attested vault” in F4 docs or UI until (1)–(3) pass on that box.

### What Anvil must not delay

- C1 chrome
- C0 daemon
- Anatomy card
- F4 overlay packages (ship without Anvil; omit attestation language)

### Suggested Anvil packets (parallel to Wave 1)

| ID | Work | Depends |
|---|---|---|
| N0 | Document profile layout (NodeProfile64 or replacement name) in Anvil; no Qualia types | — |
| N1 | `anvil-scraper probe` (or equivalent) writes live profile + keep seeded ontology | N0 |
| N2 | Qualia `anvil-core` `vault-security` dep; Sanctuary reads profile; no advertise-without-file | N1 |
| N3 | Etalon bind to live `deviceUUID` from Anvil | N1 |

N0–N1 can start **now**, in parallel with B1a. They are a different repo and a different claim.

---

## Other incompletes (do not reorder the critical path)

| Incomplete | Effect |
|---|---|
| QualiaDB dirty, behind origin by 2 | Record in live-sources CI. C0/C2 need `install_package_atomic` as it exists today. |
| OS-shell still Dioxus 0.8-alpha, StageHost placeholder | That **is** C1/C2 work, after B1a. |
| Two old desktop trees, 2026-10-04 | Tauri stays a behaviour reference until C1 paints. |
| Synth crate files may still stamp Apache-2.0 | G0; notices already CC BY-NC-ND 4.0. |
| webizen-host `Q42\x04` facsimile, `PermissiveTest` | F1/F2 before any network-facing Appliance. Not C1. |

---

## Acceptance checks for “Wave 1 done”

**B1a done when:**

- OS-shell (or a tiny Aether example) path-depends local `aether` + `aether-native`.
- `cargo run --features native` (or the example) opens a window on Windows and paints something other than a blank hang.
- No Servo-derived old-web engine linked. No `qualia-host` workspace member required.
- Pin file lists the exact Aether rev + dirty hashes used.

**Anvil track “ready to advertise” only when:**

- A live profile file exists on the target box from a probe of that box.
- Qualia Sanctuary refuses attestation claims without it.
- Marketing/docs for F4 do not say “hardware-attested” until then.

Until B1a’s window exists, do not start C1 RSX mapping, Chora worker packaging, or Appliance images as the main effort.
