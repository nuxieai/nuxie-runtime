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
  "sync/databind_null_artboard_swap.riv|0160b4572f217271df84072b08476d433a71c5bf78a9917f39fbc03239560a1f|30a0e2d42e2e6d091350d6edb816e165e27f7988"
  "sync/animated_cubic_participant.riv|4cbe2e4972ea150a49b389ddd622419de9750c03c213af629a7d1bcb007d3f4f|3de78b0d61202b67805af012dfe69a4894b62f92|layout/animated_cubic_participant.riv"
  "sync/animated_participant.riv|9f3d43fbef46f3bcc7dd70e8c4a8ed1e1ec21b78d9b20457c582fb3c3bb377be|3de78b0d61202b67805af012dfe69a4894b62f92|layout/animated_participant.riv"
  "sync/constrained_participant.riv|663b706ecfc92ea3b95af2b036480a314dac58ca715e287a60da494b6f6dfc24|3de78b0d61202b67805af012dfe69a4894b62f92|layout/constrained_participant.riv"
  "sync/display_none_participant.riv|9542270d8da850126161879131f67d76aed48f934cb2c2130c7e7237d5557aa0|3de78b0d61202b67805af012dfe69a4894b62f92|layout/display_none_participant.riv"
  "sync/fixed_participant.riv|c3d80d8fc0eae983abb37a9b5b2e381c487e0991d84e91560b015565193f8df9|3de78b0d61202b67805af012dfe69a4894b62f92|layout/fixed_participant.riv"
  "sync/grid_2x2.riv|c0443d592e6069bd7026f2b95f0a9ac070f7d26d4523becc7b6f149efca241f8|3de78b0d61202b67805af012dfe69a4894b62f92|layout/grid_2x2.riv"
  "sync/grid_auto_rows.riv|89b7085af0a98dc331749b39986c86d80e59b3126f09816adecb2bcf78a9cf23|3de78b0d61202b67805af012dfe69a4894b62f92|layout/grid_auto_rows.riv"
  "sync/grid_participant.riv|0b7b99ae54589ff1e213ff561fafa20aebff9fc64930db79c5bde9157b21a511|3de78b0d61202b67805af012dfe69a4894b62f92|layout/grid_participant.riv"
  "sync/grid_track_types.riv|cdb81c441e5c0e7230dfd7adfb6b3aac3df33a8efde7cbae58bd6280618beac4|3de78b0d61202b67805af012dfe69a4894b62f92|layout/grid_track_types.riv"
  "sync/group_participant.riv|6d6855486bd1786857721661d100cfad9268b6cf056bc6d5eb1219f3bf1ef1ba|3de78b0d61202b67805af012dfe69a4894b62f92|layout/group_participant.riv"
  "sync/hug_participant.riv|d7d83d26cc7d64e164d0b8300d1c3851671729879cca321ab67ac3f0e52e0dae|3de78b0d61202b67805af012dfe69a4894b62f92|layout/hug_participant.riv"
  "sync/list_in_group_joins_layout.riv|b39815f9b8180aed4f232103f4ebb25664403bbe098c42a3a83ee20196e94357|3de78b0d61202b67805af012dfe69a4894b62f92|layout/list_in_group_joins_layout.riv"
  "sync/nested_group_participant.riv|ec7210b9fca06c925952918663c7b28d79bfc77e8b8ee92b959af100c4e4d870|3de78b0d61202b67805af012dfe69a4894b62f92|layout/nested_group_participant.riv"
  "sync/solo_participant.riv|5ca81c977a5feac7ba7fcb9eb116db3b3482905ccfe08f87cdc9721ca411bd5d|3de78b0d61202b67805af012dfe69a4894b62f92|layout/solo_participant.riv"
  "sync/stack.riv|b6e2f1c0cbadb4c4e14d9734bf46f812dc833d71e1be3803f2bc81060907fb78|3de78b0d61202b67805af012dfe69a4894b62f92|layout/stack.riv"
  "sync/stack_participant.riv|4eb82cadc6fcefa1c33f33e68c16fda1ea1ee1b1b3fa6a95acb6a1849e8e8194|3de78b0d61202b67805af012dfe69a4894b62f92|layout/stack_participant.riv"
  "sync/styled_flex.riv|cdafcd4b12649afb379b7eccbb6f287723d9ad30c1123347f075a2fd62969671|3de78b0d61202b67805af012dfe69a4894b62f92|layout/styled_flex.riv"
  "sync/layout_grid_stack.riv|21275c1cb9946e1c93ba3b5063e003db0b5f3e647f60359c8b14ae2b2ed9d6d3|3de78b0d61202b67805af012dfe69a4894b62f92|layout_grid_stack.riv"
)

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

for entry in "${assets[@]}"; do
  IFS='|' read -r relative expected source_ref source_path <<< "$entry"
  source_ref=${source_ref:-$ref}
  source_path=${source_path:-${relative##*/}}
  name=${relative##*/}
  destination="$repo_root/fixtures/$relative"
  mkdir -p "$(dirname "$destination")"

  if [[ -n "$runtime_dir" && -f "$runtime_dir/tests/unit_tests/assets/$source_path" ]]; then
    cp "$runtime_dir/tests/unit_tests/assets/$source_path" "$destination"
  elif [[ ! -f "$destination" || "$(sha256 "$destination")" != "$expected" ]]; then
    curl --fail --location --silent --show-error \
      "$base_url/$source_ref/tests/unit_tests/assets/$source_path" \
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
