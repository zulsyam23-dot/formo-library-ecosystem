# Formo CLI, Build, dan Runtime Guide

Panduan rinci semua command utama CLI Formo dan alur artifact runtime.

## 1) Ringkasan Command

`formo <command>`

Command utama:

- `check`
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

## 3) `diagnose`

Tujuan:

- validasi plus statistik dan data diagnostik lebih lengkap.

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

## 4) `fmt`

Tujuan:

- format source Formo ke layout canonical.

Contoh:

```bash
cargo run -p formo-cli -- fmt --input main.fm
cargo run -p formo-cli -- fmt --input main.fm --check
cargo run -p formo-cli -- fmt --input main.fm --stdout
```

## 5) `doctor`

Tujuan:

- health check environment + pipeline.

Contoh:

```bash
cargo run -p formo-cli -- doctor --input main.fm
cargo run -p formo-cli -- doctor --input main.fm --json-schema
```

## 6) `build`

Tujuan:

- generate artifact target runtime.

Contoh umum:

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

Catatan `--release-exe`:

- otomatis menjalankan `cargo build --release` di generated `native-app`.
- output executable ada di `native-app/target/release`.

## 7) `bench`

Tujuan:

- benchmark compile dan first-render simulation.

Contoh:

```bash
cargo run -p formo-cli -- bench --input main.fm --iterations 20 --warmup 3 --nodes 1000 --out dist-ci/bench/benchmark.json --json-pretty
```

Budget:

```bash
cargo run -p formo-cli -- bench --input main.fm --max-compile-p95-ms 50 --max-first-render-p95-ms 30
```

## 8) Error Stage Mapping

Stage yang digunakan output JSON:

- `parser` (umumnya kode `E11xx`)
- `resolver` (`E12xx`)
- `style` (`E13xx`)
- `lowering` (`E14xx`)
- `typer` (`E2xxx`)
- `pipeline` (fallback)
- `preflight` (doctor untuk input file missing)

## 9) Artifact Layout

### Web

- `index.html`
- `app.js`
- `app.css`

### Desktop Native

- `app.native.json`
- `app.native.rs`
- `app.ir.json`
- `native-app/Cargo.toml`
- `native-app/src/*`

### Multi

- `out/web/*`
- `out/desktop/*`

## 10) Workflow CI yang Direkomendasikan

Contoh urutan:

1. `fmt --check`
2. `check --json`
3. `diagnose --json`
4. `build --target multi --prod`
5. desktop release opsional: `build --target desktop --release-exe`
6. `bench` dengan threshold budget

## 11) Workflow Debug yang Direkomendasikan

Saat ada error:

1. jalankan `check --json-pretty`.
2. lihat `stage`, `errorMeta.code`, `file`, `line`, `col`.
3. jika desktop visual tidak matching, cek `app.native.json.diagnostics`.

## 12) Tips Operasional

1. Gunakan `--json` untuk integrasi AI/editor.
2. Gunakan `--watch` saat mode iterasi.
3. Gunakan `--prod` untuk web artifact yang lebih kecil.
4. Gunakan `--release-exe` untuk distribusi desktop.
