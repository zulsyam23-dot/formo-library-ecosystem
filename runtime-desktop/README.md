# Library: runtime-desktop

## Tanggung jawab

- Emit artifact desktop native Rust dari Formo IR.
- Menjaga style parity inti antara web dan desktop native.
- Menyediakan widget parity tambahan (`Image`, `Spacer`, `Checkbox`, `Switch`, `Modal`, `Fragment`, `If`, `For`).
- Menambahkan parity diagnostics (warning style/widget unsupported) ke output native.
- Menyediakan kontrak action/state bridge untuk host desktop Rust.

## Input/Output

- Input: Formo IR.
- Output:
  - `app.native.json`, `app.native.rs`, `app.ir.json`
  - scaffold runnable app: `native-app/Cargo.toml`, `native-app/src/main.rs`, `native-app/README.md`
  - opsi CLI `formo build --target desktop --release-exe` untuk compile langsung binary release native app

## Batas domain

- Tidak mendefinisikan syntax bahasa.
- Tidak menyimpan logika parser/typer.
- Tidak mengandung runtime webview/web asset.

## Mapping implementasi saat ini

- `programs/formo-backend-desktop`

## Artefak migrasi fitur

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/artifact-layout.md`
