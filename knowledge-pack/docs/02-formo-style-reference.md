# Referensi Bahasa Style Formo (`.fs`)

## AI Quick Context
- doc_path: knowledge-pack/docs/02-formo-style-reference.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Dokumen ini menjelaskan syntax style Formo, token system, tipe value, validasi, dan praktik terbaik.

## 1) Struktur Dasar

File style mendukung dua blok:

- `token { ... }`
- `style Selector { ... }`

Contoh:

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

## 2) Token Block

Format:

```fs
token {
  key.path = value;
}
```

Aturan token key:

- karakter valid: alfanumerik, `_`, `-`, `.`
- tidak boleh duplikat
- token yang didefinisikan tapi tidak dipakai akan error

Contoh fallback:

```fs
style Accent {
  color: token(color.brand, #2266AA);
}
```

Fallback bisa nested:

```fs
style Accent {
  color: token(color.brand, token(color.primary, #0A84FF));
}
```

## 3) Style Block dan Selector

Format root selector:

```fs
style Card {
  padding: 12dp;
}
```

Format part selector:

```fs
style Card:title {
  font-weight: 700;
}
```

Aturan:

- selector tidak boleh kosong
- `style ID` dibentuk dari `component` atau `component:part`
- deklarasi wajib format `key: value;`

## 4) Property Allowlist

Property style yang diizinkan:

- `align-items`
- `align-self`
- `background`
- `background-color`
- `border`
- `border-color`
- `border-radius`
- `border-style`
- `border-width`
- `bottom`
- `box-shadow`
- `color`
- `cursor`
- `display`
- `flex`
- `flex-basis`
- `flex-direction`
- `flex-grow`
- `flex-shrink`
- `flex-wrap`
- `font-family`
- `font-size`
- `font-style`
- `font-weight`
- `gap`
- `height`
- `inset`
- `justify-content`
- `left`
- `line-height`
- `margin`
- `margin-bottom`
- `margin-left`
- `margin-right`
- `margin-top`
- `max-height`
- `max-width`
- `min-height`
- `min-width`
- `opacity`
- `overflow`
- `padding`
- `padding-bottom`
- `padding-left`
- `padding-right`
- `padding-top`
- `position`
- `right`
- `text-align`
- `top`
- `width`
- `z-index`
- semua custom property yang diawali `--`

Di luar daftar ini akan gagal validasi.

## 5) Value Type yang Didukung

### Color

- Format hex: `#RRGGBB` atau `#RRGGBBAA`

### Length

- unit: `dp`, `px`, `%`, `vw`, `vh`, `rem`, `em`
- contoh: `12dp`, `8px`, `100%`

### Bool

- `true` / `false`

### Number

- `int` dan `float`

### String

- quoted string: `"Inter"`
- identifier raw juga diterima dan dipetakan sebagai string default jika bukan tipe literal lain.

### Token Reference

- `token(name)`
- `token(name, fallback)`

## 6) Error Style Umum

- `E1300`: tidak bisa baca style module.
- `E1301`: parse/validasi style gagal (syntax/property/value).
- `E1302`: duplicate token.
- `E1303`: duplicate style id lintas file.
- `E1304`: token didefinisikan tapi tidak dipakai.

## 7) Contoh Theme Scalable

```fs
token {
  color.text.primary = #111827;
  color.text.inverse = #FFFFFF;
  color.brand.primary = #0A84FF;
  space.xs = 4dp;
  space.sm = 8dp;
  space.md = 12dp;
  radius.md = 10dp;
}

style Screen {
  background: #F8FAFC;
  padding: token(space.md);
}

style Title {
  color: token(color.text.primary);
  font-size: 20;
  font-weight: 700;
  margin-bottom: token(space.sm);
}

style PrimaryButton {
  background: token(color.brand.primary);
  color: token(color.text.inverse);
  border-radius: token(radius.md);
  padding: token(space.sm);
}
```

## 8) Praktik Terbaik Style Formo

1. Gunakan token untuk nilai berulang.
2. Pisahkan token global dan style per fitur.
3. Hindari magic number tanpa token.
4. Pakai part selector (`Component:part`) untuk elemen turunan.
5. Bersihkan token yang tidak dipakai agar tidak memicu `E1304`.

## 9) Strategi Web + Desktop Parity

Untuk menjaga hasil visual mirip:

1. Prioritaskan property core (`color`, `background`, `padding`, `margin`, `font-*`, `gap`, `border*`).
2. Pantau warning parity desktop di `app.native.json.diagnostics`.
3. Jika ada style unsupported desktop, siapkan fallback yang tetap readable.

