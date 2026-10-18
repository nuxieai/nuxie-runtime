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
            LayoutStyleApplier, LayoutSyncContext, YGAlign, YGDirection, YGDisplay,
            YGFlexDirection, YGPositionType, YGUnit, rive_align_self, rive_dimension,
            rive_hug_dimension,
        },
        layout_tree::{LayoutNodeKind, LayoutTree},
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
use taffy::prelude::{Dimension, Style};

/// Only plain groups and Solos are transparent to layout collection.
pub fn is_transparent_layout_container(component: &CoreHandle) -> bool {
    component.core_type()
        == Some(crate::mechanical_port::source::generated::node_base::NodeBase::TYPE_KEY)
        || component
            .is_type_of(crate::mechanical_port::source::generated::solo_base::SoloBase::TYPE_KEY)
}

/// Lists reached through containers join only when explicitly flagged.
pub fn joins_layout_through_container(
    component: &dyn crate::mechanical_port::source::core::CoreObject,
) -> bool {
    !crate::mechanical_port::source::core::CoreObject::is_type_of(
        component,
        crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY,
    ) || component.as_drawable().expect("component list Drawable").base.drawable_flags()
        & crate::mechanical_port::source::drawable_flag::DrawableFlag::PARTICIPATES_IN_LAYOUT.0 != 0
}

/// Content-sizing reach mirrors downward size propagation: groups, providers,
/// and non-propagating sizeable components stop it. A layout is returned first.
pub fn content_sizing_layout(mut parent: Option<CoreHandle>) -> Option<CoreHandle> {
    while let Some(component) = parent {
        if component.is_type_of(LayoutComponentBase::TYPE_KEY) {
            return Some(component);
        }
        if component.with(LayoutComponent::stops_content_sizing)?
            || crate::mechanical_port::source::layout::layout_node_provider::from_component(
                &component,
            )
            .is_some()
        {
            return None;
        }
        if component.with_mut(|component| {
            component
                .as_intrinsically_sizeable_mut()
                .is_some_and(|sizeable| !sizeable.should_propagate_size_to_children())
        })? {
            return None;
        }
        parent = component.with(|component| component.component_parent_handle())?;
    }
    None
}

/// Provider ownership crosses containers, but follows only component parents:
/// the artboard is a layout and neither walk escapes into its hosting artboard.
pub fn owning_layout(mut component: Option<CoreHandle>) -> Option<CoreHandle> {
    while let Some(owner) = component {
        if owner.is_type_of(LayoutComponentBase::TYPE_KEY) {
            return Some(owner);
        }
        component = owner.with(|owner| owner.component_parent_handle())?;
    }
    None
}

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

/// Allocated on the first retarget or non-default inherited style and retained
/// until destruction. Presence does not mean an animation is in flight.
struct LayoutAnimation {
    a: LayoutAnimationData,
    b: LayoutAnimationData,
    inherited_interpolator: Option<CoreHandle>,
    inherited_interpolation: LayoutStyleInterpolation,
    inherited_interpolation_time: f32,
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
    StyleDisplayHidden = 1 << 14,
}

/// A LayoutComponent's flags, shared with the component.
pub(crate) struct LayoutDrawFlags(Rc<Cell<u16>>);

impl LayoutDrawFlags {
    /// isCollapsed's style half: the style's display is none.
    #[inline]
    pub(crate) fn style_display_hidden(&self) -> bool {
        self.0.get() & LayoutComponentFlags::StyleDisplayHidden as u16 != 0
    }

    /// LayoutComponent::draw: restore the clip drawProxy saved.
    #[inline]
    pub(crate) fn draw(&self, renderer: &mut Renderer) {
        let flags = self.0.get();
        if flags & LayoutComponentFlags::ClipSaved as u16 != 0 {
            self.0
                .set(flags & !(LayoutComponentFlags::ClipSaved as u16));
            renderer.restore();
        }
    }
}

#[derive(Default)]
pub(crate) struct LayoutRenderPaths {
    pub background: RawPath,
    pub local: ShapePaintPath,
    pub world: ShapePaintPath,
    /// CSS mode: the clip `overflow` implies, in world space, which is not
    /// the background (docs/CSS_LAYOUT.md, 4.2). Unused in translated mode.
    pub clip: ShapePaintPath,
    /// CSS mode: the paints drawn with their own geometry (shadows with a
    /// BoxShadow, borders with a BoxBorder), in the box's space.
    pub css_paints: Vec<CssPaintPaths>,
}

/// One special paint of a CSS box and the paths it draws with.
pub(crate) struct CssPaintPaths {
    pub paint: CoreHandle,
    pub path: ShapePaintPath,
    pub clip: Option<ShapePaintPath>,
    /// A BoxBorder's Stroke fills its ring instead of stroking.
    pub fills_stroke: bool,
    /// A shadow's feather offset, and whether it is in world space.
    pub offset: Option<(Vec2D, bool)>,
}

/// A CSS box's geometry in its own space (docs/CSS_LAYOUT.md, 4).
#[derive(Clone, Copy)]
pub(crate) struct CssBoxGeometry {
    pub bounds: Aabb,
    pub radii: crate::css_layout::paint::Radii,
    pub padding: Aabb,
    pub inner: crate::css_layout::paint::Radii,
    pub overflow: taffy::geometry::Point<taffy::style::Overflow>,
}

/// What makes a paint special in CSS mode, read from its children.
enum CssPaintKind {
    Border {
        sides: u8,
    },
    Shadow {
        spread: f32,
        inset: bool,
        reach: f32,
        offset: (Vec2D, bool),
    },
}

fn css_paint_kind(paint: &CoreHandle) -> Option<CssPaintKind> {
    use crate::css_layout::keys;
    use crate::mechanical_port::source::shapes::paint::feather::Feather;
    let children = paint
        .with(|object| {
            object
                .as_container_component()
                .map(|container| container.children().to_vec())
        })
        .flatten()?;
    for child in &children {
        if child.is_type_of(keys::BOX_BORDER) {
            let sides = child
                .with_downcast::<crate::css_layout::BoxBorder, _>(|record| {
                    record.values().choice(keys::BORDER_SIDES)
                })
                .unwrap_or(15);
            return Some(CssPaintKind::Border { sides });
        }
        let Some((feather, grandchildren)) = child
            .with(|object| {
                let feather = object.as_any().downcast_ref::<Feather>()?;
                Some((
                    (
                        feather.base.strength(),
                        feather.base.inner(),
                        Vec2D::new(feather.base.offset_x(), feather.base.offset_y()),
                        crate::mechanical_port::source::transform_space::TransformSpace::from(
                            feather.base.space_value(),
                        ) == crate::mechanical_port::source::transform_space::TransformSpace::World,
                    ),
                    object.as_container_component()?.children().to_vec(),
                ))
            })
            .flatten()
        else {
            continue;
        };
        let Some(spread) = grandchildren.iter().find_map(|grandchild| {
            grandchild.with_downcast::<crate::css_layout::BoxShadow, _>(|record| {
                record.values().number(keys::SHADOW_SPREAD)
            })
        }) else {
            continue;
        };
        let (strength, inset, offset, world) = feather;
        return Some(CssPaintKind::Shadow {
            spread,
            inset,
            // A feather reaches 1.5 times its strength (three standard
            // deviations, the feather texture's extent).
            reach: 1.5 * strength.abs() + offset.x.abs() + offset.y.abs(),
            offset: (offset, world),
        });
    }
    None
}

struct VirtualGrid {
    column: i32,
    row: i32,
    pinned: bool,
}
impl Default for VirtualGrid {
    fn default() -> Self {
        Self {
            column: -1,
            row: -1,
            pinned: false,
        }
    }
}

