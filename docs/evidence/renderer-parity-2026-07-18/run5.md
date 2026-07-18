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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 6 | 489625 | 553209 | 1.129863 | 533375 | 583458 | 93833 | 506542 | 553209 | 70001 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 5 | 292792 | 349500 | 1.193680 | 318334 | 353250 | 60458 | 318041 | 372375 | 140542 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 3 | 363542 | 364833 | 1.003551 | 405834 | 851667 | 488125 | 403208 | 834167 | 469334 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 7 | 257792 | 323416 | 1.254562 | 323792 | 369042 | 111250 | 323416 | 383042 | 125917 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 5 | 388042 | 705583 | 1.818316 | 406375 | 731500 | 343458 | 658625 | 790708 | 405541 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 4 | 274292 | 260250 | 0.948806 | 336333 | 365708 | 91416 | 331584 | 367959 | 107709 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 1 | 285500 | 418958 | 1.467454 | 303459 | 374625 | 89125 | 277875 | 418958 | 154292 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 1 | 241833 | 289375 | 1.196590 | 292959 | 348625 | 106792 | 291417 | 370833 | 141958 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 3 | 526416 | 918708 | 1.745213 | 550000 | 912500 | 386084 | 888541 | 918708 | 413958 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 1 | 245084 | 299333 | 1.221349 | 290000 | 302250 | 57166 | 293833 | 299333 | 52208 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 5 | 290875 | 341750 | 1.174903 | 297333 | 315458 | 24583 | 289083 | 482208 | 198125 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 1 | 237000 | 291917 | 1.231717 | 258625 | 332000 | 95000 | 298834 | 328042 | 93834 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 5 | 321625 | 330708 | 1.028241 | 334542 | 613000 | 291375 | 341167 | 725208 | 402791 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 7 | 249542 | 339750 | 1.361494 | 326583 | 350708 | 101166 | 309459 | 339750 | 95042 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 1 | 290459 | 262291 | 0.903022 | 391125 | 451208 | 160749 | 364208 | 413750 | 202667 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 5 | 247250 | 278084 | 1.124708 | 261625 | 268250 | 21000 | 293416 | 320750 | 52833 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 5001669 ns, candidate 6327665 ns, ratio 1.265111. Worst scene: gm-batchedconvexpaths-clockwise-atomic sample 5 (1.818316).
