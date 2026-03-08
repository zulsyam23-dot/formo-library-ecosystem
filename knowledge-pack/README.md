# Library: knowledge-pack

`knowledge-pack` adalah library dokumentasi operasional Formo yang dioptimalkan untuk konsumsi AI dan developer.

## Apa yang Ditangani

- domain glossary
- quickstart dan language/style/runtime references
- troubleshooting matrix
- AI playbook dan prompt patterns
- learning path, architecture playbook, quality gates, integration checklist

## Apa yang Tidak Ditangani

- compile pipeline Rust crate
- runtime backend execution
- parser/typer/style implementation

## Status dan Capability

- status kontrak: `active`
- capability utama:
  - `domain_glossary`
  - `quickstart_guide`
  - `formo_language_reference`
  - `formo_style_reference`
  - `cli_runtime_reference`
  - `programming_techniques`
  - `troubleshooting_matrix`
  - `prompt_patterns`
  - `case_study_examples`
  - `learning_path_curriculum`
  - `project_architecture_playbook`
  - `quality_gates_reference`
  - `integration_checklist`
  - `ai_playbook`

Sumber: `contracts/CAPABILITIES.json`.

## Input dan Output

- input:
  - `official formo docs`
  - `feature contracts`
  - `diagnostic semantics`
- output:
  - dokumen referensi siap pakai untuk manusia dan AI
  - contoh prompt/checklist untuk workflow harian

## Mapping Implementasi

- tidak memiliki crate Rust (`maps_to_crates` kosong)
- implementasi utama berupa dokumen terstruktur

## Struktur Kunci

- `docs/` - referensi utama
- `examples/` - glossary, case studies, prompt templates, integration checklist
- `contracts/` - capability contract library
- `specs/` - ruang lingkup dan quality criteria

## Cara Pakai Cepat

1. Mulai dari `docs/README.md`.
2. Gunakan `docs/00-quickstart.md` untuk alur end-to-end.
3. Gunakan `examples/ai-prompts.md` saat pairing dengan AI agent.
4. Gunakan `examples/integration-checklist.md` untuk gate sebelum release.
