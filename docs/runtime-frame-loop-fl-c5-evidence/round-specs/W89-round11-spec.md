# W89 — Round-11 corrective spec (joint FL-B / FL-C5 acceptance)

Frozen review basis: publication `4ecce48a` (candidate `e729dd74`). Round-10 verdicts:
W86 (oracle), W87 (standards), W88 (FL-B) — all REJECT, all exclusively on
detector/registry mechanics. The behavioral/spec axis is unanimously clean:
W86 verified the runtime `crates` tree object is byte-identical across
`afcb7058`/`e729dd74`/`4ecce48a`; W87 replayed all ten round-9 evasions
successfully tripping; W88 confirmed FL-B behaviorally clear for the fifth
consecutive round with the full regression table.

**Scope discipline: this corrective touches ONLY
`tools/runtime-frame-loop-port/**` (detector + checker + tests) and docs.
ZERO production-crate changes. If you believe a production change is needed,
STOP and report instead.**

## Binding findings to close (all five)

### F1 — Macro fragment scanning is order-sensitive (W86 BLOCKING)

`reverse_join!(events, notify_)` expanding to `notify_events` escapes because
fragment scanning concatenates identifier tokens in lexical order only
(main.rs:813 region). Required closure: within each macro invocation /
attribute token stream, after normalization, trip if the guarded name can be
segmented into a concatenation of ANY ordering of ANY subset of the identifier
tokens present (order-insensitive composition). Implementation guidance: for
each guarded name, run a segmentation search over the token multiset (the
guarded-name list is small and names are short — a memoized DP over the name
with the token set is plenty). A single token equal to the full guarded name
must still trip (existing behavior). False-positive control: a macro whose
tokens cannot compose any guarded name must stay clean.

Permanent negatives: (a) the exact `reverse_join!(events, notify_)` form from
W86 verbatim; (b) a three-fragment out-of-order composition; (c) FP control.

### F2 — Owner-module-origin aliases and wrappers escape (W86 + W88 BLOCKING)

The checker wholly skips owner files (check.py:64, :1646/:1647), so:
- W86 form: `state_machine_instance.rs` re-exports
  `RuntimeNestedAnimationInstance::StateMachine as Chosen`; a non-owner
  matches `Chosen(owner)` — no guarded final segment anywhere in the
  non-owner file.
- W88 form: a neutral `pub fn DELIVER(...)` wrapper in the skipped owner file
  calls `notify_events`; a non-owner calls
  `state_machine_instance::DELIVER(...)`.

Required closure: owner files are no longer skipped — they are scanned in a
dedicated **export-audit mode**: guarded-mechanic hits *inside* owner files
remain allowed (they are the owners), but every `pub`/`pub(crate)` `use ... as`
re-export whose source resolves to a guarded item, and every `pub`/`pub(crate)`
fn/method whose body contains a guarded-mechanic hit, contributes its exported
name to a guarded-alias set. That set feeds the non-owner scan pass the same
way the existing audio cross-file alias propagation does (check.py:1580) —
generalize that propagation to ALL kinds, not just audio. A non-owner use of a
propagated name trips with the originating owner export cited. Fail-closed
rule: an owner export the analyzer cannot resolve (unresolved tail) also
propagates as guarded.

Permanent negatives: (a) W86's owner-origin `Chosen` re-export + non-owner
match, verbatim; (b) W88's neutral `DELIVER` wrapper + non-owner call,
verbatim; (c) FP control: an owner pub fn that touches no guarded mechanic,
consumed by a non-owner, stays clean.

### F3 — Registry anchor forgery and same-anchor relocation (W86 + W87 + W88 BLOCKING)

Registry identity is `(file, kind, anchor, guarded_name)`; `site_offset` is
computed but excluded from the key (check.py:1672/:1680), and
`qualified_anchor` omits enclosing *outer* functions (main.rs:680/:1003). Two
demonstrated attacks: (a) W88 — a local fn named
`dispatch_nested_key_input_at_focus` forged inside an unrelated method
produces exactly a blessed anchor key; (b) W86/W87 — relocating a blessed
selection to a forbidden branch of the same function consumes the same row.

Required closure, both parts:
1. **Full nesting path in the anchor**: `qualified_anchor` must include EVERY
   enclosing named item (modules, impls, and ALL enclosing fns), so W88's
   forged local fn anchors as
   `ArtboardInstance::dispatch_nested_text_input_at_focus::dispatch_nested_key_input_at_focus`
   and no longer collides with the blessed
   `ArtboardInstance::dispatch_nested_key_input_at_focus`.
