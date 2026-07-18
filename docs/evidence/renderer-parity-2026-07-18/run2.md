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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 2 | 531167 | 541042 | 1.018591 | 541875 | 569667 | 38500 | 524791 | 543000 | 57750 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 7 | 263208 | 294584 | 1.119206 | 284208 | 372458 | 109250 | 314625 | 329833 | 48541 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 1 | 377708 | 882625 | 2.336792 | 798709 | 901875 | 524167 | 526708 | 882625 | 491459 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 6 | 252292 | 254000 | 1.006770 | 276584 | 393375 | 141083 | 276417 | 376250 | 122250 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 2 | 391417 | 375666 | 0.959759 | 403625 | 703375 | 311958 | 390000 | 667833 | 308792 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 4 | 258417 | 316875 | 1.226216 | 338625 | 371584 | 113167 | 345875 | 381084 | 119459 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 3 | 276542 | 273458 | 0.988848 | 327333 | 541417 | 264875 | 274458 | 373208 | 103041 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 5 | 254916 | 236958 | 0.929553 | 281458 | 289542 | 34626 | 269917 | 303625 | 66667 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 6 | 529709 | 513000 | 0.968456 | 573000 | 900666 | 370957 | 513042 | 882417 | 381292 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 7 | 235958 | 295208 | 1.251104 | 285500 | 299167 | 63209 | 275625 | 295208 | 61583 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 2 | 300042 | 300167 | 1.000417 | 319750 | 465625 | 165583 | 301916 | 475791 | 192457 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 6 | 231541 | 320500 | 1.384204 | 266375 | 329417 | 97876 | 341750 | 410917 | 156333 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 2 | 331000 | 366792 | 1.108133 | 347125 | 623500 | 292500 | 335083 | 608709 | 284167 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 7 | 239875 | 318041 | 1.325861 | 334500 | 355000 | 115125 | 248792 | 331292 | 95208 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 5 | 286875 | 267334 | 0.931883 | 293333 | 385334 | 98459 | 295791 | 363917 | 97001 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 2 | 238917 | 241459 | 1.010640 | 254917 | 272875 | 33958 | 266833 | 273292 | 31833 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4999584 ns, candidate 5797709 ns, ratio 1.159638. Worst scene: gm-OverStroke-clockwise-atomic sample 1 (2.336792).
