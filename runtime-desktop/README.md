# Library: runtime-desktop

`runtime-desktop` adalah backend emitter Formo untuk target native desktop Rust.

## Apa yang Ditangani

- emit bundle native: `app.native.json`, `app.native.rs`, `app.ir.json`
- generate scaffold runnable app: `native-app/*`
- parity diagnostics untuk gap style/widget desktop
- readable artifacts untuk inspeksi AI/manusia
- bridge action/state untuk host desktop
- runtime style resolver canonical (`effective_style_decls`) agar baseline web/desktop konsisten
- flex sizing parity phase-2 (`flex`, `flex-grow`, `flex-shrink`, `flex-basis`)

## Apa yang Tidak Ditangani

- parser/typer bahasa Formo
- style parser `.fs`
- runtime browser/web asset

## Status dan Capability

- status kontrak: `active`
- capability utama:
  - `desktop_native_bundle_emit`
  - `desktop_native_scaffold_emit`
  - `desktop_style_parity_core`
  - `desktop_widget_parity_extended`
  - `desktop_host_actions`
  - `desktop_state_bridge`
  - `ir_snapshot_emit`
  - `desktop_parity_diagnostics`
  - `desktop_readable_artifacts`

Sumber: `contracts/CAPABILITIES.json`.

## Input dan Output

- input:
  - `public ir`
  - `compiled style map` (`decls` + `canonicalDecls`)
- output:
  - `app.native.json`
  - `app.native.rs`
  - `app.ir.json`
  - `native-app/Cargo.toml`
  - `native-app/src/*`
  - `native-app/README.md`
  - `readable/README.md`
  - `readable/native/*.json`
  - `readable/ir/*.json`

## Catatan Parity Engine

- Backend desktop membaca style lewat `formo_ir::effective_style_decls(...)`.
- `justify-content: space-*` hanya mendistribusikan ruang jika ada ukuran main-axis eksplisit.
- `%` pada `width/min/max-width` didukung langsung; `%` pada `height` diproses konservatif untuk menjaga stabilitas layout.

## Mapping Implementasi

- `programs/formo-backend-desktop`

## Validasi Cepat

```bash
cargo test -p formo-backend-desktop
```

## Build Native Release via CLI

```bash
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --release-exe
```

## Artefak Dokumentasi

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/artifact-layout.md`
