# Specs

Spesifikasi teknis library runtime-desktop.

Catatan baseline engine:

- Backend desktop membaca style dari IR canonical melalui `effective_style_decls(...)`.
- Phase sizing terbaru menambahkan dukungan `flex`, `flex-grow`, `flex-shrink`, dan `flex-basis` untuk parity web-desktop.
