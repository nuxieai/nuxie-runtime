# Fractional CSS paint isolated from wrapping

The standalone experiment reproduces the fractional-edge discrepancy without wrapping helpers. Ordinary LayoutComponent Fill and Shape/Rectangle Fill produce **identical native pixels in all 80 matched route/frame pairs**. A sampled band around the original orange failure is byte-identical in each engine between the standalone and wrapping scenes. Wrapping is therefore unnecessary to produce this specific discrepancy; changing only those two ordinary paint routes does not repair it.

This is a private validation checkpoint, not public fractional-paint qualification or proof that a file-level solution is impossible. Runtime, renderer, schema and shared dependencies remain unchanged.

## Experiment

Twenty independently authored cases cover two paint routes, both axes, offsets 64.25/64.5/64.75/65 px and a live 32.5% offset. Responsive viewports use 241/321/203 pixels on the moving axis, deliberately producing fractional positions. An ordinary unpainted flex spacer positions the paint owner; the Shape route adds a same-size Rectangle child. There are no wrapping helpers, constraints, paint replicas or extra masks.

The map describes actual finite LayoutComponent owner boxes. A separate command observer verifies the actual painted path bounds, rather than confusing owner geometry with Shape geometry. Every frame has exactly two draw paths (white artboard and the translucent rectangle), only an ordinary artboard clip, and identity/translation matrices. Path bounds agree with independently captured Chrome geometry at the existing tolerance.

| Check | Result |
| --- | --- |
| Ordinary construction | 20 cases; fresh adapter reproduction exact |
| Native/Chrome owner geometry | 160/160 pass |
| Native path bounds | 160/160 checked separately |
| Pixel gates | 136/160 pass; 24 mismatch-ratio failures retained |
| Clear independence | 320/320 cyan/transparent comparisons match |
| Layout/Shape route pairs | 80/80 native and Chrome pairs exact |
| Repeat/original-clone pixels | 100/100 pairs exact |
| Visual inspection | 60 distinct full pairs on 18 sheets; 100 exact transfers |

The 24 failed frames are the half-pixel literal controls at the narrow initial/third/return sizes, including clones, on both routes and axes. The same half-pixel edge difference exists at the wider viewport but occupies less than the global failure ratio. Passing quarter/three-quarter/responsive cases can retain fine channel/coverage differences; passing a metric gate does not mean pixel identity with Chrome. No gates were widened.

Direct inspection confirms corresponding positions and rectangle interiors, with thin horizontal/vertical half-pixel boundary differences. The orange band at x=[0,80), y=[60,115) in the standalone y=64.5 control exactly reproduces the earlier wrapped responsive frame's Chrome and native pixels. This is local defect reproduction, not whole-scene equivalence or a transfer of source geometry.

## Source-backed cause and limits

The [pinned Chrome rule](fractional-paint-chrome-rule.md) explains the observed box edges. Chrome snaps cumulative CSS paint offsets and extents in its LayoutUnit paint space, including ties toward positive infinity and a thin-box exception. It does not independently round authored dimensions. Transforms, cumulative ancestor offsets and layout quantization matter; arbitrary SVG paths use a different route.

The [earlier native audit](wrapped-derived-edge-audit.md) shows ordinary layout and rectangle paths retain floating geometry. The current fresh route comparison confirms that a simple switch from layout background to Rectangle is insufficient. No direct serialized pixel-snap field was found in those inspected controls. The schema does contain DataConverterRounder, but it belongs to the data-binding system, which is explicitly excluded from this compiler goal; its existence does not supply a current allowed live-layout paint primitive.

A possible allowed composition remains: derive paint-only rounded edges from live layout using ordinary constraints, preserving the original layout boxes. The next concrete probe tests a positive predicate using chained gain/clamp TranslationConstraints. First clamp a scalar to [0,1], then apply three stages of multiplication by 2^64 with [0,1] clamping. Over finite f32 values this would map every positive value, including the minimum subnormal, to one while preserving zero. Native denormal handling, evaluation order and cloning must be tested before using that observation to construct a bounded binary rounding graph. This is a hypothesis, not implemented support or a completed numerical certificate.

Do not silently round public authored layout, replace responsive behavior with a static snapshot, or declare all fractional geometry impossible. Keep the 48 wrapped and 24 standalone pixel failures. After the predicate probe, test a live paint-edge composition or record its precise failure, while continuing independent backlog work when a remaining premise is unresolved.

## Reproduction and evidence

- Constructor: `output/fractional-paint-constructor-r1`; [constructor review](fractional-paint-constructor-review.md). All 83 source bindings are verified. The inherited builder docstring mentions Derived, but its frozen bridge and explicit patches emit only these ordinary standalone records; no wrapping module is called.
- Capture: `output/playwright/fractional-paint-r1/receipt.json`; driver `check-fractional-paint-baseline.mjs`, frozen constructor adapter and the existing immutable probe/renderer. The driver additionally rejects nonfinite owner metrics instead of allowing null dimensions to evade comparisons.
- Actual path and route comparison: `path-receipt.json`. Direct review: `visual/review-receipt.json`. Original wrapping reproduction: `wrapping-local-reproduction.json`.
- Verifier: `fractional-paint-evidence.py`, result `output/fractional-paint-verification-r1.json`. It preserves explicit failure counts and verifies 1,614 artifact bindings, including pinned native build/dependency receipts and nine downloaded Chromium sources.
- Product source and public binaries were not changed. The prior 404 Rust/56 Node product checkpoint remains applicable to unchanged product source; no full product test rerun or new public support is claimed for these validation-only additions.

The 99-item coverage counts remain 13 qualified / 23 partial / 4 investigating / 59 pending. Public wrapping remains unadmitted. Do not rerun this completed isolation experiment without a changed hypothesis.
