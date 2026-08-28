pub trait SemanticListener: core::fmt::Debug {
    fn on_semantic_tap(&self);
    fn on_semantic_increase(&self);
    fn on_semantic_decrease(&self);
}
