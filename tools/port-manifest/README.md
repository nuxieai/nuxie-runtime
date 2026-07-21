# Port manifest

`port-manifest.toml` accounts for every hand-authored C++ source under the
upstream runtime's `src/**/*.cpp` tree. Generated object-model sources under
`src/generated/**` are excluded because their provenance is owned by the
schema/codegen gate. At the explicit manifest inventory ref `b73bc675`, this
is exactly 447 files.

The product runtime remains pinned at `d788e8ec` until Phase S approval. That
older revision has 448 in-scope files because it still contains
`src/core/field_types/core_uint64_type.cpp`; checking the candidate-driven
manifest against d788 is therefore expected to fail on that path. This is the
removal drift detector, not a tolerated exception or a product-pin advance.

Each row records an upstream path, one of `ported`, `partial`, `absent`, or
`not-applicable`, the consolidated Rust module when one exists, and a note.
Known gaps are seeded from the parity register; every `absent` row must cite
its `F` id.

Run the blocking check with:

```sh
RIVE_RUNTIME_DIR=/path/to/rive-runtime make port-manifest-check
```

The check fails for missing, duplicate, or stale upstream rows, invalid
statuses, drift from the register seeds, and declared Rust modules that no
longer exist. It prints the exact inventory and status counts on success.

After an approved Phase S classification change, update the classification
rules in `port_manifest.py`, then regenerate against the approved candidate
worktree with:

```sh
RIVE_RUNTIME_DIR=/path/to/rive-runtime make port-manifest-generate
```

Phase S runs the checker against the clean candidate checkout before the
approval gate. A newly added C++ file therefore fails inventory until its row
is reviewed; regeneration is not a substitute for triage approval.
