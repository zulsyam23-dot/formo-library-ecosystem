# Formo Feature Migration Status

Dokumen ini memetakan fitur Formo yang wajib tersedia ke library ekosistem.

Update terakhir: 2026-03-07

## Ringkasan

- Total library: 7
- Library `active`: 5
- Library `bootstrap`: 2
- Semua library sudah memiliki:
  - direktori `docs/`, `contracts/`, `examples/`, `specs/`
  - `docs/FEATURES.md`
  - `contracts/CAPABILITIES.json`

## Status Per Library

1. `language-core` (`active`)
   - Lexer + parser + resolver + typer + kontrak IR sudah dimigrasikan sebagai capability.
2. `language-style` (`active`)
   - Parser style, token system, allowlist, dan rules validasi sudah dimigrasikan.
3. `runtime-web` (`active`)
   - Emitter web artifact + runtime state/action/control-flow dimigrasikan.
4. `runtime-desktop` (`active`)
   - Bundle native Rust + scaffold GUI native + style parity core + bridge host desktop + IR snapshot dimigrasikan.
5. `tooling` (`active`)
   - Command CLI utama, watch mode, benchmark, dan diagnostic output dimigrasikan.
6. `ai-interop` (`bootstrap`)
   - Kontrak AI profile + error envelope + capability exchange dimigrasikan sebagai baseline.
7. `knowledge-pack` (`bootstrap`)
   - Glosarium domain + pattern + checklist prompt dimigrasikan sebagai baseline.

## Catatan Migrasi

- Migrasi fitur dan source code crate Rust sudah dilakukan ke folder `programs/` di tiap library.
- Root workspace Formo tetap berfungsi dengan merujuk member crate via path eksternal (`../formo-library-ecosystem/...`).
- Workspace `formo-library-ecosystem` kini mandiri lewat `Cargo.toml` sendiri.
- Dependency antar crate telah dinormalisasi ke `workspace = true`.
