#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
ref=${RIVE_RUNTIME_REF:-d788e8ec6e8b598526607d6a1e8818e8b637b60c}
runtime_dir=${RIVE_RUNTIME_DIR:-}
base_url="https://raw.githubusercontent.com/rive-app/rive-runtime"

assets=(
  "animation/smi_test.riv|51fb2ef2ca7a2014b4f4586df1c0894fef7d92d422a27ac82fef1459407b73f8"
  "animation/state_machine_transition.riv|65fc100a82b1c2015cdd6267e5b3f3dea0d7a772c1710a7e9c4a09c883e26e3e"
  "flow/component_list_2.riv|b1541dfdba9f0a873245838ac560b27c21c181f9745d8052d9133163a530ef6e"
  "flow/data_binding_test.riv|c7e61a409945ffc70eb72c35b6efcd9a6115a00de0adc74419360ab88b740308"
  "flow/replace_view_model.riv|99a04bd4ff5c0a9b333e83c6a3840861fac6a26237329c7eef6993b26b64e4f5"
  "graph/clipping_and_draw_order.riv|aac75187236fcd528e92cdba9ad5022a7e1346b8475ea2e68dd2dde1023fdc97"
  "graph/dependency_test.riv|1321815e37a5101a9176b2e147cac2b203cc9cd81366e49d46d1f97c28ec784d"
  "graph/draw_rule_cycle.riv|db0cf30b8df689dc1d29dfbf4316b69c61270743a7eb94bcd3ac27600a00c9c3"
  "minimal/long_name.riv|9f4b5f73afdd9223e7351fe853afa587f242868298576320ac6556ae91c54e9f"
  "minimal/two_artboards.riv|480472d9942711492ce37cdba9aea6266f254633f5a2ac4a9e30f9d0eca70e8c"
  "sync/scope_probe.riv|fe8c68d337616c0e0f6747012b592298a48a60655d88b28ca7a8fd91e1c02347|b73bc6755421c41281f9d5c8c04d8444fc43f585"
  "sync/bidirectional_stateful_property.riv|c2813f0ad0f5aedff70ec666f21118b41e611ab87951b5192960599c9be82583|e85a11604edd9a2a50bbe2f04da4a91b0293ccd6"
  "sync/paused_nested_artboard_opacity.riv|642c9f7fd909b9955a875e0bb745d0998d3ac4b64a11b863b09e3b0ee5682944|0a2e478ac331586387308068e01306225ecbb20d"
  "sync/solo_index_test.riv|e857c0d1f76cec0be8d8b9d8308ea9a0f581de29ed752b952940d90b5f6a16f2|38c924123ffb8ad9541ad724ef4de860e5705482"
  "sync/stateful_component_image_test.riv|47dcbcd02cd228f0e4ec71eaac84748f46f95b24737818f61b04d46242b48393|353ef4fccbf6f1801def7d737a4103657dc63a1c"
  "sync/databind_null_artboard_swap.riv|0160b4572f217271df84072b08476d433a71c5bf78a9917f39fbc03239560a1f|30a0e2d42e2e6d091350d6edb816e165e27f7988"
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
  remainder=${entry#*|}
  expected=${remainder%%|*}
  source_ref=$ref
  if [[ "$remainder" == *'|'* ]]; then
    source_ref=${remainder#*|}
  fi
  name=${relative##*/}
  destination="$repo_root/fixtures/$relative"
  mkdir -p "$(dirname "$destination")"

  source_path="tests/unit_tests/assets/$name"
  if [[ -n "$runtime_dir" ]] \
    && git -C "$runtime_dir" cat-file -e "$source_ref:$source_path" 2>/dev/null; then
    git -C "$runtime_dir" show "$source_ref:$source_path" > "$destination"
  elif [[ ! -f "$destination" || "$(sha256 "$destination")" != "$expected" ]]; then
    curl --fail --location --silent --show-error \
      "$base_url/$source_ref/tests/unit_tests/assets/$name" \
      --output "$destination"
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
