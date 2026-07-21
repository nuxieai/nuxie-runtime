# B-6 Structural Fidelity Audit Summary

Audit source: pinned C++ `/Users/levi/dev/oss/rive-runtime` at `d788e8ec6e8b598526607d6a1e8818e8b637b60c`; Rust source: current `main` during the audit. All 447 manifest rows have one on-disk record. Verdicts apply the five structural axes, the advance/update/bind mutation-timing gate, crate-wide family grep, and sibling sweep.

## Verdict totals

| Verdict | Rows |
|---|---:|
| ISOMORPHIC | 19 |
| ADAPTED | 182 |
| DIVERGENT | 162 |
| UNKNOWN | 36 |
| N/A | 48 |
| **Total** | **447** |

The 162 DIVERGENT rows contain 258 mutation-gated mechanisms in total. Counts below are per row; shared mechanism families may therefore appear in more than one row record.

## Per-cluster totals

| Cluster | Total | ISOMORPHIC | ADAPTED | DIVERGENT | UNKNOWN | N/A |
|---|---:|---:|---:|---:|---:|---:|
| [data-bind-view-model](results/data-bind-view-model.md) | 81 | 0 | 31 | 50 | 0 | 0 |
| [animation](results/animation.md) | 86 | 7 | 57 | 13 | 9 | 0 |
| [layout-shapes-paint](results/layout-shapes-paint.md) | 54 | 4 | 5 | 44 | 1 | 0 |
| [unavailable](results/unavailable.md) | 48 | 0 | 0 | 0 | 0 | 48 |
| [misc-core](results/misc-core.md) | 40 | 1 | 26 | 12 | 1 | 0 |
| [assets-importers](results/assets-importers.md) | 36 | 0 | 32 | 0 | 4 | 0 |
| [text](results/text.md) | 30 | 0 | 6 | 15 | 9 | 0 |
| [bones-math-components](results/bones-math-components.md) | 21 | 6 | 6 | 7 | 2 | 0 |
| [constraints](results/constraints.md) | 18 | 0 | 3 | 11 | 4 | 0 |
| [lua-scripting](results/lua-scripting.md) | 14 | 0 | 9 | 2 | 3 | 0 |
| [binary-core](results/binary-core.md) | 7 | 1 | 6 | 0 | 0 | 0 |
| [scripted](results/scripted.md) | 6 | 0 | 0 | 3 | 3 | 0 |
| [focus-input](results/focus-input.md) | 3 | 0 | 0 | 3 | 0 | 0 |
| [artboard](results/artboard.md) | 2 | 0 | 0 | 2 | 0 | 0 |
| [state-machine](results/state-machine.md) | 1 | 0 | 1 | 0 | 0 | 0 |
| **Total** | **447** | **19** | **182** | **162** | **36** | **48** |

## DIVERGENT rows ranked by mutation-gated mechanism count

