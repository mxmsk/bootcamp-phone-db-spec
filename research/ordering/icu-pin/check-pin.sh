#!/bin/sh
# Check of the ICU4X pinning (ADR 0039 item 4 (a), (c), (d)). Usage: check-pin.sh [PROJECT_DIR] (default -
# the current one; in the product the directory of the root package is passed, where Cargo.toml, Cargo.lock and expected-icu-tree.txt live;
# the expected list is taken from PROJECT_DIR/expected-icu-tree.txt). The network is needed for the first crate download; CARGO_HOME and CARGO_TARGET_DIR can be moved outside the repository. Exit 0 only if everything holds:
#  1. every direct icu_* dependency in Cargo.toml: exact version "=2.4.0", default-features = false, of the features
#     only compiled_data is allowed; the set of direct icu_* equals expected-icu-tree.txt;
#  2. the set and versions of icu_* crates in the whole tree (--locked) equal expected-icu-tree.txt;
#  3. no icu_* crate in the tree has the features auto, lstm, unstable, datagen, serde, experimental, export enabled.
# Limitations: the root package is checked (.packages[0] in cargo metadata --no-deps; in a workspace of several packages
# the script is run on the package that depends on icu_*). Crates from [patch]/path (a vendored copy per ADR 0039 item 6 (b))
# are compared by name and version - the "(path)" suffix in cargo tree is dropped; the CONTENTS of such a copy the script does not
# check (they are covered by the references of ADR 0039 item 5), and the presence of [patch]/path is detected by a line in the output below.
set -eu
cd "${1:-.}"
T="${TMPDIR:-/tmp}"
EXPECTED=expected-icu-tree.txt
cut -d' ' -f1 "$EXPECTED" | sort > "$T/icu-expected-names.txt"

# 1. Direct dependencies.
cargo metadata --locked --format-version 1 --no-deps > "$T/icu-meta.json"
jq -r '.packages[0].dependencies[] | select(.name | startswith("icu_")) | .name' "$T/icu-meta.json" | sort > "$T/icu-direct.txt"
diff "$T/icu-direct.txt" "$T/icu-expected-names.txt" || { echo "direct icu_* set differs from $EXPECTED" >&2; exit 1; }
BAD=$(jq -r '.packages[0].dependencies[] | select(.name | startswith("icu_"))
  | select(.req != "=2.4.0" or .uses_default_features != false or ((.features - ["compiled_data"]) | length > 0))
  | "\(.name) req=\(.req) default-features=\(.uses_default_features) features=\(.features)"' "$T/icu-meta.json")
if [ -n "$BAD" ]; then echo "direct dependency not pinned as required:" >&2; echo "$BAD" >&2; exit 1; fi

# 2. Tree.
cargo tree --locked --prefix none --format '{p}' | grep '^icu_' | sed 's/ (.*$//' | sort -u > "$T/icu-tree.txt"
diff "$T/icu-tree.txt" "$EXPECTED" || { echo "icu_* tree differs from $EXPECTED" >&2; exit 1; }

if cargo tree --locked --prefix none --format '{p}' | grep -E '^icu_[a-z0-9_]+ v[^ ]+ \(/' >/dev/null; then
  echo "NOTE: icu_* crate(s) from path/[patch] in the tree (vendored copy, ADR 0039 p. 6 (b)); contents not checked here" >&2
fi

# 3. Forbidden features on all icu_*.
cargo tree --locked -e features --prefix none > "$T/icu-features.txt"
if grep -E '^icu_[a-z0-9_]+ feature "(auto|lstm|unstable|datagen|serde|experimental|export)"' "$T/icu-features.txt"; then
  echo "forbidden feature on icu_* crate" >&2
  exit 1
fi
echo "icu_* pin OK"
