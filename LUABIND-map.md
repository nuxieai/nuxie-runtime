# LUABIND commit map

The linked-worktree sandbox blocked the final `git add`/`git commit` at `/Users/levi/dev/nuxie-runtime/.git/worktrees/nuxie-fld1/index.lock`. Earlier coherent implementation commits succeeded. Land the remaining changes as two commits:

## Intended commit: `Complete Lua property identity and dispatch semantics`

- `crates/nuxie-runtime/src/scripting.rs`
  - Share post-borrow property callback registries across one retained view-model graph.
  - Notify listeners synchronously before public host facade mutators return; allow VM-owned setters to defer until their userdata borrow is released.
  - Cover scalar, enum, trigger, asset, nested-view-model, and list mutations.
- `crates/nuxie-runtime/src/lib.rs`
  - Re-export the callback registration token used by the scripting crate to unregister watches on drop.
- `crates/nuxie-scripting/src/vm/view_model.rs`
  - Reuse direct property userdata from every named getter, matching pinned `m_propertyRefs` identity.
  - Dispatch Lua mutations after userdata borrows and host mutations immediately after runtime borrows.
  - Preserve setter-specific unknown-key behavior, non-string key errors, and trigger/list assignment errors.
  - Add exact regressions for identity, cross-access removal, host callback timing/value reads, key behavior, blob identity, and trigger error continuation.

Suggested staging command:

```sh
git add crates/nuxie-runtime/src/lib.rs crates/nuxie-runtime/src/scripting.rs crates/nuxie-scripting/src/vm/view_model.rs
git commit -m 'Complete Lua property identity and dispatch semantics'
```

## Intended commit: `Close LT-2 provenance and test ledgers`

- `LUABIND-report.md`
  - Record the borrow-safe listener, stable blob identity, trigger failure-continuation, final focused gates, shared generator-test drift, queued items, and commit inventory.
- `file-correspondence-manifest.toml`
  - Update the `lua_properties.cpp` evidence note for shared wrapper identity and post-borrow synchronous dispatch while retaining `pending-verification`.
- `port-manifest.toml`
  - Align the generated `lua_properties.cpp` Rust-owner list and LT-2 note with the generator source.
- `test-correspondence-manifest.toml`
  - Remove the over-broad `scripted properties can be passed to luau` claim.
  - Add the directly exercised stable-blob-identity case, preserving the 14/22 ratchet.
- `tools/port-manifest/port_manifest.py`
  - Make the five LT-2 legacy rows generator-owned `ported` classifications with their current Rust owners and evidence notes.
- `tools/port-manifest/test_port_manifest.py`
  - Update the three former tracked-gap expectations and the two consolidated-owner expectations.
  - Add `test_generate_promotes_lt2_lua_binding_rows`, an exact five-row generator test.
- `LUABIND-map.md`
  - Preserve this fallback staging map.

Suggested staging command:

```sh
git add LUABIND-report.md LUABIND-map.md file-correspondence-manifest.toml port-manifest.toml test-correspondence-manifest.toml tools/port-manifest/port_manifest.py tools/port-manifest/test_port_manifest.py
git commit -m 'Close LT-2 provenance and test ledgers'
```

Fixtures were synchronized and `make fixtures` passed, but they are gitignored and no fixture delta belongs in this lane. If landing inspection identifies an intentional fixture addition, it must be staged explicitly with `git add -f fixtures/<exact-path>`; do not force-add the fixture tree wholesale.
