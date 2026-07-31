# FL-C5 evidence receipt protocol

Every copied floor receipt must contain the exact production candidate SHA as
its first line:

```text
FLOOR_RECEIPT_TREE_SHA=<40-character production commit SHA>
```

The standing complete-floor reference is the E5 receipt set:

- `floor7-browser.log`
- `floor7-pixel-same.log`
- `floor7-pixel-static.log`
- `floor7-size.log`

Each names immutable combined production candidate
`171b57033845cd5ce20222dd24604c8c2b27d120` on its first line. They replace
the `floor5-*` files as publication evidence. The complete floor5 set is
preserved under `superseded/`; it is historical E4 evidence and must not be
cited for the E5 candidate. All earlier floor generations are historical.

Coordinator floor-policy directive (Levi, 2026-07-30): interim correction
rounds bind only the runtime-library, tools-differential, ordinary-golden, and
scripted-golden fast battery. The complete pixel same-runner, pixel
static-reference, browser/WebGPU, and committed-tree size floor runs once on
the final independently accepted candidate immediately before promotion, not
once per correction round. Six consecutive identical complete-floor cycles,
P2 through P7, provided zero new information for the delivery-semantics
corrections. The floor7 files therefore remain the standing full-floor
reference during E9; they are not `14b18765` receipts and must not be
represented as such.

Upstream boundary commit `afe71e30` on 2026-07-30 removed
`nux-apple-runtime` and Apple packaging to establish a pure engine boundary.
E5 therefore has no Apple/XCFramework acceptance leg. Every historical Apple
receipt, including the original dirty-tree refusal and later clean packaging
runs, remains under `superseded/` as evidence of the coverage that applied
before that boundary. The size floor remains operative.

For future reruns, write the raw log outside this evidence directory and copy
and stamp it with:

```sh
python3 tools/runtime-frame-loop-port/stamp_floor_receipt.py \
  --repo-root "$PWD" \
  --source /path/to/raw-floor.log \
  --destination docs/runtime-frame-loop-fl-c5-evidence/floor-final-name.log \
  --tree-sha <final-production-full-SHA>
```

The wrapper validates the SHA and atomically installs the stamped copy.

Historical disclosure: `superseded/floor2-apple.log` ended with an attempt-1
dirty-tree packaging refusal;
`superseded/floor2-xcframework.log` is its successful clean attempt-2.
`superseded/floor-apple.log`, `floor3-apple.log`, `floor4-apple.log`, and
`floor5-apple.log` preserve the other historical Apple runs. None is
operative after `afe71e30`.

Independent rejection verdicts W39, W40, W45, W46, W47, W50, W51, W52,
W55, W56, and W57 are archived beside the floor receipts so internal
closeouts cannot be confused with independent acceptance. `W53-report.md`
is the round-five corrective handoff for W50/W51/W52. `W58-report.md` is the
round-six corrective handoff that maps the W55/W56/W57 findings to
production, differentials, structural negatives, and the pre-E4 acceptance
run. Corrective reports are not independent acceptance verdicts.

`W66-prereview.md` and `W68-reclear.md` preserve the two pre-freeze scout
verdicts. `W67-report.md` records the round-seven corrective, and
`W69-report.md` records the final alias-resistant ratchet correction. These
scout reports explain the path to the merged E5 candidate; they are not
post-publication independent acceptance verdicts.

`W71-oracle-round7.md`, `W72-standards-round7.md`, and
`W73-flb-round7.md` preserve the three independent round-seven rejection
verdicts. `W74-report.md` is the round-eight corrective handoff that maps
their Blend1D, failing-owner-chain, and structural-detector findings to
production and the fast-suite receipts. As a corrective report, W74 is not
an independent acceptance verdict.

`W76-oracle-round8.md`, `W77-standards-round8.md`, and
`W78-flb-round8.md` preserve the three independent round-eight rejection
verdicts. They record the cfg/resolution/tripwire and allow-suppression
bypasses, non-reproducible detector packaging, premature terminal-error
visibility, weakened BlendDirect proof, and the E6 fingerprint generated from
uncommitted Cargo-lock churn. `W79-report.md` is the round-nine corrective
handoff mapping those findings to immutable candidate `afcb7058`, its
committed standalone detector lockfile, and its fast-suite receipts. As a
corrective report, W79 is not an independent acceptance verdict.

`W81-oracle-round9.md`, `W82-standards-round9.md`, and
`W83-flb-round9.md` preserve the three independent E7 rejection verdicts:
unresolved guarded tails across qualified/module/cross-file re-exports,
fragment-composed macro names, fungible file/kind registry quotas, the
planning-state checklist, incomplete FL-G03 citations, and stale publication
pointers. `W84-report.md` is the round-ten corrective handoff mapping those
findings to immutable candidate `e729dd74`, its candidate-mode checker, and
its fast-suite receipts. It records runtime 726/726, tools differentials
823/823, supplemental `nuxie --lib` 147/147, checker tests 77/77, and both
golden corpora at 317/317 entries plus 647/647 exact segments with zero
divergences; the standalone detector also builds from a clean target with
`--locked`. As a corrective report, W84 is not an independent acceptance
verdict.

The W81/W82/W83 verdicts and W84 corrective report remain archived above as
the superseded E8 round record; they are retained as historical evidence and
are not E9 acceptance receipts.

`W86-oracle-round10.md`, `W87-standards-round10.md`, and
`W88-flb-round10.md` preserve the three independent round-ten rejection
verdicts. All three hold the behavioral axis clean and reject only
detector/registry mechanics: reverse-order token fragments, owner-origin
aliases and wrappers, exhaustive catch-all selection, and forgeable or
same-anchor-relocatable registry sites. `round-specs/W89-round11-spec.md`
preserves the binding corrective specification. `W89-report.md` maps those
findings to immutable candidate `14b18765` and records runtime 726/726, tools
differentials 823/823, supplemental `nuxie --lib` 147/147, candidate-mode
checker tests 83/83, both golden corpora at 317/317 entries plus 647/647 exact
segments, and a clean-cache `--locked` detector build. As a corrective report,
W89 is not an independent acceptance verdict.

## Final review round

| Round | Independent verdicts | Outcome |
| --- | --- | --- |
| 10 | `W86-oracle-round10.md`, `W87-standards-round10.md`, `W88-flb-round10.md` | Triple rejection on detector/registry mechanics; behavioral axis unanimously clean. |
| 11 (final) | `W91-oracle-round11.md`, `W92-standards-round11.md`, `W93-flb-round11.md` | Coordinator acceptance of joint candidate `14b18765` on 2026-07-30 with the five findings recorded in the closure packet's residual-risk register. |

Round 11 is final. There is no round 12. The ownership detector is classified
as a drift lint for the documented ownership convention, validated by the
fixed permanent-negative corpus; the packet makes no
adversarial-soundness claim.
