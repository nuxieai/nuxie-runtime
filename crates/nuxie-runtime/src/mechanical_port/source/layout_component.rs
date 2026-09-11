use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponent,
    artboard::Artboard,
    component::{Component, ComponentOccurrenceHandle},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    drawable::{Drawable, DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence},
    generated::{
        core_registry::CoreCapabilities,
        layout_component_base::{LayoutComponentBase, LayoutComponentBaseCallbacks},
    },
    hit_info::HitInfo,
    importers::import_stack::ImportStack,
    layout::{
        layout_component_style::LayoutComponentStyle,
        layout_data::LayoutData,
        layout_enums::{
            LayoutAnimationStyle, LayoutDirection, LayoutScaleType, LayoutStyleInterpolation,
        },
        layout_measure_mode::LayoutMeasureMode,
        layout_node_provider::{
            LayoutNodeKey, LayoutNodeProvider, LayoutNodeProviderState, layout_node_owner_for,
        },
        layout_style_applier::{
            LayoutStyleApplier, LayoutSyncContext, YGAlign, YGDimension, YGDirection, YGDisplay,
            YGFlexDirection, YGFloatOptional, YGJustify, YGPositionType, YGStyle, YGUnit, YGValue,
        },
    },
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    renderer::{RenderPath, RenderPaint, RenderPaintStyle, Renderer},
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

#[derive(Clone, PartialEq, Eq)]
enum LayoutMeasureContext {
    Layout(CoreHandle),
    Participant(CoreHandle),
}

struct CachedLayoutNode {
    owner: CoreHandle,
    node: taffy::prelude::NodeId,
    children: Vec<usize>,
    measure: Option<LayoutMeasureContext>,
}

struct LayoutTreeCache {
    tree: taffy::prelude::TaffyTree<LayoutMeasureContext>,
    nodes: Vec<CachedLayoutNode>,
    root: usize,
    css_baselines: bool,
}

#[repr(u16)]
#[derive(Clone, Copy)]
pub(crate) enum LayoutComponentFlags {
    ParentIsRow = 1 << 0,
    ParentIsStack = 1 << 1,
    WidthIntrinsicallySizeOverride = 1 << 2,
    HeightIntrinsicallySizeOverride = 1 << 3,
    ForceUpdateLayoutBounds = 1 << 4,
    PositionLeftChanged = 1 << 5,
    PositionTopChanged = 1 << 6,
    HasForegroundDrawable = 1 << 7,
    HasComponentOrigin = 1 << 8,
    ComposeTransform = 1 << 9,
    IsSmoothingAnimation = 1 << 10,
    JustAddedToHost = 1 << 11,
    ForceDrawableProxy = 1 << 12,
    ClipSaved = 1 << 13,
}

#[derive(Default)]
pub(crate) struct LayoutRenderPaths {
    pub background: RawPath,
    pub local: ShapePaintPath,
    pub world: ShapePaintPath,
    pub css_clip: ShapePaintPath,
    pub css_overflow_rect: ShapePaintPath,
    pub css_border: ShapePaintPath,
    pub css_border_side: ShapePaintPath,
    pub css_border_paint: Option<Box<RenderPaint>>,
    pub css_gradient: ShapePaintPath,
    pub css_gradient_paint: Option<Box<RenderPaint>>,
}

/// Explicit CSS flex-item alignment, separate from Rive's sizing-derived rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CssAlignSelf {
    Auto,
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
    Baseline,
}
impl CssAlignSelf {
    fn runtime_alignment(self) -> YGAlign {
        match self {
            Self::Auto => YGAlign::Auto,
            Self::FlexStart => YGAlign::FlexStart,
            Self::Center => YGAlign::Center,
            Self::FlexEnd => YGAlign::FlexEnd,
            Self::Stretch => YGAlign::Stretch,
            Self::Baseline => YGAlign::Baseline,
        }
    }
}

/// Independent CSS factors; construction rejects non-finite or negative values.
/// None on a layout occurrence preserves Rive's shared fractional weight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFlexFactors {
    grow: f32,
    shrink: f32,
}
impl CssFlexFactors {
    pub fn new(grow: f32, shrink: f32) -> Option<Self> {
        (grow.is_finite() && shrink.is_finite() && grow >= 0.0 && shrink >= 0.0)
            .then_some(Self { grow, shrink })
    }
}

/// Explicit CSS main-axis alignment, independent of Rive's combined encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CssJustifyContent {
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
impl CssJustifyContent {
    fn runtime_alignment(self) -> YGJustify {
        match self {
            Self::FlexStart => YGJustify::FlexStart,
            Self::Center => YGJustify::Center,
            Self::FlexEnd => YGJustify::FlexEnd,
            Self::SpaceBetween => YGJustify::SpaceBetween,
            Self::SpaceAround => YGJustify::SpaceAround,
            Self::SpaceEvenly => YGJustify::SpaceEvenly,
        }
    }
}

/// Explicit CSS flex-line alignment, independent of Rive's item alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CssAlignContent {
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
impl CssAlignContent {
    fn runtime_alignment(self) -> YGAlign {
        match self {
            Self::FlexStart => YGAlign::FlexStart,
            Self::Center => YGAlign::Center,
            Self::FlexEnd => YGAlign::FlexEnd,
            Self::Stretch => YGAlign::Stretch,
            Self::SpaceBetween => YGAlign::SpaceBetween,
            Self::SpaceAround => YGAlign::SpaceAround,
            Self::SpaceEvenly => YGAlign::SpaceEvenly,
        }
    }
}

/// Opt-in one-axis CSS clip. The other axis remains unrestricted; unlike the
/// ordinary two-axis clip, border radii do not shape this strip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CssOverflowAxis { Horizontal, Vertical }

/// The reference box for an opt-in CSS overflow clip margin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CssOverflowClipBox { Content, Padding, Border }

/// A validated computed offset. This does not itself enable clipping: the
/// compiler/host must first determine whether the computed overflow applies it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssOverflowClipMargin {
    origin: CssOverflowClipBox,
    pixels: f32,
}

impl CssOverflowClipMargin {
    pub fn new(origin: CssOverflowClipBox, pixels: f32) -> Option<Self> {
        pixels.is_finite().then_some(Self { origin, pixels })
    }

    /// Resolve from current layout, never from dimensions captured at import.
    /// Border and padding edges come from the live solved layout. Public
    /// compiler border admission additionally requires border painting support.
    fn insets(self, layout: &LayoutComponent) -> [f32; 4] {
        let (left, top, right, bottom) = match self.origin {
            CssOverflowClipBox::Content => (layout.padding_left() + layout.layout_border.left(),
                layout.padding_top() + layout.layout_border.top(),
                layout.padding_right() + layout.layout_border.right(),
                layout.padding_bottom() + layout.layout_border.bottom()),
            CssOverflowClipBox::Padding => (layout.layout_border.left(), layout.layout_border.top(),
                layout.layout_border.right(), layout.layout_border.bottom()),
            CssOverflowClipBox::Border => (0., 0., 0., 0.),
        };
        [left, top, right, bottom]
    }

    pub fn bounds(self, layout: &LayoutComponent) -> Option<Aabb> {
        let [left, top, right, bottom] = self.insets(layout);
        let bounds = Aabb::new(left - self.pixels, top - self.pixels,
            layout.layout_width() - right + self.pixels,
            layout.layout_height() - bottom + self.pixels);
        [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
            .into_iter().all(f32::is_finite).then_some(bounds)
    }
}

pub struct LayoutComponent {
    pub base: LayoutComponentBase,
    paints: ShapePaintContainer,
    provider: LayoutNodeProviderState,
    style: Option<CoreHandle>,
    pub(crate) layout_data: Box<LayoutData>,
    layout_children: Vec<LayoutNodeKey>,
    layout_tree_cache: Option<LayoutTreeCache>,
    layout_tree_topology_dirty: bool,
    layout: Layout,
    layout_padding: LayoutPadding,
    solved_padding: LayoutPadding,
    layout_border: LayoutPadding,
    solved_border: LayoutPadding,
    animation_data_a: LayoutAnimationData,
    animation_data_b: LayoutAnimationData,
    inherited_interpolator: Option<CoreHandle>,
    inherited_interpolation: LayoutStyleInterpolation,
    inherited_interpolation_time: f32,
    inherited_direction: LayoutDirection,
    layout_flags: u16,
    render_paths: Option<Box<LayoutRenderPaths>>,
    css_overflow_axis: Option<CssOverflowAxis>,
    css_overflow_clip_margin: Option<CssOverflowClipMargin>,
    css_pixel_bounds: bool,
    css_corner_radii: Option<super::css_corner_radii::CssCornerRadii>,
    css_gradient: Option<super::css_linear_gradient::CssLinearGradient>,
    css_border_geometry: bool,
    css_border_color: Option<u32>,
    css_border_side_colors: Option<[u32; 4]>,
    css_percentage_spacing: bool,
    css_relative_position: bool,
    css_positioned: Option<bool>,
    css_content_box: bool,
    css_ratio_content_box: bool,
    css_ratio_size_limit: bool,
    css_ratio_pair: Option<[u32; 2]>,
    css_align_self: Option<CssAlignSelf>,
    css_align_content: Option<CssAlignContent>,
    css_justify_content: Option<CssJustifyContent>,
    css_flex_factors: Option<CssFlexFactors>,
    proxy: Option<Rc<RefCell<DrawableProxy>>>,
    width_override: f32,
    width_unit_value_override: i8,
    height_override: f32,
    height_unit_value_override: i8,
    forced_width: f32,
    forced_height: f32,
    // Files exported before 7.3 never composed a layout's own rotation/scale,
    // so any stored value was ignored. Import clears this for those files; it
    // defaults to the current behavior so a layout built outside of import
    // isn't stuck on the legacy path. See File::MINOR_VERSION.
}

impl Default for LayoutComponent {
    fn default() -> Self {
        Self {
            base: LayoutComponentBase::default(),
            paints: ShapePaintContainer::default(),
            provider: LayoutNodeProviderState::default(),
            style: None,
            layout_data: Box::new(LayoutData::default()),
            layout_children: Vec::new(),
            layout_tree_cache: None,
            layout_tree_topology_dirty: true,
            layout: Layout::default(),
            layout_padding: LayoutPadding::default(),
            solved_padding: LayoutPadding::default(),
            layout_border: LayoutPadding::default(),
            solved_border: LayoutPadding::default(),
            animation_data_a: LayoutAnimationData::default(),
            animation_data_b: LayoutAnimationData::default(),
            inherited_interpolator: None,
            inherited_interpolation: LayoutStyleInterpolation::Hold,
            inherited_interpolation_time: 0.0,
            inherited_direction: LayoutDirection::Inherit,
            layout_flags: LayoutComponentFlags::ParentIsRow as u16
                | LayoutComponentFlags::PositionLeftChanged as u16
                | LayoutComponentFlags::PositionTopChanged as u16
                | LayoutComponentFlags::ComposeTransform as u16,
            render_paths: None,
            css_overflow_axis: None,
            css_overflow_clip_margin: None,
            css_pixel_bounds: false,
            css_corner_radii: None,
            css_gradient: None,
            css_border_geometry: false,
            css_border_color: None,
            css_border_side_colors: None,
            css_percentage_spacing: false,
            css_relative_position: false,
            css_positioned: None,
            css_content_box: false,
            css_ratio_content_box: false,
            css_ratio_size_limit: false,
            css_ratio_pair: None,
            css_align_self: None,
            css_align_content: None,
            css_justify_content: None,
            css_flex_factors: None,
            proxy: None,
            width_override: f32::NAN,
            width_unit_value_override: -1,
            height_override: f32::NAN,
            height_unit_value_override: -1,
            forced_width: f32::NAN,
            forced_height: f32::NAN,
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
            .with(|owner| owner.drawable_is_hidden())
            .unwrap_or(true)
    }
    fn owner_handle(&self) -> CoreHandle {
        self.owner.clone()
    }
}

impl LayoutComponent {
    pub(crate) fn has_layout_flag(&self, flag: LayoutComponentFlags) -> bool {
        self.layout_flags & flag as u16 != 0
    }
    pub(crate) fn set_layout_flag(&mut self, flag: LayoutComponentFlags, on: bool) {
        if on {
            self.layout_flags |= flag as u16;
        } else {
            self.layout_flags &= !(flag as u16);
        }
    }
    pub(crate) fn mutable_render_paths(&mut self) -> &mut LayoutRenderPaths {
        self.render_paths.get_or_insert_with(Default::default)
    }
    pub fn needs_drawable_proxy(&self) -> bool {
        self.css_gradient.is_some() || self.css_overflow_axis.is_some() || self.base.clip()
            || !self.paints.shape_paints().is_empty()
            || self.has_layout_flag(LayoutComponentFlags::ForceDrawableProxy)
    }
    pub fn mark_clip_may_be_dynamic(&mut self) {
        self.set_layout_flag(LayoutComponentFlags::ForceDrawableProxy, true);
    }
    pub fn mark_interaction_target(&mut self) {
        self.set_layout_flag(LayoutComponentFlags::ForceDrawableProxy, true);
    }
    pub fn mark_listener_target(&mut self) {
        self.set_layout_flag(LayoutComponentFlags::ForceDrawableProxy, true);
    }
    pub(crate) fn set_compose_transform_from_import(&mut self, import_stack: &ImportStack) {
        // Files exported before 7.3 composed a layout's transform from the solved
        // slot alone, so any stored rotation/scale was written but never applied.
        // Keep that legacy behavior for those files; newer files compose it on top
        // of the slot. See File::MINOR_VERSION.
        let major = import_stack.major_version();
        let minor = import_stack.minor_version();
        self.set_layout_flag(
            LayoutComponentFlags::ComposeTransform,
            major > 7 || (major == 7 && minor >= 3),
        );
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.set_compose_transform_from_import(import_stack);
        Component::import(
            &mut self
                .base
                .base
                .base
                .base
                .base
                .base
                .base
                .base
                .base
                .base
                .base
                .base,
            import_stack,
        )
    }

    pub fn clone_core(&self) -> Self {
        let mut callbacks = Self::default();
        let mut twin = self.base.clone_into(&mut callbacks);
        twin.css_overflow_axis = self.css_overflow_axis;
        twin.css_overflow_clip_margin = self.css_overflow_clip_margin;
        twin.css_pixel_bounds = self.css_pixel_bounds;
        twin.css_corner_radii = self.css_corner_radii;
        twin.css_gradient = self.css_gradient.clone();
        twin.css_border_geometry = self.css_border_geometry;
        twin.css_border_color = self.css_border_color;
        twin.css_border_side_colors = self.css_border_side_colors;
        twin.css_percentage_spacing = self.css_percentage_spacing;
        twin.css_relative_position = self.css_relative_position;
        twin.css_positioned = self.css_positioned;
        twin.css_content_box = self.css_content_box;
        twin.css_ratio_content_box = self.css_ratio_content_box;
        twin.css_ratio_size_limit = self.css_ratio_size_limit;
        twin.css_ratio_pair = self.css_ratio_pair;
        twin.css_align_self = self.css_align_self;
        twin.css_align_content = self.css_align_content;
        twin.css_justify_content = self.css_justify_content;
        twin.css_flex_factors = self.css_flex_factors;
        twin.set_layout_flag(
            LayoutComponentFlags::ComposeTransform,
            self.has_layout_flag(LayoutComponentFlags::ComposeTransform),
        );
        twin.set_layout_flag(
            LayoutComponentFlags::ForceDrawableProxy,
            self.has_layout_flag(LayoutComponentFlags::ForceDrawableProxy),
        );
        twin
    }

