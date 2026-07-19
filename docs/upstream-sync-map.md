# Upstream Sync Map (Phase S)

Companion to `docs/porting-map-v2.md`. Defines the recurring workflow that
keeps Nuxie runtime current with `rive-app/rive-runtime` after the V2/M8
migration completes. M8 and renderer Phase R are complete. The first run is
manual. A read-only weekly drift scout may run immediately; the write-capable
parity worker remains paused until two clean manual cycles are recorded.

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

## The cycle

Each sync cycle processes `LAST_SYNCED_SHA..upstream/main` (or a chosen
release tag — prefer tags once upstream cuts them; smaller, themed batches
beat giant ones).

### 1. Fetch + inventory (agent, automatic)

- `git -C $RIVE_RUNTIME_DIR fetch` and list new commits with stats.
- Bucket every commit by path signature:
  - `dev/defs/**` → schema (regenerate `nuxie-schema` via `make schema`;
    usually mechanical, occasionally implies new runtime behavior)
  - `src/**`, `include/rive/**` → runtime (the core triage surface)
  - `renderer/**` → renderer (active sync surface: classify `PORT`, `WATCH`,
    or `SKIP`; renderer Phase R is complete, so renderer changes are never
    auto-deferred)
  - `tests/unit_tests/assets/**`, `tests/gm/**` → fixtures (recommend
    adding to corpus — free verification growth)
  - `src/lua/**`, `scripting` → scripting bindings surface
  - `dependencies/**` → vendored-dep bumps (see Version Skew below)
  - build/CI/docs/editor-only → SKIP (listed, one line each)

### 2. Triage report (agent, automatic — the deliverable)

Write `docs/sync/triage-<date>-<shortsha>.md`: one row per non-skipped
commit (or per themed group of commits when upstream lands a feature as a
series):

| upstream sha | title | bucket | impact | risk | effort | corpus signal | recommendation |

- **Impact** (what Nuxie gains): `critical-fix` (crash/correctness/security)
  / `fix` / `feature` / `perf` / `internal`.
- **Risk** (to our port): `low` (localized, well-covered by goldens) /
  `medium` / `high` (touches retention/dirt/epoch surfaces, float math, or
  import compatibility).
- **Effort**: S (<1 slice) / M (1-3 slices) / L (needs its own mini-map).
- **Corpus signal**: does bumping the pin produce diffs attributable to this
  commit? (Run the pin-bump probe on a branch to find out — diffs are listed
  per commit.) A runtime-bucket commit with NO corpus signal gets flagged:
  either the corpus lacks coverage (add a fixture) or the change genuinely
  doesn't affect us.
- **Recommendation**: score 0-10 plus verdict `PORT` / `SKIP` / `WATCH`
  (relevant but wait — e.g. half-landed upstream feature series). Rubric
  anchors: 9-10 critical fixes on ported surface;
  7-8 fixes/features Nuxie's paywall/flow content will exercise; 4-6
  features outside current content needs (user judgment); 1-3 internal or
  out-of-scope.

### 3. Approval gate (USER — hard stop)

The agent presents the report and STOPS. No port work or pin movement is
allowed without explicit user approval of specific rows, a standing category
approval recorded in State, or a cycle-scoped authorization recorded in State.
The user may approve a subset; unapproved rows are recorded as
`deferred-<date>` and resurface in the next cycle's report (with a staleness
counter) until approved or skipped. Scheduled automation never infers approval.

### 4. Port (agent, after approval)

- Port approved commits in upstream order, V2 method: mechanical
  translation, cite the upstream sha in each commit message
  (`[sync] Port rive-runtime <sha>: <title>`), goldens as the oracle.
- Schema changes: regenerate, then port any runtime consumers.
- New fixtures: add to corpus as `not-yet`, triage to exact/gated as usual.
- THEN bump every active pin listed in State (plus the status/map state) in
  the same PR/batch; the full
  ratchet (default + scripted) must be green at the new pin. Any residual
  diff = unported approved change or missed attribution — resolve before
  landing. Skipped-by-approval changes that produce corpus diffs get their
  affected entries re-expected DELIBERATELY with a Decision-log entry
  ("diverges from upstream <sha> by choice") — never silently.
- Update `LAST_SYNCED_SHA` in this file's State section.

### 5. Report back

Cycle summary appended to the triage file: what landed, ratchet numbers at
the new pin, and deferred rows.

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

The write-capable Phase S parity worker remains paused until two clean manual
cycles have been recorded. When enabled, it may run steps 1-2 and act only on
standing approvals recorded below; it never infers approval from an earlier
cycle and never merges its own pull request. Once trust is established, the
user may pre-approve categories (for example, "auto-port critical-fix +
schema-mechanical with green ratchet"); record the decision here before the
worker is enabled.

## State

- LAST_SYNCED_SHA: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2` (the V2
  snapshot; set by the migration itself)
- Clean manual cycles completed: 0
- Standing approvals: none
- Current cycle authorization: on 2026-07-18 the user requested and authorized
  an ad hoc exact sync through `d788e8ec6e8b598526607d6a1e8818e8b637b60c`,
  including applicable renderer changes and the final pin advance. This
  authorization expires when that cycle closes.
- Current cycle status: the implementation audit is closed and the candidate
  runtime pin is staged in CI. The final full default and scripted candidate
  ratchets have not yet been recorded, so `LAST_SYNCED_SHA`, the manual-cycle
  count, and this authorization remain open.
- Current-revision pin registry (advance with each completed Phase S cycle):
  - `.github/workflows/ci.yml` top-level `RIVE_RUNTIME_REF`
  - `tools/fetch-test-assets.sh`
  - `tools/check-renderer-decoder-provenance.sh`
  - `tools/generate-renderer-shaders.sh`
- Historical Phase R oracle registry (do not advance during a runtime sync;
  regenerate and review the reference artifacts first):
  - `.github/workflows/ci.yml` `renderer-golden` override
  - `tools/cpp-atlas-mask-oracle/build.sh`
  - `tools/cpp-atlas-mask-oracle/format_test.py`
  - `tools/cpp-atlas-mask-oracle/inventory_msaa_references.py`
  - `crates/nuxie-renderer/src/lib.rs` provenance assertion
