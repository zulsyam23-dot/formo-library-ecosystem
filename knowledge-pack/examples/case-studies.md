# Case Studies Formo (End-to-End)

## AI Quick Context
- doc_path: knowledge-pack/examples/case-studies.md
- intent: dokumentasi operasional Formo dengan format deterministik untuk AI agent.
- command_prefix: cargo run -p formo-cli --
- default_input: main.fm (kecuali disebut lain).


## Kasus 1: Halaman Teks Sederhana

### Tujuan

- render teks sederhana dengan style tunggal.

### `main.fm`

```fm
import "styles/base.fs" as Base;

component App() {
  <Page>
    <Text value="Halo Formo" style=BodyText/>
  </Page>
}
```

### `styles/base.fs`

```fs
style BodyText {
  color: #112233;
  font-size: 16;
}
```

### Verifikasi

```bash
cargo run -p formo-cli -- check --input main.fm
cargo run -p formo-cli -- build --target web --input main.fm --out dist-web
```

## Kasus 2: Form Input + Action

### Tujuan

- input state string + submit action.

### `main.fm`

```fm
component App(
  keyword: state<string>,
  onKeywordChange: action<string>,
  submit: action<void>
) {
  <Column>
    <Input value=keyword onChange=onKeywordChange placeholder="Cari..."/>
    <Button label="Cari" onPress=submit/>
  </Column>
}
```

### Verifikasi

```bash
cargo run -p formo-cli -- check --input main.fm
cargo run -p formo-cli -- diagnose --input main.fm --json-pretty
```

## Kasus 3: List + Conditional

### Tujuan

- menampilkan list object dengan badge aktif.

### `main.fm`

```fm
component App() {
  <For each=[{name: "A", active: true}, {name: "B", active: false}] as=item>
    <Row>
      <Text value=item.name/>
      <If when=item.active>
        <Text value="active"/>
      </If>
    </Row>
  </For>
}
```

### Verifikasi

```bash
cargo run -p formo-cli -- check --input main.fm
```

## Kasus 4: Wrapper Reusable dengan Slot

### Tujuan

- membuat container reusable.

### `main.fm`

```fm
component CardFrame(title: string) {
  <Column>
    <Text value=title/>
    <Slot/>
  </Column>
}

component App() {
  <CardFrame title="Ringkasan">
    <Text value="Isi child dari parent"/>
  </CardFrame>
}
```

### Verifikasi

```bash
cargo run -p formo-cli -- check --input main.fm
```

## Kasus 5: Build Desktop Native sampai Exe

### Tujuan

- build desktop dan langsung compile binary release.

### Command

```bash
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist-desktop --release-exe
```

### Hasil

- artifact native di `dist-desktop/*`
- binary release di `dist-desktop/native-app/target/release`

## Kasus 6: Debug Error Style

### Contoh error

`styles/base.fs`:

```fs
style BodyText {
  not-supported: #0A84FF;
}
```

### Gejala

- `E1301` unknown style property.

### Perbaikan

- ganti properti ke allowlist valid (misal `color`).

## Kasus 7: Debug Error Custom Component

### Contoh error

```fm
component Header(title: string) {
  <Text value=title/>
}

component App() {
  <Header/>
}
```

### Gejala

- `E2301` missing required prop.

### Perbaikan

```fm
component App() {
  <Header title="Dashboard"/>
}
```

