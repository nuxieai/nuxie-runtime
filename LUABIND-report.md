# LUABIND lane report

## Status

- Lane: `levi/endgame-lua-bindings` (#LT-2 remaining Lua bindings).
- Preflight base: `fb8b7afd0720eeea9563877247ad006c23b00e63`; before any work, both `HEAD` and the then-current `origin/main` resolved to this commit.
- Pinned C++ oracle: `/Users/levi/dev/oss/rive-runtime` at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- Fixture preflight: `rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/` followed by `make fixtures` passed against the pin.
- Current result: the five #LT-2 correspondence rows are promoted to `faithful` / `pending-verification`. The generated scorecard moves C++→Rust correspondence from 14 to 9 pending rows.

`origin/main` advanced to `8a1aea1cb0fe8b6a0d8efab069259e360422071a` after the verified preflight while this parallel lane was running. This lane intentionally remains based on the verified `fb8b7afd` starting point for orchestrator landing.

## Port evidence

### `src/lua/lua_data_value.cpp`

- `lua_data_value.rs` ports the pinned Lua check/coercion rules for number, string, boolean, color, and channel setters.
- Non-color channel reads produce the named `DataValue` index diagnostic; non-color channel writes and unknown assignments are silent.
- Signed/spill channel writes and unsigned color wrapping match the pinned C++ operations.
- Direct test: `scripted_data_values_match_lua_check_coercion_and_index_semantics`.

### `src/lua/lua_properties.cpp`

- The retained runtime view-model facade now exposes authored enum properties, key reads/writes, and enum value collections.
- Scalar, enum, nested-view-model, list, image, font, blob, and trigger properties expose pinned listener registration/removal. Listeners run in reverse registration order only after runtime/userdata borrows are released (synchronously after Lua mutators and before host facade setters return), retain subscriptions across userdata GC until removal, accept optional userdata, and ignore callback failures.
- Property number/string/color setters use the same pinned Lua coercion/wrapping paths as the C++ binding.
- Direct and named getters share the same cached property userdata, including cross-access listener removal. Setter-bearing properties ignore unknown string keys and reject non-string keys; trigger/list userdata retain their pinned assignment errors. Blob `.value` retains userdata identity across unchanged reads and invalidates it on mutation; trigger dispatch continues past callback failures.
- Direct tests: `property_listeners_survive_userdata_gc_while_subscribed`, `scripted_string_boolean_and_enum_properties_match_upstream_luau_access`, and `scripted_color_property_supports_direct_and_named_access`, plus existing focused list, image, blob, font, and nested-view-model tests.
- Test ledger ratchet: `scripting_properties_test.cpp` remains `partial`, increasing covered cases from 2/22 to 14/22 without weakening the pending floor.

### `src/lua/lua_state.cpp`

- `Data.<ViewModel>.new` now dispatches exactly like the pin: one nil/string-or-number/other argument selects fresh/named/nil behavior, while zero or multiple arguments create a fresh instance.
- Direct test: `data_constructor_matches_pinned_argument_count_and_type_dispatch`.

### `src/lua/renderer/lua_gradient.cpp`

- Gradient stop iteration terminates at the first non-table.
- Position and color fields use pinned number/unsigned coercion, including wrapping colors.
- Direct tests: `gradient_stops_end_at_the_first_non_table_and_wrap_unsigned_colors` and `scripted_draw_can_allocate_and_apply_gradients`.
- The one-case upstream `scripting_gradient_test.cpp` row is promoted to `ported-direct`.

### `src/lua/rive_lua_libs.cpp` and #LT-2 diagnostic

- The corpus-gated umbrella installs the completed binding equivalents before sandboxing and remains idempotent.
- `unported_context_binding_reports_the_script_and_binding_names` loads `lt2-unported-animation.luau`, touches still-unported `Context:animation`, and requires both the script name and binding name in the runtime diagnostic.
- `docs/parity-closeout-status.md` marks the #FT diagnostic spot-check complete.

## Focused lane gate

Passed:

- `cargo check -q -p nuxie-binary -p nuxie-runtime -p nuxie-scripting`
- Exact `nuxie-scripting` tests for DataValue coercion/index semantics, Gradient stop decoding, Data constructor dispatch, the named unported-binding diagnostic, property listener GC retention, scalar/enum/color property access, umbrella install ordering, and install idempotence.
- The focused `vm::view_model::tests` slice: 27 passed, 1 pre-existing ignored, covering list/image/blob/font/view-model property regressions plus borrow-safe scalar/trigger listener dispatch.
- Exact generator test `test_generate_promotes_lt2_lua_binding_rows`, covering all five LT-2 classifications and Rust owners.
- `make parity-scorecard` (including its 26 ledger/scorecard tests).
- `git diff --check`.

Per lane instructions, no full workspace batteries and no golden comparisons were run. Cargo emitted the repository's existing warning volume; no new compile error remains.

`make port-manifest-test` still has four shared, pre-existing stale assertions that expect `src/command_queue.cpp` / `src/command_server.cpp` to be `absent` even though the generator classifies those non-LT-2 rows `partial`. The LT-2 generator test passes; the unrelated shared assertions are queued below rather than changed in this lane.

## Residue, scatter, and pins

- No new or moved source file exists, so no four-place source residue entries are required.
- File-correspondence scatter is 153 rows, below the lane cap of 155. The existing crate-boundary exceptions for `lua_properties.cpp` and `rive_lua_libs.cpp` remain documented in their row notes.
- The Luau engine dependency/pin was not changed.
- No tolerance was added or widened.
- Fixtures were synchronized for local evidence only and remain gitignored; no fixture addition is part of the lane diff.

## Queued for the landing orchestrator

- Reconcile the post-preflight `origin/main` advance before landing this parallel lane.
- Run the orchestrator-owned full batteries and golden comparisons, then promote the five rows from `pending-verification` only if those landing gates pass.
- Reconcile the four stale command-queue/server expectations in `tools/port-manifest/test_port_manifest.py`; this lane updated only the LT-2 generator rules and exact ownership expectations.
- The broad composite `scripted properties can be passed to luau` case and seven other unlisted cases in `scripting_properties_test.cpp` remain pending in the test ledger; the row deliberately remains `partial`.
- `src/lua/lua_scripted_context.cpp` remains pending and supplies the intentional named-diagnostic spot-check target. `src/lua/renderer/lua_gpu.cpp` is also outside this lane and remains pending.

## Commits

- `58a077bb` — Port remaining Lua binding semantics
- `70eacf93` — Promote LT-2 Lua binding ledger rows
- `c3371549` — Retain subscribed property listener watches
- `fae9cf29` — Preserve authored Lua enum keys
- `428e0709` — Patch DataValue metatable once
- `9b1f47ef` — Dispatch Lua property listeners after mutation
- Final property-fidelity and generator/ledger/report closeout commits — blocked by the linked-worktree index-lock sandbox; exact per-commit file/hunk staging maps are in `LUABIND-map.md`.
