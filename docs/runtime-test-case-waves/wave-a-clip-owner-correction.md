# Wave A clip owner correction

Status: **PENDING FRESH FULL CONFIRMATION**

Scope: only `tests/unit_tests/runtime/clip_test.cpp#2`, **artboard is
clipped correctly**.

The rejected recording-stream proxy was removed. Its transform-text search and
test-authored `+250` arithmetic never observed the owner exercised by pinned
C++.

The replacement is a direct executable port inside `nuxie-runtime`, where the
test can read the corresponding clone-owned Artboard `m_worldPath` owner:
`runtime_shapes.paint_path_owner(0, World).retained.raw_path`. It imports the
exact `artboardclipping.riv` fixture, instantiates and advances `Center`, checks
the authored origins, and asserts the four exact retained points. It then calls
`frameOrigin(false)`, runs the component update, reads that same retained owner,
and asserts the four exact origin-space points.

The test is expected-red at the concrete owner divergence. The initial retained
points are exactly `(0,0)`, `(500,0)`, `(500,500)`, `(0,500)`. After disabling
frame origin and updating, Rust leaves those points unchanged instead of
rebuilding the retained owner to `(-250,-250)`, `(250,-250)`, `(250,250)`,
`(-250,250)`. No production behavior was changed in this correction.

Focused evidence:

- the ordinary filtered run reports the exact owner test ignored;
- an explicit ignored run executes the complete fixture/owner flow and fails
  only at the documented second four-point comparison; and
- the Wave A shard remains pending fresh full confirmation.
