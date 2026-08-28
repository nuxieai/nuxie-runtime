use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponent,
    artboard::Artboard,
    component::Component,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    drawable::{Drawable, DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence},
    generated::{
        core_registry::CoreCapabilities,
        layout_component_base::{LayoutComponentBase, LayoutComponentBaseCallbacks},
    },
    hit_info::HitInfo,
    layout::{
        layout_component_style::LayoutComponentStyle,
        layout_data::LayoutData,
        layout_enums::{
            LayoutAnimationStyle, LayoutDirection, LayoutScaleType, LayoutStyleInterpolation,
        },
        layout_measure_mode::LayoutMeasureMode,
        layout_node_provider::{LayoutNodeKey, LayoutNodeProvider, LayoutNodeProviderState},
        layout_style_applier::{
            LayoutStyleApplier, LayoutSyncContext, YGAlign, YGDimension, YGDirection, YGDisplay,
            YGFlexDirection, YGFloatOptional, YGPositionType, YGStyle, YGUnit, YGValue,
        },
    },
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    renderer::{RenderPath, Renderer},
    shapes::{
        paint::{shape_paint::ShapePaintPathKind, shape_paint_path::ShapePaintPath},
        path::Path,
        shape_paint_container::ShapePaintContainer,
    },
    status_code::StatusCode,
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
    pub fn copy_from(&mut self, source: &Self) {
        self.copy(source);
    }
}

pub struct LayoutComponent {
    pub base: LayoutComponentBase,
    paints: ShapePaintContainer,
    provider: LayoutNodeProviderState,
    style: Option<CoreHandle>,
    layout_data: Box<LayoutData>,
    layout: Layout,
    layout_padding: LayoutPadding,
    animation_data_a: LayoutAnimationData,
    animation_data_b: LayoutAnimationData,
    is_smoothing_animation: bool,
    inherited_interpolator: Option<CoreHandle>,
    inherited_interpolation: LayoutStyleInterpolation,
    inherited_interpolation_time: f32,
    inherited_direction: LayoutDirection,
    background_raw_path: RawPath,
    local_path: ShapePaintPath,
    world_path: ShapePaintPath,
    proxy: Option<Rc<RefCell<DrawableProxy>>>,
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
        Self {
            base: LayoutComponentBase::default(),
            paints: ShapePaintContainer::default(),
            provider: LayoutNodeProviderState::default(),
            style: None,
            layout_data: Box::new(LayoutData::default()),
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
        }
    }
}

struct LayoutProxy {
    owner: CoreHandle,
}
impl ProxyDrawing for LayoutProxy {
    fn draw_proxy(&mut self, renderer: &mut Renderer, _needs_save_operation: bool) {
        self.owner.with_mut(|owner| {
            if let Some(owner) = owner.as_layout_component_mut() {
                owner.draw_proxy(renderer);
            }
        });
    }
    fn is_proxy_hidden(&self) -> bool {
        self.owner
            .with(|owner| owner.as_drawable().is_none_or(Drawable::is_hidden))
            .unwrap_or(true)
    }
    fn owner_handle(&self) -> CoreHandle {
        self.owner.clone()
    }
}

