# Docs

Dokumentasi library runtime-desktop.

Ringkasan cepat:

- Target desktop menghasilkan artifact native Rust (tanpa webview).
- Output sudah termasuk scaffold app GUI (`native-app/`) yang bisa langsung dijalankan.
- Renderer desktop memakai `effective_style_decls(...)` sehingga canonical style sama dengan web.
- Dukungan layout mencakup flow/flex (`display`, `flex-direction`, `flex-wrap`, `flex`, `flex-grow`, `flex-shrink`, `flex-basis`).
- Jika ada gap parity, warning ditulis ke `app.native.json` pada field `diagnostics`.
