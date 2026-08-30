//! A player's mutation scope is its live occurrence tree, not only the graph
//! explicitly subscribed to by the host. Nested artboards own independent
//! local ViewModels; their writes must remain valid and transactional without
//! becoming notifications for an unrelated host-bound root.

use std::collections::HashSet;

use nuxie::{ArtboardInstance, RuntimeOwnedViewModelHandle};

use crate::NuxStatus;

pub(super) fn scene_roots(
    artboard: &ArtboardInstance,
    bound: &RuntimeOwnedViewModelHandle,
    limit: usize,
) -> Result<Vec<RuntimeOwnedViewModelHandle>, NuxStatus> {
    let file = artboard.native_file();
    let mut roots = vec![bound.clone()];
    let mut seen_roots = HashSet::from([bound.native_handle().identity_key()]);
    let mut pending = vec![artboard.native_handle()];
    let mut seen_artboards = HashSet::new();
    let mut seen_contexts = Vec::new();

    while let Some(artboard) = pending.pop() {
        if !seen_artboards.insert(artboard.core_handle().identity_key()) {
            continue;
        }
        if seen_artboards.len() > limit {
            return Err(NuxStatus::LimitExceeded);
        }
        let (mut context, hosts) = artboard.with_artboard(|artboard| {
            (
                artboard.base.data_context(),
                artboard
                    .base
                    .nested_artboards()
                    .into_iter()
                    .chain(artboard.base.artboard_component_lists())
                    .collect::<Vec<_>>(),
            )
        });
        while let Some(current) = context {
            if seen_contexts.iter().any(|known| current.ptr_eq(known)) {
                break;
            }
            if seen_contexts.len() >= limit {
                return Err(NuxStatus::LimitExceeded);
            }
            let (instances, parent) = current.with_context(|context| {
                (context.view_model_instances().to_vec(), context.parent())
            });
            seen_contexts.push(current);
            for instance in instances.into_iter().flatten() {
                if seen_roots.insert(instance.identity_key()) {
                    if roots.len() >= limit {
                        return Err(NuxStatus::LimitExceeded);
                    }
                    roots.push(
                        RuntimeOwnedViewModelHandle::from_native(file.clone(), instance)
                            .ok_or(NuxStatus::RuntimeError)?,
                    );
                }
            }
            context = parent;
        }
        for host in hosts {
            let children = host
                .with(|object| {
                    let host = object.as_artboard_host().ok_or(NuxStatus::RuntimeError)?;
                    if host.artboard_count() > limit {
                        return Err(NuxStatus::LimitExceeded);
                    }
                    Ok((0..host.artboard_count())
                        .filter_map(|index| host.artboard_instance(index as i32))
                        .collect::<Vec<_>>())
                })
                .ok_or(NuxStatus::RuntimeError)??;
            if pending.len().saturating_add(children.len()) > limit {
                return Err(NuxStatus::LimitExceeded);
            }
            pending.extend(children);
        }
    }
    Ok(roots)
}
