# Formo CLI, Build, dan Runtime Guide

## AI Quick Context
- doc_path: knowledge-pack/docs/03-cli-build-runtime.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Panduan command CLI Formo dan kontrak runtime untuk pipeline produksi.

## 1) Ringkasan Command

`formo <command>`

Command utama:

- `check`
- `logic`
- `diagnose`
- `fmt`
- `lsp`
- `doctor`
- `build`
- `bench`

## 2) `check`

Tujuan:

- validasi pipeline cepat.

Contoh:

```bash
cargo run -p formo-cli -- check --input main.fm
```

Opsi:

- `--json`
- `--json-pretty`
- `--json-schema`
- `--watch`

Catatan input:

- `check` fail-fast jika input bukan file `.fm`.

## 3) `logic`

Tujuan:

- validasi layer logika deklaratif `.fl`.

Contoh:

```bash
cargo run -p formo-cli -- logic --input logic/controllers/app_controller.fl --json-pretty
```

Opsi:

- `--json`
- `--json-pretty`
- `--rt-manifest-out <file>`

Catatan input:

- `logic` fail-fast jika input bukan file `.fl`.

Deklarasi standar unit `.fl`:

- `enum`
- `struct`
- `type`
- `function` (parameter wajib bertipe)
- `state` (field wajib bertipe + initializer)

Kontrol standar event `.fl`:

- `if`, `for`, `while`, `match`
- `try`, `catch`
- `action throw` (hanya di `try/catch`)
- `action break` / `action continue` (hanya di loop)
- `action return` (harus action terakhir dalam event)

Output JSON `logic` memuat metrik unit:

- `functionCount`
- `typedFunctionCount`
- `returningFunctionCount`
- `stateFieldCount`
- `typedStateFieldCount`
- `enumCount`
- `enumVariantCount`
- `structCount`
- `typedStructCount`
- `structFieldCount`
- `typeAliasCount`
- `qualifiedTypeAliasCount`

Output JSON `logic` memuat metrik event:

- `ifCount`
- `forCount`
- `whileCount`
- `matchCount`
- `tryCount`
- `catchCount`
- `setCount`
- `emitCount`
- `throwCount`
- `breakCount`
- `continueCount`
- `returnCount`

Validasi strict profile `logic`:

- event wajib `lowerCamelCase`,
- function wajib `lowerCamelCase`,
- enum/struct/type mengikuti aturan penamaan strict,
- logic wajib punya global-core action per event,
- service wajib platform-agnostic,
- logic/service dilarang direct call alias runtime `Browser`/`Desktop`,
- adapter hanya boleh `action call`,
- aksi platform web/desktop wajib simetris per-event,
- urutan blok platform harus `desktop` dulu lalu `web` (desktop-first baseline),
- untuk unit `logic`, aksi di dalam blok platform hanya boleh `action call`,
- untuk unit `logic/adapter`, aksi global wajib sebelum blok platform dan blok platform tidak boleh interleaving,
- field `state` wajib lowerCamelCase + typed + initializer literal sesuai tipe dasar (`bool/string/int/float`),
- `action set` wajib menarget field yang sudah dideklarasikan di blok `state` dan wajib ditutup `;`.
- `action set` menolak mismatch literal dasar terhadap tipe field state (`bool/string/int/float`).
- expression RHS `action set` wajib hanya mereferensikan state field yang terdaftar, dengan tipe operand kompatibel terhadap target field.
- inferensi expression dasar (`+ - * / %`, `== != < <= > >=`, `&& ||`) digunakan untuk validasi tipe `action set`.

## 4) `diagnose`

Tujuan:

- validasi plus statistik dan diagnostik lebih lengkap.

Contoh:

```bash
cargo run -p formo-cli -- diagnose --input main.fm --json-pretty
```

Opsi:

- `--json`
- `--json-pretty`
- `--json-schema`
- `--lsp`
- `--watch`

## 5) `lsp`

Tujuan:

- emit payload diagnostic format LSP untuk editor/tooling.

Contoh:

```bash
cargo run -p formo-cli -- lsp --input main.fm
cargo run -p formo-cli -- lsp --input main.fm --watch
```

Opsi:

- `--watch`

## 6) `fmt`

Tujuan:

