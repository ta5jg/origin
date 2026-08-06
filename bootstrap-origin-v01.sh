#!/usr/bin/env bash
#
# ==============================================================================
# File:           bootstrap-origin-v01.sh
# Project:        Origin
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Extends the existing Origin Rust workspace with the Origin v0.1 product
#   skeleton while preserving all existing implementation files by default.
#
# License:
#   Origin License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -Eeuo pipefail

ROOT_DIR="."
FORCE=0
DRY_RUN=0
CREATED_DATE="$(date '+%Y-%m-%d')"
VERSION="0.1.0"

usage() {
  cat <<'EOF'
Usage:
  ./bootstrap-origin-v01.sh [repository-root] [--dry-run] [--force]

Examples:
  ./bootstrap-origin-v01.sh
  ./bootstrap-origin-v01.sh ~/Projects/origin --dry-run
  ./bootstrap-origin-v01.sh ~/Projects/origin --force
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --force) FORCE=1 ;;
    --dry-run) DRY_RUN=1 ;;
    --help|-h) usage; exit 0 ;;
    -*) printf '[ERROR] Unknown option: %s\n' "$1" >&2; exit 1 ;;
    *) ROOT_DIR="${1%/}" ;;
  esac
  shift
done

CORE="$ROOT_DIR/crates/origin-core"
CLI="$ROOT_DIR/crates/origin-cli"

if [ ! -d "$CORE" ] || [ ! -d "$CLI" ]; then
  printf '[ERROR] Existing Origin workspace was not found.\n' >&2
  printf '[ERROR] Expected: %s and %s\n' "$CORE" "$CLI" >&2
  exit 1
fi

log() { printf '[%s] %s\n' "$1" "$2"; }

mkdir_safe() {
  path="$1"
  if [ "$DRY_RUN" -eq 1 ]; then
    log CREATE "directory $path"
  elif [ ! -d "$path" ]; then
    mkdir -p "$path"
    log CREATE "directory $path"
  fi
}

relative() {
  case "$1" in
    "$ROOT_DIR"/*) printf '%s\n' "${1#"$ROOT_DIR"/}" ;;
    *) printf '%s\n' "$1" ;;
  esac
}

description_for() {
  path="$1"
  case "$path" in
    */language/*) printf 'Defines language roots, provenance, normalization, or language profiles.' ;;
    */morphology/*) printf 'Defines deterministic root transformation and morphology behavior.' ;;
    */generator/*) printf 'Defines multi-mode brand-name generation behavior.' ;;
    */scoring/*) printf 'Defines explainable brand quality and risk scoring.' ;;
    */validation/*) printf 'Defines external collision screening and evidence contracts.' ;;
    */export/*) printf 'Defines deterministic campaign and finalist export behavior.' ;;
    */commands/*) printf 'Defines an Origin CLI command implementation boundary.' ;;
    */output/*) printf 'Defines stable human-readable and machine-readable CLI output.' ;;
    */tests/*) printf 'Defines automated verification for an Origin v0.1 capability.' ;;
    */benches/*|*/benchmarks/*) printf 'Defines a performance benchmark for an Origin capability.' ;;
    */data/*) printf 'Defines a versioned Origin dataset or dataset policy.' ;;
    */campaigns/*) printf 'Defines a reproducible Origin naming campaign.' ;;
    */fixtures/*) printf 'Defines a deterministic test fixture for Origin.' ;;
    */docs/*) printf 'Defines governed technical documentation for Origin.' ;;
    */scripts/*) printf 'Provides repository automation for Origin engineering workflows.' ;;
    */.github/*) printf 'Defines GitHub repository governance or automation for Origin.' ;;
    *) printf 'Defines a governed component of the Origin v0.1 repository.' ;;
  esac
}

emit_c() {
  path="$1"; desc="$2"
  cat <<EOF
/* =============================================================================
 * File:           $path
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   $CREATED_DATE
 * Version:        $VERSION
 *
 * Description:
 *   $desc
 *
 * License:
 *   Origin License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */
EOF
}

emit_hash() {
  path="$1"; desc="$2"
  cat <<EOF
# ==============================================================================
# File:           $path
# Project:        Origin
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   $CREATED_DATE
# Version:        $VERSION
#
# Description:
#   $desc
#
# License:
#   Origin License v1.0 — see LICENSE in the repository root.
# ==============================================================================
EOF
}

