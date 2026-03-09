# Specs

Spesifikasi teknis library language-core.

Catatan baseline engine:

- `formo-ir` menyediakan `normalize_style_decls(...)` dan `effective_style_decls(...)`.
- Kontrak IR style menyimpan `decls` dan `canonicalDecls` untuk menjaga kestabilan semantic lintas runtime.
