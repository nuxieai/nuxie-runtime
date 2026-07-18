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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 6 | 311167 | 392458 | 1.261246 | 317375 | 414708 | 103541 | 308750 | 427750 | 151083 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 5 | 268125 | 242000 | 0.902564 | 273208 | 292916 | 24791 | 277375 | 286000 | 56459 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 7 | 334708 | 333250 | 0.995644 | 341583 | 385667 | 50959 | 347917 | 371417 | 43501 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 7 | 204750 | 316834 | 1.547419 | 284208 | 299625 | 94875 | 307625 | 316834 | 15500 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 1 | 372291 | 339250 | 0.911250 | 386792 | 424375 | 52084 | 362875 | 440417 | 101167 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 4 | 250250 | 270792 | 1.082086 | 289083 | 339667 | 89417 | 270792 | 331666 | 92583 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 1 | 279875 | 276000 | 0.986155 | 312167 | 331666 | 51791 | 281125 | 435458 | 160916 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 4 | 237666 | 283792 | 1.194079 | 279250 | 292500 | 54834 | 282708 | 293625 | 60250 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 6 | 509125 | 478250 | 0.939357 | 532167 | 552458 | 43333 | 490584 | 513167 | 41625 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 3 | 228125 | 288000 | 1.262466 | 273750 | 296209 | 68084 | 253958 | 294625 | 65291 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 7 | 276583 | 269459 | 0.974243 | 285792 | 351375 | 74792 | 272875 | 454709 | 185250 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 4 | 240375 | 309542 | 1.287746 | 279875 | 325250 | 84875 | 256959 | 317625 | 96208 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 3 | 309834 | 317333 | 1.024203 | 323375 | 368458 | 58624 | 366417 | 432125 | 114792 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 7 | 226375 | 241167 | 1.065343 | 322834 | 333917 | 107542 | 282542 | 323917 | 82750 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 5 | 270916 | 310916 | 1.147647 | 280875 | 376792 | 105876 | 293583 | 353167 | 87625 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 3 | 246791 | 252291 | 1.022286 | 257750 | 283792 | 37001 | 271042 | 275417 | 23126 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4566956 ns, candidate 4921334 ns, ratio 1.077596. Worst scene: gm-OverStroke-msaa sample 7 (1.547419).
