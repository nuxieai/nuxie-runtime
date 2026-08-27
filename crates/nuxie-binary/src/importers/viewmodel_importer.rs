use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "ViewModel" {
        return Some(true);
    }
    definition
        .is_a("ViewModelProperty")
        .then(|| context.latest(ImportStackKey::ViewModel))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "ViewModel" {
        context.make_latest(ImportStackKey::ViewModel);
    }
}
impl RuntimeFile {
    pub(crate) fn cpp_view_models(&self) -> Vec<RuntimeViewModel<'_>> {
        let mut view_models = Vec::<RuntimeViewModel<'_>>::new();
        let mut latest_view_model = None;
        let mut latest_view_model_instance =
            None::<viewmodel_instance_importer::ViewModelInstanceImporter>;
        let mut latest_view_model_instance_list = None;

        for (index, object) in self.objects.iter().enumerate() {
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.name == "ViewModel" {
                view_models.push(RuntimeViewModel {
                    object,
                    properties: Vec::new(),
                    instances: Vec::new(),
                });
                latest_view_model = Some(view_models.len() - 1);
                continue;
            }

            if definition.is_a("ViewModelProperty") {
                if let Some(view_model_index) = latest_view_model {
                    view_models[view_model_index].properties.push(object);
                }
                continue;
            }

            if definition.name == "ViewModelInstance" {
                if let Some(previous) = latest_view_model_instance.take() {
                    let _ = previous.resolve();
                }
                let Some(view_model_index) = object.uint_property("viewModelId") else {
                    continue;
                };
                let Ok(view_model_index) = usize::try_from(view_model_index) else {
                    continue;
                };
                if let Some(view_model) = view_models.get_mut(view_model_index) {
                    view_model.instances.push(RuntimeViewModelInstance {
                        object,
                        values: Vec::new(),
                    });
                    let next = viewmodel_instance_importer::ViewModelInstanceImporter::new(
                        view_model_index,
                        view_model.instances.len() - 1,
                    );
                    latest_view_model_instance = Some(next);
                }
                continue;
            }

            if definition.is_a("ViewModelInstanceValue") {
                let Some(view_model_instance_importer) = latest_view_model_instance else {
                    if definition.name == "ViewModelInstanceList" {
                        latest_view_model_instance_list = None;
                    }
                    continue;
                };
                let Some(value_index) =
                    view_model_instance_importer.add_value(&mut view_models, object)
                else {
                    continue;
                };
                if definition.name == "ViewModelInstanceList" {
                    let (view_model_index, instance_index) =
                        view_model_instance_importer.view_model_instance();
                    latest_view_model_instance_list = Some((
                        view_model_index,
                        instance_index,
                        value_index,
                    ));
                }
                continue;
            }

            if definition.name == "ViewModelInstanceListItem" {
                let Some((view_model_index, instance_index, value_index)) =
                    latest_view_model_instance_list
                else {
                    continue;
                };
                view_models[view_model_index].instances[instance_index].values[value_index]
                    .list_items
                    .push(object);
            }
        }

        if let Some(view_model_instance_importer) = latest_view_model_instance {
            let _ = view_model_instance_importer.resolve();
        }

        view_models
    }
}
