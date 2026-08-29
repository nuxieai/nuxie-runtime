use crate::mechanical_port::source::{
    constraints::{
        draggable_constraint::{DraggableConstraint, DraggableProxy},
        scrolling::{
            scroll_bar_constraint_proxy::{ThumbDraggableProxy, TrackDraggableProxy},
            scroll_constraint::ScrollConstraint,
        },
        transform_constraint::TransformConstraint,
    },
    core::{CoreHandle, CoreObject},
    core_context::{CoreContext, StatusCode},
    generated::constraints::scrolling::scroll_bar_constraint_base::ScrollBarConstraintBase,
    generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
    generated::core_registry::CoreCapabilities,
    generated::layout_component_base::LayoutComponentBase,
    layout_component::LayoutComponent,
    math::{mat2d::Mat2D, math_types, transform_components::TransformComponents, vec2d::Vec2D},
    transform_component::TransformComponent,
};

pub struct ScrollBarConstraint {
    pub base: ScrollBarConstraintBase,
    components_a: TransformComponents,
    components_b: TransformComponents,
    scroll_constraint: Option<CoreHandle>,
}

impl Default for ScrollBarConstraint {
    fn default() -> Self {
        Self {
            base: ScrollBarConstraintBase::default(),
            components_a: TransformComponents::default(),
            components_b: TransformComponents::default(),
            scroll_constraint: None,
        }
    }
}

impl ScrollBarConstraint {
    fn thumb_handle(&self) -> Option<CoreHandle> {
        self.component_parent_handle()
    }

    fn track_handle(&self) -> Option<CoreHandle> {
        self.thumb_handle()?
            .with(|thumb| thumb.component_parent_handle())?
    }

    fn with_thumb<R>(&self, use_thumb: impl FnOnce(&LayoutComponent) -> R) -> Option<R> {
        self.thumb_handle()?
            .with(|thumb| thumb.as_layout_component().map(use_thumb))?
    }

    fn with_thumb_mut<R>(&self, use_thumb: impl FnOnce(&mut LayoutComponent) -> R) -> Option<R> {
        self.thumb_handle()?
            .with_mut(|thumb| thumb.as_layout_component_mut().map(use_thumb))?
    }

    fn with_track<R>(&self, use_track: impl FnOnce(&LayoutComponent) -> R) -> Option<R> {
        self.track_handle()?
            .with(|track| track.as_layout_component().map(use_track))?
    }

    fn with_scroll<R>(&self, use_scroll: impl FnOnce(&ScrollConstraint) -> R) -> Option<R> {
        self.scroll_constraint
            .as_ref()?
            .with_downcast::<ScrollConstraint, _>(use_scroll)
    }

    fn with_scroll_mut<R>(&self, use_scroll: impl FnOnce(&mut ScrollConstraint) -> R) -> Option<R> {
        self.scroll_constraint
            .as_ref()?
            .with_downcast_mut::<ScrollConstraint, _>(use_scroll)
    }

    pub fn computed_thumb_width(&self) -> f32 {
        if self.base.auto_size() {
            if let Some(ratio) = self.with_scroll(ScrollConstraint::visible_width_ratio) {
                return self
                    .with_track(LayoutComponent::inner_width)
                    .expect("ScrollBarConstraint track")
                    * ratio;
            }
        }
        self.with_thumb(LayoutComponent::layout_width)
            .expect("ScrollBarConstraint thumb")
    }
    pub fn computed_thumb_height(&self) -> f32 {
        if self.base.auto_size() {
            if let Some(ratio) = self.with_scroll(ScrollConstraint::visible_height_ratio) {
                return self
                    .with_track(LayoutComponent::inner_height)
                    .expect("ScrollBarConstraint track")
                    * ratio;
            }
        }
        self.with_thumb(LayoutComponent::layout_height)
            .expect("ScrollBarConstraint thumb")
    }

