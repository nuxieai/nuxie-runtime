use super::*;

/// The converter index is the Rust equivalent of pinned
/// `DataConverterGroup* m_dataConverterGroup`.
///
/// It is intentionally non-owning: dropping the importer must not drop the
/// group retained by the runtime file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DataConverterGroupImporter {
    group_index: usize,
}

impl DataConverterGroupImporter {
    /// Mechanical translation of the constructor: retain exactly the group
    /// supplied when the file creates this importer.
    fn new(group_index: usize) -> Self {
        Self { group_index }
    }

    /// Mechanical translation of the primary-header `group()` inline.
    fn group(self) -> usize {
        self.group_index
    }
}

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

        let mut current_group_importer = None;
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
                    current_group_importer = Some(DataConverterGroupImporter::new(
                        current_converter_index,
                    ));
                }
                current_converter_index += 1;
                continue;
            }

            if definition.name == "DataConverterGroupItem"
                && current_group_importer
                    .is_some_and(|importer| importer.group() == data_converter_index)
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
