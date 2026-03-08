# Formo Native Desktop App

Scaffold ini di-generate otomatis dari `formo build --target desktop`.

## Entry Component

`{{ENTRY_COMPONENT}}`

## Jalankan

```bash
cd native-app
cargo run
```

Aplikasi ini membaca tree UI dari `../app.native.json` dan menampilkan GUI desktop native Rust (tanpa webview).
Aplikasi scaffold juga sudah memasang baseline theme yang mengikuti tone visual runtime web (warna latar, stroke, spacing).
Jika ada style/widget yang belum fully parity dengan web, warning tersedia di `app.native.json` pada field `diagnostics`.

Struktur source scaffold:

- `src/main.rs` entrypoint
- `src/app.rs` lifecycle app (`eframe::App`)
- `src/model.rs` model bundle/node
- `src/style.rs` mapper style parity
- `src/render/mod.rs` dispatch renderer
- `src/render/flow.rs` renderer layout/flow
- `src/render/controls.rs` renderer controls
- `src/render/media.rs` renderer text/media/modal
- `src/render/shared.rs` helper layout/style
- `src/render/state.rs` helper state/action

Style parity core yang didukung:

- visual: `color`, `background`, `border`, `border-radius`, `box-shadow`, `opacity`
- spacing/sizing: `padding`, `margin`, `gap`, `width`, `height`, `min/max-width`, `min/max-height`
- layout/text: `align-items`, `justify-content`, `text-align`, `line-height`, `overflow`, `font-weight`, `font-style`

Widget parity tambahan:

- `Image`, `Spacer`, `Checkbox`, `Switch`, `Modal`
- `Fragment`, `If`, `For` (scope: `alias`, `aliasIndex`, `aliasKey`)
- `Modal` interaction: close button, backdrop click, dan tombol `Escape`
