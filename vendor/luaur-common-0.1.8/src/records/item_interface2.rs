pub trait ItemInterface2<K, I> {
    fn get_key(item: &I) -> &K;
    fn set_key(item: &mut I, key: K);
    fn make_empty() -> I;
}
