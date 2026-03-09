# Docs

Dokumentasi library runtime-web.

Ringkasan cepat:

- Target web menghasilkan `index.html`, `app.css`, `app.js` plus runtime source terpecah.
- CSS emitter memakai `effective_style_decls(...)` supaya semantic style sama dengan desktop.
- Pada pipeline strict parity lintas target, output web bisa menghasilkan `desktop.parity.json` jika audit parity desktop menemukan gap.
