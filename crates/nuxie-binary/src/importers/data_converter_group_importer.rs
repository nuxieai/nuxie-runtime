use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "DataConverterGroupItem" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("group item is owned by DataConverterGroupImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.name == "DataConverterGroup" {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "DataConverterGroupItem").then(|| {
        context.latest(ImportStackKey::Backboard)
            && context.latest(ImportStackKey::DataConverterGroup)
    })
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "DataConverterGroup" {
        context.make_latest(ImportStackKey::DataConverterGroup);
    }
}
impl RuntimeFile {
    pub(crate) fn cpp_data_converter_group_items(
        &self,
        data_converter_index: usize,
    ) -> Vec<RuntimeDataConverterGroupItem<'_>> {
        let Some(group) = self.data_converter(data_converter_index) else {
            return Vec::new();
        };
        if group.type_name != "DataConverterGroup" {
            return Vec::new();
        }

        let mut current_group_index = None;
        let mut current_converter_index = 0usize;
        let mut items = Vec::new();

        for (file_index, object) in self.objects.iter().enumerate() {
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.is_a("DataConverter") {
                if definition.name == "DataConverterGroup" {
                    current_group_index = Some(current_converter_index);
                }
                current_converter_index += 1;
                continue;
            }

            if definition.name == "DataConverterGroupItem"
                && current_group_index == Some(data_converter_index)
            {
                items.push(RuntimeDataConverterGroupItem {
                    object,
                    converter: self.resolved_data_converter_for_group_item_object(object),
                });
            }
        }

        items
    }

    pub(crate) fn cpp_data_converter_group_child_converter_ids(
        &self,
        data_converter: &RuntimeObject,
    ) -> Vec<usize> {
        if data_converter.type_name != "DataConverterGroup" {
            return Vec::new();
        }

        self.data_converter_group_items_for_object(data_converter)
            .into_iter()
            .filter_map(|item| {
                item.converter
                    .and_then(|converter| usize::try_from(converter.id).ok())
            })
            .collect()
    }
}
