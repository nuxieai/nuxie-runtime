use crate::mechanical_port::source::{
    constraints::{
        draggable_constraint::{DraggableConstraint, DraggableProxy},
        scrolling::{
            scroll_bar_constraint_proxy::{ThumbDraggableProxy, TrackDraggableProxy},
            scroll_constraint::ScrollConstraint,
        },
        transform_constraint::TransformConstraint,
    },
    core_context::{CoreContext, StatusCode},
    generated::constraints::scrolling::scroll_bar_constraint_base::ScrollBarConstraintBase,
    layout_component::LayoutComponent,
    math::{mat2d::Mat2D, math_types, transform_components::TransformComponents, vec2d::Vec2D},
    transform_component::TransformComponent,
};

pub struct ScrollBarConstraint {
    pub base: ScrollBarConstraintBase,
    components_a: TransformComponents,
    components_b: TransformComponents,
    // Option is the Rust representation of the resolved non-owning pointer.
    scroll_constraint: Option<*mut ScrollConstraint>,
}

impl ScrollBarConstraint {
    pub fn computed_thumb_width(&self) -> f32 {
        if self.base.auto_size() {
            if let Some(scroll) = self.scroll_constraint {
                return self.track().inner_width() * unsafe { (*scroll).visible_width_ratio() };
            }
        }
        self.thumb().layout_width()
    }
    pub fn computed_thumb_height(&self) -> f32 {
        if self.base.auto_size() {
            if let Some(scroll) = self.scroll_constraint {
                return self.track().inner_height() * unsafe { (*scroll).visible_height_ratio() };
            }
        }
        self.thumb().layout_height()
    }

    pub fn draggables(&mut self) -> Vec<Box<dyn DraggableProxy>> {
        let mut items: Vec<Box<dyn DraggableProxy>> = Vec::new();
        if self.base.parent().is::<LayoutComponent>() {
            let parent = self.base.parent_mut().as_mut::<LayoutComponent>().unwrap();
            items.push(Box::new(ThumbDraggableProxy::new(self, parent.proxy_mut())));
        }
        if self
            .base
            .parent()
            .parent()
            .is_some_and(|parent| parent.is::<LayoutComponent>())
        {
            let track = self
                .base
                .parent_mut()
                .parent_mut()
                .unwrap()
                .as_mut::<LayoutComponent>()
                .unwrap();
            items.push(Box::new(TrackDraggableProxy::new(self, track.proxy_mut())));
        }
        items
    }

    pub fn constrain(&mut self, component: &mut TransformComponent) {
        let Some(scroll_pointer) = self.scroll_constraint else {
            return;
        };
        let scroll = unsafe { &mut *scroll_pointer };
        let mut thumb_offset_x = 0.0;
        let mut thumb_offset_y = 0.0;
        if self.base.constrains_horizontal() {
            let inner_width = self.track().inner_width();
            let mut thumb_width = self.computed_thumb_width();
            let max_thumb_offset = inner_width - thumb_width;
            thumb_offset_x = if scroll.max_offset_x() == 0.0 {
                0.0
            } else {
                scroll.clamped_offset_x() / scroll.max_offset_x() * max_thumb_offset
            };
            if thumb_offset_x < 0.0 {
                thumb_width += thumb_offset_x;
                thumb_offset_x = 0.0;
            } else if thumb_offset_x > max_thumb_offset {
                thumb_width -= thumb_offset_x - max_thumb_offset;
                thumb_offset_x = if self.base.auto_size() {
                    thumb_offset_x
                } else {
                    max_thumb_offset
                };
            }
            if self.base.auto_size() {
                self.thumb_mut().set_forced_width(thumb_width);
            }
        }
        if self.base.constrains_vertical() {
            let inner_height = self.track().inner_height();
            let mut thumb_height = self.computed_thumb_height();
            let max_thumb_offset = inner_height - thumb_height;
            thumb_offset_y = if scroll.max_offset_y() == 0.0 {
                0.0
            } else {
                scroll.clamped_offset_y() / scroll.max_offset_y() * max_thumb_offset
            };
            if thumb_offset_y < 0.0 {
                thumb_height += thumb_offset_y;
                thumb_offset_y = 0.0;
            } else if thumb_offset_y > max_thumb_offset {
                thumb_height -= thumb_offset_y - max_thumb_offset;
                thumb_offset_y = if self.base.auto_size() {
                    thumb_offset_y
                } else {
                    max_thumb_offset
                };
            }
            if self.base.auto_size() {
                self.thumb_mut().set_forced_height(thumb_height);
            }
        }
        let target_transform = Mat2D::multiply(
            *component.world_transform(),
            Mat2D::from_translate(thumb_offset_x, thumb_offset_y),
        );
        TransformConstraint::constrain_world(
            component,
            *component.world_transform(),
            self.components_a,
            target_transform,
            self.components_b,
            self.base.strength(),
        );
    }

