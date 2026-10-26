//! Lua-free checked scalar writes shared by embedding hosts.
use super::*;
use crate::mechanical_port::source::viewmodel::{
    viewmodel_instance_list::ViewModelInstanceList,
    viewmodel_instance_list_item::ViewModelInstanceListItem,
};
use std::collections::{HashMap, HashSet};

const MAX_CHECKED_WRITE_BYTES: usize = 8 * 1024 * 1024;

/// A host scalar, or an explicit clear of the native value and paired marker.
pub enum RuntimeCheckedValueInput {
    Clear,
    Number(f64),
    Boolean(bool),
    Text(Vec<u8>),
}

/// Where a checked write or list read starts.
pub enum RuntimeCheckedValueRoot {
    /// One of the host's roots, by name. The empty name is the bound root.
    Named(String),
    /// A script's own view model, such as `context:viewModel()`. It must be one
    /// of the host's roots, or reached from one through view-model properties
    /// and list items, whether its Screen is awake or asleep: the graph whose
    /// changes the host's step accepts and undoes. Any other view model, such
    /// as a loose `Data.Form.new()`, is not found, and nothing is written.
    ViewModel(crate::ScriptViewModel),
}

impl RuntimeCheckedValueRoot {
    /// The bytes this root counts toward a checked path or batch bound: a
    /// name's length, or none for a view model.
    pub fn byte_len(&self) -> usize {
        match self {
            Self::Named(name) => name.len(),
            Self::ViewModel(_) => 0,
        }
    }
}

/// Resolve a checked root against the host's roots. A name must be one of
/// them. A view model is looked for by walking down from each root, sleeping
/// Screens included, and stopping at the first match.
pub fn runtime_checked_value_root(
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
    root: &RuntimeCheckedValueRoot,
) -> Result<RuntimeOwnedViewModelHandle, RuntimeValuePolicyError> {
    let model = match root {
        RuntimeCheckedValueRoot::Named(name) => {
            return roots
                .get(name)
                .cloned()
                .ok_or(RuntimeValuePolicyError::NotFound);
        }
        RuntimeCheckedValueRoot::ViewModel(model) => model,
    };
    let target = model
        .native_instance()
        .ok_or(RuntimeValuePolicyError::NotFound)?;
    let mut found = None;
    walk_checked_roots(roots, |root, owner| {
        if *owner != target {
            return false;
        }
        found = Some(owner_handle(root, owner));
        true
    });
    found.flatten().ok_or(RuntimeValuePolicyError::NotFound)
}

/// Every view model a checked view-model root may name, by native instance,
/// with the root it is reached from: one walk for a batch that names several.
pub(super) fn checked_root_owners(
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
) -> HashMap<CoreHandle, RuntimeOwnedViewModelHandle> {
    let mut owners = HashMap::new();
    walk_checked_roots(roots, |root, owner| {
        owners.insert(owner.clone(), root.clone());
        false
    });
    owners
}

/// The handle for `owner`, reached from `root`.
pub(super) fn owner_handle(
    root: &RuntimeOwnedViewModelHandle,
    owner: &CoreHandle,
) -> Option<RuntimeOwnedViewModelHandle> {
    if root.native_handle() == *owner {
        Some(root.clone())
    } else {
        RuntimeOwnedViewModelHandle::from_native(root.native_file(), owner.clone())
    }
}

/// Visit each view model reachable from `roots` once, with the root it was
/// reached from, until `visit` returns true. Unlike the step's own graph walk,
/// this one doesn't skip sleeping Screens: a write wakes its Screen's path for
/// the frame, so the step accepts it.
fn walk_checked_roots(
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
    mut visit: impl FnMut(&RuntimeOwnedViewModelHandle, &CoreHandle) -> bool,
) {
    let mut seen = HashSet::new();
    for root in roots.values() {
        let mut pending = vec![root.native_handle()];
        while let Some(owner) = pending.pop() {
            if !seen.insert(owner.clone()) {
                continue;
            }
            if visit(root, &owner) {
                return;
            }
            let properties = owner
                .with_downcast::<ViewModelInstance, _>(|owner| owner.property_values().to_vec())
                .unwrap_or_default();
            for property in properties {
                if let Some(child) = property
                    .with(|property| {
                        property
                            .as_view_model_instance_view_model()?
                            .reference_view_model_instance()
                    })
                    .flatten()
                {
                    pending.push(child);
                } else if let Some(items) = property
                    .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())
                {
                    pending.extend(items.iter().filter_map(|item| {
                        item.with_downcast::<ViewModelInstanceListItem, _>(
                            ViewModelInstanceListItem::view_model_instance,
                        )
                        .flatten()
                    }));
                }
            }
        }
    }
}

