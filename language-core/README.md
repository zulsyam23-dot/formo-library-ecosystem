# Library: language-core

## Tanggung jawab

- Definisi syntax `.fm`, lexer, parser, resolver, typer, dan model IR publik.

## Input/Output

- Input: file `.fm`.
- Output: AST tervalidasi, semantic info, dan IR.

## Batas domain

- Tidak mengurus rendering UI.
- Tidak mengurus host runtime web/desktop.

## Mapping implementasi saat ini

- `programs/formo-lexer`
- `programs/formo-parser`
- `programs/formo-resolver`
- `programs/formo-typer`
- `programs/formo-ir`

## Artefak migrasi fitur

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/app-main.fm`
