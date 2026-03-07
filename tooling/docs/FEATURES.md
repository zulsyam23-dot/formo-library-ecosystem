# Features: tooling

Daftar fitur tooling Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `check_diagnose_commands`
   - `check`, `diagnose`, `doctor`, termasuk mode JSON dan schema.
2. `build_multi_target`
   - `build --target web|desktop|multi`.
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
