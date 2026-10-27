pub trait KeyboardListener<K, M> {
    fn key_input(&mut self, key: K, modifiers: M, is_pressed: bool, is_repeat: bool) -> bool;
    fn capture_key_input(
        &mut self,
        _key: K,
        _modifiers: M,
        _is_pressed: bool,
        _is_repeat: bool,
    ) -> bool {
        false
    }
    fn text_input(&mut self, text: &str) -> bool;
}
