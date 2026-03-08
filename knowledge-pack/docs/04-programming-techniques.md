# Teknik Pemrograman Formo (Basic sampai Advanced)

## AI Quick Context
- doc_path: knowledge-pack/docs/04-programming-techniques.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Dokumen ini merangkum berbagai teknik implementasi agar codebase Formo rapi, scalable, dan stabil lintas target.

## 1) Teknik Modularisasi Fitur

Pisahkan per domain:

```text
app/
  main.fm
  views/
    auth/
      login.fm
      register.fm
    dashboard/
      page.fm
  styles/
    tokens.fs
    auth.fs
    dashboard.fs
```

Manfaat:

- komponen lebih kecil,
- style terorganisir,
- mudah dipetakan AI agent per folder.

## 2) Teknik Reusable Component

Contoh:

```fm
component SectionTitle(text: string) {
  <Text value=text style=SectionTitleText/>
}
```

Gunakan komponen ini berulang daripada copy-paste node `Text`.

## 3) Teknik Composition dengan Slot

Pattern wrapper:

```fm
component CardFrame(title: string) {
  <Column style=CardFrame>
    <Text value=title style=CardFrame:title/>
    <Slot/>
  </Column>
}
```

Penggunaan:

```fm
<CardFrame title="Ringkasan">
  <Text value="Isi dinamis"/>
</CardFrame>
```

Catatan:

- jangan beri atribut pada `<Slot/>`.
- jika komponen wrapper tanpa `<Slot/>`, inline child pada pemanggil akan error.

## 4) Teknik Data Flow yang Aman

Prinsip:

- UI input gunakan `state<string>` + `action<string>`.
- UI toggle gunakan `state<bool>` + `action<bool>`.
- trigger tombol gunakan `action<void>`.

Contoh:

```fm
component LoginForm(
  username: state<string>,
  onUsernameChange: action<string>,
  rememberMe: state<bool>,
  onRememberChange: action<bool>,
  submit: action<void>
) {
  <Column>
    <Input value=username onChange=onUsernameChange placeholder="Username"/>
    <Checkbox checked=rememberMe onChange=onRememberChange label="Remember me"/>
    <Button label="Login" onPress=submit/>
  </Column>
}
```

## 5) Teknik Render Bersyarat

Pattern:

```fm
<If when=isLoading>
  <Text value="Loading..."/>
</If>
```

Gunakan `If` untuk branch UI sederhana, bukan duplicasi komponen.

## 6) Teknik Render List

Pattern basic:

```fm
<For each=items as=item>
  <Text value=item/>
</For>
```

Pattern object:

```fm
<For each=[{name: "A"}, {name: "B"}] as=item>
  <Text value=item.name/>
</For>
```

Pattern index:

```fm
<For each=items as=item>
  <Text value="row" maxLines=itemIndex/>
</For>
```

## 7) Teknik Design System

Pisahkan token global:

```fs
token {
  color.brand.primary = #0A84FF;
  color.text.primary = #111827;
  space.md = 12dp;
  radius.md = 10dp;
}
```

Lalu style semantik:

```fs
style ScreenContainer { padding: token(space.md); }
style PrimaryButton { background: token(color.brand.primary); }
```

## 8) Teknik Parity-First (Web + Desktop)

Agar output konsisten:

1. Gunakan built-in node yang sudah didukung desktop.
2. Prioritaskan style property core.
3. Setelah build desktop, selalu cek:
   - warning parity di output CLI,
   - `app.native.json.diagnostics`.

## 9) Teknik Pemisahan Presentasi vs Aksi

Pattern:

- komponen presentasi: hanya terima value/state/action.
- komponen container: menyuplai data dan aksi ke presentasi.

Manfaat:

- test lebih mudah,
- migrasi host runtime lebih aman.

## 10) Teknik Refactor Aman

Urutan:

1. Jalankan `cargo run -p formo-cli -- fmt --input main.fm --check`.
2. Jalankan `cargo run -p formo-cli -- check --input main.fm`.
3. Jalankan `cargo run -p formo-cli -- diagnose --input main.fm --json`.
4. Jalankan `cargo run -p formo-cli -- build --target multi --input main.fm --out dist`.
5. Untuk desktop, cek parity warning.

## 11) Teknik Integrasi AI Pair Programmer

Gunakan pola kerja:

1. beri AI konteks file aktif + target fitur.
2. minta AI menulis patch kecil per langkah.
3. validasi setiap langkah dengan `cargo run -p formo-cli -- check --input main.fm`.
4. finalisasi dengan `cargo run -p formo-cli -- build --target multi --input main.fm --out dist` dan `cargo run -p formo-cli -- diagnose --input main.fm --json`.

## 12) Anti-Pattern yang Harus Dihindari

1. Satu file `.fm` berisi terlalu banyak komponen lintas domain.
2. Hardcode angka/warna berulang tanpa token.
3. Memaksa prop type (misal string untuk prop int).
4. Menaruh logic slot tanpa benar-benar menyediakan `<Slot/>`.
5. Mengabaikan warning parity desktop.