2. **Site content-hash binding**: each detector hit additionally emits a
   `site_hash` = SHA-256 of the normalized token stream of the minimal
   enclosing *statement* containing the guarded occurrence (normalization:
   strip whitespace/comments, keep token text). Registry rows in `gaps.toml`
   gain a required `site_hash` field; the checker matches hits to rows on
   `(file, kind, full_anchor, guarded_name, site_hash)`. Any relocation or
   edit of a blessed site changes the hash → unmatched hit (trip) + unmatched
   row (registry-drift error) — both directions fail closed, and legitimate
   refactors require an explicit, reviewable re-blessing of the hash.

Update the existing registry rows in `docs/runtime-frame-loop-gaps.toml` with
their real full anchors and site hashes (derive them by running the detector on
the actual candidate sources — do NOT hand-compute).

Permanent negatives: (a) W88's forged-local-fn attack verbatim (must produce a
distinct anchor AND fail row-matching); (b) same-anchor relocation with
preserved anchor and changed `site_hash` (W87's site-87→site-146 shape);
(c) FP control: exact blessed sources still pass; (d) drift controls: stale
`site_hash` in registry with no matching hit errors, and vice versa.

### F4 — Exhaustive catch-all selection of a guarded variant (W87 BLOCKING)

```rust
match animation {
    RuntimeNestedAnimationInstance::Simple { .. } => {}
    RuntimeNestedAnimationInstance::Remap { .. } => {}
    selected => move_policy(selected),
}
```
isolates `StateMachine` without naming it; the analyzer keys only on final
path segments. Required fail-closed rule: if ANY pattern within a `match` (or
`if let` / `let ... else` / `matches!`) contains a path any of whose segments
resolves to — or lexically names, under the unresolved-tail rule — the guarded
ENUM (`RuntimeNestedAnimationInstance`, in any resolved/aliased spelling),
then every wildcard (`_`) or binding catch-all arm of that same construct
records a **selection hit** (kind `selection`), anchored normally. Naming a
non-guarded variant (`Simple`, `Remap`) through the guarded enum is what marks
the construct; the catch-all is what trips. FP control: a match over an
UNRELATED enum that merely has variants named `Simple`/`Remap` (guarded enum
name absent from every pattern path) stays clean; a fully-enumerated match
over the guarded enum with NO catch-all in a non-owner file must still trip
via the existing `StateMachine` arm rule (unchanged behavior).

Permanent negatives: (a) W87's exact catch-all form; (b) an aliased-enum
spelling of the same shape (alias resolves to the guarded enum); (c) the FP
control above; (d) `matches!`-negation complement form:
`if !matches!(a, R::Simple{..}) && !matches!(a, R::Remap{..}) { use(a) }` —
this must also trip (the marking rule covers it since the patterns name the
guarded enum and `a` is then used — treat the enclosing `if` body's use of the
scrutinee binding as the catch-all analog; if that is materially harder,
implement the simpler rule that ANY `matches!`/pattern mention of the guarded
enum in a non-owner file with kind `selection` requires a registry row — state
clearly in the report which rule you implemented).

### F5 — NON-BLOCKING prose (W86 + W87)

- `docs/runtime-frame-loop-fl-c5-closure.md` says "Twelve"/"all twelve"
  adversarial rows at :480/:669/:998 — there are thirteen ("Permanent
  structural ratchets"). Fix all occurrences.
- `docs/parity-closeout-status.md:1010` canonical NEXT is one publication step
  stale — update it to the current state (round-11 corrective of candidate
  `e729dd74` under review; E9 to follow).

## Non-negotiable protocol

- **Red-first**: for EVERY new negative, first demonstrate the evasion escapes
  the round-10 detector (or reproduce the reviewer's replay), then implement,
  then show it trips. Record each red→green pair in the report.
- Never weaken any existing test or negative. All ten round-9 evasions and all
  round-10 negatives must still trip. All existing FP controls must stay green.
- Fast suites only (interim round policy): checker test suite
  (`test_check.py`, currently 77 — will grow), runtime lib tests, tools
  differentials, nuxie lib tests, both goldens. NO floor legs.
- Clean-cache `--locked` detector build from a temp CARGO_TARGET_DIR
  (`mktemp -d` under the repo or `$TMPDIR`) — and DELETE the temp dir after
  (use python3 shutil, not rm).
- `cargo fmt --all -- --check`, `git diff --check`, clean porcelain of
  tracked files. Do NOT commit. Do NOT touch `.flc5/` contents other than
  writing your report.
- Do not regenerate the trace fingerprint — that is E9's job, done separately.
- If any closure requires loosening a rule to pass an existing test, that is a
  finding, not a fix: stop and report.

Report to `.flc5/out/W89-report.md`: per-finding red→green evidence, files
touched, suite tallies, and any deviations.
