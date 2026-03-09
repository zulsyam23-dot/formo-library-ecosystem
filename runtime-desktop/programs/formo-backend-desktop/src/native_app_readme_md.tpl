# Formo Native Desktop App

Scaffold ini di-generate otomatis dari `formo build --target desktop`.

## Entry Component

`{{ENTRY_COMPONENT}}`

## Jalankan

```bash
cd native-app
cargo run
```

Aplikasi ini membaca tree UI dari `../app.native.json` lalu merendernya memakai `dioxus-desktop`.
Pendekatan render memakai struktur DOM + CSS inline agar style parity lebih mudah dipahami dan dirawat.
Scaffold juga menjalankan runtime state/action di Rust (`onClick`/`onChange`, `If`, `For`, `Modal open/close`) agar logika dasar selaras dengan runtime web.
File `src/actions.rs` otomatis meng-generate stub handler dari nama action yang ditemukan di props FM (`onPress`, `onClick`, `onChange`, `onClose`, `action`).
Jika ada style/widget yang belum fully parity dengan web, warning tersedia di `app.native.json` pada field `diagnostics`.

Struktur source scaffold:

- `src/main.rs` entrypoint
- `src/actions.rs` registry handler action runtime
- `src/app.rs` lifecycle app (`dioxus` desktop root)
- `src/model.rs` model bundle/node
- `src/style.rs` mapper style -> CSS
- `src/render/mod.rs` dispatch renderer HTML
- `src/render/flow.rs` renderer layout/flow
- `src/render/controls.rs` renderer controls
- `src/render/media.rs` renderer text/media/modal
- `src/render/shared.rs` helper HTML/style
- `src/render/state.rs` helper scope/value resolver + dispatch action bridge

Style parity core yang didukung:

- visual: `color`, `background`, `border`, `border-radius`, `box-shadow`, `opacity`
- spacing/sizing: `padding`, `margin`, `gap`, `width`, `height`, `min/max-width`, `min/max-height`
- layout/text: `align-items`, `justify-content`, `text-align`, `line-height`, `overflow`, `font-weight`, `font-style`

Widget parity tambahan:

- `Image`, `Spacer`, `Checkbox`, `Switch`, `Modal`
- `Fragment`, `If`, `For` (scope: `alias`, `aliasIndex`, `aliasKey`)
