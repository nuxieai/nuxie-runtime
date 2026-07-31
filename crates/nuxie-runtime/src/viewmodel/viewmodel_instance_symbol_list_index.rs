// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_symbol_list_index.cpp`.
// Symbol replacement order and list-index value identity.

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

    fn value(&self) -> u64 {
        match self.cell.value() {
            RuntimeViewModelCellValue::SymbolListIndex(value) => u64::from(value),
            _ => unreachable!("owned symbol-list-index slot holds a mismatched cell"),
        }
    }

    fn set_value(&mut self, value: u64) -> bool {
        if self.value() == value {
            return false;
        }
        self.cell
            .set_value(RuntimeViewModelCellValue::SymbolListIndex(
                owned_scalar_u32_payload(value),
            ));
        true
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
                        .then_some(RuntimeOwnedViewModelSymbolListIndex::new(property_index, 0))
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
        .filter_map(|source| {
            let value = file.view_model_instance_symbol_list_index_value_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelSymbolListIndex::new(
                property_index,
                value,
            ))
        })
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
