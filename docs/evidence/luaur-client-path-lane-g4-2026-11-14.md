# Luau on clients, lane G4: runtime value notifications and measured charges (2026-11-14)

Branch `claude/luaur-lane-g4-runtime-233d82`, based on `levi/luaur-client-stage1c`
(`32db50aba8`). Owned files: `crates/nuxie-runtime`, and in
`crates/nuxie-scripting/src/vm/` `lua_path.rs` and `lua_text.rs`. One test-only
commit in G3's `host_memory_budget.rs`, with the coordinator's agreement (see
"Edits outside the lane").

## Commits

| Commit | Concern |
| --- | --- |
| `cac641588e` | Part 1: a value's delegates run after its Core borrow ends |
| `49b7c0246d` | Part 2: delegates that are gone leave the list as others are added |
| `01f67db6ce` | Part 3c: RawPath reports capacity, reserves fallibly |
| `d83365939f` | Part 3b: measures report table sizes, admit each contour, bound an extract |
| `d87206ade2` | Part 3a: RawText reports the bytes its content and layout hold |
| `d193c91279` | Part 3b/c: script paths and measures are charged what they hold, before they grow |
| `aaac93402a` | Part 3a: a script Text is charged what its content and layout hold |
| `3e8f925a5e` | G3 tests: Text charges checked against the reported size (tests only) |
| `2fb4dcd042` | Review fix: a delegate cycle stops at the value that started it |
| `84380833de` | Review fix: the render copy is counted at the commands' capacity |
| `b73f93e5e3` | Review fix: a refused font list gives the content charge back |
| `4d537294e9` | Review fix: a skipped contour gives its charge back before the next |

## Part 1: the listener borrow panic

### Reproduction

`delegate_release_tests.rs` writes each value type the way a to-source data
bind writes a non-number source: through the borrowed setter inside
`with_downcast_mut` (string, color, boolean, enum, symbol list index, artboard,
image, font, blob, trigger, number). The delegate reads its own value back
through `CoreHandle::with`, as the scripted `value` getter does. On the old code
3 of the 4 tests failed with `RefCell already mutably borrowed` at
`core.rs:1057` (the fourth, a value with no Core slot, never borrowed).

### What I chose, and why

Neither of the two options in the brief as stated. Both funnel to one place:
every value type reaches `ViewModelInstanceValue::on_value_changed` (or its
handle form), so the fix is at that one place, not per type.

* **Handle-level setters for every type** would have needed a handle setter for
  each of the 11 types and every caller to use it. Any caller that still used a
  borrowed setter would panic again, so the shape would not make the panic
  impossible.
* **Deferring only while a host mutation is in progress** covers armed writes,
  but the panic also reproduces for unarmed borrowed writes. D3's direct
  `with_downcast_mut(..set_value)` is one. So is any non-script delegate that
  reads its value.

What landed: every delegate notification goes through
`dispatch_value_notification` (`host_viewmodel/cell.rs`):

1. inside a host mutation or transaction, it joins that buffer (as before for
   transactions);
2. otherwise it runs once the outermost Core borrow on the thread ends. This uses
   the queue `BorrowScope` already drains for picture-mode layout refreshes
   (`picture_layout_mode.rs`, new `Refresh::Released`);
3. a value with no Core slot (detached) notifies at once, as before.

So no delegate can run while any Core slot on the thread is mutably lent. That
covers every value type and every caller. No cost is added to `with_mut`: the
borrow counter already existed.

One correction came from codex review round 1. Deferring had let A's
suppression end before the notification A's delegate raised on B ran, so
A → B → A fed back. The test probe ran 11 A and 10 B notifications before its
stop. Notifications raised while a value's delegates run now queue on that
delivery (`Delivering` in `viewmodel_instance_value.rs`). They run before its
`SuppressDelegation` drops, so the cycle stops at A, as it did inline. Test:
`a_delegate_cycle_stops_at_the_value_that_started_it` (11/10 on the code
before the fix, 1/1 after, both unarmed and inside a host mutation).

### Order and count

* The count is unchanged: each change notifies each live delegate once. The tests
  assert exactly one call per type, both unarmed and in a host mutation.
