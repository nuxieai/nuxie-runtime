// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_viewmodel.cpp`.
// Retained child endpoint identity, nested traversal, replacement, and clone remapping.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelViewModelSourceHandle {
    property_path: Vec<usize>,
}

impl RuntimeOwnedViewModelViewModelSourceHandle {
    pub fn property_index(&self) -> usize {
        self.property_path[self.property_path.len() - 1]
    }

    pub fn path(&self) -> &[usize] {
        &self.property_path
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedViewModelViewModel {
    property_index: usize,
    property_name: String,
    endpoint: RuntimeOwnedViewModelEndpoint,
    referenced_view_model_index: Option<usize>,
    property_names: Vec<(String, usize)>,
    numbers: Vec<RuntimeOwnedViewModelNumber>,
    imported_numbers: BTreeMap<u32, Vec<RuntimeOwnedViewModelNumber>>,
    booleans: Vec<RuntimeOwnedViewModelBoolean>,
    imported_booleans: BTreeMap<u32, Vec<RuntimeOwnedViewModelBoolean>>,
    strings: Vec<RuntimeOwnedViewModelString>,
    imported_strings: BTreeMap<u32, Vec<RuntimeOwnedViewModelString>>,
    colors: Vec<RuntimeOwnedViewModelColor>,
    imported_colors: BTreeMap<u32, Vec<RuntimeOwnedViewModelColor>>,
    enums: Vec<RuntimeOwnedViewModelEnum>,
    imported_enums: BTreeMap<u32, Vec<RuntimeOwnedViewModelEnum>>,
    symbol_list_indices: Vec<RuntimeOwnedViewModelSymbolListIndex>,
    imported_symbol_list_indices: BTreeMap<u32, Vec<RuntimeOwnedViewModelSymbolListIndex>>,
    lists: Vec<RuntimeOwnedViewModelList>,
    imported_lists: BTreeMap<u32, Vec<RuntimeOwnedViewModelList>>,
    assets: Vec<RuntimeOwnedViewModelAsset>,
    imported_assets: BTreeMap<u32, Vec<RuntimeOwnedViewModelAsset>>,
    font_assets: Vec<RuntimeOwnedViewModelFontAsset>,
    imported_font_assets: BTreeMap<u32, Vec<RuntimeOwnedViewModelFontAsset>>,
    artboards: Vec<RuntimeOwnedViewModelArtboard>,
    imported_artboards: BTreeMap<u32, Vec<RuntimeOwnedViewModelArtboard>>,
    triggers: Vec<RuntimeOwnedViewModelTrigger>,
    imported_triggers: BTreeMap<u32, Vec<RuntimeOwnedViewModelTrigger>>,
    value_order: Vec<RuntimeOwnedViewModelValueOccurrence>,
    imported_value_order: BTreeMap<u32, Vec<RuntimeOwnedViewModelValueOccurrence>>,
    imported_instance_names: BTreeMap<u32, String>,
    view_model_instance_ids: Vec<u32>,
    children: Vec<RuntimeOwnedViewModelViewModel>,
    imported_children: BTreeMap<u32, Vec<RuntimeOwnedViewModelViewModel>>,
}

impl RuntimeOwnedViewModelViewModel {
    fn active_value_order(&self) -> &[RuntimeOwnedViewModelValueOccurrence] {
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &self.value_order,
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_value_order
                .get(&object_id)
                .map(Vec::as_slice)
                .unwrap_or_default(),
            _ => &[],
        }
    }

    fn active_children(&self) -> Option<&[RuntimeOwnedViewModelViewModel]> {
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => Some(self.children.as_slice()),
            RuntimeViewModelPointer::Imported { object_id } => {
                self.imported_children.get(&object_id).map(Vec::as_slice)
            }
            _ => None,
        }
    }

