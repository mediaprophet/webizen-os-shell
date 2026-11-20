# Anti-slavery licence and MPL: ring-fence, do not mix

**Date:** 2026-11-20  
**Status:** Lock. Planning + engine boundary.  
**Repos:** `C:\github\chora` (Ring C), Ring B trees, `https://github.com/mediaprophet/license`.

Timothy’s own licensing model (digital slavery, modern slavery, related exploitation) applies to **Ring B only**. It cannot absorb Mozilla Public License 2.0 Covered Software. If a future Ring B instrument cannot sit beside an MPL sibling program, **this engine is dropped**. Files are not mixed to force a single licence onto Servo.

## Direct answer

| Question | Answer |
|---|---|
| Can the anti-slavery licence relicense Servo / webizen-WWW-engine files? | **No.** MPL-2.0 is file-level copyleft. Those files stay MPL forever. Servo contributors keep those rights. |
| Can Ring B *use* the engine? | **Yes**, as a **sibling process**: a separate binary, IPC / copied frames / C ABI. Two licences, two artefacts, installer names both. |
| Is static linking / one crate with two licences allowed here? | **No** as product rule. MPL would allow a Larger Work (§1.7, §3.3 first sentence) if Covered Software stays MPL. This programme does **not** take that path. |
| If sibling use is still incompatible with the future licence? | **Delete the engine.** Write or select an old-web engine whose licence can sit beside Ring B. Do not launder Servo through Qualia. |

This is the same family of fence as Debian (Ring A): Device OS stays upstream; overlay stays Ring B; neither absorbs the other.

## Why MPL cannot be mixed into the anti-slavery licence

MPL-2.0 (`C:\github\chora\LICENSE`):

- **§1.4 Covered Software** — each source file in the engine tree, plus executables produced from it.
- **§1.7 Larger Work** — combining Covered Software with other material **in separate files**.
- **§3.1** — modifications of Covered Software stay under MPL.
- **§3.3 Secondary License** — only GPL-2.0, LGPL-2.1, AGPL-3.0. A custom anti-slavery licence is **not** a Secondary License. Exhibit B is unused here; that does not grant a right to relicense.

Nobody can put an anti-slavery header on a Servo file and call it Ring B.

`https://github.com/mediaprophet/license` is still a draft fabric (CC interim on related projects; adoption requires an authorised release). Its own rights register already expects **incorporated third-party material to keep its own licence**. Relicensing Servo is outside that fabric’s authority.

## Three rings (neither absorbs another)

| Ring | Work | Licence |
|---|---|---|
| **A** | Device OS (Debian Trixie, Windows, macOS) | Unmodified upstream |
| **B** | QualiaDB, Aether, Webizen, **Chora canvas**, Synth, Poet, Anvil, Etalon, Web Browser chrome | Temporary CC BY-NC-ND 4.0; later `mediaprophet/license` (anti-slavery / humanitarian equity) |
| **C** | **webizen-WWW-engine** (Servo-derived; local `C:\github\chora`) | **MPL-2.0 forever** for Covered Software |

**Chora** is Ring B (Qualia spatial commons). The MPL tree is the old-web engine. Do not put the word Chora on Ring C.

## Allowed composition

- Separate git repository (`C:\github\chora`).
- Separate artefact: `{install}/www-engine-worker` (today’s path `{install}/chora-worker` until renamed). Built **from the engine tree**. Licensed MPL-2.0.
- Ring B talks over IPC / copied frames / `ffi/capi`. Generic traits only (`WebViewDelegate`, `ProtocolHandler`, `SocialWebBackend`, cert/keystore).
- Installer notices name **two** licences: Ring B for the User-Agent, MPL-2.0 for the engine.
- Solid-native users never need Qualia. Qualia may plug in as a **host** backend; that plug stays Ring B.
- Chrome multiplexes **stacks like schemes**. QualiaDB / Etalon is VibeScript + Etalon (no V8, no SpiderMonkey). JavaScript stays in the old-web worker (SpiderMonkey in this engine). A mixed site is routing, not a merged VM and not a merged licence.

## Forbidden composition

- Copying an engine file into Qualia, Aether, Webizen, Synth, Poet, Etalon, or Anvil.
- Copying Qualia/Webizen implementation into the engine.
- One source file carrying both MPL Covered Software and Ring B code.
- A Ring B crate (`license = "CC-BY-NC-ND-4.0"` or future anti-slavery SPDX) **path- or git-depending on** the `servo` crate.
- Statically linking the engine into `webizen-os-shell`, `qualia-cli`, or Aether native as a way around the worker.
- Claiming the HTML/CSS/JS engine is under the anti-slavery licence.

## Known leak (close on E0)

`C:\github\Aether\packages\chora-worker`:

- Declares `license = "CC-BY-NC-ND-4.0"`.
- Path-depends on `../../../chora/components/servo` (`embedded_library`).
- `src/main.rs` `use chora::{ChoraBuilder, …}`.

That is a Larger Work mix in one crate. **E0** builds the worker binary from `C:\github\chora` (MPL) and leaves Aether with:

- `aether-chora-client` — already clean (protocol + process spawn; **no** `servo` dep).
- `aether-chora-bridge-protocol` — wire types only; already clean.

Do not add Qualia types to the worker while it still links Covered Software. Prefer moving the worker crate into the engine tree over keeping a CC wrapper around `servo`.

## Drop rule

If a reviewed Ring B instrument (when adopted) forbids combination with MPL even as a sibling binary, stop using this engine. That decision is licence compatibility, not a patch. Until then, keep the fence: **Ring B licence on Ring B files; MPL on engine files; process boundary between them.**

## Pointers

- Engine: `C:\github\chora\LICENSE-BOUNDARY.md`, `docs/WWW-STANDARDS.md`
- Names: [04-chora-name-and-servo-engine.md](04-chora-name-and-servo-engine.md)
- Dual SKU KD-6 / KD-19: [00-dual-sku-design.md](00-dual-sku-design.md)
- Draft fabric: `https://github.com/mediaprophet/license` (`PROJECT-PURPOSE.md`, `LICENSING-FRAMEWORK.md`)
