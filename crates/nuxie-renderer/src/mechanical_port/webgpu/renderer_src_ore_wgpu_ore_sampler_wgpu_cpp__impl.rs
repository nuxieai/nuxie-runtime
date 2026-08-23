//! Complete mechanical implementation translation of the intentionally
//! include-only `renderer/src/ore/wgpu/ore_sampler_wgpu.cpp` owner.

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_sampler_wgpu.cpp");
pub(crate) const SOURCE_INCLUDE_COUNT: usize = 1;
pub(crate) const SOURCE_OUT_OF_LINE_DEFINITION_COUNT: usize = 0;
const _: [(); 63] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_only_translation_unit_is_preserved() {
        assert_eq!(PINNED_SOURCE.lines().count(), 5);
        assert!(PINNED_SOURCE.contains("#include \"ore_sampler_wgpu.hpp\""));
        assert_eq!(SOURCE_INCLUDE_COUNT, 1);
        assert_eq!(SOURCE_OUT_OF_LINE_DEFINITION_COUNT, 0);
    }
}
