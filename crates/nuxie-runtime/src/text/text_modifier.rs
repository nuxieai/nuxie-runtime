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
    /// Port `TextModifier::onAddedDirty`: a concrete modifier registers with
    /// its direct `TextModifierGroup` parent in authored child order.
    fn from_group_child(
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        child_local: usize,
    ) -> Result<Option<Self>> {
        Ok(match type_for_local(graph, child_local) {
            Some("TextFollowPathModifier") => Some(Self::FollowPath(
                StaticTextFollowPathModifier::from_graph(runtime, graph, child_local)?,
            )),
            Some("TextVariationModifier") => Some(Self::Variation(
                StaticTextVariationModifier::from_graph(runtime, graph, child_local)?,
            )),
            Some("TextTargetModifier") => Some(Self::Target(StaticTextTargetModifier::from_graph(
                runtime,
                graph,
                child_local,
            )?)),
            Some("TextModifier" | "TextShapeModifier") => Some(Self::Abstract {
                local_id: child_local,
                global_id: global_for_local(graph, child_local)?,
            }),
            _ => None,
        })
    }

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

    fn is_shape_modifier(&self) -> bool {
        matches!(self, Self::Variation(_))
    }

    fn follow_path(&self) -> Option<&StaticTextFollowPathModifier> {
        match self {
            Self::FollowPath(modifier) => Some(modifier),
            _ => None,
        }
    }
}
