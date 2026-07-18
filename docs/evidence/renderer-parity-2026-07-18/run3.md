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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 5 | 521041 | 541208 | 1.038705 | 539166 | 728834 | 207793 | 512417 | 544708 | 125624 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 4 | 270875 | 400750 | 1.479465 | 308541 | 381083 | 110208 | 323542 | 400750 | 97250 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 6 | 797500 | 806375 | 1.011129 | 820084 | 892375 | 94875 | 834333 | 895042 | 108084 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 4 | 264458 | 374250 | 1.415159 | 356208 | 367875 | 103417 | 374250 | 387875 | 112500 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 6 | 450750 | 660167 | 1.464597 | 692542 | 775875 | 325125 | 688833 | 720708 | 63083 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 2 | 347875 | 349334 | 1.004194 | 358709 | 366041 | 18166 | 357709 | 365084 | 15750 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 2 | 416041 | 386583 | 0.929194 | 456417 | 476208 | 60167 | 446292 | 455000 | 68417 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 6 | 281625 | 304667 | 1.081818 | 292375 | 362250 | 80625 | 299875 | 317125 | 34167 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 6 | 897541 | 887417 | 0.988720 | 913334 | 1010500 | 112959 | 893916 | 1038458 | 539833 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 3 | 293750 | 299583 | 1.019857 | 297208 | 301167 | 7417 | 302834 | 309750 | 12709 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 1 | 463875 | 471709 | 1.016888 | 468125 | 499792 | 35917 | 472834 | 480833 | 148708 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 5 | 317417 | 324542 | 1.022447 | 334916 | 386875 | 69458 | 328834 | 355500 | 35375 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 2 | 606500 | 610417 | 1.006458 | 607875 | 635333 | 28833 | 611000 | 617375 | 10250 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 3 | 313417 | 274583 | 0.876095 | 340875 | 382125 | 68708 | 333042 | 344834 | 70251 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 4 | 286708 | 368333 | 1.284697 | 369541 | 396708 | 110000 | 371083 | 377459 | 9126 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 3 | 266209 | 281416 | 1.057124 | 268375 | 279750 | 13541 | 275541 | 281416 | 14291 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 6795582 ns, candidate 7341334 ns, ratio 1.080310. Worst scene: gm-CubicStroke-msaa sample 4 (1.479465).
