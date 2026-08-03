Implemented and committed on `levi/corpus-density`; working tree is clean and four commits ahead of `origin/main`.

Key outcomes:

- Refreshed fixtures first with the requested `rsync` and `make fixtures`.
- Densified all 226 animated t=0-only rows to at least three samples.
- Found and registered 30 post-zero failures as V11–V40 without widening tolerances or removing samples.
- Enrolled 24 of the original 31 pending rows as exact.
- Annotated the remaining seven with concrete LT-1/F6/V41–V44 blockers.
- Final corpus: 356 entries, 319 exact, 27 diverges, 10 not-yet.
- Exact segments increased to 1,051, but the floor remains 673 because the corpus is not entirely exact.
- Full per-entry evidence is in [CORPDEN-report.md](/Users/levi/dev/worktrees/nuxie-mr-c16/CORPDEN-report.md).
- V2 and every new finding are recorded in [parity-gap-register.md](/Users/levi/dev/worktrees/nuxie-mr-c16/docs/parity-gap-register.md:39).
- Added the reusable [densify-corpus.rs](/Users/levi/dev/worktrees/nuxie-mr-c16/tools/golden-compare/src/bin/densify-corpus.rs).

Gates passed:

- `make scripted-golden-compare`
- `make silver-corpus-test`
- `make check`
- `make parity-scorecard-test` — 26 tests
- `cargo test -p golden-compare` — 20 tests
- Targeted rustfmt and `git diff --check`
- Final densifier audit: `densified=0`

One honest V2 caveat remains: assets without an authored loop have no loop wrap to sample, so their finite animation endpoint is used. V2 therefore remains marked partial rather than falsely closed. Nested/library fallback is now restricted to authored `NestedArtboard` dependencies and has a focused control.

Review summary:

- Standards: 0 hard findings; one optional suggestion to additionally run the broader non-scripted golden/workspace-test sequence.
- Spec: blocker annotations were corrected; the no-authored-loop limitation remains explicitly documented.

Commits:

- `9e066390` Add authored-boundary corpus densifier
- `7cb73a4b` Densify and classify runtime corpus
- `4f437ed5` Name pending corpus blockers inline
- `8a68c86a` Follow authored nested animation boundaries