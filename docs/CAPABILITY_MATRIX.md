<!-- =============================================================================
 File:           docs/CAPABILITY_MATRIX.md
 Project:        Origin
 Author:         USDTG GROUP TECHNOLOGY LLC
 Developer:      Irfan Gedik
 Created Date:   2026-08-06
 Version:        0.1.0

 Description:
   Defines a governed component of the Origin v0.1 repository.

 License:
   Origin License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Origin v1 Capability Matrix

| Capability | v1 behavior | Evidence/output | Boundary |
| --- | --- | --- | --- |
| Candidate design | Up to 10,000 deterministic candidates from invented, ancient-inspired, hybrid, and meaning-free strategies. | Name, strategy, inspiration, quality scores. | Raw candidates are exploration, not recommendations. |
| Linguistic quality | Scores phonotactics, spelling, rhythm, memorability, and typographic balance. | Explainable `BrandReport`. | It cannot prove cultural, linguistic, or commercial suitability. |
| Semantic roots | Curated, attributed roots from the included catalog. | Root ID, source title, confidence, gloss. | The catalog is intentionally small and not a complete historical dictionary. |
| Similarity and trademark risk | Deterministic comparison against supplied references. | Risk factors and recommendation. | It is not a jurisdictional trademark search or legal opinion. |
| Availability | GitHub, crates.io, npm, PyPI, company/web exact-name sources, and eight domain TLDs. | Per-target status, source, timestamp, and detail. | Public-source outages remain `Unknown`; no result is inferred. |
| Finalist selection | Screens a bounded top-ranked pool and removes `Taken` candidates. | Final score, recommendation state, full JSON audit pool. | `Clear` means the requested sources reported available, not legal ownership. |

## Recommendation states

- **Clear**: every requested source returned `Available`.
- **Provisional**: no conflict was found but at least one source returned `Unknown`.
- **Reject**: one or more sources returned `Taken`; it is excluded from finalists.
