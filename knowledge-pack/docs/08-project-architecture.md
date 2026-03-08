# Project Architecture Playbook (Formo)

Panduan arsitektur ini membantu menjaga codebase Formo tetap rapi saat skala project bertambah.

## 1) Prinsip Inti

1. Pisahkan domain UI, style, dan integration logic.
2. Gunakan komponen kecil dan composable.
3. Hindari file monolitik.
4. Selalu parity-aware untuk target desktop.

## 2) Blueprint Skala Kecil

```text
project/
  main.fm
  styles/
    base.fs
```

Cocok untuk:

- prototype,
- proof of concept,
- landing feature tunggal.

## 3) Blueprint Skala Menengah

```text
project/
  main.fm
  views/
    home.fm
    profile.fm
    settings.fm
  components/
    layout/
      app_shell.fm
    common/
      button_row.fm
      section_title.fm
  styles/
    tokens.fs
    common.fs
    home.fs
    profile.fs
    settings.fs
```

Cocok untuk:

- produk internal,
- dashboard tim,
- fitur multi halaman.

## 4) Blueprint Skala Besar

```text
project/
  main.fm
  features/
    auth/
      views/
      components/
      styles/
    billing/
      views/
      components/
      styles/
    analytics/
      views/
      components/
      styles/
  shared/
    components/
    styles/
    tokens/
```

Cocok untuk:

- product multi domain,
- tim besar dengan ownership per fitur.

## 5) Konvensi Penamaan

- Component: `PascalCase` (`UserCard`, `BillingHeader`)
- Style ID: semantik (`ScreenContainer`, `PrimaryButton`)
- Token key: dot-path (`color.brand.primary`, `space.md`)
- File: `snake_case` atau `kebab-case` yang konsisten.

## 6) Layering yang Direkomendasikan

1. `shared/tokens`
2. `shared/styles`
3. `shared/components`
4. `feature/components`
5. `feature/views`
6. `main.fm` sebagai composition root

## 7) Composition Root Pattern

`main.fm` sebaiknya:

- import view entry,
- memilih komponen root,
- meminimalkan logic visual detail.

## 8) Pattern Integrasi Style

Urutan style:

1. token global
2. style global common
3. style per feature
4. override part style jika perlu

Tujuan:

- perubahan tema lebih aman,
- risiko duplikasi lebih rendah.

## 9) Pattern Custom Component API

Untuk komponen reusable:

1. deklarasikan prop wajib secara eksplisit.
2. gunakan tipe prop ketat (`string`, `int`, `state<string>`, dll).
3. gunakan `<Slot/>` jika komponen perlu konten dari parent.
4. hindari prop “serba guna” tanpa type.

## 10) Pattern Multi-Target Delivery

Agar web/desktop tetap sejalan:

1. build `multi` di CI.
2. treat parity warning desktop sebagai quality signal.
3. simpan daftar style/widget yang butuh fallback.

## 11) Ownership dan Review

Saran review policy:

1. perubahan `.fm` wajib `check` + `diagnose`.
2. perubahan `.fs` wajib cek error `E13xx`.
3. perubahan runtime-target wajib build target terkait.
4. perubahan lintas domain wajib update docs.

## 12) Anti-Pattern Arsitektur

1. semua komponen dalam satu file.
2. token campur dengan style per fitur tanpa struktur.
3. style hardcode tersebar tanpa semantik.
4. tidak ada pipeline verifikasi terstandar.
