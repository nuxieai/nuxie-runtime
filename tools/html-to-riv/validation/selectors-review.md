# Public selector qualification on the immutable runtime

The existing compiler-owned selector/cascade implementation is now validated through emitted ordinary Rive bytes. No runtime or compiler production code changed for this checkpoint. A new independent expectedPaint map in the validation driver asserts each authored node's intended browser color before pixel comparisons; this prevents a nonmatching or specificity-masked probe from looking like successful qualification.

Twenty-four scenes cover attribute existence, exact/token/hyphen/prefix/suffix/substring matching, default case sensitivity, ASCII-insensitive i and empty operands; adjacent/general sibling scope across comments and nesting; first/last/only-child; signed and filtered nth-child/nth-last-child; :not lists/complex selectors; :is maximum-branch specificity; :where zero specificity; nested functions; selector-list matching specificity and inline important precedence.

All192 original/clone geometry and pixel frames pass unchanged gates, with384 clear-color controls. Four new public Rust test groups compare selector compilation to independently enumerated target IDs; the complete suite has46 Rust and12 Node tests, including exact CLI/WASM corpus parity. Source maps remain read-only and are not passed to the importer.

All24 full first-frame image pairs were directly inspected on six sheets. Colors, target rows, sibling boundaries and negative controls agree. The remaining168 pairs were checked as exact decoded pixels after only white canvas extension or crop; removed cropped areas were verified white. That transfers the same visual content without claiming an additional direct inspection. The ordinary scenes were independently updated and drawn at each viewport; this image check is validation only.

S02-S08 are qualified within the admitted static box DOM and selector grammar. S01 remains partial because authored attributes are still restricted to id/class/style; arbitrary data/semantic attributes and namespaces have not been admitted. Explicit s flag, dynamic state, pseudo-elements and syntax outside the existing strict parser remain rejected. Passing selectors does not qualify a new paint/layout property, typography, scripting or host matching. Rules matching html/body are still rejected.

`public-selector-receipt.json` binds the source/CLI/WASM checkpoint, run, expected-color assertions and visual review. The earlier fractional rectangle and typography failures remain open and are not reclassified by this corpus.
