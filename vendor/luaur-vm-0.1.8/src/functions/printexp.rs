pub(crate) unsafe fn printexp(mut buf: *mut core::ffi::c_char, num: i32) -> *mut core::ffi::c_char {
    *buf = b'e' as core::ffi::c_char;
    buf = buf.add(1);

    *buf = (if num < 0 { b'-' } else { b'+' }) as core::ffi::c_char;
    buf = buf.add(1);

    let mut v = if num < 0 { -num } else { num };

    if v >= 100 {
        *buf = (b'0' + (v / 100) as u8) as core::ffi::c_char;
        buf = buf.add(1);
        v %= 100;
    }

    // kDigitTable is a static table of 2-character strings for 00-99
    // It is typically defined in lnumprint.cpp as:
    // static const char kDigitTable[] = "00010203...99";
    let digit_table = b"00010203040506070809101112131415161718192021222324252627282930313233343536373839404142434445464748495051525354555657585960616263646566676869707172737475767778798081828384858687888990919293949596979899";

    let src = digit_table.as_ptr().add((v * 2) as usize);
    core::ptr::copy_nonoverlapping(src, buf as *mut u8, 2);

    buf.add(2)
}
