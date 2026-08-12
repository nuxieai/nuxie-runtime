# Upstream Sync cycle map

Defines the recurring Upstream Sync
cycle that keeps Nuxie runtime current with `rive-app/rive-runtime` now that
the original C++→Rust port (including the renderer) is complete. Four clean
sync cycles are complete. A read-only weekly drift scout is active. The
write-capable parity worker is also active after meeting its trust-count
threshold. With no standing approvals recorded, its prompt fails closed and
permits blocker-only reporting rather than repository changes. Cycle rows keep
the stable `S<cycle>-<n>` form (for example, `S4-23`); renaming the workflow
does not renumber them.

## Why this works here

The project ported a SNAPSHOT (reference pinned at `7c778d13`, recorded in
CI and the status file). The golden harness turns upstream drift into a
mechanical signal:

- **Detection:** bump the reference pin on a branch → `make golden-compare`
  (and `scripted-golden-compare`) re-runs against the new C++ → every
  upstream behavior change that touches the ported surface shows up as a
  named stream diff on a named file.
- **Proof of port:** a change is fully ported exactly when the ratchet is
  green at the new pin. No judgment call about "did we get it all."
- **Attribution invariant:** after a pin bump, every diff must be attributed
  to a specific upstream commit before any port work starts. An unattributed
  diff means the triage missed something — stop and re-triage.

## Upstream Sync cycle

Each cycle processes `LAST_SYNCED_SHA..<candidate-sha>` from `upstream/main`
(or a chosen release tag — prefer tags once upstream cuts them). The candidate
may move while triage is still open; that extends the same cycle and report.

### 1. Triage in passes (agent, automatic)

Triage is one report assembled through as many passes as the span requires.

1. **Orchestrator preparation.** The orchestrator performs steps a sandboxed
   scout cannot: fetch upstream repository metadata, create or refresh a clean
   candidate checkout under `~/dev/worktrees/`, verify that its `HEAD` is the
   candidate SHA, and clone any vendored dependency revisions needed to build
   the candidate or its oracles. Keep the normal pinned checkout untouched.
2. **Sandboxed scout pass.** Scouts inventory commits and path signatures,
   inspect source and dependency changes, run the manifest check and available
   pin-bump probes, and return citable evidence. A scout reports a blocked
   fetch, missing candidate worktree, or missing dependency clone immediately;
   the orchestrator supplies it and completes any network-blocked probe rather
   than treating missing evidence as a classification.
3. **Top-up pass.** If upstream advances before approval, extend
   `LAST_SYNCED_SHA..<new-candidate-sha>` in the existing
   `docs/sync/triage-<date>-<shortsha>.md`. Preserve existing row IDs and
   append new `S<cycle>-<n>` rows, probe the new final cut, and update totals,
   attribution, version-skew evidence, priority, and deferred staleness in
   place. A top-up is not a second report or a second deferral-age increment.

Every pass runs
`RIVE_RUNTIME_DIR=<clean-candidate-worktree> make port-manifest-check` before
classification. Any missing or stale non-generated `src/**/*.cpp` row
(`src/generated/**` is schema/codegen-owned) is inventory evidence that must
appear in the report; do not regenerate or reclassify the manifest before the
approval gate.

Bucket every commit by path signature:

- `dev/defs/**` → schema (regenerate `nuxie-schema` via `make schema`; usually
  mechanical, occasionally implies new runtime behavior)
- `src/**`, `include/rive/**` → runtime (the core triage surface)
- `renderer/**` → renderer (active sync surface: classify `PORT`, `WATCH`, or
  `SKIP`; renderer Phase R is complete, so renderer changes are never
  auto-deferred)
- `tests/unit_tests/assets/**`, `tests/gm/**` → fixtures (recommend adding to
  corpus — free verification growth)
- `src/lua/**`, `scripting` → scripting bindings surface
- `dependencies/**` → vendored-dep bumps (see Version Skew below)
- build/CI/docs/editor-only → SKIP (listed, one line each)

The deliverable remains one row per non-skipped commit (or per themed group
when upstream lands a feature as a series):

