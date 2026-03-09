# Quality Gates Formo (CI/CD Ready)

## AI Quick Context
- doc_path: knowledge-pack/docs/09-quality-gates.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Dokumen ini mendefinisikan quality gate minimal agar proyek Formo stabil di pengembangan harian dan rilis.

## 1) Gate Wajib Sebelum Merge

1. Format gate:
   - `cargo run -p formo-cli -- fmt --input main.fm --check` harus lulus.
2. Syntax + semantic gate:
   - `cargo run -p formo-cli -- check --input main.fm --json` harus lulus.
3. Diagnostic gate:
   - `cargo run -p formo-cli -- diagnose --input main.fm --json` tidak boleh gagal.
4. Build gate:
   - minimal `cargo run -p formo-cli -- build --target web --input main.fm --out dist-web` atau sesuai target produk.
5. Jika desktop didukung:
   - `cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop`.
   - review parity warning.
6. Jika produk membutuhkan kesetaraan lintas target:
   - `cargo run -p formo-cli -- build --target web --input main.fm --out dist-web --strict-parity`.
   - untuk target web, strict parity memerlukan feature `backend-desktop`.

## 2) Gate Rekomendasi untuk Release

1. `cargo run -p formo-cli -- build --target multi --input main.fm --out dist --prod`.
2. desktop executable release:
   - `cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --release-exe`.
3. strict parity lintas target:
   - `cargo run -p formo-cli -- build --target web --input main.fm --out dist-web --prod --strict-parity`.
4. benchmark:
   - `cargo run -p formo-cli -- bench --input main.fm --iterations 20 --warmup 3 --nodes 1000 --out dist-ci/bench/benchmark.json --max-compile-p95-ms 80 --max-first-render-p95-ms 60` dengan budget p95 compile + first render.
5. validasi artifact layout final.

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

1. output JSON dari `cargo run -p formo-cli -- check --input main.fm --json` dan `cargo run -p formo-cli -- diagnose --input main.fm --json`.
2. laporan benchmark JSON.
3. checksum atau daftar artifact build.
4. changelog singkat per release.

