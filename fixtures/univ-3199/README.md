# Published text geometry fixture

Exact copies from `nuxieai/nuxie-ios` commit
`6420b013f2a8c6c1422546f4f6de88c4120c3f42`, directory
`fixtures/runtime/font-metrics-binding`. `fixture-sans.otf` is the same font
stored there under its SHA-256 filename
`2898476918b21c3f9b5ba22e86853c6d63b544f92da277a92533011a28c93af5.otf`.
The original production-publisher and asset hashes remain in `provenance.json`.

`report.json` records two authored editable boxes at (24,24) and (24,264),
both 342 by 220. The Apple product-import test resolves the external font,
changes bound font metrics, and checks the new geometry API against those
published boxes. This source-runtime probe does not qualify released SDK
native overlays or native caret/baseline alignment.
