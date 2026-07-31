//! Ordered effect ownership. Invalidating one effect dirties that effect and
//! every downstream effect on the same retained ShapePaint occurrence.

pub(crate) fn first_dirty_effect(current: Option<usize>, invalidating: usize) -> usize {
    current.map_or(invalidating, |current| current.min(invalidating))
}

pub(crate) fn invalidate_suffix<T>(
    effects: &[T],
    invalidating: usize,
    mut invalidate: impl FnMut(&T),
) {
    for effect in effects.iter().skip(invalidating) {
        invalidate(effect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalidation_keeps_the_clean_prefix() {
        let mut dirty = [false; 4];
        let indices = [0, 1, 2, 3];
        invalidate_suffix(&indices, 2, |index| dirty[*index] = true);
        assert_eq!(dirty, [false, false, true, true]);
        assert_eq!(first_dirty_effect(Some(3), 1), 1);
    }
}
