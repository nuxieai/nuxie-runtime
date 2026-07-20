#[allow(non_snake_case)]
pub fn printunsignedrev(mut end: *mut core::ffi::c_char, mut num: u64) -> *mut core::ffi::c_char {
    const kDigitTable: [u8; 200] = [
        b'0', b'0', b'0', b'1', b'0', b'2', b'0', b'3', b'0', b'4', b'0', b'5', b'0', b'6', b'0',
        b'7', b'0', b'8', b'0', b'9', b'1', b'0', b'1', b'1', b'1', b'2', b'1', b'3', b'1', b'4',
        b'1', b'5', b'1', b'6', b'1', b'7', b'1', b'8', b'1', b'9', b'2', b'0', b'2', b'1', b'2',
        b'2', b'2', b'3', b'2', b'4', b'2', b'5', b'2', b'6', b'2', b'7', b'2', b'8', b'2', b'9',
        b'3', b'0', b'3', b'1', b'3', b'2', b'3', b'3', b'3', b'4', b'3', b'5', b'3', b'6', b'3',
        b'7', b'3', b'8', b'3', b'9', b'4', b'0', b'4', b'1', b'4', b'2', b'4', b'3', b'4', b'4',
        b'4', b'5', b'4', b'6', b'4', b'7', b'4', b'8', b'4', b'9', b'5', b'0', b'5', b'1', b'5',
        b'2', b'5', b'3', b'5', b'4', b'5', b'5', b'5', b'6', b'5', b'7', b'5', b'8', b'5', b'9',
        b'6', b'0', b'6', b'1', b'6', b'2', b'6', b'3', b'6', b'4', b'6', b'5', b'6', b'6', b'6',
        b'7', b'6', b'8', b'6', b'9', b'7', b'0', b'7', b'1', b'7', b'2', b'7', b'3', b'7', b'4',
        b'7', b'5', b'7', b'6', b'7', b'7', b'7', b'8', b'7', b'9', b'8', b'0', b'8', b'1', b'8',
        b'2', b'8', b'3', b'8', b'4', b'8', b'5', b'8', b'6', b'8', b'7', b'8', b'8', b'8', b'9',
        b'9', b'0', b'9', b'1', b'9', b'2', b'9', b'3', b'9', b'4', b'9', b'5', b'9', b'6', b'9',
        b'7', b'9', b'8', b'9', b'9',
    ];

    while num >= 10000 {
        let tail = (num % 10000) as u32;
        unsafe {
            let src0 = kDigitTable.as_ptr().add((tail / 100) as usize * 2);
            core::ptr::copy_nonoverlapping(src0, (end as *mut u8).sub(4), 2);
            let src1 = kDigitTable.as_ptr().add((tail % 100) as usize * 2);
            core::ptr::copy_nonoverlapping(src1, (end as *mut u8).sub(2), 2);
        }
        num /= 10000;
        unsafe {
            end = end.sub(4);
        }
    }

    let mut rest = num as u32;

    while rest >= 10 {
        unsafe {
            let src = kDigitTable.as_ptr().add((rest % 100) as usize * 2);
            core::ptr::copy_nonoverlapping(src, (end as *mut u8).sub(2), 2);
        }
        rest /= 100;
        unsafe {
            end = end.sub(2);
        }
    }

    if rest > 0 {
        unsafe {
            *end.sub(1) = (b'0' + rest as u8) as core::ffi::c_char;
            end = end.sub(1);
        }
    }

    end
}
