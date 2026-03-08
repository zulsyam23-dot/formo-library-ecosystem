# Runtime Web Artifact Layout

Contoh artifact hasil build target web:

1. `index.html`
2. `app.css`
3. `app.js`
4. `runtime/README.md`
5. `runtime/app/*.js`

Kontrak minimal:

- `index.html` memuat root container app.
- `app.css` memuat style hasil compile `.fs`.
- `app.js` memuat runtime DOM + state/action/control-flow.
- `runtime/app/*.js` memuat source runtime yang sama dalam bentuk terpecah (readable).
