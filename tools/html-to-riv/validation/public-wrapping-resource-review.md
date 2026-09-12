# Public wrapping resource and lifecycle observations

The frozen public compiler was executed twice for each of12 owner-count requests; JavaScript/WASM matches all accepted bytes/maps and rejected diagnostics. Original and cloned ordinary files were then imported and resized by the unchanged release lifecycle probe, without source maps or CSS input.

| Profile / owners | RIV bytes | Runtime objects | Median import ms | Median clone ms | Median resize/update ms | Median CPU draw-record ms |
|---|---:|---:|---:|---:|---:|---:|
| integer-1 | 562 | 37 | 0.026 | 0.014 | 0.007 | 0.007 |
| integer-3 | 2342 | 165 | 0.084 | 0.053 | 0.025 | 0.012 |
| integer-8 | 7034 | 485 | 0.312 | 0.172 | 0.075 | 0.025 |
| integer-34 | 31520 | 2149 | 1.295 | 1.245 | 0.384 | 0.092 |
| fractional-3 | 118954 | 7238 | 10.102 | 9.088 | 1.626 | 0.089 |
| fractional-8 | 330154 | 19998 | 70.321 | 65.977 | 6.617 | 0.566 |
| fractional-34 | 1658435 | 99246 | 1743.233 | 1716.022 | 37.365 | 9.334 |

Seven scenes × two warmups and nine measured trials =77 fresh-import trials,616 original/clone frame observations;63 measured trials and504 measured frames. The native object count excludes the Backboard file record. Timing is CPU command recording, not GPU rendering; process peak RSS is not a retained-memory or leak measurement. The machine was checked for active builds/captures before starting; another project’s Cargo build had completed. No new compiler, capture or build ran concurrently with this measurement.

The integer case with84 owners succeeds (78,670 bytes). The tested1,562/1,563-owner cases reject Arithmetic(Separation), so no maximum supported integer count is established. Fractional34 succeeds (1,658,435 bytes);35 rejects ResourceBudget. All failures produce no RIV or map. These are source/graph proof and compiler budget boundaries, not evidence of a runtime impossibility. Fractional paint composition is expensive: the measured34-owner import exceeds1.7 seconds. This cost is an explicit limitation; passing visual tests does not establish acceptable application performance.

The16-fixture public visual corpus separately checks128 frames. These resource scenes were not given a new Chrome/GPU pixel qualification; lifecycle observation verifies ordinary import, resize, clone and CPU recording of the actual emitted bytes. Broader realistic compositions and a fractional graph optimization remain work.

Evidence: `output/public-wrapping-resources-r1/receipt.json`, `wasm-parity.json`, and `output/public-wrapping-lifecycle-r1/receipt.json`. The release runner checks manifest files and immutable root Cargo/Lock identities before running.