emit_html() {
  path="$1"; desc="$2"
  cat <<EOF
<!-- =============================================================================
 File:           $path
 Project:        Origin
 Author:         USDTG GROUP TECHNOLOGY LLC
 Developer:      Irfan Gedik
 Created Date:   $CREATED_DATE
 Version:        $VERSION

 Description:
   $desc

 License:
   Origin License v1.0 — see LICENSE in the repository root.
============================================================================= -->
EOF
}

create_file() {
  target="$1"
  rel="$(relative "$target")"
  desc="$(description_for "$rel")"
  parent="$(dirname "$target")"
  name="$(basename "$target")"
  ext="${name##*.}"

  mkdir_safe "$parent"

  if [ -f "$target" ] && [ "$FORCE" -ne 1 ]; then
    log SKIP "file exists $target"
    return
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log CREATE "file $target"
    return
  fi

  case "$ext" in
    rs|ts|tsx|js|jsx|css|proto) emit_c "$rel" "$desc" > "$target" ;;
    md|html) emit_html "$rel" "$desc" > "$target" ;;
    sh|bash)
      { printf '#!/usr/bin/env bash\n#\n'; emit_hash "$rel" "$desc"; printf '\nset -Eeuo pipefail\n'; } > "$target"
      chmod +x "$target"
      ;;
    json)
      cat > "$target" <<EOF
{
  "_origin_file_header": {
    "file": "$rel",
    "project": "Origin",
    "author": "USDTG GROUP TECHNOLOGY LLC",
    "developer": "Irfan Gedik",
    "created_date": "$CREATED_DATE",
    "version": "$VERSION",
    "description": "$desc",
    "license": "Origin License v1.0 — see LICENSE in the repository root."
  }
}
EOF
      ;;
    csv)
      emit_hash "$rel" "$desc" > "$target"
      printf 'language,root,normalized,meaning,phonetic,source,confidence,enabled\n' >> "$target"
      ;;
    *) emit_hash "$rel" "$desc" > "$target" ;;
  esac

  log CREATE "file $target"
}

create_campaign() {
  target="$ROOT_DIR/campaigns/logistics-platform-v1.toml"
  mkdir_safe "$(dirname "$target")"

  if [ -f "$target" ] && [ "$FORCE" -ne 1 ]; then
    log SKIP "file exists $target"
    return
  fi

  if [ "$DRY_RUN" -eq 1 ]; then
    log CREATE "file $target"
    return
  fi

  cat > "$target" <<EOF
# ==============================================================================
# File:           campaigns/logistics-platform-v1.toml
# Project:        Origin
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   $CREATED_DATE
# Version:        $VERSION
#
# Description:
#   Defines the first executable naming campaign for the logistics platform.
#
# License:
#   Origin License v1.0 — see LICENSE in the repository root.
# ==============================================================================

[campaign]
id = "logistics-platform-v1"
count = 1000
seed = 42
min_length = 5
max_length = 9

[generation]
modes = ["synthetic", "ancient", "hybrid", "mutation"]
languages = ["sumerian", "akkadian", "old-turkic", "latin", "sanskrit"]

[qualities]
trust = 95
movement = 95
intelligence = 90
longevity = 95
enterprise = 100
playful = 10

[constraints]
ascii_only = true
easy_pronunciation = true
easy_spelling = true
product_family_compatible = true

[validation]
domain_tlds = ["com", "io", "ai"]
github = true
crates_io = true
npm = true
web_collision = true
EOF

  log CREATE "file $target"
}

