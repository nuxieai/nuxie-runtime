# Rive Renderer Performance

Schema: `rive-renderer-perf-v3`
Estimator: `cpp-control-min-paired-v1`
Pair order: `counterbalanced-scene-sample-v1`
Runner protocol: `rive-renderer-perf-runner-v1`
Manifest SHA-256: `5bf53a8edb189b12b5b34ae88dcbd9c23958d2964db1f917bb5f9cacf8e78e53`
Baseline runner SHA-256: `98f37c7c87f4689309a8b37c1ab25db8b0b6445f04debfddae3031e68b00bb97`
Candidate runner SHA-256: `0c0d932292544d08de1e6a90949abba8865ade4728a5fd956a832d3aeb65c042`
Generator SHA-256: `701ace876f7b66977a32cd6846bb497fdd064b4c640fe40896b9ce79356b8ee2`
Baseline source identity: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`
Candidate source identity: `73314a8d5a4a90b24e4d590df17be89a07d1d776+renderer-dirty-sha256-a57f1051f8e1d4c8b92c82d09d1ac002e404711fb84f1693bf312b8e6efcc1cc`

| scene | mode | C++ selected sample | C++ selected ns | paired candidate ns | paired ratio | baseline p50 ns | baseline p95 ns | baseline spread ns | candidate p50 ns | candidate p95 ns | candidate spread ns | logical flushes | draws | atomic strategy partitions |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 7 | 273791 | 267500 | 0.977023 | 307375 | 432209 | 158418 | 305875 | 400375 | 136084 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 6 | 230750 | 286292 | 1.240702 | 265917 | 279625 | 48875 | 279500 | 294375 | 28542 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 3 | 347000 | 339375 | 0.978026 | 352292 | 389708 | 42708 | 340000 | 409209 | 81834 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 3 | 242750 | 324750 | 1.337796 | 320459 | 325042 | 82292 | 297750 | 327458 | 100750 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 6 | 363541 | 335708 | 0.923439 | 373500 | 449417 | 85876 | 340667 | 346291 | 10583 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 5 | 258292 | 318417 | 1.232779 | 284583 | 329417 | 71125 | 318417 | 333833 | 110624 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 2 | 265042 | 270291 | 1.019804 | 282375 | 421500 | 156458 | 285000 | 422292 | 153875 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 7 | 229209 | 242167 | 1.056534 | 282417 | 287292 | 58083 | 258375 | 279417 | 50833 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 7 | 498583 | 466125 | 0.934900 | 518583 | 608167 | 109584 | 474708 | 520042 | 63000 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 1 | 221167 | 278375 | 1.258664 | 289083 | 291417 | 70250 | 287209 | 296166 | 57749 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 6 | 280084 | 259792 | 0.927550 | 327333 | 370000 | 89916 | 274167 | 448959 | 189167 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 6 | 218542 | 274667 | 1.256816 | 296375 | 325375 | 106833 | 274667 | 319958 | 100041 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 1 | 306166 | 306208 | 1.000137 | 319625 | 401667 | 95501 | 325666 | 349959 | 44876 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 1 | 216292 | 231459 | 1.070123 | 260333 | 333708 | 117416 | 264084 | 330334 | 116792 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 5 | 253875 | 352708 | 1.389298 | 267292 | 371542 | 117667 | 348916 | 354416 | 94500 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 4 | 228125 | 266334 | 1.167492 | 258625 | 275417 | 47292 | 260583 | 275208 | 26250 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4433209 ns, candidate 4820168 ns, ratio 1.087286. Worst scene: gm-bug5099-clockwise-atomic sample 5 (1.389298).
