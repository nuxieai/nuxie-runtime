use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    match definition.name {
        "DataEnum" | "DataEnumSystem" | "DataEnumCustom" | "DataEnumValue" => Some(
            imports_successfully(object, definition, context)
                .expect("data enums are owned by EnumImporter"),
        ),
        _ => None,
    }
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.name == "DataEnumCustom" {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    match definition.name {
        "DataEnum" | "DataEnumSystem" | "DataEnumCustom" => Some(true),
        "DataEnumValue" => Some(context.latest(ImportStackKey::DataEnumCustom)),
        _ => None,
    }
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "DataEnumCustom" {
        context.make_latest(ImportStackKey::DataEnumCustom);
    }
}
impl RuntimeFile {
    pub(crate) fn cpp_data_enums(&self) -> Vec<RuntimeDataEnum<'_>> {
        let mut data_enums = Vec::<RuntimeDataEnum<'_>>::new();
        let mut latest_custom_enum = None;

        for (index, object) in self.objects.iter().enumerate() {
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };

            match object.type_name {
                "DataEnum" | "DataEnumCustom" => {
                    data_enums.push(RuntimeDataEnum {
                        object,
                        values: Vec::new(),
                    });
                    if object.type_name == "DataEnumCustom" {
                        latest_custom_enum = Some(data_enums.len() - 1);
                    }
                }
                "DataEnumValue" => {
                    if let Some(enum_index) = latest_custom_enum {
                        data_enums[enum_index].values.push(object);
                    }
                }
                _ => {}
            }
        }

        data_enums
    }
}
