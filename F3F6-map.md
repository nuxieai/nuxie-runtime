# F3F6 sandbox commit map

The sandbox cannot create the external worktree Git index lock. Apply the
following coherent commits from a Git-capable landing environment.

## Commit 1: `Port F6 semantic command queue cases`

- `crates/nuxie-runtime/src/animation.rs`
- `crates/nuxie-runtime/src/objects.rs`
- `crates/nuxie/src/command_queue.rs`
- `crates/nuxie/src/command_server.rs`
- `crates/nuxie/src/lib.rs`
- `crates/nuxie/tests/command_queue.rs`
- `tools/fetch-test-assets.sh`

```text
git add crates/nuxie-runtime/src/animation.rs \
  crates/nuxie-runtime/src/objects.rs \
  crates/nuxie/src/command_queue.rs \
  crates/nuxie/src/command_server.rs \
  crates/nuxie/src/lib.rs \
  crates/nuxie/tests/command_queue.rs \
  tools/fetch-test-assets.sh
git commit -m 'Port F6 semantic command queue cases'
```

## Commit 2: `Record F3F6 command queue evidence`

- `docs/p3f-command-queue-test-ledger.md`
- `file-correspondence-manifest.toml`
- `port-manifest.toml`
- `test-correspondence-manifest.toml`
- `tools/port-manifest/port_manifest.py`
- `F3F6-map.md`
- `F3F6-report.md` (ignored by the report pattern; force-add it)

```text
git add docs/p3f-command-queue-test-ledger.md \
  file-correspondence-manifest.toml \
  port-manifest.toml \
  test-correspondence-manifest.toml \
  tools/port-manifest/port_manifest.py \
  F3F6-map.md
git add -f F3F6-report.md
git commit -m 'Record F3F6 command queue evidence'
```

Fixture note: `make fixtures` populated the ignored fixture tree. The durable
`semantic/simpsons.riv` URL/hash is in `tools/fetch-test-assets.sh`; do not
force-add the downloaded binary unless the landing policy changes.
