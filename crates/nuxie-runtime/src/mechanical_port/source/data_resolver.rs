pub trait DataResolver {
    fn resolve_name(&self, id: i32) -> &str;
    fn resolve_path(&self, id: i32) -> &[u32];
}
