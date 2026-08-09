# Origin 1.0

> **A deterministic, explainable, and production-oriented brand name discovery engine written in Rust.**

Origin is a high-performance Rust library and CLI for discovering high-quality brand names through deterministic linguistic processing rather than random generation.

Instead of relying on probabilistic language models or black-box AI, Origin combines morphology, phonotactics, similarity analysis, trademark-aware screening, and explainable scoring into a fully reproducible pipeline.

---

## Why Origin?

Choosing a brand name is one of the most important decisions in a product's lifecycle.

Most existing generators produce thousands of random suggestions without explaining **why** a name is good or whether it is distinguishable from existing brands.

Origin was designed with different principles:

- Deterministic results
- Explainable decisions
- Linguistic correctness
- Production-ready architecture
- Reproducible pipelines
- Test-driven development

The goal is not to generate *more* names.

The goal is to discover **better** names.

---

## Features

### Deterministic Generation

The same input always produces the same output.

No hidden randomness.

No AI hallucinations.

Fully reproducible results.

---

### Unicode-Aware Normalization

Origin normalizes roots from multiple writing systems into canonical ASCII forms while preserving linguistic intent.

Supported transformations include:

- Case folding
- Diacritic removal
- Historical transliterations
- Turkish characters
- Akkadian transliterations
- Unicode combining marks
- Ligature expansion

---

### Morphological Processing

Origin contains a dedicated morphology engine capable of:

- Root normalization
- Intelligent root merging
- Boundary-overlap detection
- Duplicate syllable removal
- Configurable collapse policies
- Provenance tracking

Example:

'''
velora
oralis

↓

veloralis
'''

instead of

'''
veloraoralis
'''

---

### Explainable Transformations

Every transformation can expose its provenance.

Example:

'''
left:normalize:Š>s
right:normalize:Ō>o
merge:collapse:boundary-overlap:or
'''

Every normalization step is deterministic and inspectable.

### Semantic Roots and Composition

The initial source-backed semantic catalog is intentionally small and reviewable.
List its roots, then compose any two identifiers into a candidate whose root meaning,
morphology provenance, and scoring rationale are shown together.

```bash
origin roots
origin compose latin-lux latin-via
```

---

### Candidate Analysis

Each candidate can be evaluated through multiple independent components, including:

- Pronounceability
- Similarity
- Morphological quality
- Brand quality
- Repetition analysis
- Trademark screening
- Portfolio diversity

---

### Similarity Analysis

Origin combines several techniques to reduce collision risk:

- Damerau-Levenshtein distance
- Prefix similarity
- Suffix similarity
- N-gram comparison
- Phonetic folding
- Unicode confusable detection
- Visual confusable detection

---

### Trademark Screening

Origin performs deterministic trademark-oriented risk estimation.

The engine distinguishes between:

- Critical
- High
- Medium
- Low
- Provisionally clear

screening outcomes while remaining explainable.

### Availability Screening

Origin does not recommend a name because one development example was checked.
The product pipeline is deliberately two-stage:

1. It designs and locally ranks a large raw candidate set (up to 10,000).
2. It fully screens every candidate in a bounded finalist pool against all
   standard targets, then ranks only those evidence-backed reports.

The raw set is internal exploration, not a recommendation. A candidate becomes
recommendable only after its full report exists. The standard report includes
GitHub, crates.io, npm, PyPI, exact-name company-register search, exact-name
public-web search, and `.com`, `.net`, `.org`, `.io`, `.ai`, `.app`, `.dev`, and
`.co` domain checks. Each online result contains its source and a
Unix-millisecond evidence timestamp.

`Available` contributes positive evidence; `Taken` rejects the candidate; and
an unavailable source remains `Unknown`, never `Available`. The final score is
65% design quality and 35% availability-evidence coverage. A finalist is:

- `Clear` only when every requested source returned `Available`.
- `Provisional` when no conflict was found but one or more sources are
  `Unknown`.
- omitted from recommendations when any source is `Taken`.

The default finalist pool is three fully screened candidates per requested
finalist; use `--screen-limit` to set it explicitly. This is a practical public
API budget, while retaining the invariant that every displayed finalist received
all checks. JSON output also retains reports for every screened candidate,
including rejected ones, so the selection can be audited.

```bash
origin generate --meaning "future civilization" --industry ai --count 10000 --finalists 10 --screen-limit 50
origin generate --roots latin-via,old-turkic-kut --finalists 5 --format json
origin availability candidate-name --all --json candidate-name-availability.json
```

`qarvan` appears in tests only as a fixed fixture name; it is not a product
recommendation or a special-case availability check.

---

### Beam Search

Origin supports deterministic beam-search based candidate exploration.

The search engine prioritizes:

- linguistic quality
- explainability
- reproducibility
- stable ranking

---

## Architecture

'''
                Raw Roots
                    │
                    ▼
          Unicode Normalization
                    │
                    ▼
           Morphology Merge
                    │
                    ▼
          Boundary Collapse
                    │
                    ▼
             Mutation Engine
                    │
                    ▼
          Candidate Evaluation
                    │
                    ▼
          Similarity Analysis
                    │
                    ▼
         Trademark Screening
                    │
                    ▼
          Portfolio Ranking
                    │
                    ▼
      Full Availability Evidence
                    │
                    ▼
       Scored Finalist Recommendations
'''

Every stage is deterministic.

Every stage is independently testable.

---

## Project Structure

'''
origin/
├── crates/
│   ├── origin-core
│   └── origin-cli
├── tests/
├── examples/
├── docs/
└── README.md
'''

---

## Design Principles

Origin follows several engineering principles:

- Deterministic by default
- Explainability over opacity
- Zero hidden state
- Small composable modules
- Test-driven implementation
- Stable APIs
- Explicit error handling

---

## Example

```bash
origin generate --count 20
```

```text
Velora
Avenor
Korvex
Calith
Sorevia
Nirel
```

---

## Quality

Current project status:

- Deterministic generation
- Deterministic morphology
- Unicode-aware normalization
- Boundary-overlap engine
- Explainable provenance
- Merge engine
- Collapse engine
- Mutation engine
- Similarity analysis
- Trademark screening
- Beam search
- Portfolio generation
- Comprehensive unit tests
- Zero Clippy warnings
- Fully formatted (`cargo fmt`)
- Release-gated v1.0 candidate and finalist workflow

---

## Testing

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Origin is developed under a strict quality policy:

- formatting must pass
- Clippy must report zero warnings
- every change must be covered by tests
- deterministic behavior is required

---

## Roadmap

### Completed

- Unicode normalization
- Morphology engine
- Merge engine
- Collapse engine
- Mutation engine
- Similarity analysis
- Trademark screening
- Beam search
- Portfolio generation
- Explainable provenance

### Planned

- Advanced semantic reasoning
- Strategy-based evaluation pipeline
- Plugin architecture
- International language packs
- Additional trademark integrations
- Performance benchmarking
- Public API stabilization

---

## Philosophy

Origin is not intended to be "just another name generator."

It is designed to become an explainable brand discovery engine suitable for professional product development.

Every generated candidate should be:

- pronounceable
- memorable
- linguistically consistent
- explainable
- reproducible
- suitable for further legal and commercial review

---

## Contributing

Contributions are welcome.

Please ensure that every contribution:

- passes `cargo fmt`
- passes `cargo clippy -D warnings`
- passes all tests
- preserves deterministic behavior

---

## License

This project is released under the MIT License.

See the `LICENSE` file for details.
