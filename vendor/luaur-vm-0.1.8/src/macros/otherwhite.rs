use crate::macros::whitebits::WHITEBITS;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! otherwhite {
    ($g:expr) => {{
        unsafe { (*$g).currentwhite as i32 ^ $crate::macros::whitebits::WHITEBITS }
    }};
}

pub use otherwhite;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! isdead {
    ($g:expr, $v:expr) => {{
        unsafe {
            (((*($v as *const $crate::records::g_cheader::GCheader)).marked as i32
                & ($crate::macros::whitebits::WHITEBITS
                    | $crate::macros::bitmask::bitmask($crate::macros::fixedbit::FIXEDBIT)))
                == ($crate::macros::otherwhite::otherwhite!($g)
                    & $crate::macros::whitebits::WHITEBITS))
        }
    }};
}

pub use isdead;
