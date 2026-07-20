use crate::macros::mix::mix;

#[allow(non_snake_case)]
pub fn luaS_hash(mut str: *const core::ffi::c_char, mut len: usize) -> core::ffi::c_uint {
    // Note that this hashing algorithm is replicated in BytecodeBuilder.cpp, BytecodeBuilder::getStringHash
    let mut a: u32 = 0;
    let mut b: u32 = 0;
    let mut h: u32 = len as u32;

    // hash prefix in 12b chunks (using aligned reads) with ARX based hash (LuaJIT v2.1, lookup3)
    // note that we stop at length<32 to maintain compatibility with Lua 5.1
    while len >= 32 {
        // should compile into fast unaligned reads
        let mut block: [u32; 3] = [0; 3];
        unsafe {
            core::ptr::copy_nonoverlapping(str as *const u8, block.as_mut_ptr() as *mut u8, 12);
        }

        a = a.wrapping_add(block[0]);
        b = b.wrapping_add(block[1]);
        h = h.wrapping_add(block[2]);

        mix(14, 11, 25, &mut a, &mut b, &mut h);

        unsafe {
            str = str.add(12);
        }
        len -= 12;
    }

    // original Lua 5.1 hash for compatibility (exact match when len<32)
    for i in (1..=len).rev() {
        let char_val = unsafe { *str.add(i - 1) } as u8;
        h ^= h
            .wrapping_shl(5)
            .wrapping_add(h.wrapping_shr(2))
            .wrapping_add(char_val as u32);
    }

    h
}
