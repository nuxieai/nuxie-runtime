# Backend global ownership/lifetime/ABI review evidence

This document defines the exact ownership-review receipt contract for the
Vulkan, WebGPU, and WebGL2 port campaign. The authority checkpoint is the
completed source-semantics barrier at
`4af6b0ac961191bfd9b755223e7a52e2865ee004`; the pinned upstream revision is
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

The pass is independent, read-only, SCC-atomic, and adversarial. It actively
ignores the `implement` and `tdd` skills. Reviewers do not correct code and do
not use compiler diagnostics, tests, feature behavior, fixtures, or behavioral
failures to select or delimit work. Structural red is a successful complete
review result; correction belongs to the next gate.

The plan identity is exact:

```toml
review_kind = "global-ownership-lifetime-abi"
review_mode = "independent-read-only-scc-waves"
receipt_directory = "docs/backend-port-ownership-reviews"
source_review_receipt_directory = "docs/backend-port-source-reviews"
severity_order = ["P0", "P1", "P2", "P3"]
finding_id_rules = { component = "OR-CNNN-<positive-decimal-minimum-two-digits>", support = "OR-SUP-<positive-decimal-minimum-two-digits>", overlay = "OR-OVL-NN-<positive-decimal-minimum-two-digits>" }
```

The product boundary remains frozen: Vulkan, WebGPU, and WebGL2 are exact
ported renderers; WebGPU and WebGL2 become explicit editor choices without
automatic fallback; and legacy Rust-WGPU remains until every port independently
passes frozen closeout, after which it is deleted. Ownership review does not
jump to correction, compiler work, rooted execution, browser behavior, editor
wiring, closeout, or deletion.

## Exact evidence set

The new evidence set is exactly 117 tracked immediate files under
`docs/backend-port-ownership-reviews`:

- 115 `component-NNN.ownership-review.toml` receipts, one for every frozen SCC;
- one `support.ownership-review.toml` receipt;
- one `overlays.ownership-review.toml` receipt containing all nine overlays.

Nested, renamed, duplicated, or invented files are extra evidence and fail the
global set check. The new 117 receipts have 117 exact source-review prerequisite
receipts. Every prerequisite path, SHA-256, and byte count is enumerated in
`docs/backend-port-ownership-review-plan.toml`.

The prerequisite source-review tree contains 10,228 logical lines and
1,179,667 bytes. Its tree SHA-256 is
`5ab7b2271288fd1ee5e3de066b2f0c87c1983c17df0a0376e078605a17f30d5f`.
The component prerequisites contribute 5,037 lines and 224,576 bytes, support
contributes 614 lines and 30,922 bytes, and overlays contributes 4,577 lines
and 924,169 bytes. The 67 source findings remain frozen prerequisites: 18 P1,
29 P2, and 20 P3. They are not silently closed, corrected, or automatically
reissued as ownership findings.

The exact byte authority is:

| Class | Files | Logical lines | Bytes | Binding SHA-256 |
| --- | ---: | ---: | ---: | --- |
| Pinned sources | 200 | 55,916 | 2,277,054 | `2eb802438b2fad3e5cd8612319deb22e5e0f9f444649d86f4cb66aa672f1fc91` |
| Translated targets | 188 | 54,129 | 2,114,401 | `8351cbeed2d03ddc7fecce20939983272ab554769a73ecf85352ef6a470410ef` |
| Support artifacts | 52 | 97,253 | 3,871,115 | `ac34add6fef74cbba4444fdc342300a86aa70648bac4aa46a3db6d301b5f625e` |

The four state ledgers are exact:

