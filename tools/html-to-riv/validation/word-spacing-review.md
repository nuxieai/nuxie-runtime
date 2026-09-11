# Word spacing and native text runs

The contract is based on [CSS Text 3 word spacing](https://www.w3.org/TR/css-text-3/#word-spacing-property):
normal is zero, lengths are inherited as computed absolute values, negative
spacing is valid, and spacing applies to word separators after whitespace
processing. Fixed-width spaces are not word separators. The current compiler
profile handles SPACE and NO-BREAK SPACE; visible-script separators are rejected
explicitly pending wider typography work. See SUPPORT.md for syntax and bounds.

Neither the native TextStylePaint schema nor the pinned C++ text headers has a
word-spacing field. The compiler therefore partitions text into separator and
ordinary runs inside a single native Text object, with at most two text styles.
Only separator advances receive additional word spacing. This preserves native
measurement and reflow of the same compiled scene at 240/390/768px. No browser
measurements, viewport-specific coordinates, modified font bytes, or artificial
space characters are baked into the scene. Existing zero/normal spacing retains
the original single-run binary layout.

Source maps now include text_run_ids in logical order and expose the singular
text_run_id only when one run represents the source element. Native import tests
concatenate the mapped run text, including NBSP, to verify completeness and order.
Authored text replacement requires recompilation to regenerate this partition;
this module still does not provide bindings or interaction semantics.

The public cascade test failed before implementation with unsupported-property.
It now proves normal/zero equivalence, signed px/rem equivalence, inherited
computed em values despite a child font shorthand, initial/unset/inherit, and
rejection of malformed or unsupported declarations even on unmatched selectors.
Separate tests prove the 4096-run bound and explicit visible-separator diagnostic.
Native/WASM corpus parity covers both the expanded source map and emitted bytes.

The initial 30 browser comparisons passed 29/30. Positive/negative spacing,
left/center/right alignment, collapsed whitespace, repeated NBSP, true optional
ligatures, combined letter spacing and inherited lengths all passed. The
fixed-width-space specimen failed at 240px: Chromium fits “one two three” on the
first line, while native breaks before “three”. Metrics show 851 mismatched
pixels, ratio 0.011080729, mean channel error 3.1977246. The same specimen without
word spacing is retained as a control in the main corpus.

Source investigation: renderer.rs::is_white_space recognizes code points <=0x20,
U+2028 and U+200B; the pinned upstream renderer.cpp::isWhiteSpace uses the same
set. text_engine.rs::Font::shape_text uses this classification for break markers.
It omits U+2002/U+2003, despite their browser break opportunities. This behavior
needs a separately qualified line-breaking policy; simply adding word spacing
to those characters would violate their intended fixed-width semantics.

All geometry/pixel tolerances are unchanged. A13 remains partial while this
broader line-breaking limitation and additional-script coverage remain open.

## Final qualification

Full checked-host glyph lane: **353/356**. The failures are the preserved A09
fractional edge and the fixed-width-space specimen with word spacing 6px and 0px,
both at 240px. The zero-spacing control confirms that the wrapping discrepancy
exists without run partitioning (839 mismatched pixels, ratio 0.010924479,
mean channel error 3.1160352). All 33 new geometry checks pass, but 31/33 pixel
checks pass in each renderer profile. The same two failures occur in the vector
subset. They remain failing main-corpus tests; no tolerance or expectation was
relaxed. A13 remains partial, with a concrete line-break investigation next.

All 315 preexisting native PNGs are byte-identical to ligatures-full. Every new
browser/native pair was visually inspected at all three widths. The two failed
pairs visibly break at different words; the passing pairs agree on wrapping,
spacing, alignment and ligature behavior. Contact sheets are retained as
word-spacing-full/words-{240,390,768}.png.

All 56 Rust tests, five native/WASM JS tests, one checked-host requirements test,
two gallery tests, TypeScript, module Clippy and boundary validation pass.
Renderer-state comparisons pass 27/27 separately (the full runner stops on the
known main-corpus failures). Chromium 153.0.8010.12, DPR 1; actual Rust Metal
rendering. New source maps and manifests pass native/WASM parity.

Artifacts under output/playwright/html-to-riv: word-spacing-probe (initial
30-case probe), word-spacing-full (356 checks), word-spacing-vector (33 cases).
Reproduce with validation/run.sh native-glyphs from the module, or run the
already-built native probe through npm test -- --grep word-spacing- with
NUXIE_NATIVE_GLYPHS=0 for the vector subset.

Next: qualify a CSS line-break policy for fixed-width spaces, with explicit
legacy compatibility and alignment/trailing-space controls. Preserve NBSP's
nonbreaking behavior. A14 explicit br follows that investigation.
