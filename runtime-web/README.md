# Library: runtime-web

`runtime-web` adalah backend emitter Formo untuk target browser.

## Apa yang Ditangani

- emit artifact web (`index.html`, `app.css`, `app.js`)
- emit runtime source terpecah (`runtime/app/*.js`)
- runtime state/action/control flow untuk output web

## Apa yang Tidak Ditangani

- grammar dan type system bahasa Formo
- style parser `.fs`
- host bridge desktop native

## Status dan Capability

- status kontrak: `active`
- capability utama:
  - `web_artifact_emit`
  - `state_action_runtime`
  - `if_for_runtime`
  - `modal_accessibility`
  - `keyed_for_patch`

Sumber: `contracts/CAPABILITIES.json`.

## Input dan Output

- input:
  - `public ir`
  - `compiled style map`
- output:
  - `index.html`
  - `app.css`
  - `app.js`
  - `runtime/README.md`
  - `runtime/app/*.js`

## Mapping Implementasi

- `programs/formo-backend-web`

## Validasi Cepat

```bash
cargo test -p formo-backend-web
```

## Artefak Dokumentasi

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/artifact-layout.md`
