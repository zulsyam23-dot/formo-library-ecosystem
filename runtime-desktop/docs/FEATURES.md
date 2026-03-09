# Features: runtime-desktop

Daftar fitur runtime desktop Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `desktop_native_bundle_emit`
   - Emit bundle desktop native Rust (`app.native.json`, `app.native.rs`).
2. `desktop_native_scaffold_emit`
   - Emit scaffold app GUI desktop Rust modular
     - `native-app/src/main.rs`, `native-app/src/app.rs`
     - `native-app/src/model.rs`, `native-app/src/style.rs`
     - `native-app/src/render/mod.rs`, `native-app/src/render/*.rs`
   - Runtime scaffold berbasis `dioxus-desktop` dengan renderer tree DOM + CSS inline.
3. `desktop_style_parity_core`
   - Mapper style inti untuk desktop native (canonical-first via `effective_style_decls(...)`):
     - visual: `color`, `background`, `border`, `border-radius`, `box-shadow`, `opacity`
     - spacing/sizing: `padding`, `margin`, `gap`, `width/height`, `min/max-width`, `min/max-height`
     - layout/text: `align-items`, `align-self`, `justify-content`, `text-align`, `line-height`, `overflow`, `font-weight`, `font-style`
     - flow/flex: `display`, `flex-direction`, `flex-wrap`, `flex`, `flex-grow`, `flex-shrink`, `flex-basis`
   - Aturan behavior parity:
     - `justify-content: space-*` hanya didistribusikan saat main-axis size eksplisit.
     - `%` pada width/min/max-width didukung; `%` pada height diperlakukan konservatif agar tidak over-expand.
     - alias align (`baseline`, `normal`, `self-start`, `self-end`, `safe/unsafe start/end`) dinormalisasi ke semantic desktop yang konsisten.
4. `desktop_widget_parity_extended`
   - Widget parity tambahan yang tersedia di scaffold native:
     - `Image`, `Spacer`, `Checkbox`, `Switch`, `Modal`
     - `Fragment`, `If`, `For` (scope: `alias`, `aliasIndex`, `aliasKey`)
     - `Modal` interaction parity dasar: close via tombol, backdrop click, dan tombol `Escape`
5. `desktop_host_actions`
   - Kontrak action host Rust via `FormoDesktopHost::invoke_action(...)`.
   - Scaffold `native-app/src/actions.rs` meng-generate registry handler berdasarkan action props yang terdeteksi dari IR (`onPress/onClick/onChange/onClose/action`).
6. `desktop_state_bridge`
   - Helper state bridge Rust: `set_state_patch(...)` dan `replace_state(...)`.
7. `ir_snapshot_emit`
   - Emit `app.ir.json` untuk debugging/integrasi host.
8. `desktop_parity_diagnostics`
   - Tambahkan warning parity desktop ke `app.native.json.diagnostics` untuk:
     - style property yang belum didukung penuh
     - widget yang masih fallback renderer
9. `desktop_readable_artifacts`
   - Emit snapshot terpecah untuk audit/readability:
     - `readable/native/components.json`, `readable/native/tokens.json`, `readable/native/diagnostics.json`
     - `readable/ir/components.json`, `readable/ir/nodes.json`, `readable/ir/styles.json`, `readable/ir/tokens.json`, `readable/ir/diagnostics.json`

## Mapping Implementasi

- `programs/formo-backend-desktop`
