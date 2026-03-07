# Features: language-style

Daftar fitur style Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `style_parser`
   - Parser `.fs` untuk style block, part style, dan deklarasi token.
2. `token_system`
   - Dukungan `token(name)` dan `token(name, fallback)`.
3. `style_allowlist`
   - Validasi property style hanya untuk key yang diizinkan (plus `--*`).
4. `unused_token_detection`
   - Token yang tidak terpakai terdeteksi sebagai compile error.
5. `style_reference_validation`
   - Referensi `style=<Ref>` tervalidasi terhadap registry style.

## Mapping Implementasi

- `programs/formo-style`
