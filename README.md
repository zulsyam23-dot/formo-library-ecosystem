# Formo Library Ecosystem

Folder ini memisahkan ekosistem library Formo berdasarkan fungsi agar:

- batas domain jelas (core bahasa vs runtime vs tooling),
- mudah diindeks oleh AI agent,
- mudah diekstrak menjadi paket terpisah di tahap berikutnya.

Gunakan file registri berikut sebagai entry point otomatis untuk AI:

- `registry.formo-ai.json`
- `Cargo.toml` (workspace Cargo mandiri untuk semua program library)

Lihat aturan penulisan AI-native:

- `AI_NATIVE_RULES.md`

Status migrasi fitur:

- `MIGRATION_STATUS.md`
- `mandatory-features.formo-ai.json`
