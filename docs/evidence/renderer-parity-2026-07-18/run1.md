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
| gm-CubicStroke-clockwise-atomic | clockwise-atomic | 5 | 302459 | 312250 | 1.032371 | 457334 | 527875 | 225416 | 332875 | 432667 | 137167 | 1 | 3 | 1 |
| gm-CubicStroke-msaa | msaa | 4 | 270334 | 245500 | 0.908136 | 295292 | 328167 | 57833 | 279584 | 338416 | 103541 | 1 | 3 | 0 |
| gm-OverStroke-clockwise-atomic | clockwise-atomic | 5 | 370958 | 417083 | 1.124340 | 382250 | 498292 | 127334 | 368583 | 802625 | 453792 | 1 | 12 | 1 |
| gm-OverStroke-msaa | msaa | 5 | 231125 | 245584 | 1.062559 | 272292 | 344208 | 113083 | 260875 | 334583 | 88999 | 1 | 12 | 0 |
| gm-batchedconvexpaths-clockwise-atomic | clockwise-atomic | 7 | 383375 | 397959 | 1.038041 | 388666 | 425708 | 42333 | 370583 | 397959 | 35084 | 1 | 10 | 1 |
| gm-batchedconvexpaths-msaa | msaa | 3 | 256291 | 267250 | 1.042760 | 264375 | 337750 | 81459 | 267250 | 333791 | 92541 | 1 | 10 | 0 |
| gm-batchedtriangulations-clockwise-atomic | clockwise-atomic | 7 | 303209 | 275750 | 0.909439 | 315041 | 446916 | 143707 | 366250 | 455584 | 179834 | 1 | 4 | 1 |
| gm-batchedtriangulations-msaa | msaa | 2 | 233333 | 262959 | 1.126969 | 245000 | 294500 | 61167 | 253125 | 277291 | 48625 | 1 | 4 | 0 |
| gm-bevel180strokes-clockwise-atomic | clockwise-atomic | 5 | 527125 | 499834 | 0.948227 | 567125 | 911625 | 384500 | 513625 | 919500 | 419666 | 1 | 20 | 1 |
| gm-bevel180strokes-msaa | msaa | 4 | 242625 | 301084 | 1.240944 | 292333 | 322916 | 80291 | 292542 | 301084 | 52959 | 1 | 20 | 0 |
| gm-bug339297-clockwise-atomic | clockwise-atomic | 6 | 315084 | 440041 | 1.396583 | 395542 | 475333 | 160249 | 440041 | 490083 | 202541 | 1 | 2 | 1 |
| gm-bug339297-msaa | msaa | 5 | 222583 | 233458 | 1.048858 | 300708 | 325458 | 102875 | 257084 | 323833 | 90375 | 1 | 2 | 0 |
| gm-bug339297_as_clip-clockwise-atomic | clockwise-atomic | 6 | 328875 | 345334 | 1.050046 | 335500 | 388166 | 59291 | 363916 | 715375 | 384666 | 1 | 2 | 1 |
| gm-bug339297_as_clip-msaa | msaa | 3 | 254291 | 240583 | 0.946093 | 337791 | 353708 | 99417 | 264625 | 363875 | 129833 | 1 | 2 | 0 |
| gm-bug5099-clockwise-atomic | clockwise-atomic | 5 | 287625 | 255583 | 0.888598 | 295958 | 372292 | 84667 | 293042 | 350334 | 94751 | 1 | 1 | 1 |
| gm-bug5099-msaa | msaa | 6 | 242625 | 232541 | 0.958438 | 271084 | 281500 | 38875 | 256209 | 280750 | 48209 | 1 | 1 | 0 |

Aggregate control-selected pairs: C++ 4771917 ns, candidate 4972793 ns, ratio 1.042095. Worst scene: gm-bug339297-clockwise-atomic sample 6 (1.396583).
