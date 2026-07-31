// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_property.cpp`.
// Authored property paths, typed source handles, lookup, and ordered schema access.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelNumberSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelNumberSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelBooleanSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelBooleanSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelStringSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelStringSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelColorSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelColorSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelEnumSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelEnumSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelSymbolListIndexSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelSymbolListIndexSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelAssetSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelAssetSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelFontAssetSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelFontAssetSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelArtboardSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelArtboardSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelTriggerSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelTriggerSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelListSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelListSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

fn runtime_owned_view_model_path_key(path: &[usize]) -> u64 {
    let mut key = 0xcbf29ce484222325u64;
    for segment in path {
        key ^= *segment as u64;
        key = key.wrapping_mul(0x100000001b3);
    }
    key
}

fn runtime_owned_view_model_property_index_by_name(
    property_names: &[(String, usize)],
    property_name: &str,
) -> Option<usize> {
    property_names
        .iter()
        .find_map(|(name, index)| (name == property_name).then_some(*index))
}

pub(crate) fn runtime_project_name_hash(name: &str) -> u32 {
    name.as_bytes().iter().fold(0x811c_9dc5, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeProjectNameAmbiguity;

fn runtime_owned_view_model_property_index_by_project_name_hash(
    property_names: &[(String, usize)],
    name_hash: u32,
) -> Result<Option<usize>, RuntimeProjectNameAmbiguity> {
    let mut matches = property_names.iter().filter_map(|(name, index)| {
        (runtime_project_name_hash(name) == name_hash).then_some(*index)
    });
    let Some(property_index) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Err(RuntimeProjectNameAmbiguity);
    }
    Ok(Some(property_index))
}

fn runtime_owned_view_model_property_index_by_project_name(
    property_names: &[(String, usize)],
    property_name: &str,
) -> Result<Option<usize>, RuntimeProjectNameAmbiguity> {
    let mut matches = property_names
        .iter()
        .filter_map(|(name, index)| (name == property_name).then_some(*index));
    let Some(property_index) = matches.next() else {
        return Ok(None);
    };
    if matches.next().is_some() {
        return Err(RuntimeProjectNameAmbiguity);
    }
    Ok(Some(property_index))
}

fn runtime_owned_view_model_property_names(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<(String, usize)> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .map(|(property_index, property)| {
                    (
                        property
                            .string_property("name")
                            .unwrap_or_default()
                            .to_owned(),
                        property_index,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn runtime_imported_view_model_number_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyNumber",
    )
}

pub(super) fn runtime_imported_view_model_number_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyNumber"],
    )
}

pub(crate) fn runtime_default_view_model_number_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_number_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_number_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_number_property_path_for_name_path(file, 0, property_path)
}

pub(super) fn runtime_imported_view_model_boolean_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyBoolean",
    )
}

pub(super) fn runtime_imported_view_model_boolean_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyBoolean"],
    )
}

pub(crate) fn runtime_default_view_model_boolean_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_boolean_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_boolean_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_boolean_property_path_for_name_path(file, 0, property_path)
}

pub(super) fn runtime_imported_view_model_string_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyString",
    )
}

pub(super) fn runtime_imported_view_model_string_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyString"],
    )
}

pub(crate) fn runtime_default_view_model_string_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_string_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_string_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_string_property_path_for_name_path(file, 0, property_path)
}

pub(super) fn runtime_imported_view_model_color_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyColor",
    )
}

pub(super) fn runtime_imported_view_model_color_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyColor"],
    )
}

pub(crate) fn runtime_default_view_model_color_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_color_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_color_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_color_property_path_for_name_path(file, 0, property_path)
}

pub(super) fn runtime_imported_view_model_symbol_list_index_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertySymbolListIndex",
    )
}

pub(super) fn runtime_imported_view_model_symbol_list_index_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertySymbolListIndex"],
    )
}

