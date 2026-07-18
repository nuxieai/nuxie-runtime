# Rive Renderer Performance Parity Gate

Schema: `rive-renderer-perf-parity-gate-v1`
Estimator: `equal-order-ratio-of-median-sums-v1`
Maximum ratio: `1.000000`
Verdict: **PASS**

## Provenance

- Manifest SHA-256: `5bf53a8edb189b12b5b34ae88dcbd9c23958d2964db1f917bb5f9cacf8e78e53`
- Baseline runner SHA-256: `c0be5dea661f44751490e397759bd0b6fe1f8a7526dccf9c8677b6077fed5487`
- Candidate runner SHA-256: `d876c3c4b35ec8d116af3e1f059f51830058b9baabe0090c9e5476212055d4b9`
- Generator SHA-256: `6715a33396819b9d82e591f884b39c213a539e64056df052c81b5bfc9677930b`
- Baseline source identity: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`
- Candidate source identity: `eb0e2527dacd68cf55fc181d124cf619f7d11615+renderer-dirty-sha256-45957a307a7c93058bfbbb3dac41f1b7fc409e902d1f75e627f78c02aa4364ae`

| run | source report | report SHA-256 | overall | clockwise-atomic | MSAA | strict overall | strict clockwise-atomic | strict MSAA |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/attempt2-run1.json | `956e90dd0a0ed379d980484896ce1c63cc38bb41ed4466526d2c1382d06f3f20` | 0.969857 | 0.941193 | 1.022201 | 1.056675 | 0.988782 | 1.150493 |
| 2 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/attempt2-run2.json | `6e3c70dd4c434483a7a6c403c18c8a9f1994c4ced33d18b3b87e150c07c50cf5` | 0.980049 | 0.969374 | 0.996544 | 1.176563 | 1.160581 | 1.200326 |
| 3 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/attempt2-run3.json | `be486ea084a4583ff1c2aae72d97468b7f77e5b66971ddeff672c51efb4fdcc3` | 0.917824 | 0.878946 | 0.988743 | 1.182016 | 1.229503 | 1.114768 |
| 4 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/attempt2-run4.json | `a9eb3d36898cb5f3ae05536728a35b11b9a8786082896b86f80c66104a4e4473` | 0.965830 | 0.965445 | 0.966347 | 1.055995 | 1.027844 | 1.095979 |
| 5 | /Users/levi/.codex/worktrees/3b43/nuxie-runtime/docs/evidence/renderer-parity-2026-07-18/attempt2-run5.json | `ac0ba5ed17bf3277f6692c6fa4a03a6be33fd6262ccef5e828086b0d26c4fbab` | 0.966058 | 0.930346 | 1.019455 | 1.127866 | 1.126457 | 1.129898 |

## Gate result

| mode | median ratio | passing runs | threshold |
| --- | ---: | ---: | ---: |
| overall | 0.966058 | 5/5 | 1.000000 |
| clockwise-atomic | 0.941193 | 5/5 | 1.000000 |
| MSAA | 0.996544 | 3/5 | 1.000000 |

Strict selected diagnostic (non-gating) medians: overall `1.127866`, clockwise-atomic `1.126457`, MSAA `1.129898`.