    /// Pinned forEachLayoutProvider: groups are transparent, Solo exposes its
    /// active child, and a nested component list must explicitly opt in.
    pub fn layout_providers_occurrence(from: &CoreHandle) -> Vec<(CoreHandle, CoreHandle)> {
        Self::layout_providers_nested_occurrence(from, false)
    }
    fn layout_providers_nested_occurrence(
        from: &CoreHandle,
        nested: bool,
    ) -> Vec<(CoreHandle, CoreHandle)> {
        Self::layout_providers_nested_with_solo(from, nested, None)
    }
    fn layout_providers_nested_with_solo(
        from: &CoreHandle,
        nested: bool,
        active_solo: Option<&crate::mechanical_port::source::solo::Solo>,
    ) -> Vec<(CoreHandle, CoreHandle)> {
        let children = if let Some(solo) =
            active_solo.filter(|solo| solo.base.handle().as_ref() == Some(from))
        {
            solo.active_component().into_iter().collect()
        } else {
            from.with(|object| {
                if let Some(solo) = object
                    .as_any()
                    .downcast_ref::<crate::mechanical_port::source::solo::Solo>()
                {
                    solo.active_component().into_iter().collect::<Vec<_>>()
                } else {
                    object
                        .as_container_component()
                        .expect("layout traversal container")
                        .children()
                        .to_vec()
                }
            })
            .unwrap_or_default()
        };
        Self::layout_providers_children_with_solo(&children, nested, active_solo)
    }
    fn layout_providers_children(
        children: &[CoreHandle],
        nested: bool,
    ) -> Vec<(CoreHandle, CoreHandle)> {
        Self::layout_providers_children_with_solo(children, nested, None)
    }
    fn layout_providers_children_with_solo(
        children: &[CoreHandle],
        nested: bool,
        active_solo: Option<&crate::mechanical_port::source::solo::Solo>,
    ) -> Vec<(CoreHandle, CoreHandle)> {
        let mut result = Vec::new();
        for child in children {
            // LayoutNodeProvider::from uses the immutable core type before
            // reading provider state. In particular, an attached style may be
            // actively setting a property while this traversal visits children.
            if let Some(provider) =
                crate::mechanical_port::source::layout::layout_node_provider::from_component(child)
            {
                let joins = !nested
                    || !child.is_type_of(crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY)
                    || child.with(|object| {
                        let drawable = object.as_drawable().expect("component list Drawable");
                        drawable.base.drawable_flags() & crate::mechanical_port::source::drawable_flag::DrawableFlag::PARTICIPATES_IN_LAYOUT.0 != 0
                    }).expect("live component list");
                if joins {
                    result.push((child.clone(), provider));
                }
            } else if child.core_type()
                == Some(crate::mechanical_port::source::generated::node_base::NodeBase::TYPE_KEY)
                || child.is_type_of(
                    crate::mechanical_port::source::generated::solo_base::SoloBase::TYPE_KEY,
                )
            {
                result.extend(Self::layout_providers_nested_with_solo(
                    child,
                    true,
                    active_solo,
                ));
            }
        }
        result
    }

    pub fn on_added_clean_occurrence(
        owner: &CoreHandle,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        let code = owner
            .with_mut(|object| {
                object
                    .as_transform_component_mut()
                    .expect("Layout transform super")
                    .on_added_clean(context)
            })
            .unwrap_or(StatusCode::MissingObject);
        if code != StatusCode::Ok {
            return code;
        }
        Self::mark_layout_style_dirty_occurrence(owner);
        Self::sync_layout_children_occurrence(owner);
        let collapsed = owner
            .with(|object| object.as_layout_component().unwrap().is_collapsed())
            .unwrap();
        Self::propagate_collapse_occurrence(owner, collapsed);
        StatusCode::Ok
    }

    pub fn mark_layout_node_dirty_occurrence(owner: &CoreHandle, force: bool) {
        Self::mark_layout_node_dirty_with_host_occurrence(owner, force, None);
    }

    /// `YGNode::markDirtyAndPropagate` reaches every ancestor in the retained
    /// Yoga tree. The Taffy adaptation retains only the calculation root's
    /// materialized tree, so a child-list mutation must invalidate that same
    /// ancestor chain before the next solve.
    fn mark_layout_tree_topology_dirty_occurrence(owner: &CoreHandle) {
        let mut current = Some(owner.clone());
        let mut active = Vec::new();
        while let Some(node_owner) = current {
            assert!(
                !active.contains(&node_owner),
                "cyclic layout node ownership"
            );
            active.push(node_owner.clone());
            current = node_owner
                .with_mut(|object| {
                    let layout = object.as_layout_component_mut().expect("Layout owner");
                    layout.layout_tree_topology_dirty = true;
                    let node = layout.layout_node_key(0)?;
                    let parent = node.owner.borrow().clone();
                    parent
                })
                .flatten();
        }
    }

