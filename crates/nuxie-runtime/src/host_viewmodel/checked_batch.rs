//! Additive host policy: an atomic replacement of several values inside an active operation.
//! Ordinary Rive setters, listeners and file behavior do not enter this API.
use super::checked_write::checked_candidate;
use super::*;
use crate::mechanical_port::source::viewmodel::viewmodel_instance_list::ViewModelInstanceList;

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
/// all or none. Requires this file's active graph transaction
/// (`RuntimeOwnedViewModelGraphTransaction`, whose checkpoint restores every
/// batch write on rollback), capture and policy operation. Earlier writes are
/// settled once and preserved. A refusal writes no candidates; its native
/// reports/errors remain available at the usual boundary. Each report names the
/// entry that breaks its rule on its own, and a refusal suppresses only the
/// paired markers of entries its rules refuse.
///
/// Entries must resolve to distinct properties, including implicit paired markers
/// (duplicate aliases are invalid). At most 4096 entries and 8 MiB of names/text
/// are accepted, subject to the operation's remaining journal/report budgets.
/// Validation/capacity errors precede batch mutation. Writers address the
/// properties resolved during validation, so an accepted batch cannot stop part
/// way. Hosts still settle bindings/markers/groups and commit before publishing.
pub fn runtime_checked_value_write_batch(
    policy: &RuntimeValuePolicy,
    operation: &mut RuntimeValuePolicyOperation,
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
    entries: Vec<RuntimeCheckedValueBatchEntry>,
) -> Result<RuntimeCheckedValueBatchResult, RuntimeValuePolicyError> {
    if !crate::view_model_cell::has_graph_transaction() {
        return Err(RuntimeValuePolicyError::BorrowConflict);
    }
    if entries.len() > 4096 {
        return Err(RuntimeValuePolicyError::LimitExceeded);
    }
    let mut bytes = 0usize;
    let mut occupied = BTreeSet::new();
    let mut values = Vec::new();
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
            property,
            value: candidate,
            implicit_marker: false,
        });
        if let Some(index) = policy.marker_property(&owner, index) {
            if !occupied.insert((owner.instance_identity(), index)) {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            let property = owner
                .borrow()
                .property_by_path(&[index])
                .ok_or(RuntimeValuePolicyError::NotFound)?;
            values.push(CheckedBatchValue {
                owner,
                index,
                property,
                value: RuntimeViewModelChangeValue::Boolean(marker.unwrap_or(true)),
                implicit_marker: true,
            });
        }
    }
    RuntimeViewModelChangeCapture::with_current(|capture| {
        operation.checked_batch(policy, roots, &values, capture, || {
            for value in &values {
                capture.correcting(instance::identity(&value.property) as usize, || {
                    write_indexed(value)
                });
            }
        })
    })
    .ok_or(RuntimeValuePolicyError::BorrowConflict)?
}

/// Write one validated candidate through its resolved owner and index. The
/// candidate's type matched the property during validation, so this cannot
/// fail part way through an accepted batch.
fn write_indexed(value: &CheckedBatchValue) {
    let mut owner = value.owner.borrow_mut();
    match &value.value {
        RuntimeViewModelChangeValue::Number(number) => {
            owner.set_number_by_property_index(value.index, *number);
        }
        RuntimeViewModelChangeValue::Boolean(boolean) => {
            owner.set_boolean_by_property_index(value.index, *boolean);
        }
        RuntimeViewModelChangeValue::String(text) => {
            owner.set_string_by_property_index(value.index, text);
        }
        RuntimeViewModelChangeValue::Color(color) => {
            owner.set_color_by_property_index(value.index, *color);
        }
        RuntimeViewModelChangeValue::Enum(choice) => {
            owner.set_enum_by_property_index(value.index, *choice);
        }
        // checked_candidate produces only the empty list (a clear).
        RuntimeViewModelChangeValue::List(_) => {
            drop(owner);
            instance::mutate(|| {
                value
                    .property
                    .with_downcast_mut::<ViewModelInstanceList, _>(
                        ViewModelInstanceList::remove_all_items,
                    )
            });
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "checked_batch_tests.rs"]
mod tests;
