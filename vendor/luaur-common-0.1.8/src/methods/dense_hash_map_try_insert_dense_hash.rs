//! Generated skeleton item. @skeleton-stub
//! Node: `cxx:Method:Luau.Common:Common/include/Luau/DenseHash.h:608:dense_hash_map_try_insert`
//! Source: `Common/include/Luau/DenseHash.h`
//! Graph edges:
//! - declared_by: source_file Common/include/Luau/DenseHash.h
//! - source_includes:
//!   - includes -> source_file Common/include/Luau/HashUtil.h
//!   - includes -> source_file Common/include/Luau/Common.h
//! - incoming:
//!   - declares <- source_file Common/include/Luau/DenseHash.h
//! - outgoing:
//!   - calls -> method DenseHashTable::rehash_if_full (Common/include/Luau/DenseHash.h)
//!   - calls -> method DenseHashTable::insert_unsafe (Common/include/Luau/DenseHash.h)
//!   - type_ref -> record DenseHashMap (Common/include/Luau/DenseHash.h)
//!   - translates_to -> rust_item DenseHashMap::try_insert

// Dead duplicate skeleton node: the canonical method is implemented elsewhere.
pub fn dense_hash_map_try_insert() {
    unreachable!("canonical DenseHashMap::try_insert lives in crates/luau-common/src/records/dense_hash_map.rs; this skeleton node is unused");
}
