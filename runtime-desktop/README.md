# Library: runtime-desktop

## Tanggung jawab

- Emit bundle webview desktop dan bridge host.
- Menjembatani action/state dari app ke host desktop.

## Input/Output

- Input: Formo IR.
- Output: `index.html`, `app.css`, `app.js`, `desktop-bridge.js`, `app.ir.json`.

## Batas domain

- Tidak mendefinisikan syntax bahasa.
- Tidak menyimpan logika parser/typer.

## Mapping implementasi saat ini

- `programs/formo-backend-desktop`

## Artefak migrasi fitur

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/artifact-layout.md`
