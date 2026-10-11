//! Compile-time type-key tables derived from the generated type tree.

use crate::mechanical_port::source::generated::core_type_tree::PARENT_TYPE_KEYS;

/// One entry per type key: whether that type is `ancestor` or derives from it.
pub(crate) type DescendantTable = [bool; PARENT_TYPE_KEYS.len()];

pub(crate) const fn descendant_table(ancestor: u16) -> DescendantTable {
    let mut table = [false; PARENT_TYPE_KEYS.len()];
    let mut key = 0;
    while key < table.len() {
        let mut current = key;
        loop {
            if current == ancestor as usize {
                table[key] = true;
                break;
            }
            let parent = PARENT_TYPE_KEYS[current] as usize;
            if parent == 0 {
                break;
            }
            current = parent;
        }
        key += 1;
    }
    table
}

/// Whether `type_key` is in `table`; keys outside the tree are not.
#[inline]
pub(crate) fn contains(table: &DescendantTable, type_key: u16) -> bool {
    table.get(usize::from(type_key)).copied().unwrap_or(false)
}
