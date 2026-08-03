# S4C ordered commit reconstruction map

Git cannot stage or commit in this managed worktree because the linked index is
outside the writable root:

`fatal: Unable to create '/Users/levi/dev/nuxie-runtime/.git/worktrees/nuxie-mr-c11/index.lock': Operation not permitted`

One anomalous `git add .s4-deferred-corpus.toml` succeeded after the final
validation, then the sandbox resumed rejecting every index write (including
five `git restore --staged` retries). Before reconstructing commits, unstage
`.s4-deferred-corpus.toml`; its two entries must be split between S4-21 and
S4-23 as described below. No other path is staged. Reconstruct commits in the
order below.

## S4-1 — `1b4df2ad`

Commit message:

`[sync] Port rive-runtime 1b4df2ad: chore: Textinput improvements (#13130) e1b960c043`

Stage the S4-1 changes described below:

- `crates/nuxie-runtime/src/components.rs`
  - Add clone-cold `RuntimeTextInputState::is_focused`, defaulting to false.
- `crates/nuxie-runtime/src/text/raw_text_input.rs`
  - Add `clear_selection`, collapsing to the selection end.
  - Add the adapted upstream regression for selection collapse and its no-op case.
- `crates/nuxie-runtime/src/text_input.rs`
  - Synchronize retained TextInput focus state, collapse selection on blur, mark paint dirt, and expose the tools/debug focus projection.
- `crates/nuxie-runtime/src/text/text_input_cursor.rs`
  - Return no cursor path while the owning TextInput is unfocused.
- `crates/nuxie-runtime/src/text.rs`
  - In `runtime_text_input_shape_paint_commands`, skip the entire
    `TextInputCursor` paint-command path while unfocused, preserving upstream
    null-path behavior and paint-resource order. This is the only S4-1 hunk in
    this file; all other current `text.rs` hunks belong to S4-21.
- `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`
  - Focus a TextInput through its direct FocusData child without confusing target and FocusData local IDs.
  - Synchronize TextInput focus after pointer dispatch and state-machine advance.
  - Stage every current hunk except `normalized_hit_position` near lines
    5789-5804; that one hunk belongs to S4-23.

No schema, generated-code, fixture, corpus, golden, pin, or `Cargo.lock` change
belongs to S4-1. The upstream `.rive_head` pointer changed, but the TextInput
runtime defs/generated files are byte-identical across this commit.

Required gates: all PASS (`cargo test -p nuxie-runtime`;
`cargo test -p nuxie --features scripting`;
`make runtime-frame-loop-port-check`; `make rust-attribution-check`). The exact
candidate `text_input` differential also passes.

## S4-21 — `f5cfee3a`

Commit message:

`[sync] Port rive-runtime f5cfee3a: chore: Ensure text sizes to its parent layout with min/max sizes applied (#13223) 61c50c6f87`

Stage this whole file:

- `fixtures/sync/layout_text_match.riv`
  - Force-add this ignored binary fixture. SHA-256:
    `1fea1a6102259aacd9b164cfac0b4a2f67d4fa4587b78f5eb25a2f195de7bcdb`.

In `.s4-deferred-corpus.toml`, stage only the `layout_text_match` deferred
entry. The following `artboard_opacity_and_transform_test` entry belongs to
S4-23.

In `tools/fetch-test-assets.sh`, stage only the checksum-pinned
`layout_text_match.riv` row at upstream
`f5cfee3a5d6a6728167b58a71b47455ace063690`. The adjacent Artboard fixture row
belongs to S4-23.

In `crates/nuxie-runtime/src/draw.rs`, stage only the
`runtime_parent_layout_content_bounds` hunk near lines 6796-6825. It passes the
parent LayoutComponent content box (solved border box minus padding) to Text,
matching `LayoutComponent::propagateSizeToChildren`. Every other current hunk
in this file belongs to S4-23.

In `crates/nuxie-runtime/src/text.rs`, stage every current hunk except the
S4-1 `runtime_text_input_shape_paint_commands` / `TextInputCursor` hunk near
line 758. The S4-21 hunks are:

- allow controlled auto-sized Text to retain fixed constraint bounds,
- use fixed-sizing line iteration for layout-controlled overflow while keeping
  authored sizing for unconstrained text,
- use the controlled width and height for controlled local bounds,
- add `StaticTextSlice::overflow_as_fixed`,
- preserve controlled bounds for unshaped text,
- enable clipping, ellipsis, fit-font-size, fit transform, and vertical
  alignment for layout-controlled auto-sized text,
