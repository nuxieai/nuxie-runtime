//! Generic code that is shared between multiple layout algorithms
pub(crate) mod alignment;
pub(crate) mod sizing_keyword;

// nuxie-css: always compiled: `InflowOverflow` is in signatures that exist without `content_size`.
#[cfg_attr(not(feature = "content_size"), allow(dead_code))]
pub(crate) mod scrollable_overflow;