| Authority | Rows | File SHA-256 | Canonical typed-row SHA-256 |
| --- | ---: | --- | --- |
| Fields | 1,946 | `60663fc752031193390b258556ab08d43032985bcbbe9ed6cbd37e078a8c7d2d` | `a6bb31c8bbdd609cb04883282ad19efbbb3b5abbc2e0037c7713f5a3821d6b89` |
| Lifecycle | 2,431 | `41c7ca51347e341b82b8ba80e3434c00ab239b906a518b6e60f1b5388ddfb7a2` | `d7dff40064460ceb27a89eed93d75e6218f8fc21f100cdcbe8ce426cec0de67d` |
| Configuration | 5,409 | `5865452ffd8392a233b0536a057a5d0ccb0f3f5a5c4788b7623019f309ebfc1e` | `8a25deb0e0f46f8579b254cf5b54f7b51ca2c1775d4528c2e133e3dbbb512315` |
| Dependency | 924 | `345d6904be2271eeb54dfe3cd746a618fa8fd75b890895cb1d42de3d0e7c733c` | `200635550831eca73a1f88aff3ec19679e8ec096c99ca1592809be2883841f6c` |

`docs/backend-port-owner-contracts.toml` contributes eight owner families and is
bound to SHA-256
`b9d6aef8689ef92ac7f50de25c803c4fdf4928e9ac3da632b3536f952b4117a6`.
`docs/backend-port-field-profiles.toml` contributes four campaign profiles and
is bound to SHA-256
`7fcc6aa87d7ef650de4875b749faadb9bd52b7d0995e13bbef21aa92f5852e79`.
Every one of the 135 ownership units has exactly one same-campaign owner-family
membership. Profiles apply to components by campaign, not by the presence of a
field row; this produces 61 component/profile memberships.

The exact coverage array is ordered:

```toml
coverage = [
  "field-and-layout",
  "ownership-transfers",
  "provenance-and-aliasing",
  "callbacks-and-threading",
  "synchronization-and-mapping",
  "failure-and-loss",
  "teardown-and-destruction-order",
  "unsafe-ffi-and-abi",
  "configuration-owner-graphs",
]
```

The overlay receipt appends one item:

```toml
"cross-owner-overlays"
```

## Canonical keys and hashes

A logical-line count is `len(exact_file_bytes.splitlines())`. A nonempty
unterminated final line counts once. A byte count is the length of the exact
file byte string; it is never a character count.

For any raw TSV record `R` and typed prefix `K`, its canonical key is:

```text
K + ":" + join(U+001F, name + "=" + value for name in sorted(R.headers))
```

Header names are sorted by their UTF-8 bytes. Values are the exact decoded TSV
cells with no trimming, newline normalization, case folding, or type coercion.
The four prefixes are `field-raw`, `lifecycle-raw`, `configuration-raw`, and
`dependency-raw`.

For any authority-key set, duplicates are rejected, complete UTF-8 keys are
sorted bytewise, and the sorted keys are joined by one LF with no final LF.
SHA-256 is taken over those bytes. The empty set therefore has digest
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

The same sorted-field rule defines binding keys:

- `source-binding:` covers every field in a component source record;
- `target-binding:` covers every field in a component target record;
- `support-binding:` covers every field in a support artifact record;
- `source-review-receipt-binding:` covers exactly `byte_count`, `id`, `path`,
  and `sha256`;
- `physical-source:`, `physical-target:`, `physical-support:`,
  `physical-artifact:`, `physical-external:`, and `physical-generated:` cover
  exactly `byte_count`, `logical_lines`, `path`, and `sha256`;
- `physical-tree:` covers exactly `byte_count`, `file_count`, `logical_lines`,
  `path`, and `tree_sha256`.

The SCC-partition key is `scc-partition:` plus canonical fields `component_id`,
`order_group`, and `units`, where `units` is the semicolon-joined lexical unit
set. The 115-key digest is
`6dba6dbdf824e9080d850abe19c87ee42a77fda64c692202de767b4da98df3ea`.

An owner-family membership key is `owner-family:` plus canonical fields
`campaign`, `component_id`, `family_id`, and `ownership_unit`. The 135-key
digest is
`8d32180cf28d3d074471f1154674cad905d8e9d640437abb9880cc897c2ceefd`.
A field-profile membership key is `field-profile:` plus `campaign`,
`component_id`, and `profile_id`. The 61-key digest is
`f2d966d1ba14937adcf5165de45fbb8ef36b86a219f60cfa747b11686515107c`.

