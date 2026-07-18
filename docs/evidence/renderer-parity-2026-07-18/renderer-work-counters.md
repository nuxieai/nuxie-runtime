# Rive Renderer Work Counters

Schema: `rive-renderer-perf-counters-v2`
Protocol: `rive-renderer-perf-runner-v1`
Manifest SHA-256: `5bf53a8edb189b12b5b34ae88dcbd9c23958d2964db1f917bb5f9cacf8e78e53`
Baseline runner SHA-256: `f291ebded45728b39b47ed0f7585663e2116229dcccc61b176e99b4fc824c385`
Candidate runner SHA-256: `96eb5c8b3d797caaa79f7c34356f6d5649776bde22ddbb2c28ebf38e22853470`
Generator SHA-256: `cda2b9ad477c241fe3e128db2959e13fac8043a5deb5b3bd35da1326d9e5e22e`
Baseline source identity: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`
Candidate source identity: `eb0e2527dacd68cf55fc181d124cf619f7d11615+renderer-dirty-sha256-45957a307a7c93058bfbbb3dac41f1b7fc409e902d1f75e627f78c02aa4364ae`
Capture: 10 warmup + 1 measured frame; timing is directional only.

## Ranked Candidate Excess

| rank | scene | counter | C++ Dawn | Rust wgpu | excess | ratio |
| ---: | --- | --- | ---: | ---: | ---: | ---: |
| 1 | none | none | 0 | 0 | 0 | 1.000 |

## Directional Snapshot

| scene | C++ Dawn ns | Rust wgpu ns | ratio |
| --- | ---: | ---: | ---: |
| gm-CubicStroke-clockwise-atomic | 881417 | 458792 | 0.521 |
| gm-CubicStroke-msaa | 351209 | 289208 | 0.823 |
| gm-OverStroke-clockwise-atomic | 1204375 | 1067166 | 0.886 |
| gm-OverStroke-msaa | 345750 | 336042 | 0.972 |
| gm-batchedconvexpaths-clockwise-atomic | 688708 | 671333 | 0.975 |
| gm-batchedconvexpaths-msaa | 327667 | 352709 | 1.076 |
| gm-batchedtriangulations-clockwise-atomic | 376250 | 424667 | 1.129 |
| gm-batchedtriangulations-msaa | 296041 | 281250 | 0.950 |
| gm-bevel180strokes-clockwise-atomic | 1224375 | 890250 | 0.727 |
| gm-bevel180strokes-msaa | 309334 | 307750 | 0.995 |
| gm-bug339297-clockwise-atomic | 503334 | 540833 | 1.075 |
| gm-bug339297-msaa | 396375 | 310000 | 0.782 |
| gm-bug339297_as_clip-clockwise-atomic | 607667 | 593834 | 0.977 |
| gm-bug339297_as_clip-msaa | 361209 | 341458 | 0.945 |
| gm-bug5099-clockwise-atomic | 778791 | 484459 | 0.622 |
| gm-bug5099-msaa | 310500 | 310459 | 1.000 |
