# Pin snapshot — 2026-11-20

Recorded for A5 / live-sources. Working trees change; this file is a snapshot, not a lockfile.

## Repository HEADs (this machine)

| Repo | Branch | HEAD | Date | Dirty |
|---|---|---|---|---|
| Aether | main | `d5ccce3` | 2026-11-06 | yes (~40 tracked, ~505 untracked) |
| anvil | main | `b5e0466` | 2026-11-10 | no |
| etalon | 0.0.41.3 | `c110aaf` | 2026-11-20 | yes (5) |
| hypermedia-production | 0.0.41.3 | `f9393ae` | 2026-11-20 | yes (5) |
| synth | 0.0.41.3 | `46055533` | 2026-11-20 | yes (153) |
| qualiaDB | 0.0.41.3 | `3bd44eaee` | 2026-11-20 | yes (12) |
| chora | main | `eb08f915733` | 2026-11-20 | yes (101) |
| webizen-host | main | `436cd68` | 2026-11-03 | yes (17) |
| webizen-os-shell | main | `75328cb` | 2026-10-04 | yes (planning docs) |
| webizen-desktop | main | `a323181` | 2026-10-04 | (see design) |
| poet | main | `49f9c40` | 2026-11-06 | no |

## Cargo pins vs siblings

| Consumer | Dependency | Pin | Local sibling | Match? |
|---|---|---|---|---|
| os-shell | aether / aether-native | path `../../Aether/packages/{aether,native}` CPU softbuffer | Aether dirty B1a slice | **C1 path pin** |
| os-shell | qualia-core-db | `path = "../qualiaDB/..."` | `C:\github\qualiaDB` | **Broken path** (Qualia is not inside `webizenai/`) |
| Synth | aether / aether-native | git `d5ccce3` | same HEAD, dirty extra | GitHub commit only; misses dirty native |
| Synth | qualia-* | git `4b7fc106` + path patch | `3bd44eaee` | Path patch = local; git pin stale |
| Synth | asset-production-vibe | git `b673fd6` + path patch | `f9393ae` | Path patch = local; git pin stale |
| QualiaDB | etalon family | git `c0bddc9` | `c110aaf` | **1 commit behind** |
| Aether (dirty root) | etalon | git `575bab54` | `c110aaf` | **Far behind** |
| Aether (dirty root) | etalon-context | hypermedia branch `0.0.40.8.1` | hypermedia `0.0.41.3` | Stale branch pin |
| QualiaDB | anvil | none | `b5e0466` | **Unpinned** |
| Etalon | anvil | none | `b5e0466` | **Unpinned** (duplicated PCI table) |

## Local LLM vault (this machine)

- Env: `QUALIA_LLM_VAULT` else `C:\LLM_files` else `C:\LLM_Models`
- `C:\LLM_files` exists (~47.6 GiB): `gemma/`, `qwen/`, empty `p64/`, empty `experts/`
- Qualia MCP `list_models` uses `{storage}/Models/*.gguf` — **different directory**

## Optional `[patch]` recipe (A5) for os-shell after B1a

Do **not** put this in committed `Cargo.toml` as an env interpolation. After B1a, os-shell should path-dep:

```toml
# from C:\github\webizenai\webizen-os-shell
aether = { path = "../../Aether/packages/aether" }
aether-native = { path = "../../Aether/packages/native" }
```

Optional override if a checkout lives elsewhere:

```toml
# .cargo/config.toml (local, untracked)
[patch."https://github.com/mediaprophet/Aether.git"]
aether = { path = "C:/github/Aether/packages/aether" }
aether-native = { path = "C:/github/Aether/packages/native" }
```

Synth stays on git `d5ccce3` until G4 opts into a published B1a rev.
