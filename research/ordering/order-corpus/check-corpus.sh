#!/bin/sh
# Check of the order corpus against the snapshot (ADR 0039 item 5 (b)). Run from this directory; the network is needed for the first crate download.
#   ./check-corpus.sh          compare the SHA-256 of the order with the order-corpus.sha256 snapshot (exit 1 on a mismatch)
#   ./check-corpus.sh --write  create the snapshot (done once, manually, when preparing the reference; not run in v1)
set -eu
T="${TMPDIR:-/tmp}"
# The program exits with code 1 if the order by sort key bytes differs from compare() (ADR 0039 item 9 (d)).
cargo run --locked --release --quiet > "$T/order-corpus.out" 2> "$T/order-corpus.err" || {
  echo "order corpus: sort-key order differs from compare()" >&2
  cat "$T/order-corpus.err" >&2
  exit 1
}
N=$(sed -n 's/^names: //p' "$T/order-corpus.err")
H=$(shasum -a 256 "$T/order-corpus.out" | cut -d' ' -f1)
if [ "${1:-}" = "--write" ]; then
  echo "$H  names=$N" > order-corpus.sha256
  echo "snapshot written: $H names=$N"
  exit 0
fi
EXPECTED=$(cut -d' ' -f1 order-corpus.sha256)
EXPECTED_N=$(sed -n 's/.*names=//p' order-corpus.sha256)
if [ "$H" = "$EXPECTED" ] && [ "$N" = "$EXPECTED_N" ]; then
  echo "order corpus OK: $H names=$N"
else
  echo "order corpus MISMATCH: got $H names=$N, snapshot $EXPECTED names=$EXPECTED_N" >&2
  exit 1
fi
