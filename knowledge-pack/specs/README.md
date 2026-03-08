# Specs

## AI Quick Context
- doc_path: knowledge-pack/specs/README.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Spesifikasi teknis library knowledge-pack.

## Ruang Lingkup

Knowledge-pack memodelkan dokumentasi operasional Formo untuk:

- developer manusia,
- AI coding assistant,
- integrasi workflow tooling.

## Struktur Spesifikasi Dokumentasi

1. Referensi inti:
   - quickstart,
   - language reference,
   - style reference,
   - CLI/runtime reference.
2. Operasional:
   - programming techniques,
   - troubleshooting diagnostics,
   - quality gates.
3. AI enablement:
   - AI playbook,
   - prompt templates,
   - glossary.

## Kriteria Kualitas

1. Akurat terhadap implementasi terkini.
2. Dapat dieksekusi (contoh command valid).
3. Mudah dinavigasi (index jelas).
4. Mudah diproses AI (istilah konsisten + checklist eksplisit).

