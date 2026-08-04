use crate::records::dense_hash_table::DenseDefault;
use crate::records::item_interface2::ItemInterface2;
use crate::records::item_interface_map2::ItemInterfaceMap2;
use crate::records::item_interface_set2::ItemInterfaceSet2;

impl<K: DenseDefault> ItemInterface2<K, K> for ItemInterfaceSet2<K> {
    fn get_key(item: &K) -> &K {
        item
    }

    fn set_key(item: &mut K, key: K) {
        *item = key;
    }

    fn make_empty() -> K {
        K::dense_default()
    }
}

impl<K: DenseDefault, V: DenseDefault> ItemInterface2<K, (K, V)> for ItemInterfaceMap2<K, V> {
    fn get_key(item: &(K, V)) -> &K {
        &item.0
    }

    fn set_key(item: &mut (K, V), key: K) {
        item.0 = key;
    }

    fn make_empty() -> (K, V) {
        (K::dense_default(), V::dense_default())
    }
}
