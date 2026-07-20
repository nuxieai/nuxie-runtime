use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::small_vector::SmallVector;

pub fn small_vector_grow<T, const N: usize>(sv: &mut SmallVector<T, N>, new_size: u32) {
    let mut new_size = new_size;

    let max = sv.capacity();
    let proposed = max + (max >> 1);

    if proposed > new_size {
        new_size = proposed;
    } else {
        new_size += 4;
    }

    LUAU_ASSERT!(new_size < 0x40000000);

    unsafe {
        // Allocate uninitialized memory for new elements.
        let layout = core::alloc::Layout::array::<T>(new_size as usize).expect("invalid layout");
        let raw = alloc::alloc::alloc(layout) as *mut T;
        let new_data = raw;

        let count = sv.size() as usize;
        let ptr = sv.as_mut_slice().as_mut_ptr();

        // Move-construct elements into the new allocation.
        core::ptr::copy_nonoverlapping(ptr, new_data, count);

        // Drop old elements.
        for i in 0..count {
            core::ptr::drop_in_place(ptr.add(i));
        }

        // If the old storage was heap-backed, free it.
        if sv.capacity() != N as u32 {
            let old_layout = core::alloc::Layout::array::<T>(max as usize).expect("invalid layout");
            alloc::alloc::dealloc(sv.as_mut_slice().as_mut_ptr() as *mut u8, old_layout);
        }

        // Update metadata using existing API behavior.
        sv.reserve(new_size);
        let _ = new_data;
    }
}

extern crate alloc;
