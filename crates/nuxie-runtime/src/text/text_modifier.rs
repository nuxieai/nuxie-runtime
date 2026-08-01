/// Authored-order registration produced by pinned `TextModifier::onAddedDirty`
/// (`src/text/text_modifier.cpp:7-21`). Mutable caches remain on the artboard
/// occurrence; these descriptors contain only imported identity.
#[derive(Debug, Clone)]
enum StaticTextModifier {
    Abstract { local_id: usize, global_id: u32 },
    Variation(StaticTextVariationModifier),
    Target(StaticTextTargetModifier),
    FollowPath(StaticTextFollowPathModifier),
}

impl StaticTextModifier {
    fn local_id(&self) -> usize {
        match self {
            Self::Abstract { local_id, .. } => *local_id,
            Self::Variation(value) => value.local_id,
            Self::Target(value) => value.local_id,
            Self::FollowPath(value) => value.local_id,
        }
    }

    fn global_id(&self) -> u32 {
        match self {
            Self::Abstract { global_id, .. } => *global_id,
            Self::Variation(value) => value.global_id,
            Self::Target(value) => value.global_id,
            Self::FollowPath(value) => value.global_id,
        }
    }
}