A cross-component unit seam is `dependency-unit-seam:` plus canonical fields
`dependency_component`, `dependency_unit`, `source_component`, and
`source_unit`. The 452-key digest is
`287f25e6be86c833591fba6fd5459af27d428db7c203f8390030f3d18dac5da1`.
A deduplicated component dependency uses the literal key
`component-dependency:<source component>-><dependency component>`.

Each wave's `component_order_sha256` is SHA-256 over its UTF-8 component IDs in
first SCC-ledger appearance order, joined by one LF with no final LF.

A tree digest is SHA-256 over a concatenation built as follows. Recursively
enumerate regular files below the declared root; reject non-regular members;
sort root-relative POSIX paths bytewise; and append for each member the UTF-8
path, one NUL byte, its complete bytes, and one NUL byte. Directories add no
record. There is no other prefix, separator, or final byte.

## Independent SCC and dependency replay

The order ledger is verified, not trusted circularly. Deduplicating known-unit
pairs from all 924 raw dependency rows yields 565 pairs: 54 self pairs and 511
nonself pairs. Kosaraju's algorithm over the 511 nonself pairs must reproduce
the exact 115 SCC partition and its digest. Of those 511 declared unit edges,
59 remain inside an SCC and 452 cross components. Every cross-component edge
points strictly to a lower `order_group`; the deduplicated graph contains 413
component pairs. The 545 raw dependency rows that cross components have digest
`4bfefd8b3b5a4cd8e5634378df27f12d16bb19d591b167855afdeb284cd08534`.

Every component receipt lists its lexical, deduplicated dependency-component
set. It also binds the complete current bytes of every dependency component's
already admitted ownership receipt. These bindings cover all 413 component
pairs. The nine prerequisite source-review overlay authorities do not
substitute for this graph: together they cover 457 of the 545 cross-component
raw dependency rows and omit 88. The omitted canonical `dependency-raw` set has SHA-256
`1f83c99b7490a0570cb54c8050205aa080f373276dfd92cf150f108c54d8bf59`.
The strengthened ownership-overlay authority unions cover all 545 rows and
omit zero; their omitted-set digest is the empty-set SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
The overlay component memberships co-cover 412 of 413 component pairs; the sole uncovered pair is
`component-084->component-083`.

## Ledgers are prompts, not line scopes

The four ledgers are exhaustive prompts and configuration/owner graph
authority. They never narrow the full-file review. In particular, 87 components
have no extracted field row and 81 have no extracted lifecycle row; each still
receives complete review across every coverage domain applicable to its full
source and target bytes. An empty per-component authority block is required,
has zero records, contains an empty key array, and uses the empty-set digest.

Field records prompt exact identity, type/layout, declaration order,
construction, publication, aliasing, and reverse destruction checks. Lifecycle
records prompt allocation, transfer, submission, mapping, callback, failure,
loss, release, and teardown checks. Configuration records prompt complete
owner graphs for every admitted predicate and profile. Dependency rows prompt
retention, borrow, provenance, ordering, and destruction relationships across
owners. A row's absence never proves that the corresponding semantic category
is absent.

## Admission replay

Every checker mode performs the same immutable-authority replay before reading
an ownership receipt:

1. The upstream checkout must be exactly
   `4ac7b32798da0482e441ef09304dc3b480ed3ee5`; the source-review barrier commit
   `4af6b0ac961191bfd9b755223e7a52e2865ee004` must be a complete ancestor.
2. All source-review authority, all 117 prerequisite receipts, all 200 pinned
   sources, 188 targets, and 52 support artifacts are replayed against their
   exact hashes, logical lines, and byte counts. Updating a target and its
   receipt together cannot redefine ownership authority.
3. Source ownership and the independently reconstructed SCC graph reproduce
   200 owners, 135 units, 115 components, seven waves, 511 nonself unit edges,
   452 cross-component unit seams, and 413 component pairs.
4. All four raw ledgers rederive their exact columns, membership, counts, file
   hashes, and canonical typed-key digests. Each row names a known source and
   the exact owning unit/campaign.
5. The eight owner families and four field profiles replay from exact TOML
   bytes. Each unit matches exactly one same-campaign family. Campaign
   applicability, independently of extracted field rows, rederives all 61
   component/profile memberships.
