# Mechanical translation receipts

This directory is the evidence-of-work layer for the Bun-style Metal rewrite.
Manifest role names do not prove that a role executed. Every renderer
translation unit advances only when the corresponding checked-in receipt exists
and the campaign checker validates it.

Each unit eventually has six immutable receipts. The first four close the
per-unit translation loop; the last two belong to the later global compiler
and behavior queues:

1. `<unit>.translation.toml` — Luna xhigh's pinned source coverage, reserved
   outputs, and translation handoff. It records every source digest and must
   report zero omitted lines, declarations, conditionals, and include owners.
2. `<unit>.source-review.toml` — Sol's independent source-semantics review.
   It records the reviewed base/diff, findings, and exact pinned citations.
3. `<unit>.ownership-review.toml` — a separate Sol lifetime/ownership review.
   It records fields, threads, retain/release transfers, completion, drop order,
   unsafe invariants, and divergence dispositions.
4. `<unit>.fix.toml` — Sol's correction receipt, accepted finding resolutions,
   the post-fix source range, and `open_findings = 0`.
5. `<unit>.compile.toml` — Sol's zero-diagnostic compiler receipt, issued only
   after all 41 units have reached `fixed`.
6. `<unit>.verification.toml` — Sol's owner-bound V0–V9 behavior receipt,
   issued only after all 41 units have compiled.

These receipts are produced in global passes. Phase one creates only Luna
translation receipts while all 111 individual files are transliterated in
parallel waves. The 41 units are aggregation and later compiler-diagnostic
boundaries; a multi-file unit cannot receive its translation receipt until
every owned file has a source-shaped target and file-level evidence. Only
after all 111 files are translated does the source-review pass begin, followed
by the separate ownership-review pass and then the correction pass. A review
finding never serializes or chooses work in the still-running translation
pass.

Receipts use full 40-character upstream and workspace base revisions. A receipt
never says merely “tests pass”; it names exact commands, selected test counts,
outputs, and artifact digests. `make metal-port-check` replays those commands
and rejects a nonzero exit or a result count that differs from the receipt.
Translation receipts precede both reviews.
Review receipts are distinct files with distinct `review_run_id` values and
distinct coverage contracts. Canonical review receipts describe the final
clean reruns only: preliminary findings retain stable IDs in the fix receipt,
and fixes cannot close findings until both affected reviews rerun independently.

`pending` in the manifest means no receipt exists yet. The initial preparation
commit intentionally contains only this schema document; Luna has not been
dispatched and no empty success receipts are created in advance.

## Canonical TOML schema

Every receipt is a tracked file under this directory and uses the exact unit id
in both its filename and `unit`. Replace the example hashes and lists with real
evidence; placeholder, empty, or untracked artifacts do not advance a unit.
`workspace_base_ref` is the baseline commit for the current worktree diff. It
must be an ancestor of current `HEAD`; artifact digests bind the current tracked
bytes rather than pretending those bytes already exist at the baseline commit.

Translation (`<unit>.translation.toml`):

```toml
schema_version = 1
unit = "<unit>"
receipt_kind = "translation"
upstream_ref = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
workspace_base_ref = "<40-hex workspace commit>"
role = "luna-extra-high"
open_findings = 0
omitted_lines = 0
omitted_declarations = 0
omitted_conditionals = 0
omitted_include_owners = 0
commands = ["<exact command> :: exit=0 :: count=<selected test count>"]
evidence = ["<tracked evidence path or line-checked cpp:/rust: citation>"]
artifact_digests = { "<every tracked unit output>" = "<matching 64-hex sha256>" }
source_digests = { "<every pinned unit source>" = "<matching 64-hex sha256>" }
```

Source review (`<unit>.source-review.toml`) and ownership review
(`<unit>.ownership-review.toml`) share this shape. Set `receipt_kind` to
`source-review` or `ownership-review`; each is produced by a distinct Sol
review context.

```toml
schema_version = 1
unit = "<unit>"
receipt_kind = "source-review"
upstream_ref = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
workspace_base_ref = "<40-hex ancestor baseline commit>"
role = "sol-high"
open_findings = 0
commands = ["<exact review/check command> :: exit=0 :: count=<reviewed item count>"]
evidence = ["cpp:<owned pinned source>:<line-range>", "rust:<reviewed artifact>:<line-range>"]
artifact_digests = { "<reviewed artifact>" = "<64-hex sha256>" }
findings = []
review_run_id = "<unique review context id>"
coverage = ["owned-source-lines", "declarations", "conditionals", "include-owners", "source-semantics"]
citations = ["cpp:<pinned source>:<line-range>", "rust:<target>:<line-range>"]
```

The ownership-review form instead uses exactly `fields`, `lifetimes`,
`threads`, `retain-release`, `drop-order`, `unsafe-invariants`, and
`divergences` in `coverage`. Source review citations/evidence exactly cover
the unit's owned pinned sources. Both review kinds exactly bind every reviewed
Rust artifact through citations/evidence and current-byte SHA-256 digests.
Copying one review into the other, reusing a run id, or citing an unrelated
valid source/artifact is rejected. The union of each review's citations and
scoped evidence must cover every current line of every owned pinned source and
every reviewed Rust artifact; appending an uncited line invalidates the receipt.

Fix (`<unit>.fix.toml`):

```toml
schema_version = 1
unit = "<unit>"
receipt_kind = "fix"
upstream_ref = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
workspace_base_ref = "<40-hex ancestor baseline commit>"
role = "sol-high"
open_findings = 0
commands = ["<exact post-fix command> :: exit=0 :: count=<verified item count>"]
evidence = ["<tracked post-fix evidence>"]
artifact_digests = { "<fixed artifact>" = "<64-hex sha256>" }
resolutions = ["<stable finding id and exact resolution, or NO_FINDINGS: final clean audit>"]
```

`resolutions` is always nonempty. A clean unit records an exact `NO_FINDINGS:`
audit disposition; a corrected unit preserves every preliminary stable finding
ID and its resolution so the final-clean canonical reviews do not erase history.

Compile (`<unit>.compile.toml`) uses the common fields above, sets
`receipt_kind = "compile"`, and adds `compiler_diagnostics = 0`. Its commands
must identify the exact compiler invocation and positive checked-item count;
its artifact digest map exactly covers the unit outputs. The checker rejects
every compile receipt until all 41 translation units have completed the four-
receipt translation/review/fix loop.

Verification (`<unit>.verification.toml`) sets
`receipt_kind = "verification"` and adds an exact `suite_reports` table with
distinct tracked report paths for `V0` through `V9`. Commands, evidence, and
artifact digests remain owner-bound. No unit may enter `verified` until every
unit is already `compiled`; four translation-loop receipts alone can never
skip the compiler or behavior queues.
