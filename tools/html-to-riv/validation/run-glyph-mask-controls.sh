#!/usr/bin/env bash
# Experimental regression gate; this does not replace the main acceptance suite.
set -euo pipefail
cd "$(dirname "$0")/../../.."
if [[ "$(uname -s)" != Darwin ]]; then
  echo 'Native glyph-mask controls require macOS CoreText and Metal.' >&2
  exit 1
fi
mkdir -p output/playwright/html-to-riv
mode="${1:-swift}"
if [[ "$mode" != swift && "$mode" != rust ]]; then
  echo 'Expected optional mode: swift or rust' >&2
  exit 1
fi
cargo build -p nuxie-html-to-riv --locked --bin html-to-riv --example probe --example glyph_mask_composite
cargo build -p renderer-replay --locked --features native-metal --bin renderer-replay
if [[ "$mode" == rust ]]; then
  cargo build -p nuxie-html-to-riv --locked --features native-glyph-controls --example native_glyph_composite
  cargo test -p nuxie-html-to-riv --locked --features native-glyph-controls --test native_glyphs
else
  swiftc tools/html-to-riv/validation/native-glyph-probe.swift -o output/playwright/html-to-riv/native-glyph-probe
fi
# npm ci and Chromium installation are shared with validation/run.sh.
for width in 240 390 768; do
  node tools/html-to-riv/validation/text-diagnosis.mjs "$width" > "output/playwright/html-to-riv/glyph-source-${width}.log"
  node tools/html-to-riv/validation/glyph-mask-control.mjs "$width" "$mode"
done
