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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 2 | 313416 | 424459 | 1.354299 | 386500 | 499333 | 185917 | 354167 | 561875 | 288250 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 2 | 225792 | 253209 | 1.121426 | 245167 | 282750 | 56958 | 268958 | 284375 | 52791 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 3 | 358791 | 420458 | 1.171874 | 379166 | 389875 | 31084 | 356625 | 420458 | 74666 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 2 | 251000 | 280958 | 1.119355 | 261709 | 329042 | 78042 | 296916 | 347542 | 94209 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 7 | 384333 | 371417 | 0.966394 | 391292 | 776292 | 391959 | 370625 | 376416 | 12874 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 4 | 247625 | 265375 | 1.071681 | 304125 | 370667 | 123042 | 265375 | 347334 | 103584 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 1 | 288292 | 274708 | 0.952881 | 300375 | 310334 | 22042 | 348167 | 424917 | 150209 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 4 | 268125 | 252458 | 0.941568 | 278459 | 298667 | 30542 | 280417 | 292042 | 43792 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 7 | 519750 | 497542 | 0.957272 | 532500 | 542291 | 22541 | 510792 | 611125 | 124167 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 5 | 252333 | 252083 | 0.999009 | 286500 | 307833 | 55500 | 257834 | 286625 | 41875 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 4 | 309708 | 288833 | 0.932598 | 331042 | 381958 | 72250 | 290667 | 332958 | 49708 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 3 | 242917 | 312208 | 1.285246 | 302792 | 330208 | 87291 | 255666 | 316458 | 83166 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 4 | 332375 | 324875 | 0.977435 | 337000 | 604750 | 272375 | 329250 | 627417 | 310583 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 7 | 236083 | 247834 | 1.049775 | 294542 | 345042 | 108959 | 248500 | 339917 | 105708 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 1 | 279792 | 261750 | 0.935516 | 310958 | 426500 | 146708 | 296209 | 411083 | 153916 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 4 | 237917 | 285958 | 1.201923 | 269375 | 279292 | 41375 | 266375 | 285958 | 53667 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4748249 ns, candidate 5014125 ns, ratio 1.055995. Worst scene: gm-CubicStroke-clockwise-atomic sample 2 (1.354299).
