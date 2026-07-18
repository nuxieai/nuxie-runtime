# Rive Renderer Performance

Schema: `rive-renderer-perf-v3`
Estimator: `cpp-control-min-paired-v1`
Pair order: `counterbalanced-scene-sample-v1`
Runner protocol: `rive-renderer-perf-runner-v1`
Manifest SHA-256: `5bf53a8edb189b12b5b34ae88dcbd9c23958d2964db1f917bb5f9cacf8e78e53`
Baseline runner SHA-256: `c0be5dea661f44751490e397759bd0b6fe1f8a7526dccf9c8677b6077fed5487`
Candidate runner SHA-256: `d876c3c4b35ec8d116af3e1f059f51830058b9baabe0090c9e5476212055d4b9`
Generator SHA-256: `6715a33396819b9d82e591f884b39c213a539e64056df052c81b5bfc9677930b`
Baseline source identity: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`
Candidate source identity: `eb0e2527dacd68cf55fc181d124cf619f7d11615+renderer-dirty-sha256-45957a307a7c93058bfbbb3dac41f1b7fc409e902d1f75e627f78c02aa4364ae`

| scene | mode | C++ selected sample | C++ selected ns | paired candidate ns | paired ratio | baseline p50 ns | baseline p95 ns | baseline spread ns | candidate p50 ns | candidate p95 ns | candidate spread ns | logical flushes | draws | atomic strategy partitions |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 4 | 403292 | 551792 | 1.368220 | 523417 | 563375 | 160083 | 519042 | 602500 | 118583 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 6 | 251375 | 322792 | 1.284105 | 320375 | 346917 | 95542 | 322792 | 384959 | 114292 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 6 | 377291 | 394916 | 1.046715 | 400917 | 841833 | 464542 | 394916 | 968916 | 619166 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 3 | 282792 | 367000 | 1.297774 | 338625 | 398959 | 116167 | 365208 | 443375 | 181542 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 6 | 400167 | 414625 | 1.036130 | 664916 | 693250 | 293083 | 417250 | 685083 | 309458 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 1 | 259667 | 332542 | 1.280648 | 344333 | 414166 | 154499 | 344708 | 436458 | 182500 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 7 | 326916 | 448750 | 1.372677 | 446541 | 477917 | 151001 | 358334 | 464167 | 190584 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 1 | 260791 | 301125 | 1.154660 | 288166 | 301042 | 40251 | 280458 | 306000 | 58583 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 4 | 525292 | 493167 | 0.938844 | 541458 | 552334 | 27042 | 519166 | 885542 | 392375 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 3 | 243291 | 245584 | 1.009425 | 268833 | 300250 | 56959 | 270167 | 306458 | 60874 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 6 | 285000 | 333208 | 1.169151 | 308541 | 328958 | 43958 | 285542 | 333208 | 57833 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 2 | 229792 | 233250 | 1.015048 | 241583 | 323292 | 93500 | 265750 | 322500 | 89250 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 4 | 330083 | 322625 | 0.977406 | 366625 | 618583 | 288500 | 365625 | 613875 | 293708 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 4 | 246667 | 234125 | 0.949154 | 302250 | 346750 | 100083 | 263166 | 321583 | 89375 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 3 | 287417 | 347584 | 1.209337 | 311500 | 401083 | 113666 | 262959 | 397458 | 144875 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 6 | 262167 | 264667 | 1.009536 | 271167 | 299125 | 36958 | 268667 | 303000 | 38333 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4972000 ns, candidate 5607752 ns, ratio 1.127866. Worst scene: gm-batchedtriangulations-clockwise-atomic sample 7 (1.372677).
