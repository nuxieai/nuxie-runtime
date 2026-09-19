# Converter recovery and semantic read parity

## Semantic Boolean reads

Upstream `d4fe10229b3c148f315ba309924fa837135e7a76`,
`include/rive/generated/core_registry.hpp::getBool`, supplies the semantic
virtual-property getters missing from the original `4ac7b327` registry.
The Rust registry already had their setters and concrete getters, but omitted
the property-key dispatch for reads. A reverse BooleanNegate binding therefore
read `false` irrespective of the target and overwrote its source with `true`.

Port only those existing getters. No binding-order change is needed. The native
registry regression covers all 21 existing semantic Boolean keys; the publisher
qualification covers changed two-way selection in a cold-imported artifact.

## Protected conversion failures

Pinned upstream `4ac7b32798da0482e441ef09304dc3b480ed3ee5`,
`src/scripted/scripted_data_converter.cpp::applyConversion`, stores a new cached
value only when `rive_lua_pcall_with_context` succeeds. On failure it returns the
existing cache. Our translated converter already did that, but the embedding's
generic callback-failure side channel caused the C player to poison the entire
occurrence at host-command commit.

Keep conversion diagnostics separate from host transaction vetoes. A protected
conversion error with no host commands emitted by that call remains diagnostic;
the existing converter cache and next valid invocation continue working. Preserve
fatal treatment of resource exhaustion and failed callbacks that emitted host
commands, so partially produced commands cannot escape. This distinction is
internal to the scripting host; no C ABI, SDK API, or converter language changes.

Regressions cover the diagnostic, next-edit recovery, converter-side command
rollback, native player survival, and the existing action/drawable/transition
transaction failures. Passing these native checks does not replace signed iOS
artifact and browser qualification.
