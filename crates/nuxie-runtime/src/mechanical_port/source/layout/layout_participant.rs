#[cfg(feature = "rive-layout")]
use crate::mechanical_port::source::yoga::{
    YGAlign, YGDimension, YGDisplay, YGFloatOptional, YGJustify, YGMeasureMode, YGNode, YGSize,
    YGStyle, YGUnit, YGValue,
};
use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    animation::keyframe_interpolator::KeyFrameInterpolator,
    component::{Component, ComponentDirt},
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
    solo::Solo,
    transform_component::TransformComponent,
};

struct ParticipantAnimation {
    animated_layout: Layout,
    a: LayoutAnimationData,
    b: LayoutAnimationData,
    is_smoothing: bool,
    interpolation: LayoutStyleInterpolation,
    interpolator: Option<*mut KeyFrameInterpolator>,
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
    #[cfg(feature = "rive-layout")]
    layout_data: Option<*mut LayoutData>,
}

impl Drop for LayoutParticipant {
    fn drop(&mut self) {
        self.animation = None;
        #[cfg(feature = "rive-layout")]
        self.release_layout_data();
    }
}

impl LayoutParticipant {
    #[cfg(feature = "rive-layout")]
    pub fn add_layout_style_applier(&mut self, applier: *mut dyn LayoutStyleApplier) {
        if let Some(data) = self.layout_data {
            unsafe { (*data).add_applier(applier) };
        }
    }

    #[cfg(feature = "rive-layout")]
    fn participant_measure(
        node: &mut YGNode,
        width: f32,
        width_mode: YGMeasureMode,
        height: f32,
        height_mode: YGMeasureMode,
    ) -> YGSize {
        let component = node.context_mut::<Component>();
        let size = IntrinsicallySizeable::from(component).map_or(Vec2D::default(), |sizeable| {
            sizeable.measure_layout(
                width,
                LayoutMeasureMode::from(width_mode),
                height,
                LayoutMeasureMode::from(height_mode),
            )
        });
        YGSize {
            width: size.x,
            height: size.y,
        }
    }