6. Every retained physical artifact, dependency tree, pinned external file,
   and generated output admitted to an overlay is rehashed from actual bytes.
   Ledger rows, source-overlay counts, or receipt prose cannot stand in for
   physical byte evidence.
7. The plan's seven wave records, nine strengthened overlay records, per-file
   prerequisite list, denominators, rules, and canonical hashes rederive
   exactly.
8. Review work remains read-only and phase-correct: no correction, compiler,
   test, feature, fixture, product-cutover, or deletion work is admitted.

## Wave and structural order

One component receipt covers the complete source, target, state-ledger, owner,
profile, source-review, and dependency union of one SCC. The only structural
sequence is:

```text
g0 -> g1 -> g2 -> g3 -> g4 -> g5 -> g6 -> support -> overlays -> global
```

The exact wave component order is enumerated in the plan. A component in `gN`
requires all prior waves and every declared dependency ownership receipt. Other
same-wave components are independent because every cross-component dependency
is lower-wave. Support requires all 115 components. Overlays require all 115
components and support. No backend-specific subset can advance the global gate.

A prior receipt with open findings is structurally valid. Structural order is
about complete immutable evidence, not green behavior.

## Launch and close transitions

Launch is a two-commit transition. Launch Commit A records the complete launch
authority with `ownership_review_launch_ref = "pending"`. Ownership admission
must reject that sentinel, although source/translation checks may validate and
Commit A may be created. Activation Commit B is a distinct committed child of
A and changes only that manifest field to Commit A's full 40-hex SHA. The
checker requires A to be an ancestor distinct from current `HEAD`, validates
the one-field A-to-B manifest transition, and byte-compares the current plan,
schema, ownership checker, and dynamically imported source-review checker with
their regular Git objects at A. Each of those four frozen files must be a
regular non-symlink worktree file and must have identical A, `HEAD`, stage-zero
index, and worktree bytes. The activated or completed campaign manifest must
also be a regular tracked file whose index and worktree are clean against
`HEAD`; the pending A preparation state is rejected before activation instead.
The ownership checker performs this raw Git/blob comparison before dynamically
importing or executing the source-review checker, both in the live repository
and in the detached C replay checkout.
The checker also extracts and compares only the exact ownership review tool
assignment and two ownership review recipes from `Makefile`, independently in
the A blob, `HEAD` blob, stage-zero index blob, and regular non-symlink
worktree; other Makefile targets remain free to evolve in later phases. Any second assignment
to the ownership checker variable, under any Make assignment operator or
`override`, `export`, `private`, `unexport`, `define`, or `undefine` form, or
any second rule header that names either ownership target is rejected, so an
appended definition cannot silently override the displayed recipes.

```make
BACKEND_PORT_OWNERSHIP_REVIEW_TOOL ?= $(CURDIR)/tools/backend-port/check_ownership_review.py
backend-port-ownership-review-admission:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(BACKEND_PORT_OWNERSHIP_REVIEW_TOOL)" --repo-root "$(CURDIR)" --upstream-root "$(RIVE_RUNTIME_DIR)" --manifest "$(BACKEND_PORT_CAMPAIGN)" --admission
backend-port-ownership-review-check:
	PYTHONDONTWRITEBYTECODE=1 python3 "$(BACKEND_PORT_OWNERSHIP_REVIEW_TOOL)" --repo-root "$(CURDIR)" --upstream-root "$(RIVE_RUNTIME_DIR)" --manifest "$(BACKEND_PORT_CAMPAIGN)"
```

Both targets must remain declared `.PHONY`. A, B, C, and D must remain reachable
as their original commits; squash merges, rebases, or history rewriting that
discard these refs invalidate the authority chain. B and D are each the unique
reachable single-parent direct child of A and C respectively, and each changes
only the campaign manifest.

B's committed tree contains no file below
`docs/backend-port-ownership-reviews`. Admission also requires the live receipt
directory to be absent or empty, so ignored, untracked, or staged preloaded
receipts cannot cross activation. Receipt authoring begins only after B.

After activation, the active campaign manifest has these exact ownership fields:

