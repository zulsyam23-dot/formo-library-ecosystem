# AI Playbook untuk Formo

Dokumen ini merinci cara paling efektif menggunakan AI untuk menulis, mereview, dan memelihara project Formo.

## 1) Tujuan Playbook

- Mengurangi ambiguitas prompt.
- Menjaga output AI konsisten dengan syntax Formo.
- Memastikan patch AI selalu bisa diverifikasi.

## 2) Konteks Minimum yang Wajib Diberikan ke AI

Saat meminta AI mengubah kode Formo, sertakan:

1. file aktif (`main.fm`, `.fs`, atau file runtime/tooling),
2. target output (web/desktop/multi),
3. batasan teknis (misal: tanpa ubah parser, fokus style saja),
4. expected result yang bisa diuji command.

## 3) Template Prompt Implementasi Fitur

```text
Tujuan:
- Tambah fitur [nama fitur].

Konteks:
- File utama: [path]
- Target: [web/desktop/multi]
- Batasan: [contoh: jangan ubah kontrak IR]

Kriteria selesai:
- [daftar]

Validasi:
- jalankan [command]
```

## 4) Template Prompt Perbaikan Error

```text
Saya dapat error:
[paste error JSON atau error text]

Tolong:
1) identifikasi stage error,
2) jelaskan akar masalah,
3) berikan patch minimal,
4) berikan command verifikasi.
```

## 5) Template Prompt Refactor Aman

```text
Refactor file [path] dengan tujuan [tujuan].

Aturan:
- jangan ubah perilaku output,
- pecah file jika perlu,
- tambahkan test bila relevan,
- tampilkan ringkasan risiko.
```

## 6) Format Output AI yang Direkomendasikan

Minta AI menjawab dalam format:

1. ringkasan perubahan,
2. daftar file yang diubah,
3. command verifikasi,
4. risiko/impact,
5. next step.

## 7) Prompt Pattern untuk Bahasa Formo

### Pattern A: Generate komponen dari requirement UI

```text
Buat komponen Formo untuk:
- [fitur]
- props: [daftar props + tipe]
- gunakan built-in [list]
- wajib gunakan For/If bila cocok
- hasilkan file .fm + .fs
```

### Pattern B: Migrasi hardcoded style ke token

```text
Refactor style berikut agar semua nilai berulang dipindah ke token.
Pertahankan output visual semirip mungkin.
```

### Pattern C: Parity desktop optimization

```text
Tinjau style ini agar aman untuk desktop parity.
Utamakan property core dan beri fallback bila perlu.
```

## 8) Prompt Pattern untuk Tooling/CI

```text
Rancang pipeline CI Formo:
- fmt check
- check json
- diagnose json
- build multi
- bench budget

Berikan script command dan kriteria gagal.
```

## 9) Checklist Kualitas Output AI

1. Syntax `.fm` valid.
2. Syntax `.fs` valid.
3. Tidak pakai built-in/prop di luar referensi.
4. Tidak menulis style property di luar allowlist.
5. Ada command verifikasi.
6. Ada impact/risk note.

## 10) Anti-Pattern Prompt ke AI

1. Prompt terlalu umum tanpa file target.
2. Meminta AI ubah banyak domain sekaligus tanpa prioritas.
3. Tidak meminta langkah verifikasi.
4. Tidak menyediakan error yang lengkap saat debugging.

## 11) Integrasi AI Agent dalam Tim

Model operasional:

1. Developer menulis intent + scope.
2. AI menyiapkan patch kecil.
3. Developer review + run command.
4. AI bantu dokumentasi dan regression checklist.

## 12) Definition of Done untuk Task via AI

Task dianggap selesai jika:

1. patch sesuai intent,
2. command verifikasi sukses,
3. dokumentasi terbarui jika ada kontrak/fitur baru,
4. tidak ada warning kritikal yang diabaikan.
