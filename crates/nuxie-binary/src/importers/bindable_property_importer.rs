use super::*;

pub(super) fn imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    match definition.name {
        "BlendState1DViewModel" => Some(
            context.latest(ImportStackKey::StateMachineLayer)
                && context.latest(ImportStackKey::BindableProperty),
        ),
        "BlendAnimationDirect"
            if object.uint_property("blendSource") == Some(2)
                && !context.latest(ImportStackKey::BindableProperty) =>
        {
            Some(false)
        }
        "ListenerViewModelChange" => Some(
            context.latest(ImportStackKey::BindableProperty)
                && listener_action_imports_successfully(object, context),
        ),
        "TransitionPropertyViewModelComparator" => Some(
            context.latest(ImportStackKey::TransitionViewModelCondition)
                && context.latest(ImportStackKey::BindableProperty),
        ),
        _ => None,
    }
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.is_a("BindableProperty") {
        context.make_latest(ImportStackKey::BindableProperty);
    }
}
impl RuntimeFile {
    pub(crate) fn cpp_latest_bindable_property_for_object<'a>(
        &'a self,
        object: &RuntimeObject,
    ) -> Option<&'a RuntimeObject> {
        let object_id = usize::try_from(object.id).ok()?;
        let mut latest_bindable_property = None;
        let mut import_context = ImportContext::default();
        for candidate in self.objects.iter().take(object_id).flatten() {
            let candidate_id = usize::try_from(candidate.id).ok()?;
            let status = self.import_status(candidate_id)?;
            let Some(definition) = definition_by_type_key(candidate.type_key) else {
                continue;
            };
            if status == RuntimeImportStatus::Imported && definition.is_a("BindableProperty") {
                latest_bindable_property = Some(candidate);
            }

            // `BindablePropertyImporter::bindableProperty()` transfers its
            // pointer before delegating to `Super::import`. The transfer
            // therefore occurs even when that superclass later drops the
            // consumer for a missing owner. Reconstruct the pre-transfer
            // importer checks from the same pinned C++ call order; filtering
            // to imported consumers would incorrectly let a later action
            // reacquire an already-consumed property.
            if cpp_bindable_property_transfer_reached(candidate, definition, &import_context) {
                latest_bindable_property = None;
            }

            match status {
                RuntimeImportStatus::Imported => {
                    update_import_context(candidate, definition, &mut import_context, false);
                }
                RuntimeImportStatus::Dropped { .. } => {
                    import_context.read_dropped_object(definition);
                }
                RuntimeImportStatus::NullObject => import_context.read_null_object(),
            }
        }
        latest_bindable_property
    }
}
