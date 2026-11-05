use super::text_style::TextStyle;
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt, core::CoreHandle, core_context::CoreContext,
    generated::text::text_style_axis_base::TextStyleAxisBase, status_code::StatusCode,
};

impl std::ops::Deref for TextStyleAxis {
    type Target = TextStyleAxisBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyleAxis {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextStyleAxis {
    pub const TYPE_KEY: u16 = TextStyleAxisBase::TYPE_KEY;
}

#[derive(Default)]
pub struct TextStyleAxis {
    pub base: TextStyleAxisBase,
}

impl TextStyleAxis {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code == StatusCode::Ok {
            let (Some(style), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
                return StatusCode::InvalidObject;
            };
            let added = style
                .with_mut(|style| {
                    style
                        .as_text_style_mut()
                        .map(|style| style.add_variation(this))
                })
                .flatten()
                .is_some();
            if !added {
                return StatusCode::InvalidObject;
            }
        }
        code
    }
    /// The borrowed setters' form: the caller still holds this axis. The
    /// registered setters take `style_shape_changed_occurrence` instead.
    pub fn tag_changed(&mut self) {
        if let Some(style) = self.base.parent_handle() {
            TextStyle::add_dirt_occurrence(&style, ComponentDirt::TEXT_SHAPE, false);
        }
    }
    pub fn axis_value_changed(&mut self) {
        self.tag_changed();
    }
    /// Upstream's `parent()->addDirt(ComponentDirt::TextShape)`:
    /// `Component::addDirt` calls the style's virtual `onDirty`, which
    /// reshapes its text and rebuilds its variable font. Dirty the style's
    /// occurrence so its handler runs; the embedded Component's own
    /// `on_dirty` is empty. This runs after the axis borrow is released,
    /// because a global style's owner updates at once and reads this axis
    /// again (`SharedTextStyle::styleShapeChanged`).
    pub(crate) fn style_shape_changed_occurrence(axis: &CoreHandle) {
        let style = axis
            .with_downcast::<Self, _>(|axis| axis.base.parent_handle())
            .flatten();
        if let Some(style) = style {
            TextStyle::add_dirt_occurrence(&style, ComponentDirt::TEXT_SHAPE, false);
        }
    }
}
