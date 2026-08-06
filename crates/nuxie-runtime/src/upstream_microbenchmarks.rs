//! Opt-in support seam for pinned upstream microbenchmarks.
//!
//! This doc-hidden function is intentionally narrow. Its permanent maintenance
//! cost is justified by timing the production RawPath measurement boundary
//! without a benchmark-only command conversion.

use nuxie_render_api::RawPath;

use crate::draw::RuntimePathMeasure;

pub const fn c_rand_max() -> f32 {
    #[cfg(windows)]
    return 32_767.0;
    #[cfg(not(windows))]
    return 2_147_483_647.0;
}

pub fn measure_raw_path(path: &RawPath) -> f32 {
    RuntimePathMeasure::from_raw_path(path).length()
}

#[cfg(test)]
mod tests {
    #[test]
    fn c_rand_max_matches_supported_host_abi() {
        #[cfg(windows)]
        assert_eq!(super::c_rand_max(), 32_767.0);
        #[cfg(not(windows))]
        assert_eq!(super::c_rand_max(), 2_147_483_647.0);
    }
}
