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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 7 | 302041 | 420292 | 1.391506 | 448958 | 536541 | 234500 | 420292 | 431792 | 154500 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 2 | 235959 | 281000 | 1.190885 | 270833 | 286541 | 50582 | 280708 | 323375 | 101750 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 5 | 366083 | 355375 | 0.970750 | 462792 | 845416 | 479333 | 364000 | 808917 | 465958 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 4 | 232625 | 301291 | 1.295179 | 344041 | 363792 | 131167 | 333250 | 394084 | 124750 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 7 | 398000 | 369459 | 0.928289 | 652833 | 823958 | 425958 | 426708 | 675542 | 314125 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 4 | 252334 | 324875 | 1.287480 | 357833 | 403000 | 150666 | 324875 | 344000 | 85292 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 2 | 271500 | 449416 | 1.655308 | 457459 | 582375 | 310875 | 271375 | 458167 | 190292 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 1 | 256083 | 229583 | 0.896518 | 284459 | 309916 | 53833 | 280791 | 297125 | 67542 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 4 | 556458 | 889500 | 1.598503 | 877125 | 927125 | 370667 | 881583 | 1016125 | 517041 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 3 | 272833 | 305209 | 1.118666 | 293750 | 330292 | 57459 | 292542 | 342167 | 93209 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 5 | 295833 | 282250 | 0.954086 | 463500 | 496500 | 200667 | 327959 | 509292 | 227042 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 4 | 259583 | 259042 | 0.997916 | 326708 | 350542 | 90959 | 287542 | 343042 | 103667 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 4 | 329209 | 323709 | 0.983293 | 346667 | 636166 | 306957 | 334500 | 747875 | 432250 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 1 | 217708 | 234167 | 1.075601 | 275875 | 350208 | 132500 | 331000 | 343917 | 111917 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 3 | 290583 | 364542 | 1.254519 | 356375 | 447083 | 156500 | 362209 | 403291 | 144707 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 4 | 256958 | 276625 | 1.076538 | 267042 | 293000 | 36042 | 272958 | 281083 | 21500 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4793790 ns, candidate 5666335 ns, ratio 1.182016. Worst scene: gm-batchedtriangulations-clockwise-atomic sample 2 (1.655308).
