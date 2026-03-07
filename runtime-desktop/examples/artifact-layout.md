# Runtime Desktop Artifact Layout

Contoh artifact hasil build target desktop:

1. `index.html`
2. `app.css`
3. `app.js`
4. `desktop-bridge.js`
5. `app.ir.json`

Kontrak minimal:

- bridge host via `window.formoDesktopHost.invokeAction(...)`
- state bridge via `window.formoDesktop.setStatePatch(...)`
