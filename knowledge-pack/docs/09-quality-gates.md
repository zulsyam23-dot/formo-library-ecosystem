# Quality Gates Formo (CI/CD Ready)

Dokumen ini mendefinisikan quality gate minimal agar proyek Formo stabil di pengembangan harian dan rilis.

## 1) Gate Wajib Sebelum Merge

1. Format gate:
   - `fmt --check` harus lulus.
2. Syntax + semantic gate:
   - `check --json` harus lulus.
3. Diagnostic gate:
   - `diagnose --json` tidak boleh gagal.
4. Build gate:
   - minimal `build --target web` atau sesuai target produk.
5. Jika desktop didukung:
   - `build --target desktop`.
   - review parity warning.

## 2) Gate Rekomendasi untuk Release

1. `build --target multi --prod`.
2. desktop executable release:
   - `build --target desktop --release-exe`.
3. benchmark:
   - `bench` dengan budget p95 compile + first render.
4. validasi artifact layout final.

## 3) Contoh Pipeline Command

```bash
cargo run -p formo-cli -- fmt --input main.fm --check
cargo run -p formo-cli -- check --input main.fm --json
cargo run -p formo-cli -- diagnose --input main.fm --json
cargo run -p formo-cli -- build --target multi --input main.fm --out dist --prod
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --release-exe
cargo run -p formo-cli -- bench --input main.fm --iterations 20 --warmup 3 --nodes 1000 --out dist-ci/bench/benchmark.json --max-compile-p95-ms 80 --max-first-render-p95-ms 60
```

## 4) Gate untuk Pull Request

Checklist:

1. semua command gate lulus.
2. error/warning kritikal dijelaskan jika tidak bisa dihilangkan.
3. dokumen diperbarui jika ada perubahan kontrak fitur.

## 5) Gate untuk AI-Generated Patch

Wajib:

1. patch kecil dan fokus.
2. menyertakan command verifikasi.
3. reviewer manusia cek:
   - kompatibilitas syntax,
   - kompatibilitas runtime target,
   - risiko regression.

## 6) Kriteria Gagal (Fail Fast)

Pipeline harus fail jika:

1. stage parser/resolver/style/typer gagal.
2. build target yang diwajibkan gagal.
3. budget benchmark dilampaui.
4. artifact kunci tidak terbentuk.

## 7) Kriteria Lolos (Release Candidate)

Release candidate diterima jika:

1. gate wajib + gate release lulus.
2. desktop parity warning berada dalam batas yang disepakati tim.
3. output artifact bisa dijalankan sesuai target.

## 8) Audit Trail yang Direkomendasikan

Simpan:

1. output JSON `check`/`diagnose`.
2. laporan benchmark JSON.
3. checksum atau daftar artifact build.
4. changelog singkat per release.
