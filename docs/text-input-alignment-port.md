# Native TextInput alignment

Authority: Rive commit `7098a7c86220fefe6e0620d83b53d906f6fe8dae`.
This ports its input alignment behavior, not the separate caret-blinking and
linked-corner-radius changes bundled in that upstream commit.

- Native property keys: `alignValue` 222 and `verticalAlignValue` 1094, both uint
  and defaulting to zero. Schema lookup, registry mutation/readback,
  deserialization, and cloning preserve them.
- The alignment box is the scroll viewport's content box (padding removed),
  falling back to the input's allocated width/height when no viewport exists.
- Alignment refreshes at import, shape update, and component advance so changing
  the viewport does not require resizing the text itself.
- Horizontal line alignment uses at least the alignment width. Vertical
  alignment uses only positive spare height; overflowing text remains at the
  top for scrolling. Bounds, glyph paths, ordered lines, and caret geometry use
  the same offset.
- Existing `Text::break_lines` and `FullyShapedText::shape` callers retain
  upstream's default zero alignment dimensions through wrappers.

Qualification:

```sh
cargo test -p nuxie-runtime --features tools --test upstream_text_input_native --test upstream_raw_text_input_native
```

39 tests pass (23 raw + 16 native). New checks cover all nine horizontal/vertical
combinations, intrinsic size preservation, caret translation, overflow fallback,
measurement parity, registry-backed updates, changing viewport padding, schema
dispatch, deserialization, and cloning. Existing editing/selection tests remain
green. Log: `/tmp/nuxie-input-alignment-combined-tests.log`.

This does not qualify the editor's native-input projection or cold publication
cutover. Those still need integration and browser/device execution evidence.
