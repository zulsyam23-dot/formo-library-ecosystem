# Docs

Dokumentasi library runtime-desktop.

Ringkasan cepat:

- target desktop menghasilkan artifact native Rust (tanpa webview).
- output sudah termasuk scaffold app GUI (`native-app/`) yang bisa langsung dijalankan.
- renderer desktop native sudah memiliki style parity core untuk properti style umum.
- widget parity tambahan tersedia untuk `Image`, `Spacer`, `Checkbox`, `Switch`, `Modal`, `Fragment`, `If`, dan `For`.
- jika ada gap parity, warning akan ditulis ke `app.native.json` pada field `diagnostics`.
