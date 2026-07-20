use crate::functions::read_var_int::read_var_int;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
const LBC_TYPE_TAGGED_USERDATA_BASE: u8 = 64;

pub(crate) unsafe fn remap_userdata_types(
    data: *mut core::ffi::c_char,
    size: usize,
    userdata_remapping: *mut u8,
    count: u32,
) {
    let mut offset: usize = 0;

    let type_size = read_var_int(data, size, &mut offset);
    let upval_count = read_var_int(data, size, &mut offset);
    let local_count = read_var_int(data, size, &mut offset);

    if type_size != 0 {
        let types = data.add(offset) as *mut u8;

        // Skip two bytes of function type introduction
        for i in 2..type_size {
            let index =
                (*types.add(i as usize) as i32 - LBC_TYPE_TAGGED_USERDATA_BASE as i32) as u32;

            if index < count {
                *types.add(i as usize) = *userdata_remapping.add(index as usize);
            }
        }

        offset += type_size as usize;
    }

    if upval_count != 0 {
        let types = data.add(offset) as *mut u8;

        for i in 0..upval_count {
            let index =
                (*types.add(i as usize) as i32 - LBC_TYPE_TAGGED_USERDATA_BASE as i32) as u32;

            if index < count {
                *types.add(i as usize) = *userdata_remapping.add(index as usize);
            }
        }

        offset += upval_count as usize;
    }

    if local_count != 0 {
        for _ in 0..local_count {
            let current_byte_ptr = data.add(offset) as *mut u8;
            let index = (*current_byte_ptr as i32 - LBC_TYPE_TAGGED_USERDATA_BASE as i32) as u32;

            if index < count {
                *current_byte_ptr = *userdata_remapping.add(index as usize);
            }

            offset += 2;
            read_var_int(data, size, &mut offset);
            read_var_int(data, size, &mut offset);
        }
    }

    LUAU_ASSERT!(offset == size);
}