    fn generated_children_mut(&mut self) -> Option<&mut Vec<RuntimeOwnedViewModelViewModel>> {
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => Some(&mut self.children),
            _ => None,
        }
    }

    fn active_children_mut(&mut self) -> Option<&mut Vec<RuntimeOwnedViewModelViewModel>> {
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => Some(&mut self.children),
            RuntimeViewModelPointer::Imported { object_id } => {
                self.imported_children.get_mut(&object_id)
            }
            _ => None,
        }
    }

    pub(crate) fn property_index_by_name(&self, property_name: &str) -> Option<usize> {
        runtime_owned_view_model_property_index_by_name(&self.property_names, property_name)
    }

    fn number_value_by_property_index(&self, property_index: usize) -> Option<f32> {
        self.numbers
            .iter()
            .find(|number| number.property_index == property_index)
            .map(|number| number.value())
    }

    fn active_number_value_by_property_index(&self, property_index: usize) -> Option<f32> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .number_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.number_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_numbers
                .get(&object_id)
                .and_then(|numbers| {
                    numbers
                        .iter()
                        .find(|number| number.property_index == property_index)
                })
                .map(|number| number.value()),
            _ => None,
        }
    }

    fn boolean_value_by_property_index(&self, property_index: usize) -> Option<bool> {
        self.booleans
            .iter()
            .find(|boolean| boolean.property_index == property_index)
            .map(|boolean| boolean.value())
    }

    fn active_boolean_value_by_property_index(&self, property_index: usize) -> Option<bool> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .boolean_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.boolean_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_booleans
                .get(&object_id)
                .and_then(|booleans| {
                    booleans
                        .iter()
                        .find(|boolean| boolean.property_index == property_index)
                })
                .map(|boolean| boolean.value()),
            _ => None,
        }
    }

    fn string_value_by_property_index(&self, property_index: usize) -> Option<Arc<[u8]>> {
        self.strings
            .iter()
            .find(|string| string.property_index == property_index)
            .map(|string| string.value())
    }

    fn active_string_value_by_property_index(&self, property_index: usize) -> Option<Arc<[u8]>> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .string_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.string_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_strings
                .get(&object_id)
                .and_then(|strings| {
                    strings
                        .iter()
                        .find(|string| string.property_index == property_index)
                })
                .map(|string| string.value()),
            _ => None,
        }
    }

    fn color_value_by_property_index(&self, property_index: usize) -> Option<u32> {
        self.colors
            .iter()
            .find(|color| color.property_index == property_index)
            .map(|color| color.value())
    }

    fn active_color_value_by_property_index(&self, property_index: usize) -> Option<u32> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .color_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.color_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_colors
                .get(&object_id)
                .and_then(|colors| {
                    colors
                        .iter()
                        .find(|color| color.property_index == property_index)
                })
                .map(|color| color.value()),
            _ => None,
        }
    }

    fn enum_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.enums
            .iter()
            .find(|enum_value| enum_value.property_index == property_index)
            .map(|enum_value| enum_value.value())
    }

    fn active_enum_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .enum_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.enum_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_enums
                .get(&object_id)
                .and_then(|enums| {
                    enums
                        .iter()
                        .find(|enum_value| enum_value.property_index == property_index)
                })
                .map(|enum_value| enum_value.value()),
            _ => None,
        }
    }

    fn symbol_list_index_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.symbol_list_indices
            .iter()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
            .map(|symbol_list_index| symbol_list_index.value())
    }

    fn active_symbol_list_index_value_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<u64> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .symbol_list_index_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.symbol_list_index_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_symbol_list_indices
                .get(&object_id)
                .and_then(|symbol_list_indices| {
                    symbol_list_indices.iter().find(|symbol_list_index| {
                        symbol_list_index.property_index == property_index
                    })
                })
                .map(|symbol_list_index| symbol_list_index.value()),
            _ => None,
        }
    }

    /// The retained property cell backing this child slot's ACTIVE storage at
    /// one property index (#RB-1 e3). Mirrors the
    /// `active_*_value_by_property_index` accessors but returns the shared
    /// cell itself — the C++ analog of a `DataContext` property walk landing
    /// on the retained `ViewModelInstanceValue*`.
    fn active_scalar_cell_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<RuntimeViewModelCell> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .scalar_cell_by_property_index(property_index);
        }
        let occurrence = self
            .active_value_order()
            .iter()
            .find(|occurrence| occurrence.property_index == property_index)?;
        macro_rules! active_slot {
            ($owned:ident, $imported:ident) => {{
                match self.endpoint.value() {
                    RuntimeViewModelPointer::OwnedGenerated { .. } => Some(self.$owned.as_slice()),
                    RuntimeViewModelPointer::Imported { object_id } => {
                        self.$imported.get(&object_id).map(Vec::as_slice)
                    }
                    _ => None,
                }?
                .get(occurrence.slot_index)?
            }};
        }
        Some(match occurrence.kind {
            RuntimeOwnedViewModelValueKind::Number => {
                active_slot!(numbers, imported_numbers).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::Boolean => {
                active_slot!(booleans, imported_booleans).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::String => {
                active_slot!(strings, imported_strings).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::Color => {
                active_slot!(colors, imported_colors).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::Enum => {
                active_slot!(enums, imported_enums).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::SymbolListIndex => {
                active_slot!(symbol_list_indices, imported_symbol_list_indices)
                    .cell
                    .clone()
            }
            RuntimeOwnedViewModelValueKind::List => {
                active_slot!(lists, imported_lists).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::Asset => {
                active_slot!(assets, imported_assets).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::FontAsset => {
                active_slot!(font_assets, imported_font_assets).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::Artboard => {
                active_slot!(artboards, imported_artboards).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::Trigger => {
                active_slot!(triggers, imported_triggers).cell.clone()
            }
            RuntimeOwnedViewModelValueKind::ViewModel => match self.endpoint.value() {
                RuntimeViewModelPointer::OwnedGenerated { .. } => {
                    self.children.get(occurrence.slot_index)?.endpoint.cell()
                }
                RuntimeViewModelPointer::Imported { object_id } => self
                    .imported_children
                    .get(&object_id)?
                    .get(occurrence.slot_index)?
                    .endpoint
                    .cell(),
                _ => return None,
            },
        })
    }

    fn active_scalar_cell_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeViewModelCell> {
        if property_path.is_empty() {
            return None;
        }
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .cell_by_property_path(property_path);
        }
        if property_path.len() == 1 {
            return self.active_scalar_cell_by_property_index(property_path[0]);
        }
        let (property_index, rest) = property_path.split_first()?;
        self.active_children()?
            .iter()
            .find(|child| child.property_index == *property_index)?
            .active_scalar_cell_by_property_path(rest)
    }

    fn active_scalar_cell_by_scoped_property_path(
        &self,
        context_path: &[usize],
        property_path: &[usize],
    ) -> Option<RuntimeViewModelCell> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .cell_by_relative_scoped_property_path(context_path, property_path);
        }
        if context_path.is_empty() {
            return self.active_scalar_cell_by_property_path(property_path);
        }
        let (property_index, rest) = context_path.split_first()?;
        self.active_children()?
            .iter()
            .find(|child| child.property_index == *property_index)?
            .active_scalar_cell_by_scoped_property_path(rest, property_path)
    }

    fn active_list_handle_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeOwnedViewModelListHandle> {
        if property_path.is_empty() {
            return None;
        }
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .list_handle_by_property_path(property_path);
        }
        if property_path.len() == 1 {
            let list = self.active_list_by_property_index(property_path[0])?;
            return Some(RuntimeOwnedViewModelListHandle {
                value: Rc::clone(&list.value),
                cell: list.cell.clone(),
            });
        }
        let (property_index, rest) = property_path.split_first()?;
        self.active_children()?
            .iter()
            .find(|child| child.property_index == *property_index)?
            .active_list_handle_by_property_path(rest)
    }

    fn active_structural_source_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeOwnedViewModelStructuralSource> {
        if property_path.is_empty() {
            return None;
        }
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .structural_source_by_property_path(property_path);
        }
        if property_path.len() == 1 {
            let property_index = property_path[0];
            if let Some(list) = self.active_list_by_property_index(property_index) {
                return Some(RuntimeOwnedViewModelStructuralSource::List(
                    RuntimeOwnedViewModelListHandle {
                        value: Rc::clone(&list.value),
                        cell: list.cell.clone(),
                    },
                ));
            }
            return self
                .active_children()?
                .iter()
                .find(|child| child.property_index == property_index)
                .map(|child| child.endpoint.retained_source());
        }
        let (property_index, rest) = property_path.split_first()?;
        self.active_children()?
            .iter()
            .find(|child| child.property_index == *property_index)?
            .active_structural_source_by_property_path(rest)
    }

    fn active_view_model_value_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeViewModelPointer> {
        if property_path.is_empty() {
            return None;
        }
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .view_model_value_by_property_path(property_path);
        }
        let (property_index, rest) = property_path.split_first()?;
        let child = self
            .active_children()?
            .iter()
            .find(|child| child.property_index == *property_index)?;
        if rest.is_empty() {
            Some(child.endpoint.value())
        } else {
            child.active_view_model_value_by_property_path(rest)
        }
    }

    fn active_view_model_index_by_property_path(&self, property_path: &[usize]) -> Option<usize> {
        if property_path.is_empty() {
            return self.referenced_view_model_index;
        }
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .view_model_index_by_property_path(property_path);
        }
        let (property_index, rest) = property_path.split_first()?;
        let child = self
            .active_children()?
            .iter()
            .find(|child| child.property_index == *property_index)?;
        if rest.is_empty() {
            match child.endpoint.value() {
                RuntimeViewModelPointer::OwnedGenerated { .. }
                | RuntimeViewModelPointer::Imported { .. } => child.referenced_view_model_index,
                _ => None,
            }
        } else {
            child.active_view_model_index_by_property_path(rest)
        }
    }

    fn active_nested_instance_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeOwnedViewModelInstance> {
        if let Some(linked) = self.endpoint.linked_instance() {
            if property_path.is_empty() {
                return Some(linked.try_borrow().ok()?.clone());
            }
            return linked
                .try_borrow()
                .ok()?
                .nested_instance_by_property_path(property_path);
        }
        if property_path.is_empty() {
            return self.materialize_active_instance();
        }
        let (property_index, rest) = property_path.split_first()?;
        self.active_children()?
            .iter()
            .find(|child| child.property_index == *property_index)?
            .active_nested_instance_by_property_path(rest)
    }

    fn list_item_count_by_property_index(&self, property_index: usize) -> Option<usize> {
        self.lists
            .iter()
            .find(|list| list.property_index == property_index)
            .map(|list| list.value.borrow().item_count)
    }

    fn active_list_item_count_by_property_index(&self, property_index: usize) -> Option<usize> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .list_item_count_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.list_item_count_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_lists
                .get(&object_id)
                .and_then(|lists| {
                    lists
                        .iter()
                        .find(|list| list.property_index == property_index)
                })
                .map(|list| list.value.borrow().item_count),
            _ => None,
        }
    }

    fn active_list_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<&RuntimeOwnedViewModelList> {
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => self
                .lists
                .iter()
                .find(|list| list.property_index == property_index),
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_lists
                .get(&object_id)?
                .iter()
                .find(|list| list.property_index == property_index),
            _ => None,
        }
    }

    fn materialize_active_instance(&self) -> Option<RuntimeOwnedViewModelInstance> {
        let mut instance = self.materialize_active_instance_unbound()?;
        instance.bind_structural_ownership();
        Some(instance)
    }

    fn materialize_active_instance_unbound(&self) -> Option<RuntimeOwnedViewModelInstance> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return Some(linked.try_borrow().ok()?.clone());
        }
        let view_model_index = self.referenced_view_model_index?;
        macro_rules! active_values {
            ($owned:expr, $imported:expr) => {{
                match self.endpoint.value() {
                    RuntimeViewModelPointer::OwnedGenerated { .. } => $owned.clone(),
                    RuntimeViewModelPointer::Imported { object_id } => {
                        $imported.get(&object_id).cloned().unwrap_or_default()
                    }
                    _ => Vec::new(),
                }
            }};
        }
        let parent_relay = RuntimeOwnedViewModelParentRelay::new();
        let numbers = active_values!(&self.numbers, self.imported_numbers);
        let booleans = active_values!(&self.booleans, self.imported_booleans);
        let strings = active_values!(&self.strings, self.imported_strings);
        let colors = active_values!(&self.colors, self.imported_colors);
        let enums = active_values!(&self.enums, self.imported_enums);
        let symbol_list_indices = active_values!(
            &self.symbol_list_indices,
            self.imported_symbol_list_indices
        );
        let lists = active_values!(&self.lists, self.imported_lists);
        let assets = active_values!(&self.assets, self.imported_assets);
        let font_assets = active_values!(&self.font_assets, self.imported_font_assets);
        let artboards = active_values!(&self.artboards, self.imported_artboards);
        let triggers = active_values!(&self.triggers, self.imported_triggers);
        let view_models = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => self.children.clone(),
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_children
                .get(&object_id)
                .cloned()
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let value_order = self.active_value_order().to_vec();
        let name = match self.endpoint.value() {
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_instance_names
                .get(&object_id)
                .cloned()
                .unwrap_or_default(),
            _ => String::new(),
        };
        let item_index_symbol_slot = symbol_list_indices.len().checked_sub(1);
        let mut instance = RuntimeOwnedViewModelInstance {
            view_model_index,
            name,
            instance_identity: RuntimeOwnedViewModelInstance::next_instance_identity(),
            allocation_identity: RuntimeOwnedViewModelInstance::next_allocation_identity(),
            parent_relay,
            property_names: self.property_names.clone(),
            numbers,
            booleans,
            strings,
            colors,
            enums,
            symbol_list_indices,
            lists,
            assets,
            font_assets,
            artboards,
            triggers,
            view_models,
            value_order,
            item_index_symbol_slot,
        };
        instance.detach_list_storage_unbound();
        Some(instance)
    }

    fn asset_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.assets
            .iter()
            .find(|asset| asset.property_index == property_index)
            .map(|asset| asset.value())
    }

    fn active_asset_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .asset_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.asset_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_assets
                .get(&object_id)
                .and_then(|assets| {
                    assets
                        .iter()
                        .find(|asset| asset.property_index == property_index)
                })
                .map(|asset| asset.value()),
            _ => None,
        }
    }

    fn active_runtime_image_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<RuntimeViewModelImage> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .runtime_image_by_property_index(property_index);
        }
        let assets = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &self.assets,
            RuntimeViewModelPointer::Imported { object_id } => self.imported_assets.get(&object_id)?,
            _ => return None,
        };
        assets
            .iter()
            .find(|asset| asset.property_index == property_index)?
            .runtime_state
            .borrow()
            .live_image
            .clone()
    }

    fn active_runtime_image_by_property_path(
        &self,
        property_path: &[usize],
    ) -> Option<RuntimeViewModelImage> {
        if property_path.len() == 1 {
            return self.active_runtime_image_by_property_index(property_path[0]);
        }
        let (view_model_index, rest) = property_path.split_first()?;
        self.active_children()?
            .iter()
            .find(|view_model| view_model.property_index == *view_model_index)?
            .active_runtime_image_by_property_path(rest)
    }

    fn set_active_runtime_image_by_property_index(
        &mut self,
        property_index: usize,
        image: Option<RuntimeViewModelImage>,
    ) -> bool {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow_mut()
                .ok()
                .is_some_and(|mut linked| {
                    linked.set_runtime_image_by_property_index(property_index, image)
                });
        }
        let RuntimeViewModelPointer::OwnedGenerated { .. } = self.endpoint.value() else {
            return false;
        };
        let Some(asset) = self
            .assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        let same = match (&asset.runtime_state.borrow().live_image, &image) {
            (Some(current), Some(next)) => current.ptr_eq(next),
            (None, None) => true,
            _ => false,
        };
        if same {
            return false;
        }
        asset.runtime_state.borrow_mut().live_image = image;
        if !asset.set_value(u64::from(u32::MAX)) {
            asset.cell.notify_bindings_value_changed();
        }
        true
    }

    fn font_asset_value_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<RuntimeFontAssetValue> {
        self.font_assets
            .iter()
            .find(|asset| asset.property_index == property_index)
            .map(RuntimeOwnedViewModelFontAsset::value)
    }

    fn active_font_asset_value_by_property_index(
        &self,
        property_index: usize,
    ) -> Option<RuntimeFontAssetValue> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .font_asset_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.font_asset_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_font_assets
                .get(&object_id)
                .and_then(|assets| {
                    assets
                        .iter()
                        .find(|asset| asset.property_index == property_index)
                })
                .map(RuntimeOwnedViewModelFontAsset::value),
            _ => None,
        }
    }

    fn artboard_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.artboards
            .iter()
            .find(|artboard| artboard.property_index == property_index)
            .map(|artboard| artboard.value())
    }

    fn active_artboard_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .artboard_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.artboard_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_artboards
                .get(&object_id)
                .and_then(|artboards| {
                    artboards
                        .iter()
                        .find(|artboard| artboard.property_index == property_index)
                })
                .map(|artboard| artboard.value()),
            _ => None,
        }
    }

    pub(crate) fn trigger_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        self.triggers
            .iter()
            .find(|trigger| trigger.property_index == property_index)
            .map(|trigger| trigger.value())
    }

    fn active_trigger_value_by_property_index(&self, property_index: usize) -> Option<u64> {
        if let Some(linked) = self.endpoint.linked_instance() {
            return linked
                .try_borrow()
                .ok()?
                .trigger_value_by_property_index(property_index);
        }
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                self.trigger_value_by_property_index(property_index)
            }
            RuntimeViewModelPointer::Imported { object_id } => self
                .imported_triggers
                .get(&object_id)
                .and_then(|triggers| {
                    triggers
                        .iter()
                        .find(|trigger| trigger.property_index == property_index)
                })
                .map(|trigger| trigger.value()),
            _ => None,
        }
    }

    define_active_view_model_path_reader!(
        active_number_value_by_property_path,
        number_value_by_property_path,
        active_number_value_by_property_index,
        f32
    );
    define_active_view_model_path_reader!(
        active_boolean_value_by_property_path,
        boolean_value_by_property_path,
        active_boolean_value_by_property_index,
        bool
    );
    define_active_view_model_path_reader!(
        active_string_value_by_property_path,
        string_value_by_property_path,
        active_string_value_by_property_index,
        Arc<[u8]>
    );
    define_active_view_model_path_reader!(
        active_color_value_by_property_path,
        color_value_by_property_path,
        active_color_value_by_property_index,
        u32
    );
    define_active_view_model_path_reader!(
        active_enum_value_by_property_path,
        enum_value_by_property_path,
        active_enum_value_by_property_index,
        u64
    );
    define_active_view_model_path_reader!(
        active_symbol_list_index_value_by_property_path,
        symbol_list_index_value_by_property_path,
        active_symbol_list_index_value_by_property_index,
        u64
    );
    define_active_view_model_path_reader!(
        active_list_item_count_by_property_path,
        list_item_count_by_property_path,
        active_list_item_count_by_property_index,
        usize
    );
    define_active_view_model_path_reader!(
        active_asset_value_by_property_path,
        asset_value_by_property_path,
        active_asset_value_by_property_index,
        u64
    );
    define_active_view_model_path_reader!(
        active_font_asset_value_by_property_path,
        font_asset_value_by_property_path,
        active_font_asset_value_by_property_index,
        RuntimeFontAssetValue
    );
    define_active_view_model_path_reader!(
        active_artboard_value_by_property_path,
        artboard_value_by_property_path,
        active_artboard_value_by_property_index,
        u64
    );
    define_active_view_model_path_reader!(
        active_trigger_value_by_property_path,
        trigger_value_by_property_path,
        active_trigger_value_by_property_index,
        u64
    );

    /// Mirrors the recursive portion of C++
    /// `ViewModelInstanceViewModel::advanced()` for the currently selected
    /// nested instance. Shared list children are returned to the caller so it
    /// can recurse after releasing this instance's mutable borrow.
    fn advance_script_frame(
        &mut self,
        shared_children: &mut Vec<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
    ) -> bool {
        if let Some(linked) = self.endpoint.linked_instance() {
            shared_children.push(linked);
            return false;
        }
        let order = self.active_value_order().to_vec();
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                let mut changed = false;
                for occurrence in order {
                    match occurrence.kind {
                        RuntimeOwnedViewModelValueKind::Trigger => {
                            if let Some(trigger) = self.triggers.get_mut(occurrence.slot_index) {
                                changed |=
                                    reset_runtime_owned_triggers(std::slice::from_mut(trigger));
                            }
                        }
                        RuntimeOwnedViewModelValueKind::List => {
                            if let Some(list) = self.lists.get(occurrence.slot_index) {
                                collect_runtime_owned_list_children(
                                    std::slice::from_ref(list),
                                    shared_children,
                                );
                            }
                        }
                        RuntimeOwnedViewModelValueKind::ViewModel => {
                            if let Some(child) = self.children.get_mut(occurrence.slot_index) {
                                changed |= child.advance_script_frame(shared_children);
                            }
                        }
                        _ => {}
                    }
                }
                changed
            }
            RuntimeViewModelPointer::Imported { object_id } => {
                let mut changed = false;
                for occurrence in order {
                    match occurrence.kind {
                        RuntimeOwnedViewModelValueKind::Trigger => {
                            if let Some(trigger) = self
                                .imported_triggers
                                .get_mut(&object_id)
                                .and_then(|values| values.get_mut(occurrence.slot_index))
                            {
                                changed |=
                                    reset_runtime_owned_triggers(std::slice::from_mut(trigger));
                            }
                        }
                        RuntimeOwnedViewModelValueKind::List => {
                            if let Some(list) = self
                                .imported_lists
                                .get(&object_id)
                                .and_then(|values| values.get(occurrence.slot_index))
                            {
                                collect_runtime_owned_list_children(
                                    std::slice::from_ref(list),
                                    shared_children,
                                );
                            }
                        }
                        RuntimeOwnedViewModelValueKind::ViewModel => {
                            if let Some(child) = self
                                .imported_children
                                .get_mut(&object_id)
                                .and_then(|values| values.get_mut(occurrence.slot_index))
                            {
                                changed |= child.advance_script_frame(shared_children);
                            }
                        }
                        _ => {}
                    }
                }
                changed
            }
            RuntimeViewModelPointer::Null
            | RuntimeViewModelPointer::DataContextRoot
            | RuntimeViewModelPointer::Retained { .. } => false,
        }
    }

    fn advanced_data_context(&mut self) {
        if let Some(linked) = self.endpoint.linked_instance() {
            linked.borrow_mut().advanced_data_context();
            return;
        }
        let order = self.active_value_order().to_vec();
        match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => {
                for occurrence in order {
                    match occurrence.kind {
                        RuntimeOwnedViewModelValueKind::Trigger => {
                            if let Some(trigger) = self.triggers.get(occurrence.slot_index) {
                                trigger.advanced();
                            }
                        }
                        RuntimeOwnedViewModelValueKind::List => {
                            if let Some(list) = self.lists.get(occurrence.slot_index) {
                                advance_runtime_owned_list_children(std::slice::from_ref(list));
                            }
                        }
                        RuntimeOwnedViewModelValueKind::ViewModel => {
                            if let Some(child) = self.children.get_mut(occurrence.slot_index) {
                                child.advanced_data_context();
                            }
                        }
                        _ => {}
                    }
                }
            }
            RuntimeViewModelPointer::Imported { object_id } => {
                for occurrence in order {
                    match occurrence.kind {
                        RuntimeOwnedViewModelValueKind::Trigger => {
                            if let Some(trigger) = self
                                .imported_triggers
                                .get(&object_id)
                                .and_then(|values| values.get(occurrence.slot_index))
                            {
                                trigger.advanced();
                            }
                        }
                        RuntimeOwnedViewModelValueKind::List => {
                            if let Some(list) = self
                                .imported_lists
                                .get(&object_id)
                                .and_then(|values| values.get(occurrence.slot_index))
                            {
                                advance_runtime_owned_list_children(std::slice::from_ref(list));
                            }
                        }
                        RuntimeOwnedViewModelValueKind::ViewModel => {
                            if let Some(child) = self
                                .imported_children
                                .get_mut(&object_id)
                                .and_then(|values| values.get_mut(occurrence.slot_index))
                            {
                                child.advanced_data_context();
                            }
                        }
                        _ => {}
                    }
                }
            }
            RuntimeViewModelPointer::Null
            | RuntimeViewModelPointer::DataContextRoot
            | RuntimeViewModelPointer::Retained { .. } => {}
        }
    }

    fn set_number_by_property_name(&mut self, property_name: &str, value: f32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_number_by_property_index(property_index, value)
    }

    fn set_number_by_property_index(&mut self, property_index: usize, value: f32) -> bool {
        let Some(number) = self
            .numbers
            .iter_mut()
            .find(|number| number.property_index == property_index)
        else {
            return false;
        };
        if !number.set_value(value) {
            return false;
        }
        true
    }

    fn set_boolean_by_property_name(&mut self, property_name: &str, value: bool) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_boolean_by_property_index(property_index, value)
    }

    fn set_boolean_by_property_index(&mut self, property_index: usize, value: bool) -> bool {
        let Some(boolean) = self
            .booleans
            .iter_mut()
            .find(|boolean| boolean.property_index == property_index)
        else {
            return false;
        };
        if !boolean.set_value(value) {
            return false;
        }
        true
    }

    fn set_string_by_property_name(&mut self, property_name: &str, value: &[u8]) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_string_by_property_index(property_index, value)
    }

    fn set_string_by_property_index(&mut self, property_index: usize, value: &[u8]) -> bool {
        let Some(string) = self
            .strings
            .iter_mut()
            .find(|string| string.property_index == property_index)
        else {
            return false;
        };
        if !string.set_value(value) {
            return false;
        }
        true
    }

    fn set_color_by_property_name(&mut self, property_name: &str, value: u32) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_color_by_property_index(property_index, value)
    }

    fn set_color_by_property_index(&mut self, property_index: usize, value: u32) -> bool {
        let Some(color) = self
            .colors
            .iter_mut()
            .find(|color| color.property_index == property_index)
        else {
            return false;
        };
        if !color.set_value(value) {
            return false;
        }
        true
    }

    fn set_enum_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_enum_by_property_index(property_index, value)
    }

    fn set_enum_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(enum_value) = self
            .enums
            .iter_mut()
            .find(|enum_value| enum_value.property_index == property_index)
        else {
            return false;
        };
        if !enum_value.set_value(value) {
            return false;
        }
        true
    }

    fn set_symbol_list_index_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_symbol_list_index_by_property_index(property_index, value)
    }

    fn set_symbol_list_index_by_property_index(
        &mut self,
        property_index: usize,
        value: u64,
    ) -> bool {
        let Some(symbol_list_index) = self
            .symbol_list_indices
            .iter_mut()
            .find(|symbol_list_index| symbol_list_index.property_index == property_index)
        else {
            return false;
        };
        if !symbol_list_index.set_value(value) {
            return false;
        }
        true
    }

    fn set_list_item_count_by_property_name(
        &mut self,
        property_name: &str,
        item_count: usize,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_list_item_count_by_property_index(property_index, item_count)
    }

    fn set_list_item_count_by_property_index(
        &mut self,
        property_index: usize,
        item_count: usize,
    ) -> bool {
        let Some(list) = self
            .lists
            .iter_mut()
            .find(|list| list.property_index == property_index)
        else {
            return false;
        };
        let mut value = list.value.borrow_mut();
        if !value.set_item_count(item_count) {
            return false;
        }
        drop(value);
        list.cell.notify_bindings_value_changed();
        true
    }

    fn set_asset_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_asset_by_property_index(property_index, value)
    }

    fn set_asset_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(asset) = self
            .assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        if !asset.set_value(value) {
            return false;
        }
        true
    }

    fn set_font_asset_index_by_property_name(
        &mut self,
        property_name: &str,
        file_asset_index: u64,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_font_asset_index_by_property_index(property_index, file_asset_index)
    }

    fn set_font_asset_index_by_property_index(
        &mut self,
        property_index: usize,
        file_asset_index: u64,
    ) -> bool {
        let Some(asset) = self
            .font_assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        asset.set_file_asset_index(file_asset_index)
    }

    fn set_live_font_bytes_by_property_name(
        &mut self,
        property_name: &str,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_live_font_bytes_by_property_index(property_index, font_bytes)
    }

    fn set_live_font_bytes_by_property_index(
        &mut self,
        property_index: usize,
        font_bytes: Option<Arc<[u8]>>,
    ) -> bool {
        let Some(asset) = self
            .font_assets
            .iter_mut()
            .find(|asset| asset.property_index == property_index)
        else {
            return false;
        };
        asset.set_live_font_bytes(font_bytes)
    }

    fn set_artboard_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_artboard_by_property_index(property_index, value)
    }

    fn set_artboard_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(artboard) = self
            .artboards
            .iter_mut()
            .find(|artboard| artboard.property_index == property_index)
        else {
            return false;
        };
        if !artboard.set_value(value) {
            return false;
        }
        true
    }

    fn set_trigger_by_property_name(&mut self, property_name: &str, value: u64) -> bool {
        let Some(property_index) = self.property_index_by_name(property_name) else {
            return false;
        };
        self.set_trigger_by_property_index(property_index, value)
    }

    fn set_trigger_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let Some(trigger) = self
            .triggers
            .iter_mut()
            .find(|trigger| trigger.property_index == property_index)
        else {
            return false;
        };
        if !trigger.set_value(value) {
            return false;
        }
        true
    }

    fn sync_number_by_property_index(&mut self, property_index: usize, value: f32) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.numbers,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_numbers.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }

    fn sync_boolean_by_property_index(&mut self, property_index: usize, value: bool) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.booleans,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_booleans.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }

    fn sync_string_by_property_index(&mut self, property_index: usize, value: &[u8]) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.strings,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_strings.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }

    fn sync_color_by_property_index(&mut self, property_index: usize, value: u32) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.colors,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_colors.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }

    fn sync_enum_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.enums,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_enums.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }

    fn sync_symbol_list_index_by_property_index(
        &mut self,
        property_index: usize,
        value: u64,
    ) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.symbol_list_indices,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_symbol_list_indices.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }

    fn sync_asset_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.assets,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_assets.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }

    fn sync_font_asset_index_by_property_index(
        &mut self,
        property_index: usize,
        file_asset_index: u64,
    ) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.font_assets,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_font_assets.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        current.set_file_asset_index(file_asset_index)
    }

    fn apply_font_asset_data_bind_value_by_property_index(
        &mut self,
        property_index: usize,
        value: &RuntimeFontAssetValue,
    ) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.font_assets,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_font_assets.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        current.apply_data_bind_value(value)
    }

    fn sync_artboard_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.artboards,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_artboards.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }

    fn sync_trigger_by_property_index(&mut self, property_index: usize, value: u64) -> bool {
        let values = match self.endpoint.value() {
            RuntimeViewModelPointer::OwnedGenerated { .. } => &mut self.triggers,
            RuntimeViewModelPointer::Imported { object_id } => {
                let Some(values) = self.imported_triggers.get_mut(&object_id) else {
                    return false;
                };
                values
            }
            _ => return false,
        };
        let Some(current) = values
            .iter_mut()
            .find(|current| current.property_index == property_index)
        else {
            return false;
        };
        if !current.set_value(value) {
            return false;
        }
        true
    }
}

