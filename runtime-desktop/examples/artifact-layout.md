# Runtime Desktop Artifact Layout

Contoh artifact hasil build target desktop (native Rust):

1. `app.native.json`
2. `app.native.rs`
3. `app.ir.json`
4. `native-app/Cargo.toml`
5. `native-app/src/main.rs`
6. `native-app/src/actions.rs`
7. `native-app/src/app.rs`
8. `native-app/src/model.rs`
9. `native-app/src/style.rs`
10. `native-app/src/render/mod.rs`
11. `native-app/src/render/flow.rs`
12. `native-app/src/render/controls.rs`
13. `native-app/src/render/media.rs`
14. `native-app/src/render/shared.rs`
15. `native-app/src/render/state.rs`
16. `native-app/README.md`
17. `readable/README.md`
18. `readable/native/components.json`
19. `readable/native/tokens.json`
20. `readable/native/diagnostics.json`
21. `readable/ir/components.json`
22. `readable/ir/nodes.json`
23. `readable/ir/styles.json`
24. `readable/ir/tokens.json`
25. `readable/ir/diagnostics.json`

Kontrak minimal:

- runtime target: `rust-native` dengan scaffold `dioxus-desktop` (DOM + CSS mapping).
- style parity core (canonical-first via `effective_style_decls(...)`):
  - visual: `color`, `background`, `border`, `border-radius`, `box-shadow`, `opacity`
  - spacing/sizing: `padding`, `margin`, `gap`, `width/height`, `min/max-width`, `min/max-height`
  - layout/text: `align-items`, `align-self`, `justify-content`, `text-align`, `line-height`, `overflow`, `font-weight`, `font-style`
  - flow/flex: `display`, `flex-direction`, `flex-wrap`, `flex`, `flex-grow`, `flex-shrink`, `flex-basis`
- behavior parity penting:
  - `justify-content: space-*` aktif membagi ruang hanya jika main-axis size eksplisit.
  - `%` width/min/max-width didukung langsung; `%` height diproses konservatif agar tidak meluber.
  - alias align (`baseline`, `normal`, `self-start`, `self-end`, `safe/unsafe start/end`) dinormalisasi.
- widget parity tambahan: `Image`, `Spacer`, `Checkbox`, `Switch`, `Modal`, `Fragment`, `If`, `For`
  - scope `For`: `alias`, `aliasIndex`, `aliasKey`
  - interaction `Modal`: close button, backdrop click, `Escape`
- host action bridge: trait `FormoDesktopHost`.
- `native-app/src/actions.rs` memuat registry action dan evaluator expression (`eval_set_expression`, `eval_set_expression_rpn`) untuk sinkronisasi event FL.
- state bridge: `FormoDesktopState::set_state_patch(...)` dan `replace_state(...)`.
- parity diagnostics: warning style/widget unsupported muncul di `app.native.json.diagnostics`.
- folder `readable/` berisi snapshot JSON terpecah untuk audit manusia/AI.
- scaffold GUI native bisa langsung dijalankan dengan `cd native-app && cargo run`.
- build executable release bisa dipicu dari CLI dengan `formo build --target desktop --release-exe`
  (hasil di `native-app/target/release`, `.exe` untuk Windows).
