# Editor Next runtime-defect evidence

This directory owns the executable evidence spine for the Editor Next runtime
defect investigation. It does not own runtime, frame-loop, or renderer
production semantics.

Run:

```sh
tools/editor-next-runtime-defects/run-check.sh
```

The check fails closed on:

- the exact pinned Rive C++ commit;
- the landed, explicitly versioned Editor handoff snapshot;
- the complete defect, correction, fixture, and child-count ratchets;
- legal evidence-state transitions;
- the exact active frame-loop writer lease; and
- one-to-one atlas-to-fixture registration, including a canonical per-row
  reproduction digest and the exact standalone C++ probe registry.

Production use requires all three provenance inputs: the Editor source-artifact
root, the pinned Rive checkout, and the provenance-stamped C++ probe executable.
`--test-mode` exists only for isolated temporary unit fixtures; it must not be
used to validate the repository atlas.

`fixtures.toml` is a registry, not proof that a fixture passed. A row may move
from `registered` only when its named direct driver exists and the atlas stores
the corresponding C++, Rust, and Editor evidence. Closed historical rows remain
evidence-only and never receive synthetic probes. Qualified rows must also pin
the SHA-256 of every direct stimulus file; the production checker rehashes those
files from the runtime, pinned C++, and landed Editor roots.

`cpp_probe/registry.cpp` is the standalone C++ probe dispatcher established by
F-ED-00A. Its build script verifies the upstream checkout before compiling and
writes a source-and-executable provenance stamp next to the executable. The
production checker verifies that stamp before accepting `--list`. Individual qualification
slices add real fixture implementations behind registered IDs; registration
alone cannot promote an atlas row.

Renderer rows cannot qualify without the renderer pixel floor and complete
Dawn revision, backend, mode, feature flags, surface, reference executable,
reference stamp, command, and evidence provenance.

Each atlas row also carries its complete port closure. Reported and reproduced
rows may use `pending: REASON` values while localization is in progress.
Beginning at `qualified`, closure fields must be concrete. Revision tables use
either a full 40-hex SHA or `{ status = "pending", reason = "..." }`. The only
late pending revisions are phase-future facts: `merged_repair_sha` before
orchestrator verification, `consumed_runtime_sha` before Editor consumption,
and `consumed_superproject_sha` before closure. Renderer and executor evidence
likewise cannot remain pending after qualification; an orchestrator pass must
name `independent-orchestrator`.
