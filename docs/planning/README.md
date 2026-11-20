# Webizen OS-shell / dual-SKU planning

Planning for the successor to the two stale desktop trees under `C:\github\webizenai`, the Guest vs Appliance SKUs, installable **Qaps**, and how Synth, Poet, and Chora sit on that runtime.

These files are the durable copy of the 2026-11-20 design loop. They are **planning only**. No product source is modified by their presence. Do not treat a temp-directory design draft as canonical.

Vocabulary: human-rights, fiduciary, guardianship, human-agency. Do not use “sovereign” / “self-sovereign” in new copy.

## Documents

| File | What it is |
|---|---|
| [00-dual-sku-design.md](00-dual-sku-design.md) | Full architecture (revision 3f): Key Decisions, dual SKU, three licence rings, Qap install model, PR plan A–H |
| [01-do-first-blockers.md](01-do-first-blockers.md) | What actually blocks Guest chrome, what does not, and the work to do **before** C1 |
| [02-dependency-stack.md](02-dependency-stack.md) | Aether, Anvil, Etalon, hypermedia-production, Synth, local LLM, Poet — work list and order |
| [03-pin-snapshot.md](03-pin-snapshot.md) | 2026-11-20 HEADs, Cargo pin mismatches, LLM vault, A5 `[patch]` recipe |
| [pins/aether-b1a-2026-11-20.md](pins/aether-b1a-2026-11-20.md) | B1a slice pin: native window; Aether is the toolkit, not the app |
| [04-chora-name-and-servo-engine.md](04-chora-name-and-servo-engine.md) | Chora = spatial commons (Qualia). MPL tree = Servo-derived old-web engine. Do not collapse the names. |
| [05-licence-ring-fence.md](05-licence-ring-fence.md) | Anti-slavery licence is Ring B only. MPL engine stays MPL. Sibling process; drop the engine if even that mix is forbidden. |
| [../../Packaging WebizenOS Tri-Target Linux Distribution.md](../../Packaging WebizenOS Tri-Target Linux Distribution.md) | Earlier packaging brief (Nix/bootc/Debian). Dual-SKU design **rejects** a Debian fork; Appliance is an overlay on unmodified Trixie |

A git-tracked copy of the same files lives in `webizen-os-shell/docs/planning/` (that child is a git repo; this parent folder is not).

## Related trees

| Path | Role |
|---|---|
| `C:\github\webizenai\webizen-os-shell` | Successor chrome |
| `C:\github\webizenai\webizen-desktop` | Tauri reference only |
| `C:\github\Aether` | Toolkit; B1a native slice is the C1 pin |
| `C:\github\qualiaDB` | Installer, daemon `:4242`, Qap registry |
| `C:\github\webizen-host` | VPS overlay personality |
| `C:\github\anvil` | Hardware facts; gates **attested-vault advertising**, not Guest paint |
| `C:\github\chora` | Servo-derived MPL old-web engine (repo name is a defect); sibling worker, not a Qap |
| `C:\github\synth` | Installed service `:8787+` |
| `C:\github\poet` | Separate programme after Guest can load apps |

## First visible product

Windows Guest: HaloBar/OrbitWheel paint (C1), `qualia-cli` sidecar on `127.0.0.1:4242` (C0), Anatomy installed and opened as a native-RSX card (C2/C3). No Tauri, no tray, no Poet, no WASM, no Chora pixels.

C1 depends on **Aether B1a** (native window lifecycle without the old-web engine). It does **not** depend on Anvil, the full Aether WP00–WP17 queue, or a Debian image.
