#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct ItemInterfaceSet;

#[allow(non_snake_case)]
impl ItemInterfaceSet {
    #[inline]
    pub fn getKey<Key>(item: &Key) -> &Key {
        item
    }

    #[inline]
    pub fn setKey<Key: Clone>(item: &mut Key, key: &Key) {
        *item = key.clone();
    }

    #[inline]
    pub fn fill<Key: Clone>(data: *mut Key, count: usize, key: &Key) {
        for i in 0..count {
            unsafe {
                data.add(i).write(key.clone());
            }
        }
    }

    #[inline]
    pub fn destroy<Key>(data: *mut Key, count: usize) {
        for i in 0..count {
            unsafe {
                core::ptr::drop_in_place(data.add(i));
            }
        }
    }
}
