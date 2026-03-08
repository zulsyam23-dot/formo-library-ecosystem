# Troubleshooting dan Diagnostics Formo

## AI Quick Context
- doc_path: knowledge-pack/docs/05-troubleshooting-diagnostics.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Dokumen ini untuk mempercepat investigasi error compile/build/runtime artifact.

## 1) Strategi Debug Umum

Urutan paling efektif:

1. `cargo run -p formo-cli -- check --input main.fm --json-pretty`
2. lihat `stage`
3. lihat `errorMeta.code` + lokasi
4. perbaiki dari stage paling awal
5. jalankan ulang sampai bersih

## 2) Stage Error

Stage umum:

- `parser` (`E11xx`)
- `resolver` (`E12xx`)
- `style` (`E13xx`)
- `lowering` (`E14xx`)
- `typer` (`E2xxx`)
- `pipeline`
- `preflight` (khusus doctor)

## 3) Masalah Parser

Gejala:

- syntax tidak valid,
- token unexpected,
- import/component declaration rusak.

Perbaikan:

1. cek tanda `;` pada `import`.
2. cek kurung `()`, `{}`, dan tag `<...>`.
3. pastikan deklarasi param benar (`name: type`).

## 4) Masalah Resolver

Gejala umum:

- path import tidak ditemukan,
- cyclic import,
- alias import duplikat.

Perbaikan:

1. cek path relatif import.
2. putus dependency melingkar.
3. gunakan alias unik per module.

## 5) Masalah Style (`E13xx`)

Contoh code:

- `E1300`: file style tidak bisa dibaca
- `E1301`: syntax/allowlist/value style invalid
- `E1302`: duplicate token
- `E1303`: duplicate style id
- `E1304`: unused token

Perbaikan:

1. validasi token key.
2. pastikan style property ada di allowlist.
3. pastikan semua token yang didefinisikan benar-benar dipakai.

## 6) Masalah Typing (`E2xxx`)

Code penting:

- `E2001` component root kosong
- `E2002` component multi-root
- `E2101` nama node harus uppercase
- `E2102` node tidak dikenal
- `E2250` unknown prop built-in
- `E2251` type mismatch prop built-in
- `E2252` children tidak diizinkan pada built-in tertentu
- `E2253` required prop built-in hilang
- `E2301..E2304` masalah pemanggilan custom component
- `E2221` style pada `Slot` dilarang
- `E2222` style kosong
- `E2223` tipe style attr invalid

Perbaikan:

1. bandingkan node + prop dengan referensi built-in.
2. cek tipe param dan value.
3. pastikan komponen child punya `<Slot/>` jika ingin menerima inline children.

## 7) Masalah Lowering

Contoh gejala:

- recursive component expansion,
- penggunaan `<Slot/>` di luar konteks custom component expansion.

Perbaikan:

1. hilangkan siklus komponen (`A -> B -> A`).
2. pakai `Slot` hanya di komponen reusable yang dipanggil oleh parent.

## 8) Masalah Build Desktop

Gejala:

- parity warning muncul.
- native app belum ter-compile.

Perbaikan:

1. cek warning parity di output CLI.
2. buka `app.native.json`, cek `diagnostics`.
3. gunakan property/style yang parity-friendly.
4. untuk compile release otomatis gunakan `--release-exe`.

## 9) Contoh Debug Cepat (Template)

Langkah:

```bash
cargo run -p formo-cli -- check --input main.fm --json-pretty
cargo run -p formo-cli -- diagnose --input main.fm --json-pretty
cargo run -p formo-cli -- doctor --input main.fm --json-pretty
```

Fokus field JSON:

- `stage`
- `error`
- `errorMeta.code`
- `errorMeta.file`
- `errorMeta.line`
- `errorMeta.col`

## 10) Checklist Sebelum Menyalahkan Runtime

1. Source `.fm` valid.
2. Source `.fs` valid.
3. `cargo run -p formo-cli -- check --input main.fm --json` dan `cargo run -p formo-cli -- diagnose --input main.fm --json` bersih.
4. Build berhasil.
5. Jika desktop mismatch, bandingkan parity warning dulu.

## 11) Workflow Incident Response (Tim)

1. Reproduce di branch kecil.
2. Simpan output `cargo run -p formo-cli -- check --input main.fm --json-pretty`.
3. Klasifikasikan stage.
4. Fix minimal.
5. Re-run pipeline.
6. Tulis postmortem singkat dengan code error dan root cause.