```toml
source_review_status = "complete"
ownership_review_plan = "docs/backend-port-ownership-review-plan.toml"
ownership_review_schema = "docs/backend-port-ownership-review-schema.md"
ownership_review_receipt_directory = "docs/backend-port-ownership-reviews"
ownership_review_launch_ref = "<launch Commit A SHA>"
ownership_review_status = "active"
active_queue = "ownership-review"
```

No completion-pin field exists while the pass is active. Close Commit C closes
and validates the exact 117-file receipt set while those queue/status values
remain active. C must differ from B. Any number of component-wave receipt
commits may occur between them, but the lexical B-to-C changed-path set is
exactly the 117 canonical receipt paths, every status is an addition, and no
source, target, support, checker, authority, manifest, fixture, or other path
changes. Transition Commit D is a distinct commit: it changes the
ownership status to `complete`, advances the queue to `correction`, and adds
these exact pins for Commit C:

```text
ownership_review_barrier_ref
ownership_review_receipt_tree_sha256
ownership_review_receipt_logical_lines
ownership_review_receipt_bytes
ownership_review_finding_total
ownership_review_p0_findings
ownership_review_p1_findings
ownership_review_p2_findings
ownership_review_p3_findings
ownership_review_finding_id_sha256
```

The barrier ref is Commit C, a full ancestor whose manifest still says
`ownership-review`/`active` and contains no completion pins. The checker reads
all 117 receipt objects directly from that commit, recomputes their tree,
line/byte totals, finding total/severity counts, and finding-ID digest, and
requires every current receipt byte and every manifest pin to be identical.
Because correction and later phases may legitimately change translated target
or support bytes, complete/later replay never derives reviewed authority from
the current tree. It creates an ephemeral local shared clone detached at C,
loads the frozen source-review and ownership authority there, and performs the
exact global structural validation against C. The temporary clone creates no
persistent worktree metadata. It then separately checks the live receipt bytes,
pins, frozen launch files/recipes, and C-to-D manifest transition. Every live
receipt is a tracked regular non-symlink file; the live working bytes, index,
and committed HEAD receipt tree must all be unchanged from C. Receipt tree
hashing sorts the 117 receipt-directory-relative POSIX names and hashes each
name, NUL, full blob bytes, NUL. The finding-ID digest sorts all globally unique
IDs, joins them with LF and no final LF, and hashes the UTF-8 bytes; the empty
set therefore uses SHA-256 of zero bytes.
The close transition is invalid before Commit D exists. Later queues may replay this
closed authority only with `ownership_review_status = "complete"`; they may not
author, reopen, replace, or silently mutate its receipts.

## Component receipt

The canonical filename is
`docs/backend-port-ownership-reviews/component-NNN.ownership-review.toml`.
Its parsed top-level key set is exact:

```text
schema_version, receipt_kind, component_id, units, owner_families,
field_profiles, upstream_ref, workspace_base_ref, role, review_run_id,
review_wave, coverage, owner_contract_bindings, field_profile_bindings,
dependency_components, dependency_ownership_receipts, field_authority,
lifecycle_authority, configuration_authority, dependency_authority,
authority_record_count, authority_sha256, authority_keys,
source_review_receipts, sources, targets, findings, open_findings, attestation
```

The fixed values are:

```toml
schema_version = 1
receipt_kind = "ownership-review-component"
upstream_ref = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
workspace_base_ref = "4af6b0ac961191bfd9b755223e7a52e2865ee004"
role = "sol-high"
coverage = [
  "field-and-layout",
  "ownership-transfers",
  "provenance-and-aliasing",
  "callbacks-and-threading",
  "synchronization-and-mapping",
  "failure-and-loss",
  "teardown-and-destruction-order",
  "unsafe-ffi-and-abi",
  "configuration-owner-graphs",
]
attestation = "reviewed-complete-component-ownership-lifetime-abi-authority"
```

`component_id`, `units`, and `review_wave` mirror the SCC authority. Units keep
SCC-ledger row order. `owner_families` is the unique applicable ID list in
owner-contract TOML declaration order; this matters for the three mixed ORE/
renderer SCCs. `field_profiles` uses field-profile TOML declaration order.

