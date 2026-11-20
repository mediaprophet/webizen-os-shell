# Chora is the spatial commons. The MPL tree is the Servo-derived old-web engine.

**Date:** 2026-11-20  
**Status:** Naming lock from QualiaDB **early branches**. Planning only.  
**Repo:** `C:\github\qualiaDB` (`origin` `mediaprophet/qualiaDB`). Tip inspected: `0.0.41.3`.

Codex (and later copy) put Apache-2.0 on Timothy’s work without permission, and folded the **Chora** product name onto the Servo fork so that name sits under MPL-2.0. That is a licence/name grab. QualiaDB’s own history never made that fold. This note restores the split the early branches already recorded.

## QualiaDB branch timeline (git, not reconstruction)

Checked: `origin/0.0.5` … `origin/0.0.23` have **zero** `chora`-named files. Chora lands on **`origin/0.0.24`**. Servo is a **later** preference on the old-web chrome, not a rename of Chora.

| When | Branch / commit | What the tree actually says |
|---|---|---|
| Jun 2026 | `origin/0.0.5` … `origin/0.0.20` | No Chora module. Playground / WASM “browser” means in-browser demos, not a product named Chora. |
| 1 Jul 2026 | `origin/0.0.23` tip `1ae0bd5be` | Still **0** chora-named files. |
| **8 Jul 2026** | `9893ba770` (first Chora commit; contained by `origin/0.0.24` onward) | *“local snapshot before resuming spatio-temporal interface (STI system / **Chora user-facing**) work.”* Body: **“Chora = front-end user view of the spatio-temporal commons canvas; STI = system-level engine (geospatial + modalities/spatio_temporal + render).”** `crates/qualia-client-core/src/chora/mod.rs`: “Chora — the spatio-temporal commons canvas inside **WellFair**.” Panel: “commons canvas **explorer**.” Menu already has **Chora** `Ctrl+2` `/chora` and **Web Browser** `Ctrl+3` `/browser` as **two** items (`shell/menu.rs` on `origin/0.0.24`). |
| 8 Jul 2026 | `dcd9ecadf`, `3a69999a5` | Chora **layer library** (NASA GIBS, starfield, `.10d` conversion). Still a canvas, not HTML. |
| 12 Jul 2026 | `1bfdec35d` | Docs: “**Chora navigator**” + geospatial/space domain. |
| 9 Jul 2026 | `coordination/NOTICES.md` on `origin/0.0.24` | Claude records Timothy’s direction to **build our own (old-web) browser** because iframe `X-Frame-Options` blocked real https. Phased: P0 native WebView window, P1 own trust store, **P4 optional Servo eval**. Servo is an engine candidate for that chrome. It is not Chora. |
| **17 Jul 2026** | `3b296e8e5` on `origin/0.0.25` | *“Webizen Browser P0 — honest native WebView navigation … **Servo deferred**.”* |
| 17 Jul 2026 | `d5ed07b67` | *“feat(browser): **Chora universe as default home**.”* Start page `qualia://chora/universe`. Home page ≠ engine. |
| 17 Jul 2026 | `4f00bde65` | Servo **preference** + banner. Operator stub on `origin/0.0.25`: **OS WebView = then product path; Servo experimental; does not render pages.** |
| 0.0.28 … 0.0.40 | same two QApps | Audit: **Chora** Ctrl+2 “Spatio-temporal commons manifold”; **Web Browser** Ctrl+3. `HEAD` `chora/mod.rs` still the 8 Jul canvas wording. |
| 19–20 Oct 2026 | *outside QualiaDB* | Servo `main` snapshot forked; GitHub name later `WebCivics/chora`. QualiaDB never renamed Chora to mean that fork. |

**QualiaDB never called Servo “Chora”.** The defect is the later independent repo and Aether adapter branding that put the product name on MPL Covered Software.

## What QualiaDB already recorded (two surfaces)

| Surface | What it is | Evidence |
|---|---|---|
| **Chora** | Spatio-temporal **commons canvas** / universe **navigator** (the human-centric web’s explorer). Maps, ICRS starfield, planted assets, `qualia://chora/universe`. **Not** an HTML engine. **Not** MPL. | First commit `9893ba770` (8 Jul 2026) on `origin/0.0.24`. Module docs unchanged through `HEAD`. QApps **Ctrl+2** `/chora`. |
| **Webizen Web Browser** | Old-web https pages. Separate surface **Ctrl+3** `/browser`. | Same-day menu on `origin/0.0.24`. P0 WebView `3b296e8e5` (17 Jul). `bundled/qapps/Browser/qapp.json` display_name “Webizen Web Browser”. **Servo** = experimental engine for *this* chrome. |

The Web Browser’s default *home* is the Chora universe (`qualia://chora/universe`). That is a start page, not “Chora = the engine”.

