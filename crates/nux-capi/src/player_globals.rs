use crate::*;

/// Set a named global on a state-machine player. The player retains the shared
/// instance; NULL clears the override and binds a fresh file-authored default.
/// Bindings observe the instance from the next step, including nested copies
/// and list rows. This call invalidates the occurrence's render revision.
/// Returns NOT_FOUND for an unknown or non-global name or a non-state-machine
/// player; HANDLE_MISMATCH for a different file, model, or legacy occurrence.
/// Thread and reentrancy restrictions are the same as a view-model bind.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_set_global_view_model(
    player: *mut NuxPlayer,
    name: NuxStringView,
    instance: *const NuxViewModelInstance,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _player_call = match enter_handle(player, HandleKind::Player) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Some(player) = (unsafe { player.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let _instance_call = if instance.is_null() {
            None
        } else {
            Some(match enter_handle(instance, HandleKind::ViewModel) {
                Ok(guard) => guard,
                Err(status) => return status,
            })
        };
        let instance = unsafe { instance.as_ref() };
        match with_utf8_view(name, |name| {
            let Some(schema) = player.view_model_catalog.global_schema_named(name) else {
                return NuxStatus::NotFound;
            };
            if let Some(instance) = instance {
                if !Arc::ptr_eq(&player.file_provenance, &instance.file_provenance)
                    || schema != instance.schema_index
                    || instance
                        .binding_provenance
                        .as_ref()
                        .is_some_and(|provenance| !Arc::ptr_eq(&player.provenance, provenance))
                {
                    return NuxStatus::HandleMismatch;
                }
                if instance.instance.try_borrow().is_err() {
                    return NuxStatus::ReentrantCall;
                }
            }
            let _occurrence_call = match enter_occurrence(&player.artboard) {
                Ok(guard) => guard,
                Err(status) => return status,
            };
            let Ok(mut selected) = player.instance.try_borrow_mut() else {
                return NuxStatus::ReentrantCall;
            };
            let PlayerInstance::StateMachine(machine) = &mut *selected else {
                return NuxStatus::NotFound;
            };
            let Ok(mut globals) = player.global_view_models.values.try_borrow_mut() else {
                return NuxStatus::ReentrantCall;
            };
            let retained = instance.map(|instance| instance.instance.clone());
            if !machine.bind_global_view_model_handle(name, retained.as_ref()) {
                return NuxStatus::NotFound;
            }
            if let Some(retained) = retained {
                globals.insert(schema, retained);
            } else {
                globals.remove(&schema);
            }
            player.global_view_models.observed_generation.set(
                globals
                    .values()
                    .map(RuntimeOwnedViewModelHandle::observable_mutation_generation)
                    .max()
                    .unwrap_or(0),
            );
            player.artboard.commit_runtime_change_or_poison(true)
        }) {
            Ok(status) => status,
            Err(status) => status,
        }
    })
}

/// A player owns its overrides; the occurrence observes them weakly so every
/// snapshot boundary sees host writes without extending the player's lifetime.
#[derive(Default)]
pub(super) struct GlobalViewModels {
    pub(super) values: RefCell<std::collections::BTreeMap<usize, RuntimeOwnedViewModelHandle>>,
    pub(super) observed_generation: Cell<u64>,
}

impl GlobalViewModels {
    pub(super) fn refresh_invalidation(
        &self,
        occurrence: &ArtboardOccurrence,
    ) -> Result<(), NuxStatus> {
        let globals = self
            .values
            .try_borrow()
            .map_err(|_| NuxStatus::ReentrantCall)?;
        let generation = globals
            .values()
            .map(RuntimeOwnedViewModelHandle::observable_mutation_generation)
            .max()
            .unwrap_or(0);
        if generation != self.observed_generation.get() {
            occurrence.invalidate_render()?;
            self.observed_generation.set(generation);
        }
        Ok(())
    }
}
