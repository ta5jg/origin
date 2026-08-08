# ORIGIN v1.0 — Product Specification

Version: 1.0.0  
Status: Frozen Specification

---

## 1. Vision

ORIGIN is a deterministic, explainable and evidence-driven brand discovery engine.

Its purpose is **not** to generate random words.

Its purpose is to design strong, memorable and commercially usable identities while minimizing legal and branding risks.

Every generated candidate must be explainable.

Every decision must be reproducible.

Every external conclusion must be supported by evidence.

---

## 2. Design Principles

### Deterministic

The same input and seed must always produce identical output.

### Explainable

Every score must include the reason behind it.

### Evidence Driven

External conclusions must always contain supporting evidence.

### Offline First

The complete generation engine must work without Internet access.

### Online Enhanced

Availability and clearance modules enrich results whenever Internet access is available.

---

## 3. Architecture

```
Origin CLI

        │

Origin Engine

        │

├── Semantic Engine
├── Morphology Engine
├── Merge Engine
├── Collapse Engine
├── Mutation Engine
├── Beam Search
├── Brand Scoring
├── Similarity Analysis
├── Trademark Screening
├── Availability Engine
├── Explainability Engine
└── Report Generator
```

---

## 4. Semantic Engine

The Semantic Engine constructs candidates from curated linguistic roots.

Supported root groups include:

- Proto-Indo-European
- Sumerian
- Akkadian
- Old Turkic
- Latin
- Ancient Greek
- Sanskrit
- Persian
- Arabic
- Germanic
- Norse
- Scientific terminology
- Technology terminology
- Modern conceptual roots

Every root contains:

- Identifier
- Language
- Meaning
- Categories
- Confidence
- Source
- Transliteration
- Metadata

---

## 5. Morphology Engine

Responsible for:

- Unicode normalization
- Transliteration
- Character folding
- Merge
- Collapse
- Mutation

Must remain deterministic.

---

## 6. Brand Composer

Transforms semantic roots into candidate brands.

Possible operations:

- Merge
- Collapse
- Mutation
- Beam optimization
- Ranking

---

## 7. Brand Scoring

Every candidate receives independent scores.

Example:

- Pronounceability
- Memorability
- Rhythm
- Visual Balance
- Semantic Strength
- Novelty
- Transition Quality
- Repetition
- Brandability

Overall score is derived from these components.

---

## 8. Similarity Engine

Measures similarity using multiple independent metrics.

Including:

- Damerau-Levenshtein
- N-Grams
- Prefix
- Suffix
- Phonetic similarity
- Visual similarity
- Keyboard similarity

Outputs:

- similarity score
- confidence
- explanation

---

## 9. Trademark Screening

Offline heuristic analysis.

Produces:

- Low Risk
- Medium Risk
- High Risk
- Critical

Never claims legal certainty.

Always recommends professional legal review where appropriate.

---

## 10. Availability Engine

The Availability Engine operates in two modes.

### Offline

Provides only local analysis.

### Online

Performs evidence-backed checks against external services.

Including:

#### Domains

- .com
- .net
- .org
- .io
- .ai
- .app
- .dev
- .co

#### Git Hosting

- GitHub organizations
- GitHub repositories

#### Package Registries

- crates.io
- npm
- PyPI
- NuGet
- Go packages

#### Company Presence

Checks for obvious company-name conflicts.

#### Web Presence

Checks whether a candidate is already strongly associated with another product, company or identity.

---

## 11. Evidence System

Every online conclusion must contain evidence.

Example:

```
Source:
WHOIS

Checked:
2026-08-08T20:15Z

Result:
Available
```

Example:

```
GitHub

Repository:
Not Found

Checked:
2026-08-08T20:17Z
```

No unsupported claims are allowed.

---

## 12. Explainability

Every candidate includes:

- strengths
- weaknesses
- scoring explanation
- similarity explanation
- trademark explanation
- availability explanation

No opaque scores.

---

## 13. Output Report

Example:

```
Candidate

Qarvan

Overall

96.4

Pronounceability

98

Brandability

97

Semantic Strength

95

Similarity Risk

Low

Trademark Risk

Low

Domains

.com       Registered
.ai        Available
.dev       Available

GitHub

Organization Available

Recommendation

★★★★★ FINALIST
```

---

## 14. CLI

Primary commands:

```
origin generate
origin check
origin compare
origin improve
origin optimize
origin trademark
```

Availability commands:

```
origin availability
```

`origin availability <name> --all` checks public registries and the standard
domain TLD set. `--offline` produces explicit `Unknown` fixture results without
network traffic, and `--json <path>` writes the evidence report.

Semantic commands:

```
origin roots
origin compose <left-root-id> <right-root-id>
```

---

## 15. Non-Goals for Version 1.0

The following are explicitly postponed:

- GUI Studio
- AI Agent orchestration
- Cloud synchronization
- Team collaboration
- Marketplace
- SaaS platform
- Automatic logo generation
- Automatic slogan generation

These belong to Version 2.x.

---

## 16. Definition of Done

Origin v1.0 is complete when:

- All tests pass.
- Clippy reports zero warnings.
- Public API is stable.
- Documentation is complete.
- CLI commands are documented.
- Explainability is available.
- Availability checks produce evidence-backed reports.
- A tagged v1.0.0 release is published.

No additional features may be added before Version 1.0 is released unless they are required to satisfy these completion criteria.
