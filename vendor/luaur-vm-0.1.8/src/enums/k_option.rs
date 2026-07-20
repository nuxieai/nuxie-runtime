#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum KOption {
    Kint,       // signed integers
    Kuint,      // unsigned integers
    Kfloat,     // floating-point numbers
    Kchar,      // fixed-length strings
    Kstring,    // strings with prefixed length
    Kzstr,      // zero-terminated strings
    Kpadding,   // padding
    Kpaddalign, // padding for alignment
    Knop,       // no-op (configuration or spaces)
}

#[allow(non_upper_case_globals)]
pub type k_option = KOption;

#[allow(non_upper_case_globals)]
pub type KOption_alias = KOption;
