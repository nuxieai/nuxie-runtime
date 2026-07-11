#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
ref=${RIVE_RUNTIME_REF:-7c778d13c5d903b3b74eec1dd6bb68a811dea5f2}
runtime_dir=${RIVE_RUNTIME_DIR:-}
base_url="https://raw.githubusercontent.com/rive-app/rive-runtime/$ref/tests/unit_tests/assets"

assets=(
  "animation/smi_test.riv|51fb2ef2ca7a2014b4f4586df1c0894fef7d92d422a27ac82fef1459407b73f8"
  "animation/state_machine_transition.riv|65fc100a82b1c2015cdd6267e5b3f3dea0d7a772c1710a7e9c4a09c883e26e3e"
  "graph/clipping_and_draw_order.riv|aac75187236fcd528e92cdba9ad5022a7e1346b8475ea2e68dd2dde1023fdc97"
  "graph/dependency_test.riv|1321815e37a5101a9176b2e147cac2b203cc9cd81366e49d46d1f97c28ec784d"
  "graph/draw_rule_cycle.riv|db0cf30b8df689dc1d29dfbf4316b69c61270743a7eb94bcd3ac27600a00c9c3"
  "minimal/long_name.riv|9f4b5f73afdd9223e7351fe853afa587f242868298576320ac6556ae91c54e9f"
  "minimal/two_artboards.riv|480472d9942711492ce37cdba9aea6266f254633f5a2ac4a9e30f9d0eca70e8c"
)

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

for entry in "${assets[@]}"; do
  relative=${entry%%|*}
  expected=${entry##*|}
  name=${relative##*/}
  destination="$repo_root/fixtures/$relative"
  mkdir -p "$(dirname "$destination")"

  if [[ -n "$runtime_dir" && -f "$runtime_dir/tests/unit_tests/assets/$name" ]]; then
    cp "$runtime_dir/tests/unit_tests/assets/$name" "$destination"
  elif [[ ! -f "$destination" || "$(sha256 "$destination")" != "$expected" ]]; then
    curl --fail --location --silent --show-error "$base_url/$name" --output "$destination"
  fi

  actual=$(sha256 "$destination")
  if [[ "$actual" != "$expected" ]]; then
    echo "fixture checksum mismatch: $relative (expected $expected, got $actual)" >&2
    exit 1
  fi

  for target in fuzz_import fuzz_runtime fuzz_pointer; do
    seed_dir="$repo_root/fuzz/seeds/$target"
    mkdir -p "$seed_dir"
    cp "$destination" "$seed_dir/$name"
  done
done

echo "test assets ready (rive-runtime@$ref)"
