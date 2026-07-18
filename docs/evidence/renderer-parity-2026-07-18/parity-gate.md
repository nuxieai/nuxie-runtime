# Rive Renderer Performance Parity Gate

Schema: `rive-renderer-perf-parity-gate-v1`
Estimator: `equal-order-ratio-of-median-sums-v1`
Maximum ratio: `1.000000`
Verdict: **FAIL**

## Provenance

- Manifest SHA-256: `5bf53a8edb189b12b5b34ae88dcbd9c23958d2964db1f917bb5f9cacf8e78e53`
- Baseline runner SHA-256: `c0be5dea661f44751490e397759bd0b6fe1f8a7526dccf9c8677b6077fed5487`
- Candidate runner SHA-256: `2d8009a46bacfad25cdc1c02846d86bc8e7ed9b26d0a6e6b8d6447055381ed78`
- Generator SHA-256: `6715a33396819b9d82e591f884b39c213a539e64056df052c81b5bfc9677930b`
- Baseline source identity: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`
- Candidate source identity: `eb0e2527dacd68cf55fc181d124cf619f7d11615+renderer-dirty-sha256-b297f86d58e1b97e306b4aaa9335f50f2f7c58563e0614cf079c9b1efe9c8bc9`

| run | source report | report SHA-256 | overall | clockwise-atomic | MSAA | strict overall | strict clockwise-atomic | strict MSAA |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | docs/evidence/renderer-parity-2026-07-18/run1.json | `91814f9b1d4c5e3e4277d1677d0a7c1d2987c24b1a7838528264cba476a65227` | 0.996790 | 1.027640 | 0.957097 | 1.042095 | 1.044391 | 1.038783 |
| 2 | docs/evidence/renderer-parity-2026-07-18/run2.json | `8529898733f18179ed7d6a22f2d411508cd603590553f45d7882503a76782d6f` | 0.951455 | 0.908166 | 1.020224 | 1.159638 | 1.163872 | 1.153155 |
| 3 | docs/evidence/renderer-parity-2026-07-18/run3.json | `1ca18ee68473a0d02cff3898df310019deb2d81e3ec2186361186d58a88d82e0` | 1.004079 | 0.998105 | 1.015484 | 1.080310 | 1.065823 | 1.107614 |
| 4 | docs/evidence/renderer-parity-2026-07-18/run4.json | `11d1a0a38ee797a818cc535a9e6f9ac203ebe7b13be2c70e3b5913a5277eeb8c` | 0.997900 | 0.996712 | 0.999352 | 1.134291 | 1.107609 | 1.171061 |
| 5 | docs/evidence/renderer-parity-2026-07-18/run5.json | `1bf5e9791f883a96b23d0ffe41cb9832aa6fa57acdd526931250145cd466bc48` | 1.044855 | 1.072962 | 1.005095 | 1.265111 | 1.317973 | 1.188719 |

## Gate result

| mode | median ratio | passing runs | threshold |
| --- | ---: | ---: | ---: |
| overall | 0.997900 | 3/5 | 1.000000 |
| clockwise-atomic | 0.998105 | 3/5 | 1.000000 |
| MSAA | 1.005095 | 2/5 | 1.000000 |

Strict selected diagnostic (non-gating) medians: overall `1.134291`, clockwise-atomic `1.107609`, MSAA `1.153155`.
