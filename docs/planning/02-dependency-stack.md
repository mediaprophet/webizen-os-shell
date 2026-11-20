# Dependency stack: work to get desktop, OS, local LLM, and Poet

**Date:** 2026-11-20  
**Status:** Planning + Wave 0 started  
**Parents:** [00-dual-sku-design.md](00-dual-sku-design.md), [01-do-first-blockers.md](01-do-first-blockers.md)  
**Pins:** [03-pin-snapshot.md](03-pin-snapshot.md)

This is the saved work list for **desktop / User-Agent overlay**, then the **dependency stack** that has to move first: Aether, Anvil, Etalon, hypermedia-production, Synth, local LLM, Poet.

Do not use “sovereign” / “self-sovereign” in new copy.

---

## Layer map

```text
Guest chrome / Appliance overlay     webizen-os-shell  (C1: Aether native CPU softbuffer)
        │
        ▼
Aether native window                 C:\github\Aether   dirty, unpublished
        │
        ▼
Etalon GPU / inference               C:\github\etalon   0.0.41.3  (Qualia pin 1 commit behind)
        │
        ├── Anvil hardware facts     C:\github\anvil    seeded SKUs, no consumers
        │
        ▼
QualiaDB graph, daemon, poet_host,   C:\github\qualiaDB 0.0.41.3
GGUF/P64 decode, Qap install
        │
        ├── hypermedia-production    morphisms only (no LLM, no Poet shell)
        │
        ▼
Synth :8787 + MCP                    produce + vault listing; does not run models
        │
        ▼
Poet                                 several trees; not launched in Phase 1
```

---

## What “desktop and OS work” still is

From the dual-SKU design. **Critical path is B1a → C1.** Everything else is either hygiene, parallel, or later.

| Wave | IDs | Outcome |
|---|---|---|
| **0 Hygiene** | A1–A5, A7, G0 | Vocab, relative pins, Aether dirty hashes, licence notices |
| **1 Blocker** | **B1a** | Aether native window **without** the old-web engine, pinnable by os-shell |
| **1 parallel** | Anvil N0–N1, D0, D4, F1 | Live host profile; LaunchPlan; Anatomy seed; host auth |
| **2 Visible** | C1, C0, C5 | Halo/orbit paint; `qualia-cli :4242`; no tray |
| **3 Qaps** | C2, C3, D4, C7 | Anatomy native-RSX card; updater |
| **Later** | E0 worker, F4 overlay, H Poet, local-LLM host bind | Old-web engine worker (`chora_*` is a code id); Debian overlay; Poet programme; Synth binds Qualia inference |

---

## Two webs (do not collapse these)

| Web | Product | Engine |
|---|---|---|
| **Human-centric** | QualiaDB, VibeScript, Aether native RSX, Qaps, **Chora** (spatial commons / `qualia://chora/universe`) | Aether + Qualia. Not a browser engine. |
| **Old web** | **Webizen Web Browser** (separate Qap) | **Servo-derived MPL engine** (repo currently `WebCivics/chora` — that *name* is the defect). Sibling worker. Not WebView, not Wry, as the intended engine. |

See [04-chora-name-and-servo-engine.md](04-chora-name-and-servo-engine.md). QualiaDB already shipped **Chora** and **Web Browser** as two QApps (Ctrl+2 vs Ctrl+3). Servo was experimental for the Web Browser. Ring-fence the engine under MPL. Do not redefine Chora as “Servo updates”.

## Aether (must go first for chrome)

**State.** HEAD `d5ccce3` is the crate rename (2026-11-06). Working tree is dirty (~40 tracked, hundreds untracked). Native/Etalon/old-web-engine overhaul is **unpublished**. Synth pins GitHub `d5ccce3`, **not** the dirty tree. Os-shell still depends on **DioxusLabs dioxus** `0.8.0-alpha.1`.

**C1 path-pins B1a.** HaloBar/OrbitWheel paint on Aether native; Qualia stays C0/C2.

### B1a — native lifecycle without the old-web engine

- Slice `packages/native` + `packages/native-dom` with optional engine feature `chora` **off** (`chora` here is a **code identifier**, not the Qualia Chora product).
- Include untracked files the dirty build already needs (`asset_access.rs`, vendored Blitz/hypermedia patches) **or** stop depending on them.
- Do **not** add `chora_frame.rs` (that is B1b).
- Acceptance: Windows window opens, redraws, tears down. No Servo-derived engine linked. No `qualia-host` workspace member.
- Prefer a real commit over an eternal dirty pin. Do not commit unless asked.
- `Document::eval` may stay `NoOpDocument`.

