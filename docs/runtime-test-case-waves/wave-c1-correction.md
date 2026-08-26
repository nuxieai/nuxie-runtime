# Wave C1 semantic correction candidate

Status: **CANDIDATE; PENDING FRESH INDEPENDENT REVIEW**

Corrects candidate `9e00823cd` after rejection receipt `72bbd4386` against
pinned upstream `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

The `implement` and `tdd` skills were explicitly excluded. This correction
changes tests and evidence only; it does not change production runtime
behavior and does not self-accept Wave C1.

## Exact census

- 62/62 cases mapped; zero pending;
- 55 direct and seven adapted;
- 43 pass and 19 executable expected-red.

All 19 rows rejected by `72bbd4386` were re-adjudicated. Live occurrence
owners now replace raw-file, immutable-graph, fresh diagnostic solve, global
update, generated-property, world-bounds, and tautological proxy evidence.

The seven adaptations are four Rust-ownership adaptations for mesh-index
sharing, whole-occurrence cloning, clipping, and animation sharing; the
accepted collapsed-Solo Taffy adaptation; and two explicit Taffy equivalents
for layout leaf topology. The latter query the post-advance occurrence-owned
provider topology and name the unavailable Yoga node bit as the inapplicable
observable.

## Honest addressability limitations

Three corrected rows remain red specifically because the exact owner surface
is not addressable in Rust. They do not claim behavioral parity:

- grid line offsets read the live post-advance grid LayoutComponent's retained
  Taffy output and fail because it contains only the rectangle, while the
  pinned column, row, and non-grid outputs remain encoded;
- `nearestSnapOffsetInDirection` reads the live ScrollConstraint snapshot
  before and after its real snap-property mutation and fails because that
  owner has no addressable two-axis current/target query; the complete pinned
  call/result table remains encoded but is not claimed as executed;
- pre-advance custom-path intrinsic bounds query the live Shape and its
  noncollapsed PointsPath occurrence and fail on the absent retained/on-demand
  intrinsic path owner.

These are live state-dependent capability failures, not constant `None`,
unconditional panic, test-only surrogate algorithms, or scalar/helper proxy
implementations.

## Gates

- strict pinned identity, ordinal, source line, exact name, classification,
  adaptation, evidence locator, and ignore-reason validation: 62/62 green;
- focused normal evidence: 43/43 pass, with 19 ignored as declared;
- forced expected-red sweep: 19/19 failed individually at the declared live
  state/capability or SRIV seam, zero unexpected passes;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- non-test `nuxie-runtime` build: green, with no Wave C1 test-owner symbols in
  the resulting rlib;
- JSON parsing, scoped formatting, and diff checks: green.

All relied-on Cargo gates ran with both `CARGO_INCREMENTAL=0` and the matching
profile incremental setting disabled. Existing user and other-lane workspace
changes were preserved and are not part of this candidate.
