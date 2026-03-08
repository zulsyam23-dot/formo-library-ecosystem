# Quickstart Formo (End-to-End)

Dokumen ini untuk memulai cepat dan benar, dari nol sampai artifact web/desktop berhasil dibuat.

## 1) Prasyarat

- Rust + Cargo terpasang.
- Struktur workspace Formo tersedia.
- File entry aplikasi: `main.fm`.

## 2) Struktur Minimum Project

Contoh struktur:

```text
project/
  main.fm
  styles/
    base.fs
  views/
    header.fm
```

## 3) Tulis Aplikasi Formo Pertama

`main.fm`:

```fm
import "styles/base.fs" as Base;

component App() {
  <Page>
    <Text value="Halo Formo" style=BodyText/>
  </Page>
}
```

`styles/base.fs`:

```fs
token {
  color.primary = #0A84FF;
  space.md = 12dp;
}

style BodyText {
  color: token(color.primary);
  margin-top: token(space.md);
}
```

## 4) Validasi Pipeline

```bash
cargo run -p formo-cli -- check --input main.fm
```

Jika valid, output:

```text
check passed: main.fm
```

## 5) Diagnostik JSON (untuk tooling/AI)

```bash
cargo run -p formo-cli -- diagnose --input main.fm --json-pretty
```

Gunakan mode ini jika ingin:

- membaca stage error secara terstruktur,
- diproses LSP/editor/AI agent.

## 6) Build Target Web

```bash
cargo run -p formo-cli -- build --target web --input main.fm --out dist-web
```

Artifact umum:

- `dist-web/index.html`
- `dist-web/app.js`
- `dist-web/app.css`

## 7) Build Target Desktop Native

```bash
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop
```

Artifact umum:

- `dist-desktop/app.native.json`
- `dist-desktop/app.native.rs`
- `dist-desktop/app.ir.json`
- `dist-desktop/native-app/*`

Jalankan app desktop scaffold:

```bash
cd dist-desktop/native-app
cargo run
```

## 8) Build Langsung ke Executable Release (Desktop)

```bash
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --release-exe
```

Hasil binary:

- `dist-desktop/native-app/target/release`
- di Windows: file `.exe` berada di folder itu.

## 9) Build Multi Target

```bash
cargo run -p formo-cli -- build --target multi --input main.fm --out dist
```

Output:

- web: `dist/web/*`
- desktop: `dist/desktop/*`

## 10) Mode Watch

```bash
cargo run -p formo-cli -- check --input main.fm --watch
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --watch
```

Cocok saat iterasi harian.

## 11) Workflow Harian yang Direkomendasikan

1. `fmt --check` sebelum commit.
2. `check` untuk validasi cepat.
3. `diagnose --json` untuk integrasi tooling.
4. `build --target web|desktop|multi`.
5. desktop: pantau parity warning di hasil build.

## 12) Checklist Selesai Quickstart

- Aplikasi `.fm` berhasil di-`check`.
- Style `.fs` berhasil di-compile tanpa error.
- Build web menghasilkan artifact.
- Build desktop menghasilkan `native-app`.
- Opsi `--release-exe` berhasil menghasilkan binary release.
