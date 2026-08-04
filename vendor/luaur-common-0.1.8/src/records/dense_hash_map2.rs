//! Public `Luau::DenseHashMap2` record.

use crate::records::dense_hash_table::DenseEqDefault;
use crate::records::dense_hash_table2::DenseHashTable2;
use crate::records::item_interface_map2::ItemInterfaceMap2;
use crate::type_aliases::dense_hash_default::DenseHashDefault;

pub(crate) type MapImpl<K, V, H, E> = DenseHashTable2<K, (K, V), ItemInterfaceMap2<K, V>, H, E>;

#[derive(Clone)]
pub struct DenseHashMap2<K, V, H = DenseHashDefault<K>, E = DenseEqDefault<K>> {
    pub(crate) impl_: MapImpl<K, V, H, E>,
}