- add the controlled-height assertion to
  `layout_measure_uses_authored_sizing_before_controlled_bounds`.

No schema or generated files changed. The fixture is deferred only; no
`corpus.toml`, existing golden, pin, `Cargo.lock`, Taffy fence, or Luau pin
change belongs to S4-21.

Required gates: all PASS (`cargo test -p nuxie-runtime`;
`cargo test -p nuxie --features scripting`;
`make runtime-frame-loop-port-check`; `make rust-attribution-check`). Both
triage-attributed scripted candidate differentials (`data_binding_test` and
`data_viz_demo`) pass; ordinary `data_binding_test` passes, and ordinary
`data_viz_demo` is correctly scripted-only.

## S4-23 — `e0d4913f`

Commit message:

`[sync] Port rive-runtime e0d4913f: feat(runtime): add opacity and transform (rotation/scale) support on … (#13224) 76284ae1ea feat(runtime): add opacity and transform (rotation/scale) support on artboards`

Stage these whole files:

- `crates/nuxie-runtime/src/artboard.rs`
  - Add clone-owned `host_opacity`, kept separate from authored Artboard
    opacity, plus `child_opacity`, `has_self_transform`, and `self_transform`.
  - Propagate host opacity through root children and retained root paints.
  - Include nested Artboard self transforms in child root transforms.
  - Replace nested opacity writes to the authored property with host-opacity
    updates and adapt the deep-settlement tests to preserve authored opacity.
  - Add direct host-opacity and rotation/scale regressions.
- `crates/nuxie-runtime/src/data_bind/data_bind_context.rs`
  - Call the Artboard-owned mounted-root-transform seam while collecting and
    advancing nested data-binding contexts.
- `crates/nuxie-runtime/src/focus.rs`
  - Call that same seam while projecting nested and component-list focus
    descriptors into root space.
- `fixtures/sync/artboard_opacity_and_transform_test.riv`
  - Force-add this ignored binary fixture. SHA-256:
    `100dbf5c04159ea7e8e6f12ce16daf1ee6f15a74c2d3dc074e2dbde4e877af80`.

In `.s4-deferred-corpus.toml`, stage only the
`artboard_opacity_and_transform_test` entry. The preceding
`layout_text_match` entry belongs to S4-21.

In `tools/fetch-test-assets.sh`, stage only the
`artboard_opacity_and_transform_test.riv` row at upstream
`e0d4913fa0f88d9f4b57c53006e7f9712417205f`. The adjacent
`layout_text_match.riv` row belongs to S4-21.

In `crates/nuxie-runtime/src/draw.rs`, stage every current hunk except the
S4-21 `runtime_parent_layout_content_bounds` padding/content-box hunk near
lines 6796-6825. The S4-23 hunks are:

- use effective Artboard child opacity for preparation and draw early exits,
- mirror top-level and mounted Artboard self transforms in retained geometry
  bounds and hit queries,
- include mounted Artboard self transforms while recursively traversing nested
  and component-list geometry,
- apply host opacity to fallback mounted occurrences without overwriting the
  authored Artboard property,
- save for and apply Artboard rotation/scale after frame-origin translation,
- clip after both transforms with the retained local Artboard path and reuse
  its backend owner for the root background paint,
- multiply root Artboard paint opacity by host opacity, and
- include each component-list child Artboard's self transform in its root
  transform, and
- add adapted draw/clip/opacity and generic geometry-hit regressions.

In `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs`, stage
only the `normalized_hit_position` hunk near lines 5789-5804, which inverts the
Artboard self transform after frame-origin normalization. All other current
hunks in this file belong to S4-1.

No schema or generated files changed at `e0d4913f`; `.rive_head` alone moved
upstream. No existing golden, `corpus.toml`, product pin, `Cargo.lock`, Taffy
fence, or Luau pin belongs to S4-23.

Required gates: all PASS (`cargo test -p nuxie-runtime`;
`cargo test -p nuxie --features scripting`;
`make runtime-frame-loop-port-check`; `make rust-attribution-check`). Exact
candidate attribution checks pass for all 96 ordinary entries (279 segments)
and all 99 scripted entries (286 segments), using the triage list verbatim.

The required full candidate `make scripted-golden-compare` completed all 321
entries/657 segments and retained four out-of-set failures: `scope_probe`
(S4-3 sibling), `echo_show_demo` (S4-30 sibling), and
`data_binding_test`/`data_viz_demo`, whose final-cut layout positions require
the concurrently landing S4-38 tranche. `bankcard` and `death_knight`, which
failed in the earlier pre-review full run, are exact in the final rerun. No
S4-23-attributed entry failed.
