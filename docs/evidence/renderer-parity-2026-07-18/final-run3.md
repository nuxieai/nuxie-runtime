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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 3 | 275250 | 332083 | 1.206478 | 292709 | 428625 | 153375 | 298125 | 401083 | 135833 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 6 | 237458 | 259333 | 1.092122 | 268417 | 282625 | 45167 | 265708 | 292166 | 77125 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 6 | 345375 | 335792 | 0.972253 | 354791 | 359334 | 13959 | 335792 | 371875 | 42667 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 5 | 259500 | 333709 | 1.285969 | 320208 | 325166 | 65666 | 325833 | 333709 | 109209 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 3 | 352792 | 347750 | 0.985708 | 363833 | 377166 | 24374 | 361625 | 431250 | 101541 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 5 | 234000 | 245542 | 1.049325 | 334125 | 342416 | 108416 | 319375 | 336458 | 90916 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 4 | 262000 | 269375 | 1.028149 | 276875 | 418709 | 156709 | 269750 | 317292 | 50084 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 6 | 214916 | 260125 | 1.210357 | 283666 | 297250 | 82334 | 274292 | 283083 | 37916 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 1 | 497208 | 475417 | 0.956173 | 535542 | 553292 | 56084 | 472000 | 491667 | 29417 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 1 | 224584 | 284084 | 1.264934 | 274625 | 294667 | 70083 | 284084 | 293583 | 75792 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 6 | 272791 | 407000 | 1.491985 | 290625 | 411750 | 138959 | 292791 | 441250 | 168042 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 2 | 217875 | 252625 | 1.159495 | 300583 | 324666 | 106791 | 259500 | 319583 | 87666 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 7 | 307875 | 315833 | 1.025848 | 310917 | 378041 | 70166 | 312458 | 353250 | 48583 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 1 | 263708 | 239667 | 0.908835 | 332792 | 337542 | 73834 | 249625 | 328750 | 116416 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 7 | 266125 | 317167 | 1.191797 | 337167 | 372417 | 106292 | 317167 | 359833 | 100541 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 6 | 237625 | 263958 | 1.110817 | 262334 | 276375 | 38750 | 232666 | 263958 | 54875 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4469082 ns, candidate 4939460 ns, ratio 1.105252. Worst scene: gm-bug339297-clockwise-atomic sample 6 (1.491985).
