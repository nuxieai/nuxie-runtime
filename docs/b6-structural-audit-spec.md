# #B-6 Structural Fidelity Audit — Validated Spec

Judge-validated 2026-07-21: the brief below returned DIVERGENT with real,
independently-found mechanisms on the known-bad pre-RB1 data-binding pair
and ISOMORPHIC/ADAPTED on the known-good keyed-animation pair. Two
calibration amendments (mutation-timing gate, coverage clause) are folded
in; without them the raw brief false-positives on import-time constants and
false-negatives on cross-file compensation families.

## The audit question (per manifest row)

For one C++ source file (pin d788e8ec) and its mapped Rust module, compare
ARCHITECTURE — not behavior — on five axes:

(a) **Retained identity** — does Rust retain shared mutable objects where
    C++ holds pointers/rcp, or copy values into flattened records?
(b) **Push vs poll** — polling is a divergence ONLY where C++ pushes for
    the same relationship (confirm C++ registers an observer —
    `addDependent`/`addPropertyObserver` — before flagging a Rust poll).
    Where C++ itself indexes by time/id with no observer (keyed animation,
    `resolve(objectId())`), a Rust binary-search/loop is ISOMORPHIC.
(c) **Update-cycle ordering** — judged by PHASE SEQUENCE equivalence
    (bind, dirt collection, favored-direction application, advance). The
    REPRESENTATION of ordering (extra boolean latches vs one dirt bitset)
    is judged under (e), never double-counted here.
(d) **Ownership/lifecycle** — construction, rebind, teardown at the same
    points.
(e) **Compensation smell — with the mutation-timing gate**: a mechanism
    counts as compensation ONLY if it is written during the
    advance/update/bind cycle to track drift from a source. Fields set
    once at import/build and only read thereafter (type/subscription
    discriminants, precomputed target kinds, static defaults) are
    import-time devirtualization → ADAPTED, not DIVERGENT.
    (Canonical non-example: `data_bind_observed: bool` in animation.rs.)

**Coverage clause (mandatory):** compensation families fan across seam
files. Before any verdict, grep the whole crate for the mechanism family
(generation counters, `*_dirty` container bits, observed/snapshot/candidate
vectors, alias mirrors). A DIVERGENT finding must enumerate off-file family
members; an ISOMORPHIC/ADAPTED verdict on one seam of a multi-file
subsystem must state which sibling files were swept and cleared.

**Verdict thresholds:** ADAPTED requires naming the idiom rule (see
PORTING.md architecture-fidelity rules). DIVERGENT requires >=1 mechanism
passing the mutation-timing gate; keyword matches alone downgrade to
ADAPTED with the explaining idiom rule.

## Named idiom rules (seed set; grow in PORTING.md)

- **own-by-value**: C++ owns via `unique_ptr` and the object is not
  shared → Rust owns by value in a Vec.
- **import-time devirtualization**: precompute a type/subscription
  discriminant at build that C++ recomputes per frame via virtual
  dispatch/registry lookup.
- **rc-refcell-for-rcp**: `Rc<RefCell<..>>` handles for shared C++ `rcp`
  identity; clone shares, deep copy is explicit.
- **weak-sink-for-raw-dependent**: weak dirt-sink registration replacing
  C++ raw-pointer dependents with manual remove.

## Fan-out plan

- Pre-partition the 447 manifest rows into SUBSYSTEM CLUSTERS first
  (data-bind, state-machine, layout, text, animation, shapes, assets,
  importers, ...). Batches must keep one cluster whole: 8-12 pairs per
  batch normally; up to ~15 for isolated single-file subsystems. Expect
  ~40-55 batches. The data-bind cluster gets a dedicated batch containing
  all five seam files.
- Workers are read-only scouts; verdicts fold into a single
  `b6-audit-results` table owned by the spine. DIVERGENT rows become
  RB-n rebuild tickets or user-decided D-rows.

## Per-row output schema

```
row_id, cpp_files[], rust_module, subsystem_cluster,
sibling_files_swept[],
verdict: ISOMORPHIC | ADAPTED | DIVERGENT,
axes:
  retained_identity: {status, idiom_rule?, evidence[file:line]},
  push_vs_poll:      {status, cpp_pushes: bool, evidence},
  update_ordering:   {status, phases_cpp, phases_rust},
  ownership:         {status, evidence},
  compensation:      {status,
                      mechanisms[{name, kind, mutation_gated: true,
                                  cpp_counterpart: none, evidence}],
                      import_time_constants[{name, idiom_rule, evidence}]},
idiom_rules_invoked[], confidence: high|med|low, notes
```

Hard validity rules: DIVERGENT requires >=1 `mutation_gated: true`
mechanism; ADAPTED/ISOMORPHIC on a multi-file cluster requires non-empty
`sibling_files_swept`; every keyword match that did not become a mechanism
must appear under `import_time_constants` with an idiom rule.

## Calibration record

- Known-bad (pre-RB1 data binds at a28f74bc): DIVERGENT — found flattened
  value snapshots, generation counter, poll-by-diff, rescan loops,
  whole-container rebind bit, phase-latch proliferation. Answer-key
  self-grade: full hit on the in-file family (inventory item 6), class
  hits on 1/2/3/5, complete misses on 4/7 (off-file) — which motivated
  the coverage clause and subsystem clustering.
- Known-good (keyed animation): ISOMORPHIC hot path (line-for-line binary
  search + apply structure), ADAPTED storage; the three keyword-bait
  fields correctly classified as import-time devirtualization.
