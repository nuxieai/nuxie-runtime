#[allow(non_snake_case)]
pub unsafe fn luai_int2str(buf: *mut core::ffi::c_char, l: i64) -> *mut core::ffi::c_char {
    let mut val: u64 = if l < 0 {
        (!(l as u64)).wrapping_add(1)
    } else {
        l as u64
    };

    let mut num_digits = 1;
    let mut cap: u64 = 10;
    while num_digits < 19 && cap <= val {
        num_digits += 1;
        if let Some(next_cap) = cap.checked_mul(10) {
            cap = next_cap;
        } else {
            break;
        }
    }

    let mut pos = if l < 0 { num_digits } else { num_digits - 1 };
    *buf.add((pos + 1) as usize) = 0;

    loop {
        *buf.add(pos as usize) = (b'0' + (val % 10) as u8) as core::ffi::c_char;
        pos -= 1;
        val /= 10;
        if val == 0 {
            break;
        }
    }

    if l < 0 {
        *buf.add(pos as usize) = b'-' as core::ffi::c_char;
        pos -= 1;
    }

    luaur_common::macros::luau_assert::LUAU_ASSERT!(pos == -1);

    buf.add(if l < 0 {
        (num_digits + 1) as usize
    } else {
        num_digits as usize
    })
}

#[allow(non_snake_case)]
#[inline]
pub unsafe fn luai_int_2_str(buf: *mut core::ffi::c_char, l: i64) -> *mut core::ffi::c_char {
    luai_int2str(buf, l)
}
