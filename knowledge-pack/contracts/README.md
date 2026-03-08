# Contracts

## AI Quick Context
- doc_path: knowledge-pack/contracts/README.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Kontrak input/output library knowledge-pack.

## Tujuan

Kontrak ini memastikan knowledge-pack:

1. konsisten dengan implementasi Formo terbaru,
2. mudah dikonsumsi AI agent dan tooling,
3. memiliki cakupan dokumentasi yang terukur.

## Sumber Kebenaran

- `CAPABILITIES.json`
- gunakan penamaan capability dalam bentuk singular snake_case agar mapping tooling stabil.

## Aturan Pemeliharaan

Jika menambah dokumen besar baru, wajib:

1. update `docs/FEATURES.md`,
2. update `contracts/CAPABILITIES.json`,
3. update `docs/README.md` index,
4. tambahkan contoh terkait di `examples/` jika relevan.

