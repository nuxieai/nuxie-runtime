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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 5 | 521583 | 494084 | 0.947278 | 568417 | 637833 | 116250 | 519166 | 536042 | 79209 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 7 | 232208 | 284791 | 1.226448 | 332083 | 364042 | 131834 | 347041 | 386042 | 110376 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 5 | 365583 | 353292 | 0.966380 | 781834 | 833917 | 468334 | 390542 | 791333 | 440250 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 6 | 258541 | 338167 | 1.307982 | 357625 | 400250 | 141709 | 342292 | 376000 | 74959 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 6 | 398416 | 376875 | 0.945933 | 433958 | 763042 | 364626 | 474125 | 855167 | 490625 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 7 | 347417 | 339500 | 0.977212 | 373167 | 503750 | 156333 | 373125 | 458375 | 194000 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 1 | 280542 | 455167 | 1.622456 | 327583 | 468125 | 187583 | 453459 | 514708 | 220083 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 2 | 256500 | 290750 | 1.133528 | 297333 | 311583 | 55083 | 291875 | 337625 | 63667 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 3 | 529000 | 505084 | 0.954790 | 679708 | 904250 | 375250 | 526458 | 1064166 | 565707 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 3 | 301167 | 348708 | 1.157856 | 322333 | 388708 | 87541 | 321791 | 408791 | 117624 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 5 | 467000 | 490125 | 1.049518 | 497833 | 531291 | 64291 | 490125 | 502667 | 180667 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 5 | 232042 | 308500 | 1.329501 | 337333 | 430875 | 198833 | 313333 | 348583 | 128666 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 1 | 326500 | 638541 | 1.955715 | 336708 | 480667 | 154167 | 464708 | 687416 | 360499 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 6 | 259041 | 370333 | 1.429631 | 284708 | 352458 | 93417 | 331500 | 370333 | 143499 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 1 | 271541 | 354458 | 1.305357 | 328208 | 404959 | 133418 | 336250 | 373208 | 113542 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 5 | 238583 | 270542 | 1.133953 | 282875 | 302125 | 63542 | 270542 | 305042 | 65083 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 5285664 ns, candidate 6218917 ns, ratio 1.176563. Worst scene: gm-bug339297_as_clip-clockwise-atomic sample 1 (1.955715).
