# Runtime differential fingerprints

The runtime parity lanes publish `nuxie-runtime-differentials/v1` JSON under
`target/runtime-differentials/`. These reports are inventory artifacts, not a
second source of truth: successful reports follow the owning golden or silver
validation, while failed reports retain the gate's non-zero status and
promotion diagnostics.

Each report binds the result to the pinned C++ commit, the Rust commit, runner
binary hashes, the corpus manifest hash, and fixture/baseline hashes. Every row
has one normalized outcome:

Virtual scripted producers are recorded explicitly as `virtual: true`; only
provenance-unknown rows may carry `missing: true`. All file-backed fixtures
and baselines must hash successfully or report generation fails closed. Golden
rows hash input and view-model scripts; silver rows hash declared dependencies
and files loaded by executable actions.

- `exact`: the lane compared exact under its declared verification mode;
- `divergent`: the observed canonical first difference matched the reviewed
  signature;
- `unsupported`: a concrete missing surface or provenance gap remains filed;
- `pending`: the row is inventoried but its lane is not executable yet.

On a failed gate, the captured process exit code makes `gate_status` fail even
when compilation, a signal, or infrastructure stopped the runner before its
own diagnostic. Only cases observed before a fail-fast exit are marked
executed. A failed gate records a runner that did not build with `missing: true`
instead of losing the entire report. A previously divergent row that compares
exact is emitted as
`newly-exact`; a changed signature stays `divergent` but carries
`divergence_check: changed` and the concrete diagnostic. A declared exact row
that differs is emitted as `regressed` with the comparison diagnostic.

The forced-scripted golden lane executes every golden divergence. A divergent
row that becomes exact fails with an explicit promotion diagnostic; a changed
first difference also fails. The executable silver lane does the same for all
runtime silver divergences. Ordinary golden reports mark divergent rows as not
executed because that lane deliberately runs only exact comparisons.

The trusted macOS workflow shards forced-scripted golden and silver into
independent jobs, runs them on main pushes and a twice-weekly schedule, and
uploads each lane's JSON for 30 days. Rust golden runners remain protected by
`tools/golden-runner/rust_runner_provenance.py`; its negative integration tests
rewrite sources without advancing mtimes and prove stale artifacts are rebuilt
in both CI and the landing gate.

Run the focused report tests with:

```sh
make runtime-differential-report-test
```