| row | upstream sha | title | bucket | impact | risk | effort | corpus signal | recommendation |

- **Impact** (what Nuxie gains): `critical-fix` (crash/correctness/security)
  / `fix` / `feature` / `perf` / `internal`.
- **Risk** (to our port): `low` (localized, well-covered by goldens) /
  `medium` / `high` (touches retention/dirt/epoch surfaces, float math, or
  import compatibility).
- **Effort**: S (<1 slice) / M (1-3 slices) / L (needs its own mini-map).
- **Corpus signal**: does bumping the pin produce diffs attributable to this
  commit? Run ordinary and scripted pin-bump probes against the verified
  candidate worktree and dependency clones. A runtime-bucket commit with NO
  corpus signal gets flagged: either the corpus lacks coverage (add a fixture)
  or the change genuinely doesn't affect us.
- **Recommendation**: score 0-10 plus verdict `PORT` / `SKIP` / `WATCH`
  (relevant but wait — e.g. half-landed upstream feature series). Rubric
  anchors: 9-10 critical fixes on ported surface;
  7-8 fixes/features Nuxie's paywall/flow content will exercise; 4-6
  features outside current content needs (user judgment); 1-3 internal or
  out-of-scope.

The final-cut probes must account for every changed entry in both directions:
each observed candidate diff maps to a row, and every row that should affect
the enrolled corpus names its signal or its coverage gap. The original
attribution invariant is binding:

- **Attribution invariant:** after a pin bump, every diff must be attributed
  to a specific upstream commit before any port work starts. An unattributed
  diff means the triage missed something — stop and re-triage.

### 2. Approval gate (USER — hard stop)

The agent presents the report and STOPS. No port work or pin movement is
allowed without explicit user approval of specific rows, a standing category
approval recorded in State, or a cycle-scoped authorization recorded in State.
The user may approve a subset; unapproved rows are recorded as
`deferred-<date>` and resurface in the next cycle's report (with a staleness
counter) until approved or skipped. Scheduled automation never infers approval.

### 3. Port by subsystem-owner sets (agent, after approval)

Do not impose one global upstream-order queue on independent subsystems. After
approval, partition rows into dependency-complete subsystem-owner sets with
explicit file ownership and overlap declarations, then schedule them as S4 did:

1. Land crash, correctness, security, and lifetime fixes first.
2. Land foundational schema/runtime/dependency chains serially in dependency
   order; regenerate schema before its runtime consumers.
3. Run disjoint owner sets in parallel, each on its own branch and worktree
   under `~/dev/worktrees/`. Shared files belong to a named overlap set,
   not to two concurrent writers.

Within each set, preserve upstream order. Keep one commit per upstream change
(including an explicitly approved partial-port split), cite the upstream SHA in
the message (`[sync] Port rive-runtime <sha>: <title>`), and use the applicable
goldens, focused differentials, tests, and attribution checks as the oracle.
One set may merge before another only when their ownership and dependency
declarations prove them independent.

### 4. Standing mechanics for distributed landings

- **Deferred-corpus staging.** New fixtures that the currently pinned C++
  oracle cannot build or verify go into a cycle-local staging manifest using
  the `.s4-deferred-corpus.toml` pattern
  (`.s<cycle>-deferred-corpus.toml`). Port commits may add the fixture and only
  their own staging entry; they do not enroll it in `corpus.toml` or generate a
  current-pin expectation. The atomic close enrolls and verifies those entries
  only after the candidate pin and oracle can read them.
- **Commit maps and sandbox reconstruction.** When a worker's sandbox cannot
  write the shared Git index, it writes a commit map naming the intended
  message, exact files/hunks, staging-manifest ownership, exclusions, and gate
  results. The orchestrator reconstructs the commit in a Git-writable worktree.
  Worker reports are claims: before scheduling a reported SHA for merge or a
  dependent set, verify that the object exists and resolves as a commit (for
  example, `git cat-file -e <sha>^{commit}`), then inspect its diff.
