#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
ref=${RIVE_RUNTIME_REF:-1db281b3e82baf850635fd7aa2092920a80b6a2c}
runtime_dir=${RIVE_RUNTIME_DIR:-}
base_url="https://raw.githubusercontent.com/rive-app/rive-runtime"

assets=(
  "parity/Halloween_v3.riv|b786c27b0fc5ede17dca2365dd1830caa2e46654b89ac3f509337b792af48744|e949498e05483a852c10fbbdad2cd1941c15aebc|parity/Halloween_v3.riv"
  "parity/Knight_square_2.riv|cec7ff27afbf9506cd64c37cccf40b0e58a8eacd8a91dc7deb54c040ad9addb8|e949498e05483a852c10fbbdad2cd1941c15aebc|parity/Knight_square_2.riv"
  "parity/Tom_Morello.riv|2c2816f02811d6f349b6d73829d593e120939e554bde7f838b8c10f6c8ea0c82|e949498e05483a852c10fbbdad2cd1941c15aebc|parity/Tom_Morello.riv"
  "parity/UI_Swipe_left_to_delete.riv|712fdb564fc2cb2f646a0023999f8bb1be3f88558503a3c7da1d5610a03f40e2|e949498e05483a852c10fbbdad2cd1941c15aebc|parity/UI_Swipe_left_to_delete.riv"
  "parity/falling.riv|e130d00fc2317190c45f46f42900eaf7749bf03d89662574cacc8c101ff0f830|e949498e05483a852c10fbbdad2cd1941c15aebc|parity/falling.riv"
  "parity/popsicle_loader.riv|b97e2b06df39286470179205e4112bd72617a596fa1dbe21abeeed173631e7b5|e949498e05483a852c10fbbdad2cd1941c15aebc|parity/popsicle_loader.riv"
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
  "command_queue/two_artboards.riv|480472d9942711492ce37cdba9aea6266f254633f5a2ac4a9e30f9d0eca70e8c"
  "command_queue/multiple_state_machines.riv|62147bdb114ca6c11146209b55aa9a08a9adcb666e373429120ed0b896fc68de"
  "command_queue/entry.riv|e5512458171a2715a8736c3264475b50cbc2079c3fe2120845f5f45b8c25d60b"
  "command_queue/data_bind_test_cmdq.riv|054219ac8b15e384c8026c1ba9340385300fd00c649d8b787671d0d6fd7493b8"
  "command_queue/pointer_events.riv|f221020aebd1c5756adfee21c594b22e5af4d4f3e7cac6bdc6df097815df0c08"
  "command_queue/rapid_pointer_events.riv|e0584ba73df9bf8a7ac1a4ff1c3e381212967b10025d936e49ddab3d30a13079"
  "command_queue/settler.riv|b642028531bd087bdc9bbf021a0d24ac9eef5a7d3d5a4d0adb59ba531e5eee50"
  "command_queue/global_variables_test.riv|4c0e5946848a65d60202061ad9775f7e7d6ffc81fbf1ba514a6efcecb9c8b97a"
  "command_queue/hosted_font_file.riv|4805f22e51f2429013f98c5df6947d18b2efde50b93b70d33928e6e2c25624c6"
  "command_queue/hosted_image_file.riv|83d49331ba47368e8688a737dc6d2c15ccb5ee9e4e00c5dd92c7cce79eb18e14"
  "command_queue/batdude.png|32c86d18c059d4338cca1771faf9b43a80827ae8ea30d6cc10d64f681bfeec01"
  "command_queue/what.wav|1c7a0e0c6350a61c1be78f3d2799ba64df0ee4ba1c336e0f240babed355cf889|4ac7b32798da0482e441ef09304dc3b480ed3ee5|audio/what.wav"
  "command_queue/OpenSans-Italic.ttf|5eabd67fe3d8b5b5eee64504ea9e4a5ef7665b643577ef117f3c32fda67cd29f|4ac7b32798da0482e441ef09304dc3b480ed3ee5|fonts/OpenSans-Italic.ttf"
  "semantic/simpsons.riv|3e28691c3d3d5f09ba4ca3393f7e8f82dc3fbd72d398a7d239688c2756191c94||semantic/simpsons.riv"
  "semantic/semantic_list_scroll_focus_fixed.riv|38b2bcd006c44ecbe78ce0957d7382c36ea7e07f753aa7755116b654d0691240||semantic/semantic_list_scroll_focus_fixed.riv"
  "sync/scope_probe.riv|fe8c68d337616c0e0f6747012b592298a48a60655d88b28ca7a8fd91e1c02347|b73bc6755421c41281f9d5c8c04d8444fc43f585"
  "sync/text_style_background.riv|3209af9b117313f36521dbc3beffd40256168b516e0fc9b5ca6a0c4d10bdc360|1f04919af881fe51c929924dc773c835ca9071f0"
  "sync/scripted_path_effect_clip.riv|6bc76d33f6b3761cfd689a8de7f3dfc9dbc5ae34ae75921c63bde4ab9e7c9583|ddd1a2aacf62ee6e550b65b445d0ccdafe284e6a"
  "sync/ik_anim_test.riv|064492b51c369ebf843f16abd4e9915c89b8086d5f3e6bdc60c04e85e4fbce02|d25e6a4b6c1b8382b588f08371231373780fbcd5"
  "sync/color_passthrough_test.riv|83abb360ef1ee85e6c135b1f9975583e44c324b6e24537d12b1dc9ccb0b8aa5f|74c0d601c516f86db4847521198dba42080db06a"
  "sync/global_view_models_scripting_test.riv|4eba1794ba6c9d693d28ce902dda38b0eb38153c7dec18f73c4c2fbcab4838b1|309e901fca858a692d5ed928a87f9841b65848b3"
  "sync/bidirectional_stateful_property.riv|c2813f0ad0f5aedff70ec666f21118b41e611ab87951b5192960599c9be82583|e85a11604edd9a2a50bbe2f04da4a91b0293ccd6"
  "sync/paused_nested_artboard_opacity.riv|642c9f7fd909b9955a875e0bb745d0998d3ac4b64a11b863b09e3b0ee5682944"
  "sync/solo_index_test.riv|e857c0d1f76cec0be8d8b9d8308ea9a0f581de29ed752b952940d90b5f6a16f2|38c924123ffb8ad9541ad724ef4de860e5705482"
  "sync/stateful_component_image_test.riv|47dcbcd02cd228f0e4ec71eaac84748f46f95b24737818f61b04d46242b48393|353ef4fccbf6f1801def7d737a4103657dc63a1c"
  "sync/layout_text_match.riv|1fea1a6102259aacd9b164cfac0b4a2f67d4fa4587b78f5eb25a2f195de7bcdb|f5cfee3a5d6a6728167b58a71b47455ace063690"
  "sync/artboard_opacity_and_transform_test.riv|100dbf5c04159ea7e8e6f12ce16daf1ee6f15a74c2d3dc074e2dbde4e877af80|e0d4913fa0f88d9f4b57c53006e7f9712417205f"
  "sync/databind_null_artboard_swap.riv|0160b4572f217271df84072b08476d433a71c5bf78a9917f39fbc03239560a1f|30a0e2d42e2e6d091350d6edb816e165e27f7988"
  "sync/component_list_clipped_viewport.riv|a20c9fd4936c2b7f435011e7afddd276797e95d68b574ec2c914331afd092bac|482b24a188bb9e367e983bf05235761707a89718"
  "sync/vm_listener_fire_event.riv|683a8ed1ad102fa9dd1020d61df301594a9a9fd20b97655c1f0da62e7b994838|482b24a188bb9e367e983bf05235761707a89718"
  "sync/image_computed_transform_bind.riv|17a6d1e6e9f9713cf78d522b96957c21c12d60aa40d54285759bee151c9f4730|15da0652fc10b55ef1fbd32e3e19582c9dc271f2"
  "sync/animated_cubic_participant.riv|877569d0b8b5ab58e697371fd43be3c033685cd3583a0f712b59f5858ba20a17|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/animated_cubic_participant.riv"
  "sync/animated_participant.riv|eb10bb90842825da31b0e459c71a16cea05719935e01dd0200a2bd9bb2809d5d|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/animated_participant.riv"
  "sync/constrained_participant.riv|a75a05c15e81d1d2e7d7ae85a0c1ec5f2593ce602b8ed183eed4e7784f6d421f|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/constrained_participant.riv"
  "sync/display_none_participant.riv|83c6314ec74e5066c55e2d9388a0612838380e2038d09d1888f484c4bc8ded6c|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/display_none_participant.riv"
  "sync/fixed_participant.riv|d529a662869547d15a2343c4b9d4445b7f242d24356ec842588029953f7fa242|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/fixed_participant.riv"
  "sync/grid_2x2.riv|ee808d602a44eb506cc08deba89e2de99caef718627f19e6e98956e81ea83faf|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/grid_2x2.riv"
  "sync/grid_auto_rows.riv|8e79c43f35c7e16cbf49ed0b3613e77e7d3f69dba91ef41201838f771e0703fc|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/grid_auto_rows.riv"
  "sync/grid_participant.riv|39b13e68ee2989784217bdf16165c7f6715642df61df3aa860dfc02d48ed1e84|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/grid_participant.riv"
  "sync/grid_track_types.riv|23c89f128f73a85b8e9d3b1eabbf95d7b8105d2b0979f959afc7374728a98b05|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/grid_track_types.riv"
  "sync/group_participant.riv|39840066719da317c266095bf27aa9ce13a430e646cd5a86de33775f30213f27|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/group_participant.riv"
  "sync/hug_participant.riv|0af0188466f50b7c3925a5074f7ed14d86a9ff5a7fa0a1d9c3732765cdf2439a|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/hug_participant.riv"
  "sync/list_in_group_joins_layout.riv|41a5f6e341b2bb08c33b147ffd58a08d4d896c7c88e56d770f3eedb4c22a5bc9|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/list_in_group_joins_layout.riv"
  "sync/nested_group_participant.riv|a053a2fff417d1a6b5cd1881a9bbddb2e5bd083abd183ebcf3fdc1295535b28a|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/nested_group_participant.riv"
  "sync/solo_participant.riv|563aee63cf2470cf4eb56c152c698450fe6240cf4d7d68e32bf97c592260651d|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/solo_participant.riv"
  "sync/stack.riv|e32a430711df3538b032d2a78e2ff25990db8ab6dd5dd4a25fb244acd1edc9e3|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/stack.riv"
  "sync/scroll_participant.riv|947f21a0b7ac50cb9bf15df34380b7a162219663cec6e262501f9d7959aee20e|1db281b3e82baf850635fd7aa2092920a80b6a2c|layout/scroll_participant.riv"
  "sync/transform_offset.riv|fdbfb14e5aee19354bbc8fef7f51b19545d8cc95728cd5051f0895a3a2565602|1db281b3e82baf850635fd7aa2092920a80b6a2c|layout/transform_offset.riv"
  "sync/transform_offset_legacy.riv|f947326b2a2d5e6b9213fdd3f9869ef414844eaa87e0b5cb7ea1edf9b566addf|1db281b3e82baf850635fd7aa2092920a80b6a2c|layout/transform_offset_legacy.riv"
  "sync/ik_over_distance_constraint.riv|365b2bf2799d9e7ee03699af7bd2152eb8ccdb8a1bcb21162c790bb7b06eee2f|1db281b3e82baf850635fd7aa2092920a80b6a2c|ik_over_distance_constraint.riv"
  "sync/stack_participant.riv|4693b63499749ee3cb8b80bd95ebe6df0e1bffc9a82de225c3e843e8c2609400|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/stack_participant.riv"
  "sync/styled_flex.riv|13e79bd1cba28909ff0ce1e718fe815348e6ed397b1522790b1e7dc95ab7baee|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout/styled_flex.riv"
  "sync/layout_grid_stack.riv|9b4d1c56735f16928396e079d39e62815dce7586842ac9dfd5937a4dca231724|f41cd8f3b1bd6b14442630859d3a7bbba9d16b9c|layout_grid_stack.riv"
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
    # Upstream garbage-collects commits once they fall off every branch, so a
    # recorded source ref can start 404ing long after it vendored the asset.
    # Retry at the pin; the checksum below still decides whether the bytes are
    # the revision we recorded.
    if ! curl --fail --location --silent --show-error \
      "$base_url/$source_ref/tests/unit_tests/assets/$source_path" \
      --output "$destination"
    then
      curl --fail --location --silent --show-error \
        "$base_url/$ref/tests/unit_tests/assets/$source_path" \
        --output "$destination"
    fi
  fi

  actual=$(sha256 "$destination")
  if [[ "$actual" != "$expected" ]]; then
    echo "fixture checksum mismatch: $relative (expected $expected, got $actual)" >&2
    exit 1
  fi

  # Keep S4-42's as-yet-unpinned assets out of the active fuzz seed sets.
  if [[ "$relative" != "sync/data_bind_blob_test.riv" \
    && "$relative" != "sync/data_enum_roundtrip.rml" \
    && "$relative" != command_queue/* ]]; then
    for target in fuzz_import fuzz_runtime fuzz_pointer; do
      seed_dir="$repo_root/fuzz/seeds/$target"
      mkdir -p "$seed_dir"
      cp "$destination" "$seed_dir/$name"
    done
  fi
done

# The GM code consumes the v4 header. The separately added .rstb is an older
# v2 fixture and is retained byte-for-byte, never substituted for that header.
gm_assets=(
  "ore_gm_shaders.rstb.hpp|dda092b3d96973c4d924064e774bcb3428dcf31bfb96d5238aad19c771e7d9da"
  "ore_gm_shaders.rstb|864847b09add07eb906922b696ce397c9b8d158560e67855e61d6676faf26c8f"
)
for entry in "${gm_assets[@]}"; do
  IFS='|' read -r name expected <<< "$entry"
  destination="$repo_root/fixtures/gm/$name"
  mkdir -p "$(dirname "$destination")"
  if [[ -n "$runtime_dir" ]]; then
    git -C "$runtime_dir" show "e949498e05483a852c10fbbdad2cd1941c15aebc:tests/gm/$name" > "$destination"
  elif [[ ! -f "$destination" || "$(sha256 "$destination")" != "$expected" ]]; then
    curl --fail --location --silent --show-error \
      "$base_url/e949498e05483a852c10fbbdad2cd1941c15aebc/tests/gm/$name" --output "$destination"
  fi
  [[ "$(sha256 "$destination")" == "$expected" ]] || { echo "fixture checksum mismatch: gm/$name" >&2; exit 1; }
done

raster_font_destination="$repo_root/fixtures/fonts/sbix.ttf"
raster_font_expected="caf017485804582021c4bf67df4d8e089db5fac7f3e56ef83866ce97b197669c"
raster_font_url="https://raw.githubusercontent.com/google/skia/750673c775648c29002389a3f56fba459288eea9/resources/fonts/sbix.ttf"
mkdir -p "$(dirname "$raster_font_destination")"
if [[ -n "$runtime_dir" \
  && -f "$runtime_dir/skia/dependencies/skia/resources/fonts/sbix.ttf" ]]; then
  cp "$runtime_dir/skia/dependencies/skia/resources/fonts/sbix.ttf" \
    "$raster_font_destination"
fi
if [[ ! -f "$raster_font_destination" \
  || "$(sha256 "$raster_font_destination")" != "$raster_font_expected" ]]; then
  curl --fail --location --silent --show-error \
    "$raster_font_url" \
    --output "$raster_font_destination"
fi
raster_font_actual=$(sha256 "$raster_font_destination")
if [[ "$raster_font_actual" != "$raster_font_expected" ]]; then
  echo "fixture checksum mismatch: fonts/sbix.ttf (expected $raster_font_expected, got $raster_font_actual)" >&2
  exit 1
fi

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

color_silver_destination="$repo_root/fixtures/sync/color_passthrough_test.sriv"
color_silver_expected="6f6482774fb736db12b4fb150b8219237c9c97cc82fd65ba83c69fc349dd76b6"
if [[ -n "$runtime_dir" ]]; then
  git -C "$runtime_dir" show \
    "74c0d601c516f86db4847521198dba42080db06a:tests/unit_tests/silvers/color_passthrough_test.sriv" \
    > "$color_silver_destination"
elif [[ ! -f "$color_silver_destination" || "$(sha256 "$color_silver_destination")" != "$color_silver_expected" ]]; then
  curl --fail --location --silent --show-error \
    "$base_url/74c0d601c516f86db4847521198dba42080db06a/tests/unit_tests/silvers/color_passthrough_test.sriv" \
    --output "$color_silver_destination"
fi
color_silver_actual=$(sha256 "$color_silver_destination")
if [[ "$color_silver_actual" != "$color_silver_expected" ]]; then
  echo "fixture checksum mismatch: sync/color_passthrough_test.sriv (expected $color_silver_expected, got $color_silver_actual)" >&2
  exit 1
fi

global_view_models_silver_destination="$repo_root/fixtures/sync/global_view_models_scripting_test.sriv"
global_view_models_silver_expected="7b3cf99eed9e2d9af8476b761b744e379945220b79ee95f7d59f638121de950e"
if [[ -n "$runtime_dir" ]]; then
  git -C "$runtime_dir" show \
    "309e901fca858a692d5ed928a87f9841b65848b3:tests/unit_tests/silvers/global_view_models_scripting_test.sriv" \
    > "$global_view_models_silver_destination"
elif [[ ! -f "$global_view_models_silver_destination" \
  || "$(sha256 "$global_view_models_silver_destination")" != "$global_view_models_silver_expected" ]]; then
  curl --fail --location --silent --show-error \
    "$base_url/309e901fca858a692d5ed928a87f9841b65848b3/tests/unit_tests/silvers/global_view_models_scripting_test.sriv" \
    --output "$global_view_models_silver_destination"
fi
global_view_models_silver_actual=$(sha256 "$global_view_models_silver_destination")
if [[ "$global_view_models_silver_actual" != "$global_view_models_silver_expected" ]]; then
  echo "fixture checksum mismatch: sync/global_view_models_scripting_test.sriv (expected $global_view_models_silver_expected, got $global_view_models_silver_actual)" >&2
  exit 1
fi

echo "test assets ready (rive-runtime@$ref)"
