# Learning Path Formo (7 Hari)

## AI Quick Context
- doc_path: knowledge-pack/docs/07-learning-path.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Kurikulum ini dirancang agar developer baru bisa produktif cepat dan tetap mengikuti praktik terbaik Formo.

## Hari 1: Dasar Bahasa Formo

Target:

- paham struktur file `.fm`,
- bisa menulis `component` sederhana,
- paham root node rule.

Latihan:

1. buat `App` yang menampilkan `Text`.
2. gunakan `Page`, `Row`, `Column`.
3. sengaja buat 2 root node, lihat error `E2002`, lalu perbaiki.

Command:

```bash
cargo run -p formo-cli -- check --input main.fm
```

## Hari 2: Props, Type, dan Built-in

Target:

- paham prop required/optional,
- paham type mismatch.

Latihan:

1. gunakan `Button`, `Input`, `Checkbox`.
2. sengaja kirim tipe salah untuk prop built-in, amati `E2251`.
3. perbaiki sampai `cargo run -p formo-cli -- check --input main.fm` bersih.

## Hari 3: Style Formo dan Token

Target:

- paham `.fs`, token, selector, allowlist.

Latihan:

1. buat token warna + spacing.
2. terapkan style ke node.
3. sengaja tambah properti invalid untuk melihat `E1301`.
4. perbaiki dan pastikan token tidak unused (`E1304`).

## Hari 4: For/If dan Komponen Reusable

Target:

- paham render list + conditional,
- paham slot composition.

Latihan:

1. list users dengan `<For each=... as=item>`.
2. tampilkan badge aktif dengan `<If when=...>`.
3. buat wrapper dengan `<Slot/>`.
4. sengaja inline children ke component tanpa slot, amati `E2304`.

## Hari 5: CLI Deep Dive

Target:

- paham `check`, `diagnose`, `doctor`, `fmt`.

Latihan:

1. jalankan `cargo run -p formo-cli -- diagnose --input main.fm --json-pretty`.
2. baca field `stage` dan `errorMeta`.
3. jalankan `cargo run -p formo-cli -- doctor --input main.fm --json-schema`.
4. jalankan `cargo run -p formo-cli -- fmt --input main.fm --check`.

## Hari 6: Build Runtime (Web + Desktop)

Target:

- paham artifact web dan desktop.

Latihan:

1. build web (`--target web`).
2. build desktop (`--target desktop`).
3. jalankan `native-app` hasil desktop.
4. cek parity warning di output desktop.

Bonus:

```bash
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --release-exe
```

## Hari 7: Production Workflow

Target:

- siap workflow tim + CI + AI collaboration.

Latihan:

1. susun pipeline:
   - `cargo run -p formo-cli -- fmt --input main.fm --check`
   - `cargo run -p formo-cli -- check --input main.fm --json`
   - `cargo run -p formo-cli -- diagnose --input main.fm --json`
   - `cargo run -p formo-cli -- build --target multi --input main.fm --out dist --prod`
   - `cargo run -p formo-cli -- bench --input main.fm --iterations 20 --warmup 3 --nodes 1000 --out dist-ci/bench/benchmark.json`
2. buat checklist rilis.
3. praktikkan prompt AI untuk patch kecil + verifikasi.

## Checklist Lulus Learning Path

1. Bisa menulis komponen typed yang valid.
2. Bisa mengelola style token tanpa error.
3. Bisa debugging berdasarkan stage error.
4. Bisa build web + desktop.
5. Bisa compile desktop release executable.
6. Bisa menjalankan quality gate sebelum merge.

