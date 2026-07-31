//! StrokeEffect invalidation is occurrence-local. `invalidateEffectFromLocal`
//! rewinds this effect's retained paths and then invalidates only downstream
//! effects through the parent EffectsContainer.

use crate::ArtboardInstance;

pub(crate) fn invalidate_effect_from_local(
    artboard: &mut ArtboardInstance,
    local_id: usize,
) -> bool {
    artboard.invalidate_runtime_stroke_effect_from_local(local_id)
}
