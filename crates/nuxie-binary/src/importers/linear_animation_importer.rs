use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "LinearAnimation" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("LinearAnimation is owned by LinearAnimationImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
) {
    if definition.name == "LinearAnimation" {
        update_context(definition, context);
    }
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "LinearAnimation").then(|| context.latest(ImportStackKey::Artboard))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "LinearAnimation" {
        context.make_latest(ImportStackKey::LinearAnimation);
    }
}
impl RuntimeFile {
    pub(crate) fn cpp_artboard_linear_animations(
        &self,
        artboard_index: usize,
    ) -> Vec<RuntimeLinearAnimation<'_>> {
        let Some(range) = self.cpp_artboard_range(artboard_index) else {
            return Vec::new();
        };
        let mut local_slots =
            runtime_artboard_local_slots(&self.objects, &self.import_statuses, range);
        validate_cpp_artboard_local_slots(&mut local_slots, &self.objects);

        let mut animations = Vec::<RuntimeLinearAnimation<'_>>::new();
        let mut current_animation = None;
        let mut current_keyed_object = None;
        let mut current_keyed_property = None;

        for (offset, object) in self.objects[range.0..range.1].iter().enumerate() {
            let file_index = range.0 + offset;
            let Some(object) = object.as_ref() else {
                continue;
            };
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.name == "LinearAnimation" {
                animations.push(RuntimeLinearAnimation {
                    object,
                    keyed_objects: Vec::new(),
                });
                current_animation = Some(animations.len() - 1);
                current_keyed_object = None;
                current_keyed_property = None;
                continue;
            }

            let Some(animation_index) = current_animation else {
                continue;
            };

            if definition.name == "KeyedObject" {
                if cpp_keyed_object_target(object, &local_slots, &self.objects).is_none() {
                    current_keyed_object = None;
                    current_keyed_property = None;
                    continue;
                }

                animations[animation_index]
                    .keyed_objects
                    .push(RuntimeKeyedObject {
                        object,
                        keyed_properties: Vec::new(),
                    });
                current_keyed_object = Some(animations[animation_index].keyed_objects.len() - 1);
                current_keyed_property = None;
                continue;
            }

            if definition.name == "KeyedProperty" {
                let Some(keyed_object_index) = current_keyed_object else {
                    continue;
                };
                if !cpp_keyed_object_supports_property(
                    animations[animation_index].keyed_objects[keyed_object_index].object,
                    object,
                    &local_slots,
                    &self.objects,
                ) {
                    current_keyed_property = None;
                    continue;
                }

                animations[animation_index].keyed_objects[keyed_object_index]
                    .keyed_properties
                    .push(RuntimeKeyedProperty {
                        object,
                        first_key_frame: None,
                    });
                current_keyed_property = Some((
                    keyed_object_index,
                    animations[animation_index].keyed_objects[keyed_object_index]
                        .keyed_properties
                        .len()
                        - 1,
                ));
                continue;
            }

            if definition.is_a("KeyFrame") {
                let Some((keyed_object_index, keyed_property_index)) = current_keyed_property
                else {
                    continue;
                };
                let first_key_frame = &mut animations[animation_index].keyed_objects
                    [keyed_object_index]
                    .keyed_properties[keyed_property_index]
                    .first_key_frame;
                if first_key_frame.is_none() {
                    *first_key_frame = Some(object);
                }
            }
        }

        animations
    }
}
