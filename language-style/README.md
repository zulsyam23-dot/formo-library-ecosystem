# Library: language-style

`language-style` adalah compiler style Formo untuk file `.fs`.

## Apa yang Ditangani

- style parser
- token system
- style allowlist validation
- unused token detection
- style reference validation

## Apa yang Tidak Ditangani

- parser/typer `.fm`
- runtime rendering web/desktop
- orchestration CLI lintas target

## Status dan Capability

- status kontrak: `active`
- capability utama:
  - `style_parser`
  - `token_system`
  - `style_allowlist`
  - `unused_token_detection`
  - `style_reference_validation`

Sumber: `contracts/CAPABILITIES.json`.

## Input dan Output

- input:
  - `.fs style files`
  - `style references from fm`
- output:
  - `style registry`
  - `token map`
  - `style diagnostics`

## Mapping Implementasi

- `programs/formo-style`

## Integrasi dengan Library URI

- style module bisa diimport dari library:
  - `import "lib://matimatika/base.fs" as MathBase;`
- resolusi URI `lib://` dilakukan oleh `formo-resolver`.

## Validasi Cepat

```bash
cargo test -p formo-style
```

## Artefak Dokumentasi

- `docs/FEATURES.md`
- `contracts/CAPABILITIES.json`
- `examples/base.fs`