| Rank | Row | Cluster | Mechanisms | Confidence |
|---:|---|---|---:|---|
| 1 | B6-0123 | bones-math-components | 8 | high |
| 2 | B6-0077 | animation | 6 | high |
| 3 | B6-0117 | bones-math-components | 5 | high |
| 4 | B6-0118 | bones-math-components | 5 | high |
| 5 | B6-0119 | bones-math-components | 5 | high |
| 6 | B6-0120 | bones-math-components | 5 | high |
| 7 | B6-0200 | data-bind-view-model | 4 | med |
| 8 | B6-0115 | bones-math-components | 3 | high |
| 9 | B6-0116 | bones-math-components | 3 | high |
| 10 | B6-0141 | constraints | 3 | high |
| 11 | B6-0157 | data-bind-view-model | 3 | med |
| 12 | B6-0158 | data-bind-view-model | 3 | med |
| 13 | B6-0159 | data-bind-view-model | 3 | med |
| 14 | B6-0160 | data-bind-view-model | 3 | med |
| 15 | B6-0161 | data-bind-view-model | 3 | med |
| 16 | B6-0162 | data-bind-view-model | 3 | med |
| 17 | B6-0163 | data-bind-view-model | 3 | med |
| 18 | B6-0164 | data-bind-view-model | 3 | med |
| 19 | B6-0165 | data-bind-view-model | 3 | med |
| 20 | B6-0167 | data-bind-view-model | 3 | med |
| 21 | B6-0168 | data-bind-view-model | 3 | med |
| 22 | B6-0169 | data-bind-view-model | 3 | med |
| 23 | B6-0171 | data-bind-view-model | 3 | med |
| 24 | B6-0172 | data-bind-view-model | 3 | med |
| 25 | B6-0194 | data-bind-view-model | 3 | med |
| 26 | B6-0195 | data-bind-view-model | 3 | med |
| 27 | B6-0196 | data-bind-view-model | 3 | med |
| 28 | B6-0209 | misc-core | 3 | high |
| 29 | B6-0238 | focus-input | 3 | high |
| 30 | B6-0303 | artboard | 3 | high |
| 31 | B6-0034 | animation | 2 | high |
| 32 | B6-0036 | animation | 2 | high |
| 33 | B6-0037 | animation | 2 | high |
| 34 | B6-0040 | animation | 2 | high |
| 35 | B6-0044 | animation | 2 | high |
| 36 | B6-0058 | animation | 2 | high |
| 37 | B6-0094 | artboard | 2 | high |
| 38 | B6-0138 | constraints | 2 | high |
| 39 | B6-0146 | misc-core | 2 | high |
| 40 | B6-0166 | data-bind-view-model | 2 | med |
| 41 | B6-0170 | data-bind-view-model | 2 | med |
| 42 | B6-0197 | data-bind-view-model | 2 | med |
| 43 | B6-0239 | focus-input | 2 | high |
| 44 | B6-0240 | focus-input | 2 | high |
| 45 | B6-0304 | misc-core | 2 | high |
| 46 | B6-0319 | misc-core | 2 | high |
| 47 | B6-0419 | data-bind-view-model | 2 | med |
| 48 | B6-0435 | data-bind-view-model | 2 | med |
| 49 | B6-0436 | data-bind-view-model | 2 | med |
| 50 | B6-0013 | animation | 1 | high |
| 51 | B6-0057 | animation | 1 | high |
| 52 | B6-0079 | animation | 1 | high |
| 53 | B6-0080 | animation | 1 | high |
| 54 | B6-0091 | animation | 1 | high |
| 55 | B6-0093 | animation | 1 | high |
| 56 | B6-0095 | misc-core | 1 | high |
| 57 | B6-0125 | constraints | 1 | high |
| 58 | B6-0126 | constraints | 1 | high |
| 59 | B6-0128 | constraints | 1 | high |
| 60 | B6-0129 | constraints | 1 | high |
| 61 | B6-0131 | constraints | 1 | high |
| 62 | B6-0132 | constraints | 1 | high |
| 63 | B6-0133 | constraints | 1 | high |
| 64 | B6-0143 | constraints | 1 | high |
| 65 | B6-0144 | constraints | 1 | high |
| 66 | B6-0174 | data-bind-view-model | 1 | med |
| 67 | B6-0182 | data-bind-view-model | 1 | med |
| 68 | B6-0248 | layout-shapes-paint | 1 | high |
| 69 | B6-0249 | layout-shapes-paint | 1 | high |
| 70 | B6-0252 | layout-shapes-paint | 1 | high |
| 71 | B6-0254 | layout-shapes-paint | 1 | high |
| 72 | B6-0255 | layout-shapes-paint | 1 | high |
| 73 | B6-0258 | misc-core | 1 | high |
| 74 | B6-0269 | lua-scripting | 1 | high |
| 75 | B6-0288 | lua-scripting | 1 | high |
| 76 | B6-0314 | misc-core | 1 | high |
| 77 | B6-0315 | misc-core | 1 | high |
| 78 | B6-0316 | misc-core | 1 | high |
| 79 | B6-0317 | misc-core | 1 | high |
| 80 | B6-0318 | misc-core | 1 | high |
| 81 | B6-0320 | misc-core | 1 | high |
| 82 | B6-0322 | scripted | 1 | high |
| 83 | B6-0325 | scripted | 1 | high |
| 84 | B6-0326 | scripted | 1 | high |
| 85 | B6-0331 | layout-shapes-paint | 1 | high |
| 86 | B6-0332 | layout-shapes-paint | 1 | high |
| 87 | B6-0333 | layout-shapes-paint | 1 | high |
| 88 | B6-0334 | layout-shapes-paint | 1 | high |
| 89 | B6-0335 | layout-shapes-paint | 1 | high |
| 90 | B6-0337 | layout-shapes-paint | 1 | high |
| 91 | B6-0338 | layout-shapes-paint | 1 | high |
| 92 | B6-0340 | layout-shapes-paint | 1 | high |
| 93 | B6-0341 | layout-shapes-paint | 1 | high |
| 94 | B6-0343 | layout-shapes-paint | 1 | high |
| 95 | B6-0344 | layout-shapes-paint | 1 | high |
| 96 | B6-0345 | layout-shapes-paint | 1 | high |
| 97 | B6-0346 | layout-shapes-paint | 1 | high |
| 98 | B6-0347 | layout-shapes-paint | 1 | high |
| 99 | B6-0348 | layout-shapes-paint | 1 | high |
| 100 | B6-0349 | layout-shapes-paint | 1 | high |
| 101 | B6-0350 | layout-shapes-paint | 1 | high |
| 102 | B6-0352 | layout-shapes-paint | 1 | high |
| 103 | B6-0353 | layout-shapes-paint | 1 | high |
| 104 | B6-0354 | layout-shapes-paint | 1 | high |
| 105 | B6-0355 | layout-shapes-paint | 1 | high |
| 106 | B6-0356 | layout-shapes-paint | 1 | high |
| 107 | B6-0357 | layout-shapes-paint | 1 | high |
| 108 | B6-0358 | layout-shapes-paint | 1 | high |
| 109 | B6-0359 | layout-shapes-paint | 1 | high |
| 110 | B6-0360 | layout-shapes-paint | 1 | high |
| 111 | B6-0361 | layout-shapes-paint | 1 | high |
| 112 | B6-0362 | layout-shapes-paint | 1 | high |
| 113 | B6-0363 | layout-shapes-paint | 1 | high |
| 114 | B6-0365 | layout-shapes-paint | 1 | high |
| 115 | B6-0366 | layout-shapes-paint | 1 | high |
| 116 | B6-0367 | layout-shapes-paint | 1 | high |
| 117 | B6-0368 | layout-shapes-paint | 1 | high |
| 118 | B6-0369 | layout-shapes-paint | 1 | high |
| 119 | B6-0370 | layout-shapes-paint | 1 | high |
| 120 | B6-0371 | layout-shapes-paint | 1 | high |
| 121 | B6-0372 | layout-shapes-paint | 1 | high |
| 122 | B6-0373 | layout-shapes-paint | 1 | high |
| 123 | B6-0374 | layout-shapes-paint | 1 | high |
| 124 | B6-0379 | text | 1 | high |
| 125 | B6-0380 | text | 1 | high |
| 126 | B6-0381 | text | 1 | high |
| 127 | B6-0383 | text | 1 | high |
| 128 | B6-0385 | text | 1 | high |
| 129 | B6-0387 | text | 1 | high |
| 130 | B6-0390 | text | 1 | high |
| 131 | B6-0393 | text | 1 | high |
| 132 | B6-0396 | text | 1 | high |
| 133 | B6-0397 | text | 1 | high |
| 134 | B6-0399 | text | 1 | high |
| 135 | B6-0400 | text | 1 | high |
| 136 | B6-0402 | text | 1 | high |
| 137 | B6-0404 | text | 1 | high |
| 138 | B6-0405 | text | 1 | high |
| 139 | B6-0411 | data-bind-view-model | 1 | med |
| 140 | B6-0412 | data-bind-view-model | 1 | med |
| 141 | B6-0413 | data-bind-view-model | 1 | med |
| 142 | B6-0414 | data-bind-view-model | 1 | med |
| 143 | B6-0415 | data-bind-view-model | 1 | med |
| 144 | B6-0416 | data-bind-view-model | 1 | med |
| 145 | B6-0417 | data-bind-view-model | 1 | med |
| 146 | B6-0418 | data-bind-view-model | 1 | med |
| 147 | B6-0420 | data-bind-view-model | 1 | med |
| 148 | B6-0421 | data-bind-view-model | 1 | med |
| 149 | B6-0422 | data-bind-view-model | 1 | med |
| 150 | B6-0423 | data-bind-view-model | 1 | med |
| 151 | B6-0427 | data-bind-view-model | 1 | med |
| 152 | B6-0428 | data-bind-view-model | 1 | med |
| 153 | B6-0430 | data-bind-view-model | 1 | med |
| 154 | B6-0431 | data-bind-view-model | 1 | med |
| 155 | B6-0432 | data-bind-view-model | 1 | med |
| 156 | B6-0433 | data-bind-view-model | 1 | med |
| 157 | B6-0434 | data-bind-view-model | 1 | med |
| 158 | B6-0437 | data-bind-view-model | 1 | med |
| 159 | B6-0438 | data-bind-view-model | 1 | med |
| 160 | B6-0439 | data-bind-view-model | 1 | med |
| 161 | B6-0440 | data-bind-view-model | 1 | med |
| 162 | B6-0442 | data-bind-view-model | 1 | med |

