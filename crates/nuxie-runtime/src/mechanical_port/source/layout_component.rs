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
    pub(crate) just_added_to_host: bool,
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
    has_component_origin: bool,
    // Files exported before 7.3 never composed a layout's own rotation/scale,
    // so any stored value was ignored. Import clears this for those files; it
    // defaults to the current behavior so a layout built outside of import
    // isn't stuck on the legacy path. See File::MINOR_VERSION.
    compose_transform: bool,
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
            has_component_origin: false,
            compose_transform: true,
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
    pub(crate) fn set_compose_transform_from_import(&mut self, import_stack: &ImportStack) {
        // Files exported before 7.3 composed a layout's transform from the solved
        // slot alone, so any stored rotation/scale was written but never applied.
        // Keep that legacy behavior for those files; newer files compose it on top
        // of the slot. See File::MINOR_VERSION.
        let major = import_stack.major_version();
        let minor = import_stack.minor_version();
        self.compose_transform = major > 7 || (major == 7 && minor >= 3);
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
        twin.compose_transform = self.compose_transform;
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
                        drawable.base.drawable_flags() & u32::from(crate::mechanical_port::source::drawable_flag::DrawableFlag::PARTICIPATES_IN_LAYOUT.0) != 0
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
                layout.force_update_layout_bounds |= force;
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
                .parent_is_row = row;
        });
        Self::mark_layout_node_dirty_with_host_occurrence(owner, false, Some(host));
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
        // Keep C++'s three sweeps and applier order. In particular the first
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
        if !self.has_component_origin {
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
        self.has_component_origin = true;
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
        if !self.has_component_origin || self.is_artboard() {
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
        self.compose_transform && !self.is_artboard()
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
        if self.compose_transform
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
            layout.width_unit_value_override = unit;
            layout.parent_is_row = row;
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
            layout.height_unit_value_override = unit;
            layout.parent_is_row = row;
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
            layout.width_intrinsically_size_override = intrinsic;
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
            layout.height_intrinsically_size_override = intrinsic;
            layout.height_unit_value_override = if intrinsic { 3 } else { 1 };
        });
        Self::mark_layout_node_dirty_with_host_occurrence(owner, false, host);
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
                false,
                false,
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
                self.parent_is_row
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
        for pass in 0..3 {
            for applier in &appliers {
                let mut apply = |applier: &dyn LayoutStyleApplier| match pass {
                    0 => applier.apply_base_style(&mut taffy_style, &context),
                    1 => applier.apply_container_style(&mut taffy_style, &context),
                    _ => applier.apply_item_style(&mut taffy_style, &context),
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
                        || child.core_type() == crate::mechanical_port::source::generated::node_base::NodeBase::TYPE_KEY
                        || child.as_any().is::<crate::mechanical_port::source::solo::Solo>()
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
            LayoutTreeCache { tree, nodes, root }
        }

        fn axis(known: Option<f32>, available: AvailableSpace) -> (f32, LayoutMeasureMode) {
            if let Some(value) = known {
                return (value, LayoutMeasureMode::Exactly);
            }
            match available {
                AvailableSpace::Definite(value) => (value, LayoutMeasureMode::AtMost),
                AvailableSpace::MinContent | AvailableSpace::MaxContent => {
                    (f32::NAN, LayoutMeasureMode::Undefined)
                }
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
        cache
            .tree
            .compute_layout_with_measure(
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
        )
        .expect("valid native layout calculation");

        let mut subtree_dirty = dirty.clone();
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
            outputs.push((entry.owner.clone(), next, padding, subtree_dirty[index]));
        }
        owner.with_mut(|object| {
            object
                .as_layout_component_mut()
                .expect("layout calculation owner")
                .layout_tree_cache = Some(cache);
        });
        for (owner, next, padding, subtree_dirty) in outputs {
            owner.with_mut(|object| {
                let data = if let Some(layout) = object.as_layout_component_mut() {
                    layout.layout_data.has_new_layout |= layout.solved_padding != padding;
                    layout.solved_padding = padding;
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
                if data.elapsed_seconds != 0.0 {
                    if self.is_smoothing_animation {
                        self.animation_data_a = self.animation_data_b;
                    }
                    self.is_smoothing_animation = true;
                } else {
                    self.is_smoothing_animation = false;
                }
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
                (
                    layout.layout_data.solved_layout,
                    layout.layout,
                    layout.just_added_to_host,
                    layout.animates(),
                    layout.force_update_layout_bounds,
                    *layout.current_animation_data(),
                )
            })
            .unwrap();
        let mut changed = false;
        if just_added {
            owner.with_mut(|object| {
                let layout = object.as_layout_component_mut().unwrap();
                layout.just_added_to_host = false;
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
                        if layout.is_smoothing_animation {
                            layout.animation_data_a = layout.animation_data_b;
                        }
                        layout.is_smoothing_animation = true;
                    } else {
                        layout.is_smoothing_animation = false;
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
                .force_update_layout_bounds = false
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
                    layout.is_smoothing_animation,
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
                    layout.is_smoothing_animation = false;
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
                if layout.is_smoothing_animation {
                    layout.is_smoothing_animation = false;
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
        if self.is_smoothing_animation {
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
        if self.is_smoothing_animation {
            let factor = transform_factor(
                self.animation_data_a.elapsed_seconds,
                self.interpolation(),
                self.interpolator(),
            );
            self.animation_data_b.from = self.animation_data_a.interpolate(factor);
            if factor == 1.0 {
                self.animation_data_a = self.animation_data_b;
                self.is_smoothing_animation = false;
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
            if self.is_smoothing_animation {
                self.is_smoothing_animation = false;
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
        self.force_update_layout_bounds |= force;
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
                        (layout.position_left_changed, layout.layout.left())
                    })
                    .expect("live style layout owner");
                if !left_changed {
                    style.set_position_left(left);
                }
                let (top_changed, top) = owner
                    .with(|object| {
                        let layout = object.as_layout_component().expect("style layout owner");
                        (layout.position_top_changed, layout.layout.top())
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
                height_units = YGUnit::from(self.height_unit_value_override as u32);
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
