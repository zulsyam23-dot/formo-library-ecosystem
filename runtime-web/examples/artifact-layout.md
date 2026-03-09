# Runtime Web Artifact Layout

Contoh artifact hasil build target web:

1. `index.html`
2. `app.css`
3. `app.js`
4. `runtime/README.md`
5. `runtime/app/*.js`

Kontrak minimal:

- `index.html` memuat root container app.
- `app.css` memuat style canonical hasil `effective_style_decls(...)` dari IR.
- `app.js` memuat runtime DOM + state/action/control-flow.
- `runtime/app/*.js` memuat source runtime yang sama dalam bentuk terpecah (readable).
- `desktop.parity.json` dapat muncul saat build web dijalankan dengan `--strict-parity` dan audit desktop menemukan warning parity.
