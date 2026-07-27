/// Runtime-only fields owned by C++ `CubicWeight`.
///
/// The inherited base translation remains on `RuntimeWeightState`; this
/// derived owner retains only its independent in/out translations, matching
/// `include/rive/bones/cubic_weight.hpp:9-15`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct RuntimeCubicWeightState {
    pub(crate) in_translation: (f32, f32),
    pub(crate) out_translation: (f32, f32),
}