/// Apply a checked write within the host's active operation. The roots and
/// operation must belong to this policy's file. Hosts own the surrounding
/// graph transaction and capture, and publish reports only after committing.
/// An operation is required when rules are installed. With no rules, the native
/// write and marker clear also work without an operation.
/// Accepted writes return no code; only a refusal returns its first rule code.
pub fn runtime_checked_value_write(
    policy: &RuntimeValuePolicy,
    operation: Option<&mut RuntimeValuePolicyOperation>,
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
    root: &RuntimeCheckedValueRoot,
    path: &str,
    input: RuntimeCheckedValueInput,
) -> Result<(bool, Option<String>), RuntimeValuePolicyError> {
    if root.byte_len().saturating_add(path.len()) > MAX_CHECKED_WRITE_BYTES {
        return Err(RuntimeValuePolicyError::LimitExceeded);
    }
    if policy.has_rules() && operation.is_none() {
        return Err(RuntimeValuePolicyError::InvalidArgument);
    }
    let root = runtime_checked_value_root(roots, root)?;
    let root = &root;
    let current = policy.property_value(root, path)?;
    let (candidate, marker) = checked_candidate(input, current)?;
    let write = || {
        write_native(policy, root, path, &candidate)?;
        if marker == Some(false) {
            let (owner, index) = policy.resolve_property(root, path)?;
            if let Some(index) = policy.marker_property(&owner, index) {
                owner
                    .borrow_mut()
                    .set_boolean_by_property_index(index, false);
            }
        }
        Ok(())
    };
    if let Some(operation) = operation {
        operation.apply_pending_writes(policy, &roots.values().cloned().collect::<Vec<_>>())?;
        let result =
            operation.checked_write(policy, root, path, candidate.clone(), marker, write)?;
        let code = if result.applied {
            None
        } else {
            result
                .rule_indices
                .first()
                .and_then(|index| policy.rule(*index))
                .map(|rule| rule.code.clone())
        };
        Ok((result.applied, code))
    } else {
        policy.invalidate();
        write()?;
        Ok((true, None))
    }
}

pub(super) fn checked_candidate(
    input: RuntimeCheckedValueInput,
    current: RuntimeViewModelChangeValue,
) -> Result<(RuntimeViewModelChangeValue, Option<bool>), RuntimeValuePolicyError> {
    let marker = matches!(input, RuntimeCheckedValueInput::Clear).then_some(false);
    let candidate = match (input, current) {
        (RuntimeCheckedValueInput::Number(value), RuntimeViewModelChangeValue::Number(_)) => {
            RuntimeViewModelChangeValue::Number(value as f32)
        }
        (RuntimeCheckedValueInput::Number(value), RuntimeViewModelChangeValue::Color(_))
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            RuntimeViewModelChangeValue::Color(value as u32)
        }
        (RuntimeCheckedValueInput::Number(value), RuntimeViewModelChangeValue::Enum(_))
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            RuntimeViewModelChangeValue::Enum(value as u64)
        }
        (RuntimeCheckedValueInput::Boolean(value), RuntimeViewModelChangeValue::Boolean(_)) => {
            RuntimeViewModelChangeValue::Boolean(value)
        }
        (RuntimeCheckedValueInput::Text(value), RuntimeViewModelChangeValue::String(_))
            if value.len() <= MAX_CHECKED_WRITE_BYTES && std::str::from_utf8(&value).is_ok() =>
        {
            RuntimeViewModelChangeValue::String(Arc::from(value))
        }
        (RuntimeCheckedValueInput::Clear, RuntimeViewModelChangeValue::Number(_)) => {
            RuntimeViewModelChangeValue::Number(0.0)
        }
        (RuntimeCheckedValueInput::Clear, RuntimeViewModelChangeValue::Boolean(_)) => {
            RuntimeViewModelChangeValue::Boolean(false)
        }
        (RuntimeCheckedValueInput::Clear, RuntimeViewModelChangeValue::Color(_)) => {
            RuntimeViewModelChangeValue::Color(0)
        }
        (RuntimeCheckedValueInput::Clear, RuntimeViewModelChangeValue::Enum(_)) => {
            RuntimeViewModelChangeValue::Enum(0)
        }
        (RuntimeCheckedValueInput::Clear, RuntimeViewModelChangeValue::String(_)) => {
            RuntimeViewModelChangeValue::String(Arc::from([]))
        }
        (RuntimeCheckedValueInput::Clear, RuntimeViewModelChangeValue::List(_)) => {
            RuntimeViewModelChangeValue::List(Vec::new())
        }
        _ => return Err(RuntimeValuePolicyError::InvalidArgument),
    };
    Ok((candidate, marker))
}

fn write_native(
    policy: &RuntimeValuePolicy,
    root: &RuntimeOwnedViewModelHandle,
    path: &str,
    candidate: &RuntimeViewModelChangeValue,
) -> Result<(), RuntimeValuePolicyError> {
    let current = policy.property_value(root, path)?;
    if std::mem::discriminant(&current) != std::mem::discriminant(candidate) {
        return Err(RuntimeValuePolicyError::InvalidArgument);
    }
    if matches!(candidate, RuntimeViewModelChangeValue::Enum(value) if u32::try_from(*value).is_err())
    {
        return Err(RuntimeValuePolicyError::InvalidArgument);
    }
    let (owner, index) = policy.resolve_property(root, path)?;
    match candidate {
        RuntimeViewModelChangeValue::Number(value) => {
            owner
                .borrow_mut()
                .set_number_by_property_index(index, *value);
        }
        RuntimeViewModelChangeValue::Boolean(value) => {
            owner
                .borrow_mut()
                .set_boolean_by_property_index(index, *value);
        }
        RuntimeViewModelChangeValue::String(value) => {
            owner
                .borrow_mut()
                .set_string_by_property_index(index, value);
        }
        RuntimeViewModelChangeValue::Color(value) => {
            owner
                .borrow_mut()
                .set_color_by_property_index(index, *value);
        }
        RuntimeViewModelChangeValue::Enum(value) => {
            owner.borrow_mut().set_enum_by_property_index(index, *value);
        }
        RuntimeViewModelChangeValue::List(items) if items.is_empty() => {
            root.clear_list_items_by_property_name_path(path);
        }
        _ => return Err(RuntimeValuePolicyError::InvalidArgument),
    }
    Ok(())
}
