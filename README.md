# Formo Library Ecosystem

Monorepo ini adalah fondasi teknis Formo, bukan aplikasi end-user.
Isinya memecah seluruh stack Formo menjadi library modular agar:

- batas domain jelas,
- evolusi fitur lintas tim lebih aman,
- dokumentasi + kontrak mudah dikonsumsi AI agent dan tooling.

## Apa Ini Sebenarnya?

`formo-library-ecosystem` adalah workspace Rust mandiri yang menampung:

1. core bahasa (`.fm` dan `.fl`),
2. compiler style (`.fs`),
3. backend runtime (web + desktop),
4. CLI orchestration,
5. kontrak AI interop,
6. knowledge pack untuk prompt/tooling.

## Arsitektur Library

Repo ini memiliki 7 library level-domain:

1. `language-core` - lexer, parser, resolver, typer, IR, dan logic contract (`.fl`).
2. `language-style` - parser/validator style + token system (`.fs`).
3. `runtime-web` - emitter artifact web + runtime JS.
4. `runtime-desktop` - emitter artifact native Rust + scaffold app desktop.
5. `tooling` - command-line orchestration (`check`, `diagnose`, `build`, `logic`, `bench`, dll).
6. `ai-interop` - kontrak pertukaran capability/error profile untuk AI.
7. `knowledge-pack` - dokumentasi operasional dan prompt-ready references.

## Workspace Crates (Aktif di Cargo.toml)

Total crate Rust yang tergabung saat ini: **10**

- `formo-lexer`
- `formo-parser`
- `formo-logic`
- `formo-resolver`
- `formo-typer`
- `formo-ir`
- `formo-style`
- `formo-backend-web`
- `formo-backend-desktop`
- `formo-cli`

## Status Saat Ini

Berdasarkan `MIGRATION_STATUS.md` (update: **2026-03-07**):

- Total library: **7**
- Status `active`: **5** (`language-core`, `language-style`, `runtime-web`, `runtime-desktop`, `tooling`)
- Status `bootstrap`: **2** (`ai-interop`, `knowledge-pack`)

## Quick Start (Workspace Ini)

Prasyarat: Rust + Cargo.

```bash
# validasi cepat
cargo run -p formo-cli -- check --input main.fm

# diagnostik terstruktur
cargo run -p formo-cli -- diagnose --input main.fm --json-pretty

# build multi target
cargo run -p formo-cli -- build --target multi --input main.fm --out dist
```

Opsional verifikasi workspace:

```bash
cargo test --workspace
```

## Sumber Kebenaran

Gunakan file berikut sebagai referensi utama (manusia + AI):

1. `registry.formo-ai.json` - peta library, dependency, dan capability contract.
2. `mandatory-features.formo-ai.json` - fitur wajib lintas library.
3. `MIGRATION_STATUS.md` - status migrasi aktual per library.
4. `Cargo.toml` - daftar crate/workspace yang benar-benar dibuild.
5. `AI_NATIVE_RULES.md` - aturan penulisan AI-native.

## Konvensi Struktur per Library

Setiap library memakai pola yang sama:

- `docs/` - referensi teknis dan workflow.
- `contracts/` - capability contract machine-readable.
- `examples/` - contoh usage.
- `specs/` - spesifikasi ruang lingkup.
- `programs/` - source crate Rust (jika library punya implementasi runtime/compiler).
