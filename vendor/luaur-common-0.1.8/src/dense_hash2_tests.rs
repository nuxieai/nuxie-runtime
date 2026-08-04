use crate::records::dense_hash_map2::DenseHashMap2;
use crate::records::dense_hash_set2::DenseHashSet2;
use crate::records::dense_hash_table::{DenseEqDefault, DenseHasher};

#[derive(Default)]
struct ConstantHasher;

impl DenseHasher<u64> for ConstantHasher {
    fn hash(&self, _key: &u64) -> usize {
        0
    }
}

#[test]
fn set_grows_and_erases_across_probe_chains() {
    let mut set = DenseHashSet2::<u64>::new(0);

    for key in 0..256 {
        assert_eq!(*set.insert(key), key);
    }
    assert_eq!(set.size(), 256);

    for key in (0..256).step_by(3) {
        set.erase(&key);
    }

    for key in 0..256 {
        assert_eq!(set.contains(&key), key % 3 != 0);
    }
}

#[test]
fn map_try_insert_preserves_existing_value() {
    let mut map = DenseHashMap2::<u64, u64>::new(0);

    let (value, fresh) = map.try_insert(7, 41);
    assert!(fresh);
    assert_eq!(*value, 41);

    let (value, fresh) = map.try_insert(7, 99);
    assert!(!fresh);
    assert_eq!(*value, 41);

    *map.get_or_insert(8) = 13;
    assert_eq!(map.find(&8), Some(&13));
}

#[test]
fn erase_preserves_a_degenerate_linear_probe_cluster() {
    let mut map = DenseHashMap2::<u64, u64, ConstantHasher, DenseEqDefault<u64>>::new(16);

    // Every key starts at bucket zero, forcing Algorithm R to repair the
    // longest possible probe chain after each deletion.
    for key in 0..12 {
        map.try_insert(key, key + 100);
    }

    let mut erased = alloc::vec::Vec::new();
    for key in [0, 5, 11, 6] {
        map.erase(&key);
        erased.push(key);
        for remaining in 0..12 {
            if erased.contains(&remaining) {
                assert_eq!(map.find(&remaining), None);
            } else {
                assert_eq!(map.find(&remaining), Some(&(remaining + 100)));
            }
        }
    }
}
