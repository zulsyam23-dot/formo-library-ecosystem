# Library: tooling

## Tanggung jawab

- Menyediakan command CLI (`check`, `diagnose`, `build`, `bench`, `watch`, dll).
- Menjadi orchestrator pipeline lintas library.

## Input/Output

- Input: entry file `.fm` + opsi CLI.
- Output: diagnostic, artifact build, dan laporan benchmark.

## Batas domain

- Tidak boleh menduplikasi implementasi parser/typer/style.
- Fokus sebagai orchestration layer.

## Mapping implementasi saat ini

- `programs/formo-cli`

## Artefak migrasi fitur

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/cli-commands.md`
