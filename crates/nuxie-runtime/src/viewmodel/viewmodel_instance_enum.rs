// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_enum.cpp`.
// Enum index value identity, mutation, validation result, import, and clone
// behavior.

#[derive(Debug)]
struct RuntimeOwnedViewModelEnum {
    property_index: usize,
    value_count: usize,
    cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelEnum {
    fn new(property_index: usize, value_count: usize, value: u64) -> Self {
        Self {
            property_index,
            value_count,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::Enum(
                owned_scalar_u32_payload(value),
            )),
        }
    }

    fn for_property(
        file: &RuntimeFile,
        view_model_index: usize,
        property_index: usize,
    ) -> Option<Self> {
        let property = file
            .view_model(view_model_index)?
            .properties
            .get(property_index)
            .copied()?;
        if !matches!(
            property.type_name,
            "ViewModelPropertyEnum"
                | "ViewModelPropertyEnumCustom"
                | "ViewModelPropertyEnumSystem"
        ) {
            return None;
        }
        Some(Self::new(
            property_index,
            runtime_view_model_enum_value_count(file, property),
            0,
        ))
    }

    /// Generated `deserialize(propertyValuePropertyKey, ...)` writes the raw
    /// uint32 payload directly. It does not call either validating `value`
    /// overload, so an out-of-range authored value must survive import and
    /// cloning even though runtime `valueIndex()` later projects it to zero.
    fn for_instance_value(
        file: &RuntimeFile,
        view_model_index: usize,
        source: &RuntimeObject,
    ) -> Option<Self> {
        if source.type_name != "ViewModelInstanceEnum" {
            return None;
        }
        let property_index = usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
        let value = Self::for_property(file, view_model_index, property_index)?;
        value.cell.restore_value_silent(RuntimeViewModelCellValue::Enum(
            owned_scalar_u32_payload(source.uint_property("propertyValue")?),
        ));
        Some(value)
    }

    fn value(&self) -> u64 {
        match self.cell.value() {
            RuntimeViewModelCellValue::Enum(value) => u64::from(value),
            _ => unreachable!("owned enum slot holds a non-enum cell"),
        }
    }

    /// Generated `propertyValue(uint32_t)` followed by
    /// `propertyValueChanged()`: equality returns before dirt; a real write
    /// publishes `Bindings` dirt and the retained value-change edge.
    fn set_property_value_cell(cell: &RuntimeViewModelCell, value: u64) -> bool {
        cell.set_value(RuntimeViewModelCellValue::Enum(owned_scalar_u32_payload(
            value,
        )))
    }

    fn set_property_value(&mut self, value: u64) -> bool {
        Self::set_property_value_cell(&self.cell, value)
    }

    /// Both C++ `value(uint32_t)` and `value(std::string)` resolve a valid
    /// index through the owning `ViewModelPropertyEnum` before entering the
    /// generated property setter.
    fn set_value_index(&mut self, index: u64) -> bool {
        let Ok(index_usize) = usize::try_from(index) else {
            return false;
        };
        if index_usize >= self.value_count {
            return false;
        }
        self.set_property_value(index)
    }

    fn set_value_index_cell(cell: &RuntimeViewModelCell, index: usize, value_count: usize) -> bool {
        if index >= value_count {
            return false;
        }
        Self::set_property_value_cell(cell, index as u64)
    }

    /// `applyValue(DataValueInteger*)` deliberately bypasses enum validation
    /// and calls the generated property setter with the integer payload.
    fn apply_value(&mut self, data_value: u64) -> bool {
        self.set_property_value(data_value)
    }
}

impl Clone for RuntimeOwnedViewModelEnum {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value_count, self.value())
    }
}

fn runtime_view_model_enum_value_count(file: &RuntimeFile, property: &RuntimeObject) -> usize {
    file.data_enum_for_view_model_property_object(property)
        .map(|data_enum| data_enum.values.len())
        .unwrap_or(0)
}