* The order changes in one respect. A delegate raised under a Core borrow now
  runs when the borrow ends, not inside the setter. A notification raised during
  another delivery now runs after that delivery's delegate list, not nested
  inside the delegate that raised it. Writes made in an armed host mutation
  without publication now notify at commit, not inline. This affects
  `RuntimeValueChangeTracker` and test observers; `NativePropertyDelegate`
  already deferred there and is now mostly redundant. I left it in place.
* Both golden compares are unchanged (results below), as are the data-binding
  suites in `nuxie` (only the known `stateful_component_image_bind` red).

### D3's claimed path

`NativePropertyDelegate::value_changed` (`scripting/native_view_model.rs`)
already deferred through `defer_host_mutation_notification` when the buffer was
armed. So the script-listener path through `native_binding.rs` `mutate` was
already protected. The panic was real for unarmed borrowed writes and for
non-script delegates. `mechanical_port/source/lua/lua_properties.rs` is not
compiled.

## Other paths that run script code under a Core borrow (not fixed; report)

Method: temporary instrumentation (reverted, never committed). In
`Function::call_with`, write a backtrace whenever the thread's open Core borrow
count is above zero. Run `nuxie-scripting` and
`nuxie --features scripting,test-support`, then group the backtraces by the
script entry point and the frame that holds the borrow. Any open borrow is a
hazard if the script can reach that object. None of these panicked in the
suites, but each would if the script read the borrowed object.

| Hits | Test | Script entry | Borrow held by |
| --- | --- | --- | --- |
| 964 | upstream_wave_a_final | `ScriptViewModel::property_listener_sink` (via `invoke_before_notify`) | `StateMachineLayerInstance::perform_listener_actions` |
| 7 | audio_core | `property_listener_sink` | `StateMachineInstance::perform_listener_changes` |
| 248 | upstream_scripted_path | scripted path effect `call_path_effect_update` | `ShapePaint::update_with_path_kind` (`with_mut` of the paint) |
| 41 | scripted_layout | `ScriptingVm::call_layout_resize` | `intrinsically_sizeable::control_size_handle`; also `ScriptBackend::set_display_scale` |
| 12 | upstream_scripting_transition_condition | `ScriptedObject::evaluate_condition` | `StateTransition::allowed` |
| 5+3 | upstream_solo | `ScriptAsset::instantiate_for_occurrence`, `ScriptedObject::hydrate_occurrence` (init) | `CoreHandle::clone_occurrence_into` |
| 8 | upstream_scripting_listener_action | `ScriptedObject::perform_listener_action`, `ScriptedListenerAction::perform_stateful` | `StateMachineInstance::perform_listener_changes` |
| 2+ | upstream_scripting_interpolator, unit | `ScriptedObject::call_interpolator` | `DataConverterInterpolator::transform_factor`, `InterpolatingKeyFrame::transform`, `StateMachineLayerInstance::apply`, `LinearAnimationInstance` (`with_downcast_mut`) |
| 338+104 | transition_condition, wave_a_final | `vm::view_model::patch_property_userdata` called from `create_scripted_view_model_retained` | an outer Core borrow around the view-model wrapper creation |

The most exposed rows:
* The `property_listener_sink` row. A script property listener runs while the
  state machine layer instance is borrowed. A listener that reaches the state
  machine (or anything borrowed up that stack) would panic.
* The scripted path effect and scripted layout rows. The script's own host
  object (the paint, or the sizeable that owns the scripted layout) is borrowed
  while it runs.

A second hazard found by reading, not exercised:
`DataBind::update_source_binding_handle` calls
`context.borrow_mut().apply_to_source(..)`. That `apply_source_value` → mutate
commit runs script listeners while the `context` `RefCell` is borrowed. A
listener that reaches the same data bind context re-enters it.

These need the same treatment as Part 1 at each entry point: run the script
after the borrow ends, or pass the data instead of the borrow. That is a larger
change than this lane's brief, so I stopped at reporting it.

## Part 2: unbounded delegate list

`add_delegate` pushed without pruning, and every scripted watch
(`property_dirt_sink`) registers a delegate. When the list is full, it now
first drops delegates whose `Rc` is gone, then reserves room for as many again
as are live. Each scan is paid for by the adds since the previous one: amortised
O(1) per add. The list stays within twice its live size. No timers or sweeps.
Test: 10,000 watch-and-drop cycles leave capacity ≤ 4 and the kept delegate is
notified once.

