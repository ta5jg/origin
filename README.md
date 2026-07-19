# ORIGIN

> Engineering the next legendary technology brand.

ORIGIN is an experimental **brand discovery engine** written in Rust. It explores large naming spaces, filters weak candidates, scores phonetic and structural qualities, and prepares promising names for external validation.

ORIGIN is a working codename. One of the project's first long-term goals is to discover and validate its own permanent product name.

## Why ORIGIN?

Finding a strong technology brand should not depend on random inspiration alone. ORIGIN treats naming as an engineering problem built from generation, filtering, scoring, similarity analysis, and evidence-based validation.

The project follows one uncompromising principle:

> A candidate is not considered unique until it has been checked against relevant public use, domains, repositories, companies, and trademark databases.

No automated result is a legal opinion or a guarantee of trademark availability.

## Current milestone

**Sprint 0 — Genesis**

The first milestone establishes a clean Rust workspace and a working command-line path:

```bash
cargo run -p origin-cli -- generate --count 25
```

The initial generator is deliberately small and deterministic. It exists to validate the architecture before more advanced phonetic, evolutionary, similarity, domain, and trademark modules are added.

## Planned pipeline

```text
Candidate Generation
        ↓
Structural Filtering
        ↓
Phonetic Scoring
        ↓
Similarity Analysis
        ↓
External Validation
        ↓
Human Review
```

## Workspace

```text
origin/
├── crates/
│   ├── origin-core/   # Domain logic, generation and scoring
│   └── origin-cli/    # Command-line interface
├── docs/              # Design documents
├── .github/workflows/ # Continuous integration
├── Cargo.toml
└── README.md
```

## Development

Requirements:

- Rust stable with Rust 2024 edition support
- Cargo

Run all checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Generate candidates:

```bash
cargo run -p origin-cli -- generate --count 100 --seed 42
```

Generate JSON:

```bash
cargo run -p origin-cli -- generate --count 25 --format json
```

## Engineering principles

- Uniqueness before attachment
- Evidence before claims
- Deterministic and testable core logic
- Clippy-clean and rustfmt-clean code
- No undocumented public API
- Human review remains part of every final naming decision
- External validation is a risk screen, not legal clearance

## Roadmap

See [ROADMAP.md](ROADMAP.md).

## License

Licensed under the MIT License. See [LICENSE](LICENSE).

---

The next generation of technology companies deserves better names.

**ORIGIN exists to help discover them.**
