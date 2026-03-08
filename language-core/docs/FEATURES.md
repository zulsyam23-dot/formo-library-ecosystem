# Features: language-core

Daftar fitur inti bahasa Formo yang wajib tersedia di library ini.

## Fitur Wajib

1. `lexer_diagnostics`
   - Lexer mendukung diagnostic error code (`E1000`-`E1004`) dan posisi presisi.
2. `parser_recovery`
   - Parser dapat mengumpulkan beberapa syntax error dalam satu pass.
3. `import_resolution`
   - Resolver import `.fm`/`.fs`, termasuk deteksi siklus dan alias duplikat.
   - Mendukung library URI `lib://<library>/<module>.fm|.fs`.
   - Library root discovery berurutan: `FORMO_LIBRARY_ROOT`, `<project>/fl-libraries`, `<project>/../fl-libraries`, lalu `~/Documents/fl-libraries`.
4. `type_validation`
   - Validasi root component, node built-in/custom, prop required/unknown/type mismatch.
5. `public_ir_contract`
   - Kontrak IR publik stabil dengan schema dan versioning.
6. `logic_layer_parser`
   - Parser `.fl` untuk layer logika deklaratif (`module/use/logic/service/contract/adapter`).
   - Mendukung deklarasi standar unit: `enum`, `struct`, `type`, dan `function`.
   - `use` path mendukung file lokal `.fl` dan library URI `lib://<library>/<module>.fl`.
   - Mendukung deklarasi `state { ... }` dengan field typed + initializer.
   - Mendukung kontrol logika standar di level event: `if`, `for`, `while`, `match`.
   - Mendukung kontrol error handling: `try`, `catch`, `action throw`.
   - Mendukung aksi kontrol standar: `action break`, `action continue`, `action return`.
   - Validasi produksi:
     - alias unik, unit unik, event unik,
     - event wajib `lowerCamelCase`,
     - `function` wajib `lowerCamelCase`,
     - parameter `function` wajib bertipe (`name: Type`),
     - `struct` wajib PascalCase, field `struct` wajib lowerCamelCase dan bertipe (`field: Type`),
     - `type` alias wajib PascalCase (`type Name = Some.Type;`),
     - `enum` wajib PascalCase dan variant enum wajib PascalCase,
     - setiap blok `if/for/while/match` wajib berisi minimal satu `action`,
     - setiap blok `try/catch` wajib berisi minimal satu `action`,
     - `action throw` wajib berada di dalam blok `try` atau `catch`,
     - `action break`/`action continue` wajib berada di dalam blok `for` atau `while`,
     - `action return` wajib menjadi aksi terakhir di event,
     - valid call target `Alias.memberPath` (`memberPath` wajib `lowerCamelCase`),
     - `logic` wajib punya minimal satu global action per event,
     - `service` wajib platform-agnostic (tanpa `platform web/desktop`),
     - `logic/service` dilarang memanggil runtime alias `Browser`/`Desktop` secara langsung (harus via adapter/contract),
     - `adapter` hanya boleh `action call`,
     - parity web-desktop simetris per-event untuk unit `logic/adapter`,
     - urutan blok platform harus desktop dulu, lalu web (desktop-first policy),
     - pada unit `logic`, aksi di dalam blok platform hanya boleh `action call`,
     - pada unit `logic/adapter`, aksi global wajib sebelum blok platform,
     - pada unit `logic/adapter`, blok platform tidak boleh interleaving (desktop lalu web),
     - field `state` wajib lowerCamelCase + typed + initializer literal sesuai tipe dasar (`bool/string/int/float`),
     - `action set` wajib menarget field yang sudah dideklarasikan di blok `state` dan wajib ditutup `;`.
     - `action set` melakukan validasi mismatch literal dasar terhadap tipe field state (`bool/string/int/float`).
     - expression RHS `action set` wajib hanya mereferensikan state field terdaftar dan operand type harus kompatibel dengan target field.
     - inferensi ekspresi dasar (`+ - * / %`, `== != < <= > >=`, `&& ||`) dipakai untuk validasi tipe `action set`.

## Mapping Implementasi

- `programs/formo-lexer`
- `programs/formo-parser`
- `programs/formo-logic`
- `programs/formo-resolver`
- `programs/formo-typer`
- `programs/formo-ir`
