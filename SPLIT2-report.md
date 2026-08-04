# State-machine instance decomposition report

The split mirrors the pinned Rive C++ source tree at commit `4ac7b32798da`. The compatibility hub remains at the original Rust path, retains `StateMachineInstance` storage, and re-exports the public implementation types so existing use paths remain unchanged.

## Module tree

| Rust module | Pinned C++ source counterpart | Lines |
| --- | --- | ---: |
| `crates/nuxie-runtime/src/state_machine/state_machine_instance.rs` | Compatibility hub and retained instance storage for `src/animation/state_machine_instance.cpp` | 373 |
| `state_machine_instance/state_machine_instance.rs` | `src/animation/state_machine_instance.cpp` | 7,888 |
| `state_machine_instance/listener_viewmodel_change.rs` | `src/animation/listener_viewmodel_change.cpp` | 527 |
| `state_machine_instance/text_input_listener_group.rs` | `src/animation/text_input_listener_group.cpp` | 86 |
| `state_machine_instance/data_converter_group.rs` | `src/data_bind/converters/data_converter_group.cpp` | 540 |
| `state_machine_instance/data_bind.rs` | `src/data_bind/data_bind.cpp` | 1,781 |
| `state_machine_instance/data_bind_container.rs` | `src/data_bind/data_bind_container.cpp` | 529 |
| `state_machine_instance/data_bind_context.rs` | `src/data_bind/data_bind_context.cpp` | 451 |
| `state_machine_instance/data_context.rs` | `src/data_bind/data_context.cpp` | 1,282 |
| `state_machine_instance/rive_profile.rs` | `src/profiler/rive_profile.cpp` | 48 |
| `state_machine_instance/viewmodel_instance_trigger.rs` | `src/viewmodel/viewmodel_instance_trigger.cpp` | 300 |
| `state_machine_instance/tests/view_model_listener.rs` | Test body; original `view_model_listener_tests` module retained | 122 |
| `state_machine_instance/tests/scripted_listener_actions.rs` | Test body; original `scripted_listener_action_tests` module retained | 7,511 |

Production modules total 13,805 lines including the 373-line compatibility/storage hub. Extracted test bodies total 7,633 lines.

## Mechanical guarantees

- Production bodies were moved without changing their statement order, public signatures, or call sites.
- Former single-module private access was restored only as sibling visibility inside the compatibility owner.
- Every moved production path was updated in `file-correspondence-manifest.toml` and, where present, `docs/runtime-frame-loop-ownership.toml`.
- Frame-loop ratchets scan the declared production child modules as one logical owner. Literal test-body includes are excluded only from outside-owner policy scans.
- The two extracted test modules retain their original names and collect 1 plus 93 tests respectively.
- The correspondence scatter ratchet remains byte-for-byte at `155/155`.