    pub fn draggables(&mut self) -> Vec<Box<dyn DraggableProxy>> {
        let mut items: Vec<Box<dyn DraggableProxy>> = Vec::new();
        let constraint = self
            .core()
            .handle()
            .expect("arena-owned ScrollBarConstraint");
        if let Some(thumb) = self.thumb_handle()
            && thumb.is_type_of(LayoutComponentBase::TYPE_KEY)
        {
            let thumb = thumb
                .with_mut(|thumb| {
                    thumb
                        .as_layout_component_mut()
                        .and_then(LayoutComponent::proxy)
                })
                .flatten()
                .expect("ScrollBarConstraint thumb retains its drawable proxy");
            items.push(Box::new(ThumbDraggableProxy::new(
                constraint.clone(),
                thumb,
            )));
        }
        if let Some(track) = self.track_handle()
            && track.is_type_of(LayoutComponentBase::TYPE_KEY)
        {
            let track = track
                .with_mut(|track| {
                    track
                        .as_layout_component_mut()
                        .and_then(LayoutComponent::proxy)
                })
                .flatten()
                .expect("ScrollBarConstraint track retains its drawable proxy");
            items.push(Box::new(TrackDraggableProxy::new(constraint, track)));
        }
        items
    }

    pub fn constrain(&mut self, component: &mut TransformComponent) {
        let Some((max_offset_x, clamped_offset_x, max_offset_y, clamped_offset_y)) = self
            .with_scroll(|scroll| {
                (
                    scroll.max_offset_x(),
                    scroll.clamped_offset_x(),
                    scroll.max_offset_y(),
                    scroll.clamped_offset_y(),
                )
            })
        else {
            return;
        };
        if self.with_thumb(|_| ()).is_none() || self.with_track(|_| ()).is_none() {
            return;
        }
        let mut thumb_offset_x = 0.0;
        let mut thumb_offset_y = 0.0;
        if self.base.constrains_horizontal() {
            let inner_width = self
                .with_track(LayoutComponent::inner_width)
                .expect("validated ScrollBarConstraint track");
            let mut thumb_width = self.computed_thumb_width();
            let max_thumb_offset = inner_width - thumb_width;
            thumb_offset_x = if max_offset_x == 0.0 {
                0.0
            } else {
                clamped_offset_x / max_offset_x * max_thumb_offset
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
                self.with_thumb_mut(|thumb| thumb.set_forced_width(thumb_width))
                    .expect("validated ScrollBarConstraint thumb");
            }
        }
        if self.base.constrains_vertical() {
            let inner_height = self
                .with_track(LayoutComponent::inner_height)
                .expect("validated ScrollBarConstraint track");
            let mut thumb_height = self.computed_thumb_height();
            let max_thumb_offset = inner_height - thumb_height;
            thumb_offset_y = if max_offset_y == 0.0 {
                0.0
            } else {
                clamped_offset_y / max_offset_y * max_thumb_offset
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
                self.with_thumb_mut(|thumb| thumb.set_forced_height(thumb_height))
                    .expect("validated ScrollBarConstraint thumb");
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

    pub fn set_scroll_constraint(&mut self, constraint: CoreHandle) {
        self.scroll_constraint = Some(constraint);
    }

    pub fn start_thumb_drag(&mut self, time_stamp: f32) {
        self.with_scroll_mut(|scroll| {
            scroll.set_is_scroll_bar_dragging(true);
            scroll.accumulate_physics(Vec2D::default(), time_stamp);
        })
        .expect("resolved ScrollConstraint occurrence");
    }

    pub fn end_thumb_drag(&mut self) {
        self.with_scroll_mut(|scroll| {
            scroll.set_is_scroll_bar_dragging(false);
            scroll.clear_velocity();
        })
        .expect("resolved ScrollConstraint occurrence");
    }

    pub fn build_dependencies(&mut self) {
        let dependent = self
            .core()
            .handle()
            .expect("arena-owned ScrollBarConstraint");
        self.scroll_constraint
            .as_ref()
            .and_then(|scroll| scroll.with_mut(|scroll| scroll.component_add_dependent(dependent)))
            .filter(|added| *added)
            .expect("resolved ScrollConstraint component");
        self.base.build_dependencies();
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if result != StatusCode::Ok {
            return result;
        }
        let Some(scroll) = context.resolve(self.base.scroll_constraint_id()) else {
            return StatusCode::MissingObject;
        };
        if !scroll.is_type_of(ScrollConstraintBase::TYPE_KEY) {
            return StatusCode::MissingObject;
        }
        self.scroll_constraint = Some(scroll);
        StatusCode::Ok
    }

    pub fn hit_track(&mut self, world_position: Vec2D) {
        if self.scroll_constraint.is_none() {
            return;
        }
        let Some((inverse_world, padding_left, padding_top, inner_width, inner_height)) = self
            .with_track(|track| {
                let mut inverse_world = Mat2D::default();
                if !track.world_transform().invert(&mut inverse_world) {
                    return None;
                }
                Some((
                    inverse_world,
                    track.padding_left(),
                    track.padding_top(),
                    track.inner_width(),
                    track.inner_height(),
                ))
            })
            .flatten()
        else {
            return;
        };
        let mut local_position = inverse_world * world_position;
        let horizontal = self.base.constrains_horizontal();
        let vertical = self.base.constrains_vertical();
        let thumb_width = horizontal.then(|| self.computed_thumb_width());
        let thumb_height = vertical.then(|| self.computed_thumb_height());
        self.with_scroll_mut(|scroll| {
            if let Some(thumb_width) = thumb_width {
                local_position.x -= padding_left;
                let track_range = inner_width - thumb_width;
                let max_offset = scroll.max_offset_x();
                scroll.set_authored_scroll_offset_x(math_types::clamp(
                    local_position.x / track_range * max_offset,
                    max_offset,
                    0.0,
                ));
            }
            if let Some(thumb_height) = thumb_height {
                local_position.y -= padding_top;
                let track_range = inner_height - thumb_height;
                let max_offset = scroll.max_offset_y();
                scroll.set_authored_scroll_offset_y(math_types::clamp(
                    local_position.y / track_range * max_offset,
                    max_offset,
                    0.0,
                ));
            }
        })
        .expect("resolved ScrollConstraint occurrence");
    }

    pub fn drag_thumb(&mut self, delta: Vec2D, time_stamp: f32) {
        if self.scroll_constraint.is_none() {
            return;
        }
        let Some((inner_width, inner_height)) =
            self.with_track(|track| (track.inner_width(), track.inner_height()))
        else {
            return;
        };
        let horizontal = self.base.constrains_horizontal();
        let vertical = self.base.constrains_vertical();
        let thumb_width = horizontal.then(|| self.computed_thumb_width());
        let thumb_height = vertical.then(|| self.computed_thumb_height());
        if self.base.auto_size() {
            self.with_thumb_mut(|thumb| {
                if let Some(width) = thumb_width {
                    thumb.set_forced_width(width);
                }
                if let Some(height) = thumb_height {
                    thumb.set_forced_height(height);
                }
            })
            .expect("validated ScrollBarConstraint thumb");
        }
        self.with_scroll_mut(|scroll| {
            let previous_x = scroll.offset_x();
            let previous_y = scroll.offset_y();
            if let Some(thumb_width) = thumb_width {
                let track_range = inner_width - thumb_width;
                let max_offset = scroll.max_offset_x();
                let thumb_offset = scroll.offset_x() / max_offset * track_range + delta.x;
                scroll.set_authored_scroll_offset_x(math_types::clamp(
                    thumb_offset / track_range * max_offset,
                    max_offset,
                    0.0,
                ));
            }
            if let Some(thumb_height) = thumb_height {
                let track_range = inner_height - thumb_height;
                let max_offset = scroll.max_offset_y();
                let thumb_offset = scroll.offset_y() / max_offset * track_range + delta.y;
                scroll.set_authored_scroll_offset_y(math_types::clamp(
                    thumb_offset / track_range * max_offset,
                    max_offset,
                    0.0,
                ));
            }
            let applied_delta = Vec2D::new(
                scroll.offset_x() - previous_x,
                scroll.offset_y() - previous_y,
            );
            scroll.accumulate_physics(applied_delta, time_stamp);
        })
        .expect("resolved ScrollConstraint occurrence");
    }
    pub fn validate(&self, context: &dyn CoreContext) -> bool {
        context
            .resolve(self.base.scroll_constraint_id())
            .is_some_and(|object| object.is_type_of(ScrollConstraintBase::TYPE_KEY))
    }
}
