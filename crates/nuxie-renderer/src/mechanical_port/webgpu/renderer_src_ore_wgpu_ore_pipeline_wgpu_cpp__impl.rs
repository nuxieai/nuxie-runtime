//! Complete mechanical implementation translation of
//! `renderer/src/ore/wgpu/ore_pipeline_wgpu.cpp`.

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_pipeline_wgpu.cpp");
pub(crate) const SOURCE_INCLUDE_COUNT: usize = 2;
pub(crate) const SOURCE_OUT_OF_LINE_DEFINITION_COUNT: usize = 0;
const _: [(); 106] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_include_only_implementation_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 6);
        assert_eq!(SOURCE_INCLUDE_COUNT, 2);
        assert_eq!(SOURCE_OUT_OF_LINE_DEFINITION_COUNT, 0);
    }
}
