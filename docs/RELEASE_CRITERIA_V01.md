<!-- =============================================================================
 File:           docs/RELEASE_CRITERIA_V01.md
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

# Origin v1.0 Release Criteria

This document is the release gate for the public `1.0.0` tag. A release is not
approved by a passing build alone: the CLI, evidence contract, and public
documentation must agree.

## Required gates

| Gate | Required evidence |
| --- | --- |
| Source integrity | Clean worktree and reviewed release diff. |
| Formatting | `cargo fmt --all -- --check` passes. |
| Static analysis | `cargo clippy --workspace --all-targets -- -D warnings` passes. |
| Tests | `cargo test --workspace --all-targets` passes. |
| API | `cargo doc --workspace --no-deps` completes and public types retain documentation. |
| CLI | `origin generate --help` and `origin availability --help` expose the documented options. |
| Candidate contract | A finalist run records every source result for every screened candidate. |
| Evidence safety | `Unknown` is never presented as `Available`; candidates with `Taken` evidence are not recommended. |
| Documentation | README, v1 specification, capability matrix, and provenance policy match the released behavior. |
| Publication | Workspace version is `1.0.0`, the release commit is tagged `v1.0.0`, and the tag is pushed. |

## Release commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo doc --workspace --no-deps
cargo run -p origin-cli -- generate --help
cargo run -p origin-cli -- availability --help
```

## Scope boundary

The v1 release produces explainable candidate design and evidence-backed public
availability screening. It is not a legal clearance opinion. Trademark review,
jurisdictional rights, and use-in-commerce analysis still require qualified
professional review.