FILES='
crates/origin-core/src/candidate.rs
crates/origin-core/src/campaign.rs
crates/origin-core/src/deduplication.rs
crates/origin-core/src/selection.rs
crates/origin-core/src/language/mod.rs
crates/origin-core/src/language/model.rs
crates/origin-core/src/language/catalog.rs
crates/origin-core/src/language/sumerian.rs
crates/origin-core/src/language/akkadian.rs
crates/origin-core/src/language/old_turkic.rs
crates/origin-core/src/language/latin.rs
crates/origin-core/src/language/sanskrit.rs
crates/origin-core/src/language/synthetic.rs
crates/origin-core/src/morphology/mod.rs
crates/origin-core/src/morphology/merge.rs
crates/origin-core/src/morphology/mutate.rs
crates/origin-core/src/morphology/collapse.rs
crates/origin-core/src/morphology/normalize.rs
crates/origin-core/src/morphology/beautify.rs
crates/origin-core/src/generator/mod.rs
crates/origin-core/src/generator/synthetic.rs
crates/origin-core/src/generator/ancient.rs
crates/origin-core/src/generator/hybrid.rs
crates/origin-core/src/generator/mutation.rs
crates/origin-core/src/generator/grammar.rs
crates/origin-core/src/generator/pipeline.rs
crates/origin-core/src/scoring/mod.rs
crates/origin-core/src/scoring/pronunciation.rs
crates/origin-core/src/scoring/spelling.rs
crates/origin-core/src/scoring/rhythm.rs
crates/origin-core/src/scoring/typography.rs
crates/origin-core/src/scoring/memorability.rs
crates/origin-core/src/scoring/length.rs
crates/origin-core/src/scoring/negative_meaning.rs
crates/origin-core/src/scoring/composite.rs
crates/origin-core/src/export/mod.rs
crates/origin-core/src/export/json.rs
crates/origin-core/src/export/jsonl.rs
crates/origin-core/src/export/csv.rs
crates/origin-core/src/export/markdown.rs
crates/origin-core/src/export/manifest.rs
crates/origin-core/src/validation/mod.rs
crates/origin-core/src/validation/model.rs
crates/origin-core/src/validation/domain.rs
crates/origin-core/src/validation/github.rs
crates/origin-core/src/validation/crates_io.rs
crates/origin-core/src/validation/npm.rs
crates/origin-core/src/validation/web.rs
crates/origin-core/src/validation/trademark.rs
crates/origin-core/src/validation/evidence.rs
crates/origin-cli/src/commands/mod.rs
crates/origin-cli/src/commands/generate.rs
crates/origin-cli/src/commands/check.rs
crates/origin-cli/src/commands/compare.rs
crates/origin-cli/src/commands/validate.rs
crates/origin-cli/src/commands/campaign.rs
crates/origin-cli/src/commands/export.rs
crates/origin-cli/src/output/mod.rs
crates/origin-cli/src/output/table.rs
crates/origin-cli/src/output/progress.rs
crates/origin-core/tests/generation_modes.rs
crates/origin-core/tests/morphology_properties.rs
crates/origin-core/tests/scoring_profiles.rs
crates/origin-core/tests/campaign_pipeline.rs
crates/origin-core/tests/validation_contracts.rs
crates/origin-cli/tests/generate_cli.rs
crates/origin-cli/tests/campaign_cli.rs
crates/origin-cli/tests/validate_cli.rs
benches/generation.rs
benches/scoring.rs
benches/similarity.rs
data/README.md
data/roots/sumerian.csv
data/roots/akkadian.csv
data/roots/old_turkic.csv
data/roots/latin.csv
data/roots/sanskrit.csv
data/roots/synthetic.csv
data/forbidden/en.csv
data/forbidden/tr.csv
data/forbidden/de.csv
data/forbidden/fr.csv
data/forbidden/es.csv
data/forbidden/it.csv
data/reference-brands/technology.csv
data/reference-brands/logistics.csv
data/language-profiles/international-tech.toml
data/language-profiles/turkish.toml
data/language-profiles/english.toml
data/language-profiles/german.toml
data/language-profiles/french.toml
data/language-profiles/spanish.toml
fixtures/campaigns/minimal.toml
fixtures/campaigns/logistics.toml
fixtures/validation/domain_available.json
fixtures/validation/domain_registered.json
fixtures/validation/github_collision.json
docs/ARCHITECTURE.md
docs/CAPABILITY_MATRIX.md
docs/DATA_PROVENANCE.md
docs/VALIDATION_BOUNDARIES.md
docs/CAMPAIGN_FORMAT.md
docs/SCORING_MODEL.md
docs/NAME_GRAMMARS.md
docs/RELEASE_CRITERIA_V01.md
scripts/check.sh
scripts/generate-logistics-campaign.sh
scripts/update-language-digests.sh
scripts/validate-datasets.sh
scripts/bench.sh
scripts/release-v01.sh
.github/workflows/datasets.yml
.github/workflows/security.yml
.github/workflows/benchmarks.yml
.github/ISSUE_TEMPLATE/dataset-source.md
.github/ISSUE_TEMPLATE/collision-report.md
.github/pull_request_template.md
rustfmt.toml
clippy.toml
deny.toml
.editorconfig
.env.example
'

printf '[INFO] Origin v0.1 bootstrap started: %s\n' "$ROOT_DIR"

printf '%s\n' "$FILES" | while IFS= read -r rel; do
  [ -z "$rel" ] && continue
  create_file "$ROOT_DIR/$rel"
done

create_campaign

printf '\n[INFO] Origin v0.1 skeleton completed.\n'
printf '[INFO] Existing implementation files were preserved unless --force was used.\n'
printf '[INFO] Next coding order: candidate → language → morphology → generator → scoring → campaign → CLI.\n'
