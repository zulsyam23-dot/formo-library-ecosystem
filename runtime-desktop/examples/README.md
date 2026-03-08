# Examples

Contoh penggunaan library runtime-desktop.

Workflow cepat output desktop:

1. jalankan build desktop dari CLI Formo.
2. buka folder output.
3. jalankan scaffold GUI native:
   - `cd native-app`
   - `cargo run`

Build langsung ke executable release (sekali perintah dari CLI):

- `formo build --target desktop --input main.fm --out dist-desktop --release-exe`
- hasil binary ada di `dist-desktop/native-app/target/release` (`.exe` di Windows).
