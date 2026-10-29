#!/bin/sh
# Build and run of the ICU4C cross-check. Requires ICU4C with headers (macOS: brew install icu4c).
set -e
ICU=${ICU_PREFIX:-/opt/homebrew/opt/icu4c}
OUT=${TMPDIR:-/tmp}/icu4c-check
c++ -std=c++17 -I"$ICU/include" -L"$ICU/lib" check.cpp -licui18n -licuuc -licudata -o "$OUT"
DYLD_LIBRARY_PATH="$ICU/lib" LD_LIBRARY_PATH="$ICU/lib" "$OUT"
