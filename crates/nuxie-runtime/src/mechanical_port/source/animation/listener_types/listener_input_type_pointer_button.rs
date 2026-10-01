use crate::mechanical_port::source::{
    generated::animation::listener_types::listener_input_type_pointer_button_base::ListenerInputTypePointerButtonBase,
    pointer_button::PointerButton,
};

#[derive(Default)]
pub struct ListenerInputTypePointerButton {
    pub base: ListenerInputTypePointerButtonBase,
}

impl ListenerInputTypePointerButton {
    pub fn pointer_button(&self) -> PointerButton {
        PointerButton(i32::from(self.base.pointer_button_value()))
    }
}
