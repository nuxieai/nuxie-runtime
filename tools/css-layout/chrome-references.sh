#!/bin/sh
# Renders fixtures/css_layout/chrome/*.html with headless Chrome into
# 200 by 200 PNGs beside them: the browser's pixels that the CSS box-paint
# tests compare the runtime against (docs/CSS_LAYOUT.md, section 4).
# Usage: tools/css-layout/chrome-references.sh [path to the Chrome binary]
set -eu
chrome=${1:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
root=$(cd "$(dirname "$0")/../.." && pwd)
dir="$root/fixtures/css_layout/chrome"
profile=$(mktemp -d)
trap 'rm -rf "$profile"' EXIT
"$chrome" --version
for page in "$dir"/*.html; do
  "$chrome" --headless=new --disable-gpu --hide-scrollbars \
    --force-device-scale-factor=1 --window-size=200,200 \
    --user-data-dir="$profile" \
    --screenshot="${page%.html}.png" "file://$page" 2>/dev/null
done
