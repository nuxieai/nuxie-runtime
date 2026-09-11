# Explicit br and hidden text

The profile reset makes containers flex columns. A br in a flex container is a
flex item, not an inline forced line break, so the compiler now accepts explicit
display:block for text-only containers. Internal flex direction/gaps/alignment
are ignored for that block text; outer flex-item sizing remains. General block
flow, nested rich text and styled br remain separate work with explicit errors.
The reset stylesheet is unchanged.

A dedicated text_content module normalizes ASCII whitespace per forced line and
emits native newline characters in one Text object. Existing shaping and line
layout already handle leading, consecutive, trailing and break-only content.
No renderer changes, new runtime requirement, fake spacers or measured browser
coordinates are necessary. SourceBreak records each br's identity/path and
Unicode-scalar newline offset. The source map retains all text runs, including
word-spacing partitions. Breaks do not masquerade as separate layout objects.

The public contract regression failed before implementation at display:block.
It now covers scalar offsets after non-ASCII text, styled-break rejection,
unsupported flex/block contexts, duplicate identity, per-container 4096-break
and document-wide 8192-identity bounds. A native import test concatenates actual
mapped run text and checks newline preservation alongside word spacing.

The initial 30 browser comparisons passed. The expanded 39-case corpus also
passed aggregate metrics, but visual review caught hidden text drawing as a
thin column. This is precisely why passing aggregate pixels is not sufficient.
Nine strict absence controls failed before the fix, including direct and inherited
hiding. The compiler now writes the native Hidden drawable flag for every emitted
layout/text/image under display:none while preserving source text and identities.
This avoids changing ordinary runtime behavior. Authored style changes require
recompilation, consistent with this module's static source contract.

Strict absence controls use all-white output and require exact RGBA equality.
The earlier pale-background version had a one-level antialiased edge difference
even after hidden ink was fixed; white removes that irrelevant background edge.
No general pixel threshold was relaxed. Four controls at three sizes include
hidden br, plain text, nested text and nested text/image/background painting.
All twelve exact controls pass. A native regression additionally confirms that
hidden source text and break IDs survive import while no drawPath command occurs.

The full workflow includes Rust tests, native/WASM byte/map/requirement parity,
TypeScript, gallery checks, native glyph comparison and vector comparison of the
new fixtures. Every scene compiles at 390px and resizes to 240/390/768px without
recompilation. Artifacts are under output/playwright/html-to-riv: explicit-br-probe,
explicit-br-full, explicit-br-vector, hidden-text-red and hidden-text-green.

## Final qualification

Full glyph lane: **445/446**, retaining only A09's fractional-edge failure.
All 48 new break/hidden comparisons pass in both renderer profiles, including
12 exact blank-image controls. All 390 previous native scene PNGs are byte-identical
to space-break-full. Every new pair was visually reviewed at 240/390/768px;
line boxes, forced/soft wrapping and alignment agree, and hidden content is blank.
Contact sheets are explicit-br-full/breaks-final-{width}-{group}.png.

All 62 Rust tests, five native/WASM JS tests, one checked-host requirement test,
two gallery tests, TypeScript, module Clippy, boundary validation and 27 renderer
state controls pass. The full runner stops before its final state-control stage
because of A09, so those controls ran separately. No ordinary pixel tolerances
were widened; absence-of-paint controls strengthen the gate.

Reproduce:

```sh
bash tools/html-to-riv/validation/run.sh native-glyphs
# From tools/html-to-riv after building the probe:
NUXIE_NATIVE_GLYPHS=0 npm test -- --grep 'explicit-br-|hidden-text-|hidden-painted-'
```

A14 qualifies the documented text-only block subset. General inline styling,
block children and styled breaks remain explicitly rejected and belong to later
formatting work. Next: A15 white-space:nowrap, preserving explicit breaks and
responsive native sizing while disabling only soft wrapping.