### Do not wait on (not C1)

WP04 signed host, WP06–07 Chora frames, WP11–12 EMF/QAC, frozen v4, WDAC `getrandom` 0.3.4, pixels, device-loss, Vibe eval.

### Other Aether gaps (after B1a)

| ID | Gap |
|---|---|
| A4 | `AGENTS.md` line 78 still says “sovereign … framework” |
| Pin | Root Etalon git `575bab54` is **older** than Qualia’s `c0bddc9` / local `c110aaf` |
| G4 | Synth stays on `d5ccce3` desktop until it **opts into** a published B1a rev |
| B1b | Copied-frame widget; untracked `chora_frame.rs` |
| B3 | Native VibeScript eval (not JS) |

---

## Anvil (parallel; gates attested vaults, not C1)

**State.** `main` `b5e0466` plus local N0/N1/N2/N3/N4 (uncommitted). Scraper still writes seeded SKUs into `assets/anvil_ontology.q42`. **NodeProfile64** is 64 bytes (`NP64`) in `anvil-core`. `anvil-scraper probe` writes `assets/anvil_node_profile.np64`. Qualia `qualia-core-db` path-deps `anvil-core` `vault-security`; Sanctuary fail-closed without the file. Etalon path-deps `anvil-gpu` (N3). F4 advertising JSON is `webizen-host-attestation`. `SystemHardwareMap` is unused.

`GpuAdapterCaps::anvil_workgroup_size` calls `anvil_gpu::hints_for_pci` for known SKUs. Live UUID bind is `etalon_agent::anvil_bind` plus Qualia `persist_gpu_uuid`. This box still must not advertise (no IOMMU).

| ID | Work |
|---|---|
| N0 | Define live host profile layout (design name NodeProfile64, or replacement) in Anvil. No Qualia types. |
| N1 | `anvil-scraper probe` (or subcommand) writes that profile from **this** box. Keep seeded ontology. |
| N2 | Qualia `anvil-core` `vault-security`; Sanctuary fail-closed if profile missing when attestation is claimed. |
| N3 | Etalon path/git pin `anvil-gpu`; bind live `deviceUUID`; `anvil_workgroup_size` calls `hints_for_pci`. |
| N4 | Do not advertise “hardware-attested vault” on F4 until N1+N2+N3 pass on that box. |

N0–N1 may start **now**, beside B1a.

---

## Etalon (GPU + local LLM engine)

**State.** Branch `0.0.41.3`. Qualia workspace pins **`a8f12b3`**. Local Etalon is that rev plus uncommitted N3 (`anvil_bind`, HAL `deviceUUID`). Aether pins **`575bab54`** (much older). Dirty: `picking_readback.rs` + extra target dirs.

**Inference crates are real:** `etalon-inference`, `-kernel`, `-contract`, `-lab`, `etalon-model` (GGUF/P64). Backends: CUDA, DirectML, Metal, Accelerate, `gguf-cpu`. Qualia `qualia-inference-kernel` re-exports the kernel.

### Gaps

