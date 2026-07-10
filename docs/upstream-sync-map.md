# Upstream Sync Map (Phase S)

Companion to `docs/porting-map-v2.md`. Defines the recurring workflow that
keeps Nuxie runtime current with `rive-app/rive-runtime` after the V2/M8
migration completes. Activation: blocked by M8; the first run is manual; the
nightly cadence is enabled only after two clean manual cycles.

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
  - `dev/defs/**` → schema (regenerate `rive-schema` via `make schema`;
    usually mechanical, occasionally implies new runtime behavior)
  - `src/**`, `include/rive/**` → runtime (the core triage surface)
  - `renderer/**` → renderer (DEFER: auto-tag `phase-r-backlog` — this list
    becomes Phase R's change-queue when the renderer port activates)
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
- **Recommendation**: score 0-10 plus verdict `PORT` / `SKIP` /
  `DEFER(phase-r)` / `WATCH` (relevant but wait — e.g. half-landed upstream
  feature series). Rubric anchors: 9-10 critical fixes on ported surface;
  7-8 fixes/features Nuxie's paywall/flow content will exercise; 4-6
  features outside current content needs (user judgment); 1-3 internal or
  out-of-scope.

### 3. Approval gate (USER — hard stop)

The agent presents the report and STOPS. No port work, no pin movement,
without explicit user approval of specific rows. The user may approve a
subset; unapproved rows are recorded as `deferred-<date>` and resurface in
the next cycle's report (with a staleness counter) until approved or skipped.

### 4. Port (agent, after approval)

- Port approved commits in upstream order, V2 method: mechanical
  translation, cite the upstream sha in each commit message
  (`[sync] Port rive-runtime <sha>: <title>`), goldens as the oracle.
- Schema changes: regenerate, then port any runtime consumers.
- New fixtures: add to corpus as `not-yet`, triage to exact/gated as usual.
- THEN bump the pin (CI SHA + status file) in the same PR/batch; the full
  ratchet (default + scripted) must be green at the new pin. Any residual
  diff = unported approved change or missed attribution — resolve before
  landing. Skipped-by-approval changes that produce corpus diffs get their
  affected entries re-expected DELIBERATELY with a Decision-log entry
  ("diverges from upstream <sha> by choice") — never silently.
- Update `LAST_SYNCED_SHA` in this file's State section.

### 5. Report back

Cycle summary appended to the triage file: what landed, ratchet numbers at
the new pin, deferred rows, Phase R backlog additions.

## Version Skew (special handling, always HIGH priority in triage)

- **Luau bumps** (`dependencies/**luau**` or bytecode-version changes):
  luaur is pinned against a specific upstream Luau commit. A Rive editor
  that emits newer bytecode versions breaks our scripting. Triage must
  check bytecode-version compatibility explicitly; if luaur lags, options
  are: hold the pin (WATCH), upstream ask to pjankiewicz/luaur, or
  fall back per the recorded mlua contingency.
- **HarfBuzz/Yoga/SheenBidi/image-codec bumps**: check whether HarfRust/
  Taffy/unicode-bidi/image-crates track the change; text-shaping version
  skew can move golden text streams — attribute and re-verify tolerances
  deliberately.
- **.riv format version bumps** (runtime header major/minor): highest
  priority of all — import compatibility is the product promise.

## Nightly automation (after two clean manual cycles)

A scheduled job runs steps 1-2 only: fetch, probe pin-bump on a throwaway
branch, write/refresh the triage report, and notify the user with the
summary + top recommendations. Ports still require the step-3 approval —
the nightly job never writes to the main branch. Once trust is established,
the user may pre-approve categories (e.g. "auto-port critical-fix +
schema-mechanical with green ratchet"); record any such standing approval
as a Decision here before acting on it.

## State

- LAST_SYNCED_SHA: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2` (the V2
  snapshot; set by the migration itself)
- Standing approvals: none
- Phase R backlog: `docs/sync/phase-r-backlog.md` (created on first cycle)
