# Data Binding Graph Number Public Update Target-To-Source Runtime Contract

## Purpose

Admit the converter-free public `updateDataBinds(true)` target-to-source slice
for a main-`ToTarget | TwoWay` number bind.

C++ state-machine bindable-property actions preserve a manual main-`ToTarget |
TwoWay` target edit through `advancedDataContext()` and replay the unchanged
source during the next normal state-machine advance. The public
`DataBindContainer::updateDataBinds(true)` path is different: when that same
dirty bind is drained with `applyTargetToSource=true`, C++ writes the edited
target value back to the source, then reapplies source-to-target during the
same update.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A main-`ToTarget | TwoWay` number data bind with no converter.
- Immediate source-to-target reapplication after the source write.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public-update coverage for converter families and converter groups.
- Public-update coverage for non-number targets.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Pending add/remove behavior, observer-list parity, re-entry protection, and
  persisting-list ordering beyond this single dirty bind.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` direct number target writes the edited
  target value into the default view-model number source during public update.
- The same public update reapplies source-to-target so the bindable target
  remains equal to the edited value after the source write.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing state-machine bindable-property target-dirty tests still preserve
  manual edits through `advancedDataContext()`.
