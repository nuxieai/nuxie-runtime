use crate::records::string_ref::StringRef;

pub fn bytecode_builder_get_string_hash(key: StringRef) -> u32 {
    // Keep in sync with Lua 5.1's original hashing algorithm:
    // https://github.com/lua/lua/blob/v5.1.5/lstrlib.c (luaS_hash for short inputs)
    //
    // We intentionally omit long string processing for simplicity/independence
    // (matching the source logic).
    let str_ptr = key.data;
    let len = key.length;

    let mut h: u32 = len as u32;

    unsafe {
        let bytes = core::slice::from_raw_parts(str_ptr as *const u8, len as usize);
        for i in (0..len).rev() {
            let ch = bytes[i as usize] as u32;
            h ^= (h << 5).wrapping_add(h >> 2).wrapping_add(ch);
        }
    }

    h
}
