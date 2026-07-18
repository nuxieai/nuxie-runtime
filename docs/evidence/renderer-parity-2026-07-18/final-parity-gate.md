# Rive Renderer Performance Parity Gate

Schema: `rive-renderer-perf-parity-gate-v1`
Estimator: `equal-order-ratio-of-median-sums-v1`
Maximum ratio: `1.000000`
Verdict: **PASS**

## Provenance

- Manifest SHA-256: `5bf53a8edb189b12b5b34ae88dcbd9c23958d2964db1f917bb5f9cacf8e78e53`
- Baseline runner SHA-256: `98f37c7c87f4689309a8b37c1ab25db8b0b6445f04debfddae3031e68b00bb97`
- Candidate runner SHA-256: `0c0d932292544d08de1e6a90949abba8865ade4728a5fd956a832d3aeb65c042`
- Generator SHA-256: `701ace876f7b66977a32cd6846bb497fdd064b4c640fe40896b9ce79356b8ee2`
- Baseline source identity: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`
- Candidate source identity: `73314a8d5a4a90b24e4d590df17be89a07d1d776+renderer-dirty-sha256-a57f1051f8e1d4c8b92c82d09d1ac002e404711fb84f1693bf312b8e6efcc1cc`

| run | source report | report SHA-256 | overall | clockwise-atomic | MSAA | strict overall | strict clockwise-atomic | strict MSAA |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/final-run1.json | `d8dbb119fc6a21ac097994a863d929ff001a429a4ecb075e22b618d9be0a6197` | 0.986485 | 0.989737 | 0.982709 | 1.077596 | 1.019672 | 1.158722 |
| 2 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/final-run2.json | `2b3de83d752e1ccb00fc81099de69ca9307ba59516481ba44f9927232af0c36a` | 0.991956 | 0.994327 | 0.989055 | 1.090496 | 1.065286 | 1.126430 |
| 3 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/final-run3.json | `71d05d49be4dfe603f11f4850943e3667a6419ef438f0c3a31c1713c0d8ac8ea` | 0.960918 | 0.974436 | 0.945018 | 1.105252 | 1.085679 | 1.131969 |
| 4 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/final-run4.json | `e4a5f0823c5ccb5de20a59624231fdd8e0fcd5159f200426871a65e242607f9a` | 0.999280 | 0.993619 | 1.006602 | 1.087286 | 1.003719 | 1.204503 |
| 5 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/final-run5.json | `b74ed15a70badc66f1a580974fe90d99d25a1cd0abdbea9887bc8991be64f0a0` | 0.994079 | 0.955561 | 1.043441 | 1.069718 | 0.974605 | 1.200510 |

## Gate result

| mode | median ratio | passing runs | threshold |
| --- | ---: | ---: | ---: |
| overall | 0.991956 | 5/5 | 1.000000 |
| clockwise-atomic | 0.989737 | 5/5 | 1.000000 |
| MSAA | 0.989055 | 3/5 | 1.000000 |

Strict selected diagnostic (non-gating) medians: overall `1.087286`, clockwise-atomic `1.019672`, MSAA `1.158722`.
