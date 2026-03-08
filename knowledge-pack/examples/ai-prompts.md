# AI Prompt Templates untuk Formo

## AI Quick Context
- doc_path: knowledge-pack/examples/ai-prompts.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


## 1) Prompt Generate Fitur Baru

```text
Konteks:
- Project: Formo
- File aktif: [path]
- Target runtime: [web/desktop/multi]

Tugas:
- Buat fitur [nama fitur] menggunakan bahasa Formo (.fm) dan Formo Style (.fs).
- Gunakan built-in yang tersedia.
- Pastikan style property mengikuti allowlist.

Output yang saya minta:
1) patch file
2) penjelasan singkat
3) command verifikasi
```

## 2) Prompt Debug Error

```text
Saya dapat error berikut:
[paste error]

Tolong:
1) identifikasi stage error,
2) sebutkan akar masalah,
3) berikan patch minimal,
4) berikan command cek ulang.
```

## 3) Prompt Refactor Komponen

```text
Refactor komponen di [path]:
- pecah jadi komponen reusable,
- gunakan Slot bila perlu,
- jangan ubah behavior.

Tambahkan:
- ringkasan file yang diubah,
- checklist risiko.
```

## 4) Prompt Migrasi Style ke Token

```text
Refactor style berikut:
- pindahkan nilai berulang ke token,
- pertahankan visual,
- beri nama token yang semantik.
```

## 5) Prompt Optimasi Parity Desktop

```text
Tinjau style dan node berikut agar lebih parity-friendly untuk desktop:
[paste snippet]

Target:
- minim warning parity
- tetap mirip web
- sebutkan tradeoff jika ada properti unsupported
```

## 6) Prompt Build & Release Checklist

```text
Buat checklist rilis Formo:
- cargo run -p formo-cli -- fmt --input main.fm --check
- cargo run -p formo-cli -- check --input main.fm --json
- cargo run -p formo-cli -- diagnose --input main.fm --json
- cargo run -p formo-cli -- build --target multi --input main.fm --out dist --prod
- cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --release-exe
- validasi artifact
```

## 7) Prompt Review Code (Bug-Focused)

```text
Lakukan review kode ini dengan fokus:
- bug fungsional
- regression risk
- missing test
- mismatch web vs desktop

Output:
1) temuan (urut severity)
2) asumsi/open question
3) ringkasan perubahan yang disarankan
```

