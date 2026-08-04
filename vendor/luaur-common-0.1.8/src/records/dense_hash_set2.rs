//! Public `Luau::DenseHashSet2` record.

use crate::records::dense_hash_table::DenseEqDefault;
use crate::records::dense_hash_table2::DenseHashTable2;
use crate::records::item_interface_set2::ItemInterfaceSet2;
use crate::type_aliases::dense_hash_default::DenseHashDefault;

pub(crate) type SetImpl<K, H, E> = DenseHashTable2<K, K, ItemInterfaceSet2<K>, H, E>;

#[derive(Clone)]
pub struct DenseHashSet2<K, H = DenseHashDefault<K>, E = DenseEqDefault<K>> {
    pub(crate) impl_: SetImpl<K, H, E>,
}
