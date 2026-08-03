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
  "sync/layout_text_match.riv|1fea1a6102259aacd9b164cfac0b4a2f67d4fa4587b78f5eb25a2f195de7bcdb|f5cfee3a5d6a6728167b58a71b47455ace063690"
  "sync/artboard_opacity_and_transform_test.riv|100dbf5c04159ea7e8e6f12ce16daf1ee6f15a74c2d3dc074e2dbde4e877af80|e0d4913fa0f88d9f4b57c53006e7f9712417205f"
  "sync/databind_null_artboard_swap.riv|0160b4572f217271df84072b08476d433a71c5bf78a9917f39fbc03239560a1f|30a0e2d42e2e6d091350d6edb816e165e27f7988"
  "sync/component_list_clipped_viewport.riv|a20c9fd4936c2b7f435011e7afddd276797e95d68b574ec2c914331afd092bac|482b24a188bb9e367e983bf05235761707a89718"
  "sync/vm_listener_fire_event.riv|683a8ed1ad102fa9dd1020d61df301594a9a9fd20b97655c1f0da62e7b994838|482b24a188bb9e367e983bf05235761707a89718"
  "sync/image_computed_transform_bind.riv|17a6d1e6e9f9713cf78d522b96957c21c12d60aa40d54285759bee151c9f4730|15da0652fc10b55ef1fbd32e3e19582c9dc271f2"
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
  "sync/data_bind_blob_test.riv|46b47578e6dd6e70ecffac35449498275fd2ee8773efbc5cb04d22cad5fb7e58|36aabf60d771a91a6e32b453409add2b5831b3c5"
  "sync/data_enum_roundtrip.rml|edceb37578a7684ba3816db43ab12bd73ee75a4a911b5ee1ee32db534d22fd24|36aabf60d771a91a6e32b453409add2b5831b3c5"
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

  if [[ -n "$runtime_dir" && -n "$source_ref" ]] \
    && git -C "$runtime_dir" cat-file -e "$source_ref:tests/unit_tests/assets/$source_path" 2>/dev/null; then
    # Fetch at the recorded source ref: the pinned working tree may hold a
    # different revision of the same asset than the ref that vendored it.
    git -C "$runtime_dir" show "$source_ref:tests/unit_tests/assets/$source_path" > "$destination"
  elif [[ -n "$runtime_dir" && -f "$runtime_dir/tests/unit_tests/assets/$source_path" ]]; then
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

  # Keep S4-42's as-yet-unpinned assets out of the active fuzz seed sets.
  if [[ "$relative" != "sync/data_bind_blob_test.riv" && "$relative" != "sync/data_enum_roundtrip.rml" ]]; then
    for target in fuzz_import fuzz_runtime fuzz_pointer; do
      seed_dir="$repo_root/fuzz/seeds/$target"
      mkdir -p "$seed_dir"
      cp "$destination" "$seed_dir/$name"
    done
  fi
done

silver_destination="$repo_root/fixtures/sync/data_bind_blob_test.sriv"
silver_expected="e3fc7bfbb227bd57c77c63589607616e81f7c7223239eb0d56efebf1d90ce079"
if [[ -n "$runtime_dir" && -f "$runtime_dir/tests/unit_tests/silvers/data_bind_blob_test.sriv" ]]; then
  cp "$runtime_dir/tests/unit_tests/silvers/data_bind_blob_test.sriv" "$silver_destination"
elif [[ ! -f "$silver_destination" || "$(sha256 "$silver_destination")" != "$silver_expected" ]]; then
  curl --fail --location --silent --show-error \
    "$base_url/36aabf60d771a91a6e32b453409add2b5831b3c5/tests/unit_tests/silvers/data_bind_blob_test.sriv" \
    --output "$silver_destination"
fi
silver_actual=$(sha256 "$silver_destination")
if [[ "$silver_actual" != "$silver_expected" ]]; then
  echo "fixture checksum mismatch: sync/data_bind_blob_test.sriv (expected $silver_expected, got $silver_actual)" >&2
  exit 1
fi

echo "test assets ready (rive-runtime@$ref)"
