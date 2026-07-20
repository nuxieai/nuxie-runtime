#[allow(non_snake_case)]
#[macro_export]
macro_rules! CommonHeader {
    () => {
        pub tt: u8,
        pub marked: u8,
        pub memcat: u8,
    };
}

pub use CommonHeader;
