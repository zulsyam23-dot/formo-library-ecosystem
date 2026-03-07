# Features: runtime-desktop

Daftar fitur runtime desktop Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `desktop_bundle_emit`
   - Emit bundle desktop webview (`index.html`, `app.css`, `app.js`).
2. `desktop_bridge_actions`
   - Bridge action host: `window.formoDesktopHost.invokeAction(...)`.
3. `desktop_state_bridge`
   - Helper state bridge: `setStatePatch(...)` dan `replaceState(...)`.
4. `ir_snapshot_emit`
   - Emit `app.ir.json` untuk debugging/integrasi host.

## Mapping Implementasi

- `programs/formo-backend-desktop`
