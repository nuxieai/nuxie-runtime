# Rive Renderer Performance

Schema: `rive-renderer-perf-v3`
Estimator: `cpp-control-min-paired-v1`
Pair order: `counterbalanced-scene-sample-v1`
Runner protocol: `rive-renderer-perf-runner-v1`
Manifest SHA-256: `5bf53a8edb189b12b5b34ae88dcbd9c23958d2964db1f917bb5f9cacf8e78e53`
Baseline runner SHA-256: `c0be5dea661f44751490e397759bd0b6fe1f8a7526dccf9c8677b6077fed5487`
Candidate runner SHA-256: `2d8009a46bacfad25cdc1c02846d86bc8e7ed9b26d0a6e6b8d6447055381ed78`
Generator SHA-256: `6715a33396819b9d82e591f884b39c213a539e64056df052c81b5bfc9677930b`
Baseline source identity: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`
Candidate source identity: `eb0e2527dacd68cf55fc181d124cf619f7d11615+renderer-dirty-sha256-b297f86d58e1b97e306b4aaa9335f50f2f7c58563e0614cf079c9b1efe9c8bc9`

| scene | mode | C++ selected sample | C++ selected ns | paired candidate ns | paired ratio | baseline p50 ns | baseline p95 ns | baseline spread ns | candidate p50 ns | candidate p95 ns | candidate spread ns | logical flushes | draws | atomic strategy partitions |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 7 | 293042 | 280875 | 0.958480 | 453583 | 468459 | 175417 | 423666 | 447875 | 167000 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 6 | 243125 | 308125 | 1.267352 | 284542 | 310833 | 67708 | 305166 | 324125 | 65750 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 1 | 362333 | 770291 | 2.125920 | 403292 | 794209 | 431876 | 752666 | 784125 | 427250 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 6 | 333417 | 371042 | 1.112847 | 338083 | 394875 | 61458 | 344167 | 479542 | 208750 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 1 | 383375 | 363917 | 0.949246 | 417458 | 675917 | 292542 | 385958 | 682750 | 318833 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 1 | 262917 | 338292 | 1.286687 | 345333 | 355375 | 92458 | 345583 | 382000 | 64292 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 2 | 301667 | 286333 | 0.949169 | 445583 | 450958 | 149291 | 296000 | 441125 | 166167 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 1 | 224542 | 283208 | 1.261270 | 283584 | 307916 | 83374 | 281208 | 312916 | 50124 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 3 | 551958 | 524875 | 0.950933 | 892083 | 923167 | 371209 | 875000 | 947541 | 429500 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 3 | 251958 | 309750 | 1.229372 | 291000 | 313542 | 61584 | 306750 | 312167 | 71417 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 6 | 297750 | 298250 | 1.001679 | 322667 | 468958 | 171208 | 323792 | 494667 | 211292 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 5 | 238416 | 244333 | 1.024818 | 318708 | 337166 | 98750 | 281125 | 330125 | 87791 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 1 | 334167 | 332292 | 0.994389 | 366917 | 616542 | 282375 | 334875 | 602500 | 276958 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 5 | 238959 | 235166 | 0.984127 | 249875 | 335666 | 96707 | 248750 | 333459 | 98293 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 1 | 295708 | 266625 | 0.901650 | 391958 | 470417 | 174709 | 364916 | 380958 | 114333 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 5 | 252959 | 306417 | 1.211331 | 265500 | 275292 | 22333 | 278041 | 306417 | 41792 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4866293 ns, candidate 5519791 ns, ratio 1.134291. Worst scene: gm-OverStroke-clockwise-atomic sample 1 (2.125920).
