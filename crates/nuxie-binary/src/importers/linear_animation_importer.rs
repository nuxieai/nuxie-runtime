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
        // ImportStack retains each importer key independently across the
        // complete file. The optional output coordinates identify owners that
        // belong to this requested artboard; `None` remains a real sink for a
        // retained owner elsewhere or one rejected during later validation.
        let mut current_animation = None::<(u32, Option<usize>)>;
        let mut current_keyed_object = None::<Option<(usize, usize)>>;
        let mut current_keyed_property =
            None::<keyed_property_importer::KeyedPropertyImporter>;

        for (file_index, object) in self.objects.iter().enumerate() {
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
                let animation_index = if range.0 <= file_index && file_index < range.1 {
                    animations.push(RuntimeLinearAnimation {
                        object,
                        keyed_objects: Vec::new(),
                    });
                    Some(animations.len() - 1)
                } else {
                    None
                };
                current_animation = Some((
                    object.uint_property("fps").unwrap_or(60) as u32,
                    animation_index,
                ));
                continue;
            }

            if definition.name == "KeyedObject" {
                let Some((_, animation_index)) = current_animation else {
                    continue;
                };
                let keyed_object = animation_index.and_then(|animation_index| {
                    cpp_keyed_object_target(object, &local_slots, &self.objects)?;
                    animations[animation_index]
                        .keyed_objects
                        .push(RuntimeKeyedObject {
                            object,
                            keyed_properties: Vec::new(),
                        });
                    Some((
                        animation_index,
                        animations[animation_index].keyed_objects.len() - 1,
                    ))
                });
                current_keyed_object = Some(keyed_object);
                continue;
            }

            if definition.name == "KeyedProperty" {
                let Some((animation_fps, _)) = current_animation else {
                    continue;
                };
                let Some(keyed_object) = current_keyed_object else {
                    continue;
                };
                let keyed_property = keyed_object.and_then(|(animation_index, keyed_object_index)| {
                    if !cpp_keyed_object_supports_property(
                        animations[animation_index].keyed_objects[keyed_object_index].object,
                        object,
                        &local_slots,
                        &self.objects,
                    ) {
                        return None;
                    }
                    animations[animation_index].keyed_objects[keyed_object_index]
                        .keyed_properties
                        .push(RuntimeKeyedProperty {
                            object,
                            first_key_frame: None,
                            key_frames: Vec::new(),
                        });
                    Some((
                        animation_index,
                        keyed_object_index,
                        animations[animation_index].keyed_objects[keyed_object_index]
                            .keyed_properties
                            .len()
                            - 1,
                    ))
                });
                current_keyed_property = Some(
                    keyed_property_importer::KeyedPropertyImporter::new(
                        animation_fps,
                        keyed_property,
                    ),
                );
                continue;
            }

            if definition.is_a("KeyFrame") {
                if let Some(importer) = current_keyed_property.as_ref() {
                    importer.add_key_frame(&mut animations, object);
                }
            }
        }

        animations
    }
}
