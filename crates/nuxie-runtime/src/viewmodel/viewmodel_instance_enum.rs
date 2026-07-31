// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_enum.cpp`.
// Enum index value identity, mutation, validation result, and clone behavior.

#[derive(Debug)]
struct RuntimeOwnedViewModelEnum {
    property_index: usize,
    cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelEnum {
    fn new(property_index: usize, value: u64) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::Enum(
                owned_scalar_u32_payload(value),
            )),
        }
    }

    fn value(&self) -> u64 {
        match self.cell.value() {
            RuntimeViewModelCellValue::Enum(value) => u64::from(value),
            _ => unreachable!("owned enum slot holds a non-enum cell"),
        }
    }

    fn set_value(&mut self, value: u64) -> bool {
        if self.value() == value {
            return false;
        }
        self.cell
            .set_value(RuntimeViewModelCellValue::Enum(owned_scalar_u32_payload(
                value,
            )));
        true
    }
}

impl Clone for RuntimeOwnedViewModelEnum {
    fn clone(&self) -> Self {
        Self::new(self.property_index, self.value())
    }
}