    pub fn transform_component_mut(&mut self) -> Option<&mut TransformComponent> {
        self.base
            .parent_mut()
            .and_then(|parent| parent.as_mut::<TransformComponent>())
    }
    pub fn transform_component(&self) -> Option<&TransformComponent> {
        self.base
            .parent()
            .and_then(|parent| parent.as_ref::<TransformComponent>())
    }
    fn owning_layout(&mut self) -> Option<&mut LayoutComponent> {
        let mut component = self.base.parent_mut().map(|value| value as *mut Component);
        while let Some(pointer) = component {
            unsafe {
                if let Some(layout) = (*pointer).as_mut::<LayoutComponent>() {
                    return Some(layout);
                }
                component = (*pointer).parent_mut().map(|value| value as *mut Component);
            }
        }
        None
    }
    pub fn is_participating_in_layout(&self) -> bool {
        #[cfg(feature = "rive-layout")]
        {
            self.layout_data.is_some()
        }
        #[cfg(not(feature = "rive-layout"))]
        {
            false
        }
    }
    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        #[cfg(feature = "rive-layout")]
        self.resync();
        StatusCode::Ok
    }

    #[cfg(feature = "rive-layout")]
    fn release_layout_data(&mut self) {
        let Some(pointer) = self.layout_data.take() else {
            return;
        };
        #[cfg(feature = "rive-tools")]
        unsafe {
            (*pointer).unref();
        }
        #[cfg(not(feature = "rive-tools"))]
        unsafe {
            drop(Box::from_raw(pointer));
        }
    }

    #[cfg(feature = "rive-layout")]
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

    #[cfg(feature = "rive-layout")]
    pub fn resync(&mut self) {
        let Some(host) = self
            .transform_component_mut()
            .map(|value| value as *mut TransformComponent)
        else {
            return;
        };
        if self.layout_data.is_none() {
            let mut data = Box::new(LayoutData::default());
            data.node.config_mut().set_point_scale_factor(0.0);
            data.node.set_context(unsafe { &mut *host });
            data.node.set_measure_func(Some(Self::participant_measure));
            self.layout_data = Some(Box::into_raw(data));
            self.add_layout_style_applier(self as *mut Self as *mut dyn LayoutStyleApplier);
        }
        self.sync_style_changes();
        if let Some(layout) = self.owning_layout() {
            layout.sync_layout_children();
        }
        unsafe {
            (*host).add_dirt_recursive(ComponentDirt::WORLD_TRANSFORM);
        }
        self.mark_layout_node_dirty(true);
    }
    #[cfg(feature = "rive-layout")]
    pub fn layout_node(&mut self, _index: i32) -> *mut core::ffi::c_void {
        self.layout_data
            .map_or(std::ptr::null_mut(), |data| unsafe {
                &mut (*data).node as *mut _ as *mut _
            })
    }
    #[cfg(feature = "rive-layout")]
    fn solved_layout(&self) -> Layout {
        let Some(data) = self.layout_data else {
            return Layout::default();
        };
        let layout = unsafe { (*data).node.layout() };
        let defined = |value: f32| if value.is_nan() { 0.0 } else { value };
        Layout::new(
            defined(layout.left()),
            defined(layout.top()),
            defined(layout.width()),
            defined(layout.height()),
        )
    }
    #[cfg(not(feature = "rive-layout"))]
    fn solved_layout(&self) -> Layout {
        Layout::default()
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

    #[cfg(feature = "rive-layout")]
    fn apply_resolved_layout_size(&mut self) {
        let Some(sizeable) = self
            .transform_component_mut()
            .and_then(IntrinsicallySizeable::from)
        else {
            return;
        };
        let direction = self
            .owning_layout()
            .map_or(LayoutDirection::Inherit, |layout| layout.actual_direction());
        let resolved = self
            .animation
            .as_ref()
            .map_or_else(|| self.solved_layout(), |a| a.animated_layout);
        sizeable.control_size(
            Vec2D::new(resolved.width(), resolved.height()),
            LayoutScaleType::from(self.base.layout_width_scale_type()),
            LayoutScaleType::from(self.base.layout_height_scale_type()),
            direction,
        );
    }

    pub fn num_layout_nodes(&self) -> usize {
        #[cfg(feature = "rive-layout")]
        {
            usize::from(self.layout_data.is_some())
        }
        #[cfg(not(feature = "rive-layout"))]
        {
            0
        }
    }
    pub fn layout_bounds(&self) -> Aabb {
        #[cfg(feature = "rive-layout")]
        {
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
        #[cfg(not(feature = "rive-layout"))]
        {
            Aabb::from_ltwh(0.0, 0.0, 0.0, 0.0)
        }
    }
    pub fn layout_bounds_for_node(&self, _index: i32) -> Aabb {
        self.layout_bounds()
    }

    pub fn sync_style_changes(&mut self) -> bool {
        #[cfg(not(feature = "rive-layout"))]
        {
            return false;
        }
        #[cfg(feature = "rive-layout")]
        {
            let Some(data_pointer) = self.layout_data else {
                return false;
            };
            let data = unsafe { &mut *data_pointer };
            let width_scale = LayoutScaleType::from(self.base.layout_width_scale_type());
            let height_scale = LayoutScaleType::from(self.base.layout_height_scale_type());
            let layout = self
                .owning_layout()
                .map(|value| value as *mut LayoutComponent);
            let parent_is_row =
                layout.map_or(true, |pointer| unsafe { (*pointer).main_axis_is_row() });
            let parent_is_grid = layout.is_some_and(|pointer| unsafe {
                (*pointer).style().is_some_and(|style| style.is_grid())
            });
            let needs_measure =
                width_scale == LayoutScaleType::Hug || height_scale == LayoutScaleType::Hug;
            if needs_measure {
                data.node
                    .set_context(self.transform_component_mut().unwrap());
                data.node.set_measure_func(Some(Self::participant_measure));
            } else {
                data.node.set_measure_func(None);
            }
            let parent_is_stack = layout.is_some_and(|pointer| unsafe {
                (*pointer).style().is_some_and(|style| style.is_stack())
            });
            let justify = layout
                .and_then(|pointer| unsafe {
                    (*pointer)
                        .style()
                        .map(|style| style.base.justify_items_value())
                })
                .unwrap_or(YGJustify::Stretch as u32);
            let direction_ltr = layout.is_none_or(|pointer| unsafe {
                (*pointer).actual_direction() != LayoutDirection::Rtl
            });
            let context = LayoutSyncContext {
                parent_is_grid,
                parent_is_stack,
                container_justify_items: justify,
                inline_hugs: width_scale == LayoutScaleType::Hug,
                parent_is_row,
                is_ltr: direction_ltr,
                has_layout_parent: layout.is_some(),
            };
            data.apply_layout_styles(&mut data.style, &context);
            data.node.set_style(data.style.clone());
            data.node.mark_dirty_and_propagate();
            if let Some(host) = self.transform_component_mut() {
                let parent = host.parent_mut();
                let mut parent_hides_host =
                    parent.as_ref().is_some_and(|parent| parent.is_collapsed());
                if let Some(solo) = parent.and_then(|parent| parent.as_mut::<Solo>()) {
                    let active = solo
                        .artboard_mut()
                        .and_then(|artboard| artboard.resolve_mut(solo.active_component_id()));
                    if active.is_none_or(|active| !std::ptr::eq(active, host.as_component())) {
                        parent_hides_host = true;
                    }
                }
                host.collapse(
                    parent_hides_host
                        || YGDisplay::from(self.base.display_value()) == YGDisplay::None,
                );
            }
            true
        }
    }

    pub fn update_layout_bounds(&mut self, animate: bool) {
        #[cfg(feature = "rive-layout")]
        {
            let Some(data) = self.layout_data.map(|pointer| unsafe { &mut *pointer }) else {
                return;
            };
            if !data.node.has_new_layout() {
                return;
            }
            data.node.set_has_new_layout(false);
            let new_layout = self.solved_layout();
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
            if let Some(host) = self.transform_component_mut() {
                host.add_dirt_recursive(ComponentDirt::WORLD_TRANSFORM);
            }
        }
    }

    pub fn mark_layout_node_dirty(&mut self, force: bool) {
        #[cfg(feature = "rive-layout")]
        {
            if let Some(data) = self.layout_data {
                unsafe { (*data).node.mark_dirty_and_propagate() };
            }
            if let Some(layout) = self.owning_layout() {
                layout.mark_layout_node_dirty(force);
            }
        }
    }
    fn on_sizing_changed(&mut self) {
        #[cfg(feature = "rive-layout")]
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
    pub fn interpolator(&self) -> Option<&KeyFrameInterpolator> {
        self.animation
            .as_ref()
            .and_then(|value| value.interpolator)
            .map(|pointer| unsafe { &*pointer })
    }

    pub fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        #[cfg(feature = "rive-layout")]
        {
            if self.animation.is_none() || !flags.contains(AdvanceFlags::NEW_FRAME) {
                return false;
            }
            return self.apply_interpolation(
                elapsed_seconds,
                flags.contains(AdvanceFlags::ANIMATE)
                    || flags.contains(AdvanceFlags::ADVANCE_NESTED),
            );
        }
        #[cfg(not(feature = "rive-layout"))]
        {
            false
        }
    }

    #[cfg(feature = "rive-layout")]
    pub fn cascade_layout_style(
        &mut self,
        inherited: LayoutStyleInterpolation,
        interpolator: Option<&mut KeyFrameInterpolator>,
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
            animation.interpolator = interpolator.map(|value| value as *mut _);
            animation.interpolation_time = time;
        } else if self.animation.is_some() {
            self.animation = None;
        }
        will_animate
    }

    #[cfg(feature = "rive-layout")]
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
                if let Some(interpolator) = animation.interpolator {
                    fraction = unsafe { (*interpolator).transform(fraction) };
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
            if let Some(host) = self.transform_component_mut() {
                host.add_dirt_recursive(ComponentDirt::WORLD_TRANSFORM);
            }
            return false;
        }
        let elapsed = self.current_animation_data().elapsed_seconds;
        let mut fraction = (if time > 0.0 { elapsed / time } else { 1.0 }).min(1.0);
        if self.interpolation() != LayoutStyleInterpolation::Linear {
            if let Some(interpolator) = self.interpolator() {
                fraction = interpolator.transform(fraction);
            }
        }
        let current = self.current_animation_data().interpolate(fraction);
        if self.animation.as_ref().unwrap().animated_layout != current {
            self.animation.as_mut().unwrap().animated_layout = current;
            self.apply_resolved_layout_size();
            if let Some(host) = self.transform_component_mut() {
                host.add_dirt_recursive(ComponentDirt::WORLD_TRANSFORM);
            }
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
    #[cfg(feature = "rive-layout")]
    fn apply_base_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        LayoutParticipant::apply_base_style(self, style, context);
    }
}

impl LayoutNodeProvider for LayoutParticipant {
    fn provider_state(&mut self) -> &mut LayoutNodeProviderState {
        &mut self.provider
    }
    #[cfg(feature = "rive-layout")]
    fn layout_node(&mut self, index: i32) -> *mut core::ffi::c_void {
        LayoutParticipant::layout_node(self, index)
    }
    fn transform_component_mut(&mut self) -> Option<&mut TransformComponent> {
        LayoutParticipant::transform_component_mut(self)
    }
    fn transform_component(&self) -> Option<&TransformComponent> {
        LayoutParticipant::transform_component(self)
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
    #[cfg(feature = "rive-layout")]
    fn add_layout_style_applier(&mut self, applier: &mut dyn LayoutStyleApplier) {
        LayoutParticipant::add_layout_style_applier(self, applier);
    }
    fn num_layout_nodes(&self) -> usize {
        LayoutParticipant::num_layout_nodes(self)
    }
    #[cfg(feature = "rive-layout")]
    fn cascade_layout_style(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<&mut KeyFrameInterpolator>,
        time: f32,
        direction: LayoutDirection,
    ) -> bool {
        LayoutParticipant::cascade_layout_style(self, interpolation, interpolator, time, direction)
    }
}