pub(crate) fn runtime_view_model_reference_index_for_property_path(
    file: &RuntimeFile,
    property_path: &[u32],
) -> Option<usize> {
    let mut current_view_model_index = usize::try_from(*property_path.first()?).ok()?;
    let property_indices = property_path.get(1..)?;
    if property_indices.is_empty() {
        return None;
    }

    for (segment_index, property_index) in property_indices.iter().enumerate() {
        let view_model = file.view_model(current_view_model_index)?;
        let property = view_model
            .properties
            .into_iter()
            .nth(usize::try_from(*property_index).ok()?)?;
        if property.type_name != "ViewModelPropertyViewModel" {
            return None;
        }
        let referenced_view_model_index =
            usize::try_from(property.uint_property("viewModelReferenceId")?).ok()?;
        if segment_index + 1 == property_indices.len() {
            return Some(referenced_view_model_index);
        }
        current_view_model_index = referenced_view_model_index;
    }

    None
}

fn runtime_view_model_view_model_property_path_for_names(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &[&str],
) -> Option<Vec<u32>> {
    if property_path.is_empty() {
        return None;
    }

    let mut current_view_model_index = view_model_index;
    let mut path = vec![u32::try_from(view_model_index).ok()?];
    for (segment_index, property_name) in property_path.iter().enumerate() {
        if property_name.is_empty() {
            return None;
        }
        let is_last = segment_index + 1 == property_path.len();
        let view_model = file.view_model(current_view_model_index)?;
        let (property_index, property) =
            view_model
                .properties
                .into_iter()
                .enumerate()
                .find(|(_, property)| {
                    property.type_name == "ViewModelPropertyViewModel"
                        && property.string_property("name") == Some(*property_name)
                })?;
        path.push(u32::try_from(property_index).ok()?);

        if !is_last {
            current_view_model_index = property.uint_property("viewModelReferenceId").and_then(
                |view_model_reference_id| usize::try_from(view_model_reference_id).ok(),
            )?;
        }
    }
    Some(path)
}

