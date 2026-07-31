// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_color_runtime.cpp`.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceColorRuntime {
    value: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceColorRuntime {
    fn new(name: impl Into<String>, cell: RuntimeViewModelCell) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::Color,
                cell,
            ),
        }
    }

    pub fn value(&self) -> u32 {
        match self.value.cell().value() {
            RuntimeViewModelCellValue::Color(value) => value,
            _ => unreachable!("color runtime must retain a color cell"),
        }
    }

    pub fn set_value(&self, value: u32) -> bool {
        self.value
            .cell()
            .set_value(RuntimeViewModelCellValue::Color(value))
    }

    pub fn set_rgb(&self, red: u8, green: u8, blue: u8) -> bool {
        let alpha = (self.value() >> 24) as u8;
        self.set_argb(alpha, red, green, blue)
    }

    pub fn set_alpha(&self, alpha: u8) -> bool {
        let color = self.value();
        self.set_argb(
            alpha,
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
        )
    }

    pub fn set_argb(&self, alpha: u8, red: u8, green: u8, blue: u8) -> bool {
        self.set_value(
            (u32::from(alpha) << 24)
                | (u32::from(red) << 16)
                | (u32::from(green) << 8)
                | u32::from(blue),
        )
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
