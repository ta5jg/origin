#!/usr/bin/env bash
set -Eeuo pipefail

###############################################################################
# ORIGIN v0.1 Bootstrap
###############################################################################

PROJECT_ROOT="$(pwd)"

CORE="$PROJECT_ROOT/origin-core/src"
CLI="$PROJECT_ROOT/origin-cli/src"

echo "==> Creating Origin v0.1 structure..."

mkdir -p \
"$CORE/generator" \
"$CORE/morphology" \
"$CORE/language" \
"$CORE/scoring" \
"$CORE/export"

mkdir -p \
"$CLI"

###############################################################################
# Header
###############################################################################

create_rs() {

    local FILE="$1"
    local DESC="$2"

    [[ -f "$FILE" ]] && return

    cat > "$FILE" <<EOF
/* =============================================================================
 * File:           $(basename "$FILE")
 * Project:        Origin
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   $(date +%F)
 * Version:        0.1.0
 *
 * Description:
 *   $DESC
 *
 * License:
 *   Origin License v1.0 — see LICENSE in repository root.
 * ============================================================================= */

EOF
}

###############################################################################
# origin-core
###############################################################################

create_rs "$CORE/lib.rs" \
"Origin Core Library."

create_rs "$CORE/candidate.rs" \
"Candidate model."

create_rs "$CORE/generator.rs" \
"Name generation engine."

create_rs "$CORE/morphology.rs" \
"Morphology engine."

create_rs "$CORE/language.rs" \
"Language dataset."

create_rs "$CORE/scoring.rs" \
"Brand scoring."

create_rs "$CORE/export.rs" \
"Export utilities."

###############################################################################
# generator/
###############################################################################

create_rs "$CORE/generator/mod.rs" \
"Generator module."

create_rs "$CORE/generator/synthetic.rs" \
"Synthetic generator."

create_rs "$CORE/generator/ancient.rs" \
"Ancient language generator."

create_rs "$CORE/generator/hybrid.rs" \
"Hybrid generator."

create_rs "$CORE/generator/mutation.rs" \
"Mutation generator."

###############################################################################
# morphology/
###############################################################################

create_rs "$CORE/morphology/mod.rs" \
"Morphology module."

create_rs "$CORE/morphology/merge.rs" \
"Root merge."

create_rs "$CORE/morphology/mutate.rs" \
"Mutation algorithms."

create_rs "$CORE/morphology/normalize.rs" \
"Normalization."

create_rs "$CORE/morphology/beautify.rs" \
"Beautification."

###############################################################################
# language/
###############################################################################

create_rs "$CORE/language/mod.rs" \
"Language module."

create_rs "$CORE/language/dataset.rs" \
"Language datasets."

create_rs "$CORE/language/sumerian.rs" \
"Sumerian roots."

create_rs "$CORE/language/akkadian.rs" \
"Akkadian roots."

create_rs "$CORE/language/latin.rs" \
"Latin roots."

create_rs "$CORE/language/old_turkic.rs" \
"Old Turkic roots."

###############################################################################
# scoring/
###############################################################################

create_rs "$CORE/scoring/mod.rs" \
"Scoring module."

create_rs "$CORE/scoring/pronounce.rs" \
"Pronunciation score."

create_rs "$CORE/scoring/rhythm.rs" \
"Rhythm score."

create_rs "$CORE/scoring/visual.rs" \
"Visual score."

create_rs "$CORE/scoring/uniqueness.rs" \
"Uniqueness score."

create_rs "$CORE/scoring/length.rs" \
"Length score."

###############################################################################
# export/
###############################################################################

create_rs "$CORE/export/mod.rs" \
"Export module."

create_rs "$CORE/export/json.rs" \
"JSON exporter."

create_rs "$CORE/export/csv.rs" \
"CSV exporter."

create_rs "$CORE/export/markdown.rs" \
"Markdown exporter."

###############################################################################
# CLI
###############################################################################

create_rs "$CLI/main.rs" \
"Origin CLI."

###############################################################################
# README
###############################################################################

if [[ ! -f README.md ]]; then
cat > README.md <<EOF
# Origin

Universal Brand Intelligence Engine

EOF
fi

###############################################################################
# LICENSE
###############################################################################

touch LICENSE

echo
echo "======================================="
echo " Origin v0.1 structure created."
echo "======================================="