`dependency_components` is the lexical deduplicated lower-wave component set.
`dependency_ownership_receipts` uses the same order and contains one exact
binding per dependency. Source and target records are in lexical POSIX `path`
order.

Those dependency bindings also admit exact source/target citation ranges from
every directly listed dependency component into the consumer component's
finding scope. This is required because 88 cross-component raw dependency rows
are absent from the nine specialized prerequisite source-review overlay
authorities, even though the strengthened ownership-overlay unions cover all
545. In addition, 412 of 413 component pairs have some overlay co-membership;
component-084 to component-083 is the sole pair with none. A consumer finding
that cites dependency bytes still anchors itself with the consumer's own
`dependency-raw` and/or
`component-dependency:<consumer>-><dependency>` authority key; dependency
physical keys are not copied into the consumer's combined authority union.

The four state blocks each have exactly four keys:

```toml
field_authority = {
  kind = "field-raw",
  record_count = 0,
  sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  authority_keys = [],
}
```

The lifecycle, configuration, and dependency blocks have identical shapes and
use kinds `lifecycle-raw`, `configuration-raw`, and `dependency-raw`. Each key
array is sorted and exact. Its count and digest must agree with the array.

The component's combined `authority_keys` is the exact sorted union of:

- `component:<component_id>`;
- all keys from the four state blocks;
- the full owner-family membership key for every unit;
- every applicable component/profile membership key;
- one `source-binding:` key per source record;
- one `target-binding:` key per target record;
- the matching `source-review-receipt-binding:` key;
- one `component-dependency:` key per dependency component.

`authority_record_count` and `authority_sha256` bind that complete union. Across
all 115 components the union has 11,937 keys and digest
`78573b0c03e151f95ff85981a98bb56219a0ea678c48107647a0d8e67f83fa35`.

An owner-contract, field-profile, source-review, or dependency ownership
receipt binding has exactly four keys:

```toml
{ id = "...", path = "...", sha256 = "...", byte_count = 123 }
```

Owner and profile bindings point to their one complete canonical TOML file and
repeat once per applicable ID. Dependency ownership bindings point to canonical
tracked component ownership receipts and are validated against their current
complete bytes. The component's source-review array contains exactly its one
plan-pinned component source-review receipt.

A source record has exactly:

```toml
{
  path = "renderer/src/...",
  sha256 = "...",
  logical_lines = 337,
  byte_count = 12345,
  citation = "source:renderer/src/...:1-337",
  disposition = "translate",
}
```

A target record has the same fields except `disposition`. Every citation is the
exact full current `1-N` range. A nontranslated source contributes a source
record and no target record. Source and target membership is copied from the
exact prerequisite source-review receipt, but hashes, lines, bytes, and full
ranges are replayed independently.

`review_run_id` must start with an alphanumeric, contain at least eight
characters, use only alphanumerics plus `.`, `_`, `:`, or `-`, and must not
contain `placeholder`, case-insensitively.

## Support receipt

The canonical file is
`docs/backend-port-ownership-reviews/support.ownership-review.toml`. Its exact
top-level keys are:

```text
schema_version, receipt_kind, upstream_ref, workspace_base_ref, role,
review_run_id, review_wave, coverage, authority_record_count,
authority_sha256, authority_keys, source_review_receipts, artifacts,
findings, open_findings, attestation
```

Fixed values include:

```toml
receipt_kind = "ownership-review-support"
review_wave = "support"
attestation = "reviewed-complete-support-ownership-lifetime-abi-authority"
```

The source-review binding array contains exactly
`support.source-review.toml`. The 52 artifact records are lexical by path and
each has exactly:

```text
path, sha256, logical_lines, byte_count, citation, artifact_role,
review_overlay, source_authority, disposition
```

The last five source-review metadata fields retain their exact support receipt
values; `citation` is the full `support:path:1-N` range. Support artifacts do
not receive invented owner-family or field-profile mappings. Their combined
authority is 52 `support-binding:` keys plus one source-review binding: 53 keys,
digest
`a7772b03439e94efa03c2a9e523301264d8f9d592b447196e282185a0c01f050`.

