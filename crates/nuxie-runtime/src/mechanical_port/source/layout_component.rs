use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponent,
    animation::keyframe_interpolator::KeyFrameInterpolator,
    artboard::Artboard,
    component::Component,
    component_dirt::ComponentDirt,
    core::Core,
    core_context::CoreContext,
    drawable::{Drawable, DrawableProxy, ProxyDrawing},
    generated::layout_component_base::{LayoutComponentBase, LayoutComponentBaseCallbacks},
    hit_info::HitInfo,
    layout::{
        layout_component_style::LayoutComponentStyle,
        layout_data::LayoutData,
        layout_enums::{
            LayoutAnimationStyle, LayoutDirection, LayoutScaleType, LayoutStyleInterpolation,
        },
        layout_measure_mode::LayoutMeasureMode,
        layout_node_provider::{self, LayoutNodeProvider},
        layout_style_applier::{LayoutStyleApplier, LayoutSyncContext},
    },
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    renderer::{RenderPath, Renderer},
    shapes::{paint::shape_paint_path::ShapePaintPath, path::Path},
    status_code::StatusCode,
};

#[cfg(feature = "rive_layout")]
use crate::mechanical_port::source::yoga::{
    YGAlign, YGDimension, YGDirection, YGDisplay, YGFlexDirection, YGFloatOptional, YGPositionType,
    YGStyle, YGUnit, YGValue,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Layout {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl Layout {
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
    pub fn lerp(from: Self, to: Self, factor: f32) -> Self {
        let inverse = 1.0 - factor;
        Self::new(
            to.left * factor + from.left * inverse,
            to.top * factor + from.top * inverse,
            to.width * factor + from.width * inverse,
            to.height * factor + from.height * inverse,
        )
    }
    pub fn left(self) -> f32 {
        self.left
    }
    pub fn top(self) -> f32 {
        self.top
    }
    pub fn width(self) -> f32 {
        self.width
    }
    pub fn height(self) -> f32 {
        self.height
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutPadding {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}
impl LayoutPadding {
    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
    pub fn left(self) -> f32 {
        self.left
    }
    pub fn top(self) -> f32 {
        self.top
    }
    pub fn right(self) -> f32 {
        self.right
    }
    pub fn bottom(self) -> f32 {
        self.bottom
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LayoutAnimationData {
    pub elapsed_seconds: f32,
    pub from: Layout,
    pub to: Layout,
}
impl LayoutAnimationData {
    pub fn interpolate(&self, factor: f32) -> Layout {
        Layout::lerp(self.from, self.to, factor)
    }
    pub fn copy(&mut self, source: &Self) {
        self.from = source.from;
        self.to = source.to;
        self.elapsed_seconds = source.elapsed_seconds;
    }
}

pub struct LayoutComponent {
    pub base: LayoutComponentBase,
    style: Option<*mut LayoutComponentStyle>,
    layout_data: *mut LayoutData,
    layout: Layout,
    layout_padding: LayoutPadding,
    animation_data_a: LayoutAnimationData,
    animation_data_b: LayoutAnimationData,
    is_smoothing_animation: bool,
    inherited_interpolator: Option<*mut KeyFrameInterpolator>,
    inherited_interpolation: LayoutStyleInterpolation,
    inherited_interpolation_time: f32,
    inherited_direction: LayoutDirection,
    background_raw_path: RawPath,
    local_path: ShapePaintPath,
    world_path: ShapePaintPath,
    proxy: Option<*mut DrawableProxy>,
    just_added_to_host: bool,
    width_override: f32,
    width_unit_value_override: i32,
    height_override: f32,
    height_unit_value_override: i32,
    parent_is_row: bool,
    width_intrinsically_size_override: bool,
    height_intrinsically_size_override: bool,
    forced_width: f32,
    forced_height: f32,
    force_update_layout_bounds: bool,
    position_left_changed: bool,
    position_top_changed: bool,
    has_foreground_drawable: bool,
}

impl Default for LayoutComponent {
    fn default() -> Self {
        let layout_data = Box::into_raw(Box::new(LayoutData::default()));
        let mut value = Self {
            base: LayoutComponentBase::default(),
            style: None,
            layout_data,
            layout: Layout::default(),
            layout_padding: LayoutPadding::default(),
            animation_data_a: LayoutAnimationData::default(),
            animation_data_b: LayoutAnimationData::default(),
            is_smoothing_animation: false,
            inherited_interpolator: None,
            inherited_interpolation: LayoutStyleInterpolation::Hold,
            inherited_interpolation_time: 0.0,
            inherited_direction: LayoutDirection::Inherit,
            background_raw_path: RawPath::default(),
            local_path: ShapePaintPath::default(),
            world_path: ShapePaintPath::default(),
            proxy: None,
            just_added_to_host: false,
            width_override: f32::NAN,
            width_unit_value_override: -1,
            height_override: f32::NAN,
            height_unit_value_override: -1,
            parent_is_row: true,
            width_intrinsically_size_override: false,
            height_intrinsically_size_override: false,
            forced_width: f32::NAN,
            forced_height: f32::NAN,
            force_update_layout_bounds: false,
            position_left_changed: true,
            position_top_changed: true,
            has_foreground_drawable: false,
        };
        value.proxy = Some(Box::into_raw(Box::new(DrawableProxy::new(Box::new(
            LayoutProxy { owner: &mut value },
        )))));
        #[cfg(feature = "rive_layout")]
        unsafe {
            (*layout_data).node.config_mut().set_point_scale_factor(0.0);
        }
        value
    }
}

struct LayoutProxy {
    owner: *mut LayoutComponent,
}
impl ProxyDrawing for LayoutProxy {
    fn draw_proxy(&mut self, renderer: &mut dyn Renderer) {
        unsafe { &mut *self.owner }.draw_proxy(renderer);
    }
    fn is_proxy_hidden(&self) -> bool {
        unsafe { &*self.owner }.is_hidden()
    }
    fn as_layout_component_mut(&mut self) -> &mut LayoutComponent {
        unsafe { &mut *self.owner }
    }
}

impl LayoutComponent {
    fn layout_parent(&mut self) -> Option<&mut LayoutComponent> {
        let mut parent = self.base.base.base.base.base.parent_mut();
        while let Some(value) = parent {
            if let Some(layout) = value.as_layout_component_mut() {
                return Some(layout);
            }
            parent = value.base.base.parent_mut();
        }
        None
    }
    fn origin_child(
        &self,
    ) -> Option<&crate::mechanical_port::source::component_origin::ComponentOrigin> {
        self.base
            .base
            .base
            .base
            .base
            .children()
            .iter()
            .find_map(|child| unsafe { (&**child).as_component_origin() })
    }
    pub fn pivot_origin_x(&self) -> f32 {
        self.origin_child()
            .map_or(0.0, |origin| origin.base.origin_x())
    }
    pub fn pivot_origin_y(&self) -> f32 {
        self.origin_child()
            .map_or(0.0, |origin| origin.base.origin_y())
    }
    pub fn shape_world_transform(&self) -> Mat2D {
        self.base.base.base.base.world_transform()
    }
    pub fn get_artboard(&mut self) -> Option<&mut Artboard> {
        self.base.base.base.base.base.artboard_mut()
    }
    pub fn computed_local_x(&self) -> f32 {
        self.layout.left()
    }
    pub fn computed_local_y(&self) -> f32 {
        self.layout.top()
    }
    pub fn computed_width(&self) -> f32 {
        self.layout.width()
    }
    pub fn computed_height(&self) -> f32 {
        self.layout.height()
    }
    pub fn style(&self) -> Option<&LayoutComponentStyle> {
        self.style.map(|value| unsafe { &*value })
    }
    pub fn style_mut(&mut self) -> Option<&mut LayoutComponentStyle> {
        self.style.map(|value| unsafe { &mut *value })
    }
    pub fn set_style(&mut self, style: Option<*mut LayoutComponentStyle>) {
        self.style = style;
    }
    pub fn proxy(&mut self) -> *mut Drawable {
        self.proxy.unwrap().cast()
    }
    pub fn layout(&self) -> Layout {
        self.layout
    }
    pub fn set_layout(&mut self, left: f32, top: f32, width: f32, height: f32) {
        self.layout = Layout::new(left, top, width, height);
    }
    pub fn x(&self) -> f32 {
        self.layout.left()
    }
    pub fn y(&self) -> f32 {
        self.layout.top()
    }
    pub fn layout_x(&self) -> f32 {
        self.layout.left()
    }
    pub fn layout_y(&self) -> f32 {
        self.layout.top()
    }
    pub fn layout_width(&self) -> f32 {
        self.layout.width()
    }
    pub fn layout_height(&self) -> f32 {
        self.layout.height()
    }
    pub fn inner_width(&self) -> f32 {
        self.layout.width() - self.layout_padding.left() - self.layout_padding.right()
    }
    pub fn inner_height(&self) -> f32 {
        self.layout.height() - self.layout_padding.top() - self.layout_padding.bottom()
    }
    pub fn padding_left(&self) -> f32 {
        self.layout_padding.left()
    }
    pub fn padding_right(&self) -> f32 {
        self.layout_padding.right()
    }
    pub fn padding_top(&self) -> f32 {
        self.layout_padding.top()
    }
    pub fn padding_bottom(&self) -> f32 {
        self.layout_padding.bottom()
    }
    pub fn layout_bounds(&self) -> Aabb {
        Aabb::from_ltwh(
            self.layout.left(),
            self.layout.top(),
            self.layout.width(),
            self.layout.height(),
        )
    }
    pub fn constraint_bounds(&self) -> Aabb {
        self.local_bounds()
    }
    pub fn local_bounds(&self) -> Aabb {
        Aabb::from_ltwh(0.0, 0.0, self.layout.width(), self.layout.height())
    }
    pub fn world_bounds(&self) -> Aabb {
        let transform = self.base.base.base.base.world_transform();
        Aabb::from_ltwh(
            transform.tx(),
            transform.ty(),
            self.layout.width(),
            self.layout.height(),
        )
    }
    pub fn num_layout_nodes(&self) -> usize {
        1
    }
    pub fn forced_width(&self) -> f32 {
        self.forced_width
    }
    pub fn forced_height(&self) -> f32 {
        self.forced_height
    }
    pub fn can_have_overrides(&self) -> bool {
        false
    }
    pub fn has_shape_paints(&self) -> bool {
        !self.base.base.base.base.shape_paints().is_empty()
    }
    pub fn register_foreground_drawable(&mut self) {
        self.has_foreground_drawable = true;
    }
    pub fn mark_position_left_changed(&mut self) {
        self.position_left_changed = true;
    }
    pub fn mark_position_top_changed(&mut self) {
        self.position_top_changed = true;
    }

    pub fn build_dependencies(&mut self) {
        self.base.base.base.base.base.build_dependencies();
        if let Some(parent) = self.base.base.base.base.base.parent_mut() {
            parent.add_dependent(self.base.base.base.base.base.as_component_mut());
        }
        let blend = self.base.base.blend_mode();
        for paint in self.base.base.base.base.shape_paints_mut() {
            paint.set_blend_mode(blend);
        }
    }
    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<&mut Core> {
        None
    }
    pub fn hit_test_point(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        primary: bool,
    ) -> bool {
        let inverse = self
            .base
            .base
            .base
            .base
            .world_transform()
            .invert_or_identity();
        if inverse == Mat2D::identity()
            && self.base.base.base.base.world_transform() != Mat2D::identity()
        {
            return false;
        }
        if !(skip_on_unclipped && !self.base.clip()) {
            let mut local = inverse * *position;
            if let Some(artboard) = self.base.base.base.base.base.as_artboard_mut() {
                if artboard.origin_x() != 0.0 || artboard.origin_y() != 0.0 {
                    local += Vec2D::new(
                        artboard.origin_x() * artboard.layout_width(),
                        artboard.origin_y() * artboard.layout_height(),
                    );
                }
            }
            if !self.local_bounds().contains(local) {
                return false;
            }
        }
        self.base.base.hit_test_point(position, true, primary)
    }
    pub fn update(&mut self, value: ComponentDirt) {
        self.base.base.update(value);
        #[cfg(feature = "rive_layout")]
        if value == ComponentDirt::FILTHY {
            self.interrupt_animation();
        }
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            self.base
                .base
                .base
                .base
                .propagate_opacity(self.base.base.base.base.child_opacity());
        }
        if self.base.base.base.base.base.parent().is_some()
            && value.contains(ComponentDirt::WORLD_TRANSFORM)
        {
            let parent = self.base.base.base.base.base.parent_mut().unwrap();
            let parent_world = parent
                .as_world_transform_component_mut()
                .map_or(Mat2D::identity(), |value| value.world_transform());
            let mut location = Vec2D::new(self.layout.left(), self.layout.top());
            if let Some(artboard) = parent.as_artboard_mut() {
                location -= Vec2D::new(
                    artboard.layout_width() * artboard.origin_x(),
                    artboard.layout_height() * artboard.origin_y(),
                );
            }
            let mut slot = Mat2D::from_translation(location);
            if self.rotation() != 0.0 || self.scale_x() != 1.0 || self.scale_y() != 1.0 {
                let mut local = if self.rotation() != 0.0 {
                    Mat2D::from_rotation(self.rotation())
                } else {
                    Mat2D::identity()
                };
                local.scale_by_values(self.scale_x(), self.scale_y());
                let (ox, oy) = (self.pivot_origin_x(), self.pivot_origin_y());
                if ox != 0.0 || oy != 0.0 {
                    let (px, py) = (ox * self.layout_width(), oy * self.layout_height());
                    local = Mat2D::from_translate(px, py) * local * Mat2D::from_translate(-px, -py);
                }
                slot *= local;
            }
            self.base
                .base
                .base
                .base
                .set_world_transform(parent_world * slot);
            self.update_constraints();
        }
        if (value
            & (ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM | ComponentDirt::LAYOUT_STYLE))
            != ComponentDirt::NONE
        {
            self.update_render_path();
        }
        self.position_left_changed = false;
        self.position_top_changed = false;
    }
    pub fn width_override(&mut self, width: f32, unit: i32, row: bool) {
        self.width_override = width;
        self.width_unit_value_override = unit;
        self.parent_is_row = row;
        self.mark_layout_node_dirty(false);
    }
    pub fn height_override(&mut self, height: f32, unit: i32, row: bool) {
        self.height_override = height;
        self.height_unit_value_override = unit;
        self.parent_is_row = row;
        self.mark_layout_node_dirty(false);
    }
    pub fn set_parent_is_row(&mut self, row: bool) {
        self.parent_is_row = row;
        self.mark_layout_node_dirty(false);
    }
    pub fn set_width_intrinsically_size_override(&mut self, intrinsic: bool) {
        self.width_intrinsically_size_override = intrinsic;
        self.width_unit_value_override = if intrinsic { 3 } else { 1 };
        self.mark_layout_node_dirty(false);
    }
    pub fn set_height_intrinsically_size_override(&mut self, intrinsic: bool) {
        self.height_intrinsically_size_override = intrinsic;
        self.height_unit_value_override = if intrinsic { 3 } else { 1 };
        self.mark_layout_node_dirty(false);
    }
    pub fn set_forced_width(&mut self, value: f32) {
        if self.forced_width == value {
            return;
        }
        self.forced_width = value;
        self.mark_layout_style_dirty();
        self.mark_layout_node_dirty(false);
    }
    pub fn set_forced_height(&mut self, value: f32) {
        if self.forced_height == value {
            return;
        }
        self.forced_height = value;
        self.mark_layout_style_dirty();
        self.mark_layout_node_dirty(false);
    }
    pub fn update_constraints(&mut self) {
        for constraint in self.base.base.base.base.layout_constraints() {
            unsafe { &mut **constraint }.constrain_child(self);
        }
        self.base.base.base.base.update_constraints();
    }
    pub fn overrides_keyed_interpolation(&mut self, key: i32) -> bool {
        #[cfg(feature = "rive_layout")]
        if self.animates()
            && matches!(
                key as u16,
                LayoutComponentBase::WIDTH_PROPERTY_KEY | LayoutComponentBase::HEIGHT_PROPERTY_KEY
            )
        {
            return true;
        }
        false
    }
    pub fn is_hidden(&self) -> bool {
        self.base.base.is_hidden() || self.is_collapsed()
    }
    pub fn is_collapsed(&self) -> bool {
        if self.base.base.base.base.base.is_collapsed() {
            return true;
        }
        #[cfg(feature = "rive_layout")]
        {
            return self.style_display_hidden();
        }
        #[cfg(not(feature = "rive_layout"))]
        false
    }
    fn propagate_collapse(&mut self, value: bool) {
        let collapsed = value || self.is_collapsed();
        for child in self.base.base.base.base.base.children() {
            unsafe { &mut **child }.collapse(collapsed);
        }
        self.base.base.base.base.base.update_collapsables();
    }
    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.base.base.base.base.base.collapse(value) {
            return false;
        }
        self.propagate_collapse(value);
        true
    }
    pub fn gap_horizontal(&self) -> f32 {
        #[cfg(feature = "rive_layout")]
        {
            return self.style().map_or(0.0, |style| {
                if style.gap_horizontal_units() == YGUnit::Percent {
                    style.base.gap_horizontal() / 100.0 * self.layout_width()
                } else {
                    style.base.gap_horizontal()
                }
            });
        }
        #[cfg(not(feature = "rive_layout"))]
        0.0
    }
    pub fn gap_vertical(&self) -> f32 {
        #[cfg(feature = "rive_layout")]
        {
            return self.style().map_or(0.0, |style| {
                if style.gap_vertical_units() == YGUnit::Percent {
                    style.base.gap_vertical() / 100.0 * self.layout_height()
                } else {
                    style.base.gap_vertical()
                }
            });
        }
        #[cfg(not(feature = "rive_layout"))]
        0.0
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(style) = context
            .resolve(self.base.style_id())
            .and_then(Core::as_layout_component_style_mut)
        else {
            return StatusCode::MissingObject;
        };
        self.style = Some(style);
        self.base
            .base
            .base
            .base
            .base
            .add_child(style.as_component_mut());
        #[cfg(feature = "rive_layout")]
        {
            self.add_layout_style_applier(self);
            self.add_layout_style_applier(style);
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.mark_layout_style_dirty();
        self.sync_layout_children();
        self.propagate_collapse(self.is_collapsed());
        StatusCode::Ok
    }
    pub fn draw_proxy(&mut self, renderer: &mut dyn Renderer) {
        #[cfg(feature = "rive_layout")]
        {
            if self.base.clip() {
                renderer.save();
                renderer.clip_path(self.world_path.render_path(self));
            }
            let world = self.shape_world_transform();
            for paint in self.base.base.base.base.shape_paints_mut() {
                if paint.should_draw() {
                    if let Some(path) = paint.pick_path(self) {
                        paint.draw(renderer, path, world);
                    }
                }
            }
        }
    }
    pub fn draw(&mut self, renderer: &mut dyn Renderer) {
        #[cfg(feature = "rive_layout")]
        if self.base.clip() {
            renderer.restore();
        }
    }
    pub fn update_render_path(&mut self) {
        #[cfg(feature = "rive_layout")]
        {
            if self.is_hidden()
                || (self.base.base.base.base.shape_paints().is_empty()
                    && !self.base.clip()
                    && !self.has_foreground_drawable)
            {
                return;
            }
            let mut radii = [0.0; 4];
            if let Some(style) = self.style() {
                let ltr = self.actual_direction() != LayoutDirection::Rtl;
                if style.base.link_corner_radius() {
                    radii.fill(style.base.corner_radius_tl());
                } else {
                    radii = if ltr {
                        [
                            style.base.corner_radius_tl(),
                            style.base.corner_radius_tr(),
                            style.base.corner_radius_br(),
                            style.base.corner_radius_bl(),
                        ]
                    } else {
                        [
                            style.base.corner_radius_tr(),
                            style.base.corner_radius_tl(),
                            style.base.corner_radius_bl(),
                            style.base.corner_radius_br(),
                        ]
                    };
                }
            }
            self.background_raw_path.rewind();
            Path::add_rounded_rect(
                &mut self.background_raw_path,
                Aabb::new(0.0, 0.0, self.layout.width(), self.layout.height()),
                radii,
            );
            self.local_path.rewind();
            self.local_path.add_raw_path(&self.background_raw_path);
            self.world_path.rewind_with_rule(
                false,
                crate::mechanical_port::source::shapes::paint::fill_rule::FillRule::Clockwise,
            );
            self.world_path.add_raw_path_with_transform(
                &self.background_raw_path,
                &self.base.base.base.base.world_transform(),
            );
            for paint in self.base.base.base.base.shape_paints_mut() {
                if paint.should_draw() {
                    paint.invalidate_effects();
                }
            }
        }
    }
    pub fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        #[cfg(feature = "rive_layout")]
        {
            let mut size = Vec2D::default();
            for child in self.base.base.base.base.base.children() {
                let child = unsafe { &mut **child };
                if child.as_layout_component_mut().is_none() {
                    if let Some(sizeable) = child.as_intrinsically_sizeable_mut() {
                        let measured =
                            sizeable.measure_layout(width, width_mode, height, height_mode);
                        size = Vec2D::new(size.x.max(measured.x), size.y.max(measured.y));
                    }
                }
            }
            return size;
        }
        #[cfg(not(feature = "rive_layout"))]
        Vec2D::default()
    }
    pub fn effective_parent_is_row(&mut self) -> bool {
        if self.can_have_overrides() {
            self.parent_is_row
        } else {
            self.layout_parent()
                .is_none_or(|parent| parent.main_axis_is_row())
        }
    }
    pub fn main_axis_is_row(&self) -> bool {
        #[cfg(feature = "rive_layout")]
        {
            return self.style().is_none_or(|style| {
                matches!(
                    style.flex_direction(),
                    YGFlexDirection::Row | YGFlexDirection::RowReverse
                )
            });
        }
        #[cfg(not(feature = "rive_layout"))]
        true
    }
    pub fn main_axis_is_column(&self) -> bool {
        #[cfg(feature = "rive_layout")]
        {
            return self.style().is_some_and(|style| {
                matches!(
                    style.flex_direction(),
                    YGFlexDirection::Column | YGFlexDirection::ColumnReverse
                )
            });
        }
        #[cfg(not(feature = "rive_layout"))]
        false
    }
    pub fn layout_node(&mut self, _index: i32) -> Option<*mut ()> {
        (!self.layout_data.is_null())
            .then(|| unsafe { &mut (*self.layout_data).node as *mut _ as *mut () })
    }
    pub fn is_leaf(&mut self) -> bool {
        !self
            .base
            .base
            .base
            .base
            .base
            .children()
            .iter()
            .any(|child| unsafe { (&mut **child).as_layout_node_provider_mut().is_some() })
    }
    pub fn sync_style(&mut self) {
        #[cfg(feature = "rive_layout")]
        {
            if self.style.is_none() || self.layout_data.is_null() {
                return;
            }
            let node = unsafe { &mut (*self.layout_data).node };
            if self.style().unwrap().base.intrinsically_sized() && self.is_leaf() {
                node.set_context(self);
                node.set_measure_func(Some(Self::measure_node));
            } else {
                node.set_measure_func(None);
            }
            let parent = self.layout_parent();
            let parent_style = parent.and_then(|value| value.style());
            let context = LayoutSyncContext {
                parent_is_grid: parent_style.is_some_and(LayoutComponentStyle::is_grid),
                parent_is_stack: parent_style.is_some_and(LayoutComponentStyle::is_stack),
                container_justify_items: parent_style.map_or(YGAlign::Stretch as u32, |value| {
                    value.base.justify_items_value()
                }),
                inline_hugs: self.style().unwrap().width_scale_type() == LayoutScaleType::Hug,
                parent_is_row: self.effective_parent_is_row(),
                is_ltr: self.actual_direction() != LayoutDirection::Rtl,
                has_layout_parent: parent.is_some(),
            };
            unsafe {
                (*self.layout_data).apply_layout_styles(&mut (*self.layout_data).style, &context);
                node.set_style((*self.layout_data).style.clone());
            }
            self.sync_child_provider_styles();
        }
    }
    #[cfg(feature = "rive_layout")]
    fn measure_node(
        node: *mut crate::mechanical_port::source::yoga::YGNode,
        width: f32,
        wm: crate::mechanical_port::source::yoga::YGMeasureMode,
        height: f32,
        hm: crate::mechanical_port::source::yoga::YGMeasureMode,
    ) -> crate::mechanical_port::source::yoga::YGSize {
        let value = unsafe { &mut *((*node).context() as *mut LayoutComponent) }.measure_layout(
            width,
            wm.into(),
            height,
            hm.into(),
        );
        crate::mechanical_port::source::yoga::YGSize::new(value.x, value.y)
    }
    pub fn clear_layout_children(&mut self) {
        #[cfg(feature = "rive_layout")]
        unsafe {
            (*self.layout_data).node.remove_all_children();
        }
    }
    pub fn sync_layout_children(&mut self) {
        #[cfg(feature = "rive_layout")]
        {
            self.clear_layout_children();
            let mut index = 0;
            for child in self.base.base.base.base.base.children() {
                if let Some(provider) = unsafe { &mut **child }.as_layout_node_provider_mut() {
                    for i in 0..provider.num_layout_nodes() {
                        if let Some(node) = provider.layout_node(i as i32) {
                            unsafe {
                                (*self.layout_data).node.insert_child(node.cast(), index);
                            }
                            index += 1;
                        }
                    }
                }
            }
            self.mark_layout_node_dirty(false);
        }
    }
    pub fn propagate_size(&mut self) {
        self.propagate_size_to_children(self.base.base.base.base.base.as_container_component_mut());
    }
    fn propagate_size_to_children(
        &mut self,
        component: &mut crate::mechanical_port::source::container_component::ContainerComponent,
    ) {
        #[cfg(feature = "rive_layout")]
        if !self.is_hidden() {
            for child in component.children() {
                let child = unsafe { &mut **child };
                if child.as_layout_component_mut().is_some()
                    || child.is_transparent_layout_container()
                    || child.as_layout_node_provider_mut().is_some()
                {
                    continue;
                }
                if let Some(sizeable) = child.as_intrinsically_sizeable_mut() {
                    if let Some(style) = self.style() {
                        sizeable.control_size(
                            Vec2D::new(self.layout.width(), self.layout.height()),
                            style.width_scale_type(),
                            style.height_scale_type(),
                            self.actual_direction(),
                        );
                        if !sizeable.should_propagate_size_to_children() {
                            continue;
                        }
                    }
                }
                if let Some(container) = child.as_container_component_mut() {
                    self.propagate_size_to_children(container);
                }
            }
        }
    }
    pub fn calculate_layout_internal(&mut self, available_width: f32, available_height: f32) {
        #[cfg(feature = "rive_layout")]
        {
            let width = if available_width.is_nan()
                && self.style().is_some_and(|s| s.base.intrinsically_sized())
            {
                available_width
            } else {
                self.base.width()
            };
            let height = if available_height.is_nan()
                && self.style().is_some_and(|s| s.base.intrinsically_sized())
            {
                available_height
            } else {
                self.base.height()
            };
            unsafe {
                (*self.layout_data)
                    .node
                    .calculate_layout(width, height, YGDirection::Inherit);
            }
        }
    }
    pub fn style_display_hidden(&self) -> bool {
        self.style()
            .is_some_and(|style| style.display() == YGDisplay::None)
    }
    pub fn actual_direction(&self) -> LayoutDirection {
        self.style()
            .map_or(self.inherited_direction, |style| match style.direction() {
                YGDirection::Ltr => LayoutDirection::Ltr,
                YGDirection::Rtl => LayoutDirection::Rtl,
                _ => self.inherited_direction,
            })
    }
    pub fn on_dirty(&mut self, value: ComponentDirt) {
        self.base.base.base.base.base.on_dirty(value);
        if value.contains(ComponentDirt::WORLD_TRANSFORM) && self.base.clip() {
            self.base
                .base
                .base
                .base
                .base
                .add_dirt(ComponentDirt::PATH, false);
        }
    }
    pub fn update_layout_bounds(&mut self, animate: bool) {
        #[cfg(feature = "rive_layout")]
        {
            let node = unsafe { &mut (*self.layout_data).node };
            if !node.has_new_layout() {
                return;
            }
            node.set_has_new_layout(false);
            let next = Layout::new(
                node.layout_left(),
                node.layout_top(),
                node.layout_width(),
                node.layout_height(),
            );
            self.layout_padding = LayoutPadding::new(
                node.padding_left(),
                node.padding_top(),
                node.padding_right(),
                node.padding_bottom(),
            );
            if self.just_added_to_host {
                self.just_added_to_host = false;
                self.layout = next;
                let data = self.current_animation_data();
                data.from = next;
                data.to = next;
                data.elapsed_seconds = 0.0;
                self.propagate_size();
                self.mark_world_transform_dirty();
                self.force_update_layout_bounds = false;
                return;
            }
            if animate && self.animates() {
                let force = self.force_update_layout_bounds;
                let data = self.current_animation_data();
                if next != data.to || force {
                    self.is_smoothing_animation = data.elapsed_seconds != 0.0;
                    let from = self.layout;
                    let data = self.current_animation_data();
                    data.from = from;
                    data.to = next;
                    data.elapsed_seconds = 0.0;
                    self.propagate_size();
                    self.mark_world_transform_dirty();
                }
            } else if next != self.layout || self.force_update_layout_bounds {
                if self.layout.width() != next.width() || self.layout.height() != next.height() {
                    self.base
                        .base
                        .base
                        .base
                        .base
                        .add_dirt(ComponentDirt::PATH, false);
                }
                self.layout = next;
                self.animation_data_a.to = next;
                self.propagate_size();
                self.mark_world_transform_dirty();
            }
            self.force_update_layout_bounds = false;
        }
    }
    pub fn animates(&self) -> bool {
        self.style().is_some_and(|style| {
            style.animation_style() != LayoutAnimationStyle::None
                && self.interpolation() != LayoutStyleInterpolation::Hold
                && self.interpolation_time() > 0.0
        })
    }
    pub fn animation_style(&self) -> LayoutAnimationStyle {
        self.style().map_or(
            LayoutAnimationStyle::None,
            LayoutComponentStyle::animation_style,
        )
    }
    pub fn interpolator(&self) -> Option<&mut KeyFrameInterpolator> {
        let style = self.style()?;
        match style.animation_style() {
            LayoutAnimationStyle::Inherit => {
                self.inherited_interpolator.or(style.interpolator_ptr())
            }
            LayoutAnimationStyle::Custom => style.interpolator_ptr(),
            _ => None,
        }
        .map(|value| unsafe { &mut *value })
    }
    pub fn interpolation(&self) -> LayoutStyleInterpolation {
        self.style()
            .map_or(LayoutStyleInterpolation::Hold, |style| {
                match style.animation_style() {
                    LayoutAnimationStyle::Inherit => self.inherited_interpolation,
                    LayoutAnimationStyle::Custom => style.interpolation(),
                    _ => LayoutStyleInterpolation::Hold,
                }
            })
    }
    pub fn interpolation_time(&self) -> f32 {
        self.style()
            .map_or(0.0, |style| match style.animation_style() {
                LayoutAnimationStyle::Inherit => self.inherited_interpolation_time,
                LayoutAnimationStyle::Custom => style.base.interpolation_time(),
                _ => 0.0,
            })
    }
    fn current_animation_data(&mut self) -> &mut LayoutAnimationData {
        if self.is_smoothing_animation {
            &mut self.animation_data_b
        } else {
            &mut self.animation_data_a
        }
    }
    pub fn apply_interpolation(&mut self, elapsed: f32, animate: bool) -> bool {
        if !animate || !self.animates() || self.current_animation_data().to == self.layout {
            return false;
        }
        let time = self.interpolation_time();
        let data = self.current_animation_data();
        if data.elapsed_seconds >= time {
            self.layout = data.to;
            data.elapsed_seconds = 0.0;
            self.propagate_size();
            self.mark_world_transform_dirty();
            return false;
        }
        let mut factor = (data.elapsed_seconds / time).min(1.0);
        if self.interpolation() != LayoutStyleInterpolation::Linear {
            if let Some(interpolator) = self.interpolator() {
                factor = interpolator.transform(factor);
            }
        }
        let current = self.current_animation_data().interpolate(factor);
        if self.layout != current {
            let resized =
                self.layout.width() != current.width() || self.layout.height() != current.height();
            self.layout = current;
            if resized {
                self.propagate_size();
            }
            self.mark_world_transform_dirty();
        }
        self.current_animation_data().elapsed_seconds += elapsed;
        if factor != 1.0 {
            self.mark_layout_node_dirty(false);
            true
        } else {
            false
        }
    }
    pub fn advance_component(&mut self, elapsed: f32, flags: AdvanceFlags) -> bool {
        #[cfg(feature = "rive_layout")]
        {
            if flags.0 & AdvanceFlags::NEW_FRAME.0 == 0 || self.is_collapsed() {
                return false;
            }
            return self.apply_interpolation(
                elapsed,
                flags.0 & (AdvanceFlags::ANIMATE.0 | AdvanceFlags::ADVANCE_NESTED.0) != 0,
            );
        }
        #[cfg(not(feature = "rive_layout"))]
        false
    }
    pub fn interrupt_animation(&mut self) {
        if self.animates() {
            self.layout = self.current_animation_data().to;
            self.propagate_size();
        }
    }
    pub fn mark_layout_node_dirty(&mut self, force: bool) {
        #[cfg(feature = "rive_layout")]
        {
            self.force_update_layout_bounds |= force;
            unsafe {
                (*self.layout_data).node.mark_dirty_and_propagate();
            }
            if let Some(artboard) = self.base.base.base.base.base.artboard_mut() {
                artboard.mark_layout_dirty(self);
            }
        }
    }
    pub fn mark_layout_style_dirty(&mut self) {
        #[cfg(feature = "rive_layout")]
        {
            self.clear_inherited_interpolation();
            self.base
                .base
                .base
                .base
                .base
                .add_dirt(ComponentDirt::LAYOUT_STYLE, false);
            if let Some(artboard) = self.base.base.base.base.base.artboard_mut() {
                if !std::ptr::eq(artboard as *mut _, self as *mut _ as *mut _) {
                    artboard.mark_layout_style_dirty();
                }
            }
        }
    }
    pub fn set_inherited_interpolation(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<*mut KeyFrameInterpolator>,
        time: f32,
    ) -> bool {
        if (interpolation, interpolator, time)
            == (
                self.inherited_interpolation,
                self.inherited_interpolator,
                self.inherited_interpolation_time,
            )
        {
            return false;
        }
        self.inherited_interpolation = interpolation;
        self.inherited_interpolator = interpolator;
        self.inherited_interpolation_time = time;
        true
    }
    pub fn clear_inherited_interpolation(&mut self) {
        self.inherited_interpolation = LayoutStyleInterpolation::Hold;
        self.inherited_interpolator = None;
        self.inherited_interpolation_time = 0.0;
    }
    pub fn cascade_layout_style(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<*mut KeyFrameInterpolator>,
        time: f32,
        direction: LayoutDirection,
    ) -> bool {
        let mut updated = if self
            .style()
            .is_some_and(|s| s.animation_style() == LayoutAnimationStyle::Inherit)
        {
            self.set_inherited_interpolation(interpolation, interpolator, time)
        } else {
            self.clear_inherited_interpolation();
            false
        };
        let old = self.inherited_direction;
        self.inherited_direction = if direction == LayoutDirection::Inherit
            || self
                .style()
                .is_some_and(|s| s.direction() != YGDirection::Inherit)
        {
            LayoutDirection::Inherit
        } else {
            direction
        };
        if old != self.inherited_direction {
            self.mark_layout_node_dirty(true);
            self.base
                .base
                .base
                .base
                .base
                .add_dirt(ComponentDirt::PATH, false);
            updated = true;
        }
        updated
    }
    pub fn sync_child_provider_styles(&mut self) {
        for child in self.base.base.base.base.base.children() {
            if let Some(provider) = unsafe { &mut **child }.as_layout_node_provider_mut() {
                provider.sync_style_changes();
                provider.mark_layout_node_dirty(false);
            }
        }
    }
    pub fn position_type_changed(&mut self) {
        if let Some(style) = self.style_mut() {
            if style.position_type() == YGPositionType::Absolute {
                if !self.position_left_changed {
                    style.base.set_position_left(self.layout.left());
                }
                if !self.position_top_changed {
                    style.base.set_position_top(self.layout.top());
                }
                style.set_absolute_point_units();
            } else {
                style.clear_position();
            }
            self.mark_layout_node_dirty(false);
        }
    }
    pub fn scale_type_changed(&mut self) {
        if let Some(style) = self.style_mut() {
            style.base.set_intrinsically_sized_value(
                style.width_scale_type() == LayoutScaleType::Hug
                    || style.height_scale_type() == LayoutScaleType::Hug,
            );
            self.mark_layout_node_dirty(false);
        }
    }
    pub fn display_changed(&mut self) {
        if self.style.is_some() {
            self.propagate_collapse(self.is_collapsed());
            self.mark_layout_node_dirty(false);
        }
    }
    pub fn flex_direction_changed(&mut self) {
        self.mark_layout_node_dirty(false);
        self.sync_child_provider_styles();
    }
    pub fn layout_type_changed(&mut self) {
        self.mark_layout_node_dirty(false);
        self.sync_child_provider_styles();
    }
    pub fn direction_changed(&mut self) {
        self.mark_layout_style_dirty();
        self.mark_layout_node_dirty(true);
    }
    pub fn clip_changed(&mut self) {
        self.mark_layout_node_dirty(false);
        self.base
            .base
            .base
            .base
            .base
            .add_dirt(ComponentDirt::PATH, false);
    }
    pub fn width_changed(&mut self) {
        self.mark_layout_node_dirty(false);
    }
    pub fn height_changed(&mut self) {
        self.mark_layout_node_dirty(false);
    }
    pub fn style_id_changed(&mut self) {
        self.mark_layout_node_dirty(false);
    }
    pub fn fractional_width_changed(&mut self) {
        self.mark_layout_node_dirty(false);
    }
    pub fn fractional_height_changed(&mut self) {
        self.mark_layout_node_dirty(false);
    }
    pub fn world_path(&mut self) -> &mut ShapePaintPath {
        &mut self.world_path
    }
    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        &mut self.local_path
    }
    pub fn local_clockwise_path(&mut self) -> &mut ShapePaintPath {
        &mut self.local_path
    }
    pub fn path_builder(&mut self) -> &mut Component {
        self.base.base.base.base.base.as_component_mut()
    }
    pub fn mark_world_transform_dirty(&mut self) {
        self.base
            .base
            .base
            .base
            .base
            .add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
    }
    pub fn rotation(&self) -> f32 {
        self.base.base.base.base.rotation()
    }
    pub fn scale_x(&self) -> f32 {
        self.base.base.base.base.scale_x()
    }
    pub fn scale_y(&self) -> f32 {
        self.base.base.base.base.scale_y()
    }
    pub fn add_layout_style_applier(&mut self, applier: *mut dyn LayoutStyleApplier) {
        #[cfg(feature = "rive_layout")]
        if !self.layout_data.is_null() {
            unsafe {
                (*self.layout_data).add_applier(applier);
            }
        }
    }
    #[cfg(feature = "rive_layout")]
    pub fn apply_container_style(&mut self, style: &mut YGStyle, context: &LayoutSyncContext) {
        let Some(component_style) = self.style() else {
            return;
        };
        if component_style.is_stack() {
            return;
        }
        crate::mechanical_port::source::layout::grid_track::GridTrack::sync_container_style(
            style,
            self,
            component_style.base.justify_items_value(),
        );
    }
    #[cfg(feature = "rive_layout")]
    pub fn apply_base_style(&mut self, style: &mut YGStyle, context: &LayoutSyncContext) {
        let Some(component_style) = self.style() else {
            return;
        };
        let absolute = component_style.position_type() == YGPositionType::Absolute;
        let legacy_hug = component_style.width_scale_type() == LayoutScaleType::Fixed
            && component_style.height_scale_type() == LayoutScaleType::Fixed
            && component_style.base.intrinsically_sized()
            && self.is_leaf();
        let units = |scale, stored| {
            if absolute && scale != LayoutScaleType::Hug {
                stored
            } else if scale != LayoutScaleType::Fixed {
                YGUnit::Auto
            } else if matches!(stored, YGUnit::Point | YGUnit::Percent) {
                stored
            } else if legacy_hug {
                YGUnit::Auto
            } else {
                YGUnit::Point
            }
        };
        let mut width = self.base.width();
        let mut height = self.base.height();
        let mut width_scale = component_style.width_scale_type();
        let mut height_scale = component_style.height_scale_type();
        let mut width_units = units(width_scale, component_style.width_units());
        let mut height_units = units(height_scale, component_style.height_units());
        if self.can_have_overrides() {
            if !self.width_override.is_nan() {
                width = self.width_override;
            }
            if !self.height_override.is_nan() {
                height = self.height_override;
            }
            if self.width_unit_value_override != -1 {
                width_units = YGUnit::from(self.width_unit_value_override);
                width_scale = if width_units == YGUnit::Auto {
                    if self.width_intrinsically_size_override {
                        LayoutScaleType::Hug
                    } else {
                        LayoutScaleType::Fill
                    }
                } else {
                    LayoutScaleType::Fixed
                };
            }
            if self.height_unit_value_override != -1 {
                height_units = YGUnit::from(self.height_unit_value_override);
                height_scale = if height_units == YGUnit::Auto {
                    if self.height_intrinsically_size_override {
                        LayoutScaleType::Hug
                    } else {
                        LayoutScaleType::Fill
                    }
                } else {
                    LayoutScaleType::Fixed
                };
            }
        }
        style.dimensions_mut()[YGDimension::Width] = YGValue::new(
            if self.forced_width.is_nan() {
                width.max(0.0)
            } else {
                self.forced_width.max(0.0)
            },
            if self.forced_width.is_nan() {
                width_units
            } else {
                YGUnit::Point
            },
        );
        style.dimensions_mut()[YGDimension::Height] = YGValue::new(
            if self.forced_height.is_nan() {
                height.max(0.0)
            } else {
                self.forced_height.max(0.0)
            },
            if self.forced_height.is_nan() {
                height_units
            } else {
                YGUnit::Point
            },
        );
        if context.parent_is_grid {
            style.set_flex_grow(YGFloatOptional::new(0.0));
            style.set_flex_shrink(YGFloatOptional::new(0.0));
            style.set_align_self(if height_scale == LayoutScaleType::Fill {
                YGAlign::Stretch
            } else {
                YGAlign::Auto
            });
        } else {
            let main_scale = if context.parent_is_row {
                width_scale
            } else {
                height_scale
            };
            let fraction = if context.parent_is_row {
                self.base.fractional_width()
            } else {
                self.base.fractional_height()
            };
            match main_scale {
                LayoutScaleType::Fill => {
                    style.set_flex_grow(YGFloatOptional::new(fraction));
                    style.set_flex_shrink(YGFloatOptional::new(fraction));
                }
                _ => {
                    style.set_flex_grow(YGFloatOptional::new(0.0));
                    style.set_flex_shrink(YGFloatOptional::new(0.0));
                }
            }
            let cross_scale = if context.parent_is_row {
                height_scale
            } else {
                width_scale
            };
            style.set_align_self(if cross_scale == LayoutScaleType::Fill {
                YGAlign::Stretch
            } else {
                YGAlign::Auto
            });
        }
    }
}

impl LayoutComponentBaseCallbacks for LayoutComponent {
    fn notify_property_changed(&mut self, key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(key);
    }
    fn clip_changed(&mut self) {
        LayoutComponent::clip_changed(self);
    }
    fn width_changed(&mut self) {
        LayoutComponent::width_changed(self);
    }
    fn height_changed(&mut self) {
        LayoutComponent::height_changed(self);
    }
    fn style_id_changed(&mut self) {
        LayoutComponent::style_id_changed(self);
    }
    fn fractional_width_changed(&mut self) {
        LayoutComponent::fractional_width_changed(self);
    }
    fn fractional_height_changed(&mut self) {
        LayoutComponent::fractional_height_changed(self);
    }
}
impl AdvancingComponent for LayoutComponent {
    fn advance_component(&mut self, elapsed: f32, flags: AdvanceFlags) -> bool {
        LayoutComponent::advance_component(self, elapsed, flags)
    }
}
impl Drop for LayoutComponent {
    fn drop(&mut self) {
        if let Some(artboard) = self.base.base.base.base.base.artboard_mut() {
            artboard.clean_layout(self);
        }
        if let Some(proxy) = self.proxy.take() {
            unsafe {
                drop(Box::from_raw(proxy));
            }
        }
        if !self.layout_data.is_null() {
            unsafe {
                drop(Box::from_raw(self.layout_data));
            }
        }
    }
}
