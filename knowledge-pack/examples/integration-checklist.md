# Integration Checklist Formo

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

- [ ] `fmt --check` lulus.
- [ ] `check --json` lulus.
- [ ] `diagnose --json` lulus.
- [ ] `doctor --json` tidak ada blocker.

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
