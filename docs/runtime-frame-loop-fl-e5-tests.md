# FL-E5 W65 test disposition

W65 assigns 22 `layout_test.cpp` cases, 3 `bounds_test.cpp` cases, 17
`text_test.cpp` cases, 2 `text_modifier_test.cpp` cases, and the nested-text
cases to this SCC. This change ports one direct bounds differential plus focused
callback tests; the golden and silver fixtures exercise a wider cross-section,
but they do not reproduce every assigned C++ fixture action and assertion.
Consequently the W65 load is **partial**, and no test expectation or comparison
tolerance was weakened.

| W65 class | Upstream source | FL-E5 evidence |
|---|---|---|
| B | `bounds_test.cpp` (3 assigned) | One direct differential, `upstream_text_local_bounds_fixture_retains_origin_adjusted_bounds`, ports the `Text1`/`Text2` retained local-bounds assertions from `local_bounds.riv`, including origin adjustment. The normal golden corpus also contains `background_measure.riv` and `local_bounds.riv`; the other assigned assertion sequences remain to be ported directly. |
| B | `layout_test.cpp` (22 assigned) | The existing E4 literal covers `measure_tests.riv`; FL-E5 adds owner-local padding/style and TextStyle callback tests. The silver corpus executes `collapsing_elements`, `layout_display`, `layout_paint`, `layout_anim_bound`, `layout_anim_component_list`, `layout_anim_nested`, `layout_aspect_ratio`, `layout_fixed_fill`, and `layout_hug_artboard`. These are fixture-stream evidence, not substitutes for all 22 direct test bodies. |
| B | `text_test.cpp` (17 assigned) | Normal goldens cover `double_line`, `ellipsis`, `modifier_to_run`, `new_text`, `vertical_align_ellipsis`, `word_joiner_test`, and `zero_width_space_line_break`; runtime units cover shaping, line metrics, modifiers, opacity buckets, hit/caret geometry, and retained bounds. Direct parity for all 17 assigned test bodies remains open. |
| B | `text_modifier_test.cpp` (2 assigned) | `text_feather_falloff` remains in both golden/silver inventories; modifier-to-run and opacity/falloff behavior are covered by runtime text unit tests. The two assigned C++ assertion sequences are not both direct ports. |
| B | `serialized_rendering_test.cpp` text/layout cases | `saturation`, `spotify_kids_app_icon`, and `text_stroke_test` promote from divergent to exact. Other requested signatures remain classified at their unchanged first mismatch. |

The following are deliberately not claimed as closed by this family:

- TextInput editing/cursor/selection tests belong to the input-text family.
- The E5 map contains 27 direct C++ owners. Nine have new direct Rust modules;
  `layout.cpp`, `solo.cpp`, and 16 text owners still rely on consolidated runtime
  implementations rather than verified file/member correspondence.
- `Text::buildRenderStyles` same-update layout publication remains recorded in
  FL-G07.
- The three `layout_anim_*` point-value mismatches are retained layout geometry
  interpolation differences, not hidden as callback-chain test passes.
