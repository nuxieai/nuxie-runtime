use crate::mechanical_port::source::layout::layout_style_applier::{
    YGAlign, YGDimension, YGDisplay, YGFloatOptional, YGJustify, YGStyle, YGUnit, YGValue,
};
use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    animation::keyframe_interpolator::KeyFrameInterpolator,
    component::ComponentDirt,
    core::{CoreHandle, CoreObject},
    core_context::{CoreContext, StatusCode},
    generated::layout::layout_participant_base::LayoutParticipantBase,
    intrinsically_sizeable::IntrinsicallySizeable,
    layout::{
        layout_data::LayoutData,
        layout_enums::{LayoutDirection, LayoutScaleType, LayoutStyleInterpolation},
        layout_measure_mode::LayoutMeasureMode,
        layout_node_provider::{LayoutNodeProvider, LayoutNodeProviderState},
        layout_style_applier::{LayoutStyleApplier, LayoutSyncContext},
    },
    layout_component::{Layout, LayoutAnimationData, LayoutComponent},
    math::{aabb::Aabb, vec2d::Vec2D},
};

struct ParticipantAnimation {
    animated_layout: Layout,
    a: LayoutAnimationData,
    b: LayoutAnimationData,
    is_smoothing: bool,
    interpolation: LayoutStyleInterpolation,
    interpolator: Option<CoreHandle>,
    interpolation_time: f32,
}

impl Default for ParticipantAnimation {
    fn default() -> Self {
        Self {
            animated_layout: Layout::default(),
            a: LayoutAnimationData::default(),
            b: LayoutAnimationData::default(),
            is_smoothing: false,
            interpolation: LayoutStyleInterpolation::Hold,
            interpolator: None,
            interpolation_time: 0.0,
        }
    }
}

pub struct LayoutParticipant {
    pub base: LayoutParticipantBase,
    provider: LayoutNodeProviderState,
    animation: Option<Box<ParticipantAnimation>>,
    host_bounds: Aabb,
    host_scale_x: f32,
    host_scale_y: f32,
    has_solved_layout: bool,
    host_bounds_valid: bool,
    layout_data: Option<Box<LayoutData>>,
}

impl Default for LayoutParticipant {
    fn default() -> Self {
        Self {
            base: LayoutParticipantBase::default(),
            provider: LayoutNodeProviderState::default(),
            animation: None,
            host_bounds: Aabb::default(),
            host_scale_x: 1.0,
            host_scale_y: 1.0,
            has_solved_layout: false,
            host_bounds_valid: false,
            layout_data: None,
        }
    }
}

impl Drop for LayoutParticipant {
    fn drop(&mut self) {
        self.animation = None;
        self.release_layout_data();
    }
}

impl LayoutParticipant {
    pub fn add_layout_style_applier(&mut self, applier: CoreHandle) {
        if let Some(data) = self.layout_data.as_deref_mut() {
            data.add_applier(applier);
        }
    }

    fn owner_handle(&self) -> Option<CoreHandle> {
        self.base.parent_handle()
    }

    fn with_host_mut<R>(&self, f: impl FnOnce(&mut dyn CoreObject) -> R) -> Option<R> {
        self.owner_handle()?.with_mut(f)
    }

    fn owning_layout_handle(&self) -> Option<CoreHandle> {
        let mut current = self.owner_handle();
        while let Some(owner) = current {
            if owner
                .with(|owner| owner.as_layout_component().is_some())
                .unwrap_or(false)
            {
                return Some(owner);
            }
            current = owner
                .with(|owner| owner.component_parent_handle())
                .flatten();
        }
        None
    }

    fn with_owning_layout_mut<R>(&self, f: impl FnOnce(&mut LayoutComponent) -> R) -> Option<R> {
        self.owning_layout_handle()?
            .with_mut(|owner| owner.as_layout_component_mut().map(f))
            .flatten()
    }
    pub fn is_participating_in_layout(&self) -> bool {
        self.layout_data.is_some()
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.resync();
        StatusCode::Ok
    }

    fn release_layout_data(&mut self) {
        self.layout_data.take();
    }

