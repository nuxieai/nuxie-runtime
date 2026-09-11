#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
lane="${1:-geometry}"
case "$lane" in
  geometry) ;;
  native|native-glyphs)
    if [[ "$(uname -s)" != Darwin ]]; then
      echo 'The native runner currently requires macOS Metal.' >&2
      exit 1
    fi
    ;;
  *) echo 'Usage: run.sh [geometry|native|native-glyphs]' >&2; exit 1 ;;
esac
cargo_options=(--locked)
if [[ "$lane" == native-glyphs ]]; then
  cargo_options+=(--features native-glyph-controls)
  export NUXIE_NATIVE_GLYPHS=1
else
  unset NUXIE_NATIVE_GLYPHS
fi
NUXIE_TEXT_TEST_FONT="$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf" \
  cargo test -p nuxie-runtime --locked --lib css_
NUXIE_TEXT_TEST_FONT="$PWD/tools/html-to-riv/tests/assets/Inter-Regular.ttf" \
  cargo test -p nuxie-runtime --locked --test css_decoration
cargo test -p nuxie-html-to-riv "${cargo_options[@]}"
cargo build -p nuxie-html-to-riv "${cargo_options[@]}" --bin html-to-riv --example probe --example outline_probe
python3 tools/html-to-riv/validation/precision-wide-advance.py
node --test tools/html-to-riv/tests/runtime-requirements.test.mjs
bash tools/html-to-riv/validation/build-wasm.sh
node --test tools/html-to-riv/tests/javascript.test.mjs tools/html-to-riv/tests/gradient-parity.test.mjs
node --test tools/html-to-riv/tests/package.test.mjs
if [[ "$lane" != geometry ]]; then
  cargo build -p renderer-replay --locked --features native-metal --bin renderer-replay
fi
cd tools/html-to-riv
npm ci
npm run typecheck
npx playwright install chromium
node validation/capitalize-reference.mjs
node validation/locale-reference.mjs
node validation/underline-reference.mjs
node validation/underline-origin-reference.mjs
npm run test:report
if [[ "$lane" != geometry ]]; then npm test; else npm run test:geometry; fi
if [[ "$lane" == native-glyphs ]]; then
  node validation/glyph-state-control.mjs
  node validation/glyph-state-control.mjs underline
  node validation/underline-font-metrics-control.mjs
  node validation/underline-dpr-control.mjs
fi