## Overlay receipt

The canonical file is
`docs/backend-port-ownership-reviews/overlays.ownership-review.toml`. Its exact
top-level keys are:

```text
schema_version, receipt_kind, upstream_ref, workspace_base_ref, role,
review_run_id, review_wave, coverage, source_review_receipts, overlays,
findings, open_findings
```

Fixed values include:

```toml
receipt_kind = "ownership-review-overlays"
review_wave = "overlays"
```

Coverage is the normal nine-item array followed by `cross-owner-overlays`. The
source-review binding array contains exactly `overlays.source-review.toml`.

There are exactly nine overlay records in the plan's fixed order. Every record
has exactly these keys:

```text
id, ordinal, authority_record_count, authority_sha256, component_ids,
support_paths, source_bindings, target_bindings, support_bindings,
artifact_bindings, tree_bindings, external_bindings, generated_bindings,
authority_keys, component_receipts, support_receipts, attestation
```

The strengthened ownership overlay authorities are:

| Ordinal | Overlay | Ownership records | Ownership SHA-256 |
| ---: | --- | ---: | --- |
| 1 | `shared-authority-consumers` | 5,901 | `184c6ca1862d6fbe2db34ec3e566f8100427d5114934ee978d081f8b5f282820` |
| 2 | `webgpu-to-webgl2-load-store` | 1,008 | `cd5a390847a2c1a52bc47e6c4e3648f4f14463e8a9d68102980e5b98a75341c9` |
| 3 | `generated-authority` | 4,724 | `55a77146963bda10711351a134c134201d467260f793d6c78069520dfa56a17c` |
| 4 | `webgpu-abi` | 5,375 | `799a653ed8a7c8c2c1224e50432c0b27c31d8be524712ae1518286b97f667c61` |
| 5 | `shared-ore-contracts` | 4,304 | `0cd6739ab24db8df95a3b78791c4d8f3b9d2a757d8dbe66194efec5f83ba3c9f` |
| 6 | `shared-renderer-contracts` | 4,794 | `779e3e076f968c05c3b62533eef4b24d33fb48bc1e9a8a74777de7e370c171a9` |
| 7 | `vulkan-vma-adaptation` | 1,997 | `02c42fb360643da1f750f958c9d46504a2e64cd5ca8742bff467e7e77b7f06ee` |
| 8 | `backend-identity-and-browser-bridges` | 5,966 | `9b868e165c375e15fb2c451b6e5a52a7501b0417e104db27f45a7fb6f4c742cb` |
| 9 | `classification-boundary` | 1,301 | `7585df6d303bc46453ebb9a72c07e7d1287a8cc2d125e735b033739ba0b7bb38` |

These are new combined ownership unions, not reuse of the smaller source-
overlay digests. Each union contains the overlay's component and support
identities; four typed ledger subsets; full owner-family and profile membership
keys; an exact `source-review-overlay:` key carrying that source overlay's ID,
record count, and digest; and all seven physical binding categories. The 3,192
source-overlay keys are not copied wholesale: the exact prerequisite receipt
bytes plus each source-overlay identity bind them.

Every non-tree physical binding has exactly:

```toml
{ path = "...", sha256 = "...", logical_lines = 123, byte_count = 4567 }
```

Every tree binding has exactly:

```toml
{
  path = "...",
  tree_sha256 = "...",
  file_count = 49,
  logical_lines = 380913,
  byte_count = 19336710,
}
```

All seven physical arrays are lexical by POSIX path. `artifact_bindings` and
trees are repository-side target evidence. `external_bindings` and
`generated_bindings` are pinned-upstream source evidence. Counts, hashes,
logical lines, bytes, and tree members are recomputed from physical files;
source ledger or receipt metadata cannot replace that replay.

`component_receipts` binds every named component ownership receipt in
`component_ids` order. If `support_paths` is nonempty, `support_receipts`
contains exactly the complete support ownership receipt binding; otherwise it
is empty. Each binding has the standard `id`, `path`, `sha256`, and
`byte_count` shape.

Every overlay record uses:

```toml
attestation = "reviewed-complete-derived-ownership-overlay-authority"
```

