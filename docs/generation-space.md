# Generation Space v0.1

ORIGIN v0.1 uses a deterministic, finite candidate space rather than retry-based random generation.

## Model

Each candidate contains three fixed-width consonant-vowel syllables:

```text
CV + CV + CV
```

The initial alphabet contains:

- 20 onset consonants
- 5 vowels
- 100 possible syllables

The total search space is therefore:

```text
100 × 100 × 100 = 1,000,000 candidates
```

Every candidate is six lowercase ASCII characters long.

## Deterministic traversal

A seed selects:

1. a starting index in the candidate space;
2. a traversal step that is coprime with 1,000,000.

Because the step is not divisible by 2 or 5, traversal cannot repeat before visiting the complete space. This gives ORIGIN the following guarantees:

- identical seed and count produce identical output;
- different seeds explore the same space in different orders;
- every requested candidate is unique up to the one-million limit;
- generation requires no hash set and no retry loop.

## Current limitations

The v0.1 model proves scale and determinism, not final linguistic quality.

Its fixed CV structure intentionally postpones:

- variable-length syllables;
- phonotactic language profiles;
- forbidden-word dictionaries;
- cross-language negative-meaning checks;
- edit-distance and phonetic similarity checks;
- evolutionary mutation and crossover.

These capabilities will be layered on top of the deterministic core in later releases.

## Performance direction

The present public API returns a sorted `Vec<Candidate>`. This is appropriate for early validation and ranked output, but million-scale export will later use an iterator or streaming writer to avoid retaining every candidate in memory.