    pub fn apply_base_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        self.base.apply_sizing_base_style(style, context);
        let width_scale = LayoutScaleType::from(self.base.layout_width_scale_type());
        let height_scale = LayoutScaleType::from(self.base.layout_height_scale_type());
        let width_fill = width_scale == LayoutScaleType::Fill;
        let height_fill = height_scale == LayoutScaleType::Fill;
        style.dimensions_mut()[YGDimension::Width] = if width_scale == LayoutScaleType::Fixed {
            YGValue::new(
                self.base.width().max(0.0),
                YGUnit::from(self.base.width_units_value()),
            )
        } else {
            YGValue::new(f32::NAN, YGUnit::Auto)
        };
        style.dimensions_mut()[YGDimension::Height] = if height_scale == LayoutScaleType::Fixed {
            YGValue::new(
                self.base.height().max(0.0),
                YGUnit::from(self.base.height_units_value()),
            )
        } else {
            YGValue::new(f32::NAN, YGUnit::Auto)
        };
        if context.parent_is_grid {
            style.set_flex_grow(YGFloatOptional::new(0.0));
            style.set_flex_shrink(YGFloatOptional::new(0.0));
            style.set_align_self(if height_fill {
                YGAlign::Stretch
            } else {
                YGAlign::Auto
            });
        } else {
            let main_fill = if context.parent_is_row {
                width_fill
            } else {
                height_fill
            };
            let fraction = if context.parent_is_row {
                self.base.fractional_width()
            } else {
                self.base.fractional_height()
            };
            style.set_flex_grow(YGFloatOptional::new(if main_fill { fraction } else { 0.0 }));
            style.set_flex_shrink(YGFloatOptional::new(if main_fill { fraction } else { 0.0 }));
            style.set_flex_basis(if main_fill {
                YGValue::new(0.0, YGUnit::Point)
            } else {
                YGValue::new(f32::NAN, YGUnit::Auto)
            });
            let cross_fill = if context.parent_is_row {
                height_fill
            } else {
                width_fill
            };
            style.set_align_self(if cross_fill {
                YGAlign::Stretch
            } else {
                YGAlign::Auto
            });
        }
        if width_fill && YGUnit::from(self.base.min_width_units_value()) == YGUnit::Undefined {
            style.min_dimensions_mut()[YGDimension::Width] = YGValue::new(0.0, YGUnit::Point);
        }
        if height_fill && YGUnit::from(self.base.min_height_units_value()) == YGUnit::Undefined {
            style.min_dimensions_mut()[YGDimension::Height] = YGValue::new(0.0, YGUnit::Point);
        }
    }

    pub fn resync(&mut self) {
        if self.owner_handle().is_none() {
            return;
        }
        if self.layout_data.is_none() {
            self.layout_data = Some(Box::new(LayoutData::default()));
            if let Some(this) = self.base.handle() {
                self.add_layout_style_applier(this);
            }
        }
        self.sync_style_changes();
        self.with_owning_layout_mut(LayoutComponent::sync_layout_children);
        self.with_host_mut(|host| {
            if let Some(host) = host.as_transform_component_mut() {
                host.add_dirt_recursive(ComponentDirt::WORLD_TRANSFORM);
            }
        });
        self.mark_layout_node_dirty(true);
    }
    fn solved_layout(&self) -> Layout {
        self.layout_data
            .as_deref()
            .map(|data| data.solved_layout)
            .unwrap_or_default()
    }
    pub fn resolved_left(&self) -> f32 {
        self.animation
            .as_ref()
            .map_or_else(|| self.solved_layout().left(), |a| a.animated_layout.left())
    }
    pub fn resolved_top(&self) -> f32 {
        self.animation
            .as_ref()
            .map_or_else(|| self.solved_layout().top(), |a| a.animated_layout.top())
    }
    pub fn resolved_width(&self) -> f32 {
        self.animation.as_ref().map_or_else(
            || self.solved_layout().width(),
            |a| a.animated_layout.width(),
        )
    }
    pub fn resolved_height(&self) -> f32 {
        self.animation.as_ref().map_or_else(
            || self.solved_layout().height(),
            |a| a.animated_layout.height(),
        )
    }

    fn apply_resolved_layout_size(&mut self) {
        let direction = self
            .owning_layout_handle()
            .and_then(|layout| {
                layout.with(|layout| {
                    layout
                        .as_layout_component()
                        .map(LayoutComponent::actual_direction)
                })
            })
            .flatten()
            .unwrap_or(LayoutDirection::Inherit);
        let resolved = self
            .animation
            .as_ref()
            .map_or_else(|| self.solved_layout(), |a| a.animated_layout);
        let width_scale = LayoutScaleType::from(self.base.layout_width_scale_type());
        let height_scale = LayoutScaleType::from(self.base.layout_height_scale_type());
        self.with_host_mut(|host| {
            if let Some(component) = host.as_component_mut()
                && let Some(sizeable) = IntrinsicallySizeable::from(component)
            {
                sizeable.control_size(
                    Vec2D::new(resolved.width(), resolved.height()),
                    width_scale,
                    height_scale,
                    direction,
                );
            }
        });
    }

    pub fn num_layout_nodes(&self) -> usize {
        usize::from(self.layout_data.is_some())
    }
    pub fn layout_bounds(&self) -> Aabb {
        let resolved = self
            .animation
            .as_ref()
            .map_or_else(|| self.solved_layout(), |a| a.animated_layout);
        Aabb::from_ltwh(
            resolved.left(),
            resolved.top(),
            resolved.width(),
            resolved.height(),
        )
    }
    pub fn layout_bounds_for_node(&self, _index: i32) -> Aabb {
        self.layout_bounds()
    }

    pub fn sync_style_changes(&mut self) -> bool {
        let width_scale = LayoutScaleType::from(self.base.layout_width_scale_type());
        let layout = self.owning_layout_handle();
        let (parent_is_row, parent_is_grid, parent_is_stack, justify, direction_ltr) = layout
            .as_ref()
            .and_then(|layout| {
                layout.with(|layout| {
                    let layout = layout.as_layout_component()?;
                    let style = layout.style_handle();
                    Some((
                        layout.main_axis_is_row(),
                        style
                            .as_ref()
                            .and_then(|style| {
                                style.with_downcast::<
                                    crate::mechanical_port::source::layout::layout_component_style::LayoutComponentStyle,
                                    _,
                                >(|style| style.is_grid())
                            })
                            .unwrap_or(false),
                        style
                            .as_ref()
                            .and_then(|style| {
                                style.with_downcast::<
                                    crate::mechanical_port::source::layout::layout_component_style::LayoutComponentStyle,
                                    _,
                                >(|style| style.is_stack())
                            })
                            .unwrap_or(false),
                        style
                            .as_ref()
                            .and_then(|style| {
                                style.with_downcast::<
                                    crate::mechanical_port::source::layout::layout_component_style::LayoutComponentStyle,
                                    _,
                                >(|style| style.base.justify_items_value())
                            })
                            .unwrap_or(YGJustify::Stretch as u32),
                        layout.actual_direction() != LayoutDirection::Rtl,
                    ))
                })
            })
            .flatten()
            .unwrap_or((true, false, false, YGJustify::Stretch as u32, true));
        let context = LayoutSyncContext {
            parent_is_grid,
            parent_is_stack,
            container_justify_items: justify,
            inline_hugs: width_scale == LayoutScaleType::Hug,
            parent_is_row,
            is_ltr: direction_ltr,
            has_layout_parent: layout.is_some(),
        };
        let Some(data) = self.layout_data.as_deref_mut() else {
            return false;
        };
        let mut style = std::mem::take(&mut data.style);
        data.apply_layout_styles(&mut style, &context);
        data.style = style;
        data.dirty = true;

        let owner = self.owner_handle();
        let mut parent_hides_host = owner
            .as_ref()
            .and_then(|owner| owner.with(|owner| owner.component_parent_handle()))
            .flatten()
            .and_then(|parent| {
                parent.with(|parent| parent.as_component().map(|p| p.is_collapsed()))
            })
            .flatten()
            .unwrap_or(false);
        if let Some(owner) = owner {
            let parent = owner
                .with(|owner| owner.component_parent_handle())
                .flatten();
            if let Some(parent) = parent {
                let solo_active = parent
                    .with_downcast::<crate::mechanical_port::source::solo::Solo, _>(|solo| {
                        solo.base.active_component_id()
                    });
                if let Some(active_id) = solo_active {
                    let owner_id = parent
                        .with(|parent| parent.component_artboard_handle())
                        .flatten()
                        .and_then(|artboard| artboard.with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(|artboard| artboard.id_of(&owner)));
                    parent_hides_host |= owner_id != Some(active_id);
                }
            }
            owner.with_mut(|host| {
                if let Some(host) = host.as_transform_component_mut() {
                    host.collapse(
                        parent_hides_host
                            || YGDisplay::from(self.base.display_value()) == YGDisplay::None,
                    );
                }
            });
        }
        true
    }

    pub fn update_layout_bounds(&mut self, animate: bool) {
        {
            let Some(data) = self.layout_data.as_deref_mut() else {
                return;
            };
            if !data.has_new_layout {
                return;
            }
            data.has_new_layout = false;
            let new_layout = data.solved_layout;
            if self.animation.is_some() && animate && self.has_solved_layout {
                let animation = self.animation.as_mut().unwrap();
                let data = if animation.is_smoothing {
                    &mut animation.b
                } else {
                    &mut animation.a
                };
                if new_layout != data.to {
                    if data.elapsed_seconds != 0.0 {
                        if animation.is_smoothing {
                            animation.a.copy_from(&animation.b);
                        }
                        animation.is_smoothing = true;
                    } else {
                        animation.is_smoothing = false;
                    }
                    let data = if animation.is_smoothing {
                        &mut animation.b
                    } else {
                        &mut animation.a
                    };
                    data.from = animation.animated_layout;
                    data.to = new_layout;
                    data.elapsed_seconds = 0.0;
                }
            } else if let Some(animation) = &mut self.animation {
                animation.animated_layout = new_layout;
                animation.a.to = new_layout;
            }
            self.has_solved_layout = true;
            self.apply_resolved_layout_size();
            self.with_host_mut(|host| {
                if let Some(host) = host.as_transform_component_mut() {
                    host.add_dirt_recursive(ComponentDirt::WORLD_TRANSFORM);
                }
            });
        }
    }

    pub fn mark_layout_node_dirty(&mut self, force: bool) {
        {
            if let Some(data) = self.layout_data.as_deref_mut() {
                data.dirty = true;
            }
            self.with_owning_layout_mut(|layout| layout.mark_layout_node_dirty(force));
        }
    }
    fn on_sizing_changed(&mut self) {
        {
            self.sync_style_changes();
            self.mark_layout_node_dirty(false);
        }
    }
    pub fn layout_width_scale_type_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn layout_height_scale_type_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn width_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn height_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn fractional_width_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn fractional_height_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn min_width_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn max_width_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn min_height_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn max_height_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn min_width_units_value_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn max_width_units_value_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn min_height_units_value_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn max_height_units_value_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn width_units_value_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn height_units_value_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn justify_self_value_changed(&mut self) {
        self.on_sizing_changed();
    }
    pub fn display_value_changed(&mut self) {
        self.on_sizing_changed();
    }

    fn current_animation_data(&mut self) -> &mut LayoutAnimationData {
        let animation = self.animation.as_mut().unwrap();
        if animation.is_smoothing {
            &mut animation.b
        } else {
            &mut animation.a
        }
    }
    pub fn animates(&self) -> bool {
        self.animation.is_some()
    }
    pub fn interpolation(&self) -> LayoutStyleInterpolation {
        self.animation
            .as_ref()
            .map_or(LayoutStyleInterpolation::Hold, |value| value.interpolation)
    }
    pub fn interpolation_time(&self) -> f32 {
        self.animation
            .as_ref()
            .map_or(0.0, |value| value.interpolation_time)
    }
    pub fn interpolator(&self) -> Option<CoreHandle> {
        self.animation
            .as_ref()
            .and_then(|value| value.interpolator.clone())
    }

    pub fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        if self.animation.is_none() || !flags.contains(AdvanceFlags::NEW_FRAME) {
            return false;
        }
        self.apply_interpolation(
            elapsed_seconds,
            flags.contains(AdvanceFlags::ANIMATE) || flags.contains(AdvanceFlags::ADVANCE_NESTED),
        )
    }

    pub fn cascade_layout_style(
        &mut self,
        inherited: LayoutStyleInterpolation,
        interpolator: Option<CoreHandle>,
        time: f32,
        _direction: LayoutDirection,
    ) -> bool {
        let will_animate = inherited != LayoutStyleInterpolation::Hold && time > 0.0;
        if will_animate {
            if self.animation.is_none() {
                let current = self.solved_layout();
                let mut animation = ParticipantAnimation::default();
                animation.animated_layout = current;
                animation.a.from = current;
                animation.a.to = current;
                self.animation = Some(Box::new(animation));
            }
            let animation = self.animation.as_mut().unwrap();
            animation.interpolation = inherited;
            animation.interpolator = interpolator;
            animation.interpolation_time = time;
        } else if self.animation.is_some() {
            self.animation = None;
        }
        will_animate
    }

    fn apply_interpolation(&mut self, elapsed_seconds: f32, animate: bool) -> bool {
        let Some(animation) = &self.animation else {
            return false;
        };
        let target = if animation.is_smoothing {
            animation.b.to
        } else {
            animation.a.to
        };
        if !animate || target == animation.animated_layout {
            return false;
        }
        if self.animation.as_ref().unwrap().is_smoothing {
            let animation = self.animation.as_mut().unwrap();
            let mut fraction = (if animation.interpolation_time > 0.0 {
                animation.a.elapsed_seconds / animation.interpolation_time
            } else {
                1.0
            })
            .min(1.0);
            if animation.interpolation != LayoutStyleInterpolation::Linear {
                if let Some(interpolator) = animation.interpolator.as_ref() {
                    fraction = interpolator
                        .with(|interpolator| interpolator.keyframe_interpolator_transform(fraction))
                        .flatten()
                        .unwrap_or(fraction);
                }
            }
            animation.b.from = animation.a.interpolate(fraction);
            if fraction == 1.0 {
                animation.a.copy_from(&animation.b);
                animation.is_smoothing = false;
            } else {
                animation.a.elapsed_seconds += elapsed_seconds;
            }
        }
        let time = self.interpolation_time();
        if self.current_animation_data().elapsed_seconds >= time {
            let target = self.current_animation_data().to;
            let animation = self.animation.as_mut().unwrap();
            animation.animated_layout = target;
            if animation.is_smoothing {
                animation.is_smoothing = false;
                animation.a.copy_from(&animation.b);
                animation.a.elapsed_seconds = 0.0;
                animation.b.elapsed_seconds = 0.0;
            } else {
                animation.a.elapsed_seconds = 0.0;
            }
            self.apply_resolved_layout_size();
            self.with_host_mut(|host| {
                if let Some(host) = host.as_transform_component_mut() {
                    host.add_dirt_recursive(ComponentDirt::WORLD_TRANSFORM);
                }
            });
            return false;
        }
        let elapsed = self.current_animation_data().elapsed_seconds;
        let mut fraction = (if time > 0.0 { elapsed / time } else { 1.0 }).min(1.0);
        if self.interpolation() != LayoutStyleInterpolation::Linear {
            if let Some(interpolator) = self.interpolator() {
                fraction = interpolator
                    .with(|interpolator| interpolator.keyframe_interpolator_transform(fraction))
                    .flatten()
                    .unwrap_or(fraction);
            }
        }
        let current = self.current_animation_data().interpolate(fraction);
        if self.animation.as_ref().unwrap().animated_layout != current {
            self.animation.as_mut().unwrap().animated_layout = current;
            self.apply_resolved_layout_size();
            self.with_host_mut(|host| {
                if let Some(host) = host.as_transform_component_mut() {
                    host.add_dirt_recursive(ComponentDirt::WORLD_TRANSFORM);
                }
            });
        }
        self.current_animation_data().elapsed_seconds += elapsed_seconds;
        fraction != 1.0
    }

    pub fn host_scale_x(&self) -> f32 {
        self.host_scale_x
    }
    pub fn host_scale_y(&self) -> f32 {
        self.host_scale_y
    }
    pub fn set_host_scale(&mut self, x: f32, y: f32) {
        self.host_scale_x = x;
        self.host_scale_y = y;
    }
    pub fn host_bounds_valid(&self) -> bool {
        self.host_bounds_valid
    }
    pub fn host_bounds(&self) -> &Aabb {
        &self.host_bounds
    }
    pub fn set_host_bounds(&mut self, bounds: Aabb, cache: bool) {
        self.host_bounds = bounds;
        self.host_bounds_valid = cache;
    }
    pub fn invalidate_host_bounds(&mut self) {
        self.host_bounds_valid = false;
    }
}

