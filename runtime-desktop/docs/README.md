# Docs

Dokumentasi library runtime-desktop.

Ringkasan cepat:

- Target desktop menghasilkan artifact native Rust dengan `dioxus-desktop` (DOM + CSS).
- Output sudah termasuk scaffold app GUI (`native-app/`) yang bisa langsung dijalankan.
- Scaffold juga menghasilkan `native-app/src/actions.rs` sebagai bridge action/state dasar.
- Renderer desktop memakai `effective_style_decls(...)` sehingga canonical style sama dengan web.
- Evaluasi expression `action set` kompleks tersedia via helper RPN (`eval_set_expression_rpn`) untuk parity logika dengan runtime web.
- Dukungan layout mencakup flow/flex (`display`, `flex-direction`, `flex-wrap`, `flex`, `flex-grow`, `flex-shrink`, `flex-basis`).
- Jika ada gap parity, warning ditulis ke `app.native.json` pada field `diagnostics`.
