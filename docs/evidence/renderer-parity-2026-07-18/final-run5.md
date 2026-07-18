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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 2 | 278625 | 269667 | 0.967849 | 297791 | 430041 | 151416 | 274875 | 351208 | 81833 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 2 | 233208 | 275834 | 1.182781 | 265333 | 280000 | 46792 | 275834 | 288417 | 47417 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 4 | 349417 | 366750 | 1.049605 | 353917 | 358250 | 8833 | 338083 | 384500 | 55125 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 7 | 236875 | 265041 | 1.118907 | 267542 | 324000 | 87125 | 327083 | 334000 | 81667 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 5 | 359125 | 324750 | 0.904281 | 362167 | 387042 | 27917 | 335666 | 421708 | 96958 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 4 | 244042 | 320500 | 1.313299 | 328333 | 342375 | 98333 | 322000 | 334875 | 61584 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 7 | 271750 | 317458 | 1.168199 | 289291 | 441875 | 170125 | 317458 | 452250 | 188875 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 7 | 241125 | 250333 | 1.038188 | 284042 | 285875 | 44750 | 280292 | 287250 | 36917 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 1 | 490083 | 456959 | 0.932411 | 522666 | 557125 | 67042 | 465333 | 504541 | 58332 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 2 | 232000 | 293334 | 1.264371 | 285833 | 295041 | 63041 | 265250 | 294333 | 72250 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 3 | 271750 | 266542 | 0.980835 | 286916 | 363000 | 91250 | 306000 | 440833 | 174291 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 6 | 222250 | 266708 | 1.200036 | 261667 | 313625 | 91375 | 310208 | 318750 | 86000 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 4 | 323875 | 298291 | 0.921007 | 331334 | 343167 | 19292 | 322917 | 382333 | 84042 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 1 | 272208 | 331500 | 1.217819 | 326583 | 339042 | 66834 | 275917 | 331500 | 90416 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 3 | 260875 | 238916 | 0.915826 | 266875 | 357083 | 96208 | 259458 | 360708 | 130708 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 4 | 213042 | 271416 | 1.274002 | 250125 | 271916 | 58874 | 265292 | 272583 | 14916 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4500250 ns, candidate 4813999 ns, ratio 1.069718. Worst scene: gm-batchedconvexpaths-msaa sample 4 (1.313299).
