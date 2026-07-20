#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct ItemInterfaceMap;

impl ItemInterfaceMap {
    #[inline]
    #[allow(non_snake_case)]
    pub fn getKey<Key, Value>(item: &(Key, Value)) -> &Key {
        &item.0
    }

    #[inline]
    #[allow(non_snake_case)]
    pub fn setKey<Key, Value>(item: &mut (Key, Value), key: Key) {
        item.0 = key;
    }

    /// # Safety
    /// `data` must be valid for `count` elements.
    /// This function performs manual initialization of the memory.
    pub unsafe fn fill<Key: Clone, Value: Default>(
        data: *mut (Key, Value),
        count: usize,
        key: &Key,
    ) {
        for i in 0..count {
            let item_ptr = data.add(i);
            core::ptr::write(core::ptr::addr_of_mut!((*item_ptr).0), key.clone());
            core::ptr::write(core::ptr::addr_of_mut!((*item_ptr).1), Value::default());
        }
    }

    /// # Safety
    /// `data` must be valid for `count` elements and must be initialized.
    pub unsafe fn destroy<Key, Value>(data: *mut (Key, Value), count: usize) {
        for i in 0..count {
            core::ptr::drop_in_place(data.add(i));
        }
    }
}