impl LayoutStyleApplier for LayoutParticipant {
    fn apply_base_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        LayoutParticipant::apply_base_style(self, style, context);
    }
}

impl LayoutNodeProvider for LayoutParticipant {
    fn provider_state(&mut self) -> &mut LayoutNodeProviderState {
        &mut self.provider
    }
    fn provider_handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.base.handle()
    }
    fn owner_handle(&self) -> Option<CoreHandle> {
        LayoutParticipant::owner_handle(self)
    }
    fn layout_bounds(&self) -> Aabb {
        LayoutParticipant::layout_bounds(self)
    }
    fn layout_bounds_for_node(&self, index: usize) -> Aabb {
        LayoutParticipant::layout_bounds_for_node(self, index as i32)
    }
    fn sync_style_changes(&mut self) -> bool {
        LayoutParticipant::sync_style_changes(self)
    }
    fn update_layout_bounds(&mut self, animate: bool) {
        LayoutParticipant::update_layout_bounds(self, animate);
    }
    fn mark_layout_node_dirty(&mut self, force: bool) {
        LayoutParticipant::mark_layout_node_dirty(self, force);
    }
    fn add_layout_style_applier(&mut self, applier: CoreHandle) {
        LayoutParticipant::add_layout_style_applier(self, applier);
    }
    fn num_layout_nodes(&self) -> usize {
        LayoutParticipant::num_layout_nodes(self)
    }
    fn cascade_layout_style(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<CoreHandle>,
        time: f32,
        direction: LayoutDirection,
    ) -> bool {
        LayoutParticipant::cascade_layout_style(self, interpolation, interpolator, time, direction)
    }
}
