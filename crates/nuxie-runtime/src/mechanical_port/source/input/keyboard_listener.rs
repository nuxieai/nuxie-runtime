pub trait KeyboardListener<K, M> {
    fn key_input(&mut self, key: K, modifiers: M, is_pressed: bool, is_repeat: bool) -> bool;
    fn text_input(&mut self, text: &str) -> bool;
}
