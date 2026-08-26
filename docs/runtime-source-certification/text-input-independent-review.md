# TextInput source-pair independent semantic review

Verdict: **REJECTED — the candidate's overall non-parity conclusion is sound,
but six row classifications and four owner locators are not yet reliable**

Reviewed candidate: `27ef8f90b9b18a45dcbbd5fd10afd5e4352d9ad8`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

The complete pinned `.cpp` and handwritten header were read independently
before checking every concrete Rust owner cited by the candidate. Their bytes
and denominator are correct: `text_input.cpp` has 37 C++ function definitions
plus one `EM_JS` definition; `text_input.hpp` has five executable inline
methods. The include guard is correctly excluded. The candidate also correctly
identifies the major missing behavior: wasm Windows modifier detection,
unconditional callback dirt, multiline Enter's dirt family, initial multiline
setup and missing-style result, production world position/bounds accessors, and
unconditional TextInput keyboard acceptance.

The candidate is not accepted as the source-pair inventory until the following
narrow corrections are made. These are classification/evidence corrections;
they do not authorize production fixes.

## Required corrections

1. **Row 17, `syncSourceTextFromRaw`, is not exact.** Pinned lines 275–290
   strip a single-line raw value and write the corrected value back through
   `m_rawTextInput.textPreserveCursor(...)` before moving it to the source and
   invoking `text(...)`. Rust `sync_text_input_source_from_raw` at lines
   317–334 computes the stripped source but never writes it back to the raw
   editor. Classify this as missing/incomplete and retain the exact
   raw-write-before-source-before-property-callback order as the correction
   contract.

2. **Row 31, `worldToLocalWithViewport`, is not an exact algorithm mapping.**
   Pinned lines 564–571 zero both retained scroll velocities before attempting
   inversion. Rust `text_input_move_cursor_to_world_with_auto_scroll` at lines
   618–630 returns for a missing graph or singular transform before it writes
   the local zero values back at lines 679–685. A failed conversion can
   therefore preserve stale edge-scroll velocity. Classify the row as
   incorrect, and carry this failure-order discrepancy into rows 32, 33, and
   37, which delegate to it.

3. **Row 4, `textChanged`, is not exact in dirt order.** Pinned lines 43–52
   update source/display, call `markLayoutNodeDirty`, and only then add
   `TextShape`. Rust `text_input_property_changed` delegates after its
   source/display writes to `mark_shape_dirty`, whose lines 81–92 publish the
   text revision, Path/TextShape bit and WorldTransform before layout-node
   invalidation. Classify this as an ordering discrepancy rather than exact
   pending evidence. The same helper ordering must be acknowledged in rows 8
   and 21 rather than treated as a transparent expansion.

4. **Row 9, `localBounds`, is not a pure exact accessor.** Pinned lines 68–75
   return the already-retained raw bounds. Rust
   `text_input_local_bounds_retained` at lines 525–539 calls
   `ensure_text_input_geometry`, which can rebuild and retain geometry before
   returning. Classify this as a lazy-materialization adaptation pending proof
   that the added update work and ordering are unobservable.

5. **Row 24 does not establish the optional-output behavior.** The pinned
   `gamepadDispatch` returns false without writing
   `outDispatchedScriptedDrawable`. Rust
   `RuntimeFocusable::gamepad_dispatch_default` has no corresponding output
   channel. It supports the false result but cannot be direct evidence for
   preservation of a caller's existing optional output. Mark that observable
   as an explicit Rust-signature adaptation or missing evidence; do not label
   the whole row exact.

6. **Rows 5 and 8 misdescribe the dirt identity.** In the pinned
   `ComponentDirt` and Rust `ComponentDirt`, `TextShape` and `Path` are both
   `1 << 4`. Row 5 remains genuinely incorrect because Rust suppresses the
   repeated unchanged-radius callback while C++ adds dirt unconditionally,
   but “Path rather than TextShape” is not a second mismatch. Row 8's real
   adaptation claim is the additional revision, WorldTransform, and layout
   publication—not a different Path bit.

In addition, row 22 must name its non-literal navigation owners: pinned
`cursorUp`, `cursorDown`, and line-boundary raw calls are reconstructed through
retained geometry and `move_cursor_vertical`/`cursor_horizontal` in Rust. They
remain within an already-incorrect row, but require their own differential
evidence and must not be summarized as exact raw operations.

## Locator corrections

At the reviewed candidate commit, the named definitions begin at:

- `hit_expandable`: nested
  `state_machine_instance/state_machine_instance.rs:4506`, not line 4501;
- `invalidate_runtime_layout_text_host`: `artboard.rs:6882`, not line 6868;
- `update_runtime_text_render_styles`: `draw.rs:18038`, not line 18017; and
- `publish_focusable_keyboard_capabilities`: nested
  `state_machine_instance/state_machine_instance.rs:2199`, not line 2197.

All other denominator rows survived the independent source read at the
candidate granularity. Their `pending evidence`, `incomplete`, `incorrect`, or
`missing` qualifiers remain in force; none is promoted by this review.

## Checks

- Pinned checkout identity, file lengths, byte counts, and both recorded
  SHA-256 values match.
- The candidate commit changes only the correspondence document.
- No runtime source, test, fixture, or evidence ledger was changed or executed
  by this review.

## Narrow correction rereview

Correction reviewed: `fe69bde91252d000301714506c059381767e7876`.

Verdict: **ACCEPTED as the independently reviewed TextInput discrepancy
inventory; the source pair remains correctly classified as not at parity**.

The correction addresses every item enumerated above without changing the
frozen denominator or source authority. Rows 4, 9, 17, 24, and 31 now carry
the required ordering, lazy-materialization, missing raw write-back,
Rust-signature, and failed-inversion classifications. Rows 32, 33, and 37
inherit the failed-conversion velocity defect. Rows 5 and 8 correctly identify
Path/TextShape as the same bit and retain only the real gating and additional
publication differences. Row 22 explicitly records the reconstructed
navigation owners and required differentials.

All four corrected locators resolve to their named definitions at the reviewed
commit: `hit_expandable` at 4506,
`invalidate_runtime_layout_text_host` at 6882,
`update_runtime_text_render_styles` at 18038, and
`publish_focusable_keyboard_capabilities` at 2199. The added
`move_cursor_vertical` locator also resolves at 598.

The pinned upstream hashes remain unchanged, the denominator remains 37 C++
definitions plus one `EM_JS` definition and five executable inline header
methods, and the correction changes only `text-input.md`. No production or
test behavior changed, so no runtime test gate was required for this narrow
document-only rereview.