pub(crate) fn runtime_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    view_model_index: usize,
    property_path: &str,
) -> Option<Vec<u32>> {
    let property_path = property_path.split('/').collect::<Vec<_>>();
    runtime_view_model_view_model_property_path_for_names(file, view_model_index, &property_path)
}

pub(crate) fn runtime_default_view_model_view_model_property_path_for_name(
    file: &RuntimeFile,
    property_name: &str,
) -> Option<Vec<u32>> {
    runtime_view_model_view_model_property_path_for_names(file, 0, &[property_name])
}

pub(crate) fn runtime_default_view_model_view_model_property_path_for_name_path(
    file: &RuntimeFile,
    property_path: &str,
) -> Option<Vec<u32>> {
    runtime_view_model_view_model_property_path_for_name_path(file, 0, property_path)
}

fn runtime_owned_view_model_view_model_children(
    file: &RuntimeFile,
    view_model_index: usize,
    parent_path: &[usize],
    ancestor_view_model_indices: &[usize],
) -> Vec<RuntimeOwnedViewModelViewModel> {
    if ancestor_view_model_indices.contains(&view_model_index) {
        return Vec::new();
    }
    if file.view_model(view_model_index).is_none() {
        return Vec::new();
    }
    let mut child_ancestors = ancestor_view_model_indices.to_vec();
    child_ancestors.push(view_model_index);

    runtime_owned_view_model_property_children(
        file,
        view_model_index,
        None,
        parent_path,
        &child_ancestors,
        true,
    )
}

