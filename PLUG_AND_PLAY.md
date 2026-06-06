# PLUG_AND_PLAY — Tidelight

> Ternary tide-level prediction via cyclic temporal patterns

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ternary-tidelight = { git = "https://github.com/SuperInstance/ternary-tidelight" }
```

Use in your code:

```rust
use ternary_tidelight::TidePredictor;

let mut pred = TidePredictor::new(24);
pred.train(&historical_data);
let forecast = pred.predict(7);
```

## 📚 Available Documentation

| Document | Description |
|----------|-------------|
| `docs/FROM_BINARY.md` | Understanding ternary concepts as a binary programmer |
| `docs/MIGRATION.md` | Version migration guide |
| `docs/FUTURE-INTEGRATION.md` | Planned features and roadmap |

## 🔗 Integration

This crate is part of the [SuperInstance ternary fleet](https://github.com/SuperInstance). It uses the canonical `Ternary` type from `ternary-types` for cross-crate compatibility.

## 📄 License

MIT
