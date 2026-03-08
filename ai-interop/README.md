# Library: ai-interop

`ai-interop` adalah lapisan kontrak untuk integrasi AI di ekosistem Formo.

## Apa yang Ditangani

- capability profile machine-readable
- normalized error envelope
- interop convention lintas library

## Apa yang Tidak Ditangani

- compile pipeline `.fm/.fs/.fl`
- runtime web/desktop execution
- command orchestration CLI

## Status dan Capability

- status kontrak: `bootstrap`
- capability utama:
  - `capability_profile`
  - `normalized_error_envelope`
  - `machine_readable_registry`

Sumber: `contracts/CAPABILITIES.json`.

## Input dan Output

- input:
  - `registry metadata`
  - `diagnostic payloads`
  - `library capability declarations`
- output:
  - `ai capability profile`
  - `normalized error envelope`
  - `interop conventions`

## Mapping Implementasi

- belum ada crate Rust (`maps_to_crates` kosong)
- fokus saat ini pada kontrak + konvensi integrasi

## Artefak Dokumentasi

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/error-envelope.json`
