# Webizen OS-Shell

Successor-chrome planning (dual SKU, Qaps, Aether B1a first): [docs/planning/README.md](docs/planning/README.md).

Guest chrome on the **Aether native** toolkit (Blitz + CPU softbuffer). Aether is the GUI toolkit, not this product. Window title is **Webizen OS-Shell**.

C1 paints HaloBar and OrbitWheel. StageHost is a placeholder. Qualia (`127.0.0.1:4242`) returns at C0/C2. No Tauri, no tray, no WASM, no Chora pixels.

## Features

- **Spatial Halo Bar**: Identity DID presence, P2P peer mesh counter, and Nym mixnet status.
- **Poet Hub & Orbit Wheel**: 8 orbital volume sectors (Talk, Mail, Directory, Wallet, Wellfair, Library, Projects, Settings).
- **Stage Host**: Placeholder modal until C2 installs a Qap as a native-RSX card.

## Running

Native (isolated target; CPU softbuffer):

```powershell
Set-Location C:\github\webizenai\webizen-os-shell
$env:CARGO_TARGET_DIR = "C:\github\webizenai\webizen-os-shell\target-c1"
cargo run --offline
```

Bounded smoke (three redraws, then exit):

```powershell
$env:WEBIZEN_C1_SMOKE = "1"
$env:CARGO_TARGET_DIR = "C:\github\webizenai\webizen-os-shell\target-c1"
cargo run --offline
```

Aether is path-pinned at `../../Aether/packages/{aether,native}` (B1a slice). Synth stays on git Aether `d5ccce3` until G4.
