# Pinned public definition input

The JSON files in this directory are the unmodified `dev/defs` tree from
rive-app/rive-runtime `9b3319623a210449145097e9f7c7447cf2e50f3b`, the last public
definition snapshot before d4fe10229b3c148f315ba309924fa837135e7a76 removed it.
They were extracted with `git archive <revision> dev/defs`, stripping the two
leading path components. Builds need neither network access nor Git history.

This is a historical build input, not the current runtime specification.
`make schema` applies `defs/upstream-reconciliation` and then the pending
forward-port definitions in `defs/upstream-overlay`. Current pinned generated
C++ headers and CoreRegistry remain the comparison authority.

At d4fe1022, Folder (102) leaves the runtime registry; the reconciliation marks
that definition editor-only. Declared Id fields retain uint runtime storage
and wire encoding but gain separate registry dispatch metadata. Semantic
bitmask bool getters (989–1009) are emitted by the generator. Editor-only
FractionalIndex and editor extension properties are not runtime schema input.
