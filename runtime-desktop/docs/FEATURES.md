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
3. `desktop_style_parity_core`
   - Mapper style inti untuk desktop native:
     - visual: `color`, `background`, `border`, `border-radius`, `box-shadow`, `opacity`
     - spacing/sizing: `padding`, `margin`, `gap`, `width/height`, `min/max-width`, `min/max-height`
     - layout/text: `align-items`, `justify-content`, `text-align`, `line-height`, `overflow`, `font-weight`, `font-style`
4. `desktop_widget_parity_extended`
   - Widget parity tambahan yang tersedia di scaffold native:
     - `Image`, `Spacer`, `Checkbox`, `Switch`, `Modal`
     - `Fragment`, `If`, `For` (scope: `alias`, `aliasIndex`, `aliasKey`)
     - `Modal` interaction parity dasar: close via tombol, backdrop click, dan tombol `Escape`
5. `desktop_host_actions`
   - Kontrak action host Rust via `FormoDesktopHost::invoke_action(...)`.
6. `desktop_state_bridge`
   - Helper state bridge Rust: `set_state_patch(...)` dan `replace_state(...)`.
7. `ir_snapshot_emit`
   - Emit `app.ir.json` untuk debugging/integrasi host.
8. `desktop_parity_diagnostics`
   - Tambahkan warning parity desktop ke `app.native.json.diagnostics` untuk:
     - style property yang belum didukung penuh
     - widget yang masih fallback renderer

## Mapping Implementasi

- `programs/formo-backend-desktop`
