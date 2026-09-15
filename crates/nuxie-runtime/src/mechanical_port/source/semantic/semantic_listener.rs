pub trait SemanticListener: core::fmt::Debug {
    /// Whether this live listener accepts the runtime semantic action code.
    fn supports_semantic_action(&self, _action: u8) -> bool {
        false
    }
    fn on_semantic_tap(&self);
    fn on_semantic_increase(&self);
    fn on_semantic_decrease(&self);
}
