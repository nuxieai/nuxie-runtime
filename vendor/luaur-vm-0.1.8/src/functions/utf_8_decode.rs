use core::ffi::c_char;

const MAX_UNICODE: u32 = 0x10FFFF;

pub fn utf_8_decode(o: *const c_char, val: *mut i32) -> *const c_char {
    let limits: [u32; 4] = [0xFF, 0x7F, 0x7FF, 0xFFFF];
    let s = o as *const u8;

    unsafe {
        let c = *s as u32;
        let mut res: u32 = 0;

        if c < 0x80 {
            res = c;
        } else {
            let mut count: usize = 0;
            let mut c_val = c;

            while (c_val & 0x40) != 0 {
                count += 1;
                let cc = *s.add(count) as u32;
                if (cc & 0xC0) != 0x80 {
                    return core::ptr::null();
                }
                res = (res << 6) | (cc & 0x3F);
                c_val <<= 1;
            }

            res |= (c_val & 0x7F) << (count * 5);

            if count > 3 || res > MAX_UNICODE || res <= limits[count] {
                return core::ptr::null();
            }

            if (res.wrapping_sub(0xD800)) < 0x800 {
                return core::ptr::null();
            }

            let s_ptr = s.add(count);
            if !val.is_null() {
                *val = res as i32;
            }
            return s_ptr.add(1) as *const c_char;
        }

        if !val.is_null() {
            *val = res as i32;
        }
        s.add(1) as *const c_char
    }
}
