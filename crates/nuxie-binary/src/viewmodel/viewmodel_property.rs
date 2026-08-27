use crate::{definition_by_type_key, RuntimeObject};
use serde::Serialize;

/// Direct representation of C++ `ViewModelProperty::Direction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[repr(u8)]
pub enum RuntimeViewModelPropertyDirection {
    None = 0,
    Input = 1,
    Output = 2,
    Both = 3,
}

impl RuntimeViewModelPropertyDirection {
    /// `ViewModelProperty::direction`: direction occupies bits 0-1 of
    /// `componentProps`; all other component flags are ignored.
    pub const fn from_component_props(component_props: u64) -> Self {
        match component_props & 0x3 {
            1 => Self::Input,
            2 => Self::Output,
            3 => Self::Both,
            _ => Self::None,
        }
    }

    /// Direct `ViewModelProperty::isInput`.
    pub const fn is_input(self) -> bool {
        matches!(self, Self::Input | Self::Both)
    }

    /// Direct `ViewModelProperty::isOutput`.
    pub const fn is_output(self) -> bool {
        matches!(self, Self::Output | Self::Both)
    }
}

fn is_view_model_property(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key)
        .is_some_and(|definition| definition.is_a("ViewModelProperty"))
}

impl RuntimeObject {
    /// Direct `ViewModelProperty::constName`, with `None` as the Rust-safe
    /// wrong-type/invalid-UTF-8 boundary.
    pub fn view_model_property_const_name(&self) -> Option<&str> {
        is_view_model_property(self).then(|| self.string_property("name"))?
    }

    /// Direct `ViewModelProperty::direction` over the retained generated
    /// `componentProps` value and its generated zero default.
    pub fn view_model_property_direction(&self) -> Option<RuntimeViewModelPropertyDirection> {
        is_view_model_property(self).then(|| {
            RuntimeViewModelPropertyDirection::from_component_props(
                self.uint_property("componentProps").unwrap_or(0),
            )
        })
    }

    /// Direct `ViewModelProperty::isInput`.
    pub fn view_model_property_is_input(&self) -> Option<bool> {
        self.view_model_property_direction()
            .map(RuntimeViewModelPropertyDirection::is_input)
    }

    /// Direct `ViewModelProperty::isOutput`.
    pub fn view_model_property_is_output(&self) -> Option<bool> {
        self.view_model_property_direction()
            .map(RuntimeViewModelPropertyDirection::is_output)
    }
}
