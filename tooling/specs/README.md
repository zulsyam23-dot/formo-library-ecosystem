# Specs

Spesifikasi teknis library tooling.

Catatan baseline engine:

- CLI `build --strict-parity` berlaku untuk `web`, `desktop`, dan `multi`.
- Untuk target `web`, strict parity menjalankan audit parity desktop saat feature `backend-desktop` aktif.
- CLI menulis `engine.bridge.json` pada output build untuk audit standar `FM/FS/FL`.
- `build --strict-engine` memaksa warning bridge `W770x` menjadi fail-fast.
