# formo-parser Fuzzing

Fuzz target untuk hardening parser terhadap input acak.

## Menjalankan

Dari folder `crates/formo-parser`:

```bash
cargo fuzz run parser_parse
```

Smoke run singkat:

```bash
cargo fuzz run parser_parse -- -max_total_time=20
```

Corpus seed ada di `fuzz/corpus/parser_parse/`.