1. **Anvil N4 wired locally** (`webizen-host-attestation` + Qualia copy gate). This box still reports `hardware_attested_vault: false` (no IOMMU). `shared_gpu` does not auto-bind yet: git pin `a8f12b3` has no `anvil_bind`.
2. **Three Etalon revisions in play:** Aether `575bab54`, Qualia git `a8f12b3`, local `a8f12b3` plus uncommitted N3. Align Aether native and Qualia to one live rev **after** B1a compiles, not as a drive-by in the dirty Aether tree.
3. **Gemma-4 full decoder** historically incomplete; NVFP4 tests exist if `C:\LLM_files\gemma\` is present.
4. **P64** format is implemented; local vault `C:\LLM_files\p64\` is **empty** (GGUF/safetensors only).
5. Sensing-bridge excluded from Qualia workspace — not a desktop-chrome blocker.

---

## hypermedia-production (morphisms, not a host)

**State.** `0.0.41.3` HEAD `f9393ae`. Synth git-pins `b673fd6` and **path-patches** the local tree. Role: 150 `hypermedia.*` morphisms via `asset-production-vibe`. **No LLM host. No Poet shell. No GPU backend.**

Native Poet rebuild is deferred (README). The only LLM-shaped code is a small JSON packet formatter for behavioral macros.

**Work for this programme**

| ID | Work |
|---|---|
| HP1 | Keep Synth `[patch]` onto sibling hypermedia; refresh git pin when a rev is published |
| HP2 | Do **not** add an LLM runtime here |
| HP3 | Poet consumes morphisms through Synth `synth_produce_present` / a future Qap — not by invading this crate |

---

## Synth (produce + MCP + vault list)

**State.** `0.0.41.3` HEAD `46055533` (CC BY-NC-ND restore). Dirty **153**. Aether git `d5ccce3`. Qualia git pin `4b7fc106` path-patched to local (local HEAD **ahead**). Hypermedia pin `b673fd6` path-patched (local **ahead**).

**Local LLM: inventory, not decode.** `tray_env.rs` scans `QUALIA_LLM_VAULT` → `C:\LLM_files` → `C:\LLM_Models`. This machine: `C:\LLM_files` ~47.6 GiB, Gemma + Qwen GGUF/safetensors, **0 `.p64`**. MCP `list_models` looks at Qualia `{storage}/Models/*.gguf`, **not** `C:\LLM_files`. `llm_infer` / `llm_chat` are Qualia tools; they fail closed unless a native host has bound `NativeProductSession`. Synth never calls `bind_mcp_inference_session`.

`:8787` is Synth Studio. Guest must not bind it. Qualia daemon is `:4242`.

| ID | Work |
|---|---|
| S1 | Bind MCP inference to Qualia native session (signed actor + model lease) so `llm_infer` works against the vault |
| S2 | Point Qualia `list_models` at the same vault Synth scans (`QUALIA_LLM_VAULT` / `C:\LLM_files`) |
| S3 | Optional: GGUF→P64 into `C:\LLM_files\p64` via Qualia/Etalon writer |
| S4 | Keep produce on hypermedia morphisms; do not merge PoetSnapshot catalog into ProductionSession |
| G0 | Confirm crate `LICENSE`/`Cargo.toml` match CC BY-NC-ND 4.0 (HEAD message already restores this) |
| G4 | Do not move Synth onto dirty Aether native until B1a is a published rev |

---

## QualiaDB (foundation: daemon, Qaps, poet_host, decode)

**State.** `0.0.41.3` HEAD `3bd44eaee`, dirty 12. Installer `install_package_atomic` + `{storage}/Qapps` is the Guest install path. Daemon `:4242`. `poet_host` is the **real VibeScript host** (`PoetSnapshot::live()`, `ALL_BOUND`). Native GGUF decode exists and is **governance-gated**.

| ID | Work |
|---|---|
| D0 | `LaunchPlan` without `APP_STATE` |
| D4 | Anatomy seed install only |
| D5 | `qualiadb.service` after A7 |
| Q-LLM | Document that MCP `llm_*` need `configure(NativeProductSession)`; Synth S1 is the bind |
| Q-Anvil | N3 cargo dep when N1 exists |

---

## Poet (separate programme after Guest can load apps)

Several extant trees. Phase 1 **does not launch Poet** and does not pick a winner.

| Tree | What it is |
|---|---|
| `C:\github\poet` `49f9c40` | WASM HyperCanvas 0.0.34 + JS Canvas_Workbench + specs. **No** Qualia/Etalon/GGUF/MCP |
| webizen-desktop `commands::poet*` | Tauri harness (stale) |
| Qualia `poet_host` | Vibe host + HyperCanvas **receipts** |
| Website WASM | Qualia playground; LLM adapter slot still a placeholder |
| Synth | MCP re-exports `vibe_invoke` / `poet_artifact_lifecycle`; `hypermedia.*` via produce only |

**To “power Poet” later (Group H, after C3):**

1. Guest can install/load Qaps (C2/C3).
2. One Poet surface loads as a Qap (H2), talking Qualia `poet_host` on `:4242`.
3. Local LLM via Qualia native session (S1), not llama.cpp, not Ollama (Ollama is embeddings-only in `qualia-semantic-library`).
4. Asset morphisms stay on Synth `:8787` / `synth_produce_present`.
5. Do not absorb Poet into the Qualia workspace.

---

## Recommended execution order (now)

1. **Wave 0 hygiene** — A1, A2, A3, A4, A5 pin snapshot (this session).
2. **B1a** — make dirty Aether native pinnable (next session; large; do not mix Chora).
3. **N0/N1** Anvil live profile (parallel with 2).
4. **C1** os-shell → `aether` + `aether-native` path pin; paint halo/orbit.
5. **S1/S2** Synth/Qualia local-LLM vault + inference bind (powers later Poet; not C1).
6. Align Etalon pins (Aether ← Qualia live rev) only when B1a compiles.

Do not start Appliance images, Chora worker packaging, or a Poet repo merge as the main effort until C1 paints.
