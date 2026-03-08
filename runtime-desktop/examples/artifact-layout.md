# Runtime Desktop Artifact Layout

Contoh artifact hasil build target desktop (native Rust):

1. `app.native.json`
2. `app.native.rs`
3. `app.ir.json`
4. `native-app/Cargo.toml`
5. `native-app/src/main.rs`
6. `native-app/src/app.rs`
7. `native-app/src/model.rs`
8. `native-app/src/style.rs`
9. `native-app/src/render/mod.rs`
10. `native-app/src/render/flow.rs`
11. `native-app/src/render/controls.rs`
12. `native-app/src/render/media.rs`
13. `native-app/src/render/shared.rs`
14. `native-app/src/render/state.rs`
15. `native-app/README.md`
16. `readable/README.md`
17. `readable/native/components.json`
18. `readable/native/tokens.json`
19. `readable/native/diagnostics.json`
20. `readable/ir/components.json`
21. `readable/ir/nodes.json`
22. `readable/ir/styles.json`
23. `readable/ir/tokens.json`
24. `readable/ir/diagnostics.json`

Kontrak minimal:

- runtime target: `rust-native` (tanpa webview).
- style parity core:
  - visual: `color`, `background`, `border`, `border-radius`, `box-shadow`, `opacity`
  - spacing/sizing: `padding`, `margin`, `gap`, `width/height`, `min/max-width`, `min/max-height`
  - layout/text: `align-items`, `justify-content`, `text-align`, `line-height`, `overflow`, `font-weight`, `font-style`
- widget parity tambahan: `Image`, `Spacer`, `Checkbox`, `Switch`, `Modal`, `Fragment`, `If`, `For`
  - scope `For`: `alias`, `aliasIndex`, `aliasKey`
  - interaction `Modal`: close button, backdrop click, `Escape`
- host action bridge: trait `FormoDesktopHost`.
- state bridge: `FormoDesktopState::set_state_patch(...)` dan `replace_state(...)`.
- parity diagnostics: warning style/widget unsupported muncul di `app.native.json.diagnostics`.
- folder `readable/` berisi snapshot JSON terpecah untuk audit manusia/AI.
- scaffold GUI native bisa langsung dijalankan dengan `cd native-app && cargo run`.
- build executable release bisa dipicu dari CLI dengan `formo build --target desktop --release-exe`
  (hasil di `native-app/target/release`, `.exe` untuk Windows).
