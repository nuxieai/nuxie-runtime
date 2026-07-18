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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 7 | 291625 | 400833 | 1.374481 | 327416 | 506959 | 215334 | 315083 | 433458 | 154583 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 5 | 258000 | 292083 | 1.132105 | 277583 | 286084 | 28084 | 292083 | 353167 | 74417 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 2 | 453250 | 443625 | 0.978764 | 748333 | 792750 | 339500 | 747291 | 830334 | 479334 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 2 | 258083 | 361417 | 1.400391 | 341792 | 420083 | 162000 | 346625 | 362125 | 19958 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 1 | 394250 | 388792 | 0.986156 | 656042 | 743583 | 349333 | 631834 | 656709 | 267917 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 1 | 246250 | 336833 | 1.367850 | 350917 | 388458 | 142208 | 342166 | 375083 | 39292 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 6 | 297583 | 262000 | 0.880427 | 442291 | 472208 | 174625 | 335959 | 431875 | 169875 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 1 | 245083 | 251125 | 1.024653 | 269000 | 304500 | 59417 | 274500 | 288917 | 49709 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 3 | 521750 | 489583 | 0.938348 | 882958 | 900708 | 378958 | 876125 | 955709 | 466126 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 6 | 254375 | 302000 | 1.187224 | 300750 | 309000 | 54625 | 290583 | 302000 | 53375 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 1 | 332958 | 304500 | 0.914530 | 462166 | 504542 | 171584 | 299708 | 479250 | 204084 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 1 | 290833 | 322167 | 1.107739 | 312125 | 354041 | 63208 | 315833 | 333708 | 100041 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 1 | 331417 | 314334 | 0.948455 | 590667 | 632625 | 301208 | 593125 | 614291 | 299957 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 1 | 312166 | 271458 | 0.869595 | 333250 | 359167 | 47001 | 329875 | 419292 | 198667 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 6 | 259250 | 246084 | 0.949215 | 348833 | 400833 | 141583 | 355000 | 389459 | 143375 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 4 | 220875 | 262459 | 1.188269 | 272875 | 299375 | 78500 | 262459 | 308000 | 95000 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4967748 ns, candidate 5249293 ns, ratio 1.056675. Worst scene: gm-OverStroke-msaa sample 2 (1.400391).
