# Library: runtime-web

`runtime-web` adalah backend emitter Formo untuk target browser.

## Apa yang Ditangani

- emit artifact web (`index.html`, `app.css`, `app.js`)
- emit runtime source terpecah (`runtime/app/*.js`)
- runtime state/action/control flow untuk output web
- sinkronisasi handler FL ke runtime web (`app.js` + `runtime/app/50_actions_state.js`) saat build target `web|multi`
- evaluator expression `action set` kompleks berbasis RPN untuk menjaga parity logika dengan desktop
- render style dari canonical IR (`effective_style_decls`)

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
  - `compiled style map` (`decls` + `canonicalDecls`)
- output:
  - `index.html`
  - `app.css`
  - `app.js`
  - `runtime/README.md`
  - `runtime/app/*.js`

## Catatan Parity

- CSS emitter web membaca style via `formo_ir::effective_style_decls(...)`.
- Jika `canonicalDecls` tersedia di IR, backend web akan selalu mengutamakan canonical semantics agar sinkron dengan desktop.
- Build tooling menyuntik script event FL ke runtime web sebagai `formoGeneratedActions` dengan fallback `window.formoActions`.
- Evaluasi `action set` kompleks di runtime web mengikuti urutan RPN (precedence operator + kurung), sejajar dengan runtime desktop.

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