pub struct LayoutComponent {
    pub base: LayoutComponentBase,
    paints: ShapePaintContainer,
    provider: LayoutNodeProviderState,
    style: Option<CoreHandle>,
    pub(crate) layout_data: Box<LayoutData>,
    layout_children: Vec<LayoutNodeKey>,
    /// The persistent solve tree, present on a layout root.
    layout_tree: Option<Box<LayoutTree>>,
    /// A solve of this root is running; a nested request is ignored.
    layout_solving: bool,
    /// This node's or a descendant's layout children changed since the last
    /// solve of its root.
    layout_tree_topology_dirty: bool,
    /// This node's own layout children changed.
    layout_children_changed: bool,
    /// The virtual grid contributions changed since the last solve.
    virtual_grid_changed: bool,
    layout: Layout,
    layout_padding: LayoutPadding,
    solved_padding: LayoutPadding,
    animation: Option<Box<LayoutAnimation>>,
    inherited_direction: LayoutDirection,
    // Shared with the draw list, which reads StyleDisplayHidden and runs
    // draw's ClipSaved restore without borrowing this component.
    layout_flags: Rc<Cell<u16>>,
    render_paths: Option<std::rc::Rc<std::cell::RefCell<LayoutRenderPaths>>>,
    proxy: Option<Rc<RefCell<DrawableProxy>>>,
    width_override: f32,
    width_unit_value_override: i8,
    height_override: f32,
    height_unit_value_override: i8,
    forced_width: f32,
    forced_height: f32,
    virtual_grid: Option<Box<VirtualGrid>>,
    /// The natural size this box last measured for each host (UNIV-4844).
    measured_natural: RefCell<Vec<(CoreHandle, Option<Vec2D>)>>,
    /// The solved border widths (top, right, bottom, left) and Taffy's
    /// scroll range (scroll width and height), which CSS mode paints and
    /// scrolls with (docs/CSS_LAYOUT.md, sections 3 and 4).
    solved_border: [f32; 4],
    /// CSS mode: the content box size last given to the content layout
    /// resizes, so a padding or border change alone resizes it again.
    propagated_content: Option<Vec2D>,
    scroll_range: Vec2D,
    grid_column_lines: Vec<f32>,
    grid_row_lines: Vec<f32>,
    grid_column_gap: f32,
    grid_row_gap: f32,
    grid_virtual_rows: Vec<f32>,
    grid_virtual_columns: Vec<f32>,
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
            layout_tree: None,
            layout_solving: false,
            layout_tree_topology_dirty: true,
            layout_children_changed: true,
            virtual_grid_changed: false,
            layout: Layout::default(),
            layout_padding: LayoutPadding::default(),
            solved_padding: LayoutPadding::default(),
            animation: None,
            inherited_direction: LayoutDirection::Inherit,
            layout_flags: Rc::new(Cell::new(
                LayoutComponentFlags::ParentIsRow as u16
                    | LayoutComponentFlags::PositionLeftChanged as u16
                    | LayoutComponentFlags::PositionTopChanged as u16
                    | LayoutComponentFlags::ComposeTransform as u16,
            )),
            render_paths: None,
            proxy: None,
            width_override: f32::NAN,
            width_unit_value_override: -1,
            height_override: f32::NAN,
            height_unit_value_override: -1,
            forced_width: f32::NAN,
            forced_height: f32::NAN,
            virtual_grid: None,
            measured_natural: RefCell::new(Vec::new()),
            solved_border: [0.0; 4],
            propagated_content: None,
            scroll_range: Vec2D::default(),
            grid_column_lines: Vec::new(),
            grid_row_lines: Vec::new(),
            grid_column_gap: 0.0,
            grid_row_gap: 0.0,
            grid_virtual_rows: Vec::new(),
            grid_virtual_columns: Vec::new(),
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

crate::mechanical_port::source::transform_component::impl_transform_update!(LayoutComponent,
    |owner, dirt| { crate::mechanical_port::source::layout_component::LayoutComponent::update_occurrence::<Self>(owner, dirt, |object| object.update_render_path()); },
    |object: &mut Self| object.update_transform(),
    crate::mechanical_port::source::node::Node::update_world_transform_occurrence::<Self>,
    |object: &mut Self| object.compose_world_transform(),
    crate::mechanical_port::source::layout_component::LayoutComponent::update_constraints_occurrence::<Self>);

impl LayoutComponent {
    pub(crate) fn update_constraints_occurrence<
        T: crate::mechanical_port::source::transform_component::TransformUpdate,
    >(
        owner: &CoreHandle,
    ) {
        let Some((layout, transforms)) = owner.with_downcast::<T, _>(|object| {
            (
                object
                    .as_layout_component()
                    .expect("LayoutComponent receiver")
                    .layout_constraint_handles(),
                object
                    .as_transform_component()
                    .expect("transform receiver")
                    .constraints()
                    .to_vec(),
            )
        }) else {
            return;
        };
        crate::mechanical_port::source::transform_component::apply_constraint_lists(
            owner,
            layout,
            Vec::new(),
            transforms,
            false,
        );
    }
    pub(crate) fn update_occurrence<
        T: crate::mechanical_port::source::transform_component::TransformUpdate,
    >(
        owner: &CoreHandle,
        dirt: ComponentDirt,
        render_path: fn(&mut T),
    ) {
        if dirt == ComponentDirt::FILTHY {
            Self::interrupt_animation_occurrence(owner);
        }
        let super_dirt = if dirt.contains(ComponentDirt::WORLD_TRANSFORM)
            && owner
                .with_downcast::<T, _>(|object| object.component_parent_handle().is_some())
                .unwrap_or(false)
        {
            dirt | ComponentDirt::TRANSFORM
        } else {
            dirt
        };
        crate::mechanical_port::source::transform_component::update_transform_super::<T>(
            owner, super_dirt,
        );
        owner.with_downcast_mut::<T, _>(|object| {
            let opacity = object
                .world_transform_child_opacity()
                .expect("LayoutComponent opacity");
            object
                .as_layout_component_mut()
                .expect("LayoutComponent receiver")
                .update_after_transform_super(dirt, opacity);
        });
        owner.with_downcast_mut::<T, _>(|object| {
            if dirt.intersects(
                ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM | ComponentDirt::LAYOUT_STYLE,
            ) {
                render_path(object);
            }
            object
                .as_layout_component_mut()
                .expect("LayoutComponent receiver")
                .reset_update_flags();
        });
        if dirt.intersects(ComponentDirt::PATH | ComponentDirt::LAYOUT_STYLE) {
            Self::refresh_box_gradients_occurrence(owner);
        }
    }
    /// A CSS box's BoxGradients follow its size: their gradients rebuild
    /// their shaders when the box's paths change (docs/CSS_LAYOUT.md, 4.2).
    fn refresh_box_gradients_occurrence(owner: &CoreHandle) {
        let Some(paints) = owner
            .with(|object| {
                let layout = object.as_layout_component()?;
                layout
                    .is_css_mode()
                    .then(|| layout.paints.shape_paints().to_vec())
            })
            .flatten()
        else {
            return;
        };
        for paint in paints {
            let children = paint
                .with(|object| {
                    object
                        .as_container_component()
                        .map(|container| container.children().to_vec())
                })
                .flatten()
                .unwrap_or_default();
            for gradient in children {
                let placed = gradient
                    .with(|object| {
                        object.as_container_component().is_some_and(|container| {
                            container.children().iter().any(|child| {
                                child.is_type_of(crate::css_layout::keys::BOX_GRADIENT)
                            })
                        })
                    })
                    .unwrap_or(false);
                if placed {
                    gradient
                        .with_mut(|object| object.component_add_dirt(ComponentDirt::PAINT, false));
                }
            }
        }
    }
    pub fn painted_world_bounds(
        &mut self,
        out: &mut crate::mechanical_port::source::math::aabb::Aabb,
    ) -> crate::mechanical_port::source::drawable::BoundsFidelity {
        // A CSS box's special paths (a spread shadow) reach past it; the
        // paints' feathers and offsets add their reach as for any paint.
        let mut local = self.local_bounds();
        if let Some(paths) = self.render_paths.as_ref() {
            for paint in &paths.borrow().css_paints {
                local.expand(paint.path.raw_path().bounds());
            }
        }
        crate::mechanical_port::source::drawable::Drawable::painted_bounds_from_local(
            &local,
            &self.base.world_transform(),
            Some(&self.paints),
            out,
        )
    }
    pub(crate) fn has_layout_flag(&self, flag: LayoutComponentFlags) -> bool {
        self.layout_flags.get() & flag as u16 != 0
    }
    pub(crate) fn set_layout_flag(&mut self, flag: LayoutComponentFlags, on: bool) {
        let flags = self.layout_flags.get();
        self.layout_flags.set(if on {
            flags | flag as u16
        } else {
            flags & !(flag as u16)
        });
    }
    /// This component's flags, for the draw list's range step.
    pub(crate) fn draw_flags(&self) -> LayoutDrawFlags {
        LayoutDrawFlags(Rc::clone(&self.layout_flags))
    }
    pub(crate) fn mutable_render_paths(&mut self) -> std::cell::RefMut<'_, LayoutRenderPaths> {
        self.render_paths
            .get_or_insert_with(Default::default)
            .borrow_mut()
    }
    /// Retain the existing path owner while a renderer callback releases the
    /// component receiver. The lazy allocation and each path identity stay fixed.
    pub(crate) fn render_paths_handle(
        &self,
    ) -> Option<std::rc::Rc<std::cell::RefCell<LayoutRenderPaths>>> {
        self.render_paths.clone()
    }
    /// Whether this box clips its children: Rive's clip at its outer edge in
    /// translated mode, and in CSS mode an `overflow` other than visible.
    pub fn clips(&self) -> bool {
        let (x, y) = self.clipped_axes();
        x || y
    }
    /// A CSS box always keeps its proxy: its overflow is a record value,
    /// which animation, binding and scripts can change.
    pub fn needs_drawable_proxy(&self) -> bool {
        self.clips()
            || self.is_css_mode()
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
                    || child.with(joins_layout_through_container).expect("live component list");
                if joins {
                    result.push((child.clone(), provider));
                }
            } else if is_transparent_layout_container(child) {
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

    /// A child-list change is reconciled by the root's next solve, which
    /// descends only along the marked ancestor chain to the changed node.
    fn mark_layout_tree_topology_dirty_occurrence(owner: &CoreHandle) {
        owner.with_mut(|object| {
            object
                .as_layout_component_mut()
                .expect("Layout owner")
                .layout_children_changed = true;
        });
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
                layout.layout_data.mark_dirty();
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
        owner.notify_property_changed(LayoutComponentBase::CLIP_PROPERTY_KEY);
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
            owner.notify_property_changed(key);
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
        Self::sync_layout_children_with_active_owners(owner, active_participant, None, None);
    }
    pub(crate) fn sync_layout_children_from_solo(
        owner: &CoreHandle,
        solo: &crate::mechanical_port::source::solo::Solo,
    ) {
        Self::sync_layout_children_with_active_owners(owner, None, Some(solo), None);
    }
    pub(crate) fn sync_layout_children_from_nested(
        owner: &CoreHandle,
        nested: &crate::source::nested_artboard::NestedArtboard,
    ) {
        Self::sync_layout_children_with_active_owners(owner, None, None, Some(nested));
    }
    fn sync_layout_children_with_active_owners(
        owner: &CoreHandle,
        active_participant: Option<
            &crate::mechanical_port::source::layout::layout_participant::LayoutParticipant,
        >,
        active_solo: Option<&crate::mechanical_port::source::solo::Solo>,
        active_nested: Option<&crate::source::nested_artboard::NestedArtboard>,
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
        let mut nodes = Vec::new();
        for (_, provider) in Self::layout_providers_nested_with_solo(owner, false, active_solo) {
            let active = active_participant
                .filter(|participant| participant.base.handle().as_ref() == Some(&provider));
            if let Some(participant) = active {
                for index in 0..participant.num_layout_nodes() {
                    if let Some(node) = participant.layout_node_key(index) {
                        nodes.push(node);
                    }
                }
            } else if let Some(nested) = active_nested.filter(|nested| {
                crate::source::core::CoreObject::core(*nested)
                    .handle()
                    .as_ref()
                    == Some(&provider)
            }) {
                // NestedArtboardLayout::layoutNode(0) exposes its mounted
                // artboard's native node. The provider is already borrowed
                // by nest(), so use that same live instance directly.
                if let Some(instance) = nested.artboard_instance_handle(0) {
                    if let Some(node) = instance.with_artboard_mut(|artboard| {
                        artboard.take_layout_data();
                        artboard.layout_node_key(0)
                    }) {
                        nodes.push(node);
                    }
                }
            } else {
                provider
                    .with(|object| {
                        object
                            .as_layout_node_provider()
                            .expect("layout provider")
                            .collect_layout_nodes(&mut nodes)
                    })
                    .expect("live layout provider");
            }
        }
        for node in nodes {
            *node.owner.borrow_mut() = Some(owner.clone());
            owner.with_mut(|object| {
                let layout = object.as_layout_component_mut().expect("Layout owner");
                #[cfg(feature = "tools")]
                layout.layout_data.children.push(node.provider.clone());
                layout.layout_children.push(node);
            });
        }
        Self::mark_layout_tree_topology_dirty_occurrence(owner);
        Self::mark_layout_node_dirty_occurrence(owner, false);
    }

    pub fn propagate_collapse_occurrence(owner: &CoreHandle, value: bool) {
        let own_collapsed =
            owner.with(|object| object.as_layout_component().unwrap().is_collapsed());
        if let Some(own_collapsed) = own_collapsed {
            Self::propagate_resolved_collapse_occurrence(owner, value || own_collapsed, None, None);
        }
    }
    pub(crate) fn propagate_collapse_from_layout_occurrence(
        owner: &CoreHandle,
        value: bool,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        if let Some(own_collapsed) = owner.with(|object| object.component_is_collapsed()) {
            Self::propagate_resolved_collapse_occurrence(
                owner,
                value || own_collapsed,
                None,
                Some((active, active_handle)),
            );
        }
    }
    fn propagate_resolved_collapse_occurrence(
        owner: &CoreHandle,
        collapsed: bool,
        mut active_style: Option<&mut LayoutComponentStyle>,
        mut active_layout: Option<(
            &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
            &CoreHandle,
        )>,
    ) {
        let Some(children) =
            owner.with(|object| object.as_container_component().unwrap().children().to_vec())
        else {
            return;
        };
        for child in children {
            // A CSS record has no collapsed state of its own, and it is the
            // caller when its display changes.
            if child.is_type_of(crate::css_layout::keys::LAYOUT_CSS_STYLE) {
                continue;
            }
            if let Some(style) = active_style
                .as_deref_mut()
                .filter(|style| style.handle().as_ref() == Some(&child))
            {
                // The source calls this same child while its style setter is
                // active. Use that actual owner, not a second arena borrow.
                CoreCapabilities::component_collapse(style, collapsed);
            } else if let Some((active, active_handle)) = active_layout.as_mut() {
                ComponentOccurrenceHandle::Authored(child).collapse_from_layout(
                    collapsed,
                    active,
                    active_handle,
                );
            } else {
                ComponentOccurrenceHandle::Authored(child).collapse(collapsed);
            }
        }
        // propagateCollapse calls updateCollapsables after the children. Both
        // the virtual collapsed state and membership are current at that site.
        if let Some((own_collapsed, collapsables)) = owner.with(|object| {
            (
                object.component_is_collapsed(),
                object.as_component().unwrap().collapsables_snapshot(),
            )
        }) {
            for collapsable in collapsables.iter() {
                crate::source::data_bind::data_bind::DataBind::collapse_handle(
                    collapsable,
                    own_collapsed,
                );
            }
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
                    layout.layout_data.take_style(),
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
        owner.with_mut(|object| {
            object
                .as_layout_component_mut()
                .unwrap()
                .clear_virtual_grid_placement(&mut style)
        });
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
        for applier in &appliers {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_css_style(&mut style, &context);
                }
            });
        }
        owner.with_mut(|object| {
            let layout = object.as_layout_component_mut().unwrap();
            layout.apply_virtual_grid_placement(&mut style);
            layout.layout_data.set_style(style);
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
            for index in roots {
                let Some(root) = Self::hosted_layout_root(provider, index) else {
                    continue;
                };
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
        let transform = self.build_own_transform();
        self.base
            .base
            .base
            .base
            .base
            .base
            .with_transform_mut(|local| *local = transform);
    }
    pub(crate) fn update_transform_for_artboard(&mut self, origin_x: f32, origin_y: f32) {
        let transform = self.build_own_transform_with(false, origin_x, origin_y);
        self.base
            .base
            .base
            .base
            .base
            .base
            .with_transform_mut(|local| *local = transform);
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
                            .map(|parent| parent.world_transform())
                    })
                    .flatten()
            })
            .unwrap_or_else(Mat2D::identity);
        let base = Mat2D::from_translation(self.layout_translation());
        let own = self.base.base.base.base.base.base.transform();
        self.base
            .base
            .base
            .base
            .set_world_transform(parent_world * base * own);
    }
    fn computed_origin_local(&self) -> Vec2D {
        self.layout_translation()
            + self.base.base.base.base.base.base.transform() * self.local_anchor()
    }
    pub fn shape_world_transform(&self) -> Mat2D {
        self.base.base.base.base.world_transform()
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
        (self.base.base.base.base.world_transform() * self.local_anchor()).x
    }
    pub fn computed_world_y(&self) -> f32 {
        (self.base.base.base.base.world_transform() * self.local_anchor()).y
    }
    pub fn computed_root_x(&self) -> f32 {
        let Some(artboard) = self.artboard_handle() else {
            return 0.0;
        };
        let point = self.base.base.base.base.world_transform() * self.local_anchor();
        artboard
            .with_downcast_mut::<Artboard, _>(|artboard| artboard.root_transform(point).x)
            .expect("computedRootX requires a live artboard")
    }
    pub fn computed_root_y(&self) -> f32 {
        let Some(artboard) = self.artboard_handle() else {
            return 0.0;
        };
        let point = self.base.base.base.base.world_transform() * self.local_anchor();
        artboard
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
        self.refresh_style_display_hidden();
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
        self.sync_shape_paint_blend_modes();
    }
    pub fn additive_amount_changed(&mut self) {
        self.sync_shape_paint_blend_modes();
    }
    pub fn sync_shape_paint_blend_modes(&mut self) {
        let blend = self.base.base.blend_mode();
        let amount = self.base.base.additive_amount();
        for paint in self.paints.shape_paints().iter().cloned() {
            paint.with_mut(|paint| {
                if let Some(paint) = paint.as_shape_paint_mut() {
                    paint.blend_mode(blend.into(), amount);
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
        if !(skip_on_unclipped && !self.clips()) {
            let mut local = inverse * *position;
            if let Some(origin) = root_origin {
                local += origin;
            }
            // Through a CSS box's clip (a hit on a descendant), its clip as
            // its children draw: the padding box on the clipped axes, with
            // inner radii when both clip. The box itself is its border box.
            let inside = match self.css_box_geometry() {
                Some(geometry) if skip_on_unclipped && !primary => {
                    crate::css_layout::paint::overflow_clip_contains(
                        local,
                        geometry.padding,
                        geometry.inner,
                        self.clipped_axes(),
                    )
                }
                _ => self.local_bounds().contains(local),
            };
            if !inside {
                return false;
            }
        }
        self.base.base.hit_test_point(position, true, primary)
    }
    /// The axes this box clips its children on: both in translated mode
    /// when it clips, and in CSS mode each axis whose overflow is not
    /// visible.
    pub(crate) fn clipped_axes(&self) -> (bool, bool) {
        match self.with_css(|css| css.overflow()) {
            Some(overflow) => (
                overflow.x != taffy::style::Overflow::Visible,
                overflow.y != taffy::style::Overflow::Visible,
            ),
            None => (self.base.clip(), self.base.clip()),
        }
    }
    /// Execute the local part of `LayoutComponent::update` after the complete
    /// TransformComponent super call. Returning true requests the pinned
    /// Layout-then-Transform constraint pass after this CoreHandle borrow ends.
    pub(crate) fn update_after_transform_super(
        &mut self,
        value: ComponentDirt,
        child_opacity: f32,
    ) {
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            self.paints.propagate_opacity(child_opacity);
        }
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
        crate::mechanical_port::source::component::ActiveLayoutOwner::Layout(self)
            .collapse_after_component(value);
    }
    pub fn add_collapsable(&mut self, collapsable: CoreHandle) {
        CoreCapabilities::component_add_collapsable(self, collapsable);
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
        self.refresh_style_display_hidden();
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
        let save_for_clip = self.clips();
        // Rive clips the box's own paints with its children. CSS's overflow
        // clips only the content: the box paints first (docs/CSS_LAYOUT.md,
        // 4.2).
        let css = self.is_css_mode();
        if save_for_clip && !css {
            self.save_clip(renderer);
        }
        self.draw_own_paints(renderer);
        if save_for_clip && css {
            self.save_clip(renderer);
        }
        self.set_layout_flag(LayoutComponentFlags::ClipSaved, save_for_clip);
    }
    /// Save the renderer and clip to this box's clip path, which `draw`
    /// restores after its children.
    fn save_clip(&mut self, renderer: &mut Renderer) {
        renderer.save();
        let factory = self
            .with_artboard(|artboard| artboard.factory())
            .flatten()
            .expect("a drawable LayoutComponent has its imported factory");
        let css = self.is_css_mode();
        let mut paths = self.mutable_render_paths();
        let clip = if css {
            &mut paths.clip
        } else {
            &mut paths.world
        };
        renderer.clip_path(clip.render_path(&factory));
    }
    fn draw_own_paints(&mut self, renderer: &mut Renderer) {
        let world = self.shape_world_transform();
        // The paint list can't change while this LayoutComponent is borrowed.
        let paints = self.paints.cells().paints();
        let css_paints = self
            .render_paths
            .as_ref()
            .is_some_and(|paths| !paths.borrow().css_paints.is_empty());
        for paint in paints.iter() {
            if css_paints && self.draw_css_paint(renderer, &paint.handle, world) {
                continue;
            }
            // A plain paint whose loan, and its mutator's, would succeed and run
            // nothing draws from its cells, as a Shape's plain paints do.
            if paint.cells.is_plain()
                && paint.handle.is_occupied_settled()
                && let Some(mutator_visible) = paint.cells.settled_mutator_is_visible()
            {
                // ShapePaint::shouldDraw.
                if !(paint.cells.is_visible() && mutator_visible) {
                    continue;
                }
                let fill_rule = paint.cells.fill_rule();
                let path = match paint.cells.path_kind() {
                    ShapePaintPathKind::Local | ShapePaintPathKind::LocalClockwise => {
                        self.local_path()
                    }
                    ShapePaintPathKind::World => self.world_path(),
                };
                let Some(mut path) = path else {
                    continue;
                };
                let handle = &paint.handle;
                paint
                    .cells
                    .draw_plain(renderer, &mut path, world, true, fill_rule, &|| {
                        // The factory the paint's own draw would read.
                        handle
                            .with(|object| {
                                object
                                    .as_component()
                                    .and_then(|component| {
                                        component.with_artboard(|artboard| artboard.factory())
                                    })
                                    .flatten()
                            })
                            .flatten()
                            .expect("a drawing ShapePaint has its imported Artboard factory")
                    });
                continue;
            }
            paint.handle.with_mut(|paint| {
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
                let Some(mut path) = path else {
                    return;
                };
                paint.shape_paint_mut().draw_with_active_container(
                    renderer,
                    &mut path,
                    world,
                    false,
                    None,
                    true,
                    fill_rule,
                    &|| world,
                    None,
                );
            });
        }
    }
    /// Draw `paint` with its CSS geometry (docs/CSS_LAYOUT.md, 4.2), when it
    /// has one. A shadow's feather blurs the path the box gives it, offset
    /// past a clip that stays with the box. A BoxBorder's Stroke fills its
    /// ring. Returns whether the paint was drawn here.
    pub(crate) fn draw_css_paint(
        &mut self,
        renderer: &mut Renderer,
        paint: &CoreHandle,
        world: Mat2D,
    ) -> bool {
        self.draw_css_paint_with(renderer, paint, world, None)
    }
    /// As [`Self::draw_css_paint`], with the renderer factory given, as an
    /// artboard drawing its own paints passes it.
    pub(crate) fn draw_css_paint_with(
        &mut self,
        renderer: &mut Renderer,
        paint: &CoreHandle,
        world: Mat2D,
        factory: Option<crate::RuntimeFactoryHandle>,
    ) -> bool {
        let Some(paths) = self.render_paths.clone() else {
            return false;
        };
        let mut paths = paths.borrow_mut();
        let Some(special) = paths
            .css_paints
            .iter_mut()
            .find(|special| &special.paint == paint)
        else {
            return false;
        };
        let fills_stroke = special.fills_stroke;
        let Some((visible, render_paint)) = paint
            .with_mut(|object| {
                let behavior = object.as_shape_paint_behavior_mut()?;
                // A BoxBorder ignores its Stroke's thickness, which a
                // Stroke's own visibility reads.
                let visible = if fills_stroke {
                    behavior.shape_paint().base.is_visible()
                        && behavior.shape_paint().mutator_is_visible()
                } else {
                    behavior.should_draw()
                };
                Some((visible, behavior.shape_paint_mut().render_paint_handle()))
            })
            .flatten()
        else {
            return true;
        };
        let Some(render_paint) = render_paint.filter(|_| visible) else {
            return true;
        };
        let Some(factory) =
            factory.or_else(|| self.with_artboard(|artboard| artboard.factory()).flatten())
        else {
            return true;
        };
        renderer.save();
        if let Some((offset, true)) = special.offset {
            renderer.translate(offset.x, offset.y);
        }
        renderer.transform(nuxie_render_api::Mat2D(*world.values()));
        if let Some(clip) = special.clip.as_mut() {
            renderer.clip_path(clip.render_path(&factory));
        }
        if let Some((offset, false)) = special.offset {
            renderer.translate(offset.x, offset.y);
        }
        let mut render_paint = render_paint.borrow_mut();
        if special.fills_stroke {
            render_paint.style(nuxie_render_api::RenderPaintStyle::Fill);
        }
        renderer.draw_path(special.path.render_path(&factory), render_paint.as_ref());
        if special.fills_stroke {
            render_paint.style(nuxie_render_api::RenderPaintStyle::Stroke);
        }
        drop(render_paint);
        renderer.restore();
        true
    }
    pub fn draw(&mut self, renderer: &mut Renderer) {
        if self.has_layout_flag(LayoutComponentFlags::ClipSaved) {
            self.set_layout_flag(LayoutComponentFlags::ClipSaved, false);
            renderer.restore();
        }
    }
    pub fn update_render_path(&mut self) {
        {
            if self.is_hidden()
                || (self.paints.shape_paints().is_empty()
                    && !self.clips()
                    && !self.has_layout_flag(LayoutComponentFlags::HasForegroundDrawable))
            {
                return;
            }
            if self.is_css_mode() {
                self.update_css_render_path();
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
            let bounds = Aabb::new(0.0, 0.0, self.layout.width(), self.layout.height());
            let world = self.base.base.base.base.world_transform();
            let mut paths_owner = self.mutable_render_paths();
            let paths = &mut *paths_owner;
            paths.background.rewind();
            Path::add_rounded_rect(&mut paths.background, bounds, radii);
            paths.background.prune_empty_segments();
            if paths.local.raw_path() != &paths.background {
                paths.local.rewind();
                paths.local.add_path(&paths.background, None);
            }
            paths
                .world
                .rewind_as(false, nuxie_render_api::FillRule::Clockwise);
            paths.world.add_path(&paths.background, Some(&world));
            drop(paths_owner);
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
    /// A CSS box's geometry in its own space (docs/CSS_LAYOUT.md, 4): its
    /// border box with CSS radii (points from the style, percentages from
    /// the record, scaled together when they overlap), its padding box with
    /// the inner radii, and its computed overflow. None in translated mode.
    pub(crate) fn css_box_geometry(&self) -> Option<CssBoxGeometry> {
        use crate::css_layout::{keys, paint};
        let points = self
            .with_style(|style| {
                if style.base.link_corner_radius() {
                    [style.base.corner_radius_tl(); 4]
                } else {
                    [
                        style.base.corner_radius_tl(),
                        style.base.corner_radius_tr(),
                        style.base.corner_radius_br(),
                        style.base.corner_radius_bl(),
                    ]
                }
            })
            .unwrap_or([0.0; 4]);
        let (percents, overflow) = self.with_css(|css| {
            (
                std::array::from_fn(|corner| {
                    css.number(keys::RADIUS_TOP_LEFT_PERCENT + corner as u16)
                }),
                css.overflow(),
            )
        })?;
        let (width, height) = (self.layout.width(), self.layout.height());
        let bounds = Aabb::new(0.0, 0.0, width, height);
        let radii = paint::border_radii(width, height, points, percents);
        let border = self.solved_border;
        Some(CssBoxGeometry {
            bounds,
            radii,
            padding: paint::padding_box(bounds, border),
            inner: paint::inner_radii(radii, border),
            overflow,
        })
    }
    /// The paths this box's special paints (BoxBorder and BoxShadow) draw
    /// with, from its CSS geometry (docs/CSS_LAYOUT.md, 4.2), moved by
    /// `translation` when its drawing space is not its layout's.
    pub(crate) fn css_paint_paths(
        &self,
        geometry: &CssBoxGeometry,
        translation: Option<Mat2D>,
    ) -> Vec<CssPaintPaths> {
        use crate::css_layout::paint;
        let CssBoxGeometry {
            bounds,
            radii,
            padding,
            inner,
            ..
        } = *geometry;
        let local_path = |region: &paint::Region| {
            let mut path = ShapePaintPath::new(true);
            path.rewind_as(true, nuxie_render_api::FillRule::NonZero);
            path.add_path(&region.outline, translation.as_ref());
            path.add_path_backwards(&region.holes, translation.as_ref());
            path
        };
        self.paints
            .shape_paints()
            .iter()
            .filter_map(|paint| {
                let (geometry, fills_stroke, offset) = match css_paint_kind(paint)? {
                    CssPaintKind::Border { sides } => (
                        paint::box_border(bounds, radii, padding, inner, sides),
                        true,
                        None,
                    ),
                    CssPaintKind::Shadow {
                        spread,
                        inset,
                        reach,
                        offset,
                    } => (
                        paint::box_shadow(bounds, radii, padding, inner, spread, inset, reach),
                        false,
                        Some(offset),
                    ),
                };
                Some(CssPaintPaths {
                    paint: paint.clone(),
                    path: local_path(&geometry.path),
                    clip: geometry.clip.as_ref().map(local_path),
                    fills_stroke,
                    offset,
                })
            })
            .collect()
    }
    /// CSS mode's box paths (docs/CSS_LAYOUT.md, 4): the border box with CSS
    /// radii (points from the style, percentages from the record, scaled
    /// together when they overlap) and, when `overflow` clips, the padding
    /// box with its inner radii. Radii are physical: no RTL swap.
    fn update_css_render_path(&mut self) {
        use crate::css_layout::paint;
        let Some(CssBoxGeometry {
            bounds,
            radii,
            padding,
            inner,
            overflow,
        }) = self.css_box_geometry()
        else {
            return;
        };
        let world = self.base.base.base.base.world_transform();
        let mut paths_owner = self.mutable_render_paths();
        let paths = &mut *paths_owner;
        paths.background.rewind();
        paint::add_rounded_rect(&mut paths.background, bounds, radii);
        paths.background.prune_empty_segments();
        if paths.local.raw_path() != &paths.background {
            paths.local.rewind();
            paths.local.add_path(&paths.background, None);
        }
        paths
            .world
            .rewind_as(false, nuxie_render_api::FillRule::Clockwise);
        paths.world.add_path(&paths.background, Some(&world));
        let mut clip = RawPath::default();
        let visible = taffy::style::Overflow::Visible;
        paint::add_overflow_clip(
            &mut clip,
            padding,
            inner,
            (overflow.x != visible, overflow.y != visible),
        );
        paths
            .clip
            .rewind_as(false, nuxie_render_api::FillRule::Clockwise);
        paths.clip.add_path(&clip, Some(&world));
        drop(paths_owner);
        let css_paints = self.css_paint_paths(
            &CssBoxGeometry {
                bounds,
                radii,
                padding,
                inner,
                overflow,
            },
            None,
        );
        self.mutable_render_paths().css_paints = css_paints;
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
    pub fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        let (width_mode, height_mode) = self.content_measure_modes(width_mode, height_mode);
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
    // Shared with the handle-based measurement adapter, which releases the
    // layout owner before invoking children that can read their parent.
    fn content_measure_modes(
        &self,
        mut width_mode: LayoutMeasureMode,
        mut height_mode: LayoutMeasureMode,
    ) -> (LayoutMeasureMode, LayoutMeasureMode) {
        use crate::mechanical_port::source::layout::layout_measure_mode::unbound_measure_mode;
        // A CSS box measures with Taffy's modes: the Rive style's Hug and
        // hugUnbounded are not its sizing (docs/CSS_LAYOUT.md, 2.1).
        if self.is_css_mode() {
            return (width_mode, height_mode);
        }
        if self
            .with_style(|style| style.hug_unbounded())
            .unwrap_or(false)
        {
            let mut width_hugs = self.effective_width_scale_type() == LayoutScaleType::Hug;
            let mut height_hugs = self.effective_height_scale_type() == LayoutScaleType::Hug;
            if !width_hugs && !height_hugs {
                let intrinsic = self
                    .with_style(|style| style.intrinsically_sized())
                    .unwrap_or(false);
                width_hugs = intrinsic;
                height_hugs = intrinsic;
            }
            if width_hugs {
                width_mode = unbound_measure_mode(width_mode);
            }
            if height_hugs {
                height_mode = unbound_measure_mode(height_mode);
            }
        }
        (width_mode, height_mode)
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
    pub fn effective_width_scale_type(&self) -> LayoutScaleType {
        if self.can_have_overrides() && self.width_unit_value_override != -1 {
            match YGUnit::from(self.width_unit_value_override as u32) {
                YGUnit::Point | YGUnit::Percent => return LayoutScaleType::Fixed,
                YGUnit::Auto => {
                    return if self
                        .has_layout_flag(LayoutComponentFlags::WidthIntrinsicallySizeOverride)
                    {
                        LayoutScaleType::Hug
                    } else {
                        LayoutScaleType::Fill
                    };
                }
                _ => {}
            }
        }
        self.with_style(LayoutComponentStyle::width_scale_type)
            .unwrap_or(LayoutScaleType::Fixed)
    }
    pub fn effective_height_scale_type(&self) -> LayoutScaleType {
        if self.can_have_overrides() && self.height_unit_value_override != -1 {
            match YGUnit::from(self.height_unit_value_override as u32) {
                YGUnit::Point | YGUnit::Percent => return LayoutScaleType::Fixed,
                YGUnit::Auto => {
                    return if self
                        .has_layout_flag(LayoutComponentFlags::HeightIntrinsicallySizeOverride)
                    {
                        LayoutScaleType::Hug
                    } else {
                        LayoutScaleType::Fill
                    };
                }
                _ => {}
            }
        }
        self.with_style(LayoutComponentStyle::height_scale_type)
            .unwrap_or(LayoutScaleType::Fixed)
    }
    pub fn main_axis_is_row(&self) -> bool {
        if let Some(row) =
            self.with_css(|css| css.choice(crate::css_layout::keys::FLEX_DIRECTION) <= 1)
        {
            return row;
        }
        self.with_style(|style| {
            matches!(
                style.flex_direction(),
                YGFlexDirection::Row | YGFlexDirection::RowReverse
            )
        })
        .unwrap_or(true)
    }
    pub fn virtual_grid_cell(&mut self, column: i32, row: i32) {
        if self.update_virtual_grid_cell(column, row) {
            self.mark_layout_node_dirty(false);
        }
    }
    pub(crate) fn set_virtual_grid_cell_with_host_occurrence(
        owner: &CoreHandle,
        column: i32,
        row: i32,
        host: &mut dyn crate::source::artboard_host::ArtboardHost,
    ) {
        let changed = owner
            .with_mut(|object| {
                object
                    .as_layout_component_mut()
                    .unwrap()
                    .update_virtual_grid_cell(column, row)
            })
            .unwrap_or(false);
        if changed {
            Self::mark_layout_node_dirty_with_host_occurrence(owner, false, Some(host));
        }
    }
    fn update_virtual_grid_cell(&mut self, column: i32, row: i32) -> bool {
        if self.virtual_grid.is_none() && column < 0 {
            return false;
        }
        let grid = self.virtual_grid.get_or_insert_with(Default::default);
        if grid.column == column && grid.row == row {
            return false;
        }
        grid.column = column;
        grid.row = row;
        true
    }
    pub fn virtual_grid_contributions(&mut self, rows: &[f32], columns: &[f32]) {
        let finite = |v: &f32| if v.is_finite() { *v } else { 0.0 };
        if self
            .grid_virtual_rows
            .iter()
            .copied()
            .eq(rows.iter().map(finite))
            && self
                .grid_virtual_columns
                .iter()
                .copied()
                .eq(columns.iter().map(finite))
        {
            return;
        }
        self.grid_virtual_rows = rows.iter().map(finite).collect();
        self.grid_virtual_columns = columns.iter().map(finite).collect();
        // Contributions become virtual grid items, resynced by the next solve.
        self.virtual_grid_changed = true;
        self.mark_layout_node_dirty(false);
    }
    pub fn hugs_lines(&self) -> bool {
        self.with_style(|style| if self.main_axis_is_row() { style.width_scale_type() } else { style.height_scale_type() } == LayoutScaleType::Hug).unwrap_or(false)
    }
    pub fn wraps_lines(&self) -> bool {
        self.with_style(|style| {
            !style.is_grid()
                && style.flex_wrap() == crate::source::layout::layout_style_applier::YGWrap::Wrap
                && matches!(
                    style.flex_direction(),
                    YGFlexDirection::Row | YGFlexDirection::Column
                )
                && !(self.hugs_lines() && style.hug_unbounded())
        })
        .unwrap_or(false)
    }
    pub fn child_alignment(&self) -> crate::source::layout::layout_enums::LayoutContainerAlignment {
        use crate::source::layout::layout_enums::*;
        self.with_style(|style| {
            container_alignment(style.alignment_type(), self.main_axis_is_row())
        })
        .unwrap_or(LayoutContainerAlignment {
            main: LayoutMainDistribute::Start,
            cross: LayoutCrossAlign::Start,
        })
    }
    pub fn is_grid_container(&self) -> bool {
        if let Some(grid) = self.with_css(|css| css.choice(crate::css_layout::keys::DISPLAY) == 2) {
            return grid;
        }
        self.with_style(|style| style.is_grid() && !style.is_stack())
            .unwrap_or(false)
    }
    /// This box's `LayoutCssStyle`: present exactly when the box is in CSS
    /// mode (docs/CSS_LAYOUT.md, 2.1).
    pub fn css_record(&self) -> Option<CoreHandle> {
        self.layout_data
            .appliers
            .as_deref()?
            .iter()
            .find(|applier| applier.is_type_of(crate::css_layout::keys::LAYOUT_CSS_STYLE))
            .cloned()
    }
    pub fn is_css_mode(&self) -> bool {
        self.css_record().is_some()
    }
    pub fn with_css<R>(&self, f: impl FnOnce(crate::css_layout::CssValues<'_>) -> R) -> Option<R> {
        crate::css_layout::record::with_values(&self.css_record()?, f)
    }
    /// What a CSS box tells its children's sync context: whether it is a
    /// grid (never a Rive stack) and its justify-items as Rive numbers it.
    pub(crate) fn css_child_context(&self) -> Option<(bool, bool, u8)> {
        use crate::mechanical_port::source::layout::layout_style_applier::YGJustify;
        self.with_css(|css| {
            // Rive's grid items read whether the container stretches.
            let justify = match css.choice(crate::css_layout::keys::JUSTIFY_ITEMS) {
                2 | 4 | 6 | 9 => YGJustify::FlexStart,
                3 | 5 | 7 => YGJustify::FlexEnd,
                8 => YGJustify::Center,
                _ => YGJustify::Stretch,
            };
            (
                css.choice(crate::css_layout::keys::DISPLAY) == 2,
                false,
                justify as u8,
            )
        })
    }
    /// The record's display changed, or the record was attached: in CSS mode
    /// `display: none` hides the box as the style's display none does. The
    /// record passes its value, since it is borrowed while it calls.
    pub(crate) fn css_display_changed_occurrence(owner: &CoreHandle, hidden: bool) {
        let changed = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().expect("layout component");
                let changed = layout.style_display_hidden() != hidden;
                layout.set_layout_flag(LayoutComponentFlags::StyleDisplayHidden, hidden);
                changed
            })
            .unwrap_or(false);
        if !changed {
            return;
        }
        let collapsed = owner
            .with(|object| {
                object
                    .as_component()
                    .expect("layout component")
                    .is_collapsed()
                    || hidden
            })
            .expect("live layout owner");
        Self::propagate_resolved_collapse_occurrence(owner, collapsed, None, None);
        Self::mark_layout_node_dirty_occurrence(owner, false);
    }
    /// A CSS box's display or flex direction changed. Its children's
    /// styles read its main axis and whether it is a grid, so each syncs
    /// again on the next style pass.
    pub(crate) fn css_flow_changed_occurrence(owner: &CoreHandle) {
        for (_, provider) in Self::layout_providers_occurrence(owner) {
            if provider.is_type_of(LayoutComponentBase::TYPE_KEY) {
                Self::mark_layout_node_dirty_occurrence(&provider, false);
            } else {
                provider.with_mut(|object| object.layout_provider_mark_node_dirty(false));
            }
        }
    }
    /// A CSS leaf box places its content (text, images, shapes, groups,
    /// nested artboards) at its content box's origin, its border and
    /// padding, as a browser does, at each layout: bound, animated and
    /// breakpoint padding all follow. The content is authored at (0, 0).
    /// Translated boxes leave content where it was authored, as Rive does,
    /// and so does a box with layout children (docs/CSS_LAYOUT.md, 5).
    fn place_css_content_occurrence(owner: &CoreHandle) {
        let Some((offset, children)) = owner
            .with(|object| {
                let layout = object.as_layout_component()?;
                if !layout.is_css_mode() {
                    return None;
                }
                // An artboard draws its background from its origin.
                let start = object.as_artboard().map_or(Vec2D::default(), |artboard| {
                    Vec2D::new(
                        -artboard.layout_width() * artboard.origin_x(),
                        -artboard.layout_height() * artboard.origin_y(),
                    )
                });
                let offset = if layout.is_leaf() {
                    let [top, _, _, left] = layout.solved_border;
                    let padding = layout.solved_padding;
                    Vec2D::new(
                        start.x + left + padding.left(),
                        start.y + top + padding.top(),
                    )
                } else {
                    Vec2D::default()
                };
                Some((offset, layout.base.children().to_vec()))
            })
            .flatten()
        else {
            return;
        };
        let mut changed = false;
        for child in children {
            // Laid-out children and the box's own foreground paint stay.
            if child.is_type_of(LayoutComponentBase::TYPE_KEY)
                || child.is_type_of(
                    crate::source::generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase::TYPE_KEY,
                )
            {
                continue;
            }
            changed |= child
                .with_mut(|object| {
                    object.layout_provider_handle().is_none()
                        && object
                            .as_transform_component_mut()
                            .is_some_and(|transform| transform.set_layout_content_offset(offset))
                })
                .unwrap_or(false);
        }
        // The content's world transforms follow, as when the box moves.
        if changed {
            owner.with_mut(|object| {
                if let Some(layout) = object.as_layout_component_mut() {
                    CoreCapabilities::world_transform_mark_dirty(layout);
                }
            });
        }
    }
    /// A record value that only the box's paint reads changed.
    pub(crate) fn css_paint_changed_occurrence(owner: &CoreHandle) {
        owner.with_mut(|object| object.component_add_dirt(ComponentDirt::PATH, false));
    }
    pub fn grid_lines(&self, rows: bool, out: &mut Vec<f32>) -> f32 {
        out.clear();
        out.extend_from_slice(if rows {
            &self.grid_row_lines
        } else {
            &self.grid_column_lines
        });
        if rows {
            self.grid_row_gap
        } else {
            self.grid_column_gap
        }
    }
    fn clear_virtual_grid_placement(&mut self, style: &mut Style) {
        if let Some(grid) = self.virtual_grid.as_mut() {
            if grid.column < 0 && grid.pinned {
                crate::source::layout::grid_track::GridTrack::sync_item_lines(style, 0, 0, 1, 1);
                grid.pinned = false;
            }
        }
    }
    fn apply_virtual_grid_placement(&mut self, style: &mut Style) {
        if let Some(grid) = self.virtual_grid.as_mut() {
            if grid.column >= 0 {
                crate::source::layout::grid_track::GridTrack::sync_item_lines(
                    style,
                    grid.column + 1,
                    grid.row + 1,
                    1,
                    1,
                );
                grid.pinned = true;
            }
        }
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
    /// A Rive stack; a CSS box never is one.
    pub fn is_stack_container(&self) -> bool {
        !self.is_css_mode()
            && self
                .with_style(LayoutComponentStyle::is_stack)
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
        self.style_handle()?;
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
        let css_parent = parent
            .as_ref()
            .and_then(|parent| {
                parent.with(|parent| {
                    parent
                        .as_layout_component()
                        .and_then(LayoutComponent::css_child_context)
                })
            })
            .flatten();
        let (parent_is_grid, parent_is_stack, container_justify_items) = active_parent_style
            .map(|snapshot| {
                (
                    snapshot.is_grid,
                    snapshot.is_stack,
                    snapshot.justify_items as u8,
                )
            })
            .or(css_parent)
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
        let width_scale = self.effective_width_scale_type();
        Some(LayoutSyncContext {
            parent_is_grid,
            parent_is_stack,
            container_justify_items: u32::from(container_justify_items),
            inline_hugs: width_scale == LayoutScaleType::Hug,
            width_fills: width_scale == LayoutScaleType::Fill,
            parent_is_row: if self.can_have_overrides() {
                self.has_layout_flag(LayoutComponentFlags::ParentIsRow)
            } else if let Some(snapshot) = active_parent_style {
                snapshot.is_row
            } else {
                self.effective_parent_is_row()
            },
            is_ltr: self.actual_direction() != LayoutDirection::Rtl,
            has_layout_parent: parent.is_some(),
            css_role: crate::css_layout::record::role(self.is_artboard(), false),
            css: self.is_css_mode(),
        })
    }
    pub fn sync_style(&mut self) {
        let Some(context) = self.style_sync_context(None) else {
            return;
        };
        let mut taffy_style = self.layout_data.take_style();
        self.clear_virtual_grid_placement(&mut taffy_style);
        let this = self.base.handle();
        let appliers = self
            .layout_data
            .appliers
            .as_deref()
            .cloned()
            .unwrap_or_default();
        for pass in 0..5 {
            for applier in &appliers {
                let mut apply = |applier: &dyn LayoutStyleApplier| match pass {
                    0 => applier.apply_base_style(&mut taffy_style, &context),
                    1 => applier.apply_container_style(&mut taffy_style, &context),
                    2 => applier.apply_item_style(&mut taffy_style, &context),
                    3 => applier.apply_placement_style(&mut taffy_style, &context),
                    _ => applier.apply_css_style(&mut taffy_style, &context),
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
        self.apply_virtual_grid_placement(&mut taffy_style);
        self.layout_data.set_style(taffy_style);
        for (child, provider) in Self::layout_providers_children(self.base.children(), false) {
            let excluded = matches!(child.core_type(), Some(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY | crate::mechanical_port::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY | crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY));
            if !excluded {
                Self::sync_provider_style_occurrence(&provider);
            }
        }
    }
    /// CSS style from the authored appliers. The owning tree resolves the
    /// compatibility minimum for unsupported intrinsic subtrees at solve time.
    pub fn layout_style(&self) -> &Style {
        self.layout_data.style()
    }
    /// A hugging leaf is measured, as upstream. A non-artboard leaf that holds
    /// non-reflowing content (a shape, image, drawing group, n-slice or
    /// joystick, or a nested artboard on an axis the box does not Fill) is
    /// measured too, but only for its automatic minimum, so the box keeps
    /// that content's natural size. Text in a box that does not hug keeps
    /// upstream's unmeasured answer. An artboard keeps upstream's rule: its
    /// leaf check resolves Solo children through the artboard itself, which
    /// its callers have borrowed.
    pub fn is_intrinsic_leaf(&self) -> bool {
        let hugs = self.hugs();
        (hugs
            || (!self.is_artboard()
                && (0..2).any(|axis| !self.natural_content_hosts(axis).is_empty())))
            && self.is_leaf()
    }
    /// Records the natural size this box measured for `host`.
    fn note_measured_natural(&self, host: &CoreHandle, natural: Option<Vec2D>) {
        let mut measured = self.measured_natural.borrow_mut();
        match measured.iter_mut().find(|(measured, _)| measured == host) {
            Some(entry) => entry.1 = natural,
            None => measured.push((host.clone(), natural)),
        }
    }
    /// The update pass recomputed something inside `host`, a child of
    /// `owner`. When `owner` measures drawing content and measured `host`,
    /// compare its natural size now, from the matrices and paths the pass
    /// computed, with the one measured; a change marks the box, which lays
    /// out on the next pass, as upstream does for a participant whose
    /// intrinsic bounds change (PathComposer::intrinsicBoundsChanged).
    /// Layout's own write-back is not authored, so it never registers.
    pub(crate) fn check_measured_host_occurrence(owner: &CoreHandle, host: &CoreHandle) {
        let measured = owner
            .with(|object| {
                let layout = object.as_layout_component()?;
                if !layout.measures_drawing_content() {
                    return None;
                }
                layout
                    .measured_natural
                    .borrow()
                    .iter()
                    .find(|(measured, _)| measured == host)
                    .map(|(_, natural)| *natural)
            })
            .flatten();
        let Some(measured) = measured else {
            return;
        };
        let Some(natural) = owner
            .with(|object| {
                object
                    .as_layout_component()
                    .map(|layout| layout.natural_of(host))
            })
            .flatten()
        else {
            return;
        };
        // Within a few ULPs, as upstream compares a participant's bounds.
        let moved =
            |a: f32, b: f32| (a - b).abs() > 16.0 * f32::EPSILON * a.abs().max(b.abs()).max(1.0);
        let changed = match (measured, natural) {
            (Some(a), Some(b)) => moved(a.x, b.x) || moved(a.y, b.y),
            (a, b) => a.is_some() != b.is_some(),
        };
        if !changed {
            return;
        }
        owner.with(|object| {
            if let Some(layout) = object.as_layout_component() {
                layout.note_measured_natural(host, natural);
            }
        });
        Self::mark_layout_node_dirty_occurrence(owner, false);
    }
    /// Whether this box measures drawing content for its natural size: a
    /// leaf (no child layout nodes) that hugs, or any leaf but an artboard,
    /// as in is_intrinsic_leaf without resolving its content hosts.
    pub(crate) fn measures_drawing_content(&self) -> bool {
        self.layout_children.is_empty() && (self.hugs() || !self.is_artboard())
    }
    /// Whether this box measures all its content, as a hugging Rive box
    /// does. A CSS box always does: an auto size is its content's size
    /// (docs/CSS_LAYOUT.md, 5).
    fn hugs(&self) -> bool {
        self.is_css_mode()
            || self
                .with_style(|style| style.intrinsically_sized())
                .unwrap_or(false)
    }
    /// The children whose natural size, from authored geometry, answers a
    /// non-hugging leaf's min-content query on `axis` (0 width, 1 height).
    /// A nested artboard is an opaque host: in a box that Fills that axis it
    /// has neither a definite nor a hugging size, so it contributes nothing.
    fn natural_content_hosts(&self, axis: usize) -> Vec<CoreHandle> {
        let fills = self
            .with_style(|style| {
                if axis == 0 {
                    style.width_scale_type()
                } else {
                    style.height_scale_type()
                }
            })
            .is_some_and(|scale| scale == LayoutScaleType::Fill);
        self.children()
            .iter()
            .filter(|child| {
                !child.is_type_of(LayoutComponentBase::TYPE_KEY)
                    && !(fills
                        && child
                            .with(|object| object.as_nested_artboard().is_some())
                            .unwrap_or(false))
                    && !child
                        .with_downcast::<crate::source::shapes::shape::Shape, _>(
                            crate::source::shapes::shape::Shape::is_resized_by_layout,
                        )
                        .unwrap_or(false)
                    && {
                        let natural = self.natural_of(child);
                        // What the update pass compares against (UNIV-4844).
                        self.note_measured_natural(child, natural);
                        natural.is_some()
                    }
            })
            .cloned()
            .collect()
    }
    /// The natural size of `host`, a child of this box. A CSS box measures
    /// all its content through its paths' transforms, and a translated box
    /// that hugs all its content as Rive does; a translated box that does
    /// not hug leaves out the shapes it resizes, its paint
    /// (docs/CSS_LAYOUT.md, 5).
    pub(crate) fn natural_of(&self, host: &CoreHandle) -> Option<Vec2D> {
        if self.is_css_mode() {
            crate::source::intrinsically_sizeable::natural_css_size_for_layout_host(host)
        } else if self.hugs() {
            crate::source::intrinsically_sizeable::natural_size_for_layout_host(host)
        } else {
            crate::source::intrinsically_sizeable::natural_content_size_for_layout_host(host)
        }
    }
    /// The hosts a measured Layout leaf asks on each axis: every non-layout
    /// child when it hugs, as upstream, and otherwise its natural content.
    fn measured_hosts(&self) -> [Vec<CoreHandle>; 2] {
        if self.hugs() {
            let hosts = self
                .children()
                .iter()
                .filter(|child| !child.is_type_of(LayoutComponentBase::TYPE_KEY))
                .cloned()
                .collect::<Vec<_>>();
            [hosts.clone(), hosts]
        } else {
            [self.natural_content_hosts(0), self.natural_content_hosts(1)]
        }
    }
    pub fn set_solved_layout(&mut self, layout: Layout, padding: LayoutPadding) {
        self.layout_data.solved_layout = layout;
        self.solved_padding = padding;
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
        self.layout_children_changed = true;
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
    /// Whether the real collection walk reaches this child and takes its node.
    /// Walking upward from the child is not equivalent through opaque containers.
    #[cfg(feature = "tools")]
    pub fn collects_for_layout(&self, child: &CoreHandle) -> bool {
        let mut found = false;
        for (component, _) in Self::layout_providers_children(self.base.children(), false) {
            if &component == child {
                found = true;
            }
        }
        found
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
        let size = self.propagated_size();
        if self.is_css_mode() {
            self.propagated_content = Some(size);
        }
        Self::propagate_size_to_children(
            self.base.children().to_vec(),
            self.is_hidden(),
            size,
            style,
            self.is_css_mode(),
        );
    }
    /// The size this box gives the content layout resizes: its border box,
    /// as Rive gives it, or in CSS mode its content box, where Taffy
    /// measured that content (docs/CSS_LAYOUT.md, 5).
    fn propagated_size(&self) -> Vec2D {
        let (width, height) = (self.layout.width(), self.layout.height());
        if !self.is_css_mode() {
            return Vec2D::new(width, height);
        }
        let [top, right, bottom, left] = self.solved_border;
        let padding = self.layout_padding;
        Vec2D::new(
            (width - padding.left() - padding.right() - left - right).max(0.0),
            (height - padding.top() - padding.bottom() - top - bottom).max(0.0),
        )
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

    /// `css`: the box is in CSS mode, where shapes and images are content
    /// that keeps its natural size, so only text follows the box
    /// (docs/CSS_LAYOUT.md, 5).
    fn propagate_size_to_children(
        children: Vec<CoreHandle>,
        hidden: bool,
        size: Vec2D,
        style: Option<(LayoutScaleType, LayoutScaleType, LayoutDirection)>,
        css: bool,
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
                        || (css && {
                            use crate::mechanical_port::source::core::CoreObject;
                            CoreObject::is_type_of(child, crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY)
                                || CoreObject::is_type_of(child, crate::source::generated::shapes::image_base::ImageBase::TYPE_KEY)
                        })
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
                    Self::propagate_size_to_children(children, false, size, style, css);
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
        let Some((children, hidden, size, style, css)) = owner.with(|object| {
            let layout = object.as_layout_component().unwrap();
            (
                object.as_container_component().unwrap().children().to_vec(),
                layout.is_hidden(),
                layout.propagated_size(),
                layout.with_style(|style| {
                    (
                        style.width_scale_type(),
                        style.height_scale_type(),
                        layout.actual_direction(),
                    )
                }),
                layout.is_css_mode(),
            )
        }) else {
            return;
        };
        if css {
            owner.with_mut(|object| {
                if let Some(layout) = object.as_layout_component_mut() {
                    layout.propagated_content = Some(size);
                }
            });
        }
        Self::propagate_size_to_children(children, hidden, size, style, css);
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

    /// Solve this layout root with stock Taffy (CSS semantics; layout is a
    /// deliberate divergence from upstream Rive's Yoga). The root keeps one
    /// persistent tree in which every LayoutComponent and LayoutParticipant
    /// below it, hosted artboards included, owns a stable node. A solve applies
    /// only queued style changes and changed child lists, so Taffy's caches
    /// carry every untouched subtree across solves.
    pub fn calculate_layout_occurrence(
        owner: &CoreHandle,
        available_width: f32,
        available_height: f32,
    ) {
        let Some((size, tree)) = owner
            .with_mut(|object| {
                let layout = object
                    .as_layout_component_mut()
                    .expect("layout calculation owner");
                // A measure callback (scripted layout) may request a solve of
                // the root it is being measured for; that root is solving.
                if layout.layout_solving {
                    return None;
                }
                layout.layout_solving = true;
                Some((
                    layout.layout_solve_available_size(available_width, available_height),
                    layout.layout_tree.take(),
                ))
            })
            .expect("layout calculation owner")
        else {
            return;
        };
        let mut touched = Vec::new();
        let mut removed = Vec::new();
        let mut tree = match tree {
            Some(mut tree) if Self::layout_tree_root_is_current(owner, &tree) => {
                let root = tree.root;
                Self::reconcile_layout_node(&mut tree, owner, root, &mut touched, &mut removed);
                tree
            }
            _ => {
                let mut tree = Box::new(LayoutTree::new(owner.clone(), LayoutNodeKind::Layout));
                let root = tree.root;
                Self::adopt_layout_node(&mut tree, owner, root, &mut touched, &mut removed);
                Self::sync_layout_root_style(&mut tree, owner, (size.x, size.y), &mut touched);
                tree
            }
        };
        for (node_owner, link) in tree.sweep(removed) {
            node_owner.with_mut(|object| {
                if let Some(data) = Self::layout_data_of(object) {
                    data.unlink(&link);
                }
            });
        }
        Self::flush_layout_queue(&mut tree, &mut touched);
        let same = |a: f32, b: f32| a == b || (a.is_nan() && b.is_nan());
        if !same(tree.root_available.0, size.x) || !same(tree.root_available.1, size.y) {
            Self::sync_layout_root_style(&mut tree, owner, (size.x, size.y), &mut touched);
        }
        Self::sync_minimum_contracts(&mut tree, &mut touched);
        let root = tree.root;
        let space = |value: f32| {
            if value.is_nan() {
                taffy::prelude::AvailableSpace::MaxContent
            } else {
                taffy::prelude::AvailableSpace::Definite(value)
            }
        };
        tree.taffy
            .compute_layout_with_measure(
                root,
                taffy::prelude::Size {
                    width: space(size.x),
                    height: space(size.y),
                },
                |inputs, _node, context, style| {
                    let measured = context
                        .filter(|context| context.measured)
                        .map(|context| (context.owner.clone(), context.kind));
                    // PerformLayout omits known dimensions in the leaf callback,
                    // even when the solver has already fixed the inline slot.
                    let intrinsic_width = inputs.known_dimensions.width.is_none()
                        && (style.size.width.is_auto() || style.size.width.is_sizing_keyword());
                    let width_limits =
                        crate::mechanical_port::source::layout::layout_tree::content_width_limits(
                            style,
                            inputs.parent_size,
                        );
                    let height_limits =
                        crate::mechanical_port::source::layout::layout_tree::content_height_limits(
                            style,
                            inputs.parent_size,
                        );
                    taffy::compute_leaf_layout(
                        inputs,
                        style,
                        |_, _| 0.0,
                        |known, available| match &measured {
                            Some((owner, kind)) => Self::measure_layout_node(
                                owner,
                                *kind,
                                known,
                                available,
                                intrinsic_width,
                                width_limits,
                                height_limits,
                            ),
                            None => taffy::prelude::Size::ZERO,
                        },
                    )
                },
            )
            .expect("valid layout calculation");
        Self::publish_layout_tree(&tree, &touched);
        #[cfg(debug_assertions)]
        Self::verify_layout_against_fresh_solve(owner, (size.x, size.y));
        owner.with_mut(|object| {
            let layout = object
                .as_layout_component_mut()
                .expect("layout calculation owner");
            layout.layout_tree = Some(tree);
            layout.layout_solving = false;
        });
    }

    /// Diagnostic (debug builds, `NUXIE_LAYOUT_VERIFY=<file>`): re-solve the
    /// root in a throwaway tree built from scratch and append every published
    /// layout that differs to that file. The persistent tree must agree with a fresh solve;
    /// a mismatch is a stale style, topology, cache or publish bug, never a
    /// CSS-versus-Yoga difference.
    #[cfg(debug_assertions)]
    fn verify_layout_against_fresh_solve(owner: &CoreHandle, available: (f32, f32)) {
        use crate::mechanical_port::source::layout::layout_tree::{
            root_style, virtual_item_styles,
        };
        use taffy::prelude::{NodeId, Size, TaffyTree};
        type Context = (CoreHandle, LayoutNodeKind, bool, [bool; 2]);
        static REPORT: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
        let Some(report) =
            REPORT.get_or_init(|| std::env::var_os("NUXIE_LAYOUT_VERIFY").map(Into::into))
        else {
            return;
        };
        fn build(
            taffy: &mut TaffyTree<Context>,
            owner: &CoreHandle,
            nodes: &mut Vec<(CoreHandle, NodeId)>,
            parent: Option<&Style>,
            is_root: bool,
        ) -> Option<NodeId> {
            let parent_is_flex =
                parent.is_some_and(|parent| parent.display == taffy::Display::Flex);
            let (mut style, kind, measured, children, virtual_tracks) = owner.with(|object| {
                if let Some(layout) = object.as_layout_component() {
                    Some((
                        layout.layout_data.style().clone(),
                        LayoutNodeKind::Layout,
                        layout.is_intrinsic_leaf(),
                        layout.layout_children.clone(),
                        (
                            layout.grid_virtual_rows.clone(),
                            layout.grid_virtual_columns.clone(),
                        ),
                    ))
                } else {
                    let data = object
                        .as_any()
                        .downcast_ref::<crate::mechanical_port::source::layout::layout_participant::LayoutParticipant>()?
                        .native_layout_data()?;
                    Some((
                        data.style().clone(),
                        LayoutNodeKind::Participant,
                        true,
                        Vec::new(),
                        Default::default(),
                    ))
                }
            })??;
            let mut ids = Vec::new();
            for child in &children {
                if let Some(child) = layout_node_owner_for(child) {
                    ids.extend(build(taffy, &child, nodes, Some(&style), false));
                }
            }
            for style in virtual_item_styles(&virtual_tracks.0, &virtual_tracks.1) {
                ids.push(taffy.new_leaf(style).ok()?);
            }
            let supported = LayoutComponent::own_minimum_contract(owner, kind, measured, is_root)
                .unwrap_or([true; 2]);
            LayoutComponent::apply_definite_artboard_minimum(owner, is_root, parent, &mut style);
            LayoutComponent::apply_minimum_contract(
                &mut style,
                supported.map(|known| !known && parent_is_flex),
            );
            let id = taffy
                .new_leaf_with_context(style, (owner.clone(), kind, measured, supported))
                .ok()?;
            taffy.set_children(id, &ids).ok()?;
            nodes.push((owner.clone(), id));
            Some(id)
        }
        let mut taffy = TaffyTree::<Context>::new();
        taffy.disable_rounding();
        let mut nodes = Vec::new();
        let Some(root) = build(&mut taffy, owner, &mut nodes, None, true) else {
            return;
        };
        let style = taffy.style(root).expect("fresh root").clone();
        taffy
            .set_style(root, root_style(&style, available))
            .expect("fresh root");
        let space = |value: f32| {
            if value.is_nan() {
                taffy::prelude::AvailableSpace::MaxContent
            } else {
                taffy::prelude::AvailableSpace::Definite(value)
            }
        };
        taffy
            .compute_layout_with_measure(
                root,
                Size {
                    width: space(available.0),
                    height: space(available.1),
                },
                |inputs, _node, context, style| {
                    let measured = context
                        .filter(|context| context.2)
                        .map(|context| (context.0.clone(), context.1));
                    // PerformLayout omits known dimensions in the leaf callback,
                    // even when the solver has already fixed the inline slot.
                    let intrinsic_width = inputs.known_dimensions.width.is_none()
                        && (style.size.width.is_auto() || style.size.width.is_sizing_keyword());
                    let width_limits =
                        crate::mechanical_port::source::layout::layout_tree::content_width_limits(
                            style,
                            inputs.parent_size,
                        );
                    let height_limits =
                        crate::mechanical_port::source::layout::layout_tree::content_height_limits(
                            style,
                            inputs.parent_size,
                        );
                    taffy::compute_leaf_layout(
                        inputs,
                        style,
                        |_, _| 0.0,
                        |known, space| match &measured {
                            Some((owner, kind)) => Self::measure_layout_node(
                                owner,
                                *kind,
                                known,
                                space,
                                intrinsic_width,
                                width_limits,
                                height_limits,
                            ),
                            None => Size::ZERO,
                        },
                    )
                },
            )
            .expect("fresh layout");
        for (node_owner, id) in nodes {
            let output = taffy.layout(id).expect("fresh node");
            let fresh = Layout::new(
                output.location.x,
                output.location.y,
                output.size.width,
                output.size.height,
            );
            let published = node_owner.with(|object| {
                if let Some(layout) = object.as_layout_component() {
                    Some(layout.layout_data.solved_layout)
                } else {
                    object
                        .as_any()
                        .downcast_ref::<crate::mechanical_port::source::layout::layout_participant::LayoutParticipant>()?
                        .native_layout_data()
                        .map(|data| data.solved_layout)
                }
            });
            if let Some(Some(published)) = published {
                if published != fresh {
                    use std::io::Write;
                    if let Ok(mut file) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(report)
                    {
                        let _ = writeln!(
                            file,
                            "mismatch: core type {:?} published {published:?} fresh {fresh:?}",
                            node_owner.core_type()
                        );
                    }
                }
            }
        }
    }

    fn layout_data_of(
        object: &mut dyn crate::mechanical_port::source::core::CoreObject,
    ) -> Option<&mut LayoutData> {
        if object.as_layout_component().is_some() {
            return object
                .as_layout_component_mut()
                .map(|layout| &mut *layout.layout_data);
        }
        object
            .as_any_mut()
            .downcast_mut::<crate::mechanical_port::source::layout::layout_participant::LayoutParticipant>()?
            .native_layout_data_mut()
    }

    fn layout_tree_root_is_current(owner: &CoreHandle, tree: &LayoutTree) -> bool {
        owner
            .with(|object| {
                object
                    .as_layout_component()
                    .and_then(|layout| layout.layout_data.node().cloned())
            })
            .flatten()
            .is_some_and(|link| link.id() == tree.root && tree.owns(&link, owner))
    }

    /// The node `owner` holds in `tree`, adopting it (and its subtree) if it
    /// holds none there yet.
    fn ensure_layout_node(
        tree: &mut LayoutTree,
        owner: &CoreHandle,
        touched: &mut Vec<taffy::prelude::NodeId>,
        removed: &mut Vec<taffy::prelude::NodeId>,
    ) -> Option<taffy::prelude::NodeId> {
        let (link, kind) = owner
            .with_mut(|object| {
                let kind = if object.as_layout_component().is_some() {
                    LayoutNodeKind::Layout
                } else {
                    LayoutNodeKind::Participant
                };
                Some((Self::layout_data_of(object)?.node().cloned(), kind))
            })
            .flatten()?;
        if let Some(link) = link.filter(|link| tree.owns(link, owner)) {
            return Some(link.id());
        }
        let id = tree.new_node(owner.clone(), kind);
        Self::adopt_layout_node(tree, owner, id, touched, removed);
        Some(id)
    }

    /// Link `owner` to the fresh node `id` and build the subtree below it.
    fn adopt_layout_node(
        tree: &mut LayoutTree,
        owner: &CoreHandle,
        id: taffy::prelude::NodeId,
        touched: &mut Vec<taffy::prelude::NodeId>,
        removed: &mut Vec<taffy::prelude::NodeId>,
    ) {
        let link = tree.link(id);
        let Some((style, measured, children, virtual_tracks, stale_tree)) = owner
            .with_mut(|object| {
                if let Some(layout) = object.as_layout_component_mut() {
                    layout.layout_tree_topology_dirty = false;
                    layout.layout_children_changed = false;
                    layout.virtual_grid_changed = false;
                    let measured = layout.is_intrinsic_leaf();
                    Some((
                        layout.layout_data.link(link),
                        measured,
                        layout.layout_children.clone(),
                        (
                            layout.grid_virtual_rows.clone(),
                            layout.grid_virtual_columns.clone(),
                        ),
                        // A former root (a hosted artboard now mounted) gives
                        // up its own tree; a later solo solve rebuilds it.
                        layout.layout_tree.take(),
                    ))
                } else {
                    Self::layout_data_of(object)
                        .map(|data| (data.link(link), true, Vec::new(), Default::default(), None))
                }
            })
            .flatten()
        else {
            return;
        };
        // Release its owner handles outside the owner's borrow.
        drop(stale_tree);
        tree.taffy.set_style(id, style).expect("live layout node");
        tree.taffy
            .get_node_context_mut(id)
            .expect("live layout node")
            .measured = measured;
        touched.push(id);
        Self::set_layout_node_children(tree, id, &children, &virtual_tracks, touched, removed);
    }

    fn set_layout_node_children(
        tree: &mut LayoutTree,
        id: taffy::prelude::NodeId,
        children: &[LayoutNodeKey],
        (virtual_rows, virtual_columns): &(Vec<f32>, Vec<f32>),
        touched: &mut Vec<taffy::prelude::NodeId>,
        removed: &mut Vec<taffy::prelude::NodeId>,
    ) {
        use taffy::TraversePartialTree;
        let mut ids = Vec::with_capacity(children.len());
        for child in children {
            let Some(child_owner) = layout_node_owner_for(child) else {
                continue;
            };
            if let Some(child_id) = Self::ensure_layout_node(tree, &child_owner, touched, removed) {
                ids.push(child_id);
            }
        }
        ids.extend(tree.sync_virtual_items(id, virtual_rows, virtual_columns));
        let old = tree.taffy.child_ids(id).collect::<Vec<_>>();
        if old != ids {
            let kept = ids
                .iter()
                .copied()
                .collect::<std::collections::HashSet<_>>();
            removed.extend(old.into_iter().filter(|child| !kept.contains(child)));
            tree.taffy.set_children(id, &ids).expect("live layout node");
            touched.push(id);
        }
    }

    /// Apply child-list changes below `id`, descending only along the marked
    /// ancestor chains of changed nodes.
    fn reconcile_layout_node(
        tree: &mut LayoutTree,
        owner: &CoreHandle,
        id: taffy::prelude::NodeId,
        touched: &mut Vec<taffy::prelude::NodeId>,
        removed: &mut Vec<taffy::prelude::NodeId>,
    ) {
        use taffy::TraversePartialTree;
        let Some(changed) = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut()?;
                let topology = std::mem::take(&mut layout.layout_tree_topology_dirty);
                let children = std::mem::take(&mut layout.layout_children_changed);
                if !topology && !children {
                    return None;
                }
                Some(children.then(|| {
                    layout.virtual_grid_changed = false;
                    (
                        layout.layout_children.clone(),
                        (
                            layout.grid_virtual_rows.clone(),
                            layout.grid_virtual_columns.clone(),
                        ),
                        layout.is_intrinsic_leaf(),
                    )
                }))
            })
            .flatten()
        else {
            return;
        };
        if let Some((children, virtual_tracks, measured)) = changed {
            let context = tree
                .taffy
                .get_node_context_mut(id)
                .expect("live layout node");
            if context.measured != measured {
                context.measured = measured;
                tree.taffy.mark_dirty(id).expect("live layout node");
                touched.push(id);
            }
            Self::set_layout_node_children(tree, id, &children, &virtual_tracks, touched, removed);
        }
        let children = tree
            .taffy
            .child_ids(id)
            .filter_map(|child| {
                tree.taffy
                    .get_node_context(child)
                    .filter(|context| context.kind == LayoutNodeKind::Layout)
                    .map(|context| (child, context.owner.clone()))
            })
            .collect::<Vec<_>>();
        for (child, child_owner) in children {
            Self::reconcile_layout_node(tree, &child_owner, child, touched, removed);
        }
    }

    /// Copy queued style changes into the tree and drop the cached results of
    /// changed nodes; every queued node is republished.
    fn flush_layout_queue(tree: &mut LayoutTree, touched: &mut Vec<taffy::prelude::NodeId>) {
        use crate::mechanical_port::source::layout::layout_tree::root_style;
        for id in tree.take_queue() {
            let Some(context) = tree.taffy.get_node_context(id) else {
                continue;
            };
            let owner = context.owner.clone();
            let link = tree.link(id);
            let is_root = id == tree.root;
            let root_available = tree.root_available;
            let virtual_tracks = owner.with_mut(|object| {
                let measured = object
                    .as_layout_component()
                    .map(LayoutComponent::is_intrinsic_leaf);
                let virtual_tracks = object.as_layout_component_mut().and_then(|layout| {
                    std::mem::take(&mut layout.virtual_grid_changed).then(|| {
                        (
                            layout.grid_virtual_rows.clone(),
                            layout.grid_virtual_columns.clone(),
                        )
                    })
                });
                let Some(data) = Self::layout_data_of(object) else {
                    return None;
                };
                let Some(update) = data.take_update(&link) else {
                    return virtual_tracks;
                };
                let mut invalidate = update.invalidated;
                if let Some(style) = update.style {
                    invalidate |= if is_root {
                        tree.sync_style(id, &root_style(style, root_available))
                    } else {
                        tree.sync_style(id, style)
                    };
                }
                if let Some(measured) = measured {
                    let context = tree
                        .taffy
                        .get_node_context_mut(id)
                        .expect("live layout node");
                    if context.measured != measured {
                        context.measured = measured;
                        invalidate = true;
                    }
                }
                if invalidate {
                    tree.taffy.mark_dirty(id).expect("live layout node");
                }
                touched.push(id);
                virtual_tracks
            });
            if let Some(Some((rows, columns))) = virtual_tracks {
                Self::sync_layout_virtual_items(tree, id, &rows, &columns, touched);
            }
        }
    }

    /// Minimum contracts belong to measured leaves. Containers, including a
    /// mounted artboard's root, keep CSS auto minima, so every known child and
    /// definite size still contributes; only a leaf whose host has no answer
    /// keeps a zero automatic minimum.
    fn own_minimum_contract(
        owner: &CoreHandle,
        kind: LayoutNodeKind,
        measured: bool,
        is_root: bool,
    ) -> Option<[bool; 2]> {
        use crate::source::layout::layout_participant::{
            LayoutParticipant, host_supports_min_content_width,
        };
        if !is_root
            && owner
                .with(|object| object.as_artboard().is_some())
                .unwrap_or(false)
        {
            return Some([true; 2]);
        }
        let hosts: Option<Vec<CoreHandle>> = owner
            .with(|object| {
                if !measured {
                    return None;
                }
                Some(match kind {
                    LayoutNodeKind::Participant => object
                        .as_any()
                        .downcast_ref::<LayoutParticipant>()
                        .and_then(LayoutParticipant::measurement_host_handle)
                        .into_iter()
                        .collect(),
                    LayoutNodeKind::Layout => {
                        let [width, height] = object
                            .as_layout_component()
                            .expect("layout measurement owner")
                            .measured_hosts();
                        let extra = height
                            .into_iter()
                            .filter(|host| !width.contains(host))
                            .collect::<Vec<_>>();
                        width.into_iter().chain(extra).collect()
                    }
                })
            })
            .flatten();
        hosts.map(|hosts| [hosts.iter().all(host_supports_min_content_width); 2])
    }

    /// A mounted artboard is an opaque host. On an axis where it has a
    /// definite size, that size is its specified size suggestion, so a flex
    /// container never squeezes it below its authored size. A CSS automatic
    /// minimum is clamped by a definite maximum, while an explicit minimum
    /// would override it, so the synthesized minimum takes the smaller of the
    /// two. A percentage maximum is resolved against `parent`'s content box
    /// when the parent's size on that axis is a definite length; otherwise
    /// it is treated as not limiting, so the artboard keeps its size. A
    /// scroll container's automatic minimum is zero, so its axes get none.
    fn apply_definite_artboard_minimum(
        owner: &CoreHandle,
        is_root: bool,
        parent: Option<&Style>,
        style: &mut Style,
    ) {
        use taffy::style::CompactLength;
        if is_root
            || !owner
                .with(|object| object.as_artboard().is_some())
                .unwrap_or(false)
        {
            return;
        }
        let parent_content = |horizontal: bool| -> Option<f32> {
            let parent = parent?;
            let size = if horizontal {
                parent.size.width
            } else {
                parent.size.height
            };
            if size.tag() != CompactLength::LENGTH_TAG {
                return None;
            }
            if parent.box_sizing == taffy::BoxSizing::ContentBox {
                return Some(size.value());
            }
            let edges = if horizontal {
                [
                    parent.padding.left.into_raw(),
                    parent.padding.right.into_raw(),
                    parent.border.left.into_raw(),
                    parent.border.right.into_raw(),
                ]
            } else {
                [
                    parent.padding.top.into_raw(),
                    parent.padding.bottom.into_raw(),
                    parent.border.top.into_raw(),
                    parent.border.bottom.into_raw(),
                ]
            };
            let mut inner = size.value();
            for edge in edges {
                if edge.tag() != CompactLength::LENGTH_TAG {
                    return None;
                }
                inner -= edge.value();
            }
            Some(inner.max(0.0))
        };
        let minimum = |size: taffy::prelude::Dimension,
                       maximum: taffy::prelude::LengthPercentageAuto,
                       horizontal: bool| {
            if size.tag() != CompactLength::LENGTH_TAG {
                return None;
            }
            let maximum = maximum.into_raw();
            if maximum.is_auto() {
                Some(size.value())
            } else if maximum.tag() == CompactLength::LENGTH_TAG {
                Some(size.value().min(maximum.value()))
            } else if maximum.tag() == CompactLength::PERCENT_TAG {
                Some(parent_content(horizontal).map_or(size.value(), |inner| {
                    size.value().min(inner * maximum.value())
                }))
            } else {
                None
            }
        };
        if style.min_size.width.is_auto()
            && !style.overflow.x.is_scroll_container()
            && let Some(width) = minimum(style.size.width, style.max_size.width, true)
        {
            style.min_size.width = taffy::prelude::LengthPercentageAuto::length(width);
        }
        if style.min_size.height.is_auto()
            && !style.overflow.y.is_scroll_container()
            && let Some(height) = minimum(style.size.height, style.max_size.height, false)
        {
            style.min_size.height = taffy::prelude::LengthPercentageAuto::length(height);
        }
    }

    fn apply_minimum_contract(style: &mut Style, zero: [bool; 2]) {
        if zero[0] && style.min_size.width.is_auto() {
            style.min_size.width = taffy::prelude::LengthPercentageAuto::length(0.0);
        }
        if zero[1] && style.min_size.height.is_auto() {
            style.min_size.height = taffy::prelude::LengthPercentageAuto::length(0.0);
        }
    }

    /// Revisit only changed nodes and their ancestors. Cached contract bits
    /// keep untouched subtrees out of the walk, just like the retained solve.
    fn sync_minimum_contracts(tree: &mut LayoutTree, touched: &mut Vec<taffy::prelude::NodeId>) {
        use taffy::TraversePartialTree;
        let mut affected = std::collections::HashSet::new();
        for &id in touched.iter() {
            let mut next = Some(id);
            while let Some(id) = next {
                if !affected.insert(id) {
                    break;
                }
                next = tree.taffy.parent(id);
            }
        }
        if affected.is_empty() {
            return;
        }
        fn update(
            tree: &mut LayoutTree,
            id: taffy::prelude::NodeId,
            affected: &std::collections::HashSet<taffy::prelude::NodeId>,
            touched: &mut Vec<taffy::prelude::NodeId>,
        ) {
            use crate::source::layout::layout_participant::LayoutParticipant;
            use crate::source::layout::layout_tree::root_style;
            let Some(context) = tree.taffy.get_node_context(id) else {
                return;
            };
            let (owner, kind, measured, old_supported, old_zero) = (
                context.owner.clone(),
                context.kind,
                context.measured,
                context.minimum_supported,
                context.zero_auto_minimums,
            );
            let changed = affected.contains(&id);
            let authored_style = || {
                owner
                    .with(|object| {
                        if let Some(layout) = object.as_layout_component() {
                            layout.layout_data.style().clone()
                        } else {
                            object
                                .as_any()
                                .downcast_ref::<LayoutParticipant>()
                                .unwrap()
                                .native_layout_data()
                                .unwrap()
                                .style()
                                .clone()
                        }
                    })
                    .unwrap()
            };
            let supported = if changed {
                for child in tree.taffy.child_ids(id).collect::<Vec<_>>() {
                    update(tree, child, affected, touched);
                }
                LayoutComponent::own_minimum_contract(&owner, kind, measured, id == tree.root)
                    .unwrap_or([true; 2])
            } else {
                old_supported
            };
            let parent_is_flex = tree.taffy.parent(id).is_some_and(|parent| {
                tree.taffy
                    .style(parent)
                    .is_ok_and(|style| style.display == taffy::Display::Flex)
            });
            let zero = supported.map(|known| !known && parent_is_flex);
            // A mounted artboard's minimum reads its parent's style: a
            // percentage maximum resolves against the parent's length.
            let parent_dependent = id != tree.root
                && tree
                    .taffy
                    .parent(id)
                    .is_some_and(|parent| affected.contains(&parent))
                && owner
                    .with(|object| object.as_artboard().is_some())
                    .unwrap_or(false);
            if changed || zero != old_zero || parent_dependent {
                let mut style = authored_style();
                if id == tree.root {
                    style = root_style(&style, tree.root_available);
                }
                let parent_style = tree
                    .taffy
                    .parent(id)
                    .and_then(|parent| tree.taffy.style(parent).ok().cloned());
                LayoutComponent::apply_definite_artboard_minimum(
                    &owner,
                    id == tree.root,
                    parent_style.as_ref(),
                    &mut style,
                );
                LayoutComponent::apply_minimum_contract(&mut style, zero);
                let context = tree.taffy.get_node_context_mut(id).unwrap();
                context.minimum_supported = supported;
                context.zero_auto_minimums = zero;
                if tree.sync_style(id, &style) {
                    touched.push(id);
                }
            }
        }
        update(tree, tree.root, &affected, touched);
    }

    /// Replace a grid's virtual items, keeping its realized children.
    fn sync_layout_virtual_items(
        tree: &mut LayoutTree,
        id: taffy::prelude::NodeId,
        rows: &[f32],
        columns: &[f32],
        touched: &mut Vec<taffy::prelude::NodeId>,
    ) {
        use taffy::TraversePartialTree;
        let old = tree.taffy.child_ids(id).collect::<Vec<_>>();
        let previous = tree
            .taffy
            .get_node_context(id)
            .expect("live layout node")
            .virtual_items
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>();
        let mut ids = old
            .iter()
            .copied()
            .filter(|child| !previous.contains(child))
            .collect::<Vec<_>>();
        ids.extend(tree.sync_virtual_items(id, rows, columns));
        if tree.taffy.child_ids(id).collect::<Vec<_>>() != ids {
            tree.taffy.set_children(id, &ids).expect("live layout node");
            touched.push(id);
        }
    }

    fn sync_layout_root_style(
        tree: &mut LayoutTree,
        owner: &CoreHandle,
        available: (f32, f32),
        touched: &mut Vec<taffy::prelude::NodeId>,
    ) {
        use crate::mechanical_port::source::layout::layout_tree::root_style;
        tree.root_available = available;
        let style = owner
            .with(|object| {
                object
                    .as_layout_component()
                    .map(|layout| root_style(layout.layout_data.style(), available))
            })
            .flatten();
        if let Some(style) = style {
            let root = tree.root;
            if tree.sync_style(root, &style) {
                touched.push(root);
            }
        }
    }

    fn measure_layout_node(
        owner: &CoreHandle,
        kind: LayoutNodeKind,
        known: taffy::prelude::Size<Option<f32>>,
        available: taffy::prelude::Size<taffy::prelude::AvailableSpace>,
        intrinsic_width: bool,
        width_limits: (Option<f32>, Option<f32>),
        height_limits: (Option<f32>, Option<f32>),
    ) -> taffy::prelude::Size<f32> {
        use crate::mechanical_port::source::layout::{
            layout_measure_mode::unbound_measure_mode,
            layout_participant::{
                LayoutParticipant, host_min_content_height, host_min_content_width,
                host_supports_min_content_width,
            },
            layout_tree::{measure_axis, measure_fit_content},
        };
        let css_owner = owner
            .with(|object| {
                object
                    .as_layout_component()
                    .is_some_and(LayoutComponent::is_css_mode)
            })
            .unwrap_or(false);
        let measure_host = |host: &CoreHandle,
                            width: f32,
                            width_mode: LayoutMeasureMode,
                            height: f32,
                            height_mode: LayoutMeasureMode|
         -> Vec2D {
            #[cfg(test)]
            layout_fit_content_tests::note_host_measure();
            // Image already measures exact offers as exact sizes. Preserve that
            // protocol; controlled geometry and drawing groups use authored bounds.
            // In a CSS box an image is content and keeps its natural size.
            let natural = if host
                .is_type_of(crate::source::generated::shapes::image_base::ImageBase::TYPE_KEY)
                && !css_owner
            {
                None
            } else {
                // What the update pass compares against (UNIV-4844).
                owner
                    .with(|object| match object.as_layout_component() {
                        Some(layout) => {
                            let natural = layout.natural_of(host);
                            layout.note_measured_natural(host, natural);
                            natural
                        }
                        None => {
                            crate::source::intrinsically_sizeable::natural_size_for_layout_host(
                                host,
                            )
                        }
                    })
                    .flatten()
            };
            let mut measured = if let Some(natural) = natural {
                // CSS content keeps its natural size past its box, so its
                // overflow reaches the scrollable area; Rive's measure
                // clamps it to the offer.
                let axis = |natural: f32, offer: f32, mode| {
                    if mode == LayoutMeasureMode::Undefined || css_owner {
                        natural
                    } else {
                        natural.min(offer)
                    }
                };
                Vec2D::new(
                    axis(natural.x, width, width_mode),
                    axis(natural.y, height, height_mode),
                )
            } else {
                host.with_mut(|object| {
                    object.as_intrinsically_sizeable_mut().map(|sizeable| {
                        sizeable.measure_layout(width, width_mode, height, height_mode)
                    })
                })
                .flatten()
                .unwrap_or_default()
            };
            if known.height.is_none()
                && available.height == taffy::prelude::AvailableSpace::MinContent
            {
                if let Some(minimum) =
                    host_min_content_height(host, known.width, width_limits, css_owner)
                {
                    measured.y = minimum;
                }
            }
            measured
        };
        // An unbounded axis carries no number.
        let offer = |value: f32, mode: LayoutMeasureMode| {
            if mode == LayoutMeasureMode::Undefined {
                f32::NAN
            } else {
                value
            }
        };
        // Min-content is a content contribution even for a Fixed item:
        // Taffy caps its automatic minimum by the specified size itself.
        let min_content =
            known.width.is_none() && available.width == taffy::prelude::AvailableSpace::MinContent;
        let (width, width_mode) = measure_axis(known.width, available.width);
        let (height, height_mode) = measure_axis(known.height, available.height);
        let layout_parent = owner
            .with(|object| {
                if let Some(layout) = object.as_layout_component() {
                    layout.layout_parent_handle()
                } else {
                    object
                        .as_any()
                        .downcast_ref::<LayoutParticipant>()
                        .and_then(LayoutParticipant::owning_layout_handle)
                }
            })
            .flatten();
        let mut ancestor = layout_parent;
        let parent_is_grid = loop {
            let Some(parent) = ancestor else {
                break false;
            };
            let Some((is_grid, next)) = parent
                .with(|object| {
                    object.as_layout_component().map(|layout| {
                        (
                            layout
                                .with_style(LayoutComponentStyle::is_grid)
                                .unwrap_or(false),
                            layout.layout_parent_handle(),
                        )
                    })
                })
                .flatten()
            else {
                break false;
            };
            if is_grid {
                break true;
            }
            ancestor = next;
        };
        // An unimplemented minimum is zero, as before automatic minima were
        // enabled. An unbounded Rive answer is max-content, not a substitute
        // minimum. Keep this query-time decision live when host modes change.
        let minimum_contract = |mut measured: Vec2D, supports: bool| {
            // Grid and stack items retain their existing CSS auto contribution.
            if !supports && !parent_is_grid {
                if known.width.is_none()
                    && available.width == taffy::prelude::AvailableSpace::MinContent
                {
                    measured.x = 0.0;
                }
                if known.height.is_none()
                    && available.height == taffy::prelude::AvailableSpace::MinContent
                {
                    measured.y = 0.0;
                }
            }
            measured
        };
        let measured = match kind {
            LayoutNodeKind::Participant => {
                let Some((host, width_mode, height_mode)) = owner
                    .with(|object| {
                        let participant = object.as_any().downcast_ref::<LayoutParticipant>()?;
                        let hug = |scale: u8, mode: LayoutMeasureMode| {
                            if participant.hug_unbounded() && scale == LayoutScaleType::Hug as u8 {
                                unbound_measure_mode(mode)
                            } else {
                                mode
                            }
                        };
                        Some((
                            participant.measurement_host_handle(),
                            hug(participant.layout_width_scale_type(), width_mode),
                            hug(participant.layout_height_scale_type(), height_mode),
                        ))
                    })
                    .flatten()
                else {
                    return taffy::prelude::Size::ZERO;
                };
                host.map(|host| {
                    let supports = host_supports_min_content_width(&host);
                    minimum_contract(
                        measure_fit_content(
                            (offer(width, width_mode), width_mode),
                            (offer(height, height_mode), height_mode),
                            width_limits,
                            min_content,
                            (intrinsic_width || min_content) && supports,
                            || {
                                host_min_content_width(
                                    &host,
                                    known.height,
                                    height_limits,
                                    css_owner,
                                )
                            },
                            |width, width_mode, height, height_mode| {
                                measure_host(&host, width, width_mode, height, height_mode)
                            },
                        ),
                        supports,
                    )
                })
                .unwrap_or_default()
            }
            LayoutNodeKind::Layout => {
                let Some((hosts, (width_mode, height_mode), hugs)) = owner.with(|object| {
                    let layout = object
                        .as_layout_component()
                        .expect("layout measurement owner");
                    (
                        layout.measured_hosts(),
                        layout.content_measure_modes(width_mode, height_mode),
                        layout.hugs(),
                    )
                }) else {
                    return taffy::prelude::Size::ZERO;
                };
                // A leaf that does not hug is measured only for its automatic
                // minimum. Every other request keeps upstream's unmeasured
                // answer, so its basis and Rive's measure calls are unchanged.
                let min_content_axes = [
                    min_content,
                    known.height.is_none()
                        && available.height == taffy::prelude::AvailableSpace::MinContent,
                ];
                if !hugs && min_content_axes == [false; 2] {
                    return taffy::prelude::Size::ZERO;
                }
                let width = offer(width, width_mode);
                let height = offer(height, height_mode);
                let measure_children = |children: &[CoreHandle]| {
                    let supports = children.iter().all(host_supports_min_content_width);
                    minimum_contract(
                        measure_fit_content(
                            (width, width_mode),
                            (height, height_mode),
                            width_limits,
                            min_content,
                            (intrinsic_width || min_content) && supports,
                            || {
                                children.iter().try_fold(0.0f32, |minimum, child| {
                                    host_min_content_width(
                                        child,
                                        known.height,
                                        height_limits,
                                        css_owner,
                                    )
                                    .map(|next| minimum.max(next))
                                })
                            },
                            |width, width_mode, height, height_mode| {
                                let mut measured = Vec2D::default();
                                for child in children {
                                    let next =
                                        measure_host(child, width, width_mode, height, height_mode);
                                    measured.x = measured.x.max(next.x);
                                    measured.y = measured.y.max(next.y);
                                }
                                measured
                            },
                        ),
                        supports,
                    )
                };
                if hugs {
                    measure_children(&hosts[0])
                } else {
                    Vec2D::new(
                        if min_content_axes[0] && !hosts[0].is_empty() {
                            measure_children(&hosts[0]).x
                        } else {
                            0.0
                        },
                        if min_content_axes[1] && !hosts[1].is_empty() {
                            measure_children(&hosts[1]).y
                        } else {
                            0.0
                        },
                    )
                }
            }
        };
        taffy::prelude::Size {
            width: measured.x,
            height: measured.y,
        }
    }

    /// Taffy places an absolute box against its containing block, its
    /// nearest positioned ancestor, which in CSS mode can be above its
    /// parent: a static box is not one. Owners keep parent-relative bounds,
    /// so this is the parent's offset from that containing block (zero when
    /// the parent is it, as for every translated box).
    fn hoisted_offset(tree: &LayoutTree, id: taffy::prelude::NodeId) -> Vec2D {
        use taffy::TraversePartialTree;
        let mut offset = Vec2D::default();
        let positioned = |node| {
            tree.taffy
                .style(node)
                .is_ok_and(|style| style.position != taffy::style::Position::Static)
        };
        if !tree
            .taffy
            .style(id)
            .is_ok_and(|style| style.position == taffy::style::Position::Absolute)
        {
            return offset;
        }
        let mut ancestor = tree.taffy.parent(id);
        while let Some(node) = ancestor {
            let parent = tree.taffy.parent(node);
            if parent.is_none() || positioned(node) {
                break;
            }
            if let Ok(layout) = tree.taffy.layout(node) {
                offset = Vec2D::new(offset.x + layout.location.x, offset.y + layout.location.y);
            }
            ancestor = parent;
        }
        offset
    }
    /// Publish solved layouts to their owners. Descends only into nodes whose
    /// result changed or whose subtree held a touched node: Taffy leaves an
    /// untouched subtree's results in place when its parent's result holds.
    fn publish_layout_tree(tree: &LayoutTree, touched: &[taffy::prelude::NodeId]) {
        use crate::mechanical_port::source::layout::layout_tree::{
            grid_lines, is_grid, resolve_gap,
        };
        use taffy::TraversePartialTree;
        let mut visited = std::collections::HashSet::with_capacity(touched.len() * 2);
        for &node in touched {
            let mut current = Some(node);
            while let Some(node) = current {
                if !visited.insert(node) {
                    break;
                }
                current = tree.taffy.parent(node);
            }
        }
        // Each node, and whether it was reached only past an unchanged static
        // box (below).
        let mut stack = vec![(tree.root, false)];
        while let Some((id, past_static)) = stack.pop() {
            let Some(context) = tree.taffy.get_node_context(id) else {
                continue;
            };
            let output = tree.taffy.layout(id).expect("solved layout node");
            let hoisted = Self::hoisted_offset(tree, id);
            let next = Layout::new(
                output.location.x - hoisted.x,
                output.location.y - hoisted.y,
                output.size.width,
                output.size.height,
            );
            let padding = LayoutPadding::new(
                output.padding.left,
                output.padding.top,
                output.padding.right,
                output.padding.bottom,
            );
            let border = [
                output.border.top,
                output.border.right,
                output.border.bottom,
                output.border.left,
            ];
            let scroll_range = Vec2D::new(output.scroll_width(), output.scroll_height());
            let grid = (context.kind == LayoutNodeKind::Layout).then(|| {
                let style = tree.taffy.style(id).expect("solved layout node");
                match tree.taffy.detailed_layout_info(id) {
                    taffy::tree::DetailedLayoutInfo::Grid(info) if is_grid(style) => {
                        let content_width = output.size.width
                            - output.padding.left
                            - output.padding.right
                            - output.border.left
                            - output.border.right;
                        let content_height = output.size.height
                            - output.padding.top
                            - output.padding.bottom
                            - output.border.top
                            - output.border.bottom;
                        let (columns, column_gap) =
                            grid_lines(&info.columns, resolve_gap(style.gap.width, content_width));
                        let (rows, row_gap) =
                            grid_lines(&info.rows, resolve_gap(style.gap.height, content_height));
                        (columns, rows, column_gap, row_gap)
                    }
                    _ => (Vec::new(), Vec::new(), 0.0, 0.0),
                }
            });
            let touched = visited.contains(&id);
            let changed = context
                .owner
                .with_mut(|object| {
                    if let Some(layout) = object.as_layout_component_mut() {
                        if let Some(grid) = grid {
                            (
                                layout.grid_column_lines,
                                layout.grid_row_lines,
                                layout.grid_column_gap,
                                layout.grid_row_gap,
                            ) = grid;
                        }
                        let padding_changed = layout.solved_padding != padding
                            || layout.solved_border != border
                            || layout.scroll_range != scroll_range;
                        layout.solved_padding = padding;
                        layout.solved_border = border;
                        layout.scroll_range = scroll_range;
                        let changed = padding_changed || layout.layout_data.solved_layout != next;
                        layout.layout_data.has_new_layout |= touched || changed;
                        layout.layout_data.solved_layout = next;
                        changed
                    } else if let Some(data) = Self::layout_data_of(object) {
                        let changed = data.solved_layout != next;
                        data.has_new_layout |= touched || changed;
                        data.solved_layout = next;
                        changed
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            let position = |node| {
                tree.taffy
                    .style(node)
                    .map_or(taffy::style::Position::Relative, |style| style.position)
            };
            if changed && past_static {
                // The bounds update descends only through boxes with a new
                // layout: open the way down to this one.
                let mut ancestor = tree.taffy.parent(id);
                while let Some(node) = ancestor {
                    if let Some(context) = tree.taffy.get_node_context(node) {
                        context.owner.with_mut(|object| {
                            if let Some(layout) = object.as_layout_component_mut() {
                                layout.layout_data.has_new_layout = true;
                            } else if let Some(data) = Self::layout_data_of(object) {
                                data.has_new_layout = true;
                            }
                        });
                    }
                    ancestor = tree.taffy.parent(node);
                }
            }
            if changed || touched {
                stack.extend(tree.taffy.child_ids(id).map(|child| (child, false)));
            } else if position(id) == taffy::style::Position::Static {
                // CSS mode: a static box is no containing block. An absolute
                // box below it is placed against an ancestor above, which may
                // have changed, and its published offset subtracts the static
                // boxes between, which may have moved.
                stack.extend(
                    tree.taffy
                        .child_ids(id)
                        .filter(|&child| {
                            matches!(
                                position(child),
                                taffy::style::Position::Static | taffy::style::Position::Absolute
                            )
                        })
                        .map(|child| (child, true)),
                );
            }
        }
    }
    pub fn style_display_hidden(&self) -> bool {
        self.has_layout_flag(LayoutComponentFlags::StyleDisplayHidden)
    }
    /// Taffy's maximum scroll offsets for this box: its scrollable overflow
    /// less its padding box, as `scrollWidth - clientWidth` in a browser.
    pub fn scroll_range(&self) -> Vec2D {
        self.scroll_range
    }
    /// The solved border widths: top, right, bottom, left.
    pub fn solved_border(&self) -> [f32; 4] {
        self.solved_border
    }
    fn refresh_style_display_hidden(&mut self) {
        let hidden = self
            .with_css(|css| css.display_none())
            .or_else(|| self.with_style(|style| style.display() == YGDisplay::None))
            .unwrap_or(false);
        self.set_layout_flag(LayoutComponentFlags::StyleDisplayHidden, hidden);
    }
    /// Whether this box sets its own direction rather than inheriting it:
    /// its record's in CSS mode, its style's otherwise.
    fn has_own_direction(&self) -> bool {
        if self.is_css_mode() {
            return self.with_css(|css| css.direction()).flatten().is_some();
        }
        self.with_style(|style| style.direction() != YGDirection::Inherit)
            .unwrap_or(false)
    }
    pub fn actual_direction(&self) -> LayoutDirection {
        if self.is_css_mode() {
            return match self.with_css(|css| css.direction()).flatten() {
                Some(true) => LayoutDirection::Ltr,
                Some(false) => LayoutDirection::Rtl,
                None => self.inherited_direction,
            };
        }
        self.with_style(|style| match style.direction() {
            YGDirection::Ltr => LayoutDirection::Ltr,
            YGDirection::Rtl => LayoutDirection::Rtl,
            _ => self.inherited_direction,
        })
        .unwrap_or(self.inherited_direction)
    }
    pub fn on_dirty(&mut self, value: ComponentDirt) {
        self.base.base.base.base.base.on_dirty(value);
        if value.contains(ComponentDirt::WORLD_TRANSFORM) && self.clips() {
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
        if self.has_layout_flag(LayoutComponentFlags::JustAddedToHost) {
            self.set_layout_flag(LayoutComponentFlags::JustAddedToHost, false);
            self.layout = next;
            if let Some(data) = self.current_animation_data() {
                data.from = next;
                data.to = next;
                data.elapsed_seconds = 0.0;
            }
            self.propagate_size();
            CoreCapabilities::world_transform_mark_dirty(self);
            self.set_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds, false);
            return;
        }
        if animate && self.animates() {
            let force = self.has_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds);
            let target = self
                .current_animation_data()
                .map(|data| data.to)
                .unwrap_or(self.layout);
            if next != target || force {
                self.ensure_animation();
                let data = self.current_animation_data().unwrap();
                if data.elapsed_seconds != 0.0 {
                    if self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
                        self.animation.as_mut().unwrap().a = self.animation.as_mut().unwrap().b;
                    }
                    self.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, true);
                } else {
                    self.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                }
                let from = self.layout;
                let data = self.current_animation_data().unwrap();
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
            if let Some(animation) = self.animation.as_mut() {
                animation.a.to = next;
            }
            self.propagate_size();
            CoreCapabilities::world_transform_mark_dirty(self);
        }
        self.set_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds, false);
    }
    fn hosted_layout_roots(provider: &CoreHandle) -> Option<(bool, Vec<i32>)> {
        provider.with(|object| {
            if let Some(nested) = object.as_any().downcast_ref::<crate::mechanical_port::source::nested_artboard_layout::NestedArtboardLayout>() {
                Some((false, nested.base.base.artboard_instance_handle(0).map(|_| 0).into_iter().collect()))
            } else if let Some(list) = object.as_any().downcast_ref::<crate::mechanical_port::source::artboard_component_list::ArtboardComponentList>() {
                let mut indices = Vec::new();
                list.realized_indices(&mut indices);
                Some((true, indices))
            } else { None }
        }).flatten()
    }
    fn hosted_layout_root(provider: &CoreHandle, index: i32) -> Option<CoreHandle> {
        provider.with(|object| {
            if let Some(nested) = object.as_any().downcast_ref::<crate::source::nested_artboard_layout::NestedArtboardLayout>() {
                nested.base.base.artboard_instance_handle(index).map(|instance| instance.core_handle())
            } else {
                object.as_any().downcast_ref::<crate::source::artboard_component_list::ArtboardComponentList>()?.item(index).map(|instance| instance.core_handle())
            }
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
            let has_virtual_window = is_list && provider.with_downcast::<crate::source::artboard_component_list::ArtboardComponentList, _>(|list| list.has_visible_virtual_window()).unwrap();
            for index in roots {
                let Some(root) = Self::hosted_layout_root(provider, index) else { continue; };
                Self::update_layout_bounds_occurrence(&root, animate);
                if is_list && (!has_virtual_window || provider.with_downcast::<crate::source::artboard_component_list::ArtboardComponentList, _>(|list| list.is_within_visible_window(index)).unwrap()) {
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
                (
                    layout.layout_data.solved_layout,
                    layout.layout,
                    layout.has_layout_flag(LayoutComponentFlags::JustAddedToHost),
                    layout.animates(),
                    layout.has_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds),
                    layout.current_animation_data().copied(),
                )
            })
            .unwrap();
        let mut changed = false;
        if just_added {
            owner.with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                layout.set_layout_flag(LayoutComponentFlags::JustAddedToHost, false);
                layout.layout = next;
                if let Some(data) = layout.current_animation_data() {
                    *data = LayoutAnimationData {
                        from: next,
                        to: next,
                        elapsed_seconds: 0.0,
                    };
                }
            });
            changed = true;
        } else if animate && animates {
            if next != current.map(|data| data.to).unwrap_or(old) || force {
                owner.with_mut(|object| {
                    let layout = object.as_layout_component_mut().unwrap();
                    layout.ensure_animation();
                    if layout.current_animation_data().unwrap().elapsed_seconds != 0.0 {
                        if layout.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
                            layout.animation.as_mut().unwrap().a =
                                layout.animation.as_mut().unwrap().b;
                        }
                        layout.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, true);
                    } else {
                        layout.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                    }
                    *layout.current_animation_data().unwrap() = LayoutAnimationData {
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
                if let Some(animation) = layout.animation.as_mut() {
                    animation.a.to = next;
                }
            });
            changed = true;
        }
        // A CSS box whose padding or border alone changed resizes its
        // content too (docs/CSS_LAYOUT.md, 5).
        let content_changed = owner
            .with(|object| {
                object.as_layout_component().is_some_and(|layout| {
                    layout.is_css_mode()
                        && layout.propagated_content != Some(layout.propagated_size())
                })
            })
            .unwrap_or(false);
        if content_changed && !changed {
            Self::propagate_size_occurrence(owner);
        }
        if changed {
            Self::propagate_size_occurrence(owner);
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            )
            .add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
        }
        // After the new layout is in place: an artboard places its content
        // from its current size.
        Self::place_css_content_occurrence(owner);
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
            layout.inherited_direction =
                if direction == LayoutDirection::Inherit || layout.has_own_direction() {
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
        for (_, provider) in Self::layout_providers_occurrence(owner) {
            let interpolation = owner
                .with(|object| object.as_layout_component().unwrap().interpolation())
                .unwrap();
            let interpolator = Self::interpolator_occurrence(owner);
            let (time, direction) = owner
                .with(|object| {
                    let layout = object.as_layout_component().unwrap();
                    (layout.interpolation_time(), layout.actual_direction())
                })
                .unwrap();
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
                for index in roots {
                    let Some(root) = Self::hosted_layout_root(&provider, index) else { continue; };
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
        if flags.0 & AdvanceFlags::NEW_FRAME.0 == 0 {
            return false;
        }
        let animate = flags.0 & (AdvanceFlags::ANIMATE.0 | AdvanceFlags::ADVANCE_NESTED.0) != 0;
        // isCollapsed, then applyInterpolation's opening test, under one loan:
        // nothing runs between them. Only an interpolating layout goes on.
        let interpolating = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                !layout.is_collapsed() && Self::interpolation_pending(layout, animate)
            })
            .unwrap_or(false);
        interpolating && Self::apply_interpolation_occurrence(owner, elapsed, animate)
    }

    /// applyInterpolation's opening test: an animating layout whose current
    /// animation has not reached its layout.
    fn interpolation_pending(layout: &mut Self, animate: bool) -> bool {
        let target = layout.layout;
        let Some(data) = layout.current_animation_data().copied() else {
            return false;
        };
        animate && layout.animates() && layout.style.is_some() && data.to != target
    }

    pub fn apply_interpolation_occurrence(owner: &CoreHandle, elapsed: f32, animate: bool) -> bool {
        let Some(true) = owner.with_mut(|object| {
            Self::interpolation_pending(object.as_layout_component_mut().unwrap(), animate)
        }) else {
            return false;
        };
        let (smoothing, data_a) = owner
            .with(|object| {
                let layout = object.as_layout_component().unwrap();
                (
                    layout.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation),
                    layout.animation.as_ref().unwrap().a,
                )
            })
            .unwrap();
        let factor = |seconds: f32| {
            let time = owner
                .with(|object| object.as_layout_component().unwrap().interpolation_time())
                .unwrap();
            let factor = 1.0_f32.min(if time > 0.0 { seconds / time } else { 1.0 });
            let interpolation = owner
                .with(|object| object.as_layout_component().unwrap().interpolation())
                .unwrap();
            if interpolation == LayoutStyleInterpolation::Linear {
                return factor;
            }
            if Self::interpolator_occurrence(owner).is_none() {
                return factor;
            }
            Self::interpolator_occurrence(owner)
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
                layout.animation.as_mut().unwrap().b.from =
                    layout.animation.as_mut().unwrap().a.interpolate(f);
                if f == 1.0 {
                    layout.animation.as_mut().unwrap().a = layout.animation.as_mut().unwrap().b;
                    layout.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                } else {
                    layout.animation.as_mut().unwrap().a.elapsed_seconds += elapsed;
                }
            });
        }
        let (data, old) = owner
            .with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                (*layout.current_animation_data().unwrap(), layout.layout)
            })
            .unwrap();
        if data.elapsed_seconds
            >= owner
                .with(|object| object.as_layout_component().unwrap().interpolation_time())
                .unwrap()
        {
            if old.width() != data.to.width() || old.height() != data.to.height() {
                owner.with_mut(|object| object.component_add_dirt(ComponentDirt::PATH, false));
            }
            owner.with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                layout.layout = data.to;
                if layout.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
                    layout.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                    layout.animation.as_mut().unwrap().a = layout.animation.as_mut().unwrap().b;
                    layout.animation.as_mut().unwrap().b.elapsed_seconds = 0.0;
                }
                layout.animation.as_mut().unwrap().a.elapsed_seconds = 0.0;
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
                .unwrap()
                .elapsed_seconds += elapsed
        });
        if f != 1.0 {
            if owner.is_type_of(
                crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
            ) {
                Self::mark_layout_node_dirty_occurrence(owner, false);
            }
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
                if let Some(data) = layout.current_animation_data() {
                    layout.layout = data.to;
                }
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
        Self::resolve_interpolator_request(self.interpolator_request())
    }
    pub fn interpolator_occurrence(owner: &CoreHandle) -> Option<CoreHandle> {
        // Script clone initialization can call back into the owner. Snapshot
        // the requested identity before resolving it, with no owner borrow.
        let request =
            owner.with(|object| object.as_layout_component().unwrap().interpolator_request())?;
        Self::resolve_interpolator_request(request)
    }
    fn interpolator_request(&self) -> (Option<CoreHandle>, Option<(CoreHandle, u32)>) {
        match self.animation_style() {
            LayoutAnimationStyle::Inherit => {
                if let Some(inherited) = self
                    .animation
                    .as_ref()
                    .and_then(|animation| animation.inherited_interpolator.clone())
                {
                    (Some(inherited), None)
                } else {
                    self.style_interpolator_request()
                }
            }
            LayoutAnimationStyle::Custom => self.style_interpolator_request(),
            _ => (None, None),
        }
    }
    fn style_interpolator_request(&self) -> (Option<CoreHandle>, Option<(CoreHandle, u32)>) {
        let Some((interpolator, id)) =
            self.with_style(|style| (style.interpolator(), style.base.interpolator_id()))
        else {
            return (None, None);
        };
        let owner = self.artboard_handle().map(|artboard| (artboard, id));
        (interpolator, owner)
    }
    fn resolve_interpolator_request(
        (interpolator, owner): (Option<CoreHandle>, Option<(CoreHandle, u32)>),
    ) -> Option<CoreHandle> {
        let interpolator = interpolator?;
        let scripted_type = crate::source::generated::scripted::scripted_interpolator_base::ScriptedInterpolatorBase::TYPE_KEY;
        if !interpolator.is_type_of(scripted_type) {
            return Some(interpolator);
        }
        let Some((artboard, id)) = owner else {
            return Some(interpolator);
        };
        let source = artboard
            .with_downcast::<Artboard, _>(Artboard::artboard_source_handle)
            .flatten()
            .and_then(|source| {
                source
                    .with_downcast::<Artboard, _>(|source| source.resolve_handle(id))
                    .flatten()
            });
        let Some(source) = source.filter(|source| source.is_type_of(scripted_type)) else {
            return Some(interpolator);
        };
        Artboard::stateful_interpolator_handle(&artboard, &source).or(Some(interpolator))
    }
    pub fn interpolation(&self) -> LayoutStyleInterpolation {
        self.with_style(|style| match style.animation_style() {
            LayoutAnimationStyle::Inherit => self
                .animation
                .as_ref()
                .map_or(LayoutStyleInterpolation::Hold, |animation| {
                    animation.inherited_interpolation
                }),
            LayoutAnimationStyle::Custom => style.interpolation(),
            _ => LayoutStyleInterpolation::Hold,
        })
        .unwrap_or(LayoutStyleInterpolation::Hold)
    }
    pub fn interpolation_time(&self) -> f32 {
        self.with_style(|style| match style.animation_style() {
            LayoutAnimationStyle::Inherit => self
                .animation
                .as_ref()
                .map_or(0.0, |animation| animation.inherited_interpolation_time),
            LayoutAnimationStyle::Custom => style.base.interpolation_time(),
            _ => 0.0,
        })
        .unwrap_or(0.0)
    }
    fn current_animation_data(&mut self) -> Option<&mut LayoutAnimationData> {
        let smoothing = self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation);
        let animation = self.animation.as_mut()?;
        Some(if smoothing {
            &mut animation.b
        } else {
            &mut animation.a
        })
    }
    fn ensure_animation(&mut self) -> &mut LayoutAnimation {
        self.animation.get_or_insert_with(|| {
            let data = LayoutAnimationData {
                from: self.layout,
                to: self.layout,
                elapsed_seconds: 0.0,
            };
            Box::new(LayoutAnimation {
                a: data,
                b: data,
                inherited_interpolator: None,
                inherited_interpolation: LayoutStyleInterpolation::Hold,
                inherited_interpolation_time: 0.0,
            })
        })
    }
    pub fn apply_interpolation(&mut self, elapsed: f32, animate: bool) -> bool {
        let target = self.layout;
        let Some(data) = self.current_animation_data().copied() else {
            return false;
        };
        if !animate || !self.animates() || self.style.is_none() || data.to == target {
            return false;
        }
        let transform_factor = |layout: &Self, seconds: f32| {
            let time = layout.interpolation_time();
            let factor = 1.0_f32.min(if time > 0.0 { seconds / time } else { 1.0 });
            if layout.interpolation() == LayoutStyleInterpolation::Linear {
                return factor;
            }
            if layout.interpolator().is_none() {
                return factor;
            }
            layout
                .interpolator()
                .and_then(|interpolator| {
                    interpolator
                        .with_mut(|object| object.keyframe_interpolator_transform(factor))
                        .flatten()
                })
                .unwrap_or(factor)
        };
        if self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
            let factor = transform_factor(self, self.animation.as_ref().unwrap().a.elapsed_seconds);
            self.animation.as_mut().unwrap().b.from =
                self.animation.as_mut().unwrap().a.interpolate(factor);
            if factor == 1.0 {
                self.animation.as_mut().unwrap().a = self.animation.as_mut().unwrap().b;
                self.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
            } else {
                self.animation.as_mut().unwrap().a.elapsed_seconds += elapsed;
            }
        }
        let data = *self.current_animation_data().unwrap();
        if data.elapsed_seconds >= self.interpolation_time() {
            if self.layout.width() != data.to.width() || self.layout.height() != data.to.height() {
                CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
            }
            self.layout = data.to;
            if self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
                self.set_layout_flag(LayoutComponentFlags::IsSmoothingAnimation, false);
                self.animation.as_mut().unwrap().a = self.animation.as_mut().unwrap().b;
                self.animation.as_mut().unwrap().b.elapsed_seconds = 0.0;
            }
            self.animation.as_mut().unwrap().a.elapsed_seconds = 0.0;
            self.propagate_size();
            CoreCapabilities::world_transform_mark_dirty(self);
            return false;
        }
        let factor = transform_factor(self, data.elapsed_seconds);
        let current = self.current_animation_data().unwrap().interpolate(factor);
        if self.layout != current {
            let resized =
                self.layout.width() != current.width() || self.layout.height() != current.height();
            self.layout = current;
            if resized {
                self.propagate_size();
            }
            CoreCapabilities::world_transform_mark_dirty(self);
        }
        self.current_animation_data().unwrap().elapsed_seconds += elapsed;
        if factor != 1.0 {
            if self.is_artboard() {
                self.mark_layout_node_dirty(false);
            }
            true
        } else {
            false
        }
    }
    pub fn quiet_state(&self) -> crate::source::advancing_component::QuietState {
        use crate::source::advancing_component::QuietState;
        let Some(animation) = &self.animation else {
            return QuietState::Quiet;
        };
        let data = if self.has_layout_flag(LayoutComponentFlags::IsSmoothingAnimation) {
            &animation.b
        } else {
            &animation.a
        };
        if self.is_collapsed() || !self.animates() || self.style.is_none() || data.to == self.layout
        {
            QuietState::Quiet
        } else {
            QuietState::Busy
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
            if let Some(data) = self.current_animation_data() {
                self.layout = data.to;
            }
            self.propagate_size();
        }
    }
    /// Only this box's layout data, for an owner that marks its artboard
    /// itself (an artboard marking its own layout).
    pub(crate) fn mark_layout_data_dirty(&mut self) {
        self.layout_data.mark_dirty();
    }
    pub fn mark_layout_node_dirty(&mut self, force: bool) {
        if force {
            self.set_layout_flag(LayoutComponentFlags::ForceUpdateLayoutBounds, true);
        }
        self.layout_data.mark_dirty();
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
        if self.animation.is_none() {
            if interpolation == LayoutStyleInterpolation::Hold
                && interpolator.is_none()
                && time == 0.0
            {
                return false;
            }
            self.ensure_animation();
        }
        let animation = self.animation.as_mut().unwrap();
        if interpolation == animation.inherited_interpolation
            && interpolator == animation.inherited_interpolator
            && time == animation.inherited_interpolation_time
        {
            return false;
        }
        animation.inherited_interpolation = interpolation;
        animation.inherited_interpolator = interpolator;
        animation.inherited_interpolation_time = time;
        true
    }
    pub fn clear_inherited_interpolation(&mut self) {
        let Some(animation) = self.animation.as_mut() else {
            return;
        };
        animation.inherited_interpolation = LayoutStyleInterpolation::Hold;
        animation.inherited_interpolator = None;
        animation.inherited_interpolation_time = 0.0;
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
        self.inherited_direction =
            if direction == LayoutDirection::Inherit || self.has_own_direction() {
                LayoutDirection::Inherit
            } else {
                direction
            };
        if old != self.inherited_direction {
            self.mark_layout_node_dirty(true);
            CoreCapabilities::component_add_dirt(self, ComponentDirt::PATH, false);
            updated = true;
        }
        for (_, provider) in Self::layout_providers_children(self.base.children(), false) {
            let (interpolation, interpolator, time, direction) = (
                self.interpolation(),
                self.interpolator(),
                self.interpolation_time(),
                self.actual_direction(),
            );
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
                for index in roots {
                    let Some(root) = Self::hosted_layout_root(&provider, index) else { continue; };
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
        let css = owner
            .with(|object| {
                object
                    .as_layout_component()
                    .is_some_and(LayoutComponent::is_css_mode)
            })
            .unwrap_or(false);
        if css {
            return;
        }
        if let Some(display_hidden) = Self::with_callback_style(owner, active_style, |style| {
            style.display() == YGDisplay::None
        }) {
            owner.with_mut(|object| {
                object
                    .as_layout_component_mut()
                    .expect("layout component")
                    .set_layout_flag(LayoutComponentFlags::StyleDisplayHidden, display_hidden);
            });
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
                Some(active_style),
                None,
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
    pub fn world_path(&mut self) -> Option<std::cell::RefMut<'_, ShapePaintPath>> {
        self.render_paths
            .as_ref()
            .map(|paths| std::cell::RefMut::map(paths.borrow_mut(), |paths| &mut paths.world))
    }
    /// The path this box clips its children to, in world space: Rive's box
    /// path, or in CSS mode the clip its overflow implies.
    pub fn clip_world_path(&mut self) -> Option<std::cell::RefMut<'_, ShapePaintPath>> {
        let css = self.is_css_mode();
        self.render_paths.as_ref().map(|paths| {
            std::cell::RefMut::map(paths.borrow_mut(), |paths| {
                if css {
                    &mut paths.clip
                } else {
                    &mut paths.world
                }
            })
        })
    }
    /// The rectangle this box clips its children to, in its own space: its
    /// bounds, or in CSS mode its padding box on the clipped axes.
    pub fn clip_local_bounds(&self) -> Aabb {
        let Some(geometry) = self.css_box_geometry() else {
            return self.local_bounds();
        };
        let (x, y) = self.clipped_axes();
        let padding = geometry.padding;
        let far = 1.0e7;
        Aabb::new(
            if x { padding.left() } else { -far },
            if y { padding.top() } else { -far },
            if x { padding.right() } else { far },
            if y { padding.bottom() } else { far },
        )
    }
    pub fn local_path(&mut self) -> Option<std::cell::RefMut<'_, ShapePaintPath>> {
        self.render_paths
            .as_ref()
            .map(|paths| std::cell::RefMut::map(paths.borrow_mut(), |paths| &mut paths.local))
    }
    pub fn local_clockwise_path(&mut self) -> Option<std::cell::RefMut<'_, ShapePaintPath>> {
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
    pub fn apply_container_style(&self, style: &mut Style, context: &LayoutSyncContext) {
        use crate::source::{
            constraints::{
                draggable_constraint::DraggableConstraintDirection,
                scrolling::scroll_constraint::ScrollConstraint,
            },
            layout::layout_style_applier::YGOverflow,
        };
        use taffy::Overflow;
        // Drawing clip only cuts paint. CSS overflow:clip keeps the content
        // minimum, while authored hidden/scroll establish a scroll container.
        let overflow = match self
            .with_style(LayoutComponentStyle::overflow)
            .unwrap_or_default()
        {
            YGOverflow::Hidden => Overflow::Hidden,
            YGOverflow::Scroll => Overflow::Scroll,
            YGOverflow::Visible if self.base.clip() => Overflow::Clip,
            YGOverflow::Visible => Overflow::Visible,
        };
        style.overflow = taffy::geometry::Point {
            x: overflow,
            y: overflow,
        };
        // A ScrollConstraint is parented to content; content's parent is its
        // viewport. Infer scrollability from that same relationship as Rive.
        for content in self.children() {
            let directions = content
                .with(|object| {
                    object.as_layout_component().map(|content| {
                        content
                            .children()
                            .iter()
                            .filter_map(|child| {
                                child.with_downcast::<ScrollConstraint, _>(
                                    ScrollConstraint::direction,
                                )
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .flatten()
                .unwrap_or_default();
            for direction in directions {
                if direction != DraggableConstraintDirection::Vertical {
                    style.overflow.x = Overflow::Scroll;
                }
                if direction != DraggableConstraintDirection::Horizontal {
                    style.overflow.y = Overflow::Scroll;
                }
            }
        }
        // CSS computes visible to auto and clip to hidden when the other axis
        // scrolls. Taffy represents auto's layout semantics as Scroll; no
        // scrollbar space is reserved and Rive still owns drawing and input.
        if style.overflow.x == Overflow::Scroll || style.overflow.y == Overflow::Scroll {
            for axis in [&mut style.overflow.x, &mut style.overflow.y] {
                *axis = match *axis {
                    Overflow::Visible => Overflow::Scroll,
                    Overflow::Clip => Overflow::Hidden,
                    other => other,
                };
            }
        }
        // A CSS box reads its GridTracks whatever the Rive layout type.
        let justify = self.with_style(|component_style| {
            (context.css || !component_style.is_stack())
                .then_some(component_style.base.justify_items_value())
        });
        if let Some(Some(justify)) = justify {
            crate::mechanical_port::source::layout::grid_track::GridTrack::sync_container_style(
                style,
                self,
                u32::from(justify),
            );
        }
    }
    pub fn apply_base_style(&self, style: &mut Style, context: &LayoutSyncContext) {
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
            hug_unbounded,
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
                component_style.base.hug_unbounded(),
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
                width_scale = self.effective_width_scale_type();
            }
            if self.height_unit_value_override != -1 {
                height_units = YGUnit::from(self.height_unit_value_override as u32);
                height_scale = self.effective_height_scale_type();
            }
        }
        // A content-sized axis that does not fill hugs its content. For an
        // absolutely positioned box CSS `auto` already is that hug: shrink to
        // fit, or stretch between two authored insets.
        let size = |forced: f32, value: f32, units: YGUnit, scale: LayoutScaleType| {
            if !forced.is_nan() {
                Dimension::length(forced.max(0.0))
            } else if units == YGUnit::Auto && scale != LayoutScaleType::Fill {
                if absolute && !hug_unbounded {
                    Dimension::auto()
                } else {
                    rive_hug_dimension(hug_unbounded)
                }
            } else {
                rive_dimension(value.max(0.0), units)
            }
        };
        style.size.width = size(self.forced_width, width, width_units, width_scale);
        style.size.height = size(self.forced_height, height, height_units, height_scale);
        let grow = |fraction: f32| if fraction.is_nan() { 0.0 } else { fraction };
        if context.parent_is_grid {
            style.flex_grow = 0.0;
            style.flex_shrink = 0.0;
            style.align_self = rive_align_self(if height_scale == LayoutScaleType::Fill {
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
            // Fill is CSS `flex: <fraction> 1 0%`: siblings share the line by
            // their fractions, whatever their content (docs/CSS_LAYOUT.md,
            // 2.2). As in Rive, only a Fill reads the authored flexBasis (the
            // editor writes 0 points on every style): points or percent
            // replace its 0% basis. Fixed and Hug keep an automatic basis.
            let fill = main_scale == LayoutScaleType::Fill;
            let basis = rive_dimension(flex_basis, flex_basis_units);
            style.flex_grow = if fill { grow(fraction) } else { 0.0 };
            style.flex_shrink = 1.0;
            style.flex_basis = if !fill {
                Dimension::auto()
            } else if basis.is_auto() {
                Dimension::percent(0.0)
            } else {
                basis
            };
            let cross_scale = if context.parent_is_row {
                height_scale
            } else {
                width_scale
            };
            style.align_self = rive_align_self(if cross_scale == LayoutScaleType::Fill {
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
    fn quiet_state(&self) -> crate::source::advancing_component::QuietState {
        LayoutComponent::quiet_state(self)
    }
    fn advance_component(&mut self, elapsed: f32, flags: AdvanceFlags) -> bool {
        LayoutComponent::advance_component(self, elapsed, flags)
    }
}
impl LayoutStyleApplier for LayoutComponent {
    fn apply_base_style(&self, style: &mut Style, context: &LayoutSyncContext) {
        LayoutComponent::apply_base_style(self, style, context);
    }

    fn apply_container_style(&self, style: &mut Style, context: &LayoutSyncContext) {
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
        self.virtual_grid.take();
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
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[cfg(test)]
mod packed_layout_tests {
    use super::*;

    #[test]
    fn registry_parametric_hole_change_invalidates_ancestor_layout() {
        use crate::mechanical_port::source::{
            core::{CoreArena, CoreHandle},
            core_context::CoreContext,
            generated::{core_registry::CoreRegistry, shapes::path_base::PathBase},
            shapes::rectangle::Rectangle,
        };
        struct Context<'a> {
            arena: &'a CoreArena,
            parent: CoreHandle,
        }
        impl CoreContext for Context<'_> {
            fn core_arena(&self) -> &CoreArena {
                self.arena
            }
            fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
                (id == 1).then(|| self.parent.clone())
            }
        }
        let arena = CoreArena::default();
        let layout = arena.insert(LayoutComponent::default());
        let path = arena.insert(Rectangle::default());
        path.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .base
                .set_parent_id_value(1);
        });
        let mut context = Context {
            arena: &arena,
            parent: layout.clone(),
        };
        assert_eq!(
            path.with_mut(|owner| owner
                .as_component_mut()
                .unwrap()
                .on_added_dirty(&mut context)),
            Some(StatusCode::Ok)
        );
        layout.with_downcast_mut::<LayoutComponent, _>(|owner| {
            owner.layout_data.clear_dirty_for_test()
        });
        path.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });

        assert!(CoreRegistry::set_bool_handle(
            &path,
            i32::from(PathBase::IS_HOLE_PROPERTY_KEY),
            true
        ));
        assert!(
            layout
                .with_downcast::<LayoutComponent, _>(|owner| owner.layout_data.is_dirty())
                .unwrap()
        );
        assert!(
            path.with(|owner| owner.as_component().unwrap().has_dirt(ComponentDirt::PATH))
                .unwrap()
        );
        layout.with_downcast_mut::<LayoutComponent, _>(|owner| {
            owner.layout_data.clear_dirty_for_test()
        });
        assert!(CoreRegistry::set_bool_handle(
            &path,
            i32::from(PathBase::IS_HOLE_PROPERTY_KEY),
            true
        ));
        assert!(
            !layout
                .with_downcast::<LayoutComponent, _>(|owner| owner.layout_data.is_dirty())
                .unwrap()
        );
    }

    #[test]
    fn clip_change_dirties_layout_path_without_invalidating_dependent_text() {
        use crate::mechanical_port::source::{
            core::CoreArena, generated::core_registry::CoreRegistry, text::text::Text,
        };

        for occurrence_dispatch in [false, true] {
            let arena = CoreArena::default();
            let layout = arena.insert(LayoutComponent::default());
            let text = arena.insert(Text::default());
            layout.with_mut(|owner| {
                owner.component_add_dependent(text.clone());
                owner
                    .as_component_mut()
                    .unwrap()
                    .set_dirt(ComponentDirt::NONE);
                owner
                    .as_layout_component_mut()
                    .unwrap()
                    .layout_data
                    .clear_dirty_for_test();
            });
            text.with_mut(|owner| {
                owner
                    .as_component_mut()
                    .unwrap()
                    .set_dirt(ComponentDirt::NONE);
            });

            if occurrence_dispatch {
                assert!(CoreRegistry::set_bool_handle(
                    &layout,
                    i32::from(LayoutComponentBase::CLIP_PROPERTY_KEY),
                    true,
                ));
            } else {
                layout.with_downcast_mut::<LayoutComponent, _>(|owner| owner.set_clip(true));
            }

            assert!(
                layout
                    .with(|owner| owner.as_component().unwrap().has_dirt(ComponentDirt::PATH))
                    .unwrap()
            );
            assert!(
                !text
                    .with(|owner| owner.as_component().unwrap().has_dirt(ComponentDirt::PATH))
                    .unwrap()
            );
            assert!(
                layout
                    .with_downcast::<LayoutComponent, _>(|owner| owner.layout_data.is_dirty())
                    .unwrap()
            );
        }
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
            layout.layout_flags.get(),
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

#[cfg(test)]
#[path = "layout_fit_content_tests.rs"]
mod layout_fit_content_tests;

#[cfg(test)]
#[path = "layout_measured_content_tests.rs"]
mod layout_measured_content_tests;

#[cfg(test)]
#[path = "layout_css_mode_tests.rs"]
mod layout_css_mode_tests;
