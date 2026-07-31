//! GroupEffect applies its retained children in authored order and forwards
//! invalidation to every TargetEffect proxy before downstream local effects.

pub(crate) fn update_effects<T: Clone, E>(
    source: &T,
    effects: &[E],
    mut update: impl FnMut(&E, &T) -> Option<T>,
) -> Option<T> {
    let mut current = source.clone();
    let mut has_effect_path = false;
    for effect in effects {
        let Some(next) = update(effect, &current) else {
            continue;
        };
        current = next;
        has_effect_path = true;
    }
    has_effect_path.then_some(current)
}
