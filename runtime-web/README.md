# Library: runtime-web

## Tanggung jawab

- Emit `index.html`, `app.css`, `app.js`.
- Menyediakan runtime DOM untuk state/action/control-flow.

## Input/Output

- Input: Formo IR.
- Output: bundle web siap jalan.

## Batas domain

- Tidak mendefinisikan grammar bahasa Formo.
- Tidak berisi business rule parser/typer.

## Mapping implementasi saat ini

- `programs/formo-backend-web`

## Artefak migrasi fitur

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/artifact-layout.md`