impl LayoutComponent {
    fn layout_parent_handle(&self) -> Option<CoreHandle> {
        let mut parent = self.base.base.base.base.base.parent_handle();
        while let Some(value) = parent {
            if value
                .with(|value| value.as_layout_component().is_some())
                .unwrap_or(false)
            {
                return Some(value);
            }
            parent = value
                .with(|value| value.component_parent_handle())
                .flatten();
        }
        None
    }
    fn origin(&self) -> Option<(f32, f32)> {
        self.base
            .base
            .base
            .base
            .base
            .children()
            .iter()
            .find_map(|child| {
                child.with_downcast::<
                    crate::mechanical_port::source::component_origin::ComponentOrigin,
                    _,
                >(|origin| (origin.base.origin_x(), origin.base.origin_y()))
            })
    }
    pub fn pivot_origin_x(&self) -> f32 {
        self.origin().map_or(0.0, |origin| origin.0)
    }
    pub fn pivot_origin_y(&self) -> f32 {
        self.origin().map_or(0.0, |origin| origin.1)
    }
    pub fn shape_world_transform(&self) -> Mat2D {
        *self.base.base.base.base.world_transform()
    }
    pub fn artboard_handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.artboard_handle()
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
    pub fn style_handle(&self) -> Option<CoreHandle> {
        self.style.clone()
    }
    pub fn with_style<R>(&self, f: impl FnOnce(&LayoutComponentStyle) -> R) -> Option<R> {
        self.style
            .as_ref()?
            .with_downcast::<LayoutComponentStyle, _>(f)
    }
    pub fn with_style_mut<R>(&self, f: impl FnOnce(&mut LayoutComponentStyle) -> R) -> Option<R> {
        self.style
            .as_ref()?
            .with_downcast_mut::<LayoutComponentStyle, _>(f)
    }
    pub fn set_style(&mut self, style: Option<CoreHandle>) {
        self.style = style;
    }
    pub fn proxy(&mut self) -> Option<RuntimeDrawableOccurrence> {
        if self.proxy.is_none() {
            let owner = self.base.base.base.base.base.handle()?;
            self.proxy = Some(Rc::new(RefCell::new(DrawableProxy::new(Box::new(
                LayoutProxy { owner },
            )))));
        }
        self.proxy
            .as_ref()
            .cloned()
            .map(RuntimeDrawableOccurrence::runtime_proxy)
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
        !self.paints.shape_paints().is_empty()
    }
    pub fn shape_paint_container(&self) -> &ShapePaintContainer {
        &self.paints
    }
    pub fn shape_paint_container_mut(&mut self) -> &mut ShapePaintContainer {
        &mut self.paints
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
        if let (Some(parent), Some(this)) = (
            self.base.base.base.base.base.parent_handle(),
            self.base.base.base.base.base.handle(),
        ) {
            parent.with_mut(|parent| parent.component_add_dependent(this));
        }
        let blend = self.base.base.blend_mode();
        for paint in self.paints.shape_paints().iter().cloned() {
            paint.with_mut(|paint| {
                if let Some(paint) = paint.as_shape_paint_mut() {
                    paint.blend_mode(blend);
                }
            });
        }
    }
    pub fn hit_test(&mut self, _info: &mut HitInfo, _transform: &Mat2D) -> Option<CoreHandle> {
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
    /// Execute the local part of `LayoutComponent::update` after the complete
    /// TransformComponent super call. Returning true requests the pinned
    /// Layout-then-Transform constraint pass after this CoreHandle borrow ends.
    pub(crate) fn update_after_transform_super(
        &mut self,
        value: ComponentDirt,
        child_opacity: f32,
    ) -> bool {
        if value == ComponentDirt::FILTHY {
            self.interrupt_animation();
        }
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            self.paints.propagate_opacity(child_opacity);
        }
        let needs_layout_constraints = self.base.base.base.base.base.parent_handle().is_some()
            && value.contains(ComponentDirt::WORLD_TRANSFORM);
        if needs_layout_constraints {
            let parent = self
                .base
                .base
                .base
                .base
                .base
                .parent_handle()
                .expect("checked parent");
            let (parent_world, artboard_origin) = parent
                .with(|parent| {
                    let world = parent
                        .as_world_transform_component()
                        .map_or(Mat2D::identity(), |value| *value.world_transform());
                    let origin = parent.as_artboard().map(|artboard| {
                        Vec2D::new(
                            artboard.layout_width() * artboard.origin_x(),
                            artboard.layout_height() * artboard.origin_y(),
                        )
                    });
                    (world, origin)
                })
                .unwrap_or((Mat2D::identity(), None));
            let mut location = Vec2D::new(self.layout.left(), self.layout.top());
            if let Some(origin) = artboard_origin {
                location -= origin;
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
        }
        needs_layout_constraints
    }

    /// Called after the most-derived render-path update, preserving the pinned
    /// virtual-call boundary before resetting the position flags.
    pub(crate) fn reset_update_flags(&mut self) {
        self.position_left_changed = false;
        self.position_top_changed = false;
    }

    pub(crate) fn layout_constraint_handles(&self) -> Vec<CoreHandle> {
        self.provider.layout_constraints().to_vec()
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
    pub fn overrides_keyed_interpolation(&mut self, key: i32) -> bool {
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
        self.style_display_hidden()
    }
    pub(crate) fn collapse_after_component(&mut self, value: bool) {
        let collapsed = value || self.is_collapsed();
        for child in self.base.base.base.base.base.children() {
            child.with_mut(|child| {
                child.component_collapse(collapsed);
            });
        }
        self.base.base.base.base.base.update_collapsables();
    }
    pub fn collapse(&mut self, value: bool) -> bool {
        CoreCapabilities::component_collapse(self, value)
    }
    pub fn gap_horizontal(&self) -> f32 {
        self.with_style(|style| {
            if style.gap_horizontal_units() == YGUnit::Percent {
                style.base.gap_horizontal() / 100.0 * self.layout_width()
            } else {
                style.base.gap_horizontal()
            }
        })
        .unwrap_or(0.0)
    }
    pub fn gap_vertical(&self) -> f32 {
        self.with_style(|style| {
            if style.gap_vertical_units() == YGUnit::Percent {
                style.base.gap_vertical() / 100.0 * self.layout_height()
            } else {
                style.base.gap_vertical()
            }
        })
        .unwrap_or(0.0)
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(style) = context.resolve(self.base.style_id()).filter(|style| {
            style
                .with_downcast::<LayoutComponentStyle, _>(|_| ())
                .is_some()
        }) else {
            return StatusCode::MissingObject;
        };
        self.style = Some(style.clone());
        let Some(this) = self.base.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        self.add_layout_style_applier(this);
        self.add_layout_style_applier(style);
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.mark_layout_style_dirty();
        self.sync_layout_children();
        self.collapse_after_component(self.is_collapsed());
        StatusCode::Ok
    }
    pub fn draw_proxy(&mut self, renderer: &mut Renderer) {
        if self.base.clip() {
            renderer.save();
            let factory = self
                .with_artboard(|artboard| artboard.factory())
                .flatten()
                .expect("a drawable LayoutComponent has its imported factory");
            renderer.clip_path(self.world_path.render_path(&factory));
        }
        let world = self.shape_world_transform();
        for paint in self.paints.shape_paints().to_vec() {
            paint.with_mut(|paint| {
                let Some(paint) = paint.as_shape_paint_behavior_mut() else {
                    return;
                };
                if !paint.should_draw() {
                    return;
                }
                let fill_rule = paint.fill_rule();
                let path = match paint.pick_path_kind() {
                    ShapePaintPathKind::Local | ShapePaintPathKind::LocalClockwise => {
                        &mut self.local_path
                    }
                    ShapePaintPathKind::World => &mut self.world_path,
                };
                paint
                    .shape_paint_mut()
                    .draw_with_fill_rule(renderer, path, world, false, None, true, fill_rule);
            });
        }
    }
    pub fn draw(&mut self, renderer: &mut Renderer) {
        if self.base.clip() {
            renderer.restore();
        }
    }
    pub fn update_render_path(&mut self) {
        {
            if self.is_hidden()
                || (self.paints.shape_paints().is_empty()
                    && !self.base.clip()
                    && !self.has_foreground_drawable)
            {
                return;
            }
            let mut radii = [0.0; 4];
            let ltr = self.actual_direction() != LayoutDirection::Rtl;
            self.with_style(|style| {
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
            });
            self.background_raw_path.rewind();
            Path::add_rounded_rect(
                &mut self.background_raw_path,
                Aabb::new(0.0, 0.0, self.layout.width(), self.layout.height()),
                radii,
            );
            self.local_path.rewind();
            self.local_path.add_path(&self.background_raw_path, None);
            self.world_path
                .rewind_as(false, nuxie_render_api::FillRule::Clockwise);
            self.world_path.add_path(
                &self.background_raw_path,
                Some(self.base.base.base.base.world_transform()),
            );
            for paint in self.paints.shape_paints().iter().cloned() {
                paint.with_mut(|paint| {
                    if let Some(paint) = paint.as_shape_paint_behavior_mut() {
                        if paint.should_draw() {
                            paint.shape_paint_mut().invalidate_effects();
                        }
                    }
                });
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
        let mut size = Vec2D::default();
        for child in self.base.base.base.base.base.children() {
            let measured = child
                .with_mut(|child| {
                    if child.as_layout_component().is_some() {
                        return None;
                    }
                    child.as_intrinsically_sizeable_mut().map(|sizeable| {
                        sizeable.measure_layout(width, width_mode, height, height_mode)
                    })
                })
                .flatten();
            if let Some(measured) = measured {
                size = Vec2D::new(size.x.max(measured.x), size.y.max(measured.y));
            }
        }
        size
    }
    pub fn effective_parent_is_row(&mut self) -> bool {
        if self.can_have_overrides() {
            self.parent_is_row
        } else {
            self.layout_parent_handle()
                .and_then(|parent| {
                    parent.with(|parent| {
                        parent
                            .as_layout_component()
                            .map(LayoutComponent::main_axis_is_row)
                    })
                })
                .flatten()
                .unwrap_or(true)
        }
    }
    pub fn main_axis_is_row(&self) -> bool {
        self.with_style(|style| {
            matches!(
                style.flex_direction(),
                YGFlexDirection::Row | YGFlexDirection::RowReverse
            )
        })
        .unwrap_or(true)
    }
    pub fn main_axis_is_column(&self) -> bool {
        self.with_style(|style| {
            matches!(
                style.flex_direction(),
                YGFlexDirection::Column | YGFlexDirection::ColumnReverse
            )
        })
        .unwrap_or(false)
    }
    pub fn layout_node_key(&self, index: usize) -> Option<LayoutNodeKey> {
        let provider = self.base.base.base.base.base.handle()?;
        (index == 0).then_some(LayoutNodeKey { provider, index })
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
            .any(|child| {
                child
                    .with(|child| child.layout_provider_handle().is_some())
                    .unwrap_or(false)
            })
    }
    pub fn sync_style(&mut self) {
        let Some(style) = self.style_handle() else {
            return;
        };
        let parent = self.layout_parent_handle();
        let parent_style = parent.as_ref().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_layout_component()
                        .and_then(LayoutComponent::style_handle)
                })
                .flatten()
        });
        let (parent_is_grid, parent_is_stack, container_justify_items) = parent_style
            .as_ref()
            .and_then(|style| {
                style.with_downcast::<LayoutComponentStyle, _>(|style| {
                    (
                        style.is_grid(),
                        style.is_stack(),
                        style.base.justify_items_value(),
                    )
                })
            })
            .unwrap_or((false, false, YGAlign::Stretch as u32));
        let inline_hugs = style
            .with_downcast::<LayoutComponentStyle, _>(|style| {
                style.width_scale_type() == LayoutScaleType::Hug
            })
            .unwrap_or(false);
        let context = LayoutSyncContext {
            parent_is_grid,
            parent_is_stack,
            container_justify_items,
            inline_hugs,
            parent_is_row: self.effective_parent_is_row(),
            is_ltr: self.actual_direction() != LayoutDirection::Rtl,
            has_layout_parent: parent.is_some(),
        };
        let mut taffy_style = std::mem::take(&mut self.layout_data.style);
        self.layout_data
            .apply_layout_styles(&mut taffy_style, &context);
        self.layout_data.style = taffy_style;
        self.layout_data.dirty = true;
        self.sync_child_provider_styles();
    }
    pub fn taffy_style(&self) -> taffy::style::Style {
        self.layout_data.style.taffy_style()
    }
    pub fn is_intrinsic_leaf(&self) -> bool {
        self.is_leaf()
            && self
                .with_style(|style| style.base.intrinsically_sized())
                .unwrap_or(false)
    }
    pub fn set_solved_layout(&mut self, layout: Layout, padding: LayoutPadding) {
        self.layout_data.solved_layout = layout;
        self.layout_padding = padding;
        self.layout_data.has_new_layout = true;
    }
    pub fn clear_layout_children(&mut self) {
        #[cfg(feature = "tools")]
        self.layout_data.clear_children();
    }
    pub fn sync_layout_children(&mut self) {
        self.clear_layout_children();
        #[cfg(feature = "tools")]
        for child in self.base.base.base.base.base.children() {
            if child
                .with(|child| child.layout_provider_handle().is_some())
                .unwrap_or(false)
            {
                self.layout_data.children.push(child.clone());
            }
        }
        self.mark_layout_node_dirty(false);
    }
    pub fn propagate_size(&mut self) {
        let Some(owner) = self.base.base.base.base.base.handle() else {
            return;
        };
        let direction = self.actual_direction();
        let style = self.with_style(|style| {
            (
                style.width_scale_type(),
                style.height_scale_type(),
                direction,
            )
        });
        Self::propagate_size_to_children(
            owner,
            self.is_hidden(),
            Vec2D::new(self.layout.width(), self.layout.height()),
            style,
        );
    }
    fn propagate_size_to_children(
        component: CoreHandle,
        hidden: bool,
        size: Vec2D,
        style: Option<(LayoutScaleType, LayoutScaleType, LayoutDirection)>,
    ) {
        if hidden {
            return;
        }
        let children = component
            .with(|component| {
                component
                    .as_container_component()
                    .map(|container| container.children().to_vec())
            })
            .flatten()
            .unwrap_or_default();
        for child in children {
            let skip = child
                .with(|child| {
                    child.as_layout_component().is_some()
                        || child.is_transparent_layout_container()
                        || child.layout_provider_handle().is_some()
                })
                .unwrap_or(true);
            if skip {
                continue;
            }
            let propagate = child
                .with_mut(|child| {
                    let Some(sizeable) = child.as_intrinsically_sizeable_mut() else {
                        return true;
                    };
                    if let Some((width, height, direction)) = style {
                        sizeable.control_size(size, width, height, direction);
                    }
                    sizeable.should_propagate_size_to_children()
                })
                .unwrap_or(false);
            if propagate {
                Self::propagate_size_to_children(child, false, size, style);
            }
        }
    }
    pub fn layout_solve_available_size(
        &self,
        available_width: f32,
        available_height: f32,
    ) -> Vec2D {
        let intrinsically_sized = self
            .with_style(|style| style.base.intrinsically_sized())
            .unwrap_or(false);
        Vec2D::new(
            if available_width.is_nan() && intrinsically_sized {
                available_width
            } else {
                self.base.width()
            },
            if available_height.is_nan() && intrinsically_sized {
                available_height
            } else {
                self.base.height()
            },
        )
    }
    pub fn style_display_hidden(&self) -> bool {
        self.with_style(|style| style.display() == YGDisplay::None)
            .unwrap_or(false)
    }
    pub fn actual_direction(&self) -> LayoutDirection {
        self.with_style(|style| match style.direction() {
            YGDirection::Ltr => LayoutDirection::Ltr,
            YGDirection::Rtl => LayoutDirection::Rtl,
            _ => self.inherited_direction,
        })
        .unwrap_or(self.inherited_direction)
    }
    pub fn on_dirty(&mut self, value: ComponentDirt) {
        self.base.base.base.base.base.on_dirty(value);
        if value.contains(ComponentDirt::WORLD_TRANSFORM) && self.base.clip() {
            CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
        }
    }
    pub fn update_layout_bounds(&mut self, animate: bool) {
        if !self.layout_data.has_new_layout {
            return;
        }
        self.layout_data.has_new_layout = false;
        let next = self.layout_data.solved_layout;
        if self.just_added_to_host {
            self.just_added_to_host = false;
            self.layout = next;
            let data = self.current_animation_data();
            data.from = next;
            data.to = next;
            data.elapsed_seconds = 0.0;
            self.propagate_size();
            CoreCapabilities::world_transform_mark_dirty(self);
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
                CoreCapabilities::world_transform_mark_dirty(self);
            }
        } else if next != self.layout || self.force_update_layout_bounds {
            if self.layout.width() != next.width() || self.layout.height() != next.height() {
                CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
            }
            self.layout = next;
            self.animation_data_a.to = next;
            self.propagate_size();
            CoreCapabilities::world_transform_mark_dirty(self);
        }
        self.force_update_layout_bounds = false;
    }
    pub fn animates(&self) -> bool {
        self.animation_style() != LayoutAnimationStyle::None
            && self.interpolation() != LayoutStyleInterpolation::Hold
            && self.interpolation_time() > 0.0
    }
    pub fn animation_style(&self) -> LayoutAnimationStyle {
        self.with_style(LayoutComponentStyle::animation_style)
            .unwrap_or(LayoutAnimationStyle::None)
    }
    pub fn interpolator(&self) -> Option<CoreHandle> {
        self.with_style(|style| match style.animation_style() {
            LayoutAnimationStyle::Inherit => self
                .inherited_interpolator
                .clone()
                .or_else(|| style.interpolator()),
            LayoutAnimationStyle::Custom => style.interpolator(),
            _ => None,
        })
        .flatten()
    }
    pub fn interpolation(&self) -> LayoutStyleInterpolation {
        self.with_style(|style| match style.animation_style() {
            LayoutAnimationStyle::Inherit => self.inherited_interpolation,
            LayoutAnimationStyle::Custom => style.interpolation(),
            _ => LayoutStyleInterpolation::Hold,
        })
        .unwrap_or(LayoutStyleInterpolation::Hold)
    }
    pub fn interpolation_time(&self) -> f32 {
        self.with_style(|style| match style.animation_style() {
            LayoutAnimationStyle::Inherit => self.inherited_interpolation_time,
            LayoutAnimationStyle::Custom => style.base.interpolation_time(),
            _ => 0.0,
        })
        .unwrap_or(0.0)
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
            CoreCapabilities::world_transform_mark_dirty(self);
            return false;
        }
        let mut factor = (data.elapsed_seconds / time).min(1.0);
        if self.interpolation() != LayoutStyleInterpolation::Linear {
            if let Some(interpolator) = self.interpolator() {
                factor = interpolator
                    .with(|interpolator| interpolator.keyframe_interpolator_transform(factor))
                    .flatten()
                    .unwrap_or(factor);
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
            CoreCapabilities::world_transform_mark_dirty(self);
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
        if flags.0 & AdvanceFlags::NEW_FRAME.0 == 0 || self.is_collapsed() {
            return false;
        }
        self.apply_interpolation(
            elapsed,
            flags.0 & (AdvanceFlags::ANIMATE.0 | AdvanceFlags::ADVANCE_NESTED.0) != 0,
        )
    }
    pub fn interrupt_animation(&mut self) {
        if self.animates() {
            self.layout = self.current_animation_data().to;
            self.propagate_size();
        }
    }
    pub fn mark_layout_node_dirty(&mut self, force: bool) {
        self.force_update_layout_bounds |= force;
        self.layout_data.dirty = true;
        if let (Some(artboard), Some(this)) = (
            self.base.base.base.base.base.artboard_handle(),
            self.base.base.base.base.base.handle(),
        ) {
            artboard.with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.mark_layout_dirty(this);
            });
        }
    }
    pub fn mark_layout_style_dirty(&mut self) {
        self.clear_inherited_interpolation();
        CoreCapabilities::component_add_dirt(self, ComponentDirt::LAYOUT_STYLE, false);
        if let (Some(artboard), Some(this)) = (
            self.base.base.base.base.base.artboard_handle(),
            self.base.base.base.base.base.handle(),
        ) {
            if artboard != this {
                artboard.with_downcast_mut::<Artboard, _>(|artboard| {
                    artboard.mark_layout_style_dirty();
                });
            }
        }
    }
    pub fn set_inherited_interpolation(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<CoreHandle>,
        time: f32,
    ) -> bool {
        if interpolation == self.inherited_interpolation
            && interpolator == self.inherited_interpolator
            && time == self.inherited_interpolation_time
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
        interpolator: Option<CoreHandle>,
        time: f32,
        direction: LayoutDirection,
    ) -> bool {
        let inherits_animation = self
            .with_style(|style| style.animation_style() == LayoutAnimationStyle::Inherit)
            .unwrap_or(false);
        let mut updated = if inherits_animation {
            self.set_inherited_interpolation(interpolation, interpolator, time)
        } else {
            self.clear_inherited_interpolation();
            false
        };
        let old = self.inherited_direction;
        self.inherited_direction = if direction == LayoutDirection::Inherit
            || self
                .with_style(|style| style.direction() != YGDirection::Inherit)
                .unwrap_or(false)
        {
            LayoutDirection::Inherit
        } else {
            direction
        };
        if old != self.inherited_direction {
            self.mark_layout_node_dirty(true);
            CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
            updated = true;
        }
        updated
    }
    pub fn sync_child_provider_styles(&mut self) {
        for child in self.base.base.base.base.base.children() {
            let provider = child.with(|child| child.layout_provider_handle()).flatten();
            if let Some(provider) = provider {
                provider.with_mut(|provider| {
                    provider.layout_provider_sync_style_changes();
                    provider.layout_provider_mark_node_dirty(false);
                });
            }
        }
    }
    pub fn position_type_changed(&mut self) {
        let left = self.layout.left();
        let top = self.layout.top();
        let position_left_changed = self.position_left_changed;
        let position_top_changed = self.position_top_changed;
        let changed = self.with_style_mut(|style| {
            if style.position_type() == YGPositionType::Absolute {
                if !position_left_changed {
                    style.base.set_position_left(left);
                }
                if !position_top_changed {
                    style.base.set_position_top(top);
                }
                style.set_absolute_point_units();
            } else {
                style.clear_position();
            }
        });
        if changed.is_some() {
            self.mark_layout_node_dirty(false);
        }
    }
    pub fn scale_type_changed(&mut self) {
        let changed = self.with_style_mut(|style| {
            style.base.set_intrinsically_sized_value(
                style.width_scale_type() == LayoutScaleType::Hug
                    || style.height_scale_type() == LayoutScaleType::Hug,
            );
        });
        if changed.is_some() {
            self.mark_layout_node_dirty(false);
        }
    }
    pub fn display_changed(&mut self) {
        if self.style.is_some() {
            self.collapse_after_component(self.is_collapsed());
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
        CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
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
        CoreCapabilities::world_transform_mark_dirty(self);
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
    pub fn add_layout_style_applier(&mut self, applier: CoreHandle) {
        self.layout_data.add_applier(applier);
    }
    pub fn apply_container_style(&self, style: &mut YGStyle, _context: &LayoutSyncContext) {
        let justify = self.with_style(|component_style| {
            (!component_style.is_stack()).then_some(component_style.base.justify_items_value())
        });
        if let Some(Some(justify)) = justify {
            crate::mechanical_port::source::layout::grid_track::GridTrack::sync_container_style(
                style, self, justify,
            );
        }
    }
    pub fn apply_base_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        let Some(component_style) = self.style_handle() else {
            return;
        };
        let Some((
            absolute,
            legacy_hug,
            stored_width_scale,
            stored_height_scale,
            stored_width_units,
            stored_height_units,
        )) = component_style.with_downcast::<LayoutComponentStyle, _>(|component_style| {
            (
                component_style.position_type() == YGPositionType::Absolute,
                component_style.width_scale_type() == LayoutScaleType::Fixed
                    && component_style.height_scale_type() == LayoutScaleType::Fixed
                    && component_style.base.intrinsically_sized()
                    && self.is_leaf(),
                component_style.width_scale_type(),
                component_style.height_scale_type(),
                component_style.width_units(),
                component_style.height_units(),
            )
        })
        else {
            return;
        };
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
        let mut width_scale = stored_width_scale;
        let mut height_scale = stored_height_scale;
        let mut width_units = units(width_scale, stored_width_units);
        let mut height_units = units(height_scale, stored_height_units);
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
impl LayoutStyleApplier for LayoutComponent {
    fn apply_base_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        LayoutComponent::apply_base_style(self, style, context);
    }

    fn apply_container_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        LayoutComponent::apply_container_style(self, style, context);
    }
}