fn runtime_owned_view_model_view_model_children_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
    parent_path: &[usize],
    ancestor_view_model_indices: &[usize],
) -> Vec<RuntimeOwnedViewModelViewModel> {
    if ancestor_view_model_indices.contains(&view_model_index) {
        return Vec::new();
    }
    let mut child_ancestors = ancestor_view_model_indices.to_vec();
    child_ancestors.push(view_model_index);

    runtime_owned_view_model_property_children(
        file,
        view_model_index,
        Some(view_model_instance),
        parent_path,
        &child_ancestors,
        false,
    )
}

fn runtime_owned_view_model_property_children(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: Option<&RuntimeObject>,
    parent_path: &[usize],
    child_ancestors: &[usize],
    use_generated_defaults: bool,
) -> Vec<RuntimeOwnedViewModelViewModel> {
    let Some(view_model) = file.view_model(view_model_index) else {
        return Vec::new();
    };

    let ordered_properties = if let Some(instance) = view_model_instance {
        runtime_owned_view_model_instance_value_objects(file, view_model_index, instance)
            .into_iter()
            .filter_map(|value| {
                if value.type_name != "ViewModelInstanceViewModel" {
                    return None;
                }
                let property_index =
                    usize::try_from(value.uint_property("viewModelPropertyId")?).ok()?;
                let property = *view_model.properties.get(property_index)?;
                Some((property_index, property, Some(value)))
            })
            .collect::<Vec<_>>()
    } else {
        view_model
            .properties
            .into_iter()
            .enumerate()
            .filter(|(_, property)| property.type_name == "ViewModelPropertyViewModel")
            .map(|(property_index, property)| (property_index, property, None))
            .collect()
    };

    ordered_properties
        .into_iter()
        .filter_map(|(property_index, property, authored_value)| {
            let referenced_view_model_index = property
                .uint_property("viewModelReferenceId")
                .and_then(|view_model_reference_id| usize::try_from(view_model_reference_id).ok());
            let referenced_view_model = referenced_view_model_index
                .and_then(|view_model_index| file.view_model(view_model_index));
            let mut path = parent_path.to_vec();
            path.push(property_index);
            let imported_value = authored_value
                .and_then(|value| {
                    let referenced_instance_index =
                        usize::try_from(value.uint_property("propertyValue")?).ok()?;
                    let referenced_instance = referenced_view_model
                        .as_ref()?
                        .instances
                        .get(referenced_instance_index)?;
                    Some(RuntimeViewModelPointer::Imported {
                        object_id: referenced_instance.object.id,
                    })
                })
                .or_else(|| view_model_instance
                .and_then(|view_model_instance| {
                    let path = [
                        u32::try_from(view_model_index).ok()?,
                        u32::try_from(property_index).ok()?,
                    ];
                    file.data_context_view_model_instance_for_instance(view_model_instance, &path)
                })
                .map(|reference| RuntimeViewModelPointer::Imported {
                    object_id: reference.object.id,
                }))
                .or_else(|| {
                    // C++ ViewModelInstanceViewModel::referenceViewModelInstance
                    // reads this serialized index even when the earlier
                    // ArtboardImporter-scoped relationship was unavailable.
                    let view_model_instance = view_model_instance?;
                    let value = file.view_model_instance_value_for_property_id_object(
                        view_model_instance,
                        u32::try_from(property_index).ok()?,
                    )?;
                    if value.type_name != "ViewModelInstanceViewModel" {
                        return None;
                    }
                    let instance_index =
                        usize::try_from(value.uint_property("propertyValue")?).ok()?;
                    let referenced_instance = referenced_view_model
                        .as_ref()?
                        .instances
                        .get(instance_index)?;
                    Some(RuntimeViewModelPointer::Imported {
                        object_id: referenced_instance.object.id,
                    })
                });
            let value = if let Some(value) = imported_value {
                value
            } else if use_generated_defaults && referenced_view_model.is_some() {
                RuntimeViewModelPointer::OwnedGenerated {
                    view_model_index,
                    property_index,
                    path_key: runtime_owned_view_model_path_key(&path),
                }
            } else {
                RuntimeViewModelPointer::Null
            };
            let has_referenced_view_model = referenced_view_model.is_some();
            let view_model_instance_ids = referenced_view_model
                .map(|view_model| {
                    view_model
                        .instances
                        .into_iter()
                        .map(|instance| instance.object.id)
                        .collect()
                })
                .unwrap_or_default();
            let children = if has_referenced_view_model {
                referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_view_model_children(
                            file,
                            view_model_index,
                            &path,
                            &child_ancestors,
                        )
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let imported_children = referenced_view_model_index
                .and_then(|referenced_view_model_index| {
                    file.view_model(referenced_view_model_index)
                        .map(|view_model| (referenced_view_model_index, view_model))
                })
                .map(|(referenced_view_model_index, view_model)| {
                    view_model
                        .instances
                        .into_iter()
                        .map(|instance| {
                            (
                                instance.object.id,
                                runtime_owned_view_model_view_model_children_for_instance(
                                    file,
                                    referenced_view_model_index,
                                    instance.object,
                                    &path,
                                    child_ancestors,
                                ),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some(RuntimeOwnedViewModelViewModel {
                property_index,
                property_name: property
                    .string_property("name")
                    .unwrap_or_default()
                    .to_owned(),
                endpoint: RuntimeOwnedViewModelEndpoint::new(value),
                referenced_view_model_index,
                property_names: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_property_names(file, view_model_index)
                    })
                    .unwrap_or_default(),
                numbers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_numbers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_numbers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_numbers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                booleans: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_booleans(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_booleans: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_booleans(file, view_model_index)
                    })
                    .unwrap_or_default(),
                strings: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_strings(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_strings: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_strings(file, view_model_index)
                    })
                    .unwrap_or_default(),
                colors: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_colors(file, view_model_index))
                    .unwrap_or_default(),
                imported_colors: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_colors(file, view_model_index)
                    })
                    .unwrap_or_default(),
                enums: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_enums(file, view_model_index))
                    .unwrap_or_default(),
                imported_enums: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_enums(file, view_model_index)
                    })
                    .unwrap_or_default(),
                symbol_list_indices: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_symbol_list_indices(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_symbol_list_indices: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_symbol_list_indices(
                            file,
                            view_model_index,
                        )
                    })
                    .unwrap_or_default(),
                lists: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_lists(file, view_model_index))
                    .unwrap_or_default(),
                imported_lists: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_lists(file, view_model_index)
                    })
                    .unwrap_or_default(),
                assets: referenced_view_model_index
                    .map(|view_model_index| runtime_owned_view_model_assets(file, view_model_index))
                    .unwrap_or_default(),
                imported_assets: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_assets(file, view_model_index)
                    })
                    .unwrap_or_default(),
                font_assets: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_font_assets(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_font_assets: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_font_assets(file, view_model_index)
                    })
                    .unwrap_or_default(),
                artboards: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_artboards(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_artboards: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_artboards(file, view_model_index)
                    })
                    .unwrap_or_default(),
                triggers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_triggers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                imported_triggers: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_imported_triggers(file, view_model_index)
                    })
                    .unwrap_or_default(),
                value_order: referenced_view_model_index
                    .map(|view_model_index| {
                        runtime_owned_view_model_value_order(file, view_model_index, None)
                    })
                    .unwrap_or_default(),
                imported_value_order: referenced_view_model_index
                    .and_then(|view_model_index| {
                        file.view_model(view_model_index)
                            .map(|view_model| (view_model_index, view_model))
                    })
                    .map(|(view_model_index, view_model)| {
                        view_model
                            .instances
                            .into_iter()
                            .map(|instance| {
                                (
                                    instance.object.id,
                                    runtime_owned_view_model_value_order(
                                        file,
                                        view_model_index,
                                        Some(instance.object),
                                    ),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                imported_instance_names: referenced_view_model_index
                    .and_then(|view_model_index| file.view_model(view_model_index))
                    .map(|view_model| {
                        view_model
                            .instances
                            .into_iter()
                            .map(|instance| {
                                (
                                    instance.object.id,
                                    instance
                                        .object
                                        .string_property("name")
                                        .unwrap_or_default()
                                        .to_owned(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                view_model_instance_ids,
                children,
                imported_children,
            })
        })
        .collect()
}
