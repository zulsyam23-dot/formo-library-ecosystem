# Library: tooling

`tooling` adalah orchestration layer CLI untuk seluruh pipeline Formo.

## Apa yang Ditangani

- command validasi: `check`, `diagnose`, `doctor`
- command logic: `logic` untuk `.fl`
- command output: `build` (web/desktop/multi)
- command quality: `fmt`, `bench`
- mode `watch` dan payload `lsp`

## Apa yang Tidak Ditangani

- implementasi parser/typer/style compiler inti
- implementasi runtime backend web/desktop

## Status dan Capability

- status kontrak: `active`
- capability utama:
  - `check_diagnose_commands`
  - `build_multi_target`
  - `watch_mode`
  - `benchmark_mode`
  - `lsp_output`
  - `optional_backend_features`

Sumber: `contracts/CAPABILITIES.json`.

## Input dan Output

- input:
  - `entry fm path`
  - `cli command and flags`
- output:
  - `diagnostic output`
  - `build artifacts`
  - `benchmark reports`
  - `lsp diagnostics payload`

## Mapping Implementasi

- `programs/formo-cli`

## Quick Start

```bash
cargo run -p formo-cli -- check --input main.fm
cargo run -p formo-cli -- diagnose --input main.fm --json-pretty
cargo run -p formo-cli -- build --target multi --input main.fm --out dist
```

## Validasi Cepat

```bash
cargo test -p formo-cli
```

## Artefak Dokumentasi

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/cli-commands.md`
