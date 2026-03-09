# Specs

Spesifikasi teknis library language-style.

Catatan baseline engine:

- Parser `.fs` menghasilkan style raw (`decls`) dan style canonical (`canonicalDecls`).
- Normalisasi dilakukan dengan `formo-ir::normalize_style_decls(...)` agar backend web/desktop membaca semantic style yang sama.
