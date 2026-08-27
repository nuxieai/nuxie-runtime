//! Mechanical translation of pinned `TransitionViewModelConditionImporter`.

use super::*;

pub(super) fn dispatch_imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition
        .is_a("TransitionComparator")
        .then(|| context.latest(ImportStackKey::TransitionViewModelCondition))
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.is_a("TransitionViewModelCondition") {
        context.make_latest(ImportStackKey::TransitionViewModelCondition);
    }
}

/// Rust retains the exact condition occurrence instead of the pinned raw
/// `TransitionViewModelCondition*`.
struct TransitionViewModelConditionImporter<'a> {
    condition: &'a RuntimeObject,
    left: Option<&'a RuntimeObject>,
    right: Option<&'a RuntimeObject>,
}

struct ResolvedTransitionViewModelCondition<'a> {
    condition: &'a RuntimeObject,
    comparators: RuntimeTransitionViewModelConditionComparators<'a>,
}

impl<'a> TransitionViewModelConditionImporter<'a> {
    /// Mechanical translation of the constructor: retain the supplied
    /// condition and begin with both comparator pointers null.
    fn new(condition: &'a RuntimeObject) -> Self {
        Self {
            condition,
            left: None,
            right: None,
        }
    }

    /// Mechanical translation of `setComparator`: fill the left slot once,
    /// then assign every later comparator to the right slot.
    fn set_comparator(&mut self, comparator: &'a RuntimeObject) {
        if self.left.is_none() {
            self.left = Some(comparator);
        } else {
            self.right = Some(comparator);
        }
    }

    /// Pinned `resolve` initializes the retained condition and returns Ok.
    /// The binary layer publishes the finalized comparator pointers; the
    /// runtime condition owner consumes them to perform that initialization.
    fn resolve(self) -> ResolvedTransitionViewModelCondition<'a> {
        ResolvedTransitionViewModelCondition {
            condition: self.condition,
            comparators: RuntimeTransitionViewModelConditionComparators {
                left: self.left,
                right: self.right,
            },
        }
    }
}

pub(crate) fn comparators_for_condition<'a>(
    objects: &'a [Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
    condition: &RuntimeObject,
) -> RuntimeTransitionViewModelConditionComparators<'a> {
    let Some(condition_file_index) = usize::try_from(condition.id).ok() else {
        return RuntimeTransitionViewModelConditionComparators::default();
    };
    let Some(target_condition) = objects
        .get(condition_file_index)
        .and_then(Option::as_ref)
    else {
        return RuntimeTransitionViewModelConditionComparators::default();
    };
    let mut latest = None::<TransitionViewModelConditionImporter<'a>>;

    for (file_index, object) in objects.iter().enumerate() {
        let Some(object) = object.as_ref() else {
            continue;
        };
        if import_statuses.get(file_index) != Some(&RuntimeImportStatus::Imported) {
            continue;
        }
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };

        if definition.is_a("TransitionViewModelCondition") {
            // File constructs the replacement before makeLatest resolves the
            // previous importer, then installs the replacement.
            let next = TransitionViewModelConditionImporter::new(object);
            if let Some(previous) = latest.take() {
                let resolved = previous.resolve();
                if std::ptr::eq(resolved.condition, target_condition) {
                    return resolved.comparators;
                }
            }
            latest = Some(next);
            continue;
        }

        if definition.is_a("TransitionComparator")
            && let Some(importer) = latest.as_mut()
        {
            importer.set_comparator(object);
        }
    }

    if let Some(importer) = latest {
        let resolved = importer.resolve();
        if std::ptr::eq(resolved.condition, target_condition) {
            return resolved.comparators;
        }
    }
    RuntimeTransitionViewModelConditionComparators::default()
}
