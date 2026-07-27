use crate::artboard::ArtboardInstance;
use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;

/// Clone-owned counterpart of C++ `DrawTarget::m_Drawable`, `first`, and
/// `last` (`include/rive/draw_target.hpp`).
#[derive(Debug, Clone)]
pub(crate) struct RuntimeDrawTarget {
    local_id: usize,
    drawable_local: Option<usize>,
    first: Option<usize>,
    last: Option<usize>,
}

impl RuntimeDrawTarget {
    pub(crate) fn new(local_id: usize, drawable_local: Option<usize>) -> Self {
        Self {
            local_id,
            drawable_local,
            first: None,
            last: None,
        }
    }

    pub(crate) fn local_id(&self) -> usize {
        self.local_id
    }

    pub(crate) fn drawable_local(&self) -> Option<usize> {
        self.drawable_local
    }

    pub(crate) fn first(&self) -> Option<usize> {
        self.first
    }

    pub(crate) fn last(&self) -> Option<usize> {
        self.last
    }

    pub(crate) fn reset_drawables(&mut self) {
        self.first = None;
        self.last = None;
    }

    /// Append one drawable occurrence to the target-owned group and return
    /// the previous tail for Artboard's linked draw-order splice.
    pub(crate) fn append_drawable(&mut self, drawable: usize) -> Option<usize> {
        let previous = self.last;
        if previous.is_none() {
            self.first = Some(drawable);
        }
        self.last = Some(drawable);
        previous
    }
}

/// Direct generated-property callback counterpart of
/// `DrawTarget::placementValueChanged`.
pub(crate) fn apply_uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> bool {
    if artboard.slot(local_id).and_then(|slot| slot.type_name) != Some("DrawTarget")
        || property_key_for_name("DrawTarget", "placementValue") != Some(property_key)
    {
        return false;
    }
    artboard.add_dirt(0, ComponentDirt::DRAW_ORDER, false)
}
