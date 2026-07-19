# Phonotactic Engine v0.1

ORIGIN's first linguistic layer evaluates whether a candidate is structurally easy to pronounce before expensive collision, domain or trademark checks are performed.

## Current rules

The deterministic v0.1 analyzer checks:

- lowercase ASCII normalization;
- supported length from 4 to 12 letters;
- adjacent duplicate letters;
- long vowel runs;
- long consonant runs;
- vowel balance;
- vowel diversity;
- repeated two-letter syllables.

The output contains a score from 0 to 100, an acceptance decision and human-readable warnings.

```bash
cargo run -p origin-cli -- check danoti
cargo run -p origin-cli -- check folele
cargo run -p origin-cli -- check folele --format json
```

## Scope

This model is intentionally language-neutral and conservative. It does not claim to predict pronunciation in every language. Future versions will add language profiles, weighted phoneme transitions, stress estimation and market-specific ambiguity checks.
