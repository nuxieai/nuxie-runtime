// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_symbol_list_index.cpp`.
// Symbol replacement order, retained uint32 value identity, generated-property
// mutation, raw import, `applyValue`, and clone behavior.

pub(crate) fn set_component_list_item_index(
    _file: &RuntimeFile,
    instance: &mut RuntimeOwnedViewModelInstance,
    index: usize,
) -> bool {
    // `ViewModelInstanceValue::registerSymbol` overwrites the itemIndex
    // symbol as values are registered. Generated instances register in
    // property order; imported instances register in instance-value order.
    // The constructors preserve that winner as the last entry here.
    let Some(property_index) = instance
        .item_index_symbol_slot
        .and_then(|slot| instance.symbol_list_indices.get(slot))
        .map(|symbol_list_index| symbol_list_index.property_index)
    else {
        return false;
    };
    instance.set_symbol_list_index_by_property_index(property_index, index as u64)
}

#[derive(Debug)]
struct RuntimeOwnedViewModelSymbolListIndex {
    property_index: usize,
    cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelSymbolListIndex {
    fn new(property_index: usize, value: u64) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::SymbolListIndex(
                owned_scalar_u32_payload(value),
            )),
        }
    }

    fn for_property(property_index: usize) -> Self {
        Self::new(property_index, 0)
    }

    /// Generated `deserialize(propertyValuePropertyKey, ...)` writes the raw
    /// uint32 payload without calling `propertyValueChanged()`. Construct the
    /// retained cell silently so authored values begin clean just like C++.
    fn for_instance_value(source: &RuntimeObject) -> Option<Self> {
        if source.type_name != "ViewModelInstanceSymbolListIndex" {
            return None;
        }
        let property_index = usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
        let value = Self::for_property(property_index);
        value
            .cell
            .restore_value_silent(RuntimeViewModelCellValue::SymbolListIndex(
                owned_scalar_u32_payload(source.uint_property("propertyValue")?),
            ));
        Some(value)
    }

    fn value(&self) -> u64 {
        match self.cell.value() {
            RuntimeViewModelCellValue::SymbolListIndex(value) => u64::from(value),
            _ => unreachable!("owned symbol-list-index slot holds a mismatched cell"),
        }
    }

    /// Generated `propertyValue(uint32_t)` followed by
    /// `propertyValueChanged()`: project to the C++ uint32 boundary before
    /// equality, then publish `Bindings` dirt and the retained change edge.
    fn set_property_value_cell(cell: &RuntimeViewModelCell, value: u64) -> bool {
        cell.set_value(RuntimeViewModelCellValue::SymbolListIndex(
            owned_scalar_u32_payload(value),
        ))
    }

    fn set_property_value(&mut self, value: u64) -> bool {
        Self::set_property_value_cell(&self.cell, value)
    }

    /// `ViewModelInstanceSymbolListIndex::applyValue(DataValueInteger*)`.
    fn apply_value(&mut self, data_value: u64) -> bool {
        self.set_property_value(data_value)
    }
}

impl Clone for RuntimeOwnedViewModelSymbolListIndex {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value())
    }
}

fn runtime_owned_view_model_symbol_list_indices(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelSymbolListIndex> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertySymbolListIndex")
                        .then_some(RuntimeOwnedViewModelSymbolListIndex::for_property(
                            property_index,
                        ))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_symbol_list_indices_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelSymbolListIndex> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(RuntimeOwnedViewModelSymbolListIndex::for_instance_value)
        .collect()
}

fn runtime_owned_view_model_imported_symbol_list_indices(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelSymbolListIndex>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_symbol_list_indices_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}
