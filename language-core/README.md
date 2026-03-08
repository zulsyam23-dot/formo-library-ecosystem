# Library: language-core

`language-core` adalah fondasi compiler Formo untuk bahasa `.fm` dan `.fl`.

## Apa yang Ditangani

- lexer diagnostics
- parser + recovery
- import resolution
- type validation
- public IR contract
- strict profile untuk logic `.fl`

## Apa yang Tidak Ditangani

- rendering runtime web/desktop
- style compilation `.fs`
- orchestration CLI end-to-end

## Status dan Capability

- status kontrak: `active`
- capability utama:
  - `lexer_diagnostics`
  - `parser_recovery`
  - `import_resolution`
  - `type_validation`
  - `public_ir_contract`

Sumber: `contracts/CAPABILITIES.json`.

## Input dan Output

- input:
  - `.fm source files`
  - `.fs import references`
  - `component graph`
- output:
  - `tokens`
  - `ast`
  - `resolved modules`
  - `typed semantic model`
  - `public ir`

## Mapping Implementasi

- `programs/formo-lexer`
- `programs/formo-parser`
- `programs/formo-logic`
- `programs/formo-resolver`
- `programs/formo-typer`
- `programs/formo-ir`

## Aturan Integrasi Library

- import `fm/fs` via URI library:
  - `import "lib://nama-library/modul.fm" as Alias;`
  - `import "lib://nama-library/theme.fs" as Theme;`
- import `fl` via:
  - `use "lib://nama-library/core.fl" as Core;`

Resolver root search order:

1. env `FORMO_LIBRARY_ROOT`
2. `<project>/fl-libraries`
3. `<project>/../fl-libraries`
4. `~/Documents/fl-libraries`

## Validasi Cepat

```bash
cargo test -p formo-lexer
cargo test -p formo-parser
cargo test -p formo-logic
cargo test -p formo-resolver
cargo test -p formo-typer
cargo test -p formo-ir
```

## Artefak Dokumentasi

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/app-main.fm`
- `examples/app-controller.fl`
