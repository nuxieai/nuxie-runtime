//! The mutation scope is the active occurrence index plus explicitly subscribed
//! roots. Nuxie row collapse uses the same index for every host frame path.
use crate::NuxStatus;
use nuxie::{ArtboardInstance, RuntimeOwnedViewModelHandle};
use std::collections::HashSet;
pub(super) fn scene_roots(
    artboard: &ArtboardInstance,
    bound: &RuntimeOwnedViewModelHandle,
    limit: usize,
) -> Result<Vec<RuntimeOwnedViewModelHandle>, NuxStatus> {
    scene_roots_across(artboard, std::slice::from_ref(bound), limit)
}
pub(super) fn scene_roots_across(
    artboard: &ArtboardInstance,
    subscribed: &[RuntimeOwnedViewModelHandle],
    limit: usize,
) -> Result<Vec<RuntimeOwnedViewModelHandle>, NuxStatus> {
    let mut index = nuxie::hosted_scene::HostedIndex::default();
    collect(
        artboard,
        subscribed,
        limit,
        &index.active(&artboard.native_handle()),
    )
}
fn collect(
    artboard: &ArtboardInstance,
    subscribed: &[RuntimeOwnedViewModelHandle],
    limit: usize,
    boards: &[nuxie::CoreHandle],
) -> Result<Vec<RuntimeOwnedViewModelHandle>, NuxStatus> {
    if boards.len() > limit {
        return Err(NuxStatus::LimitExceeded);
    }
    let file = artboard.native_file();
    let mut roots = Vec::new();
    let mut seen = HashSet::new();
    let mut contexts = Vec::new();
    for root in subscribed {
        if seen.insert(root.native_handle()) {
            roots.push(root.clone());
        }
    }
    // Authored inline models are part of the scene too, including a flight
    // slot's model after its fresh child is retired during initial settlement.
    // This shared boundary supplies both eager and lazy checkpoints with the
    // same real scene roots, never with arbitrary owners from the write journal.
    for instance in
        nuxie::hosted_scene::HostedIndex::default().active_owned_models(&artboard.native_handle())
    {
        if seen.insert(instance.clone()) {
            roots.push(
                RuntimeOwnedViewModelHandle::from_native(file.clone(), instance)
                    .ok_or(NuxStatus::RuntimeError)?,
            );
        }
    }
    for board in boards {
        #[cfg(feature = "test-support")]
        if *board != artboard.native_handle().core_handle() {
            nuxie::hosted_scene::record_walk(nuxie::hosted_scene::Walk::OwnerCapture);
        }
        let mut context = board
            .with_downcast::<nuxie::Artboard, _>(|b| b.data_context())
            .flatten();
        while let Some(current) = context {
            if contexts.iter().any(|c| current.ptr_eq(c)) {
                break;
            }
            if contexts.len() >= limit {
                return Err(NuxStatus::LimitExceeded);
            }
            let (instances, parent) =
                current.with_context(|c| (c.view_model_instances().to_vec(), c.parent()));
            contexts.push(current);
            for instance in instances.into_iter().flatten() {
                if seen.insert(instance.clone()) {
                    roots.push(
                        RuntimeOwnedViewModelHandle::from_native(file.clone(), instance)
                            .ok_or(NuxStatus::RuntimeError)?,
                    );
                }
            }
            context = parent;
        }
    }
    if roots.len() > limit {
        return Err(NuxStatus::LimitExceeded);
    }
    Ok(roots)
}