- **Semantic merge resolvers.** Any set that overlaps another gets a named
  resolver responsible for combining behavior by upstream owner semantics,
  rerunning the focused oracles, and producing the one-commit-per-upstream
  history. A resolver does not choose one branch wholesale or use a textual
  conflict resolution as evidence of correctness.
- **Attributed divergences.** When an approved port intentionally retains
  current behavior until the pin advances, mark the affected staged corpus
  rows with the upstream SHA and intended disposition.

### 5. Close the cycle atomically

The cycle closes with one landing assembled after all approved port sets are
present. That single landing must contain all of the following:

1. advance every active/current-revision pin listed in State, together with
   `LAST_SYNCED_SHA` and current status/map statements;
2. rebuild candidate-dependent ordinary, scripted, runtime, frame-loop,
   renderer, or other oracles required by the changed surface;
3. enroll every verified cycle-local deferred-corpus entry and remove the
   staging manifest;
4. run the full ordinary, scripted, runtime, frame-loop, attribution, manifest,
   and affected renderer ratchet at the new pin, with zero unattributed
   residual difference; and
5. append the cycle summary, landed commit map, ratchet numbers, and deferred
   rows with staleness counters to the triage report.

The full ratchet must be green at the new pin. Any residual diff = unported
approved change or missed attribution — resolve before landing.
Skipped-by-approval changes that produce corpus diffs get their
affected entries re-expected DELIBERATELY with a Decision-log entry
("diverges from upstream <sha> by choice") — never silently.

Active pins move together; historical evidence does not. Audit pins,
`rust_ref` source citations, prior sync reports, prior fixture provenance, and
historical Phase R oracle revisions stay frozen unless a separately authorized
regeneration/review explicitly names them. Never blanket-rewrite old SHAs while
advancing the current cut.

A vendored dependency change may also change the runner's build contract.
Update its build configuration in the atomic close when required—for example,
S4's Yoga fork made `layout_sizing_style.hpp` include Yoga publicly, so both
runner builds needed the Yoga-renames include path/forced include before the
new-pin oracle could compile.

## Version Skew (special handling, always HIGH priority in triage)

Check these in product-risk order:

1. **.riv format version bumps** (runtime header major/minor): highest
   priority of all — import compatibility is the product promise.
2. **Luau bumps** (`dependencies/**luau**` or bytecode-version changes):
   luaur is pinned against a specific upstream Luau commit. A Rive editor that
   emits newer bytecode versions breaks our scripting. Triage must check
   bytecode-version compatibility explicitly; if luaur lags, options are:
   hold the pin (WATCH), upstream ask to pjankiewicz/luaur, or fall back per
   the recorded mlua contingency.
3. **HarfBuzz/Yoga/SheenBidi/image-codec bumps**: check whether HarfRust/
   Taffy/unicode-bidi/image-crates track the change; text-shaping version skew
   can move golden text streams — attribute and re-verify tolerances
   deliberately.

## Scheduled automation

The active weekly drift scout is read-only: it inventories new upstream work,
checks the repository's pin consistency, and reports a ranked delta queue. It
does not edit a checkout, port code, or open a pull request.