impl LayoutNodeProvider for LayoutComponent {
    fn provider_state(&mut self) -> &mut LayoutNodeProviderState {
        &mut self.provider
    }

    fn provider_handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.handle()
    }

    fn owner_handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.handle()
    }

    fn layout_bounds(&self) -> Aabb {
        LayoutComponent::layout_bounds(self)
    }

    fn sync_style_changes(&mut self) -> bool {
        self.sync_style();
        true
    }

    fn update_layout_bounds(&mut self, animate: bool) {
        LayoutComponent::update_layout_bounds(self, animate);
    }

    fn mark_layout_node_dirty(&mut self, force: bool) {
        LayoutComponent::mark_layout_node_dirty(self, force);
    }

    fn add_layout_style_applier(&mut self, applier: CoreHandle) {
        LayoutComponent::add_layout_style_applier(self, applier);
    }

    fn num_layout_nodes(&self) -> usize {
        LayoutComponent::num_layout_nodes(self)
    }

    fn cascade_layout_style(
        &mut self,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<CoreHandle>,
        time: f32,
        direction: LayoutDirection,
    ) -> bool {
        LayoutComponent::cascade_layout_style(self, interpolation, interpolator, time, direction)
    }
}
impl Drop for LayoutComponent {
    fn drop(&mut self) {
        let this = self.base.base.base.base.base.handle();
        if let (Some(artboard), Some(this)) =
            (self.base.base.base.base.base.artboard_handle(), this)
        {
            artboard.with_downcast_mut::<Artboard, _>(|artboard| artboard.clean_layout(&this));
        }
        self.proxy.take();
    }
}

impl std::ops::Deref for LayoutComponent {
    type Target = LayoutComponentBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for LayoutComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
use std::{cell::RefCell, rc::Rc};