## Part 3: charges

Where the size is known in advance, the charge comes before the allocation.
That covers path growth, a measure's tables per contour, the contour iterator's
copy, an extract (bounded by its interval) and a Text's content on append.
A Text's layout is known only after `update` builds it, so it is charged
immediately after the build, before anything reads it. A layout over the budget
is an error, and the layout is released again.

### Measured (printed by the tests, Bazel fastbuild, fixture font IBM Plex Sans Arabic)

| What | Before | After |
| --- | --- | --- |
| Text, 10,000 ASCII characters, laid out | 240,320 bytes (24/input byte + 64 + 256) | 5,025,360 bytes (`RawText::heap_bytes`) |
| Path, 1,000 `lineTo` | 27,027 bytes (3 × len estimate) | 18,234 bytes (capacity + one render copy) |
| `Path:measure()`, 200 cubics | 0 (uncharged) | 93,712 bytes |
| `Path:contours()`, iterator + first contour | 0 | 99,519 bytes; second (2-point) contour 100 bytes |

The Text figure is higher than the reviewer's 2.78 MB. Two reasons: style
paths' render copies are counted at the commands' capacity (review fix
`84380833de`), and the glyph lookup and draw commands are counted. Not counted:
the emoji image cache, fonts (charged on their own), and the backend's real
render path size. The backend does not report it, so one copy of the commands
stands in.

### What scripts now observe differently

* A Text, Path or measure hits the host memory budget error at its real size.
  Large Texts error much earlier than before (about 21× the old charge for
  ASCII). A path errors a little later than before (capacity + one copy instead
  of 3 × len).
* `Path:measure()` and `Path:contours()` can now fail with the budget error.
  Before they were free. A measure over the budget is refused before its tables
  are built. A contour iterator charges its path copy, and then each contour as
  it is produced.
* An extract (`extract`/`getSegment`) reserves only what its interval can
  append (`segment_room`), not the whole source.
* A Text read (bounds, lines, glyphs, draw) whose layout does not fit raises the
  budget error and drops the layout. Every later layout read errors the same
  way until the content shrinks or the budget allows it.
* A failed append (content or font list) leaves the Text and the charge as they
  were.

## Edits outside the lane

* `crates/nuxie-scripting/src/vm/host_memory_budget.rs`, tests only, commit
  `3e8f925a5e`, agreed with the coordinator. Two tests hard-coded the old Text
  estimate:
  `text_append_stops_at_the_budget_and_clear_keeps_the_charge` and
  `a_font_stays_charged_while_anything_keeps_its_face`. They now compare against
  `ScriptedText::held_bytes()` (a `#[cfg(test)]` accessor in `lua_text.rs`)
  plus the font list. The first test's message changed from "a clear keeps the
  high-water charge" to "the charge is what the text reports holding": a clear
  now refunds down to what the text still holds, which keeps its capacity.

## Changes needed in other lanes (not made)

* **G3, `lua_font.rs`** (codex rounds 1 and 2, P1). When a decoded font is
  assigned to a view-model font property, only the `ScriptFont` is retained, not
  its charge. Collecting the original userdata refunds the bytes while the
  property still owns the face. Reading the property back returns an uncharged
  wrapper. Codex's probe decoded a second face under a one-font budget.
* **G3, `lua_font.rs` `with_options`** (codex round 4). Each derived font
  clones and keeps its axis and feature maps but shares only the face's charge.
  Codex kept twelve variants built from a 10,000-entry feature table under a
  1 MB budget.
* **G3, `lua_artboards.rs` `addToPath`** grows a path through
  `with_render_raw_path_mut`, uncharged. It should reserve through
  `RawPath::try_reserve` after charging, as `lua_path.rs` `reserve` does.

## Checks

All with `NUXIE_EXECUTION=local`, `RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3`,
and `RIVE_CPP_PROBE` pointing at the worktree's `make cpp-probe` build (without it, the
`*_cpp_probe` and `state_holds` targets fail with "the pinned C++ oracle is missing").

