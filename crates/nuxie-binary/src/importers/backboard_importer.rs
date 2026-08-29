//! Direct import owner for pinned C++ `src/importers/backboard_importer.cpp`
//! and `include/rive/importers/backboard_importer.hpp`.
//!
//! The immutable Rust file reconstructs the C++ importer's retained vectors
//! from imported file order. Cross-owner callbacks remain at their concrete
//! source owners: FileAssetReferencer assignment is in
//! `assets/file_asset_referencer.rs`, Artboard collection is in
//! `importers/artboard_importer.rs`, and ViewModelInstance attachment is in
//! `importers/viewmodel_importer.rs`. All Backboard-owned selection and
//! resolve rules are routed through this module.

use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "Backboard" {
        return Some(true);
    }
    if definition.is_a("DataBind") || definition.is_a("DataConverter") {
        return Some(context.latest(ImportStackKey::Backboard));
    }
    if definition.is_a("ScrollPhysics") {
        return Some(context.latest(ImportStackKey::Backboard));
    }
    definition.is_a("KeyFrameInterpolator").then(|| {
        context.latest(ImportStackKey::Artboard) || context.latest(ImportStackKey::Backboard)
    })
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "Backboard" {
        context.make_latest(ImportStackKey::Backboard);
    }
}

/// Pinned `BackboardImporter::addFileAsset`, including the editor-bug-4204
/// duplicate-id repair after every imported, backboard-owned asset is added.
pub(crate) fn normalize_file_asset_ids(
    objects: &mut [Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
) {
    let mut file_asset_ids = Vec::new();
    for index in 0..objects.len() {
        if import_statuses.get(index) != Some(&RuntimeImportStatus::Imported) {
            continue;
        }

        let Some(object) = objects[index].as_ref() else {
            continue;
        };
        if object.type_name == "Backboard" {
            // ImportStack::makeLatest resolves and destroys the previous
            // BackboardImporter before installing this fresh owner.
            file_asset_ids.clear();
            continue;
        }
        // FileAsset::import calls addFileAsset only when addsToBackboard() is
        // true. ManifestAsset is the pinned exception.
        let is_backboard_file_asset =
            definition_by_type_key(object.type_key).is_some_and(|definition| {
                definition.is_a("FileAsset") && definition.name != "ManifestAsset"
            });
        if !is_backboard_file_asset {
            continue;
        }

        file_asset_ids.push(index);
        normalize_file_asset_ids_for_imported_assets(objects, &file_asset_ids);
    }
}

fn normalize_file_asset_ids_for_imported_assets(
    objects: &mut [Option<RuntimeObject>],
    file_asset_ids: &[usize],
) {
    // C++ uses unordered_set only for membership, so ordered Rust storage does
    // not change observable iteration or duplicate selection.
    let mut ids = BTreeSet::new();
    let mut next_id = 1u32;

    for object_id in file_asset_ids {
        let object = objects[*object_id]
            .as_mut()
            .expect("file_asset_ids only contains present objects");
        let asset_id = object.uint_property("assetId").unwrap_or(0) as u32;
        if ids.contains(&asset_id) {
            upsert_runtime_property(
                &mut object.properties,
                RuntimeProperty {
                    key: 204,
                    name: "assetId",
                    owner: "FileAsset",
                    value: FieldValue::Uint(u64::from(next_id)),
                },
            );
        } else {
            ids.insert(asset_id);
            if asset_id >= next_id {
                next_id = asset_id.wrapping_add(1);
            }
        }
    }
}

impl RuntimeFile {
    pub(crate) fn cpp_backboard_range_for_object(
        &self,
        owner: &RuntimeObject,
    ) -> Option<(usize, usize)> {
        let owner_file_index = usize::try_from(owner.id).ok()?;
        if self.import_status(owner_file_index) != Some(RuntimeImportStatus::Imported) {
            return None;
        }
        self.cpp_backboard_range_for_file_index(owner_file_index)
    }

    fn cpp_backboard_range_for_file_index(&self, file_index: usize) -> Option<(usize, usize)> {
        let backboard_start = self
            .objects
            .get(..=file_index)?
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, object)| {
                (self.import_status(index) == Some(RuntimeImportStatus::Imported)
                    && object
                        .as_ref()
                        .is_some_and(|object| object.type_name == "Backboard"))
                .then_some(index)
            })?;

        let backboard_end = self.objects[backboard_start + 1..]
            .iter()
            .enumerate()
            .find_map(|(offset, object)| {
                let index = backboard_start + 1 + offset;
                (self.import_status(index) == Some(RuntimeImportStatus::Imported)
                    && object
                        .as_ref()
                        .is_some_and(|object| object.type_name == "Backboard"))
                .then_some(index)
            })
            .unwrap_or(self.objects.len());
        Some((backboard_start, backboard_end))
    }

    fn cpp_latest_backboard_range(&self) -> Option<(usize, usize)> {
        let backboard_start =
            self.objects
                .iter()
                .enumerate()
                .rev()
                .find_map(|(index, object)| {
                    (self.import_status(index) == Some(RuntimeImportStatus::Imported)
                        && object
                            .as_ref()
                            .is_some_and(|object| object.type_name == "Backboard"))
                    .then_some(index)
                })?;
        Some((backboard_start, self.objects.len()))
    }

    /// Pinned `addArtboardReferencer` plus the artboard-reference loop in
    /// `resolve`. The immutable file representation observes the assigned
    /// pointer by resolving the retained serialized artboard index on demand.
    pub fn resolved_artboard_for_referencer(&self, object_id: usize) -> Option<&RuntimeObject> {
        let referencer = self.object(object_id)?;
        self.resolved_artboard_for_referencer_object(referencer)
    }

    pub fn resolved_artboard_for_referencer_object(
        &self,
        referencer: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(referencer.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let artboard_index = usize::try_from(cpp_artboard_referencer_index(referencer)?).ok()?;
        self.cpp_backboard_artboard_for_referencer(referencer, artboard_index)
    }

    /// Reconstructs `m_ArtboardLookup` for the BackboardImporter that accepted
    /// this referencer. Unlike `File::artboard(index)`, this table retains the
    /// holes created by `addMissingArtboard`.
    fn cpp_backboard_artboard_for_referencer(
        &self,
        referencer: &RuntimeObject,
        artboard_index: usize,
    ) -> Option<&RuntimeObject> {
        let (backboard_start, backboard_end) = self.cpp_backboard_range_for_object(referencer)?;

        let mut next_artboard_id = 0usize;
        for (offset, object) in self.objects[backboard_start + 1..backboard_end]
            .iter()
            .enumerate()
        {
            let file_index = backboard_start + 1 + offset;
            let Some(object) = object.as_ref() else {
                continue;
            };
            if object.type_name != "Artboard" {
                continue;
            }

            let candidate_id = next_artboard_id;
            next_artboard_id += 1;
            if candidate_id == artboard_index
                && self.import_status(file_index) == Some(RuntimeImportStatus::Imported)
            {
                return Some(object);
            }
        }
        None
    }

    pub(crate) fn cpp_data_converters(&self) -> impl Iterator<Item = &RuntimeObject> {
        self.cpp_latest_backboard_range()
            .into_iter()
            .flat_map(|range| self.cpp_data_converters_in_range(range))
    }

    fn cpp_data_converters_for_backboard_owner(
        &self,
        owner: &RuntimeObject,
    ) -> Vec<&RuntimeObject> {
        self.cpp_backboard_range_for_object(owner)
            .map(|range| self.cpp_data_converters_in_range(range))
            .unwrap_or_default()
    }

    fn cpp_data_converters_in_range(&self, (start, end): (usize, usize)) -> Vec<&RuntimeObject> {
        self.objects[start + 1..end]
            .iter()
            .enumerate()
            .filter_map(|(offset, object)| {
                let index = start + 1 + offset;
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                definition_by_type_key(object.type_key)
                    .is_some_and(|definition| definition.is_a("DataConverter"))
                    .then_some(object)
            })
            .collect()
    }

    pub(crate) fn cpp_file_assets_for_backboard_owner(
        &self,
        owner: &RuntimeObject,
    ) -> Vec<&RuntimeObject> {
        let Some((start, end)) = self.cpp_backboard_range_for_object(owner) else {
            return Vec::new();
        };
        self.cpp_file_assets_in_range((start, end))
    }

    pub(crate) fn cpp_file_assets_for_backboard_owner_before(
        &self,
        owner: &RuntimeObject,
    ) -> Vec<&RuntimeObject> {
        let Some((start, _)) = self.cpp_backboard_range_for_object(owner) else {
            return Vec::new();
        };
        let Ok(owner_file_index) = usize::try_from(owner.id) else {
            return Vec::new();
        };
        self.cpp_file_assets_in_range((start, owner_file_index))
    }

    fn cpp_file_assets_in_range(&self, (start, end): (usize, usize)) -> Vec<&RuntimeObject> {
        self.objects[start + 1..end]
            .iter()
            .enumerate()
            .filter_map(|(offset, object)| {
                let index = start + 1 + offset;
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }
                let object = object.as_ref()?;
                cpp_file_assets_contains(object).then_some(object)
            })
            .collect()
    }

    /// Pinned `addPhysics`, `physics`, and the consumer-side lookup that
    /// observes the cloned physics pointer installed during import.
    pub fn scroll_physics(&self) -> Vec<&RuntimeObject> {
        self.cpp_scroll_physics().collect()
    }

    pub fn scroll_physics_object(&self, index: usize) -> Option<&RuntimeObject> {
        self.cpp_scroll_physics().nth(index)
    }

    pub fn resolved_scroll_physics_for_constraint(
        &self,
        scroll_constraint_id: usize,
    ) -> Option<&RuntimeObject> {
        let scroll_constraint = self.object(scroll_constraint_id)?;
        self.resolved_scroll_physics_for_constraint_object(scroll_constraint)
    }

    pub fn resolved_scroll_physics_for_constraint_object(
        &self,
        scroll_constraint: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(scroll_constraint.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported)
            || scroll_constraint.type_name != "ScrollConstraint"
        {
            return None;
        }

        let physics_index = usize::try_from(scroll_constraint.uint_property("physicsId")?).ok()?;
        self.cpp_scroll_physics_for_backboard_owner(scroll_constraint)
            .into_iter()
            .nth(physics_index)
    }

    /// The two converter cases in pinned `resolve`, retaining the exact
    /// unsigned sentinel/out-of-range behavior of the C++ vector lookup.
    pub fn resolved_interpolator_for_data_converter(
        &self,
        data_converter_id: usize,
    ) -> Option<&RuntimeObject> {
        let data_converter = self.object(data_converter_id)?;
        self.resolved_interpolator_for_data_converter_object(data_converter)
    }

    pub fn resolved_interpolator_for_data_converter_object(
        &self,
        data_converter: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(data_converter.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported)
            || !matches!(
                data_converter.type_name,
                "DataConverterRangeMapper" | "DataConverterInterpolator"
            )
        {
            return None;
        }

        let interpolator_index =
            usize::try_from(data_converter.uint_property("interpolatorId")?).ok()?;
        self.cpp_data_converter_interpolators_for_backboard_owner(data_converter)
            .into_iter()
            .nth(interpolator_index)
    }

    /// Pinned `addDataConverterReferencer` plus the DataBind resolve loop.
    /// Rust retains immutable references, so the C++ per-bind converter clone
    /// has no separately observable mutable identity in this representation.
    pub fn resolved_data_converter_for_data_bind(
        &self,
        data_bind_id: usize,
    ) -> Option<&RuntimeObject> {
        let data_bind = self.object(data_bind_id)?;
        self.resolved_data_converter_for_data_bind_object(data_bind)
    }

    pub fn resolved_data_converter_for_data_bind_object(
        &self,
        data_bind: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(data_bind.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let definition = definition_by_type_key(data_bind.type_key)?;
        if !definition.is_a("DataBind") {
            return None;
        }

        let converter_index = usize::try_from(data_bind.uint_property("converterId")?).ok()?;
        self.cpp_data_converters_for_backboard_owner(data_bind)
            .into_iter()
            .nth(converter_index)
    }

    /// Pinned `addDataConverterGroupItemReferencer` plus its non-cloning
    /// resolve loop, kept separate from DataBind's cloning relationship.
    pub fn resolved_data_converter_for_group_item(
        &self,
        group_item_id: usize,
    ) -> Option<&RuntimeObject> {
        let group_item = self.object(group_item_id)?;
        self.resolved_data_converter_for_group_item_object(group_item)
    }

    pub fn resolved_data_converter_for_group_item_object(
        &self,
        group_item: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(group_item.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported)
            || group_item.type_name != "DataConverterGroupItem"
        {
            return None;
        }

        let converter_index = usize::try_from(group_item.uint_property("converterId")?).ok()?;
        self.cpp_data_converters_for_backboard_owner(group_item)
            .into_iter()
            .nth(converter_index)
    }

    /// Pinned `BackboardImporter::addInterpolator`. An Artboard importer takes
    /// precedence once it exists, so only the imported file-global prefix is
    /// retained by this owner.
    pub(crate) fn cpp_data_converter_interpolators(&self) -> Vec<&RuntimeObject> {
        self.cpp_latest_backboard_range()
            .map(|range| self.cpp_data_converter_interpolators_in_range(range))
            .unwrap_or_default()
    }

    fn cpp_data_converter_interpolators_for_backboard_owner(
        &self,
        owner: &RuntimeObject,
    ) -> Vec<&RuntimeObject> {
        self.cpp_backboard_range_for_object(owner)
            .map(|range| self.cpp_data_converter_interpolators_in_range(range))
            .unwrap_or_default()
    }

    fn cpp_data_converter_interpolators_in_range(
        &self,
        (start, end): (usize, usize),
    ) -> Vec<&RuntimeObject> {
        // ArtboardImporter is a distinct ImportStack key and survives a later
        // Backboard replacement, so its presence must be reconstructed from
        // the full prefix rather than only this Backboard segment.
        let mut latest_artboard_importer =
            self.objects[..=start]
                .iter()
                .enumerate()
                .any(|(index, object)| {
                    self.import_status(index) == Some(RuntimeImportStatus::Imported)
                        && object
                            .as_ref()
                            .is_some_and(|object| object.type_name == "Artboard")
                });
        let mut interpolators = Vec::new();

        for (offset, object) in self.objects[start + 1..end].iter().enumerate() {
            let index = start + 1 + offset;
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.name == "Artboard" {
                latest_artboard_importer = true;
                continue;
            }

            if definition.is_a("KeyFrameInterpolator") && !latest_artboard_importer {
                interpolators.push(object);
            }
        }

        interpolators
    }

    /// Pinned `BackboardImporter::addPhysics`, preserving import order.
    pub(crate) fn cpp_scroll_physics(&self) -> impl Iterator<Item = &RuntimeObject> {
        self.cpp_latest_backboard_range()
            .into_iter()
            .flat_map(|range| self.cpp_scroll_physics_in_range(range))
    }

    fn cpp_scroll_physics_for_backboard_owner(&self, owner: &RuntimeObject) -> Vec<&RuntimeObject> {
        self.cpp_backboard_range_for_object(owner)
            .map(|range| self.cpp_scroll_physics_in_range(range))
            .unwrap_or_default()
    }

    fn cpp_scroll_physics_in_range(&self, (start, end): (usize, usize)) -> Vec<&RuntimeObject> {
        self.objects[start + 1..end]
            .iter()
            .enumerate()
            .filter_map(|(offset, object)| {
                let index = start + 1 + offset;
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                definition_by_type_key(object.type_key)
                    .is_some_and(|definition| definition.is_a("ScrollPhysics"))
                    .then_some(object)
            })
            .collect()
    }
}

fn cpp_artboard_referencer_index(object: &RuntimeObject) -> Option<u64> {
    let definition = definition_by_type_key(object.type_key)?;
    (definition.is_a("NestedArtboard") || definition.name == "ScriptInputArtboard")
        .then(|| object.uint_property("artboardId"))
        .flatten()
}