## Low-confidence rows

| Row | Cluster | Verdict |
|---|---|---|
| B6-0260 | lua-scripting | UNKNOWN |
| B6-0267 | lua-scripting | UNKNOWN |
| B6-0270 | lua-scripting | UNKNOWN |
| B6-0321 | scripted | UNKNOWN |
| B6-0323 | scripted | UNKNOWN |
| B6-0324 | scripted | UNKNOWN |
| B6-0339 | layout-shapes-paint | UNKNOWN |
| B6-0378 | text | UNKNOWN |
| B6-0384 | text | UNKNOWN |
| B6-0388 | text | UNKNOWN |
| B6-0389 | text | UNKNOWN |
| B6-0391 | text | UNKNOWN |
| B6-0392 | text | UNKNOWN |
| B6-0398 | text | UNKNOWN |
| B6-0401 | text | UNKNOWN |
| B6-0406 | text | UNKNOWN |

## Manifest mapping issues

- `B6-0106` (`src/assets/script_asset.cpp` → `crates/nuxie-runtime/src/objects.rs`): the mapped Rust file only stores imported fields; the corresponding VM registration, hydration, and scripted-object lifecycle resides outside that row mapping. Verdict: UNKNOWN.
- `B6-0339` (`src/list_path.cpp` → `crates/nuxie-runtime/src/draw.rs`): the manifest marks partial coverage, but no `ListPath`/`VertexListener` implementation or mapped region was found; current list-path matches are unrelated view-model APIs. Verdict: UNKNOWN.

Separately, the data-bind/view-model coverage plan named the legacy sibling `crates/nuxie-runtime/src/state_machine/data_bind_graph.rs`, which is absent on current main. The sweep used the live replacement `crates/nuxie-runtime/src/data_bind_graph.rs` for all 81 rows; this is a coverage-path correction, not an additional row verdict.

## Audit boundary

Behavior-gap observations remain one-line notes in their row records. This summary makes no remediation, rebuild, or triage decision.
