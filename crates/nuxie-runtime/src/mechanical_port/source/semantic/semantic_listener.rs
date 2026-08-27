pub trait SemanticListener {
    fn on_semantic_tap(&mut self);
    fn on_semantic_increase(&mut self);
    fn on_semantic_decrease(&mut self);
}
