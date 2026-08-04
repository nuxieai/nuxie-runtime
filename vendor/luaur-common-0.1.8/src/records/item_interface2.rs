pub trait ItemInterface2<K, I> {
    fn get_key(item: &I) -> &K;
    fn make(key: K) -> I;
}
