//! Lua-free checked scalar writes shared by embedding hosts.
use super::*;

const MAX_CHECKED_WRITE_BYTES: usize = 8 * 1024 * 1024;

/// A host scalar, or an explicit clear of the native value and paired marker.
pub enum RuntimeCheckedValueInput {
    Clear,
    Number(f64),
    Boolean(bool),
    Text(Vec<u8>),
}

/// Apply a checked write within the host's active operation. The root names
/// and operation must belong to this policy's file. Hosts own the surrounding
/// graph transaction and capture, and publish reports only after committing.
/// An operation is required when rules are installed. With no rules, the native
/// write and marker clear also work without an operation.
/// Accepted writes return no code; only a refusal returns its first rule code.
pub fn runtime_checked_value_write(
    policy: &RuntimeValuePolicy,
    operation: Option<&mut RuntimeValuePolicyOperation>,
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
    root_name: &str,
    path: &str,
    input: RuntimeCheckedValueInput,
) -> Result<(bool, Option<String>), RuntimeValuePolicyError> {
    if root_name.len().saturating_add(path.len()) > MAX_CHECKED_WRITE_BYTES {
        return Err(RuntimeValuePolicyError::LimitExceeded);
    }
    if policy.has_rules() && operation.is_none() {
        return Err(RuntimeValuePolicyError::InvalidArgument);
    }
    let root = roots
        .get(root_name)
        .ok_or(RuntimeValuePolicyError::NotFound)?;
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

pub(super) fn write_native(
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
