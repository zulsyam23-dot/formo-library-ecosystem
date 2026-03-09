# Docs: Formo Complete Knowledge Pack

## AI Quick Context
- doc_path: knowledge-pack/docs/README.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Dokumentasi ini disusun sebagai manual lengkap penggunaan:

- library Formo (pipeline + tooling),
- bahasa Formo (`.fm`),
- bahasa logic Formo (`.fl`),
- bahasa style Formo (`.fs`),
- workflow build web/desktop,
- teknik pemrograman dari basic sampai advanced,
- pola kerja yang mudah dipahami AI agent.

Baseline engine terbaru:

- `FM` + `FL` + `FS` adalah kontrak standar sumber.
- layer runtime wajib membaca style canonical yang sama (`effective_style_decls(...)`) agar hasil web/desktop sinkron.

## Peta Dokumen

1. [00-quickstart.md](./00-quickstart.md)
   - Mulai dari nol sampai aplikasi pertama jalan.
2. [01-formo-language-reference.md](./01-formo-language-reference.md)
   - Referensi rinci syntax/semantik bahasa Formo.
3. [02-formo-style-reference.md](./02-formo-style-reference.md)
   - Referensi rinci style, token, value, dan aturan validasi.
4. [03-cli-build-runtime.md](./03-cli-build-runtime.md)
   - Semua command CLI, artifact, target web/desktop/multi, release exe.
5. [04-programming-techniques.md](./04-programming-techniques.md)
   - Berbagai teknik pemrograman (komposisi, slot, flow data, parity-first).
6. [05-troubleshooting-diagnostics.md](./05-troubleshooting-diagnostics.md)
   - Debugging dan perbaikan error berdasarkan stage dan code.
7. [06-ai-playbook.md](./06-ai-playbook.md)
   - Panduan prompt dan format kerja yang optimal untuk AI.
8. [07-learning-path.md](./07-learning-path.md)
   - Kurikulum belajar bertahap (7 hari) dari basic hingga production workflow.
9. [08-project-architecture.md](./08-project-architecture.md)
   - Blueprint arsitektur project Formo untuk skala kecil, menengah, dan besar.
10. [09-quality-gates.md](./09-quality-gates.md)
   - Quality gate dan checklist CI/CD untuk menjaga stabilitas.

## Dokumen Contoh

- [../examples/glossary.md](../examples/glossary.md)
- [../examples/case-studies.md](../examples/case-studies.md)
- [../examples/ai-prompts.md](../examples/ai-prompts.md)
- [../examples/integration-checklist.md](../examples/integration-checklist.md)

## Prinsip Penulisan

- Contoh selalu runnable (mengikuti fitur yang ada di kode).
- Command ditulis dalam bentuk runnable penuh (`cargo run -p formo-cli -- ...`) untuk menghindari ambiguitas parsing AI.
- Istilah konsisten dengan implementasi (`check`, `diagnose`, `build`, `doctor`, `bench`).
- Fokus pada akurasi teknis dan langkah praktis.

