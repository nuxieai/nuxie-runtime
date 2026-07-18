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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 5 | 306584 | 403500 | 1.316116 | 318625 | 415417 | 108833 | 403500 | 431166 | 126625 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 4 | 238417 | 264792 | 1.110626 | 272792 | 280916 | 42499 | 273083 | 311375 | 81625 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 7 | 356250 | 394916 | 1.108536 | 362959 | 380000 | 23750 | 365500 | 413625 | 70375 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 1 | 266583 | 314833 | 1.180994 | 319542 | 339417 | 72834 | 266500 | 339041 | 87750 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 4 | 382000 | 366959 | 0.960626 | 390583 | 457417 | 75417 | 371416 | 391500 | 30208 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 7 | 256417 | 254916 | 0.994146 | 328291 | 338542 | 82125 | 314250 | 346041 | 91125 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 6 | 297167 | 288667 | 0.971397 | 301333 | 326750 | 29583 | 288667 | 421375 | 149500 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 4 | 245750 | 277334 | 1.128521 | 278584 | 285333 | 39583 | 274459 | 281583 | 16875 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 4 | 516583 | 515375 | 0.997662 | 524917 | 560750 | 44167 | 492500 | 524708 | 34250 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 3 | 257834 | 299208 | 1.160468 | 291167 | 314667 | 56833 | 286542 | 299208 | 42875 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 6 | 316042 | 301000 | 0.952405 | 323875 | 460375 | 144333 | 299625 | 320750 | 33708 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 4 | 212375 | 276709 | 1.302926 | 238875 | 307834 | 95459 | 307375 | 316125 | 39416 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 7 | 305291 | 314666 | 1.030708 | 311375 | 346375 | 41084 | 316292 | 344750 | 43125 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 1 | 223958 | 223208 | 0.996651 | 290250 | 325250 | 101292 | 261458 | 331500 | 108292 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 1 | 259292 | 332959 | 1.284108 | 274250 | 357500 | 98208 | 269833 | 361458 | 105958 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 3 | 220334 | 253625 | 1.151093 | 270916 | 272833 | 52499 | 257708 | 266792 | 21376 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4660877 ns, candidate 5082667 ns, ratio 1.090496. Worst scene: gm-CubicStroke-clockwise-atomic sample 5 (1.316116).
