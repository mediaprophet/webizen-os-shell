# Webizen OS-Shell

> Pure Dioxus-native spatial desktop shell, halo chrome, and QApp volume launcher powered by QualiaDB.

`webizen-os-shell` replaces the legacy Tauri-based shell with a pure Rust Dioxus architecture that compiles to Desktop (`dioxus-desktop`), Web/WASM (`dioxus-web`), and mobile PWA without C++/WebView2 IPC glue.

## Features

- **Spatial Halo Bar**: Identity DID presence, P2P peer mesh counter, and Nym mixnet status.
- **Poet Hub & Orbit Wheel**: 8 orbital volume sectors (Talk, Mail, Directory, Wallet, Wellfair, Library, Projects, Settings).
- **Stage Host Window Manager**: High-performance stage modal with direct in-process bindings to QualiaDB.
- **Zero Heap & 42MB Sentinel**: Connects directly to `qualia-core-db` zero-copy ABI.

## Running

Desktop:
```bash
cargo run --features desktop
```

Web / WASM:
```bash
dx serve --web
```
