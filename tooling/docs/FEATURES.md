# Features: tooling

Daftar fitur tooling Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `check_diagnose_commands`
   - `check`, `diagnose`, `doctor`, termasuk mode JSON dan schema.
   - termasuk `logic` untuk validasi file `.fl`.
   - `logic` mendukung ekspor runtime contract via `--rt-manifest-out`.
   - `logic` menegakkan strict profile (event/function naming, typed function params, enum/struct/type rules, logic global-core action, service platform-agnostic, adapter call-only, parity platform simetris, throw try/catch-only, break/continue loop-only, return-last-action).
2. `build_multi_target`
   - `build --target web|desktop|multi`.
   - `build --strict-parity` untuk memaksa zero desktop parity warning (`W7601`/`W7602`).
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
