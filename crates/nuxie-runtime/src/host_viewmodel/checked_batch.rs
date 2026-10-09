//! Additive host policy: an atomic answer replacement inside an active operation.
//! Ordinary Rive setters, listeners and file behavior do not enter this API.
use super::checked_write::{checked_candidate, write_native};
use super::*;

/// One scalar candidate (or explicit clear), addressed through the caller's roots.
pub struct RuntimeCheckedValueBatchEntry {
    pub root_name: String,
    pub path: String,
    pub value: RuntimeCheckedValueInput,
}

/// The first refusing native rule, in input order then installed rule order.
#[derive(Debug, PartialEq, Eq)]
pub struct RuntimeCheckedValueRefusal {
    pub rule_index: usize,
    pub code: String,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RuntimeCheckedValueBatchResult {
    pub applied: bool,
    /// Marking breaches do not refuse a replacement and return no refusal here.
    /// All relevant outcomes remain in the operation's reports and group outputs.
    pub refusal: Option<RuntimeCheckedValueRefusal>,
}

pub(super) struct CheckedBatchValue {
    pub owner: RuntimeOwnedViewModelHandle,
    pub index: usize,
    pub property: CoreHandle,
    pub value: RuntimeViewModelChangeValue,
    pub implicit_marker: bool,
}

/// Check a complete replacement against the final candidate values, applying
/// all or none. Requires this file's active graph transaction, capture and policy
/// operation. Earlier writes are settled once and preserved. A refusal writes no
/// candidates; its native reports/errors remain available at the usual boundary.
///
/// Entries must resolve to distinct properties, including implicit paired markers
/// (duplicate aliases are invalid). At most 4096 entries and 8 MiB of names/text
/// are accepted, subject to the operation's remaining journal/report budgets.
/// Validation/capacity errors precede batch mutation. As with scalar checked
/// writes, an unexpected native writer error requires aborting the outer operation.
/// Hosts still settle bindings/markers/groups and commit before publishing.
pub fn runtime_checked_value_write_batch(
    policy: &RuntimeValuePolicy,
    operation: &mut RuntimeValuePolicyOperation,
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
    entries: Vec<RuntimeCheckedValueBatchEntry>,
) -> Result<RuntimeCheckedValueBatchResult, RuntimeValuePolicyError> {
    if !crate::view_model_cell::has_host_transaction() {
        return Err(RuntimeValuePolicyError::BorrowConflict);
    }
    if entries.len() > 4096 {
        return Err(RuntimeValuePolicyError::LimitExceeded);
    }
    let mut bytes = 0usize;
    let mut occupied = BTreeSet::new();
    let mut values = Vec::new();
    let mut writers = Vec::new();
    for entry in entries {
        bytes = bytes
            .saturating_add(entry.root_name.len())
            .saturating_add(entry.path.len());
        if let RuntimeCheckedValueInput::Text(text) = &entry.value {
            bytes = bytes.saturating_add(text.len());
        }
        if bytes > 8 * 1024 * 1024 {
            return Err(RuntimeValuePolicyError::LimitExceeded);
        }
        let root = roots
            .get(&entry.root_name)
            .ok_or(RuntimeValuePolicyError::NotFound)?;
        let (candidate, marker) =
            checked_candidate(entry.value, policy.property_value(root, &entry.path)?)?;
        let (owner, index) = policy.resolve_property(root, &entry.path)?;
        let property = owner
            .borrow()
            .property_by_path(&[index])
            .ok_or(RuntimeValuePolicyError::NotFound)?;
        if !occupied.insert((owner.instance_identity(), index)) {
            return Err(RuntimeValuePolicyError::InvalidArgument);
        }
        values.push(CheckedBatchValue {
            owner: owner.clone(),
            index,
            property: property.clone(),
            value: candidate.clone(),
            implicit_marker: false,
        });
        let marker = policy
            .marker_property(&owner, index)
            .map(|index| (index, marker.unwrap_or(true)));
        if let Some((index, value)) = marker {
            if !occupied.insert((owner.instance_identity(), index)) {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            let property = owner
                .borrow()
                .property_by_path(&[index])
                .ok_or(RuntimeValuePolicyError::NotFound)?;
            values.push(CheckedBatchValue {
                owner: owner.clone(),
                index,
                property,
                value: RuntimeViewModelChangeValue::Boolean(value),
                implicit_marker: true,
            });
        }
        writers.push((root.clone(), entry.path, candidate, property, owner, marker));
    }
    RuntimeViewModelChangeCapture::with_current(|capture| {
        operation.checked_batch(policy, roots, &values, capture, || {
            for (root, path, candidate, property, owner, marker) in &writers {
                capture.correcting(instance::identity(property) as usize, || {
                    write_native(policy, root, path, candidate)
                })?;
                if let Some((index, value)) = marker {
                    let property = owner
                        .borrow()
                        .property_by_path(&[*index])
                        .ok_or(RuntimeValuePolicyError::NotFound)?;
                    capture.correcting(instance::identity(&property) as usize, || {
                        owner
                            .borrow_mut()
                            .set_boolean_by_property_index(*index, *value);
                    });
                }
            }
            Ok(())
        })
    })
    .ok_or(RuntimeValuePolicyError::BorrowConflict)?
}

#[cfg(test)]
#[path = "checked_batch_tests.rs"]
mod tests;