Servo was the **experimental engine for the Web Browser**, documented as not yet rendering. The later independent MPL repo (`WebCivics/chora`, local `C:\github\chora`) is that Servo fork, ring-fenced because MPL-2.0 is file-level copyleft. Ring-fencing the **engine** is correct. **Calling that engine Chora** is the defect.

## Two webs (corrected names)

| Web | Product | Engine |
|---|---|---|
| **Human-centric** | QualiaDB, VibeScript, Aether native RSX, Qaps, **Chora** (spatial commons) | Aether toolkit + Qualia. Not a browser engine. |
| **Old web** | **Webizen Web Browser** (Qap / host chrome that loads https) | **webizen-WWW-engine** (Servo-derived, MPL-2.0; GitHub still `WebCivics/chora` until renamed). Sibling worker. Solid read-write web + WebID-TLS in this engine. Qualia plug is optional. |

WebView was a temporary host path while Servo was experimental. It is not the intended old-web engine. It also is not Chora.

## Two stacks, routed like two protocol families

The User-Agent is one window. It multiplexes **stacks** the way a browser multiplexes schemes (`https:`, `mailto:`, `qualia:`). QualiaDB / Etalon is not a JavaScript engine with extra URLs.

| Stack | Speaks | Script | GPU / layout | Process |
|---|---|---|---|---|
| **Human-centric** | `qualia:`, Qaps, Chora canvas, vault / Vibe ABI | **VibeScript** (`crates/vibe`, `vibescript-abi`). No V8. No SpiderMonkey. | Etalon + Aether native RSX | Ring B (shell / Qualia daemon) |
| **Old web** | `http:`, `https:`, Solid LDP / WebID | **JavaScript** in the engine. Servo’s VM is **SpiderMonkey**, not V8. | Servo layout + the engine’s WebGPU path | Ring C sibling worker |

A mixed site is chrome **routing**: Qualia surfaces stay on Vibe/Etalon; an https document goes to the worker. Do not eval JS as Vibe, or Vibe as JS. Do not put V8 into Qualia to “make it a browser.” Old-web V8/Node compatibility, if ever required, is an engine question and stays inside Ring C.

Licence follows the stack, not the window: Ring B files stay Ring B; MPL files stay in the worker. Scheme multiplexing does not merge those grants.

## Licence rings (name-accurate)

| Ring | What | Licence |
|---|---|---|
| A | Device OS (Debian / Windows / macOS) | Upstream |
| B | Qualia, Aether, Webizen, **Chora (spatial commons)**, Synth, Poet, Anvil, Etalon, Web Browser chrome | CC BY-NC-ND 4.0 (temporary) |
| C | **Servo-derived old-web engine** (Covered Software in `C:\github\chora`) | MPL-2.0 |

Do not put the word **Chora** on Ring C in product copy. Do not put MPL on the Chora canvas. Do not copy engine files into Ring B. Do not add Qualia crates inside the engine.

Unauthorized Apache-2.0 stamps on Timothy’s crates (Synth `LICENSE` / `Cargo.toml`) are the same class of defect as this rename. G0 aligns Synth to CC BY-NC-ND 4.0. The engine stays MPL because **Servo** is MPL, not because Chora is.

## Repo / product name

**Product name (locked):** **webizen-WWW-engine**.

GitHub is still `https://github.com/WebCivics/chora` until an explicit rename to
`WebCivics/webizen-www-engine`. Local path stays `C:\github\chora` until that
rename. Cargo package stays `servo`. C ABI `chora_*` / `servo_*` waits for a
dedicated `www_*` pass (E5).

## Licence ring-fence (must hold for the anti-slavery licence)

Timothy’s own licence (digital slavery / modern slavery / related) applies to
**Ring B only**. MPL-2.0 is file-level copyleft. It cannot be absorbed into that
licence. MPL §3.3 “Secondary License” is only GPL/LGPL/AGPL — a custom licence
is not a Secondary License.

Keep the engine as a **sibling worker binary** (copied frames, generic FFI).
Do not copy engine files into Qualia, or Qualia files into the engine. If the
future Ring B licence cannot sit beside an MPL sibling program, **drop this
engine**. Do not mix files to force a single licence onto Servo.

See [05-licence-ring-fence.md](05-licence-ring-fence.md),
`C:\github\chora\LICENSE-BOUNDARY.md`, and `docs/WWW-STANDARDS.md`.

## B1a / B1b (unchanged sequence, corrected words)

- **B1a** — Aether native window for human-centric chrome. Does not link the Servo-derived engine.
- **B1b / E0** — copied-frame **old-web engine** worker for the Webizen Web Browser. Optional Aether feature currently still named `chora` in `aether-native` (code identifier; product name stays Chora = spatial commons).
