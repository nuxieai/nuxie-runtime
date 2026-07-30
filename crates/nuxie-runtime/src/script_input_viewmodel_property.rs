//! Pinned `src/script_input_viewmodel_property.cpp` occurrence semantics.
//!
//! `dataBindPathIds` is retained bytes resolved during live hydration; it is
//! not a scalar `DataBindContextValue` target. This owner therefore keeps the
//! context-relative path lookup beside the typed `ScriptViewModel` projection
//! instead of hiding it in the broad scripting module.

use std::rc::Rc;

use nuxie_binary::{RuntimeFile, RuntimeObject};

use crate::data_bind_graph::RuntimeDataBindGraphValue;
use crate::scripting::{ScriptViewModel, build_script_view_model, script_view_model_from_owned};
use crate::{RuntimeOwnedViewModelContextHandle, RuntimeOwnedViewModelInstance};

/// Occurrence-owned clone of `ScriptInputViewModelProperty::dataBindPath`.
///
/// Pinned C++ deep-clones both the path object and its id buffers for every
/// scripted-object occurrence (`script_input_viewmodel_property_base.hpp:41-45`;
/// `script_input_viewmodel_property.cpp:26-30`;
/// `data_bind_path_referencer.cpp:15-21`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptInputViewModelPropertyPath {
    pub path_ids: Vec<u32>,
    pub resolved_path_ids: Vec<u32>,
    pub is_relative: bool,
}

impl ScriptInputViewModelPropertyPath {
    #[doc(hidden)]
    pub fn from_imported(file: &RuntimeFile, input: &RuntimeObject) -> Option<Self> {
        if input.type_name != "ScriptInputViewModelProperty" {
            return None;
        }
        let path = file.data_bind_path_for_referencer_object(input)?;
        let resolved_path_ids = if path.is_relative && path.path_ids.len() == 1 {
            match file.scripting_manifest() {
                Some(manifest) => manifest
                    .resolve_path(path.path_ids[0])
                    .map_or_else(Vec::new, <[u32]>::to_vec),
                None => path.resolved_path_ids,
            }
        } else {
            path.resolved_path_ids
        };
        Some(Self {
            path_ids: path.path_ids,
            resolved_path_ids,
            is_relative: path.is_relative,
        })
    }
}

pub(crate) fn value_property_key() -> Option<u16> {
    None
}

pub(crate) fn authored_target(
    _input: &RuntimeObject,
    _property_key: u32,
) -> Option<RuntimeDataBindGraphValue> {
    None
}

/// Resolves a `ScriptInputViewModelProperty` after its scripted object has a
/// data context. C++ treats hydration as all-or-nothing, so `None` means the
/// caller must defer every input and user `init`, not install a nil stand-in.
pub fn bound_script_view_model_from_owned_context(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelContextHandle,
    input: &RuntimeObject,
) -> Option<ScriptViewModel> {
    let path = ScriptInputViewModelPropertyPath::from_imported(file, input)?;
    bound_script_view_model_from_owned_path(file, context, &path)
}

/// Resolve one concrete cloned `ScriptInputViewModelProperty` path.
pub fn bound_script_view_model_from_owned_path(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelContextHandle,
    path: &ScriptInputViewModelPropertyPath,
) -> Option<ScriptViewModel> {
    bound_script_view_model_property_from_owned_path(file, context, path).flatten()
}

/// Resolve the retained property cell separately from its currently selected
/// child instance.
///
/// The outer `Option` is the C++ prerequisite result: `None` means the path
/// did not resolve to a `ViewModelInstanceViewModel` property. The inner
/// `Option` is that property's nullable `referenceViewModelInstance`. Pinned
/// C++ accepts the former property cell even when the latter pointer is null;
/// hydration then leaves the scripted table field unchanged and continues
/// with later inputs (`script_input_viewmodel_property.cpp:60-113`;
/// `scripted_object.cpp:399-426`).
pub(crate) fn bound_script_view_model_property_from_owned_path(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelContextHandle,
    path: &ScriptInputViewModelPropertyPath,
) -> Option<Option<ScriptViewModel>> {
    let use_resolver = path.is_relative && file.scripting_manifest().is_some();
    let source_path = if use_resolver {
        path.resolved_path_ids.as_slice()
    } else {
        path.path_ids.as_slice()
    };
    let root = context.root_handle();
    let property_path = if use_resolver {
        root.borrow().property_path_for_context_resolved_name_path(
            file,
            context.scope_path(),
            source_path,
            true,
        )
    } else {
        root.borrow()
            .property_path_for_context_source_path_with_manifest_mode(
                file,
                context.scope_path(),
                source_path,
                false,
                false,
            )
    }?;
    if root
        .borrow()
        .view_model_value_by_property_path(&property_path)
        .is_none()
    {
        return None;
    }
    let Some(concrete) = root.linked_view_model_by_property_path(&property_path) else {
        return Some(None);
    };
    Some(script_view_model_from_owned(file, &concrete))
}

/// Hydrate a detached scripting snapshot from a detached owned context.
///
/// Retained runtime integrations should use
/// [`bound_script_view_model_from_owned_context`] so nested mutations keep the
/// same graph identity and invalidation path.
pub fn bound_script_view_model_snapshot(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    input: &RuntimeObject,
) -> Option<ScriptViewModel> {
    let path = ScriptInputViewModelPropertyPath::from_imported(file, input)?;
    bound_script_view_model_snapshot_from_path(file, context, &path)
}

/// Detached-snapshot companion to
/// [`bound_script_view_model_from_owned_path`].
pub fn bound_script_view_model_snapshot_from_path(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    path: &ScriptInputViewModelPropertyPath,
) -> Option<ScriptViewModel> {
    bound_script_view_model_property_snapshot_from_path(file, context, path).flatten()
}

pub(crate) fn bound_script_view_model_property_snapshot_from_path(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    path: &ScriptInputViewModelPropertyPath,
) -> Option<Option<ScriptViewModel>> {
    let use_resolver = path.is_relative && file.scripting_manifest().is_some();
    let source_path = if use_resolver {
        path.resolved_path_ids.as_slice()
    } else {
        path.path_ids.as_slice()
    };
    let property_path = if use_resolver {
        context.property_path_for_context_resolved_name_path(file, &[], source_path, true)
    } else {
        context.property_path_for_context_source_path_with_manifest_mode(
            file,
            &[],
            source_path,
            false,
            false,
        )
    }?;
    if context
        .view_model_value_by_property_path(&property_path)
        .is_none()
    {
        return None;
    }
    let Some(view_model_index) = context.view_model_index_by_property_path(&property_path) else {
        return Some(None);
    };
    let Some(instance) = context.nested_instance_by_property_path(&property_path) else {
        return Some(None);
    };
    Some(build_script_view_model(
        Rc::new(file.clone()),
        view_model_index,
        instance,
        &[],
    ))
}
