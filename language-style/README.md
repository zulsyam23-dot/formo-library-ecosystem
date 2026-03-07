# Library: language-style

## Tanggung jawab

- Definisi syntax `.fs`, token style, dan kompilasi style ke IR/style output.

## Input/Output

- Input: file `.fs`.
- Output: style registry, token map, dan diagnostic style.

## Batas domain

- Tidak memproses parser `.fm`.
- Tidak mengurus runtime rendering.

## Mapping implementasi saat ini

- `programs/formo-style`

## Artefak migrasi fitur

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/base.fs`
