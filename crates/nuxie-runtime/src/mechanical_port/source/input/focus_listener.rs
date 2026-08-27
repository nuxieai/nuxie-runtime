pub trait FocusListener {
    fn on_focused(&mut self);
    fn on_blurred(&mut self);
}
