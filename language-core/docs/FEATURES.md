# Features: language-core

Daftar fitur inti bahasa Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `lexer_diagnostics`
   - Lexer mendukung diagnostic error code (`E1000`-`E1004`) dan posisi presisi.
2. `parser_recovery`
   - Parser dapat mengumpulkan beberapa syntax error dalam satu pass.
3. `import_resolution`
   - Resolver import `.fm`/`.fs`, termasuk deteksi siklus dan alias duplikat.
4. `type_validation`
   - Validasi root component, node built-in/custom, prop required/unknown/type mismatch.
5. `public_ir_contract`
   - Kontrak IR publik stabil dengan schema dan versioning.

## Mapping Implementasi

- `programs/formo-lexer`
- `programs/formo-parser`
- `programs/formo-resolver`
- `programs/formo-typer`
- `programs/formo-ir`
