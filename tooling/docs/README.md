# Docs

Dokumentasi library tooling.

Ringkasan cepat:

- CLI `formo` mengorkestrasi pipeline parser/resolver/style/typer/runtime.
- Command `check/diagnose/build/fmt/lsp/doctor/bench` sekarang fail-fast jika input bukan file `.fm`.
- Command `logic` sekarang fail-fast jika input bukan file `.fl`.
- `build --strict-parity` berlaku untuk target `web|desktop|multi`.
- `build --strict` adalah preset untuk mengaktifkan `--strict-parity` + `--strict-engine`.
- Untuk target `web`, strict parity menjalankan audit desktop dan bisa menghasilkan `desktop.parity.json`.
