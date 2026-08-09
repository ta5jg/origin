<!-- =============================================================================
 File:           docs/DATA_PROVENANCE.md
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

# Origin Data Provenance Policy

## Local linguistic data

Historical roots carry an identifier, normalized form, gloss, confidence, and
source metadata. Origin transforms those roots into phonetic inspiration; it
does not present a generated name as an authentic historical word.

The shipped catalog is intentionally reviewed and small. Adding roots requires
an attributable source, a conservative confidence value, and a test that
validates the catalog entry.

## Live availability evidence

Each public lookup result records:

- target identifier;
- queried canonical candidate name;
- `Available`, `Taken`, or `Unknown` status;
- provider/source URL;
- lookup timestamp when the result came from a live provider; and
- a short provider detail.

Transport failures, authentication failures, malformed responses, and
unsupported provider responses produce `Unknown`. They must never be converted
to positive availability evidence.

## Interpretation boundary

Availability evidence is time-bound and source-specific. It is not proof of
exclusive rights, domain ownership, corporate availability in every country, or
trademark clearance. Final JSON reports preserve the full screened pool so a
recommendation remains auditable after the ranked table is produced.
