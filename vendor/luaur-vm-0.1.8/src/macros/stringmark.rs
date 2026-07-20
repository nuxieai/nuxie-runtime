use crate::macros::reset_2_bits::reset2bits;

pub const WHITE0BIT: i32 = 0;
pub const WHITE1BIT: i32 = 1;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! stringmark {
    ($s:expr) => {
        $crate::macros::reset_2_bits::reset2bits!(
            // TString embeds CommonHeader as `hdr`; C++ reads ts->marked directly
            (*$s).hdr.marked,
            $crate::macros::stringmark::WHITE0BIT,
            $crate::macros::stringmark::WHITE1BIT
        )
    };
}

pub use stringmark;