pub(crate) fn runtime_default_view_model_symbol_list_index_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_symbol_list_index_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_symbol_list_index_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_symbol_list_index_property_path_for_name_path(
        file,
        0,
        property_path,
    )
}

pub(super) fn runtime_imported_view_model_asset_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &["ViewModelPropertyAsset", "ViewModelPropertyAssetImage"],
    )
}

pub(super) fn runtime_imported_view_model_asset_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyAsset", "ViewModelPropertyAssetImage"],
    )
}

pub(crate) fn runtime_default_view_model_asset_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_asset_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_asset_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_asset_property_path_for_name_path(file, 0, property_path)
}

pub(super) fn runtime_imported_view_model_artboard_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyArtboard",
    )
}

pub(super) fn runtime_imported_view_model_artboard_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyArtboard"],
    )
}

pub(crate) fn runtime_default_view_model_artboard_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_artboard_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_artboard_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_artboard_property_path_for_name_path(file, 0, property_path)
}

pub(super) fn runtime_imported_view_model_trigger_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyTrigger",
    )
}

pub(super) fn runtime_imported_view_model_trigger_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyTrigger"],
    )
}

pub(crate) fn runtime_default_view_model_trigger_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_trigger_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_trigger_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_trigger_property_path_for_name_path(file, 0, property_path)
}

pub(super) fn runtime_imported_view_model_list_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyList",
    )
}

pub(super) fn runtime_imported_view_model_list_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyList"],
    )
}

pub(crate) fn runtime_default_view_model_list_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_list_property_path_for_name(file, 0, property_name)
}

pub(crate) fn runtime_default_view_model_list_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_list_property_path_for_name_path(file, 0, property_path)
}

pub(super) fn runtime_imported_view_model_view_model_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name(
        file,
        view_model_index,
        property_name,
        "ViewModelPropertyViewModel",
    )
}

pub(super) fn runtime_imported_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_name_path(
        file,
        view_model_index,
        property_path,
        &["ViewModelPropertyViewModel"],
    )
}

fn runtime_imported_view_model_property_path_for_name(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
    property_type_name: &str,
) -> Option<Vec<u32>> {
    runtime_imported_view_model_property_path_for_type_names(
        file,
        view_model_index,
        property_name,
        &[property_type_name],
    )
}

fn runtime_imported_view_model_property_path_for_type_names(
    file: &RuntimeFile,
    view_model_index: usize,
    property_name: &str,
    property_type_names: &[&str],
) -> Option<Vec<u32>> {
    if property_name.is_empty() {
        return None;
    }
    let view_model = file.view_model(view_model_index)?;
    view_model
        .properties
        .into_iter()
        .enumerate()
        .find_map(|(property_index, property)| {
            if !property_type_names.contains(&property.type_name) {
                return None;
            }
            if property.string_property("name")? != property_name {
                return None;
            }
            Some(vec![
                u32::try_from(view_model_index).ok()?,
                u32::try_from(property_index).ok()?,
            ])
        })
}

fn runtime_imported_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
    property_type_names: &[&str],
) -> Option<Vec<u32>> {
    let property_names = property_path.split('/').collect::<Vec<_>>();
    if property_names.is_empty() || property_names.iter().any(|segment| segment.is_empty()) {
        return None;
    }

    let mut current_view_model_index = view_model_index;
    let mut path = vec![u32::try_from(view_model_index).ok()?];
    for (property_name_index, property_name) in property_names.iter().enumerate() {
        let view_model = file.view_model(current_view_model_index)?;
        let (property_index, property) = view_model
            .properties
            .into_iter()
            .enumerate()
            .find(|(_, property)| property.string_property("name") == Some(*property_name))?;
        path.push(u32::try_from(property_index).ok()?);
        if property_name_index + 1 == property_names.len() {
            return property_type_names
                .contains(&property.type_name)
                .then_some(path);
        }
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        current_view_model_index =
            usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
    }

    None
}
