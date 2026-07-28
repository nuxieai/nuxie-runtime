use crate::ArtboardInstance;

impl ArtboardInstance {
    /// Live TextInput key delegate called by KeyboardListenerGroup.
    ///
    /// The complete editable-text owner is the mapped FL-E `text_input.cpp`
    /// family. Until that owner lands, this retains pinned C++'s no-text
    /// feature result at the correct mutable owner boundary instead of
    /// hardcoding TextInput behavior inside the keyboard listener.
    pub(crate) fn text_input_key_input(
        &mut self,
        _text_input_local_id: usize,
        _key: u32,
        _modifiers: u32,
        _is_pressed: bool,
        _is_repeat: bool,
    ) -> bool {
        false
    }

    /// Live committed-text delegate called by KeyboardListenerGroup.
    ///
    /// Pinned C++ returns true at this boundary even when the text feature is
    /// disabled. FL-E replaces the body with the complete retained TextInput
    /// editing owner without changing the listener-group call shape.
    pub(crate) fn text_input_text_input(
        &mut self,
        _text_input_local_id: usize,
        _text: &str,
    ) -> bool {
        true
    }
}
