# Docs

Dokumentasi library runtime-web.

Ringkasan cepat:

- Target web menghasilkan `index.html`, `app.css`, `app.js` plus runtime source terpecah.
- CSS emitter memakai `effective_style_decls(...)` supaya semantic style sama dengan desktop.
- Saat build menemukan logic bridge FL, runtime web disinkronkan dengan handler generated di `app.js` dan `runtime/app/50_actions_state.js`.
- Expression `action set` kompleks di runtime web dievaluasi memakai RPN agar parity dengan desktop tetap konsisten.
- Pada pipeline strict parity lintas target, output web bisa menghasilkan `desktop.parity.json` jika audit parity desktop menemukan gap.
