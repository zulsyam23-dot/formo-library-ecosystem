# Tooling CLI Examples

Contoh command penting:

```bash
cargo run -p formo-cli -- check main.fm
cargo run -p formo-cli -- logic --input language-core/examples/app-controller.fl --json-pretty --rt-manifest-out dist-ci/runtime/logic.manifest.json
cargo run -p formo-cli -- diagnose --input main.fm --json
cargo run -p formo-cli -- build --target web --input main.fm --out dist
cargo run -p formo-cli -- build --target web --input main.fm --out dist --strict-parity
cargo run -p formo-cli -- build --target web --input main.fm --out dist --strict-engine
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist --strict-parity
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist --strict-engine
cargo run -p formo-cli -- build --target desktop --input main.fm --out dist --release-exe
cargo run -p formo-cli -- bench --input main.fm --iterations 12 --warmup 3 --nodes 1000 --out dist-ci/bench/benchmark.json --json-pretty
cargo run -p formo-cli --no-default-features --features backend-web -- build --target web --input main.fm --out dist-web-only
cargo run -p formo-cli --no-default-features --features backend-desktop -- build --target desktop --input main.fm --out dist-desktop-only
cargo run -p formo-cli --no-default-features --features backend-web,backend-desktop -- build --target web --input main.fm --out dist-web --strict-parity
cargo run -p formo-cli --no-default-features -- check main.fm
```

Catatan:

- `--strict-parity` pada target `web` memerlukan feature `backend-desktop` untuk audit parity desktop.
- setiap build menulis `engine.bridge.json`; gunakan `--strict-engine` untuk memaksa standar bridge `FM/FS/FL` tanpa warning.