- format source Formo ke layout canonical.

Contoh:

```bash
cargo run -p formo-cli -- fmt --input main.fm
cargo run -p formo-cli -- fmt --input main.fm --check
cargo run -p formo-cli -- fmt --input main.fm --stdout
```

## 7) `doctor`

Tujuan:

- health check environment + pipeline.

Contoh:

```bash
cargo run -p formo-cli -- doctor --input main.fm
cargo run -p formo-cli -- doctor --input main.fm --json
cargo run -p formo-cli -- doctor --input main.fm --json-pretty
cargo run -p formo-cli -- doctor --input main.fm --json-schema
```

Opsi:

- `--json`
- `--json-pretty`
- `--json-schema`

## 8) `build`

Tujuan:

- generate artifact target runtime.

Contoh:

```bash
cargo run -p formo-cli -- build --target web --input main.fm --out dist-web
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop
cargo run -p formo-cli -- build --target multi --input main.fm --out dist
```

Opsi:

- `--target web|desktop|multi`
- `--input <file>`
- `--out <dir>`
- `--watch`
- `--prod`
- `--release-exe` (khusus `desktop`/`multi`)
- `--strict` (preset gabungan: `--strict-parity` + `--strict-engine`)
- `--strict-parity` (berlaku untuk `web|desktop|multi`, build gagal jika ada parity warning desktop)
  - pada target `web`, audit parity desktop membutuhkan feature `backend-desktop`
- `--strict-engine` (berlaku untuk `web|desktop|multi`, build gagal jika audit bridge `FM/FS/FL` masih ada warning)

## 9) `bench`

Tujuan:

- benchmark compile dan first-render simulation.

Contoh:

```bash
cargo run -p formo-cli -- bench --input main.fm --iterations 20 --warmup 3 --nodes 1000 --out dist-ci/bench/benchmark.json --json-pretty
```

Opsi penting:

- `--iterations <N>`
- `--warmup <N>`
- `--nodes <N>`
- `--out <file>`
- `--json-pretty`
- `--max-compile-p95-ms <N>`
- `--max-first-render-p95-ms <N>`

## 10) Artifact Layout

### Web

- `index.html`
- `app.js`
- `app.css`
- `runtime/README.md`
- `runtime/app/*.js` (source runtime terpecah untuk readability)
- `desktop.parity.json` (opsional; muncul jika `--strict-parity` pada target web menemukan warning parity desktop)
- `engine.bridge.json` (selalu ada; manifest audit standar `FM/FS/FL`)

### Desktop Native

- `app.native.json`
- `app.native.rs`
- `app.ir.json`
- `native-app/Cargo.toml`
- `native-app/src/*`
- `readable/README.md`
- `readable/native/*.json`
- `readable/ir/*.json`

### Multi

- `web/*`
- `desktop/*`

## 11) Workflow CI Direkomendasikan

1. `cargo run -p formo-cli -- fmt --input main.fm --check`
2. `cargo run -p formo-cli -- check --input main.fm --json`
3. `cargo run -p formo-cli -- diagnose --input main.fm --json`
4. `cargo run -p formo-cli -- logic --input logic/controllers/app_controller.fl --json-pretty --rt-manifest-out dist-ci/runtime/logic.manifest.json`
5. `cargo run -p formo-cli -- build --target web --input main.fm --out dist-web --prod --strict-parity`
6. `cargo run -p formo-cli -- build --target web --input main.fm --out dist-web --prod --strict-engine`
7. `cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --prod --strict-parity --strict-engine`
8. desktop release opsional: `cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --release-exe`
9. `cargo run -p formo-cli -- bench --input main.fm --iterations 20 --warmup 3 --nodes 1000 --out dist-ci/bench/benchmark.json --max-compile-p95-ms 80 --max-first-render-p95-ms 60`

## 12) Baseline Engine FM/FL/FS

Untuk memastikan output web/desktop setara:

1. `FM` mendefinisikan struktur UI.
2. `FL` mendefinisikan event/logic platform-agnostic.
3. `FS` mendefinisikan style declarative.
4. IR menyimpan style sebagai `decls` + `canonicalDecls`.
5. Backend web dan desktop selalu membaca style via `effective_style_decls(...)`.