## Findings and exact citations

A component or support finding has exactly:

```toml
[[findings]]
id = "OR-C097-01"
severity = "P1"
summary = "Describe one concrete ownership, lifetime, layout, ABI, or unsafe-boundary mismatch"
review_domains = ["ownership-transfers", "teardown-and-destruction-order"]
citations = [
  "source:renderer/src/gl/pls_impl_webgl.cpp:20-35",
  "target:crates/nuxie-renderer/src/mechanical_port/webgl2/renderer_src_gl_pls_impl_webgl_cpp__impl.rs:40-61",
  "authority:docs/backend-port-lifecycle-events.tsv:100-100",
]
authority_keys = ["<one or more exact keys from this receipt's combined authority>"]
```

An overlay finding adds exactly one key:

```toml
overlay_id = "webgpu-to-webgl2-load-store"
```

Finding IDs are exact:

- component: `OR-C<three-digit component number>-<positive decimal ordinal>`;
- support: `OR-SUP-<positive decimal ordinal>`;
- overlay: `OR-OVL-<fixed 01 through 09 overlay ordinal>-<positive decimal ordinal>`.

The ordinal is nonzero, has at least two digits (`01` through `09`, `10` through
`99`, `100`, and so on), has no extra leading zeroes, and is local to its
receipt/overlay. IDs are globally unique across all 117 receipts. Severity is
one of P0-P3. Summary is nonempty.
`review_domains` is a nonempty duplicate-free subset of the receipt coverage.
`authority_keys` is a nonempty duplicate-free subset of the receipt's exact
combined authority. A raw ledger key identifies state authority but never
substitutes for the source/target ranges needed to understand a mismatch.

Every citation has exact grammar:

```text
(source|target|support|authority|source-review):<path>:<first>-<last>
```

`first` is at least one, `last` is not less than `first`, and the range is in
bounds for the exact bound bytes. Component citations may name their own source
and target files; source and target files from every directly listed and
byte-bound dependency component; the six authority files; and their component
source-review receipt. Support citations may name support files, the six
authority files, and the support source-review receipt. Overlay citations may
additionally name repo artifacts and tree members as `target:`, and upstream
external/generated files as `source:`. Reviewers cite every exact side needed
to demonstrate the finding; broad whole-file citations do not replace a
precise mismatch range.

The six `authority:` paths are exactly the field, lifecycle, configuration,
dependency, owner-contract, and field-profile authorities named by the campaign
manifest. No other prose or ledger file is finding authority.

`open_findings` is an integer exactly equal to the finding-record count. It may
be nonzero.

## Structural red is success

Component, support, overlay, partial, and global validation succeed
structurally with open P0-P3 findings. A complete set reports `audit=red` and
the exact open count; it reports `audit=green` only at zero. Structural red
means the independent pass is complete and the correction queue is now fully
defined. It does not waive, downgrade, or close a finding.

The only files created by this pass are the canonical ownership-review
receipts. No source, target, support, manifest authority, prerequisite
source-review receipt, compiler state, test, feature, or fixture is modified.
Corrections begin only after the complete global ownership/lifetime/ABI receipt
set closes structurally.

## Checker modes

Admission mode replays authority without requiring an ownership receipt:

```text
python3 tools/backend-port/check_ownership_review.py \
  --repo-root . \
  --upstream-root /Users/levi/dev/oss/rive-runtime \
  --manifest docs/backend-port-campaign.toml \
  --admission
```

Partial mode validates one canonical tracked receipt and all structural
prerequisites:

```text
python3 tools/backend-port/check_ownership_review.py \
  --repo-root . \
  --upstream-root /Users/levi/dev/oss/rive-runtime \
  --manifest docs/backend-port-campaign.toml \
  --receipt docs/backend-port-ownership-reviews/component-097.ownership-review.toml
```

Global mode omits both mutually exclusive mode flags. It requires the exact
117-file set and proves nonoverlapping component coverage of 135 units, 200
sources, 188 targets, every row in all four ledgers, all owner/profile
memberships, all 413 component dependencies, 52 support artifacts, all nine
strengthened overlays, and global finding-ID uniqueness.