At `b73f93e5e3` (every runtime change; the last commit touches only `lua_path.rs`):

| Check | Result | Load average (1/5/15) at start → end |
| --- | --- | --- |
| `panic-lints.sh` (default rows) | PASS; nuxie-scripting 0 forced sites native and wasm | 16.5 → 10.1 |
| `unsafe-count.sh` | PASS, +0 everywhere | 10.1 |
| nuxie-runtime panic lints vs base `32db50aba8` (scratch baseline) | 5043 = 5043, every lint +0 (tip `unwrap_in_result` 48, base 48) | 16.5 → 14.7 |
| `runtime.py test -p nuxie-runtime` | 256/256 targets pass | 10.1 → 7.3 |
| `runtime.py test -p nuxie-scripting` | 39/39 targets pass | 8.5 → 13.2 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41/42; only `stateful_component_image_bind` (known red) | 13.2 → 9.0 |
| `make scripted-golden-compare` | entries=364 exact=348 diverges=16 | 9.0 → 8.1 |
| `make golden-compare` | entries=364 exact=356 diverges=8 | 8.1 → 6.2 |

At tip `4d537294e9`: `panic-lints.sh` PASS, `unsafe-count.sh` PASS,
`nuxie-scripting` 39/39, `nuxie` 41/42 (same known red),
`make scripted-golden-compare` exact=348 diverges=16 (load 6.7 → 7.4).
`make golden-compare` was not rerun at the tip: the last commit changes only
script contour charging, which unscripted content does not reach.

Every fix's test was run red on the code before it: Part 1 (3 of 4 failed at
`core.rs:1057`), the delegate cycle (11/10 vs 1/1), the render copy count
after a rewind, the font-list rollback (226,792 vs 226,004 bytes held), and the
skipped contour ("exceeded its 413-byte host memory budget"). The Part 2 and
Part 3 tests cannot compile against the old code, which lacks the APIs they
call.

### codex review (`--base levi/luaur-client-stage1c`), 4 rounds

1. P1 delegate cycle (fixed `2fb4dcd042`); P2 quadratic contour byte total in
   `from_path_admitted` (fixed, folded into `d83365939f`); P2 Text recounted on
   every read (fixed: `RawText::lay_out` returns whether it built anything,
   folded into `d87206ade2`/`aaac93402a`); P2 `lua_font.rs` (G3, reported).
2. P1 layout admitted after it is built (not changed, see "Not verified");
   P1 `lua_font.rs` (G3); P2 font-list rollback (fixed `b73f93e5e3`); P2 render
   copy after a rewind (fixed `84380833de`).
3. P1 layout (as above); P1 `lua_font.rs` (G3); P2 skipped contour's charge
   (fixed `4d537294e9`).
4. P1 layout (as above); P1 `lua_font.rs` twice (G3: property round trip, and
   derived fonts' option storage from `with_options` uncharged). Nothing new in
   this lane's files.

## Not verified

* That a layout over the budget cannot exhaust device memory before the check:
  it cannot be guaranteed. The layout is built before its size is known, so its
  allocation exceeds the budget once before the error (codex round 2: 2.75 MB
  past 600 KB of headroom). Admitting the layout's growth fallibly while it is
  built would need `update`'s shaping and line breaking to be budget-aware. That
  is a larger change; I report it rather than build it.
* The quadratic fixes from codex round 1 (a running contour byte total in
  `from_path_admitted`; a Text is recounted only when its layout was rebuilt)
  have no test that fails on the old code, because neither is observable except
  as time. Tests are flaky under the machine's load, so I did not add a timing
  test.
* The backend's real render path sizes are approximated by one copy of the
  commands.

## Where the design was wrong

* Deferring delivery dropped the delegate-cycle guard (fixed in `2fb4dcd042`).
* The first `try_next` returned `Result` around the existing `try_into().unwrap()`
  conversions. That added 4 `clippy::unwrap_in_result` sites in nuxie-runtime
  (52 vs 48). The admission now returns a bool through an `Option`-returning
  step, back to 48.
* `ShapePaintPath::heap_bytes` first counted the render copy at the commands'
  length, which a rewind lowers while the backend path is kept.
* Settling a Text after every read made line-by-line reads quadratic.
