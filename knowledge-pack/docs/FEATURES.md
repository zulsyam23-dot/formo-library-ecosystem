# Features: knowledge-pack

## AI Quick Context
- doc_path: knowledge-pack/docs/FEATURES.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Daftar fitur knowledge pack dokumentasi Formo yang wajib tersedia.

## Fitur Wajib

1. `domain_glossary`
   - Istilah domain Formo agar AI tidak salah interpretasi.
2. `quickstart_guide`
   - Panduan mulai cepat dari nol sampai build web/desktop.
3. `formo_language_reference`
   - Referensi rinci bahasa `.fm` (component, built-in, props, tipe data, scope).
4. `formo_style_reference`
   - Referensi rinci bahasa `.fs` (token, selector, value type, allowlist, canonical style baseline).
5. `cli_runtime_reference`
   - Referensi command CLI Formo, artifact, target runtime, strict parity, dan workflow CI.
6. `programming_techniques`
   - Kumpulan teknik pemrograman Formo untuk berbagai kasus (basic-advanced).
7. `troubleshooting_matrix`
   - Matrix stage error, code, penyebab, dan langkah perbaikan.
8. `ai_playbook`
   - Prompt pattern, checklist output AI, dan definition of done task berbasis AI.
9. `case_study_examples`
   - Kumpulan contoh kasus end-to-end yang mudah diadaptasi.
10. `learning_path_curriculum`
    - Kurikulum bertahap untuk onboarding developer baru.
11. `project_architecture_playbook`
    - Blueprint struktur project untuk skala kecil, menengah, dan besar.
12. `quality_gates_reference`
    - Standar quality gate dan checklist CI/CD berbasis command Formo.
13. `integration_checklist`
    - Checklist integrasi lintas tahap (struktur project, validasi, build, dan release).
14. `engine_standard_reference`
    - Panduan menjaga standar engine `FM/FL/FS` agar web dan desktop memakai semantic style canonical yang sama.

## Catatan

- Knowledge pack ini diposisikan sebagai sumber dokumentasi operasional untuk manusia dan AI.
- Fokus utama: akurasi implementasi saat ini + langkah praktis yang bisa langsung dijalankan.

