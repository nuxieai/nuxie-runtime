use super::*;

/// Rust retains the owning ViewModel/instance slots instead of the pinned raw
/// `ViewModelInstance*`.
#[derive(Debug, Clone, Copy)]
pub(super) struct ViewModelInstanceImporter {
    view_model_index: usize,
    instance_index: usize,
}

impl ViewModelInstanceImporter {
    /// Mechanical translation of `ViewModelInstanceImporter`'s constructor.
    pub(super) fn new(view_model_index: usize, instance_index: usize) -> Self {
        Self {
            view_model_index,
            instance_index,
        }
    }

    /// Mechanical translation of `viewModelInstance()` using stable owner
    /// slots in place of the C++ pointer.
    pub(super) fn view_model_instance(self) -> (usize, usize) {
        (self.view_model_index, self.instance_index)
    }

    /// Mechanical translation of `addValue`.
    pub(super) fn add_value<'a>(
        self,
        view_models: &mut [RuntimeViewModel<'a>],
        object: &'a RuntimeObject,
    ) -> Option<usize> {
        let (view_model_index, instance_index) = self.view_model_instance();
        let values = &mut view_models
            .get_mut(view_model_index)?
            .instances
            .get_mut(instance_index)?
            .values;
        values.push(RuntimeViewModelInstanceValue {
            object,
            list_items: Vec::new(),
        });
        Some(values.len() - 1)
    }

    /// Mechanical translation of `resolve() -> StatusCode::Ok`.
    pub(super) fn resolve(self) -> bool {
        true
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "ViewModelInstance" {
        return Some(context.latest(ImportStackKey::Backboard));
    }
    if matches!(
        definition.name,
        "ViewModelInstanceAsset"
            | "ViewModelInstanceAssetImage"
            | "ViewModelInstanceAssetFont"
            | "ViewModelInstanceAssetBlob"
    ) {
        return Some(
            context.latest(ImportStackKey::Backboard)
                && context.latest(ImportStackKey::ViewModelInstance),
        );
    }
    definition
        .is_a("ViewModelInstanceValue")
        .then(|| context.latest(ImportStackKey::ViewModelInstance))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "ViewModelInstance" {
        context.make_latest(ImportStackKey::ViewModelInstance);
    }
}
