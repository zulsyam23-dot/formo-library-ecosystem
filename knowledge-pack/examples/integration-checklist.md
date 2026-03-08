# Integration Checklist Formo

## AI Quick Context
- doc_path: knowledge-pack/examples/integration-checklist.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Checklist ini dipakai saat onboarding project baru atau sebelum release.

## A) Struktur Project

- [ ] `main.fm` tersedia sebagai entry.
- [ ] folder style `.fs` terpisah dan terorganisir.
- [ ] import path antar module valid.

## B) Kualitas Bahasa Formo

- [ ] tidak ada component multi-root.
- [ ] prop built-in valid.
- [ ] type param konsisten dengan pemakaian.
- [ ] penggunaan `Slot` sesuai aturan.

## C) Kualitas Style

- [ ] property style hanya dari allowlist.
- [ ] token tidak duplikat.
- [ ] style id tidak duplikat.
- [ ] tidak ada unused token (`E1304`).

## D) Command Validasi

- [ ] `cargo run -p formo-cli -- fmt --input main.fm --check` lulus.
- [ ] `cargo run -p formo-cli -- check --input main.fm --json` lulus.
- [ ] `cargo run -p formo-cli -- diagnose --input main.fm --json` lulus.
- [ ] `cargo run -p formo-cli -- doctor --input main.fm --json` tidak ada blocker.

## E) Build Target

- [ ] build web berhasil (jika target web dipakai).
- [ ] build desktop berhasil (jika target desktop dipakai).
- [ ] build multi berhasil (jika produk lintas target).

## F) Desktop Native (Jika Dipakai)

- [ ] `app.native.json` terbentuk.
- [ ] `native-app` scaffold terbentuk.
- [ ] `--release-exe` berhasil compile binary release.
- [ ] parity warning desktop ditinjau.

## G) Rilis

- [ ] benchmark budget lulus.
- [ ] artifact final tervalidasi.
- [ ] changelog diperbarui.
- [ ] dokumentasi fitur baru diperbarui.

