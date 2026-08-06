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
        let mut latest_view_model_instance = None;
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
                latest_view_model_instance = None;
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
                    latest_view_model_instance =
                        Some((view_model_index, view_model.instances.len() - 1));
                }
                continue;
            }

            if definition.is_a("ViewModelInstanceValue") {
                let Some((view_model_index, instance_index)) = latest_view_model_instance else {
                    if definition.name == "ViewModelInstanceList" {
                        latest_view_model_instance_list = None;
                    }
                    continue;
                };
                view_models[view_model_index].instances[instance_index]
                    .values
                    .push(RuntimeViewModelInstanceValue {
                        object,
                        list_items: Vec::new(),
                    });
                if definition.name == "ViewModelInstanceList" {
                    latest_view_model_instance_list = Some((
                        view_model_index,
                        instance_index,
                        view_models[view_model_index].instances[instance_index]
                            .values
                            .len()
                            - 1,
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

        view_models
    }
}
