# Referensi Bahasa Formo (`.fm`)

## AI Quick Context
- doc_path: knowledge-pack/docs/01-formo-language-reference.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


Dokumen ini menjelaskan syntax dan aturan semantik bahasa Formo berdasarkan implementasi saat ini.

## 1) Bentuk Dasar File

Urutan umum:

1. `import`
2. `component`

Contoh:

```fm
import "views/header.fm" as Header;
import "styles/base.fs" as Base;

component App(title: string) {
  <Page>
    <Header title=title/>
  </Page>
}
```

Catatan:

- `import` wajib diakhiri `;`.
- alias import harus identifier valid.

## 2) Deklarasi Component

Format:

```fm
component Name(param: type, optionalParam?: type) {
  <RootNode/>
}
```

Aturan penting:

- Nama component harus unik.
- Setiap component harus punya tepat satu root node.
- Parameter boleh bertipe eksplisit atau tidak diberi tipe.
- Parameter optional ditulis `param?: type`.

## 3) Tipe Data Parameter

Tipe yang digunakan pipeline saat ini:

- `string`
- `bool`
- `int`
- `float`
- `len`
- `color`
- `object`
- `list<T>` (termasuk nested list)
- `state<string>`
- `state<bool>`
- `state<list<T>>`
- `action<void>` atau `action`
- `action<string>`
- `action<bool>`

Contoh:

```fm
component SearchBox(
  keyword: state<string>,
  onChangeKeyword: action<string>,
  submit: action<void>,
  enabled: state<bool>,
  tags: list<string>
) {
  <Column>
    <Input value=keyword onChange=onChangeKeyword placeholder="Cari..."/>
    <Button label="Submit" onPress=submit disabled=enabled/>
  </Column>
}
```

## 4) Built-in Node

Built-in node yang dikenali:

- `Window`
- `Page`
- `Row`
- `Column`
- `Stack`
- `Card`
- `Text`
- `Image`
- `Button`
- `Input`
- `Checkbox`
- `Switch`
- `Scroll`
- `Spacer`
- `Modal`
- `If`
- `For`
- `Slot`

## 5) Built-in Props (Ringkas)

### `Window`

- required: `title: string`
- optional: `width`, `height`, `minWidth`, `minHeight` (`len`), `resizable` (`bool`)

### `Page`

- optional: `id` (`string`), `padding` (`len`), `scroll` (`string or identifier`)

### `Row`

- optional: `gap` (`len`), `align`, `justify` (`string or identifier`), `wrap` (`bool`)

### `Column`

- optional: `gap` (`len`), `align`, `justify` (`string or identifier`)

### `Stack`

- optional: `align` (`string or identifier`)

### `Card`

- optional: `variant` (`string or identifier`), `padding` (`len`), `radius` (`len`)

### `Text`

- required: `value: string`
- optional: `variant`, `align` (`string or identifier`), `color` (`color`), `maxLines` (`int`), `ellipsis` (`bool`)

### `Image`

- required: `src` (`string or identifier`)
- optional: `alt` (`string`), `fit` (`string or identifier`), `width` (`len`), `height` (`len`)

### `Button`

- required: `label: string`, `onPress: action<void>`
- optional: `variant`, `leadingIcon` (`string or identifier`), `disabled` (`bool`)

### `Input`

- required: `value: state<string>`, `onChange: action<string>`
- optional: `placeholder` (`string`), `inputType` (`string or identifier`), `disabled` (`bool`)

### `Checkbox`

- required: `checked: state<bool>`, `onChange: action<bool>`
- optional: `label` (`string`), `disabled` (`bool`)

### `Switch`

- required: `checked: state<bool>`, `onChange: action<bool>`
- optional: `disabled` (`bool`)

### `Scroll`

- optional: `axis` (`string or identifier`)

### `Spacer`

- required: `size: len`

### `Modal`

- required: `open: state<bool>`, `onClose: action<void>`

### `If`

- required: `when` (`bool` atau `state<bool>`)

### `For`

- required: `each` (sumber list), `as` (alias item)

### `Slot`

- tidak menerima attribute.
- dipakai untuk menerima inline children dari pemanggil component.

## 6) Aturan Children

Node yang boleh punya children:

- `Window`, `Page`, `Row`, `Column`, `Stack`, `Card`, `Scroll`, `Modal`, `If`, `For`

Node lain dianggap leaf (tidak boleh children).

## 7) Style Attribute

Format:

```fm
<Text value="Halo" style=BodyText/>
<Text value="Halo" style="BodyText, BodyTextBold"/>
```

Aturan:

- style tidak boleh kosong.
- style harus string-compatible.
- `Slot` tidak boleh memakai `style`.

## 8) `For` dan Scope Lokal

Contoh:

```fm
component App(items: list<string>) {
  <For each=items as=item>
    <Text value=item/>
    <Text value="index" maxLines=itemIndex/>
  </For>
}
```

Kemampuan:

- alias `item` dapat dipakai di subtree `For`.
- `itemIndex` tersedia otomatis.
- list literal object mendukung akses field (`item.name`) dan index (`item.tags.0`).

## 9) Literal yang Didukung

- string: `"teks"`
- bool: `true`, `false`
- int: `1`
- float: `2.5`
- list: `["A", "B"]`, `[1, 2, 3]`, `[{name: "A"}]`
- object literal untuk data list/object.

## 10) Error Semantik Umum

Contoh kategori code:

- `E2001` / `E2002`: root component invalid.
- `E2101` / `E2102`: nama node invalid / node unknown.
- `E2250..E2253`: masalah prop built-in.
- `E2301..E2304`: masalah pemanggilan custom component.
- `E2221..E2223`: masalah style attribute.

Lihat dokumen troubleshooting untuk matriks perbaikan.

## 11) Contoh Pola Komponen Reusable + Slot

```fm
component HeaderFrame(title: string) {
  <Column>
    <Text value=title/>
    <Slot/>
  </Column>
}

component App() {
  <HeaderFrame title="Dashboard">
    <Text value="Konten tambahan dari pemanggil"/>
  </HeaderFrame>
}
```

Jika `HeaderFrame` tidak memiliki `<Slot/>`, inline children akan ditolak.

## 12) Best Practice Bahasa Formo

1. Gunakan nama component PascalCase yang jelas.
2. Definisikan tipe param untuk komponen publik.
3. Pisahkan komponen per fitur/per halaman.
4. Gunakan `For` untuk list, `If` untuk conditional, jangan branch manual di luar DSL.
5. Gunakan style ref konsisten (tanpa hardcode visual di banyak node).

