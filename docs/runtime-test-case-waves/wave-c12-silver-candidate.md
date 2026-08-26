# Wave C12 scripted-property Silver fragment

Status: **CANDIDATE FRAGMENT; PENDING MERGE AND INDEPENDENT REVIEW**

This fragment covers exactly pinned `scripting_properties_test.cpp` ordinals
9-16 at upstream SHA `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
It does not reuse the corpus manifest's `pending-scripted` markers as evidence.

## Census

- 8/8 pinned identities mapped;
- eight individually forceable real expected reds;
- zero passing and zero pending cases;
- no source strings, metadata-only claims, aggregate runner, fake owners, or
  unconditional failure sentinels.

## High-level owner evidence

The isolated test target defines eight distinct tests and one shared execution
owner. Each test contains its literal fixture and action stream. The shared
owner imports with the scripting profile, registers the File VM, instantiates
the real Artboard and StateMachine, attaches scripted occurrences, binds the
real ViewModel, draws through `PersistentFactory<SerializingFactory>`, parses
the resulting SRIV, and compares it against the exact pinned `.sriv` file.

All eight replays reach the comparator and diverge:

- #9 `viewmodel_access`: frame 0, op 32 expects transform, got save;
- #10 `viewmodel_from_instance`: frame 0, op 8 expects makeRenderPaint, got
  frameSize;
- #11 `replace_view_model`: frame 0, op 42 transform `tx` expects 0, got 250;
- #12 `remove_from_list`: frame 0, op 165 expects save, got restore;
- #13 `list_index_script_access`: frame 0, op 80 addRawPath expects 33 fields,
  got 808;
- #14 `scripted_property_image`: frame 0, op 18 expects save, got restore;
- #15 `image_scripting_property_value`: frame 0, op 23 transform `tx` expects
  -702, got -139;
- #16 `reset_shared_viewmodel_instance_test`: frame 0, op 10 expects
  makeRenderPaint, got frameSize.

Case #16 preserves all six draws and timings (0 then five 0.016 advances), five
frame boundaries, two `tri1` trigger writes, and both pointer down/up pairs at
exactly `(45,165)`.

## Gates

- focused non-incremental target: eight ignored expected reds, no failures;
- each of the eight reds forced individually and failed at its live SRIV
  comparison;
- correspondence checker: 157 files / 1,404 cases;
- correspondence checker unit suite: 24/24;
- scoped format/diff/evidence-locator checks: pass;
- non-test `silver-corpus` LLVM IR excludes every Wave C12 Silver test symbol.

No production behavior or shared corpus manifest changed.
