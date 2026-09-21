# Empty-shaped-text selection failure

Tracked in [UNIV-3330](https://universe.basis.dev/issue/UNIV-3330).

During native TextInput qualification, clearing an input using the single-glyph
`roboto-a.ttf` fixture produced no shaped glyphs or ordered lines for U+200B.
Rust panics when `Cursor::selection_rects` indexes line zero. A full UI font and
the input-capable Roboto subset pass empty-input shaping.

The cursor assumption is also broken in unmodified upstream C++ commit
`9ed5b5168d95aab07e873db341fb65613d317cfc`: the standalone empty-shape reproducer
faults at `Cursor::selectionRects`, `cursor.cpp:270`, through
`OrderedLine::glyphLine`. No upstream tracked sources were edited for this test.
The reproduction isolates cursor handling; it does not claim an end-to-end C++
font-import/RawTextInput comparison. System HarfBuzz 14.2.1 independently produces
zero glyphs for U+200B with the single-glyph font, but is not the pinned shaper.

From this repository, with `RIVE_RUNTIME_DIR` pointing to the upstream checkout:

```sh
clang++ -std=c++17 -O1 -g -fsanitize=address -DWITH_RIVE_TEXT \
  -ffunction-sections -fdata-sections -I "$RIVE_RUNTIME_DIR/include" \
  tools/cpp-probe/empty-input-selection.cpp \
  "$RIVE_RUNTIME_DIR/src/text/cursor.cpp" \
  "$RIVE_RUNTIME_DIR/src/text/glyph_lookup.cpp" \
  "$RIVE_RUNTIME_DIR/src/text/text_engine.cpp" \
  -Wl,-dead_strip -o /tmp/nuxie-empty-input-selection
/tmp/nuxie-empty-input-selection
```

Observed: `ordered-lines=0`, then an AddressSanitizer null-page read at offset
0x30. macOS Clang command; linker dead stripping keeps the probe small without
building the complete runtime. Logs from qualification:
`/tmp/nuxie-cpp-empty-cursor-build.log` and
`/tmp/nuxie-cpp-empty-cursor-run.log`.

The user approved the bounded correction: an empty shape contributes no selection
rectangles. The implemented guard preserves caller-owned output entries and all
nonempty-shape behavior. `empty_shaped_text_has_no_selection_rectangles` covers
the guard, and the parent authoring suite's `raw_input_with_subset_font_can_be_cleared`
regression passes. The current raw/native input suites pass all 41 tests after
rebasing onto runtime main (log: `/tmp/nuxie-input-rebased-alignment.log`).

This is an upstream bug correction, not restoration of C++ port parity. The C++
probe above records the original upstream failure; it is not evidence of a
corrected upstream C++ build.
