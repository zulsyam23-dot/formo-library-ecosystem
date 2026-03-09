# Specs

Spesifikasi teknis library tooling.

Catatan baseline engine:

- Input command `check/diagnose/build/fmt/lsp/doctor/bench` wajib berekstensi `.fm` (fail-fast sebelum pipeline).
- Input command `logic` wajib berekstensi `.fl` (fail-fast sebelum parser logic).
- CLI `build --strict` mengaktifkan strict profile gabungan (`--strict-parity` + `--strict-engine`).
- CLI `build --strict-parity` berlaku untuk `web`, `desktop`, dan `multi`.
- Untuk target `web`, strict parity menjalankan audit parity desktop saat feature `backend-desktop` aktif.
- CLI menulis `engine.bridge.json` pada output build untuk audit standar `FM/FS/FL`.
- `build --strict-engine` memaksa warning bridge `W770x` menjadi fail-fast.
- Warning `W7705` menandakan binding action di FM tidak punya event FL yang sesuai.
