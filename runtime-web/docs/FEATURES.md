# Features: runtime-web

Daftar fitur runtime web Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `web_artifact_emit`
   - Emit `index.html`, `app.css`, `app.js`.
2. `state_action_runtime`
   - Runtime state/action melalui `window.formoState` dan `window.formoActions`.
3. `if_for_runtime`
   - Eksekusi control-flow runtime untuk `If` dan `For`.
4. `modal_accessibility`
   - Modal memiliki `role="dialog"`, `aria-modal`, close `Esc`, trap fokus `Tab`.
5. `keyed_for_patch`
   - Patch list minimal untuk skenario `For each=<stateKey>`.

## Mapping Implementasi

- `programs/formo-backend-web`
