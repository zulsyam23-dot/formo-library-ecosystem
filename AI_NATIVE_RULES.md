# AI-Native Rules (Formo)

Tujuan aturan ini adalah membuat struktur Formo konsisten dan mudah dipahami AI.

## 1) Kontrak eksplisit

- Setiap library wajib punya `README.md` yang menjawab:
  - tanggung jawab utama,
  - input/output,
  - dependensi yang diizinkan,
  - yang bukan tanggung jawabnya.

## 2) Naming stabil

- Gunakan nama domain yang deskriptif (`language-core`, `runtime-web`, `ai-interop`).
- Hindari singkatan ambigu.
- Pertahankan ID library stabil di `registry.formo-ai.json`.

## 3) Batas domain ketat

- Core bahasa tidak boleh tergantung runtime.
- Runtime hanya memakai IR publik, tidak memakai internal parser detail.
- Tooling boleh orkestrasi, tetapi tidak menyalin logika domain core.

## 4) Metadata machine-readable dulu

- Semua library harus terdaftar di `registry.formo-ai.json`.
- Perubahan penting:
  - naikkan `version`,
  - isi `status`,
  - perbarui `updated_at`.

## 5) Jalur migrasi aman

- Lakukan pemisahan bertahap:
  - dokumentasikan dulu,
  - pindahkan implementasi saat siap,
  - jaga kompatibilitas CLI/build yang berjalan.