    pub fn scroll_constraint_mut(&mut self) -> &mut ScrollConstraint {
        unsafe { &mut *self.scroll_constraint.unwrap() }
    }
    pub fn set_scroll_constraint(&mut self, constraint: &mut ScrollConstraint) {
        self.scroll_constraint = Some(constraint);
    }

    pub fn build_dependencies(&mut self) {
        unsafe {
            (*self.scroll_constraint.unwrap()).add_dependent(self.base.as_component_mut_ptr())
        };
        self.base.build_dependencies();
    }

    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if result != StatusCode::Ok {
            return result;
        }
        let object = context.resolve_mut(self.base.scroll_constraint_id());
        let Some(scroll) = object.and_then(|object| object.as_mut::<ScrollConstraint>()) else {
            return StatusCode::MissingObject;
        };
        self.scroll_constraint = Some(scroll);
        StatusCode::Ok
    }

    pub fn hit_track(&mut self, world_position: Vec2D) {
        let Some(scroll_pointer) = self.scroll_constraint else {
            return;
        };
        let Some(inverse_world) = self.track().world_transform().inverted() else {
            return;
        };
        let mut local_position = inverse_world * world_position;
        let scroll = unsafe { &mut *scroll_pointer };
        if self.base.constrains_horizontal() {
            local_position.x -= self.track().padding_left();
            let track_range = self.track().inner_width() - self.computed_thumb_width();
            let max_offset = scroll.max_offset_x();
            scroll.set_scroll_offset_x(math_types::clamp(
                local_position.x / track_range * max_offset,
                max_offset,
                0.0,
            ));
        }
        if self.base.constrains_vertical() {
            local_position.y -= self.track().padding_top();
            let track_range = self.track().inner_height() - self.computed_thumb_height();
            let max_offset = scroll.max_offset_y();
            scroll.set_scroll_offset_y(math_types::clamp(
                local_position.y / track_range * max_offset,
                max_offset,
                0.0,
            ));
        }
    }

    pub fn drag_thumb(&mut self, delta: Vec2D, time_stamp: f32) {
        let Some(scroll_pointer) = self.scroll_constraint else {
            return;
        };
        let scroll = unsafe { &mut *scroll_pointer };
        let previous_x = scroll.offset_x();
        let previous_y = scroll.offset_y();
        if self.base.constrains_horizontal() {
            let inner_width = self.track().inner_width();
            let thumb_width = self.computed_thumb_width();
            if self.base.auto_size() {
                self.thumb_mut().set_forced_width(thumb_width);
            }
            let track_range = inner_width - thumb_width;
            let max_offset = scroll.max_offset_x();
            let thumb_offset = scroll.offset_x() / max_offset * track_range + delta.x;
            scroll.set_scroll_offset_x(math_types::clamp(
                thumb_offset / track_range * max_offset,
                max_offset,
                0.0,
            ));
        }
        if self.base.constrains_vertical() {
            let inner_height = self.track().inner_height();
            let thumb_height = self.computed_thumb_height();
            if self.base.auto_size() {
                self.thumb_mut().set_forced_height(thumb_height);
            }
            let track_range = inner_height - thumb_height;
            let max_offset = scroll.max_offset_y();
            let thumb_offset = scroll.offset_y() / max_offset * track_range + delta.y;
            scroll.set_scroll_offset_y(math_types::clamp(
                thumb_offset / track_range * max_offset,
                max_offset,
                0.0,
            ));
        }
        let delta = Vec2D::new(
            scroll.offset_x() - previous_x,
            scroll.offset_y() - previous_y,
        );
        if let Some(physics) = scroll.physics_mut() {
            physics.accumulate(delta, time_stamp);
        }
    }

    pub fn thumb(&self) -> &LayoutComponent {
        self.base.parent().as_ref::<LayoutComponent>().unwrap()
    }
    pub fn thumb_mut(&mut self) -> &mut LayoutComponent {
        self.base.parent_mut().as_mut::<LayoutComponent>().unwrap()
    }
    pub fn track(&self) -> &LayoutComponent {
        self.base
            .parent()
            .parent()
            .unwrap()
            .as_ref::<LayoutComponent>()
            .unwrap()
    }
    pub fn validate(&self, context: &CoreContext) -> bool {
        context
            .resolve(self.base.scroll_constraint_id())
            .is_some_and(|object| object.is::<ScrollConstraint>())
    }
}