The write-capable Upstream Sync cycle parity worker may be enabled only after
two clean manual cycles have been recorded. That trust-count threshold is now
met, but Standing approvals remains `none`. The worker is active, but its prompt fails
closed and makes no repository changes when no applicable approval exists. It
may run steps 1-2 and act only on standing approvals recorded below; it never
infers approval from an earlier cycle and never merges its own pull request.
The user may pre-approve categories (for example, "auto-port critical-fix +
schema-mechanical with green ratchet"); record the decision here before the
worker may act on that category.

## State

- LAST_SYNCED_SHA: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`
- 2026-08-02: **S4 cycle-scoped authorization (Levi), completed.** All 30 PORT rows of
  `docs/sync/triage-2026-08-02-e0d4913f.md` approved as listed (including the
  partial-port splits S4-27, S4-43, S4-47 and the reviewed waves S4-23,
  S4-38, S4-42). Luau/luaur decision (firmed by Levi 2026-08-02): the engine pin is
  FROZEN until the parity closeout completes — "if it works today, don't
  break it." Upstream Luau 0.730-0.732 revs stay WATCH rows with staleness
  counters, but no sync cycle re-opens the engine question before the
  closeout scorecard is green; skew evidence in scripted differentials is
  the only early-reopen trigger.
  The approved ports landed through PRs #195, #196, #199, #201, and #202,
  followed by the S4C shared-file closeout on `levi/s4-ports-s4c`. The product
  pin advanced to `4ac7b327` only after the full ordinary, scripted, runtime,
  frame-loop, and attribution ratchets were green.
- Clean manual cycles completed: 4 (S4 closed at `4ac7b327`)
- Standing approvals: none
- Current cycle authorization: closed. The S4 authorization was exhausted by
  the 30/30 approved PORT rows and pin-advance closeout; no cycle-scoped
  authorization remains active.
- Current cycle status: S4 behavior and all required ratchets are closed at
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; see the appended cycle summary
  in `docs/sync/triage-2026-08-02-e0d4913f.md`. The uncommitted closeout handoff
  records one remaining standards item: extract the four new layout owners
  from `draw.rs` into direct FLR-16 files before landing. Deferred WATCH rows
  remain queued with updated staleness counters. With Standing approvals at
  `none`, the write-capable worker is fail-closed until a new inventory is
  explicitly authorized.
- Current-revision pin registry (advance with each completed Phase S cycle):
  - `.github/workflows/_trusted-macos.yml` `RIVE_RUNTIME_REF`
  - `.github/workflows/ci.yml` top-level `RIVE_RUNTIME_REF`
  - `.github/workflows/ci.yml` `RIVE_SHADER_RUNTIME_REF`
  - `.github/workflows/ci.yml` `RIVE_SAME_RUNNER_RUNTIME_REF`
  - `Makefile` `RIVE_RUNTIME_REF`
  - `Makefile` `PERF_EXPECTED_RIVE_RUNTIME_REF`
  - `tools/fetch-test-assets.sh`
  - `tools/check-renderer-decoder-provenance.sh`
  - `tools/generate-renderer-shaders.sh`
  - `tools/golden-runner/runtime-provenance.sh`
  - `tools/renderer-dawn-live-reference-bootstrap.sh`
  - `tools/runtime-frame-loop-port/build-trace-runners.sh
- `port-manifest.toml` `upstream_ref` (missed in the S4 close; found stale at the S3 cut)`
  - `tools/runtime-frame-loop-port/capture_trace.py`
  - `docs/runtime-drawing-gaps.toml` `upstream_ref`
  - `docs/runtime-drawing-ownership.toml` `upstream_ref`
  - `docs/runtime-frame-loop-gaps.toml` `upstream_ref`
  - `docs/runtime-frame-loop-ownership.toml` `upstream_ref`
  - `tools/runtime-frame-loop-port/README.md` trace-runner checkout contract
  - `file-correspondence-manifest.toml` `upstream_ref` (not
    `audit_upstream_ref`)
  - `test-correspondence-manifest.toml` `upstream_ref`
  - `docs/parity-gap-register.md` current upstream-reference statement
  - `tools/parity-scorecard/test_parity_scorecard.py` current-pin assertion
- Port-manifest inventory registry (advance these two together whenever an
  approved manifest classification update changes its upstream cut; never
  strand CI and the generated manifest at different revisions):
  - `.github/workflows/ci.yml` top-level `PORT_MANIFEST_RIVE_RUNTIME_REF`
  - `port-manifest.toml` `upstream_ref`
- Historical parity-evidence registry (do not advance during a normal sync;
  regenerate the complete registry only after all enrolled proofs are reviewed
  and recaptured):
  - `parity-evidence-proofs.json` capture refs
- Historical Phase R oracle registry (do not advance during a runtime sync;
  regenerate and review the reference artifacts first):
  - `.github/workflows/ci.yml` `renderer-golden` override
  - `tools/cpp-atlas-mask-oracle/build.sh`
  - `tools/cpp-atlas-mask-oracle/format_test.py`
  - `tools/cpp-atlas-mask-oracle/inventory_msaa_references.py`
  - `crates/nuxie-renderer/src/lib.rs` provenance assertion
