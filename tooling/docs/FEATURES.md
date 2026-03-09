# Features: tooling

Daftar fitur tooling Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `check_diagnose_commands`
   - `check`, `diagnose`, `doctor`, termasuk mode JSON dan schema.
   - `check|diagnose|doctor|fmt|lsp|bench|build` mewajibkan input berekstensi `.fm` (fail-fast).
   - termasuk `logic` untuk validasi file `.fl`.
   - `logic` mewajibkan input berekstensi `.fl` (fail-fast).
   - `logic` mendukung ekspor runtime contract via `--rt-manifest-out`.
   - `logic` menegakkan strict profile (event/function naming, typed function params, enum/struct/type rules, logic global-core action, service platform-agnostic, adapter call-only, parity platform simetris, throw try/catch-only, break/continue loop-only, return-last-action).
2. `build_multi_target`
   - `build --target web|desktop|multi`.
   - `build --strict` mengaktifkan strict profile gabungan (`--strict-parity` + `--strict-engine`).
   - `build --strict-parity` untuk memaksa zero desktop parity warning (`W7601`/`W7602`) pada target `web`, `desktop`, maupun `multi`.
   - Pada target `web`, `--strict-parity` membutuhkan feature `backend-desktop` agar audit parity dapat dijalankan.
   - Jika audit parity desktop gagal pada target `web`, CLI menulis `desktop.parity.json` di output web.
   - CLI menulis `engine.bridge.json` di output build untuk audit standar `FM/FS/FL` (canonical style + logic bridge).
   - `build --strict-engine` memaksa zero warning bridge (`W7701`..`W7705`) pada target `web`, `desktop`, maupun `multi`.
   - `W7705` dipakai saat action binding FM tidak punya event FL yang cocok.
   - Pada target `desktop|multi`, CLI menyinkronkan `native-app/src/actions.rs` dengan event FL (termasuk body handler global `set/call/emit` sebagai runtime scaffold awal).
   - Mapping `action set` memakai metadata operand/operator FL; kasus langsung (`stateRef`/literal tunggal) dirender sebagai assignment state yang lebih presisi.
3. `watch_mode`
   - Watch mode untuk `check`, `diagnose`, dan `build`.
4. `benchmark_mode`
   - `bench` dengan laporan JSON dan budget gate.
5. `lsp_output`
   - Output diagnostics untuk mode LSP/JSON-RPC.
6. `optional_backend_features`
   - Backend renderer dipanggil opsional lewat feature:
     - `backend-web`
     - `backend-desktop`

## Mapping Implementasi

- `programs/formo-cli`
