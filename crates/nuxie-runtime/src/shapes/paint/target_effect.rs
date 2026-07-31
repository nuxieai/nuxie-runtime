//! TargetEffect owns a proxy PathProvider for every provider occurrence so
//! repeated targeting of one GroupEffect never aliases EffectPath state.

pub(crate) fn update_effect<T: Clone, E>(
    source: &T,
    group_effects: &[E],
    update: impl FnMut(&E, &T) -> Option<T>,
) -> Option<T> {
    super::group_effect::update_effects(source, group_effects, update)
}
