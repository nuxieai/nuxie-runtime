//! File-scoped host policy over the operation's native write journal.
use super::*;
use crate::mechanical_port::source::generated::viewmodel::{
    viewmodel_property_boolean_base::ViewModelPropertyBooleanBase,
    viewmodel_property_color_base::ViewModelPropertyColorBase,
    viewmodel_property_enum_base::ViewModelPropertyEnumBase,
    viewmodel_property_list_base::ViewModelPropertyListBase,
    viewmodel_property_number_base::ViewModelPropertyNumberBase,
    viewmodel_property_string_base::ViewModelPropertyStringBase,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeValueRuleMode {
    Refuse,
    Mark,
}

#[derive(Clone, Debug)]
pub struct RuntimeValueRule {
    pub model: String,
    pub property: String,
    pub kind: RuntimeValueRuleKind,
    pub mode: RuntimeValueRuleMode,
    pub code: String,
    pub message: String,
}

struct Rule {
    model: usize,
    property: usize,
    entry: RuntimeValueRule,
    compiled: RuntimeCompiledValueRule,
}

#[derive(Clone, Debug)]
pub struct RuntimeValueMarker {
    pub model: String,
    pub value: String,
    pub marker: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeValuePolicyError {
    NotFound,
    InvalidArgument,
    LimitExceeded,
    BorrowConflict,
}

struct Marker {
    model: usize,
    value: usize,
    marker: usize,
}

/// A host retains one policy with its imported file and shares it with every
/// operation on that file. Replacing a table validates the whole replacement.
pub struct RuntimeValuePolicy {
    file: RuntimeFileHandle,
    markers: Vec<Marker>,
    rules: Vec<Rule>,
}

impl RuntimeValuePolicy {
    pub fn new(file: RuntimeFileHandle) -> Self {
        Self {
            file,
            markers: Vec::new(),
            rules: Vec::new(),
        }
    }

    pub fn has_markers(&self) -> bool {
        !self.markers.is_empty()
    }

    pub fn has_rules(&self) -> bool {
        !self.rules.is_empty()
    }

    pub fn rule(&self, index: usize) -> Option<&RuntimeValueRule> {
        self.rules.get(index).map(|rule| &rule.entry)
    }

    pub fn resolve_property(
        &self,
        root: &RuntimeOwnedViewModelHandle,
        path: &str,
    ) -> Result<(RuntimeOwnedViewModelHandle, usize), RuntimeValuePolicyError> {
        if !root.native_file().ptr_eq(&self.file) {
            return Err(RuntimeValuePolicyError::InvalidArgument);
        }
        let path = root
            .borrow()
            .path_named(path)
            .ok_or(RuntimeValuePolicyError::NotFound)?;
        let (index, parents) = path.split_last().ok_or(RuntimeValuePolicyError::NotFound)?;
        let owner = if parents.is_empty() {
            root.clone()
        } else {
            root.linked_view_model_by_property_path(parents)
                .ok_or(RuntimeValuePolicyError::NotFound)?
        };
        Ok((owner, *index))
    }

    pub fn property_value(
        &self,
        root: &RuntimeOwnedViewModelHandle,
        path: &str,
    ) -> Result<RuntimeViewModelChangeValue, RuntimeValuePolicyError> {
        let (owner, index) = self.resolve_property(root, path)?;
        let property = owner
            .borrow()
            .property_by_path(&[index])
            .ok_or(RuntimeValuePolicyError::NotFound)?;
        operation::read(&property).ok_or(RuntimeValuePolicyError::InvalidArgument)
    }

    pub fn marker_property(
        &self,
        owner: &RuntimeOwnedViewModelHandle,
        index: usize,
    ) -> Option<usize> {
        if !owner.native_file().ptr_eq(&self.file) {
            return None;
        }
        let model = owner.borrow().view_model_index();
        self.markers
            .iter()
            .find(|pair| pair.model == model && pair.value == index)
            .map(|pair| pair.marker)
    }

    /// Replace the entire ordered rule table atomically. Model/property names
    /// are resolved from this file; no naming convention selects a property.
    pub fn set_rules(
        &mut self,
        entries: &[RuntimeValueRule],
    ) -> Result<(), RuntimeValuePolicyError> {
        if entries.len() > 4_096 {
            return Err(RuntimeValuePolicyError::LimitExceeded);
        }
        let mut rules = Vec::with_capacity(entries.len());
        for entry in entries {
            let (model, property, native) = self.property(&entry.model, &entry.property)?;
            use RuntimeValueRuleKind::*;
            let valid = match &entry.kind {
                NumberMinimum(_) | NumberMaximum(_) => {
                    native.is_type_of(ViewModelPropertyNumberBase::TYPE_KEY)
                }
                TextMinimum(_) | TextMaximum(_) | Length { .. } | Pattern(_) | Url | Date => {
                    native.is_type_of(ViewModelPropertyStringBase::TYPE_KEY)
                }
                AllowedValues(_) => {
                    native.is_type_of(ViewModelPropertyStringBase::TYPE_KEY)
                        || native.is_type_of(ViewModelPropertyEnumBase::TYPE_KEY)
                }
                ItemCount { .. } | PickedCount { .. } => {
                    native.is_type_of(ViewModelPropertyListBase::TYPE_KEY)
                }
                Required => [
                    ViewModelPropertyNumberBase::TYPE_KEY,
                    ViewModelPropertyBooleanBase::TYPE_KEY,
                    ViewModelPropertyColorBase::TYPE_KEY,
                    ViewModelPropertyEnumBase::TYPE_KEY,
                    ViewModelPropertyStringBase::TYPE_KEY,
                    ViewModelPropertyListBase::TYPE_KEY,
                ]
                .iter()
                .any(|kind| native.is_type_of(*kind)),
            };
            if !valid {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            if let PickedCount {
                property: picked, ..
            } = &entry.kind
            {
                let models = self.file.with_file(|file| {
                    (0..file.view_model_count())
                        .filter_map(|index| {
                            file.view_model(index)?
                                .with(|model| {
                                    model
                                        .as_view_model()
                                        .map(|model| model.base.name().to_owned())
                                })
                                .flatten()
                        })
                        .collect::<Vec<_>>()
                });
                let mut found = false;
                let mut boolean = false;
                for model in models {
                    if let Ok((_, _, property)) = self.property(&model, picked) {
                        found = true;
                        boolean |= property.is_type_of(ViewModelPropertyBooleanBase::TYPE_KEY);
                    }
                }
                if !found {
                    return Err(RuntimeValuePolicyError::NotFound);
                }
                if !boolean {
                    return Err(RuntimeValuePolicyError::InvalidArgument);
                }
            }
            rules.push(Rule {
                model,
                property,
                compiled: RuntimeCompiledValueRule::compile(entry.kind.clone())?,
                entry: entry.clone(),
            });
        }
        for lower in &rules {
            for upper in &rules {
                if (lower.model, lower.property) != (upper.model, upper.property) {
                    continue;
                }
                let reversed = match (lower.compiled.kind(), upper.compiled.kind()) {
                    (
                        RuntimeValueRuleKind::NumberMinimum(a),
                        RuntimeValueRuleKind::NumberMaximum(b),
                    ) => a > b,
                    (
                        RuntimeValueRuleKind::TextMinimum(a),
                        RuntimeValueRuleKind::TextMaximum(b),
                    ) => a.encode_utf16().cmp(b.encode_utf16()).is_gt(),
                    (
                        RuntimeValueRuleKind::PickedCount { property: a, .. },
                        RuntimeValueRuleKind::PickedCount { property: b, .. },
                    ) => a != b,
                    _ => false,
                };
                if reversed {
                    return Err(RuntimeValuePolicyError::InvalidArgument);
                }
            }
        }
        self.rules = rules;
        Ok(())
    }

    /// Enable ordered host/script writes before an operation starts. A file
    /// without markers keeps the ordinary change-only journal unchanged.
    pub fn prepare_capture(&self, capture: &RuntimeViewModelChangeCapture) {
        if self.has_markers() || self.has_rules() {
            capture.track_unchanged_writes();
        }
    }

    /// Publish marker bindings without advancing animations or listeners.
    /// Call after a marker pass changes a value, then run the marker pass again
    /// to include any writes caused by those bindings in the same journal.
    pub fn flush_marker_bindings(&self, artboard: &crate::ArtboardInstance) -> bool {
        (self.has_markers() || self.has_rules()) && artboard.native_handle().update_pass(true)
    }

    pub fn set_markers(
        &mut self,
        entries: &[RuntimeValueMarker],
    ) -> Result<(), RuntimeValuePolicyError> {
        if entries.len() > 4_096 {
            return Err(RuntimeValuePolicyError::LimitExceeded);
        }
        let mut markers = Vec::with_capacity(entries.len());
        let mut values = BTreeSet::new();
        let mut targets = BTreeSet::new();
        for entry in entries {
            let (model, value, value_type) = self.property(&entry.model, &entry.value)?;
            let (_, marker, marker_type) = self.property(&entry.model, &entry.marker)?;
            if ![
                ViewModelPropertyNumberBase::TYPE_KEY,
                ViewModelPropertyBooleanBase::TYPE_KEY,
                ViewModelPropertyColorBase::TYPE_KEY,
                ViewModelPropertyEnumBase::TYPE_KEY,
            ]
            .iter()
            .any(|kind| value_type.is_type_of(*kind))
                || !marker_type.is_type_of(ViewModelPropertyBooleanBase::TYPE_KEY)
                || !values.insert((model, value))
                || !targets.insert((model, marker))
            {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            markers.push(Marker {
                model,
                value,
                marker,
            });
        }
        if !values.is_disjoint(&targets) {
            return Err(RuntimeValuePolicyError::InvalidArgument);
        }
        self.markers = markers;
        Ok(())
    }

    fn property(
        &self,
        model_name: &str,
        property_name: &str,
    ) -> Result<(usize, usize, CoreHandle), RuntimeValuePolicyError> {
        self.file.with_file(|file| {
            let model_index = (0..file.view_model_count())
                .find(|index| {
                    file.view_model(*index).is_some_and(|model| {
                        model.with(|model| {
                            model
                                .as_view_model()
                                .is_some_and(|model| model.base.name() == model_name)
                        }) == Some(true)
                    })
                })
                .ok_or(RuntimeValuePolicyError::NotFound)?;
            let model = file
                .view_model(model_index)
                .ok_or(RuntimeValuePolicyError::NotFound)?;
            let properties = model
                .with(|model| {
                    model
                        .as_view_model()
                        .map(|model| model.properties().to_vec())
                })
                .flatten()
                .ok_or(RuntimeValuePolicyError::NotFound)?;
            properties
                .into_iter()
                .enumerate()
                .find(|(_, property)| {
                    property.with(|property| {
                        property
                            .as_view_model_property()
                            .is_some_and(|property| property.const_name() == property_name)
                    }) == Some(true)
                })
                .map(|(index, property)| (model_index, index, property))
                .ok_or(RuntimeValuePolicyError::NotFound)
        })
    }

    /// Run before resolving the journal, inside the caller's transaction.
    /// Roots include the pre-operation and current occurrence graphs so a
    /// detached item retains its place in the write order. The writer must
    /// record its ordinary boolean mutation in that same journal/transaction.
    pub fn apply_markers(
        &self,
        capture: &RuntimeViewModelChangeCapture,
        roots: &[RuntimeOwnedViewModelHandle],
        mut write: impl FnMut(
            &RuntimeOwnedViewModelHandle,
            usize,
            bool,
        ) -> Result<bool, RuntimeValuePolicyError>,
    ) -> Result<bool, RuntimeValuePolicyError> {
        if self.markers.is_empty() {
            return Ok(false);
        }
        if roots
            .iter()
            .any(|root| !root.native_file().ptr_eq(&self.file))
        {
            return Err(RuntimeValuePolicyError::InvalidArgument);
        }
        let changes = RuntimeOwnedViewModelHandle::resolve_change_snapshot(roots, capture)
            .ok_or(RuntimeValuePolicyError::LimitExceeded)?;
        let mut last = BTreeMap::new();
        for (order, (owner, change)) in changes.iter().enumerate() {
            last.insert((owner.instance_identity(), change.property_index), order);
        }
        let mut applied = BTreeSet::new();
        let mut changed = false;
        for (owner, change) in changes {
            for marker in &self.markers {
                let key = (owner.instance_identity(), marker.value);
                if owner.borrow().view_model_index() != marker.model
                    || change.property_index != marker.value
                    || !applied.insert(key)
                {
                    continue;
                }
                let marker_order = last.get(&(owner.instance_identity(), marker.marker));
                if marker_order
                    .is_none_or(|order| last.get(&key).is_some_and(|value| order < value))
                {
                    changed |= write(&owner, marker.marker, true)?;
                }
            }
        }
        capture
            .snapshot()
            .map_err(|_| RuntimeValuePolicyError::LimitExceeded)?;
        Ok(changed)
    }
}

#[cfg(test)]
#[path = "value_policy_tests.rs"]
mod tests;

#[path = "value_policy_operation.rs"]
mod operation;
pub(crate) use operation::capture_initial_policy_owner;
pub use operation::*;
