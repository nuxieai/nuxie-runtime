# Compiler-derived first-baseline alignment

The public compiler now admits `align-self: baseline` and `first baseline` for evidenced metrics in horizontal LTR single-line row and row-reverse containers. Values are derived from authored styles; the public path no longer uses fixture adapters, selected object IDs or fixture constants. Unresolved contexts diagnose before any output is published. L03 remains partial.

## Admitted metric forms

Empty boxes can use fixed heights with fixed min/max bounds, or percentage heights with a fixed minimum and no maximum. The ordinary helper evaluates the maximum percentage coefficient with the maximum fixed floor against the original parent. Mixed non-baseline siblings are excluded from that baseline maximum but retain their layout and alignment.

A fixed-height normal-column box can propagate a proven fixed baseline from its first order-modified descendant. The baseline must be within the box; ascent and descent contribute to parent extent. Nested summaries recurse through these evidenced forms. Combining responsive ascent with nonzero nested descent remains rejected, as do unresolved automatic-height baseline participants, explicit auto minima, percentage min/max bounds, capped percentage heights, last baseline, column baseline containers and unresolved nested row/reversed-column topology. Existing percentage-height guards still apply. Baseline inheritance preserves the keyword but cannot bypass these context restrictions.

## Ordinary file composition

Sequential child emission retains compact object IDs, ordering keys and scalar baseline summaries. After the children are emitted, the compiler appends a zero-width measurement LayoutComponent, a Node landmark with TransformConstraint, and ComponentOrigin/Y-only TranslationConstraint records for the participants. Fixed helper extent is maximum ascent plus maximum descent; responsive empty-box extent is an ordinary percentage with a fixed minimum. Zero extents use a zero anchor to avoid division by zero. Computed fixed helper extent above 1,000,000px diagnoses.

The helper participates in layout, so automatic parent size and following siblings account for baseline descent before constraints position the visible boxes. Main-axis slots, original percentage containing blocks, source identities and selector order remain intact. Generated records add four per participating sibling group plus two per participant, at most six per authored element. No full sibling Style or custom-property environments are retained. No baseline participants means no extra records; all 253 prior fixtures retain identical Rive bytes and source maps.

## Validation and limits

The main 34-scene public corpus passes 272 geometry/pixel frames and 544 clear controls on the immutable renderer and pinned Chrome. It covers both row directions; parent fixed/auto/min/max sizing; mixed center alignment; ordering; zero heights; fixed bounds; percentage crossovers; variables; fixed nested baselines; following siblings; and translated/centered parents. All 84 distinct viewport pairs were inspected visually with no divergence found. Exact decoded RGBA crop/white-extension proofs cover the remaining 188 frames. Original and cloned files resize without recompilation. The frozen compiler reproduces every main scene and diagnostic map exactly.

All 84 Rust tests and 23 Node tests pass, including public CLI/WASM output and rejected-context diagnostic parity. TypeScript and the immutable-source guard pass. The Rust tests vary authored dimensions to verify derived measurements rather than fixture constants, and check finite zero anchors, bounds, unsupported combinations and excessive helper extents.

The separate fractional nested row-reverse fixture passes all eight geometry frames but fails two pixel frames around a small leaf edge. All Chrome and native images exactly match the earlier experiment and its unconstrained public-compiler control. This remains a pixel limitation, not a passing qualification or a reason to alter tolerances. See public-first-baseline-receipt.json for all bindings. The earlier baseline experiment reviews are historical; this document supersedes their blanket public-baseline rejection statements.

Further work includes automatic/intrinsic participant metrics, more nested topologies, last-baseline groups and broader bounded responsive expressions. These remain unresolved rather than impossible. The full backlog and immutable-runtime contract are unchanged.

The subsequent intrinsic extension admits empty auto boxes and bounded normal-column intrinsic metrics; see baseline-intrinsic-review.md for its separate scope and evidence. Counts above describe this earlier checkpoint.
