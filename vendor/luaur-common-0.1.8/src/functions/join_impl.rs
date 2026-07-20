use crate::macros::luau_assert::LUAU_ASSERT;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr::copy_nonoverlapping;

#[allow(non_snake_case)]
pub(crate) fn joinImpl<S>(segments: &Vec<S>, delimiter: &str) -> String
where
    S: AsRef<[u8]>,
{
    if segments.is_empty() {
        return String::new();
    }

    let mut len = (segments.len() - 1) * delimiter.len();
    for sv in segments {
        len += sv.as_ref().len();
    }

    let mut result = Vec::with_capacity(len);
    unsafe {
        result.set_len(len);
        let mut dest = result.as_mut_ptr();

        let mut it = segments.iter();
        if let Some(first) = it.next() {
            let first_bytes = first.as_ref();
            copy_nonoverlapping(first_bytes.as_ptr(), dest, first_bytes.len());
            dest = dest.add(first_bytes.len());

            for segment in it {
                copy_nonoverlapping(delimiter.as_ptr(), dest, delimiter.len());
                dest = dest.add(delimiter.len());

                let segment_bytes = segment.as_ref();
                copy_nonoverlapping(segment_bytes.as_ptr(), dest, segment_bytes.len());
                dest = dest.add(segment_bytes.len());
            }
        }

        LUAU_ASSERT!(dest == result.as_mut_ptr().add(len));

        String::from_utf8_unchecked(result)
    }
}
