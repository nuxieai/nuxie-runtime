use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "Artboard" {
        return Some(context.latest(ImportStackKey::Backboard));
    }
    None
}

pub(super) fn component_imports_successfully(
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition
        .is_a("Component")
        .then(|| context.latest(ImportStackKey::Artboard))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "Artboard" {
        context.artboard_local_nested_inputs.clear();
        context.make_latest(ImportStackKey::Artboard);
    }
}

pub(crate) fn validate_cpp_import_resolution(
    objects: &[Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
) -> Result<()> {
    state_machine_layer_importer::validate_cpp_state_machine_layers(objects, import_statuses)?;
    for range in runtime_artboard_ranges(objects, import_statuses) {
        let mut slots = runtime_artboard_local_slots(objects, import_statuses, range);
        validate_cpp_artboard_local_slots(&mut slots, objects);

        validate_cpp_constraint_parentage(&slots, objects)?;
        validate_cpp_text_parentage(&slots, objects)?;
        validate_cpp_paint_effects(&slots, objects)?;

        for (local_index, slot) in slots.iter().enumerate() {
            let Some(file_index) = *slot else {
                continue;
            };
            let Some(object) = objects[file_index].as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };
            if definition.is_a("Drawable") {
                let raw_blend_mode = object.uint_property("blendModeValue").unwrap_or(3);
                let blend_mode = raw_blend_mode as u8;
                if !cpp_drawable_blend_mode_is_valid(blend_mode) {
                    bail!(
                        "drawable object {} ({}) has invalid blendModeValue {}",
                        object.id,
                        object.type_name,
                        raw_blend_mode
                    );
                }
            }

            if definition.name == "Mesh" {
                validate_cpp_mesh_indices(object, local_index, &slots, objects)?;
            }
        }
    }

    Ok(())
}
impl RuntimeFile {
    pub(crate) fn cpp_artboards(&self) -> impl Iterator<Item = &RuntimeObject> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                (object.type_name == "Artboard").then_some(object)
            })
    }

    pub(crate) fn cpp_has_latest_artboard_importer_before(&self, object_index: usize) -> bool {
        self.objects
            .iter()
            .take(object_index)
            .enumerate()
            .any(|(index, object)| {
                self.import_status(index) == Some(RuntimeImportStatus::Imported)
                    && object
                        .as_ref()
                        .is_some_and(|object| object.type_name == "Artboard")
            })
    }

    pub(crate) fn cpp_artboard_objects_named(
        &self,
        artboard_index: usize,
        type_name: &'static str,
    ) -> Vec<&RuntimeObject> {
        let Some((start, end)) = self.cpp_artboard_range(artboard_index) else {
            return Vec::new();
        };

        self.objects[start..end]
            .iter()
            .enumerate()
            .filter_map(|(offset, object)| {
                let file_index = start + offset;
                if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                (object.type_name == type_name).then_some(object)
            })
            .collect()
    }

    pub(crate) fn cpp_artboard_range(&self, artboard_index: usize) -> Option<(usize, usize)> {
        runtime_artboard_ranges(&self.objects, &self.import_statuses)
            .get(artboard_index)
            .copied()
    }

    pub(crate) fn cpp_artboard_index(
        &self,
        artboard_index: usize,
    ) -> Option<RuntimeArtboardIndex<'_>> {
        let range = self.cpp_artboard_range(artboard_index)?;
        Some(RuntimeArtboardIndex::new(
            &self.objects,
            &self.import_statuses,
            range,
        ))
    }

    pub(crate) fn cpp_artboard_local_context_for_object(
        &self,
        object: &RuntimeObject,
    ) -> Option<(usize, (usize, usize), Vec<Option<usize>>, usize)> {
        let file_index = usize::try_from(object.id).ok()?;
        for (artboard_index, range) in runtime_artboard_ranges(&self.objects, &self.import_statuses)
            .into_iter()
            .enumerate()
        {
            if file_index < range.0 || file_index >= range.1 {
                continue;
            }

            let mut slots =
                runtime_artboard_local_slots(&self.objects, &self.import_statuses, range);
            validate_cpp_artboard_local_slots(&mut slots, &self.objects);
            let local_index = slots.iter().position(|slot| *slot == Some(file_index))?;
            return Some((artboard_index, range, slots, local_index));
        }

        None
    }

    pub(crate) fn resolved_axis_animation_for_joystick_object(
        &self,
        joystick: &RuntimeObject,
        property_name: &str,
    ) -> Option<&RuntimeObject> {
        if joystick.type_name != "Joystick" {
            return None;
        }

        let joystick_id = usize::try_from(joystick.id).ok()?;
        if self.import_status(joystick_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let (artboard_index, _, _, _) = self.cpp_artboard_local_context_for_object(joystick)?;
        let animation_index = usize::try_from(joystick.uint_property(property_name)?).ok()?;
        self.artboard_animation(artboard_index, animation_index)
    }
}