    fn mark_layout_node_dirty_with_host_occurrence(
        owner: &CoreHandle,
        force: bool,
        host: Option<&mut dyn crate::mechanical_port::source::artboard_host::ArtboardHost>,
    ) {
        let artboard = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().expect("Layout owner");
                if force {
                    layout.set_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds, true);
                }
                layout.layout_data.dirty = true;
                layout.artboard_handle()
            })
            .flatten();
        if let Some(artboard) = artboard {
            Artboard::mark_layout_dirty_occurrence(&artboard, owner.clone(), host);
        }
    }

    pub(crate) fn set_parent_is_row_with_host_occurrence(
        owner: &CoreHandle,
        row: bool,
        host: &mut dyn crate::mechanical_port::source::artboard_host::ArtboardHost,
    ) {
        owner.with_mut(|object| {
            object
                .as_layout_component_mut()
                .expect("Layout owner")
                .set_layout_flag(LayoutComponentFlags::ParentIsRow, row);
        });
        Self::mark_layout_node_dirty_with_host_occurrence(owner, false, Some(host));
    }

    pub(crate) fn set_parent_is_stack_with_host_occurrence(
        owner: &CoreHandle,
        is_stack: bool,
        host: Option<&mut dyn crate::mechanical_port::source::artboard_host::ArtboardHost>,
    ) {
        let changed = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().expect("Layout owner");
                if layout.has_layout_flag(LayoutComponentFlags::ParentIsStack) == is_stack {
                    return false;
                }
                layout.set_layout_flag(LayoutComponentFlags::ParentIsStack, is_stack);
                true
            })
            .expect("live Layout owner");
        if changed {
            Self::mark_layout_node_dirty_with_host_occurrence(owner, false, host);
        }
    }

    pub(crate) fn set_clip_occurrence(owner: &CoreHandle, value: bool) {
        let changed = owner
            .with_mut(|object| {
                object
                    .as_layout_component_mut()
                    .expect("Layout owner")
                    .base
                    .set_clip_value(value)
            })
            .expect("live Layout owner");
        if !changed {
            return;
        }
        Self::mark_layout_node_dirty_occurrence(owner, false);
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
            owner.clone(),
        )
        .add_dirt(ComponentDirt::PATH, false);
        owner.with_mut(|object| {
            object
                .core_mut()
                .notify_property_changed(LayoutComponentBase::CLIP_PROPERTY_KEY);
        });
    }

    pub(crate) fn set_dimension_occurrence(owner: &CoreHandle, key: u16, value: f32) -> bool {
        let changed = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut()?;
                match key {
                    LayoutComponentBase::WIDTH_PROPERTY_KEY => {
                        Some(layout.base.set_width_value(value))
                    }
                    LayoutComponentBase::HEIGHT_PROPERTY_KEY => {
                        Some(layout.base.set_height_value(value))
                    }
                    _ => None,
                }
            })
            .flatten();
        let Some(changed) = changed else {
            return false;
        };
        if changed {
            // The generated setter invokes width/heightChanged before the
            // property notification. Release the owner for its root-layout
            // callback; for an Artboard, that root is this same occurrence.
            Self::mark_layout_node_dirty_occurrence(owner, false);
            owner.with_mut(|object| object.core_mut().notify_property_changed(key));
        }
        true
    }

    pub fn mark_layout_style_dirty_occurrence(owner: &CoreHandle) {
        let artboard = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().expect("Layout owner");
                layout.clear_inherited_interpolation();
                layout.artboard_handle()
            })
            .flatten();
        owner.with_mut(|object| object.component_add_dirt(ComponentDirt::LAYOUT_STYLE, false));
        if let Some(artboard) = artboard.filter(|artboard| artboard != owner) {
            Self::mark_layout_style_dirty_occurrence(&artboard);
        }
    }

    pub fn sync_layout_children_occurrence(owner: &CoreHandle) {
        Self::sync_layout_children_with_participant_occurrence(owner, None);
    }

    pub(crate) fn sync_layout_children_with_participant_occurrence(
        owner: &CoreHandle,
        active_participant: Option<
            &crate::mechanical_port::source::layout::layout_participant::LayoutParticipant,
        >,
    ) {
        Self::sync_layout_children_with_active_owners(owner, active_participant, None);
    }
    pub(crate) fn sync_layout_children_from_solo(
        owner: &CoreHandle,
        solo: &crate::mechanical_port::source::solo::Solo,
    ) {
        Self::sync_layout_children_with_active_owners(owner, None, Some(solo));
    }
    fn sync_layout_children_with_active_owners(
        owner: &CoreHandle,
        active_participant: Option<
            &crate::mechanical_port::source::layout::layout_participant::LayoutParticipant,
        >,
        active_solo: Option<&crate::mechanical_port::source::solo::Solo>,
    ) {
        let detached = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().expect("Layout owner");
                #[cfg(feature = "tools")]
                layout.layout_data.clear_children();
                std::mem::take(&mut layout.layout_children)
            })
            .expect("live Layout owner");
        Self::clear_detached_layout_ownership(Some(owner), &detached);
        for (_, provider) in Self::layout_providers_nested_with_solo(owner, false, active_solo) {
            let active = active_participant
                .filter(|participant| participant.base.handle().as_ref() == Some(&provider));
            let count = if let Some(participant) = active {
                participant.num_layout_nodes()
            } else {
                provider
                    .with_mut(|object| {
                        object
                            .as_layout_node_provider_mut()
                            .expect("layout provider")
                            .num_layout_nodes()
                    })
                    .expect("live layout provider")
            };
            for index in 0..count {
                let node = if let Some(participant) = active {
                    participant.layout_node_key(index)
                } else {
                    crate::mechanical_port::source::layout::layout_node_provider::layout_node_for(
                        &provider, index,
                    )
                };
                let Some(node) = node else {
                    continue;
                };
                *node.owner.borrow_mut() = Some(owner.clone());
                owner.with_mut(|object| {
                    let layout = object.as_layout_component_mut().expect("Layout owner");
                    #[cfg(feature = "tools")]
                    layout.layout_data.children.push(node.provider.clone());
                    layout.layout_children.push(node);
                });
            }
        }
        Self::mark_layout_tree_topology_dirty_occurrence(owner);
        Self::mark_layout_node_dirty_occurrence(owner, false);
    }

    pub fn propagate_collapse_occurrence(owner: &CoreHandle, value: bool) {
        let own_collapsed =
            owner.with(|object| object.as_layout_component().unwrap().is_collapsed());
        if let Some(own_collapsed) = own_collapsed {
            Self::propagate_resolved_collapse_occurrence(
                owner,
                value || own_collapsed,
                own_collapsed,
                None,
            );
        }
    }
    fn propagate_resolved_collapse_occurrence(
        owner: &CoreHandle,
        collapsed: bool,
        own_collapsed: bool,
        mut active_style: Option<&mut LayoutComponentStyle>,
    ) {
        let Some((children, collapsables)) = owner.with(|object| {
            let component = object.as_component().unwrap();
            (
                object.as_container_component().unwrap().children().to_vec(),
                component.collapsables_snapshot(),
            )
        }) else {
            return;
        };
        for child in children {
            if let Some(style) = active_style
                .as_deref_mut()
                .filter(|style| style.handle().as_ref() == Some(&child))
            {
                // The source calls this same child while its style setter is
                // active. Use that actual owner, not a second arena borrow.
                CoreCapabilities::component_collapse(style, collapsed);
            } else {
                ComponentOccurrenceHandle::Authored(child).collapse(collapsed);
            }
        }
        for collapsable in collapsables {
            collapsable.with_mut(|object| {
                if let Some(bind) = object.as_data_bind_mut() {
                    bind.collapse(own_collapsed);
                }
            });
        }
    }

    pub fn sync_style_occurrence(owner: &CoreHandle) {
        Self::sync_style_with_parent_style_occurrence(owner, None);
    }
    pub(crate) fn sync_style_with_parent_style_occurrence(
        owner: &CoreHandle,
        parent_style: Option<&crate::mechanical_port::source::layout::layout_style_applier::LayoutParentStyleSnapshot>,
    ) {
        let Some((mut style, context, appliers)) = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                let context = layout.style_sync_context(parent_style)?;
                Some((
                    std::mem::take(&mut layout.layout_data.style),
                    context,
                    layout
                        .layout_data
                        .appliers
                        .as_deref()
                        .cloned()
                        .unwrap_or_default(),
                ))
            })
            .flatten()
        else {
            return;
        };
        // Keep C++'s four sweeps and applier order. In particular the first
        // applier is this LayoutComponent, borrowed only after extraction above.
        for applier in &appliers {
            applier.with(|object| {
                if let Some(applier) = object.as_layout_style_applier() {
                    applier.apply_base_style(&mut style, &context);
                }
            });
        }
        for applier in &appliers {
            applier.with(|object| {
                if let Some(applier) = object.as_layout_style_applier() {
                    applier.apply_container_style(&mut style, &context);
                }
            });
        }
        for applier in &appliers {
            applier.with(|object| {
                if let Some(applier) = object.as_layout_style_applier() {
                    applier.apply_item_style(&mut style, &context);
                }
            });
        }
        for applier in &appliers {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_placement_style(&mut style, &context);
                }
            });
        }
        owner.with_mut(|object| {
            let layout = object.as_layout_component_mut().unwrap();
            layout.layout_data.style = style;
            layout.layout_data.dirty = true;
        });
        for (child, provider) in Self::layout_providers_occurrence(owner) {
            let excluded = matches!(child.core_type(), Some(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY | crate::mechanical_port::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY | crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY));
            if !excluded {
                Self::sync_provider_style_with_parent_style_occurrence(&provider, parent_style);
            }
        }
    }

    fn sync_provider_style_occurrence(provider: &CoreHandle) -> bool {
        Self::sync_provider_style_with_parent_style_occurrence(provider, None)
    }
    fn sync_provider_style_with_parent_style_occurrence(
        provider: &CoreHandle,
        parent_style: Option<&crate::mechanical_port::source::layout::layout_style_applier::LayoutParentStyleSnapshot>,
    ) -> bool {
        let kind = (
            provider.is_type_of(crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY),
            provider.is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY),
        );
        if kind.0 {
            return Artboard::sync_style_changes_with_parent_style_handle(provider, parent_style);
        }
        if kind.1 {
            Self::sync_style_with_parent_style_occurrence(provider, parent_style);
            return true;
        }
        if let Some((_, roots)) = Self::hosted_layout_roots(provider) {
            let mut changed = false;
            for (_, root) in roots {
                changed |=
                    Artboard::sync_style_changes_with_parent_style_handle(&root, parent_style);
            }
            return changed;
        }
        if provider.is_type_of(
            crate::mechanical_port::source::layout::layout_participant::LayoutParticipant::TYPE_KEY,
        ) {
            return crate::mechanical_port::source::layout::layout_participant::LayoutParticipant::sync_style_changes_occurrence(provider, parent_style);
        }
        provider
            .with_mut(|object| object.layout_provider_sync_style_changes())
            .flatten()
            .unwrap_or(false)
    }

    pub fn sync_child_provider_styles_occurrence(owner: &CoreHandle) {
        Self::sync_child_provider_styles_with_parent_style_occurrence(owner, None);
    }
    fn sync_child_provider_styles_with_parent_style_occurrence(
        owner: &CoreHandle,
        parent_style: Option<&crate::mechanical_port::source::layout::layout_style_applier::LayoutParentStyleSnapshot>,
    ) {
        for (_, provider) in Self::layout_providers_occurrence(owner) {
            Self::sync_provider_style_with_parent_style_occurrence(&provider, parent_style);
            if provider
                .is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY)
            {
                Self::mark_layout_node_dirty_occurrence(&provider, false);
            } else {
                provider.with_mut(|object| object.layout_provider_mark_node_dirty(false));
            }
        }
    }

    fn layout_parent_handle(&self) -> Option<CoreHandle> {
        let mut parent = self.base.base.base.base.base.parent_handle();
        while let Some(value) = parent {
            if value
                .is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY)
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
        if !self.has_layout_flag(LayoutComponentFlags::HasComponentOrigin) {
            return None;
        }
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
    pub fn mark_has_component_origin(&mut self) {
        self.set_layout_flag(LayoutComponentFlags::HasComponentOrigin, true);
    }
    pub fn origin_offset(&self) -> Vec2D {
        self.origin_offset_with(self.pivot_origin_x(), self.pivot_origin_y())
    }
    fn origin_offset_with(&self, origin_x: f32, origin_y: f32) -> Vec2D {
        Vec2D::new(
            origin_x * self.layout.width(),
            origin_y * self.layout.height(),
        )
    }
    pub fn local_anchor(&self) -> Vec2D {
        if !self.has_layout_flag(LayoutComponentFlags::HasComponentOrigin) || self.is_artboard() {
            Vec2D::default()
        } else {
            self.origin_offset()
        }
    }
    pub fn layout_translation(&self) -> Vec2D {
        // Our own origin deliberately does not enter here: contents sit within
        // our box whatever it is, so nothing inside has to compensate. An
        // artboard's origin does define where its local zero sits, so step back
        // by it to align to its box.
        let mut location = Vec2D::new(self.layout.left(), self.layout.top());
        if let Some(parent) = self.base.base.base.base.base.parent_handle() {
            if let Some(origin) = parent
                .with(|parent| {
                    parent.as_artboard().map(|artboard| {
                        Vec2D::new(
                            artboard.layout_width() * artboard.origin_x(),
                            artboard.layout_height() * artboard.origin_y(),
                        )
                    })
                })
                .flatten()
            {
                location -= origin;
            }
        }
        location
    }
    pub fn composes_layout_offset(&self) -> bool {
        self.has_layout_flag(LayoutComponentFlags::ComposeTransform) && !self.is_artboard()
    }
    pub fn composed_translation(&self) -> Vec2D {
        if self.composes_layout_offset() {
            Vec2D::new(
                self.base.base.base.base.base.x(),
                self.base.base.base.base.base.y(),
            )
        } else {
            self.layout_translation()
        }
    }
    pub fn build_own_transform(&self) -> Mat2D {
        self.build_own_transform_with(
            self.composes_layout_offset(),
            self.pivot_origin_x(),
            self.pivot_origin_y(),
        )
    }
    fn build_own_transform_with(
        &self,
        composes_layout_offset: bool,
        origin_x: f32,
        origin_y: f32,
    ) -> Mat2D {
        // Outermost, matching TransformComponent's T * R * S, so the offset is
        // not rotated or scaled by our own transform.
        let mut own = if composes_layout_offset {
            Mat2D::from_translation(Vec2D::new(
                self.base.base.base.base.base.x(),
                self.base.base.base.base.base.y(),
            ))
        } else {
            Mat2D::identity()
        };

        // Pivot about the origin. The box stays put, so wrap rather than fold
        // into the frame.
        if self.has_layout_flag(LayoutComponentFlags::ComposeTransform)
            && (self.rotation() != 0.0 || self.scale_x() != 1.0 || self.scale_y() != 1.0)
        {
            let mut local = if self.rotation() != 0.0 {
                Mat2D::from_rotation(self.rotation())
            } else {
                Mat2D::identity()
            };
            local.scale_by_values(self.scale_x(), self.scale_y());
            let pivot = self.origin_offset_with(origin_x, origin_y);
            if pivot.x != 0.0 || pivot.y != 0.0 {
                local = Mat2D::from_translate(pivot.x, pivot.y)
                    * local
                    * Mat2D::from_translate(-pivot.x, -pivot.y);
            }
            own *= local;
        }
        own
    }
    pub fn update_transform(&mut self) {
        *self.base.base.base.base.base.base.mutable_transform() = self.build_own_transform();
    }
    pub(crate) fn update_transform_for_artboard(&mut self, origin_x: f32, origin_y: f32) {
        *self.base.base.base.base.base.base.mutable_transform() =
            self.build_own_transform_with(false, origin_x, origin_y);
    }
    pub fn compose_world_transform(&mut self) {
        let parent_world = self
            .base
            .base
            .base
            .base
            .base
            .parent_handle()
            .and_then(|parent| {
                parent
                    .with(|parent| {
                        parent
                            .as_world_transform_component()
                            .map(|parent| *parent.world_transform())
                    })
                    .flatten()
            })
            .unwrap_or_else(Mat2D::identity);
        let base = Mat2D::from_translation(self.layout_translation());
        let own = *self.base.base.base.base.base.base.transform();
        self.base
            .base
            .base
            .base
            .set_world_transform(parent_world * base * own);
    }
    fn computed_origin_local(&self) -> Vec2D {
        self.layout_translation()
            + *self.base.base.base.base.base.base.transform() * self.local_anchor()
    }
    pub fn shape_world_transform(&self) -> Mat2D {
        *self.base.base.base.base.world_transform()
    }
    pub fn artboard_handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.artboard_handle()
    }
    pub fn computed_local_x(&self) -> f32 {
        self.computed_origin_local().x
    }
    pub fn computed_local_y(&self) -> f32 {
        self.computed_origin_local().y
    }
    pub fn computed_world_x(&self) -> f32 {
        (*self.base.base.base.base.world_transform() * self.local_anchor()).x
    }
    pub fn computed_world_y(&self) -> f32 {
        (*self.base.base.base.base.world_transform() * self.local_anchor()).y
    }
    pub fn computed_root_x(&self) -> f32 {
        let point = *self.base.base.base.base.world_transform() * self.local_anchor();
        self.artboard_handle()
            .expect("computedRootX requires an artboard")
            .with_downcast_mut::<Artboard, _>(|artboard| artboard.root_transform(point).x)
            .expect("computedRootX requires a live artboard")
    }
    pub fn computed_root_y(&self) -> f32 {
        let point = *self.base.base.base.base.world_transform() * self.local_anchor();
        self.artboard_handle()
            .expect("computedRootY requires an artboard")
            .with_downcast_mut::<Artboard, _>(|artboard| artboard.root_transform(point).y)
            .expect("computedRootY requires a live artboard")
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
        self.base.base.base.base.base.x()
    }
    pub fn y(&self) -> f32 {
        self.base.base.base.base.base.y()
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
        let inner = self.layout.width() - self.layout_padding.left() - self.layout_padding.right();
        if self.css_border_geometry { (inner - self.layout_border.left() - self.layout_border.right()).max(0.0) }
        else { inner }
    }
    pub fn inner_height(&self) -> f32 {
        let inner = self.layout.height() - self.layout_padding.top() - self.layout_padding.bottom();
        if self.css_border_geometry { (inner - self.layout_border.top() - self.layout_border.bottom()).max(0.0) }
        else { inner }
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
        self.is_artboard()
    }
    fn is_artboard(&self) -> bool {
        self.base.handle().and_then(|handle| handle.core_type())
            == Some(
                crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
            )
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
        self.set_layout_flag(LayoutComponentFlags::HasForegroundDrawable, true);
    }
    pub fn mark_position_left_changed(&mut self) {
        self.set_layout_flag(LayoutComponentFlags::PositionLeftChanged, true);
    }
    pub fn mark_position_top_changed(&mut self) {
        self.set_layout_flag(LayoutComponentFlags::PositionTopChanged, true);
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
                    paint.blend_mode(blend.into());
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
        self.hit_test_point_with_origin(position, skip_on_unclipped, primary, None)
    }
    pub(crate) fn hit_test_point_with_origin(
        &mut self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        primary: bool,
        root_origin: Option<Vec2D>,
    ) -> bool {
        let mut inverse = Mat2D::default();
        if !self.base.world_transform().invert(&mut inverse) {
            return false;
        }
        if !(skip_on_unclipped && !self.base.clip()) {
            let mut local = inverse * *position;
            if let Some(origin) = root_origin {
                local += origin;
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
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            self.paints.propagate_opacity(child_opacity);
        }
        let needs_layout_constraints = self.base.base.base.base.base.parent_handle().is_some()
            && value.contains(ComponentDirt::WORLD_TRANSFORM);
        if needs_layout_constraints {
            // Not left to Super's Transform-dirt pass: the pivot scales by the
            // solved size, so a re-solve alone can stale it.
            self.update_transform();
            self.compose_world_transform();
        }
        needs_layout_constraints
    }

    /// Called after the most-derived render-path update, preserving the pinned
    /// virtual-call boundary before resetting the position flags.
    pub(crate) fn reset_update_flags(&mut self) {
        self.set_layout_flag(LayoutComponentFlags::PositionLeftChanged, false);
        self.set_layout_flag(LayoutComponentFlags::PositionTopChanged, false);
    }

    pub(crate) fn layout_constraint_handles(&self) -> Vec<CoreHandle> {
        self.provider.layout_constraints().to_vec()
    }
    pub fn width_override(&mut self, width: f32, unit: i32, row: bool) {
        self.width_override = width;
        self.width_unit_value_override = unit as i8;
        self.set_layout_flag(LayoutComponentFlags::ParentIsRow, row);
        self.mark_layout_node_dirty(false);
    }
    pub(crate) fn width_override_occurrence(
        owner: &CoreHandle,
        width: f32,
        unit: i32,
        row: bool,
        host: Option<&mut dyn crate::mechanical_port::source::artboard_host::ArtboardHost>,
    ) {
        owner.with_mut(|object| {
            let layout = object.as_layout_component_mut().expect("Layout owner");
            layout.width_override = width;
            layout.width_unit_value_override = unit as i8;
            layout.set_layout_flag(LayoutComponentFlags::ParentIsRow, row);
        });
        Self::mark_layout_node_dirty_with_host_occurrence(owner, false, host);
    }
    pub(crate) fn height_override_occurrence(
        owner: &CoreHandle,
        height: f32,
        unit: i32,
        row: bool,
        host: Option<&mut dyn crate::mechanical_port::source::artboard_host::ArtboardHost>,
    ) {
        owner.with_mut(|object| {
            let layout = object.as_layout_component_mut().expect("Layout owner");
            layout.height_override = height;
            layout.height_unit_value_override = unit as i8;
            layout.set_layout_flag(LayoutComponentFlags::ParentIsRow, row);
        });
        Self::mark_layout_node_dirty_with_host_occurrence(owner, false, host);
    }
    pub(crate) fn set_width_intrinsically_size_override_occurrence(
        owner: &CoreHandle,
        intrinsic: bool,
        host: Option<&mut dyn crate::mechanical_port::source::artboard_host::ArtboardHost>,
    ) {
        owner.with_mut(|object| {
            let layout = object.as_layout_component_mut().expect("Layout owner");
            layout.set_layout_flag(
                LayoutComponentFlags::WidthIntrinsicallySizeOverride,
                intrinsic,
            );
            layout.width_unit_value_override = if intrinsic { 3 } else { 1 };
        });
        Self::mark_layout_node_dirty_with_host_occurrence(owner, false, host);
    }
    pub(crate) fn set_height_intrinsically_size_override_occurrence(
        owner: &CoreHandle,
        intrinsic: bool,
        host: Option<&mut dyn crate::mechanical_port::source::artboard_host::ArtboardHost>,
    ) {
        owner.with_mut(|object| {
            let layout = object.as_layout_component_mut().expect("Layout owner");
            layout.set_layout_flag(
                LayoutComponentFlags::HeightIntrinsicallySizeOverride,
                intrinsic,
            );
            layout.height_unit_value_override = if intrinsic { 3 } else { 1 };
        });
        Self::mark_layout_node_dirty_with_host_occurrence(owner, false, host);
    }
    pub fn height_override(&mut self, height: f32, unit: i32, row: bool) {
        self.height_override = height;
        self.height_unit_value_override = unit as i8;
        self.set_layout_flag(LayoutComponentFlags::ParentIsRow, row);
        self.mark_layout_node_dirty(false);
    }
    pub fn set_parent_is_row(&mut self, row: bool) {
        self.set_layout_flag(LayoutComponentFlags::ParentIsRow, row);
        self.mark_layout_node_dirty(false);
    }
    pub fn set_parent_is_stack(&mut self, is_stack: bool) {
        if self.has_layout_flag(LayoutComponentFlags::ParentIsStack) == is_stack {
            return;
        }
        self.set_layout_flag(LayoutComponentFlags::ParentIsStack, is_stack);
        self.mark_layout_node_dirty(false);
    }
    pub fn set_width_intrinsically_size_override(&mut self, intrinsic: bool) {
        self.set_layout_flag(
            LayoutComponentFlags::WidthIntrinsicallySizeOverride,
            intrinsic,
        );
        self.width_unit_value_override = if intrinsic { 3 } else { 1 };
        self.mark_layout_node_dirty(false);
    }
    pub fn set_height_intrinsically_size_override(&mut self, intrinsic: bool) {
        self.set_layout_flag(
            LayoutComponentFlags::HeightIntrinsicallySizeOverride,
            intrinsic,
        );
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
        let code = self.base.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(style) = context.resolve(self.base.style_id()).filter(|style| {
            style
                .is_type_of(crate::mechanical_port::source::generated::layout::layout_component_style_base::LayoutComponentStyleBase::TYPE_KEY)
        }) else {
            return StatusCode::MissingObject;
        };
        self.style = Some(style.clone());
        self.base.add_child(style.clone());
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
    /// Open the live layout clip for both ordinary and deferred CSS paint.
    /// The caller owns the matching restore and its own ClipSaved bookkeeping.
    fn begin_layout_clip(&mut self, renderer: &mut Renderer) -> Option<bool> {
        if let Some(axis) = self.css_overflow_axis {
            let horizontal = axis == CssOverflowAxis::Horizontal;
            let (start, extent) = self.css_axis_clip_interval(horizontal);
            let world = nuxie_render_api::Mat2D(*self.shape_world_transform().values());
            renderer.save();
            assert!(renderer.clip_axis_transformed(horizontal, start, extent, world),
                "CSS axis overflow requires a compatible renderer and finite layout transform");
            return Some(true);
        }
        if !self.base.clip() { return Some(false); }
        let factory = self.with_artboard(|a| a.factory()).flatten()?;
        if let Some(margin) = self.css_overflow_clip_margin.or_else(|| self.css_border_geometry.then_some(
            CssOverflowClipMargin { origin: CssOverflowClipBox::Padding, pixels: 0.0 })) {
            let (bounds, radii, world) = self.paint_geometry();
            let snapped_outsets = self.css_clip_paint_outsets(margin, bounds, world);
            let rounded = radii.iter().any(|pair| pair[0] > 0. && pair[1] > 0.);
            let outsets = if rounded { margin.insets(self).map(|inset| margin.pixels - inset) }
                else { snapped_outsets };
            let mut raw = RawPath::default();
            assert!(super::css_clip_path::elliptical_outset_path(&mut raw, bounds, radii, outsets),
                "CSS clip margin requires finite rounded clip geometry");
            let paths = self.mutable_render_paths();
            paths.css_clip.rewind_as(false, nuxie_render_api::FillRule::Clockwise);
            paths.css_clip.add_path(&raw, Some(&world));
            renderer.save();
            renderer.clip_path(paths.css_clip.render_path(&factory));
            if rounded && snapped_outsets != outsets {
                // Blink keeps the rounded border clip and snapped overflow
                // rectangle as separate nodes. Their intersection preserves
                // fractional rounded edges without leaking beyond the rect.
                let mut rect = RawPath::default();
                assert!(super::css_clip_path::elliptical_outset_path(&mut rect, bounds, [[0.;2];4], snapped_outsets));
                paths.css_overflow_rect.rewind_as(false, nuxie_render_api::FillRule::Clockwise);
                paths.css_overflow_rect.add_path(&rect, Some(&world));
                renderer.clip_path(paths.css_overflow_rect.render_path(&factory));
            }
        } else {
            renderer.save();
            renderer.clip_path(self.mutable_render_paths().world.render_path(&factory));
        }
        Some(true)
    }

    fn css_clip_paint_outsets(&self, margin: CssOverflowClipMargin, bounds: Aabb, world: Mat2D) -> [f32;4] {
        let outsets = margin.insets(self).map(|inset| margin.pixels - inset);
        let values = world.values();
        if !self.css_pixel_bounds || values[0] != 1. || values[1] != 0.
            || values[2] != 0. || values[3] != 1. { return outsets; }
        // Derive the clip from unrounded layout dimensions, then snap once.
        // Applying fractional insets to the already-snapped outer paint box
        // would introduce a second rounding step and shift clip edges.
        let snap = |edge: f32, origin: f32| (edge + origin + 0.5).floor() - origin;
        [bounds.min_x - snap(-outsets[0], values[4]),
         bounds.min_y - snap(-outsets[1], values[5]),
         snap(self.layout.width() + outsets[2], values[4]) - bounds.max_x,
         snap(self.layout.height() + outsets[3], values[5]) - bounds.max_y]
    }

    /// Open an ancestral CSS clip without painting its background or changing
    /// ClipSaved, which belongs to the ordinary proxy/authored draw pair.
    /// None suppresses the deferred group; true requires one renderer restore.
    pub(crate) fn begin_css_ancestor_clip(&mut self, renderer: &mut Renderer) -> Option<bool> {
        if self.is_hidden() || self.is_collapsed() { return None; }
        self.begin_layout_clip(renderer)
    }

    pub fn draw_proxy(&mut self, renderer: &mut Renderer) {
        // A content-edge clip can lie inside the background. CSS clips the
        // descendants, so open this override only after painting the box.
        let defer_margin_clip = (self.css_overflow_clip_margin.is_some()
            && self.css_overflow_axis.is_none() && self.base.clip())
            || (self.css_border_geometry && (self.base.clip() || self.css_overflow_axis.is_some()));
        let save_for_clip = !defer_margin_clip && self.begin_layout_clip(renderer)
            .expect("a drawable LayoutComponent has its imported factory");
        self.set_layout_flag(LayoutComponentFlags::ClipSaved, save_for_clip);
        let world = self.shape_world_transform();
        let mut paint_index = 0;
        while let Some(paint) = self.paints.shape_paints().get(paint_index).cloned() {
            paint_index += 1;
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
                        self.local_path()
                    }
                    ShapePaintPathKind::World => self.world_path(),
                };
                let Some(path) = path else {
                    return;
                };
                paint
                    .shape_paint_mut()
                    .draw_with_fill_rule(renderer, path, world, false, None, true, fill_rule);
            });
        }
        self.draw_css_gradient(renderer);
        self.draw_css_border(renderer);
        if defer_margin_clip {
            let saved = self.begin_layout_clip(renderer)
                .expect("a drawable LayoutComponent has its imported factory");
            self.set_layout_flag(LayoutComponentFlags::ClipSaved, saved);
        }
    }
    pub fn draw(&mut self, renderer: &mut Renderer) {
        if self.has_layout_flag(LayoutComponentFlags::ClipSaved) {
            self.set_layout_flag(LayoutComponentFlags::ClipSaved, false);
            renderer.restore();
        }
    }
    /// Per-occurrence override; None restores the original Rive sizing policy.
    /// Releases the owner borrow before synchronizing retained layout state.
    pub fn set_css_align_self_occurrence(owner: &CoreHandle, alignment: Option<CssAlignSelf>) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_align_self != alignment;
            layout.css_align_self = alignment;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Per-occurrence override; None restores the original Rive line alignment.
    /// Releases the owner borrow before synchronizing retained layout state.
    pub fn set_css_align_content_occurrence(owner: &CoreHandle, alignment: Option<CssAlignContent>) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_align_content != alignment;
            layout.css_align_content = alignment;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Per-occurrence override; None restores the original Rive main-axis alignment.
    /// Releases the owner borrow before synchronizing retained layout state.
    pub fn set_css_justify_content_occurrence(owner: &CoreHandle, alignment: Option<CssJustifyContent>) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_justify_content != alignment;
            layout.css_justify_content = alignment;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Installs independent factors; None restores ordinary Rive sizing.
    pub fn set_css_flex_factors_occurrence(owner: &CoreHandle, factors: Option<CssFlexFactors>) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_flex_factors != factors;
            layout.css_flex_factors = factors;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Select content-box dimensions for this occurrence. False restores the
    /// original border-box policy; cloning retains the occurrence's selection.
    pub fn set_css_content_box_occurrence(owner: &CoreHandle, enabled: bool) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_content_box != enabled;
            layout.css_content_box = enabled;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Select the content box for preferred-ratio calculations only. Authored
    /// width/height retain their own box sizing; clones retain this policy.
    pub fn set_css_ratio_content_box_occurrence(owner: &CoreHandle, enabled: bool) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_ratio_content_box != enabled || !layout.css_ratio_size_limit;
            layout.css_ratio_content_box = enabled;
            layout.css_ratio_size_limit = true;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Preserve a computed CSS ratio pair across runtime layout and cloning.
    pub fn set_css_ratio_pair_occurrence(owner: &CoreHandle, pair: Option<[u32; 2]>) -> bool {
        if pair.is_some_and(|p| p.iter().any(|v| *v == 0 || *v > i32::MAX as u32)) { return false; }
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_ratio_pair != pair;
            layout.css_ratio_pair = pair;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Install CSS percentage spacing for this occurrence's layout tree.
    pub fn set_css_percentage_spacing_occurrence(owner: &CoreHandle, enabled: bool) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_percentage_spacing != enabled;
            layout.css_percentage_spacing = enabled;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    pub(crate) fn has_absolute_position_wire(&self) -> bool {
        self.with_style(|style| style.position_type() == YGPositionType::Absolute).unwrap_or(false)
    }
    /// Enable CSS containing-block ancestry for this occurrence. Some(false)
    /// is authored static, Some(true) establishes a containing block, and None
    /// retains legacy layout. The position wire still determines absolute flow.
    pub fn set_css_positioned_occurrence(owner: &CoreHandle, positioned: Option<bool>) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_positioned != positioned;
            layout.css_positioned = positioned;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Resolve this occurrence's relative insets using the final containing size.
    pub fn set_css_relative_position_occurrence(owner: &CoreHandle, enabled: bool) -> bool {
        let changed = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let changed = layout.css_relative_position != enabled;
            layout.css_relative_position = enabled;
            Some(changed)
        }).flatten();
        let Some(changed) = changed else { return false; };
        if changed {
            Self::sync_style_occurrence(owner);
            Self::mark_layout_node_dirty_occurrence(owner, false);
            Self::mark_layout_style_dirty_occurrence(owner);
        }
        true
    }
    /// Paint uses live schema border widths; no imported paint is modified.
    pub fn set_css_border_color(&mut self, color: Option<u32>) {
        if self.css_border_color != color || self.css_border_side_colors.is_some() {
            self.css_border_side_colors = None;
            self.css_border_color = color;
            if color.is_some() { self.mark_clip_may_be_dynamic(); }
            self.update_render_path();
        }
    }
    /// Diagnostic colors in top/right/bottom/left order, installed after the
    /// checked uniform policy supplies border geometry and drawable proxies.
    pub fn set_experimental_css_border_side_colors(&mut self, colors: Option<[u32; 4]>) {
        if self.css_border_side_colors != colors {
            self.css_border_side_colors = colors;
            self.update_render_path();
        }
    }
    pub fn set_experimental_css_gradient(&mut self, gradient: Option<super::css_linear_gradient::CssLinearGradient>) {
        self.css_gradient = gradient;
    }

    fn draw_css_gradient(&mut self, renderer: &mut Renderer) {
        let Some(gradient) = self.css_gradient.as_ref() else { return; };
        let (bounds, radii, world) = self.paint_geometry();
        let left = self.layout_border.left();
        let top = self.layout_border.top();
        let width = (bounds.width() - left - self.layout_border.right()).max(0.);
        let height = (bounds.height() - top - self.layout_border.bottom()).max(0.);
        if width == 0. || height == 0. { return; }
        let resolved = gradient.resolve(width, height).expect("CSS gradient exceeds finite paint geometry");
        let Some(factory) = self.with_artboard(|a| a.factory()).flatten() else { return; };
        let opacity = self.base.base.render_opacity();
        let colors = resolved.colors.iter().map(|color|
            super::shapes::paint::color::color_modulate_opacity(*color, opacity)).collect::<Vec<_>>();
        let origin_x = bounds.min_x + left;
        let origin_y = bounds.min_y + top;
        let tiled = left != 0. || top != 0. || self.layout_border.right() != 0. || self.layout_border.bottom() != 0.;
        let shader = factory.with_factory_mut(|factory| {
            if tiled {
                factory.make_tiled_premultiplied_linear_gradient(
                    resolved.start[0] + origin_x, resolved.start[1] + origin_y,
                    resolved.end[0] + origin_x, resolved.end[1] + origin_y,
                    [origin_x, origin_y, width, height], &colors, &resolved.positions)
            } else {
                factory.make_premultiplied_linear_gradient(
                    resolved.start[0] + origin_x, resolved.start[1] + origin_y,
                    resolved.end[0] + origin_x, resolved.end[1] + origin_y,
                    &colors, &resolved.positions)
            }
        });
        let mut raw = RawPath::default();
        assert!(super::css_clip_path::elliptical_outset_path(&mut raw, bounds, radii, [0.; 4]));
        let blend = self.base.base.blend_mode();
        let paths = self.mutable_render_paths();
        paths.css_gradient.rewind_as(false, nuxie_render_api::FillRule::NonZero);
        paths.css_gradient.add_path(&raw, None);
        let paint = paths.css_gradient_paint.get_or_insert_with(|| factory.with_factory_mut(|f| f.make_render_paint()));
        let shader = shader.expect("CSS gradients require premultiplied interpolation support");
        paint.shader(Some(shader.as_ref()));
        paint.blend_mode(blend.into());
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(*world.values()));
        renderer.draw_path(paths.css_gradient.render_path(&factory), paint.as_ref());
        renderer.restore();
    }

    fn draw_css_border(&mut self, renderer: &mut Renderer) {
        let Some(color) = self.css_border_color else { return; };
        let side_colors = self.css_border_side_colors;
        if side_colors.map_or(color >> 24 == 0, |colors| colors.iter().all(|c| c >> 24 == 0)) { return; }
        let Some(factory) = self.with_artboard(|a| a.factory()).flatten() else { return; };
        let (bounds, radii, world) = self.paint_geometry();
        let borders = [self.layout_border.left(), self.layout_border.top(),
            self.layout_border.right(), self.layout_border.bottom()];
        let mut raw = RawPath::default();
        assert!(super::css_clip_path::elliptical_border_ring_path(&mut raw, bounds, radii, borders),
            "CSS border requires finite rounded geometry and nonnegative used widths");
        if raw.points().is_empty() { return; }
        let opacity = self.base.base.render_opacity();
        let color = super::shapes::paint::color::color_modulate_opacity(color, opacity);
        let blend = self.base.base.blend_mode();
        let paths = self.mutable_render_paths();
        paths.css_border.rewind_as(false, nuxie_render_api::FillRule::EvenOdd);
        paths.css_border.add_path(&raw, Some(&world));
        let paint = paths.css_border_paint.get_or_insert_with(||
            factory.with_factory_mut(|factory| factory.make_render_paint()));
        paint.style(RenderPaintStyle::Fill);
        paint.color(color);
        paint.blend_mode(blend.into());
        if let Some(colors) = side_colors.filter(|colors| colors.iter().any(|c| *c != colors[0])) {
            for (side, color) in colors.into_iter().enumerate() {
                if color >> 24 == 0 || colors[..side].contains(&color) { continue; }
                // Paint equal-color partitions together. Separate antialiased
                // clips otherwise leave a light seam along their shared edge.
                paths.css_border_side.rewind_as(false, nuxie_render_api::FillRule::NonZero);
                for (matching_side, matching_color) in colors.iter().enumerate() {
                    if *matching_color != color { continue; }
                    let mut clip = RawPath::default();
                    assert!(super::css_clip_path::elliptical_border_side_clip_path(&mut clip, bounds, radii, borders, matching_side));
                    paths.css_border_side.add_path(&clip, Some(&world));
                }
                paint.color(super::shapes::paint::color::color_modulate_opacity(color, opacity));
                renderer.save();
                renderer.clip_path(paths.css_border_side.render_path(&factory));
                renderer.draw_path(paths.css_border.render_path(&factory), paint.as_ref());
                renderer.restore();
            }
        } else {
            if let Some(colors) = side_colors {
                paint.color(super::shapes::paint::color::color_modulate_opacity(colors[0], opacity));
            }
            renderer.draw_path(paths.css_border.render_path(&factory), paint.as_ref());
        }
    }
    /// Border-aware content measurements and descendant clips. Ordinary Rive
    /// behavior remains unchanged until a checked CSS host opts in.
    pub fn set_css_border_geometry(&mut self, enabled: bool) {
        if self.css_border_geometry != enabled {
            self.css_border_geometry = enabled;
            self.mark_layout_node_dirty(false);
            self.clip_changed();
        }
    }
    fn css_axis_clip_interval(&self, horizontal: bool) -> (f32, f32) {
        let extent = if horizontal { self.layout_width() } else { self.layout_height() };
        if !self.css_border_geometry { return (0.0, extent); }
        let (start, end) = if horizontal { (self.layout_border.left(), self.layout_border.right()) }
            else { (self.layout_border.top(), self.layout_border.bottom()) };
        (start, (extent - end).max(start))
    }
    /// Opt-in CSS paint geometry. Logical layout and hit bounds remain unchanged.
    pub fn set_css_pixel_bounds(&mut self, enabled: bool) {
        if self.css_pixel_bounds != enabled {
            self.css_pixel_bounds = enabled;
            self.update_render_path();
        }
    }
    /// Experimental occurrence policy; checked compiler/host transport is pending.
    pub fn set_experimental_css_corner_radii(&mut self, radii: Option<super::css_corner_radii::CssCornerRadii>) {
        if self.css_corner_radii != radii {
            self.css_corner_radii = radii;
            self.update_render_path();
            self.clip_changed();
        }
    }
    fn paint_geometry(&self) -> (Aabb, [[f32; 2]; 4], Mat2D) {
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
        let mut bounds = Aabb::new(0.0, 0.0, self.layout.width(), self.layout.height());
        let world = *self.base.base.base.base.world_transform();
        let values = world.values();
        if self.css_pixel_bounds && values[0] == 1.0 && values[1] == 0.0
            && values[2] == 0.0 && values[3] == 1.0
        {
            let snap = |edge: f32, origin: f32| (edge + origin + 0.5).floor() - origin;
            let min_x = snap(bounds.min_x, values[4]);
            let min_y = snap(bounds.min_y, values[5]);
            // Blink's SnapSizeToPixel retains a pixel when a nonzero extent
            // exceeds four raw LayoutUnit steps (each step is 1/64 CSS px).
            // Keep this paint-only: layout and hit bounds retain their size.
            let preserve_extent = |min: f32, max: f32, size: f32| {
                if max == min && (size * 64.0).trunc() > 4.0 {
                    min + 1.0
                } else {
                    max
                }
            };
            bounds = Aabb::new(min_x, min_y,
                preserve_extent(min_x, snap(bounds.max_x, values[4]), self.layout.width()),
                preserve_extent(min_y, snap(bounds.max_y, values[5]), self.layout.height()));
        }
        let radii = match self.css_corner_radii {
            Some(authored) => authored.resolve_for_paint(self.layout.width(), self.layout.height(), bounds.width(), bounds.height())
                .expect("CSS corner values must resolve to finite border-box geometry"),
            None => radii.map(|radius| [radius, radius]),
        };
        (bounds, radii, world)
    }
    pub fn update_render_path(&mut self) {
        {
            if self.is_hidden()
                || (self.paints.shape_paints().is_empty()
                    && !self.base.clip()
                    && !self.has_layout_flag(LayoutComponentFlags::HasForegroundDrawable))
            {
                return;
            }
            let (bounds, radii, world) = self.paint_geometry();
            let css_corners = self.css_corner_radii.is_some()
                || (self.css_pixel_bounds && radii.iter().any(|r| *r != radii[0]));
            let paths = self.mutable_render_paths();
            paths.background.rewind();
            if css_corners {
                // CSS scales every corner by one common overlap factor. The
                // native rounded rectangle clamps each corner independently.
                assert!(super::css_clip_path::elliptical_outset_path(&mut paths.background, bounds, radii, [0.; 4]),
                    "CSS corners require finite rounded geometry");
            } else {
                Path::add_rounded_rect(&mut paths.background, bounds, radii.map(|r| r[0]));
            }
            paths.local.rewind();
            paths.local.add_path(&paths.background, None);
            paths
                .world
                .rewind_as(false, nuxie_render_api::FillRule::Clockwise);
            paths.world.add_path(&paths.background, Some(&world));
            for paint in self.paints.shape_paints().iter().cloned() {
                let should_draw = paint
                    .with_mut(|paint| {
                        paint
                            .as_shape_paint_behavior_mut()
                            .is_some_and(|paint| paint.should_draw())
                    })
                    .unwrap_or(false);
                if should_draw {
                    crate::mechanical_port::source::shapes::paint::effects_container::invalidate_effects_handle(
                        &paint, None,
                    );
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
            self.has_layout_flag(LayoutComponentFlags::ParentIsRow)
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
    pub fn is_stack_container(&self) -> bool {
        self.with_style(LayoutComponentStyle::is_stack)
            .unwrap_or(false)
    }
    pub fn layout_node_key(&self, index: usize) -> Option<LayoutNodeKey> {
        let provider = self.base.base.base.base.base.handle()?;
        (index == 0).then(|| self.provider.node_key(provider, index))
    }
    pub fn is_leaf(&self) -> bool {
        Self::layout_providers_children(self.base.children(), false).is_empty()
    }
    fn style_sync_context(
        &mut self,
        active_parent_style: Option<&crate::mechanical_port::source::layout::layout_style_applier::LayoutParentStyleSnapshot>,
    ) -> Option<LayoutSyncContext> {
        let style = self.style_handle()?;
        let parent = self.layout_parent_handle();
        let active_parent_style =
            active_parent_style.filter(|snapshot| parent.as_ref() == Some(&snapshot.owner));
        let parent_style = parent.as_ref().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_layout_component()
                        .and_then(LayoutComponent::style_handle)
                })
                .flatten()
        });
        let (parent_is_grid, parent_is_stack, container_justify_items) = active_parent_style
            .map(|snapshot| {
                (
                    snapshot.is_grid,
                    snapshot.is_stack,
                    snapshot.justify_items as u8,
                )
            })
            .or_else(|| {
                parent_style.as_ref().and_then(|style| {
                    style.with_downcast::<LayoutComponentStyle, _>(|style| {
                        (
                            style.is_grid(),
                            style.is_stack(),
                            style.base.justify_items_value(),
                        )
                    })
                })
            })
            .unwrap_or((
                self.can_have_overrides()
                    && self.has_layout_flag(LayoutComponentFlags::ParentIsStack),
                self.can_have_overrides()
                    && self.has_layout_flag(LayoutComponentFlags::ParentIsStack),
                crate::mechanical_port::source::layout::layout_style_applier::YGJustify::Stretch
                    as u8,
            ));
        let inline_hugs = style
            .with_downcast::<LayoutComponentStyle, _>(|style| {
                style.width_scale_type() == LayoutScaleType::Hug
            })
            .unwrap_or(false);
        Some(LayoutSyncContext {
            parent_is_grid,
            parent_is_stack,
            container_justify_items: u32::from(container_justify_items),
            inline_hugs,
            parent_is_row: if self.can_have_overrides() {
                self.has_layout_flag(LayoutComponentFlags::ParentIsRow)
            } else if let Some(snapshot) = active_parent_style {
                snapshot.is_row
            } else {
                self.effective_parent_is_row()
            },
            is_ltr: self.actual_direction() != LayoutDirection::Rtl,
            has_layout_parent: parent.is_some(),
        })
    }
    pub fn sync_style(&mut self) {
        let Some(context) = self.style_sync_context(None) else {
            return;
        };
        let mut taffy_style = std::mem::take(&mut self.layout_data.style);
        let this = self.base.handle();
        let appliers = self
            .layout_data
            .appliers
            .as_deref()
            .cloned()
            .unwrap_or_default();
        for pass in 0..4 {
            for applier in &appliers {
                let mut apply = |applier: &dyn LayoutStyleApplier| match pass {
                    0 => applier.apply_base_style(&mut taffy_style, &context),
                    1 => applier.apply_container_style(&mut taffy_style, &context),
                    2 => applier.apply_item_style(&mut taffy_style, &context),
                    _ => applier.apply_placement_style(&mut taffy_style, &context),
                };
                if this.as_ref() == Some(applier) {
                    apply(self);
                } else {
                    applier.with(|object| {
                        if let Some(applier) = object.as_layout_style_applier() {
                            apply(applier);
                        }
                    });
                }
            }
        }
        self.layout_data.style = taffy_style;
        self.layout_data.dirty = true;
        for (child, provider) in Self::layout_providers_children(self.base.children(), false) {
            let excluded = matches!(child.core_type(), Some(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY | crate::mechanical_port::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY | crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY));
            if !excluded {
                Self::sync_provider_style_occurrence(&provider);
            }
        }
    }
    pub fn taffy_style(&self) -> taffy::style::Style {
        self.layout_data.style.taffy_style()
    }
    pub fn is_intrinsic_leaf(&self) -> bool {
        self.is_leaf()
            && self
                .with_style(|style| style.intrinsically_sized())
                .unwrap_or(false)
    }
    pub fn set_solved_layout(&mut self, layout: Layout, padding: LayoutPadding) {
        self.layout_data.solved_layout = layout;
        self.solved_padding = padding;
        self.solved_border = LayoutPadding::default();
        self.layout_data.has_new_layout = true;
    }
    pub fn clear_layout_children(&mut self) {
        self.mark_layout_tree_topology_dirty();
        let detached = std::mem::take(&mut self.layout_children);
        #[cfg(feature = "tools")]
        self.layout_data.clear_children();
        let owner = self.base.base.base.base.base.handle();
        Self::clear_detached_layout_ownership(owner.as_ref(), &detached);
    }
    fn mark_layout_tree_topology_dirty(&mut self) {
        self.layout_tree_topology_dirty = true;
        let this = self.base.base.base.base.base.handle();
        let mut active = this.into_iter().collect::<Vec<_>>();
        let mut current = self
            .layout_node_key(0)
            .and_then(|node| node.owner.borrow().clone());
        while let Some(node_owner) = current {
            assert!(
                !active.contains(&node_owner),
                "cyclic layout node ownership"
            );
            active.push(node_owner.clone());
            current = node_owner
                .with_mut(|object| {
                    let layout = object.as_layout_component_mut().expect("Layout owner");
                    layout.layout_tree_topology_dirty = true;
                    let node = layout.layout_node_key(0)?;
                    let parent = node.owner.borrow().clone();
                    parent
                })
                .flatten();
        }
    }
    fn clear_detached_layout_ownership(owner: Option<&CoreHandle>, nodes: &[LayoutNodeKey]) {
        let owns_children = owner.is_some_and(|owner| {
            nodes.first().is_some_and(|first| {
                first
                    .owner
                    .borrow()
                    .as_ref()
                    .is_some_and(|value| value == owner)
            })
        });
        if !owns_children {
            return;
        }
        for node in nodes {
            *node.owner.borrow_mut() = None;
            let Some(node_owner) = layout_node_owner_for(node) else {
                continue;
            };
            node_owner.with_mut(|object| {
                if let Some(layout) = object.as_layout_component_mut() {
                    // YGNodeRemoveAllChildren replaces the cached YGLayout with
                    // a default layout while deliberately retaining hasNewLayout.
                    layout.layout_data.solved_layout =
                        Layout::new(0.0, 0.0, f32::NAN, f32::NAN);
                    layout.solved_padding = LayoutPadding::default();
                    layout.solved_border = LayoutPadding::default();
                } else if let Some(participant) = object
                    .as_any_mut()
                    .downcast_mut::<crate::mechanical_port::source::layout::layout_participant::LayoutParticipant>()
                {
                    if let Some(data) = participant.native_layout_data_mut() {
                        data.solved_layout = Layout::new(0.0, 0.0, f32::NAN, f32::NAN);
                    }
                }
            });
        }
    }
    pub fn sync_layout_children(&mut self) {
        self.clear_layout_children();
        for (_, provider) in Self::layout_providers_children(self.base.children(), false) {
            let count = provider
                .with_mut(|object| {
                    object
                        .as_layout_node_provider_mut()
                        .expect("layout provider")
                        .num_layout_nodes()
                })
                .unwrap_or(0);
            for index in 0..count {
                if let Some(node) =
                    crate::mechanical_port::source::layout::layout_node_provider::layout_node_for(
                        &provider, index,
                    )
                {
                    if let Some(owner) = self.base.base.base.base.base.handle() {
                        *node.owner.borrow_mut() = Some(owner);
                    }
                    self.layout_children.push(node);
                }
            }
        }
        #[cfg(feature = "tools")]
        {
            self.layout_data.children = self
                .layout_children
                .iter()
                .map(|node| node.provider.clone())
                .collect();
        }
        self.mark_layout_node_dirty(false);
    }
    pub fn propagate_size(&mut self) {
        let direction = self.actual_direction();
        let style = self.with_style(|style| {
            (
                style.width_scale_type(),
                style.height_scale_type(),
                direction,
            )
        });
        Self::propagate_size_to_children(
            self.base.children().to_vec(),
            self.is_hidden(),
            Vec2D::new(self.layout.width(), self.layout.height()),
            style,
        );
    }
    // Plain groups contain free content; Solo children still receive the
    // enclosing layout's content size. Provider collection remains separately
    // transparent through both groups and Solos.
    fn stops_content_sizing(
        component: &dyn crate::mechanical_port::source::core::CoreObject,
    ) -> bool {
        component.core_type()
            == crate::mechanical_port::source::generated::node_base::NodeBase::TYPE_KEY
    }

    fn propagate_size_to_children(
        children: Vec<CoreHandle>,
        hidden: bool,
        size: Vec2D,
        style: Option<(LayoutScaleType, LayoutScaleType, LayoutDirection)>,
    ) {
        if hidden {
            return;
        }
        for child in children {
            let skip = child
                .with(|child| {
                    child.as_layout_component().is_some()
                        || Self::stops_content_sizing(child)
                        || child.layout_provider_handle().is_some()
                })
                .unwrap_or(true);
            if skip {
                continue;
            }
            let propagate = if let Some((width, height, direction)) = style {
                let controlled =
                    crate::mechanical_port::source::intrinsically_sizeable::control_size_handle(
                        &child, size, width, height, direction,
                    );
                !controlled
                    || child
                        .with_mut(|child| {
                            child
                                .as_intrinsically_sizeable_mut()
                                .expect("controlSize resolved an IntrinsicallySizeable")
                                .should_propagate_size_to_children()
                        })
                        .unwrap_or(false)
            } else {
                true
            };
            if propagate {
                if let Some(children) = child
                    .with(|object| {
                        object
                            .as_container_component()
                            .map(|container| container.children().to_vec())
                    })
                    .flatten()
                {
                    Self::propagate_size_to_children(children, false, size, style);
                }
            }
        }
    }

    pub fn propagate_size_occurrence(owner: &CoreHandle) {
        if owner.is_type_of(
            crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
        ) {
            Artboard::propagate_size_handle(owner);
            return;
        }
        let Some((children, hidden, size, style)) = owner.with(|object| {
            let layout = object.as_layout_component().unwrap();
            (
                object.as_container_component().unwrap().children().to_vec(),
                layout.is_hidden(),
                Vec2D::new(layout.layout.width(), layout.layout.height()),
                layout.with_style(|style| {
                    (
                        style.width_scale_type(),
                        style.height_scale_type(),
                        layout.actual_direction(),
                    )
                }),
            )
        }) else {
            return;
        };
        Self::propagate_size_to_children(children, hidden, size, style);
    }
    pub fn layout_solve_available_size(
        &self,
        available_width: f32,
        available_height: f32,
    ) -> Vec2D {
        let intrinsically_sized = self
            .with_style(|style| style.intrinsically_sized())
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

    /// Adapter for the pinned YGNodeCalculateLayout call. The tree contains
    /// actual provider nodes, including complete hosted Artboard subtrees.
    /// Taffy replaces Yoga only at this calculation boundary.
    pub fn calculate_layout_occurrence(
        owner: &CoreHandle,
        available_width: f32,
        available_height: f32,
    ) {
        use crate::mechanical_port::source::layout::layout_participant::LayoutParticipant;
        use taffy::prelude::{AvailableSpace, Dimension, Display, Size, TaffyTree};

        struct TopologyNode {
            owner: CoreHandle,
            children: Vec<usize>,
            measure: Option<LayoutMeasureContext>,
        }

        fn node_state(
            owner: CoreHandle,
        ) -> Option<(taffy::style::Style, Option<LayoutMeasureContext>, bool)> {
            owner.with(|object| {
                if let Some(layout) = object.as_layout_component() {
                    let measure = layout
                        .is_intrinsic_leaf()
                        .then(|| LayoutMeasureContext::Layout(owner.clone()));
                    (layout.taffy_style(), measure, layout.layout_data.dirty)
                } else {
                    let participant = object
                        .as_any()
                        .downcast_ref::<LayoutParticipant>()
                        .expect("native layout node owner");
                    let data = participant
                        .native_layout_data()
                        .expect("participating node");
                    let host = participant
                        .measurement_host_handle()
                        .expect("participant host");
                    (
                        data.style.taffy_style(),
                        Some(LayoutMeasureContext::Participant(host)),
                        data.dirty,
                    )
                }
            })
        }

        fn collect_topology(
            owner: CoreHandle,
            nodes: &mut Vec<TopologyNode>,
            active: &mut Vec<CoreHandle>,
        ) -> usize {
            assert!(!active.contains(&owner), "cyclic layout node ownership");
            active.push(owner.clone());
            let children = owner
                .with(|object| {
                    if let Some(layout) = object.as_layout_component() {
                        layout.layout_children.clone()
                    } else {
                        Vec::new()
                    }
                })
                .expect("live layout node");
            let mut child_indices = Vec::new();
            for child in children {
                if let Some(child) = layout_node_owner_for(&child) {
                    child_indices.push(collect_topology(child, nodes, active));
                }
            }
            let measure = node_state(owner.clone()).expect("live layout node").1;
            assert!(
                measure.is_none() || child_indices.is_empty(),
                "a measured Rive layout is a leaf"
            );
            let index = nodes.len();
            nodes.push(TopologyNode {
                owner,
                children: child_indices,
                measure,
            });
            active.pop();
            index
        }

        fn build_cache(owner: CoreHandle) -> LayoutTreeCache {
            let mut topology = Vec::new();
            let root = collect_topology(owner, &mut topology, &mut Vec::new());
            let mut tree = TaffyTree::<LayoutMeasureContext>::new();
            tree.disable_rounding();
            let mut nodes: Vec<CachedLayoutNode> = Vec::with_capacity(topology.len());
            for entry in topology {
                let children = entry
                    .children
                    .iter()
                    .map(|index| nodes[*index].node)
                    .collect::<Vec<_>>();
                let node = if let Some(measure) = entry.measure.clone() {
                    tree.new_leaf_with_context(taffy::style::Style::default(), measure)
                } else {
                    tree.new_with_children(taffy::style::Style::default(), &children)
                }
                .expect("valid native layout node");
                nodes.push(CachedLayoutNode {
                    owner: entry.owner,
                    node,
                    children: entry.children,
                    measure: entry.measure,
                });
            }
            LayoutTreeCache { tree, nodes, root, css_baselines: false }
        }

        fn axis(known: Option<f32>, available: AvailableSpace) -> (f32, LayoutMeasureMode) {
            if let Some(value) = known {
                return (value, LayoutMeasureMode::Exactly);
            }
            match available {
                AvailableSpace::Definite(value) => (value, LayoutMeasureMode::AtMost),
                // A zero available extent requests the smallest unbreakable
                // contribution. Undefined requests the unwrapped maximum.
                AvailableSpace::MinContent => (0.0, LayoutMeasureMode::AtMost),
                AvailableSpace::MaxContent => (f32::NAN, LayoutMeasureMode::Undefined),
            }
        }
        fn measure_host(
            host: &CoreHandle,
            width: f32,
            width_mode: LayoutMeasureMode,
            height: f32,
            height_mode: LayoutMeasureMode,
        ) -> Vec2D {
            host.with_mut(|object| {
                object
                    .as_intrinsically_sizeable_mut()
                    .map(|sizeable| sizeable.measure_layout(width, width_mode, height, height_mode))
            })
            .flatten()
            .unwrap_or_default()
        }

        let size = owner
            .with(|object| {
                object
                    .as_layout_component()
                    .unwrap()
                    .layout_solve_available_size(available_width, available_height)
            })
            .expect("layout calculation owner");
        let (cached, topology_dirty) = owner
            .with_mut(|object| {
                let layout = object
                    .as_layout_component_mut()
                    .expect("layout calculation owner");
                let topology_dirty = layout.layout_tree_topology_dirty;
                layout.layout_tree_topology_dirty = false;
                (layout.layout_tree_cache.take(), topology_dirty)
            })
            .expect("layout calculation owner");
        let mut cache = if topology_dirty {
            build_cache(owner.clone())
        } else {
            cached.unwrap_or_else(|| build_cache(owner.clone()))
        };

        let mut read_states = |cache: &LayoutTreeCache| {
            let mut styles = Vec::with_capacity(cache.nodes.len());
            let mut measures = Vec::with_capacity(cache.nodes.len());
            let mut dirty = Vec::with_capacity(cache.nodes.len());
            for entry in &cache.nodes {
                let (style, measure, node_dirty) = node_state(entry.owner.clone())?;
                styles.push(style);
                measures.push(measure);
                dirty.push(node_dirty);
            }
            Some((styles, measures, dirty))
        };
        let (mut styles, measures, dirty) = if let Some(states) = read_states(&cache) {
            states
        } else {
            // A dynamic list can retire an occurrence between topology sync and
            // the owning root's solve. A Yoga node is destroyed with that
            // occurrence, so discard the corresponding retained Taffy tree.
            cache = build_cache(owner.clone());
            read_states(&cache).expect("rebuilt layout topology contains live nodes")
        };
        // CSS ratio transfer amplifies gap rounding errors in ancestor flex
        // allocation. Apply gap precision throughout this opted-in solve tree,
        // including containers without a ratio of their own. Derive it each
        // solve so original and cloned occurrences use the same policy.
        let css_ratio_tree = styles.iter().any(|style| style.aspect_ratio_size_limit.is_some());
        let css_percentage_spacing = styles.iter().any(|style| style.css_percentage_spacing);
        for style in &mut styles {
            style.css_percentage_spacing = css_percentage_spacing;
            style.quantize_gap = css_ratio_tree;
            style.css_intrinsic_sizing = css_ratio_tree;
            if css_ratio_tree {
                // The compiler admits pixel padding. Ancestor padding is
                // subtracted before descendants receive their available size,
                // so each edge needs CSS precision at this earlier boundary.
                style.padding = style.padding.map(|padding| {
                    let raw = padding.into_raw();
                    if raw.tag() == taffy::style::CompactLength::LENGTH_TAG {
                        taffy::style::LengthPercentage::length((raw.value() * 64.0).floor() / 64.0)
                    } else { padding }
                });
            }
        }
        for parent in 0..cache.nodes.len() {
            if styles[parent].display != Display::Flex {
                continue;
            }
            for child in &cache.nodes[parent].children {
                // Yoga's flex YGNodeBoundAxis enforces explicit min dimensions
                // and padding/border only. Taffy's Auto instead imposes a
                // content-based minimum, preventing a fill viewport from
                // shrinking below its scrolling contents. Adapt this solve
                // node only: Yoga's grid algorithm does have automatic minima.
                if styles[*child].min_size.width.is_auto() {
                    styles[*child].min_size.width = Dimension::length(0.0);
                }
                if styles[*child].min_size.height.is_auto() {
                    styles[*child].min_size.height = Dimension::length(0.0);
                }
            }
        }
        let css_baselines = cache.nodes.iter().any(|entry| entry.owner.with(|object| {
            object.as_layout_component().is_some_and(|layout| layout.css_align_self == Some(CssAlignSelf::Baseline))
        }).unwrap_or(false));
        if cache.css_baselines != css_baselines {
            for entry in &cache.nodes { cache.tree.mark_dirty(entry.node).expect("valid baseline policy node"); }
            cache.css_baselines = css_baselines;
        }
        for index in 0..cache.nodes.len() {
            let entry = &mut cache.nodes[index];
            if entry.measure != measures[index] {
                cache
                    .tree
                    .set_node_context(entry.node, measures[index].clone())
                    .expect("valid native measure context");
                entry.measure = measures[index].clone();
            }
            if cache.tree.style(entry.node).expect("valid native node") != &styles[index] {
                cache
                    .tree
                    .set_style(entry.node, styles[index].clone())
                    .expect("valid native style");
            }
            if dirty[index] {
                cache
                    .tree
                    .mark_dirty(entry.node)
                    .expect("valid dirty native node");
            }
        }

        let root = cache.nodes[cache.root].node;
        let root_style = owner
            .with(|object| {
                object
                    .as_layout_component()
                    .expect("layout calculation owner")
                    .layout_data
                    .style
                    .taffy_calculation_root_style(size.x, size.y)
            })
            .expect("live layout calculation owner");
        if cache.tree.style(root).expect("valid calculation root") != &root_style {
            cache
                .tree
                .set_style(root, root_style)
                .expect("valid calculation root");
        }
        let position_modes: Vec<_> = cache.nodes.iter().map(|entry| entry.owner.with(|object| {
            object.as_layout_component().and_then(|layout| layout.css_positioned)
        }).flatten()).collect();
        let mut parents = vec![None; cache.nodes.len()];
        for (parent, entry) in cache.nodes.iter().enumerate() {
            for child in &entry.children { parents[*child] = Some(parent); }
        }
        // Absolute descendants do not contribute to ancestor flow sizing. A
        // bounded solve per ancestry level propagates updated containing boxes
        // through nested absolute nodes without changing paint parentage.
        for pass in 0..=cache.nodes.len() {
        cache
            .tree
            .compute_layout_with_measure_and_baseline(
            root,
            Size {
                width: if size.x.is_nan() {
                    AvailableSpace::MaxContent
                } else {
                    AvailableSpace::Definite(size.x)
                },
                height: if size.y.is_nan() {
                    AvailableSpace::MaxContent
                } else {
                    AvailableSpace::Definite(size.y)
                },
            },
            |known, available, _, context, _| {
                let (width, width_mode) = axis(known.width, available.width);
                let (height, height_mode) = axis(known.height, available.height);
                let measured = match context {
                    Some(LayoutMeasureContext::Participant(host)) => {
                        measure_host(host, width, width_mode, height, height_mode)
                    }
                    Some(LayoutMeasureContext::Layout(owner)) => {
                        let children = owner
                            .with(|object| {
                                object.as_container_component().unwrap().children().to_vec()
                            })
                            .expect("measurement owner");
                        let mut measured = Vec2D::default();
                        for child in children {
                            let is_layout = child
                                .is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY);
                            if is_layout {
                                continue;
                            }
                            let next = measure_host(&child, width, width_mode, height, height_mode);
                            if measured.x < next.x {
                                measured.x = next.x;
                            }
                            if measured.y < next.y {
                                measured.y = next.y;
                            }
                        }
                        measured
                    }
                    None => Vec2D::default(),
                };
                Size {
                    width: known.width.unwrap_or(measured.x),
                    height: known.height.unwrap_or(measured.y),
                }
            },
            if css_baselines { Some(|_, context, _| {
                match context? {
                    LayoutMeasureContext::Participant(host) => host.with(|object| object.as_text().and_then(|text| text.measured_css_baseline())).flatten(),
                    LayoutMeasureContext::Layout(_) => None,
                }
            }) } else { None },
        )
        .expect("valid native layout calculation");

            let mut updates = Vec::new();
            for (index, entry) in cache.nodes.iter().enumerate() {
                let style = cache.tree.style(entry.node).expect("live CSS position node");
                if position_modes[index].is_none() || style.position != taffy::style::Position::Absolute { continue; }
                let Some(parent) = parents[index] else { continue; };
                let mut ancestor = parent;
                let mut offset = taffy::geometry::Point { x: 0.0, y: 0.0 };
                while ancestor != cache.root && position_modes[ancestor] != Some(true) {
                    let layout = cache.tree.unrounded_layout(cache.nodes[ancestor].node);
                    offset.x += layout.location.x;
                    offset.y += layout.location.y;
                    ancestor = parents[ancestor].expect("non-root CSS ancestor has parent");
                }
                let containing = cache.tree.unrounded_layout(cache.nodes[ancestor].node);
                let context = Some((Size {
                    width: containing.size.width - containing.border.left - containing.border.right,
                    height: containing.size.height - containing.border.top - containing.border.bottom,
                }, taffy::geometry::Point {
                    x: containing.border.left - offset.x,
                    y: containing.border.top - offset.y,
                }));
                if style.css_absolute_containing_block != context {
                    let mut next = style.clone();
                    next.css_absolute_containing_block = context;
                    updates.push((entry.node, next));
                }
            }
            if updates.is_empty() { break; }
            assert!(pass < cache.nodes.len(), "CSS containing-block layout did not converge");
            for (node, style) in updates { cache.tree.set_style(node, style).expect("live CSS containing-block node"); }
        }

        let mut subtree_dirty = dirty.clone();
        // A remote containing block can move a descendant while its immediate
        // parent's own box stays unchanged. Keep that parent traversable when
        // publishing the updated descendant layout.
        for (index, entry) in cache.nodes.iter().enumerate() {
            if position_modes[index].is_none() { continue; }
            let output = cache.tree.layout(entry.node).expect("solved CSS node");
            let next = Layout::new(output.location.x, output.location.y, output.size.width, output.size.height);
            subtree_dirty[index] |= entry.owner.with(|object| object.as_layout_component()
                .is_some_and(|layout| layout.layout_data.solved_layout != next)).unwrap_or(false);
        }
        for index in 0..cache.nodes.len() {
            subtree_dirty[index] |= cache.nodes[index]
                .children
                .iter()
                .any(|child| subtree_dirty[*child]);
        }
        let mut outputs = Vec::with_capacity(cache.nodes.len());
        for (index, entry) in cache.nodes.iter().enumerate() {
            let output = cache.tree.layout(entry.node).expect("solved native node");
            let next = Layout::new(
                output.location.x,
                output.location.y,
                output.size.width,
                output.size.height,
            );
            let padding = LayoutPadding::new(
                output.padding.left,
                output.padding.top,
                output.padding.right,
                output.padding.bottom,
            );
            let border = LayoutPadding::new(output.border.left, output.border.top,
                output.border.right, output.border.bottom);
            outputs.push((entry.owner.clone(), next, padding, border, subtree_dirty[index]));
        }
        owner.with_mut(|object| {
            object
                .as_layout_component_mut()
                .expect("layout calculation owner")
                .layout_tree_cache = Some(cache);
        });
        for (owner, next, padding, border, subtree_dirty) in outputs {
            owner.with_mut(|object| {
                let data = if let Some(layout) = object.as_layout_component_mut() {
                    layout.layout_data.has_new_layout |= layout.solved_padding != padding || layout.solved_border != border;
                    layout.solved_padding = padding;
                    layout.solved_border = border;
                    &mut *layout.layout_data
                } else {
                    object
                        .as_any_mut()
                        .downcast_mut::<LayoutParticipant>()
                        .unwrap()
                        .native_layout_data_mut()
                        .unwrap()
                };
                // Yoga publishes a new layout only for a visited subtree or a
                // changed cached result. Taffy's retained cache now supplies the
                // same visit boundary across root calculations.
                data.has_new_layout |= subtree_dirty || data.solved_layout != next;
                data.solved_layout = next;
                data.dirty = false;
            });
        }
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
        for (_, provider) in Self::layout_providers_children(self.base.children(), false) {
            Self::update_provider_layout_bounds(&provider, animate);
        }
        let next = self.layout_data.solved_layout;
        self.layout_padding = self.solved_padding;
        self.layout_border = self.solved_border;
        if self.has_layout_flag(LayoutComponentFlags::JustAddedToHost) {
            self.set_layout_flag(LayoutComponentFlags::JustAddedToHost, false);
            self.layout = next;
            let data = self.current_animation_data();
            data.from = next;
            data.to = next;
            data.elapsed_seconds = 0.0;
            self.propagate_size();
            CoreCapabilities::world_transform_mark_dirty(self);
            self.set_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds, false);
            return;
        }
        if animate && self.animates() {
            let force = self.has_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds);
            let data = self.current_animation_data();
            if next != data.to || force {
                if data.elapsed_seconds != 0.0 {
                    if self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
                        self.animation_data_a = self.animation_data_b;
                    }
                    self.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, true);
                } else {
                    self.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                }
                let from = self.layout;
                let data = self.current_animation_data();
                data.from = from;
                data.to = next;
                data.elapsed_seconds = 0.0;
                self.propagate_size();
                CoreCapabilities::world_transform_mark_dirty(self);
            }
        } else if next != self.layout
            || self.has_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds)
        {
            if self.layout.width() != next.width() || self.layout.height() != next.height() {
                CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
            }
            self.layout = next;
            self.animation_data_a.to = next;
            self.propagate_size();
            CoreCapabilities::world_transform_mark_dirty(self);
        }
        self.set_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds, false);
    }
    fn hosted_layout_roots(provider: &CoreHandle) -> Option<(bool, Vec<(i32, CoreHandle)>)> {
        provider.with(|object| {
            if let Some(nested) = object.as_any().downcast_ref::<crate::mechanical_port::source::nested_artboard_layout::NestedArtboardLayout>() {
                Some((false, nested.base.base.artboard_instance_handle(0).map(|instance| (0, instance.core_handle())).into_iter().collect()))
            } else if let Some(list) = object.as_any().downcast_ref::<crate::mechanical_port::source::artboard_component_list::ArtboardComponentList>() {
                Some((true, (0..list.artboard_count() as i32).filter_map(|index| list.item(index).map(|instance| (index, instance.core_handle()))).collect()))
            } else { None }
        }).flatten()
    }

    fn update_provider_layout_bounds(provider: &CoreHandle, animate: bool) {
        if provider.is_type_of(crate::mechanical_port::source::generated::layout::layout_participant_base::LayoutParticipantBase::TYPE_KEY) {
            crate::mechanical_port::source::layout::layout_participant::LayoutParticipant::update_layout_bounds_occurrence(provider, animate);
        } else if provider
            .is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY)
        {
            Self::update_layout_bounds_occurrence(provider, animate);
        } else if let Some((is_list, roots)) = Self::hosted_layout_roots(provider) {
            for (index, root) in roots {
                Self::update_layout_bounds_occurrence(&root, animate);
                if is_list {
                    let bounds = root
                        .with(|object| object.as_layout_component().unwrap().layout_bounds())
                        .unwrap();
                    provider.with_mut(|object| object.as_any_mut().downcast_mut::<crate::mechanical_port::source::artboard_component_list::ArtboardComponentList>().unwrap().set_item_size(Vec2D::new(bounds.width(), bounds.height()), index));
                }
            }
            if is_list {
                crate::mechanical_port::source::artboard_component_list::ArtboardComponentList::finish_layout_bounds_occurrence(provider);
            }
        } else {
            provider.with_mut(|object| object.layout_provider_update_layout_bounds(animate));
        }
    }

    pub fn update_layout_bounds_occurrence(owner: &CoreHandle, animate: bool) {
        let updated = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                std::mem::take(&mut layout.layout_data.has_new_layout)
            })
            .unwrap_or(false);
        if !updated {
            return;
        }
        for (_, provider) in Self::layout_providers_occurrence(owner) {
            Self::update_provider_layout_bounds(&provider, animate);
        }
        let (next, old, just_added, animates, force, current) = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                layout.layout_padding = layout.solved_padding;
                layout.layout_border = layout.solved_border;
                (
                    layout.layout_data.solved_layout,
                    layout.layout,
                    layout.has_layout_flag(LayoutComponentFlags::JustAddedToHost),
                    layout.animates(),
                    layout.has_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds),
                    *layout.current_animation_data(),
                )
            })
            .unwrap();
        let mut changed = false;
        if just_added {
            owner.with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                layout.set_layout_flag(LayoutComponentFlags::JustAddedToHost, false);
                layout.layout = next;
                *layout.current_animation_data() = LayoutAnimationData {
                    from: next,
                    to: next,
                    elapsed_seconds: 0.0,
                };
            });
            changed = true;
        } else if animate && animates {
            if next != current.to || force {
                owner.with_mut(|object| {
                    let layout = object.as_layout_component_mut().unwrap();
                    if current.elapsed_seconds != 0.0 {
                        if layout.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
                            layout.animation_data_a = layout.animation_data_b;
                        }
                        layout.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, true);
                    } else {
                        layout.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                    }
                    *layout.current_animation_data() = LayoutAnimationData {
                        from: old,
                        to: next,
                        elapsed_seconds: 0.0,
                    };
                });
                changed = true;
            }
        } else if next != old || force {
            if next.width() != old.width() || next.height() != old.height() {
                owner.with_mut(|object| object.component_add_dirt(ComponentDirt::PATH, false));
            }
            owner.with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                layout.layout = next;
                layout.animation_data_a.to = next;
            });
            changed = true;
        }
        if changed {
            Self::propagate_size_occurrence(owner);
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            )
            .add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
        }
        owner.with_mut(|object| {
            object
                .as_layout_component_mut()
                .unwrap()
                .set_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds, false)
        });
    }

    pub fn cascade_layout_style_occurrence(
        owner: &CoreHandle,
        interpolation: LayoutStyleInterpolation,
        interpolator: Option<CoreHandle>,
        time: f32,
        direction: LayoutDirection,
    ) -> bool {
        let Some((mut updated, direction_changed)) = owner.with_mut(|object| {
            let layout = object.as_layout_component_mut().unwrap();
            let inherits = layout
                .with_style(|style| style.animation_style() == LayoutAnimationStyle::Inherit)
                .unwrap_or(false);
            let updated = if inherits {
                layout.set_inherited_interpolation(interpolation, interpolator, time)
            } else {
                layout.clear_inherited_interpolation();
                false
            };
            let old = layout.inherited_direction;
            layout.inherited_direction = if direction == LayoutDirection::Inherit
                || layout
                    .with_style(|style| style.direction() != YGDirection::Inherit)
                    .unwrap_or(false)
            {
                LayoutDirection::Inherit
            } else {
                direction
            };
            (updated, old != layout.inherited_direction)
        }) else {
            return false;
        };
        if direction_changed {
            Self::mark_layout_node_dirty_occurrence(owner, true);
            owner.with_mut(|object| object.component_add_dirt(ComponentDirt::PATH, false));
            updated = true;
        }
        let (interpolation, interpolator, time, direction) = owner
            .with(|object| {
                let layout = object.as_layout_component().unwrap();
                (
                    layout.interpolation(),
                    layout.interpolator(),
                    layout.interpolation_time(),
                    layout.actual_direction(),
                )
            })
            .unwrap();
        for (_, provider) in Self::layout_providers_occurrence(owner) {
            if provider
                .is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY)
            {
                Self::cascade_layout_style_occurrence(
                    &provider,
                    interpolation,
                    interpolator.clone(),
                    time,
                    direction,
                );
            } else if let Some((_, roots)) = Self::hosted_layout_roots(&provider) {
                for (_, root) in roots {
                    Self::cascade_layout_style_occurrence(
                        &root,
                        interpolation,
                        interpolator.clone(),
                        time,
                        direction,
                    );
                }
            } else {
                provider.with_mut(|object| {
                    object.layout_provider_cascade_style(
                        interpolation,
                        interpolator.clone(),
                        time,
                        direction,
                    )
                });
            }
        }
        updated
    }

    pub fn advance_component_occurrence(
        owner: &CoreHandle,
        elapsed: f32,
        flags: AdvanceFlags,
    ) -> bool {
        if flags.0 & AdvanceFlags::NEW_FRAME.0 == 0
            || owner
                .with(|object| object.as_layout_component().unwrap().is_collapsed())
                .unwrap_or(true)
        {
            return false;
        }
        Self::apply_interpolation_occurrence(
            owner,
            elapsed,
            flags.0 & (AdvanceFlags::ANIMATE.0 | AdvanceFlags::ADVANCE_NESTED.0) != 0,
        )
    }

    pub fn apply_interpolation_occurrence(owner: &CoreHandle, elapsed: f32, animate: bool) -> bool {
        let Some((time, interpolation, interpolator, smoothing, data_a)) = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                let target = layout.layout;
                if !animate || !layout.animates() || layout.current_animation_data().to == target {
                    return None;
                }
                Some((
                    layout.interpolation_time(),
                    layout.interpolation(),
                    layout.interpolator(),
                    layout.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation),
                    layout.animation_data_a,
                ))
            })
            .flatten()
        else {
            return false;
        };
        let factor = |seconds: f32| {
            let factor = 1.0_f32.min(if time > 0.0 { seconds / time } else { 1.0 });
            if interpolation == LayoutStyleInterpolation::Linear {
                return factor;
            }
            interpolator
                .as_ref()
                .and_then(|interpolator| {
                    interpolator
                        .with_mut(|object| object.keyframe_interpolator_transform(factor))
                        .flatten()
                })
                .unwrap_or(factor)
        };
        if smoothing {
            let f = factor(data_a.elapsed_seconds);
            owner.with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                layout.animation_data_b.from = layout.animation_data_a.interpolate(f);
                if f == 1.0 {
                    layout.animation_data_a = layout.animation_data_b;
                    layout.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                } else {
                    layout.animation_data_a.elapsed_seconds += elapsed;
                }
            });
        }
        let (data, old) = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                (*layout.current_animation_data(), layout.layout)
            })
            .unwrap();
        if data.elapsed_seconds >= time {
            if old.width() != data.to.width() || old.height() != data.to.height() {
                owner.with_mut(|object| object.component_add_dirt(ComponentDirt::PATH, false));
            }
            owner.with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                layout.layout = data.to;
                if layout.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
                    layout.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                    layout.animation_data_a = layout.animation_data_b;
                    layout.animation_data_b.elapsed_seconds = 0.0;
                }
                layout.animation_data_a.elapsed_seconds = 0.0;
            });
            Self::propagate_size_occurrence(owner);
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            )
            .add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
            return false;
        }
        let f = factor(data.elapsed_seconds);
        let current = data.interpolate(f);
        if current != old {
            let resized = old.width() != current.width() || old.height() != current.height();
            owner.with_mut(|object| object.as_layout_component_mut().unwrap().layout = current);
            if resized {
                Self::propagate_size_occurrence(owner);
            }
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            )
            .add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
        }
        owner.with_mut(|object| {
            object
                .as_layout_component_mut()
                .unwrap()
                .current_animation_data()
                .elapsed_seconds += elapsed
        });
        if f != 1.0 {
            Self::mark_layout_node_dirty_occurrence(owner, false);
            true
        } else {
            false
        }
    }

    pub fn interrupt_animation_occurrence(owner: &CoreHandle) {
        let changed = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                if !layout.animates() {
                    return false;
                }
                layout.layout = layout.current_animation_data().to;
                true
            })
            .unwrap_or(false);
        if changed {
            Self::propagate_size_occurrence(owner);
        }
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
        if self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
            &mut self.animation_data_b
        } else {
            &mut self.animation_data_a
        }
    }
    pub fn apply_interpolation(&mut self, elapsed: f32, animate: bool) -> bool {
        let target = self.layout;
        if !animate || !self.animates() || self.current_animation_data().to == target {
            return false;
        }
        let time = self.interpolation_time();
        let transform_factor = |seconds: f32,
                                interpolation: LayoutStyleInterpolation,
                                interpolator: Option<CoreHandle>| {
            let factor = 1.0_f32.min(if time > 0.0 { seconds / time } else { 1.0 });
            if interpolation == LayoutStyleInterpolation::Linear {
                return factor;
            }
            interpolator
                .and_then(|interpolator| {
                    interpolator
                        .with_mut(|object| object.keyframe_interpolator_transform(factor))
                        .flatten()
                })
                .unwrap_or(factor)
        };
        if self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
            let factor = transform_factor(
                self.animation_data_a.elapsed_seconds,
                self.interpolation(),
                self.interpolator(),
            );
            self.animation_data_b.from = self.animation_data_a.interpolate(factor);
            if factor == 1.0 {
                self.animation_data_a = self.animation_data_b;
                self.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
            } else {
                self.animation_data_a.elapsed_seconds += elapsed;
            }
        }
        let data = *self.current_animation_data();
        if data.elapsed_seconds >= time {
            if self.layout.width() != data.to.width() || self.layout.height() != data.to.height() {
                CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
            }
            self.layout = data.to;
            if self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
                self.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                self.animation_data_a = self.animation_data_b;
                self.animation_data_b.elapsed_seconds = 0.0;
            }
            self.animation_data_a.elapsed_seconds = 0.0;
            self.propagate_size();
            CoreCapabilities::world_transform_mark_dirty(self);
            return false;
        }
        let factor = transform_factor(
            data.elapsed_seconds,
            self.interpolation(),
            self.interpolator(),
        );
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
        if force {
            self.set_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds, true);
        }
        self.layout_data.dirty = true;
        if let (Some(artboard), Some(this)) = (
            self.base.base.base.base.base.artboard_handle(),
            self.base.base.base.base.base.handle(),
        ) {
            Artboard::mark_layout_dirty_occurrence(&artboard, this, None);
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
        let (interpolation, interpolator, time, direction) = (
            self.interpolation(),
            self.interpolator(),
            self.interpolation_time(),
            self.actual_direction(),
        );
        for (_, provider) in Self::layout_providers_children(self.base.children(), false) {
            if provider
                .is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY)
            {
                Self::cascade_layout_style_occurrence(
                    &provider,
                    interpolation,
                    interpolator.clone(),
                    time,
                    direction,
                );
            } else if let Some((_, roots)) = Self::hosted_layout_roots(&provider) {
                for (_, root) in roots {
                    Self::cascade_layout_style_occurrence(
                        &root,
                        interpolation,
                        interpolator.clone(),
                        time,
                        direction,
                    );
                }
            } else {
                provider.with_mut(|object| {
                    object.layout_provider_cascade_style(
                        interpolation,
                        interpolator.clone(),
                        time,
                        direction,
                    )
                });
            }
        }
        updated
    }
    pub fn sync_child_provider_styles(&mut self) {
        for (_, provider) in Self::layout_providers_children(self.base.children(), false) {
            provider.with_mut(|provider| {
                provider.layout_provider_sync_style_changes();
                provider.layout_provider_mark_node_dirty(false);
            });
        }
    }
    fn with_callback_style<R>(
        owner: &CoreHandle,
        active_style: &mut LayoutComponentStyle,
        f: impl FnOnce(&mut LayoutComponentStyle) -> R,
    ) -> Option<R> {
        let style = owner
            .with(|object| object.as_layout_component().and_then(Self::style_handle))
            .flatten()?;
        if active_style.handle().as_ref() == Some(&style) {
            Some(f(active_style))
        } else {
            style.with_downcast_mut::<LayoutComponentStyle, _>(f)
        }
    }
    pub(crate) fn position_type_changed_from_style(
        owner: &CoreHandle,
        active_style: &mut LayoutComponentStyle,
    ) {
        let changed = Self::with_callback_style(owner, active_style, |style| {
            if style.position_type() == YGPositionType::Absolute {
                let (left_changed, left) = owner
                    .with(|object| {
                        let layout = object.as_layout_component().expect("style layout owner");
                        (
                            layout.has_layout_flag(LayoutComponentFlags::PositionLeftChanged),
                            layout.layout.left(),
                        )
                    })
                    .expect("live style layout owner");
                if !left_changed {
                    style.set_position_left(left);
                }
                let (top_changed, top) = owner
                    .with(|object| {
                        let layout = object.as_layout_component().expect("style layout owner");
                        (
                            layout.has_layout_flag(LayoutComponentFlags::PositionTopChanged),
                            layout.layout.top(),
                        )
                    })
                    .expect("live style layout owner");
                if !top_changed {
                    style.set_position_top(top);
                }
                style.set_position_right(0.0);
                style.set_position_bottom(0.0);
                style.set_position_left_units_value(YGUnit::Point as u8);
                style.set_position_top_units_value(YGUnit::Point as u8);
                style.set_position_right_units_value(YGUnit::Undefined as u8);
                style.set_position_bottom_units_value(YGUnit::Undefined as u8);
            } else {
                style.set_position_left(0.0);
                style.set_position_top(0.0);
                style.set_position_right(0.0);
                style.set_position_bottom(0.0);
                style.set_position_left_units_value(YGUnit::Undefined as u8);
                style.set_position_top_units_value(YGUnit::Undefined as u8);
                style.set_position_right_units_value(YGUnit::Undefined as u8);
                style.set_position_bottom_units_value(YGUnit::Undefined as u8);
            }
        });
        if changed.is_some() {
            Self::mark_layout_node_dirty_occurrence(owner, false);
        }
    }
    pub(crate) fn scale_type_changed_from_style(
        owner: &CoreHandle,
        active_style: &mut LayoutComponentStyle,
    ) {
        let changed = Self::with_callback_style(owner, active_style, |style| {
            style.set_intrinsically_sized_value(
                style.width_scale_type() == LayoutScaleType::Hug
                    || style.height_scale_type() == LayoutScaleType::Hug,
            );
        });
        if changed.is_some() {
            Self::mark_layout_node_dirty_occurrence(owner, false);
        }
    }
    pub(crate) fn display_changed_from_style(
        owner: &CoreHandle,
        active_style: &mut LayoutComponentStyle,
    ) {
        if let Some(display_hidden) = Self::with_callback_style(owner, active_style, |style| {
            style.display() == YGDisplay::None
        }) {
            let collapsed = owner
                .with(|object| {
                    object
                        .as_component()
                        .expect("layout component")
                        .is_collapsed()
                        || display_hidden
                })
                .expect("live layout owner");
            Self::propagate_resolved_collapse_occurrence(
                owner,
                collapsed,
                collapsed,
                Some(active_style),
            );
            Self::mark_layout_node_dirty_occurrence(owner, false);
        }
    }
    pub(crate) fn flow_style_changed_from_style(
        owner: &CoreHandle,
        active_style: &mut LayoutComponentStyle,
    ) {
        Self::mark_layout_node_dirty_occurrence(owner, false);
        let snapshot = Self::with_callback_style(owner, active_style, |style| {
            let inherited_direction = owner
                .with(|object| {
                    object
                        .as_layout_component()
                        .expect("layout style owner")
                        .inherited_direction
                })
                .expect("live layout owner");
            crate::mechanical_port::source::layout::layout_style_applier::LayoutParentStyleSnapshot {
                owner: owner.clone(),
                is_grid: style.is_grid(),
                is_stack: style.is_stack(),
                justify_items: u32::from(style.base.justify_items_value()),
                is_row: matches!(style.flex_direction(), YGFlexDirection::Row | YGFlexDirection::RowReverse),
                is_ltr: match style.direction() { YGDirection::Ltr => true, YGDirection::Rtl => false, _ => inherited_direction != LayoutDirection::Rtl },
            }
        });
        Self::sync_child_provider_styles_with_parent_style_occurrence(owner, snapshot.as_ref());
    }
    pub(crate) fn direction_changed_occurrence(owner: &CoreHandle) {
        Self::mark_layout_style_dirty_occurrence(owner);
        Self::mark_layout_node_dirty_occurrence(owner, true);
    }
    pub fn clip_changed(&mut self) {
        self.mark_layout_node_dirty(false);
        CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
    }
    /// Set a live occurrence override; None restores the imported two-axis
    /// clipping flag. Requires Renderer::clip_axis_transformed support when drawn.
    /// Keep a proxy after clearing so later toggles also work on plain boxes.
    pub fn set_css_overflow_axis(&mut self, axis: Option<CssOverflowAxis>) {
        if self.css_overflow_axis != axis {
            self.css_overflow_axis = axis;
            self.mark_clip_may_be_dynamic();
            self.clip_changed();
        }
    }
    /// Install only for computed two-axis `clip`. Hidden and single-axis
    /// overflow ignore CSS clip margins in the pinned Chromium profile.
    /// Clearing restores the imported background-shaped clipping path.
    pub fn set_css_overflow_clip_margin(&mut self, margin: Option<CssOverflowClipMargin>) {
        if self.css_overflow_clip_margin != margin {
            self.css_overflow_clip_margin = margin;
            self.clip_changed();
        }
    }
    pub fn css_overflow_clip_margin(&self) -> Option<CssOverflowClipMargin> {
        self.css_overflow_clip_margin
    }

    pub fn css_overflow_axis(&self) -> Option<CssOverflowAxis> {
        self.css_overflow_axis
    }

    pub fn set_clip(&mut self, value: bool) {
        if self.base.set_clip_value(value) {
            self.clip_changed();
            LayoutComponentBaseCallbacks::notify_property_changed(
                self,
                LayoutComponentBase::CLIP_PROPERTY_KEY,
            );
        }
    }
    pub fn set_width(&mut self, value: f32) {
        if self.base.set_width_value(value) {
            self.width_changed();
            LayoutComponentBaseCallbacks::notify_property_changed(
                self,
                LayoutComponentBase::WIDTH_PROPERTY_KEY,
            );
        }
    }
    pub fn set_height(&mut self, value: f32) {
        if self.base.set_height_value(value) {
            self.height_changed();
            LayoutComponentBaseCallbacks::notify_property_changed(
                self,
                LayoutComponentBase::HEIGHT_PROPERTY_KEY,
            );
        }
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
    pub fn world_path(&mut self) -> Option<&mut ShapePaintPath> {
        self.render_paths.as_mut().map(|paths| &mut paths.world)
    }
    pub fn local_path(&mut self) -> Option<&mut ShapePaintPath> {
        self.render_paths.as_mut().map(|paths| &mut paths.local)
    }
    pub fn local_clockwise_path(&mut self) -> Option<&mut ShapePaintPath> {
        self.local_path()
    }
    pub fn path_builder(&mut self) -> &mut Component {
        self
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
                style,
                self,
                u32::from(justify),
            );
        }
    }
    pub fn apply_base_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        style.taffy.box_sizing = if self.css_content_box {
            taffy::style::BoxSizing::ContentBox
        } else {
            taffy::style::BoxSizing::BorderBox
        };
        style.taffy.aspect_ratio_pair = self.css_ratio_pair;
        style.taffy.css_percentage_spacing = self.css_percentage_spacing;
        style.taffy.css_relative_position = self.css_relative_position;
        style.taffy.item_is_replaced = self.css_ratio_pair.is_some()
            && self.with_style(|style| style.intrinsically_sized()).unwrap_or(false);
        style.taffy.aspect_ratio_size_limit = self.css_ratio_size_limit.then_some(33_554_432.0);
        style.taffy.aspect_ratio_box_sizing = self.css_ratio_content_box
            .then_some(taffy::style::BoxSizing::ContentBox);
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
            flex_basis,
            flex_basis_units,
        )) = component_style.with_downcast::<LayoutComponentStyle, _>(|component_style| {
            (
                component_style.position_type() == YGPositionType::Absolute,
                component_style.width_scale_type() == LayoutScaleType::Fixed
                    && component_style.height_scale_type() == LayoutScaleType::Fixed
                    && component_style.intrinsically_sized()
                    && self.is_leaf(),
                component_style.width_scale_type(),
                component_style.height_scale_type(),
                component_style.width_units(),
                component_style.height_units(),
                component_style.base.flex_basis(),
                component_style.flex_basis_units(),
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
                width_units = YGUnit::from(self.width_unit_value_override as u32);
                width_scale = if width_units == YGUnit::Auto {
                    if self.has_layout_flag(LayoutComponentFlags::WidthIntrinsicallySizeOverride) {
                        LayoutScaleType::Hug
                    } else {
                        LayoutScaleType::Fill
                    }
                } else {
                    LayoutScaleType::Fixed
                };
            }
            if self.height_unit_value_override != -1 {
                height_units = YGUnit::from(self.height_unit_value_override as u32);
                height_scale = if height_units == YGUnit::Auto {
                    if self.has_layout_flag(LayoutComponentFlags::HeightIntrinsicallySizeOverride) {
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
                    style.set_flex_basis(YGValue::new(flex_basis, flex_basis_units));
                }
                _ => {
                    style.set_flex_grow(YGFloatOptional::new(0.0));
                    style.set_flex_shrink(YGFloatOptional::new(0.0));
                    style.set_flex_basis(YGValue::new(flex_basis, YGUnit::Auto));
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
        if let Some(factors) = self.css_flex_factors.filter(|_| !context.parent_is_grid) {
            style.set_flex_grow(YGFloatOptional::new(factors.grow));
            style.set_flex_shrink(YGFloatOptional::new(factors.shrink));
            // CSS basis is independent of the Rive fixed/fill sizing choice.
            style.set_flex_basis(YGValue::new(flex_basis, flex_basis_units));
        }
        if let Some(alignment) = self.css_align_self.filter(|value| *value != CssAlignSelf::Auto) {
            // The compiler represents parent align-items:stretch through the
            // child's cross-axis Fill sizing. Preserve that resolved default
            // for CSS auto; YGAlign::Auto alone would lose the stretch.
            style.set_align_self(alignment.runtime_alignment());
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
    fn apply_placement_style(&self, style: &mut YGStyle, _context: &LayoutSyncContext) {
        if let Some(alignment) = self.css_justify_content {
            style.set_justify_content(alignment.runtime_alignment());
        }
        // Container style translation couples Rive line and item alignment.
        // The final sweep applies the explicit CSS line policy independently.
        if let Some(alignment) = self.css_align_content {
            style.set_align_content(alignment.runtime_alignment());
        }
    }

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

#[cfg(test)]
mod packed_layout_tests {
    use super::*;

    #[test]
    fn css_border_geometry_is_opt_in_and_survives_clone() {
        let mut layout = LayoutComponent::default();
        layout.layout = Layout::new(0., 0., 100., 80.);
        layout.layout_padding = LayoutPadding::new(12., 5., 18., 10.);
        layout.layout_border = LayoutPadding::new(8., 9., 10., 11.);
        assert_eq!((layout.inner_width(), layout.inner_height()), (70., 65.));
        assert_eq!(layout.css_axis_clip_interval(true), (0., 100.));
        layout.set_css_border_geometry(true);
        assert_eq!((layout.inner_width(), layout.inner_height()), (52., 45.));
        assert_eq!(layout.css_axis_clip_interval(true), (8., 90.));
        assert_eq!(layout.css_axis_clip_interval(false), (9., 69.));
        let mut clone = layout.clone_core();
        assert!(clone.css_border_geometry);
        clone.set_css_border_geometry(false);
        assert!(layout.css_border_geometry);
        layout.layout = Layout::new(0., 0., 10., 10.);
        assert_eq!((layout.inner_width(), layout.inner_height()), (0., 0.));
        assert_eq!(layout.css_axis_clip_interval(true), (8., 8.));
        layout.set_css_border_geometry(false);
        assert_eq!(layout.css_axis_clip_interval(true), (0., 10.));
    }

    #[test]
    fn css_content_clip_snaps_final_fractional_insets() {
        let mut layout = LayoutComponent::default();
        layout.layout = Layout::new(0.,0.,200.,140.);
        layout.layout_border = LayoutPadding::new(4.,4.,4.,4.);
        layout.layout_padding = LayoutPadding::new(12.5,8.25,12.5,8.25);
        let margin = CssOverflowClipMargin::new(CssOverflowClipBox::Content,8.).unwrap();
        let (bounds,_,world) = layout.paint_geometry();
        assert_eq!(layout.css_clip_paint_outsets(margin,bounds,world),[-8.5,-4.25,-8.5,-4.25]);
        layout.set_css_pixel_bounds(true);
        let (bounds,_,world) = layout.paint_geometry();
        // Chrome control: x=[9,192], y=[4,136] in the local 200x140 box.
        assert_eq!(layout.css_clip_paint_outsets(margin,bounds,world),[-9.,-4.,-8.,-4.]);
        let translated = Mat2D::new(1.,0.,0.,1.,20.625,20.25);
        let snapped_outer = Aabb::new(0.375,-0.25,200.375,139.75);
        assert_eq!(layout.css_clip_paint_outsets(margin,snapped_outer,translated),[-8.,-5.,-9.,-4.]);
        let scaled = Mat2D::new(2.,0.,0.,2.,0.,0.);
        assert_eq!(layout.css_clip_paint_outsets(margin,bounds,scaled),[-8.5,-4.25,-8.5,-4.25]);
    }

    #[test]
    fn css_clip_origins_distinguish_live_border_padding_and_content_edges() {
        let mut layout = LayoutComponent::default();
        layout.layout_padding = LayoutPadding::new(12., 5., 18., 10.);
        for (width, border) in [(240., 8.), (768., 24.), (390., 1.), (240., 8.)] {
            layout.layout = Layout::new(0., 0., width, 100.);
            layout.layout_border = LayoutPadding::new(border, border + 1., border + 2., border + 3.);
            for margin in [-8., 0., 24.] {
                let bounds = |origin| CssOverflowClipMargin::new(origin, margin).unwrap().bounds(&layout).unwrap();
                assert_eq!(bounds(CssOverflowClipBox::Border),
                    Aabb::new(-margin, -margin, width + margin, 100. + margin));
                assert_eq!(bounds(CssOverflowClipBox::Padding),
                    Aabb::new(border - margin, border + 1. - margin,
                        width - border - 2. + margin, 97. - border + margin));
                assert_eq!(bounds(CssOverflowClipBox::Content),
                    Aabb::new(border + 12. - margin, border + 6. - margin,
                        width - border - 20. + margin, 87. - border + margin));
            }
        }
    }

    #[test]
    fn css_clip_margin_bounds_follow_live_size_and_asymmetric_padding() {
        let content = CssOverflowClipMargin::new(CssOverflowClipBox::Content, 8.).unwrap();
        let padding = CssOverflowClipMargin::new(CssOverflowClipBox::Padding, 8.).unwrap();
        let border = CssOverflowClipMargin::new(CssOverflowClipBox::Border, 8.).unwrap();
        let mut layout = LayoutComponent::default();
        for (width, height, inset) in [(240., 100., 12.), (390., 140., 20.),
            (768., 80., 30.), (240., 100., 12.)] {
            layout.layout = Layout::new(70., 90., width, height);
            layout.layout_padding = LayoutPadding::new(inset, 5., 18., 10.);
            assert_eq!(content.bounds(&layout),
                Some(Aabb::new(inset - 8., -3., width - 10., height - 2.)));
            assert_eq!(padding.bounds(&layout),
                Some(Aabb::new(-8., -8., width + 8., height + 8.)));
            assert_eq!(border.bounds(&layout), padding.bounds(&layout));
            assert_eq!(layout.layout_width(), width);
            assert_eq!(layout.padding_left(), inset);
        }
    }

    #[test]
    fn css_clip_margin_rejects_nonfinite_offsets_and_resolved_edges() {
        for pixels in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(CssOverflowClipMargin::new(CssOverflowClipBox::Padding, pixels).is_none());
        }
        let margin = CssOverflowClipMargin::new(CssOverflowClipBox::Padding, f32::MAX).unwrap();
        let mut layout = LayoutComponent::default();
        layout.layout = Layout::new(0., 0., f32::MAX, 10.);
        assert!(margin.bounds(&layout).is_none());
        assert!(CssOverflowClipMargin::new(CssOverflowClipBox::Content, 0.).is_some());
        assert!(CssOverflowClipMargin::new(CssOverflowClipBox::Content, -8.).is_some());
    }

    #[test]
    fn css_pixel_bounds_preserve_thin_boxes_without_inflating_zero() {
        for (size, origin, start, end) in [
            (0.0, 0.0, 0.0, 0.0),
            (0.0625, 0.0, 0.0, 0.0),
            (0.078125, 0.0, 0.0, 1.0),
            (0.25, 0.0, 0.0, 1.0),
            (0.25, 0.5, 1.0, 2.0),
            (0.25, 0.75, 1.0, 2.0),
            (0.75, 0.0, 0.0, 1.0),
            (0.75, 0.5, 1.0, 2.0),
        ] {
            let mut layout = LayoutComponent::default();
            layout.set_clip(true);
            layout.layout = Layout::new(0.0, 0.0, size, size);
            layout.base.base.base.base.set_world_transform(
                Mat2D::new(1.0, 0.0, 0.0, 1.0, origin, origin),
            );
            layout.set_css_pixel_bounds(true);
            layout.update_render_path();
            assert_eq!(layout.world_path().unwrap().raw_path().bounds(),
                Aabb::new(start, start, end, end), "size={size}, origin={origin}");
            assert_eq!(layout.layout.width(), size);
            assert_eq!(layout.layout.height(), size);
        }
    }

    #[test]
    fn css_pixel_bounds_survive_clone_and_can_be_disabled() {
        fn prepare(layout: &mut LayoutComponent) {
            layout.set_clip(true);
            layout.layout = Layout::new(0.0, 0.0, 80.25, 40.5);
            layout.base.base.base.base.set_world_transform(
                Mat2D::new(1.0, 0.0, 0.0, 1.0, 3.75, 0.5),
            );
            layout.update_render_path();
        }
        let mut source = LayoutComponent::default();
        prepare(&mut source);
        let raw_bounds = source.world_path().unwrap().raw_path().bounds();
        assert_eq!(raw_bounds, Aabb::new(3.75, 0.5, 84.0, 41.0));
        source.set_css_pixel_bounds(true);
        let css_bounds = source.world_path().unwrap().raw_path().bounds();
        assert_eq!(css_bounds, Aabb::new(4.0, 1.0, 84.0, 41.0));

        let mut clone = source.clone_core();
        prepare(&mut clone);
        assert_eq!(clone.world_path().unwrap().raw_path().bounds(), css_bounds);
        assert_eq!(clone.layout, source.layout);
        clone.set_css_pixel_bounds(false);
        assert_eq!(clone.world_path().unwrap().raw_path().bounds(), raw_bounds);
        assert_eq!(source.world_path().unwrap().raw_path().bounds(), css_bounds);
        assert_eq!(clone.layout, source.layout);
    }

    #[test]
    fn css_corner_occurrence_resolves_after_resize_and_clone_and_clears() {
        use super::super::css_corner_radii::{CssCornerRadii, CssRadiusValue::Percent};
        let mut source = LayoutComponent::default();
        source.set_clip(true);
        source.layout = Layout::new(0.,0.,200.,80.);
        source.set_experimental_css_corner_radii(Some(CssCornerRadii::new([[Percent(25.),Percent(25.)];4]).unwrap()));
        assert_eq!(source.paint_geometry().1, [[50.,20.];4]);
        assert_eq!(source.world_path().unwrap().raw_path().points()[0], Vec2D::new(0.,20.));
        let mut clone = source.clone_core();
        clone.layout = Layout::new(0.,0.,100.,160.);
        clone.update_render_path();
        assert_eq!(clone.paint_geometry().1, [[25.,40.];4]);
        assert_eq!(clone.world_path().unwrap().raw_path().points()[0], Vec2D::new(0.,40.));
        clone.set_experimental_css_corner_radii(None);
        assert_eq!(clone.paint_geometry().1, [[0.;2];4]);
        assert_eq!(clone.world_path().unwrap().raw_path().points()[0], Vec2D::new(0.,0.));
        assert_eq!(source.paint_geometry().1, [[50.,20.];4]);
    }

    #[test]
    fn css_corner_overflow_resolves_to_live_finite_paths() {
        use super::super::css_corner_radii::{CssCornerRadii, CssRadiusValue::Percent};
        let mut source = LayoutComponent::default();
        source.set_clip(true);
        source.layout = Layout::new(0.,0.,200.,80.);
        source.set_experimental_css_corner_radii(Some(CssCornerRadii::new([[Percent(f32::MAX);2];4]).unwrap()));
        assert_eq!(source.world_path().unwrap().raw_path().points()[0], Vec2D::new(0.,40.));
        let mut clone = source.clone_core();
        clone.layout = Layout::new(0.,0.,100.,160.);
        clone.update_render_path();
        assert_eq!(clone.world_path().unwrap().raw_path().points()[0], Vec2D::new(0.,80.));
        assert_eq!(source.world_path().unwrap().raw_path().points()[0], Vec2D::new(0.,40.));
    }

    #[test]
    fn plain_container_paths_and_proxy_are_lazy() {
        let mut layout = LayoutComponent::default();
        assert!(layout.render_paths.is_none());
        assert!(layout.proxy.is_none());
        assert!(layout.world_path().is_none());
        assert!(layout.local_path().is_none());
        assert!(layout.local_clockwise_path().is_none());
        assert!(!layout.needs_drawable_proxy());
        assert_eq!(
            layout.layout_flags,
            (1 << 0) | (1 << 5) | (1 << 6) | (1 << 9)
        );
        layout.mutable_render_paths();
        assert!(layout.world_path().is_some());
        assert!(layout.local_path().is_some());
        assert!(layout.proxy.is_none());
    }

    #[test]
    fn source_proxy_marks_survive_clone_without_render_allocations() {
        for mark in [
            LayoutComponent::mark_clip_may_be_dynamic,
            LayoutComponent::mark_interaction_target,
            LayoutComponent::mark_listener_target,
        ] {
            let mut source = LayoutComponent::default();
            mark(&mut source);
            source.set_layout_flag(LayoutComponentFlags::ComposeTransform, false);
            let clone = source.clone_core();
            assert!(clone.needs_drawable_proxy());
            assert!(!clone.has_layout_flag(LayoutComponentFlags::ComposeTransform));
            assert!(clone.render_paths.is_none());
            assert!(clone.proxy.is_none());
        }
    }
}
