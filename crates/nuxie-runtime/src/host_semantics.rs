//! Typed host observations over settled translated artboard occurrences.
//!
//! This module deliberately retains no parallel scene graph. Every read walks
//! the exact occurrence handles owned by the translated runtime, and every
//! write goes through the generated property callback path of that owner.

use std::{collections::HashSet, ops::Range};

use nuxie_render_api::{Aabb, Mat2D, Vec2D};

use crate::{
    ArtboardInstance, RuntimeGeometryHitOccurrence, RuntimeGeometryHitPathSegment,
    StateMachineEventContext, StateMachineInputKind,
    mechanical_port::source::{
        animation::{
            nested_state_machine::NestedStateMachine,
            state_machine_instance::RuntimeStateMachineInstanceHandle,
        },
        artboard::{Artboard, RuntimeArtboardInstanceHandle, RuntimeArtboardInstanceWeakHandle},
        artboard_component_list::ArtboardComponentList,
        assets::image_asset::ImageAsset,
        constraints::scrolling::{scroll_constraint::ScrollConstraint, scroll_physics},
        core::CoreHandle,
        drawable::{Drawable, RuntimeDrawableOccurrence},
        generated::{
            assets::drawable_asset_base::DrawableAssetBase,
            core_registry::{CoreCapabilities, CoreRegistry},
        },
        math::{
            aabb::Aabb as NativeAabb, mat2d::Mat2D as NativeMat2D, vec2d::Vec2D as NativeVec2D,
        },
        text::{
            cursor::{Cursor, CursorPosition},
            fully_shaped_text::FullyShapedText,
            text::{StyledText, Text},
        },
        text_engine::TextOverflow,
        viewmodel::viewmodel_instance_boolean::ViewModelInstanceBoolean,
    },
};

/// One exact descent edge from a root artboard to a mounted child occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeArtboardOccurrenceSegment {
    NestedArtboard {
        host_local_id: usize,
    },
    ComponentListItem {
        host_local_id: usize,
        item_index: usize,
        /// Stable identity of the retained list item handle at observation time.
        occurrence_identity: u64,
    },
}

/// Opaque equality token for settled semantic geometry in one occurrence.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SemanticGeometryRevision {
    occurrence_identity: u64,
    fingerprint: u64,
}

impl std::fmt::Debug for SemanticGeometryRevision {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SemanticGeometryRevision(..)")
    }
}

/// Solved border box retained by one translated `LayoutComponent`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeLayoutBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// One currently retained child Artboard occurrence owned by an exact
/// `NestedArtboard` or `ArtboardComponentList` host.
///
/// Cloning this value copies only weak occurrence identity and mount fences;
/// it never extends the translated host's child lifetime, instantiates a
/// fallback child, or copies scene state. Every observation and write
/// revalidates the complete translated host path, so a detached nested child
/// or a virtualized list row reused for another item fails closed.
#[derive(Clone)]
pub struct RuntimeNestedArtboardOccurrence {
    native: RuntimeArtboardInstanceWeakHandle,
    instance_identity: u64,
    mounts: Vec<RuntimeNestedArtboardMount>,
}

#[derive(Clone)]
struct RuntimeNestedArtboardMount {
    parent: RuntimeArtboardInstanceWeakHandle,
    host: CoreHandle,
    list_item: Option<CoreHandle>,
    child: RuntimeArtboardInstanceWeakHandle,
}

impl RuntimeNestedArtboardMount {
    fn is_current(&self) -> bool {
        let Some(parent) = self.parent.upgrade() else {
            return false;
        };
        let host_is_mounted = parent.with_artboard(|parent| {
            parent
                .base
                .objects()
                .iter()
                .flatten()
                .any(|object| object == &self.host)
        });
        if !host_is_mounted {
            return false;
        }
        let current = if let Some(expected_item) = self.list_item.as_ref() {
            self.host
                .with_downcast::<ArtboardComponentList, _>(|list| {
                    (0..list.artboard_count()).find_map(|index| {
                        (list.list_item(index as i32).as_ref() == Some(expected_item))
                            .then(|| list.artboard_instance(index as i32))
                            .flatten()
                    })
                })
                .flatten()
        } else {
            self.host
                .with(|object| object.as_nested_artboard()?.artboard_instance_default())
                .flatten()
        };
        current.is_some_and(|current| current.downgrade().ptr_eq(&self.child))
    }
}

impl std::fmt::Debug for RuntimeNestedArtboardOccurrence {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeNestedArtboardOccurrence")
            .field("instance_identity", &self.instance_identity())
            .field("artboard_dimensions", &self.artboard_dimensions())
            .finish()
    }
}

impl RuntimeNestedArtboardOccurrence {
    fn from_native(
        native: RuntimeArtboardInstanceHandle,
        mounts: Vec<RuntimeNestedArtboardMount>,
    ) -> Self {
        Self {
            instance_identity: retained_occurrence_identity(&native.core_handle()),
            native: native.downgrade(),
            mounts,
        }
    }

    /// Whether this exact child is still mounted at every retained host edge.
    pub fn is_current(&self) -> bool {
        self.native.upgrade().is_some()
            && self
                .mounts
                .iter()
                .all(RuntimeNestedArtboardMount::is_current)
    }

    fn current_native(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.is_current().then(|| self.native.upgrade()).flatten()
    }

    fn object_handle(&self, local_id: usize) -> Option<CoreHandle> {
        let native = self.current_native()?;
        let local_id = u32::try_from(local_id).ok()?;
        native.with_artboard(|artboard| artboard.base.resolve_handle(local_id))
    }

    /// Opaque identity of this exact retained occurrence.
    pub fn instance_identity(&self) -> u64 {
        self.instance_identity
    }

    pub fn artboard_dimensions(&self) -> Option<(f32, f32)> {
        let native = self.current_native()?;
        Some(native.with_artboard(|artboard| {
            (artboard.base.layout_width(), artboard.base.layout_height())
        }))
    }

    /// Set the translated Artboard's width and height through its generated
    /// callbacks. The result reports whether either exact value changed.
    pub fn set_artboard_dimensions(&mut self, width: f32, height: f32) -> bool {
        let Some(native) = self.current_native() else {
            return false;
        };
        let (current_width, current_height) =
            native.with_artboard(|artboard| (artboard.base.width(), artboard.base.height()));
        let width_changed = current_width != width;
        let height_changed = current_height != height;
        // `set_size` uses the translated occurrence setter, which releases
        // the Artboard borrow before synchronous root-layout callbacks re-enter
        // this same owner.
        native.set_size(width, height);
        width_changed || height_changed
    }

    pub fn set_double_property(&mut self, local_id: usize, key: u16, value: f32) -> bool {
        let Some(object) = self.object_handle(local_id) else {
            return false;
        };
        let supported = object
            .with(|object| CoreRegistry::object_supports_property(object, u32::from(key)))
            .unwrap_or(false);
        if !supported {
            return false;
        }
        let Some(before) = CoreRegistry::get_double_handle(&object, i32::from(key)) else {
            return false;
        };
        if before == value || !CoreRegistry::set_double_handle(&object, i32::from(key), value) {
            return false;
        }
        CoreRegistry::get_double_handle(&object, i32::from(key))
            .is_some_and(|after| after != before)
    }

    pub fn set_color_property(&mut self, local_id: usize, key: u16, value: u32) -> bool {
        let Some(object) = self.object_handle(local_id) else {
            return false;
        };
        let supported = object
            .with(|object| CoreRegistry::object_supports_property(object, u32::from(key)))
            .unwrap_or(false);
        if !supported {
            return false;
        }
        let value = value as i32;
        let Some(before) = CoreRegistry::get_color_handle(&object, i32::from(key)) else {
            return false;
        };
        if before == value || !CoreRegistry::set_color_handle(&object, i32::from(key), value) {
            return false;
        }
        CoreRegistry::get_color_handle(&object, i32::from(key)).is_some_and(|after| after != before)
    }

    pub fn update_components(&mut self) -> bool {
        self.current_native()
            .is_some_and(|native| native.update_pass(true))
    }
}

/// World-space endpoints of one caret in the exact settled Text occurrence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaretGeometry {
    pub top: Vec2D,
    pub bottom: Vec2D,
}

/// Read-only state of one exact translated `ScrollConstraint` occurrence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeScrollConstraintSnapshot {
    pub constraint_local_id: usize,
    pub constraint_authored_id: u32,
    pub content_local_id: usize,
    pub content_authored_id: u32,
    pub offset: (f32, f32),
    pub lower_bound: (f32, f32),
    pub upper_bound: (f32, f32),
    pub clamped_offset: (f32, f32),
    pub physics_present: bool,
    pub physics_running: bool,
    pub velocity: (f32, f32),
    pub scroll_active: bool,
}

/// One exact drawable occurrence and its settled world-space bounds.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeGeometryHit {
    pub path: Vec<RuntimeGeometryHitPathSegment>,
    pub occurrence: Vec<RuntimeGeometryHitOccurrence>,
    pub bounds: Aabb,
    pub text_value: Option<String>,
}

impl StateMachineEventContext {
    /// Preserve the exact rendered occurrence that caused a pointer event.
    pub fn from_geometry_hit(hit: &RuntimeGeometryHit) -> Self {
        Self::new(hit.path.clone(), hit.occurrence.clone())
    }
}

/// One retained `Text` drawable occurrence and its resolved value.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeSemanticTextHit {
    pub path: Vec<RuntimeGeometryHitPathSegment>,
    pub occurrence: Vec<RuntimeGeometryHitOccurrence>,
    pub bounds: Aabb,
    pub value: String,
}

/// A host-supplied intrinsic image size conflicts with the exact asset owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeImageDimensionConflict {
    asset_global: u32,
    expected: (u32, u32),
    actual: (u32, u32),
}

impl std::fmt::Display for RuntimeImageDimensionConflict {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "image asset {} has registered dimensions {}x{} but received {}x{}",
            self.asset_global, self.expected.0, self.expected.1, self.actual.0, self.actual.1,
        )
    }
}

impl std::error::Error for RuntimeImageDimensionConflict {}

fn native_to_render_matrix(value: NativeMat2D) -> Mat2D {
    Mat2D(*value.values())
}

fn native_to_render_bounds(value: NativeAabb) -> Aabb {
    Aabb::new(value.left(), value.top(), value.right(), value.bottom())
}

fn render_to_native_point(value: Vec2D) -> NativeVec2D {
    NativeVec2D::new(value.x, value.y)
}

fn transform_bounds(bounds: NativeAabb, transform: NativeMat2D) -> Aabb {
    native_to_render_bounds(transform.map_bounding_box(bounds))
}

fn semantic_bounds(bounds: NativeAabb, transform: NativeMat2D) -> Option<Aabb> {
    if bounds.is_empty_or_nan() {
        return None;
    }
    let world_bounds = transform.map_bounding_box(bounds);
    if world_bounds.is_empty_or_nan() {
        return None;
    }
    Some(native_to_render_bounds(world_bounds))
}

fn source_artboard_global_id(artboard: &RuntimeArtboardInstanceHandle) -> Option<u32> {
    let source = artboard.with_artboard(|artboard| artboard.base.artboard_source_handle())?;
    u32::try_from(source.identity_key().1).ok()
}

fn source_object_global_id(
    artboard: &RuntimeArtboardInstanceHandle,
    local_id: usize,
) -> Option<u32> {
    let source = artboard.with_artboard(|artboard| artboard.base.artboard_source_handle())?;
    let source_object = source
        .with_downcast::<Artboard, _>(|source| source.objects().get(local_id).cloned())
        .flatten()
        .flatten()?;
    u32::try_from(source_object.identity_key().1).ok()
}

fn local_id_for_handle(
    artboard: &RuntimeArtboardInstanceHandle,
    handle: &CoreHandle,
) -> Option<usize> {
    artboard.with_artboard(|artboard| usize::try_from(artboard.object_index(handle)).ok())
}

fn retained_occurrence_identity(handle: &CoreHandle) -> u64 {
    let (arena, slot, generation) = handle.identity_key();
    let mut value = 0xcbf2_9ce4_8422_2325u64;
    for part in [arena as u64, slot as u64, generation] {
        value ^= part;
        value = value.wrapping_mul(0x100_0000_01b3);
    }
    value
}

fn source_artboard_name(artboard: &RuntimeArtboardInstanceHandle) -> Option<String> {
    let source = artboard.with_artboard(|artboard| artboard.base.artboard_source_handle())?;
    source.with_downcast::<Artboard, _>(|source| source.base.name().to_owned())
}

fn retained_child_artboards(
    artboard: &RuntimeArtboardInstanceHandle,
) -> Vec<RuntimeNestedArtboardMount> {
    // The translated Artboard's private `artboard_hosts` vector is populated
    // while walking this authored object array. Filter that same array instead
    // of concatenating its type-specific caches, which would reorder an
    // authored NestedArtboard/List/NestedArtboard sequence.
    let hosts = artboard.with_artboard(|artboard| artboard.base.objects().to_vec());
    let mut children = Vec::new();
    for host in hosts.into_iter().flatten() {
        if let Some(nested) = host
            .with(|object| object.as_nested_artboard()?.artboard_instance_default())
            .flatten()
        {
            children.push(RuntimeNestedArtboardMount {
                parent: artboard.downgrade(),
                host,
                list_item: None,
                child: nested.downgrade(),
            });
            continue;
        }
        if let Some(retained) = host.with_downcast::<ArtboardComponentList, _>(|list| {
            (0..list.artboard_count())
                .filter_map(|index| {
                    Some((
                        list.list_item(index as i32)?,
                        list.artboard_instance(index as i32)?,
                    ))
                })
                .collect::<Vec<_>>()
        }) {
            children.extend(retained.into_iter().map(|(list_item, child)| {
                RuntimeNestedArtboardMount {
                    parent: artboard.downgrade(),
                    host: host.clone(),
                    list_item: Some(list_item),
                    child: child.downgrade(),
                }
            }));
        }
    }
    children
}

fn collect_nested_artboards_named(
    artboard: &RuntimeArtboardInstanceHandle,
    source_name: &str,
    ancestors: &mut HashSet<(usize, usize, u64)>,
    mounts: &mut Vec<RuntimeNestedArtboardMount>,
    output: &mut Vec<RuntimeNestedArtboardOccurrence>,
) {
    for mount in retained_child_artboards(artboard) {
        let Some(child) = mount.child.upgrade() else {
            continue;
        };
        mounts.push(mount);
        if source_artboard_name(&child).as_deref() == Some(source_name) {
            output.push(RuntimeNestedArtboardOccurrence::from_native(
                child.clone(),
                mounts.clone(),
            ));
        }
        let identity = child.core_handle().identity_key();
        if ancestors.insert(identity) {
            collect_nested_artboards_named(&child, source_name, ancestors, mounts, output);
            ancestors.remove(&identity);
        }
        mounts.pop();
    }
}

fn snapshot_scroll_constraint(
    artboard: &RuntimeArtboardInstanceHandle,
    constraint_local_id: usize,
    handle: &CoreHandle,
) -> Option<RuntimeScrollConstraintSnapshot> {
    handle
        .with_downcast::<ScrollConstraint, _>(|constraint| {
            let content = constraint.content_handle()?;
            let content_local_id = local_id_for_handle(artboard, &content)?;
            let physics = constraint.physics();
            let physics_running = physics.as_ref().is_some_and(|physics| {
                physics
                    .with(|physics| {
                        scroll_physics::from_core(physics).is_some_and(|p| p.is_running())
                    })
                    .unwrap_or(false)
            });
            Some(RuntimeScrollConstraintSnapshot {
                constraint_local_id,
                constraint_authored_id: source_object_global_id(artboard, constraint_local_id)?,
                content_local_id,
                content_authored_id: source_object_global_id(artboard, content_local_id)?,
                offset: (constraint.offset_x(), constraint.offset_y()),
                lower_bound: (constraint.max_offset_x(), constraint.max_offset_y()),
                upper_bound: (constraint.min_offset_x(), constraint.min_offset_y()),
                clamped_offset: (constraint.clamped_offset_x(), constraint.clamped_offset_y()),
                physics_present: physics.is_some(),
                physics_running,
                velocity: (constraint.velocity_x(), constraint.velocity_y()),
                scroll_active: constraint.scroll_active(),
            })
        })
        .flatten()
}

fn scroll_snapshots(
    artboard: &RuntimeArtboardInstanceHandle,
) -> Vec<RuntimeScrollConstraintSnapshot> {
    let objects = artboard.with_artboard(|artboard| artboard.objects().to_vec());
    objects
        .into_iter()
        .enumerate()
        .filter_map(|(local_id, object)| snapshot_scroll_constraint(artboard, local_id, &object?))
        .collect()
}

impl ArtboardInstance {
    /// Opaque identity of this exact retained root occurrence.
    pub fn instance_identity(&self) -> u64 {
        retained_occurrence_identity(&self.native_handle().core_handle())
    }

    /// Visit retained nested and component-list children in translated host
    /// order, depth first, and return every occurrence whose exact source
    /// Artboard has `source_artboard_name`.
    pub fn nested_artboard_occurrences_named(
        &self,
        source_artboard_name: &str,
    ) -> Vec<RuntimeNestedArtboardOccurrence> {
        let root = self.native_handle();
        let mut ancestors = HashSet::from([root.core_handle().identity_key()]);
        let mut mounts = Vec::new();
        let mut output = Vec::new();
        collect_nested_artboards_named(
            &root,
            source_artboard_name,
            &mut ancestors,
            &mut mounts,
            &mut output,
        );
        output
    }

    pub fn scroll_constraint_occurrences(&self) -> Vec<RuntimeScrollConstraintSnapshot> {
        scroll_snapshots(&self.native_handle())
    }

    pub fn scroll_constraint_for_content(
        &self,
        content_local_id: usize,
    ) -> Option<RuntimeScrollConstraintSnapshot> {
        self.scroll_constraint_occurrences()
            .into_iter()
            .find(|snapshot| snapshot.content_local_id == content_local_id)
    }

    pub fn scroll_constraint_for_authored_id(
        &self,
        constraint_authored_id: u32,
    ) -> Option<RuntimeScrollConstraintSnapshot> {
        self.scroll_constraint_occurrences()
            .into_iter()
            .find(|snapshot| snapshot.constraint_authored_id == constraint_authored_id)
    }

    pub fn scroll_constraint_for_content_authored_id(
        &self,
        content_authored_id: u32,
    ) -> Option<RuntimeScrollConstraintSnapshot> {
        self.scroll_constraint_occurrences()
            .into_iter()
            .find(|snapshot| snapshot.content_authored_id == content_authored_id)
    }

    pub fn layout_bounds(&self, local_id: usize) -> Option<RuntimeLayoutBounds> {
        let bounds = self.object_handle(local_id)?.with(|object| {
            object
                .as_layout_component()
                .map(|layout| layout.layout_bounds())
        })??;
        Some(RuntimeLayoutBounds {
            x: bounds.left(),
            y: bounds.top(),
            width: bounds.width(),
            height: bounds.height(),
        })
    }

    pub fn world_transform(&mut self, local_id: usize) -> Option<Mat2D> {
        self.update_components();
        let transform = self.object_handle(local_id)?.with(|object| {
            object
                .as_world_transform_component()
                .map(|component| *component.world_transform())
        })??;
        Some(native_to_render_matrix(transform))
    }

    /// The translated constraint owner applies scroll directly to the retained
    /// world transform, so this is an observation of that same canonical value.
    pub fn world_transform_with_scroll(&mut self, local_id: usize) -> Option<Mat2D> {
        self.world_transform(local_id)
    }

    pub fn world_bounds(&mut self, local_id: usize) -> Option<Aabb> {
        self.update_components();
        let handle = self.object_handle(local_id)?;
        let (bounds, world) = handle.with(|object| {
            Some((
                object.semantic_provider_local_bounds()?,
                *object.as_world_transform_component()?.world_transform(),
            ))
        })??;
        semantic_bounds(bounds, world)
    }

    pub fn scrolled_layout_bounds(&mut self, local_id: usize) -> Option<RuntimeLayoutBounds> {
        self.update_components();
        let layout = self.layout_bounds(local_id)?;
        let queried = self.object_handle(local_id)?;
        let mut child = queried.clone();
        let mut current = queried.with(|object| object.component_parent_handle())?;
        let mut displacement = NativeVec2D::default();
        while let Some(ancestor) = current {
            let child_world = child.with(|object| {
                object
                    .as_world_transform_component()
                    .map(|component| *component.world_transform())
            })??;
            let (constraints, parent) = ancestor.with(|object| {
                let transform = object.as_transform_component()?;
                Some((
                    transform.constraints().to_vec(),
                    object.component_parent_handle(),
                ))
            })??;
            for constraint in constraints {
                let offset = constraint.with_downcast::<ScrollConstraint, _>(|scroll| {
                    NativeVec2D::new(
                        if scroll.base.constrains_horizontal() {
                            scroll.clamped_offset_x()
                        } else {
                            0.0
                        },
                        if scroll.base.constrains_vertical() {
                            scroll.clamped_offset_y()
                        } else {
                            0.0
                        },
                    )
                });
                if let Some(offset) = offset {
                    // `constrainChild` post-multiplies this translation onto
                    // the direct branch child's exact world matrix. Its
                    // translation changes, but its linear part does not.
                    let values = child_world.values();
                    displacement.x += values[0].mul_add(offset.x, values[2] * offset.y);
                    displacement.y += values[1].mul_add(offset.x, values[3] * offset.y);
                }
            }
            child = ancestor;
            current = parent;
        }
        Some(RuntimeLayoutBounds {
            x: layout.x + displacement.x,
            y: layout.y + displacement.y,
            ..layout
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GeometryVisibility {
    Visible,
    Retained,
}

fn drawable_chain(
    artboard: &RuntimeArtboardInstanceHandle,
    visibility: GeometryVisibility,
) -> Vec<RuntimeDrawableOccurrence> {
    if visibility == GeometryVisibility::Retained {
        return artboard
            .with_artboard(|artboard| artboard.objects().to_vec())
            .into_iter()
            .flatten()
            .filter(|handle| {
                handle
                    .with(|object| object.as_drawable().is_some())
                    .unwrap_or(false)
            })
            .map(RuntimeDrawableOccurrence::authored)
            .collect();
    }
    let mut result = Vec::new();
    let mut current = artboard.with_artboard(|artboard| artboard.base.first_drawable());
    while let Some(drawable) = current {
        current = drawable.with(Drawable::prev_drawable).flatten();
        if drawable.will_draw() {
            result.push(drawable);
        }
    }
    result
}

fn authored_geometry_is_catalogued(handle: &CoreHandle, visibility: GeometryVisibility) -> bool {
    handle
        .with(|object| {
            if let Some(text) = object.as_text() {
                if visibility == GeometryVisibility::Retained {
                    return true;
                }
                let render_opacity = text.base.render_opacity();
                return render_opacity.is_finite() && render_opacity > 0.0;
            }
            if object.as_shape().is_none() {
                return false;
            }
            if visibility == GeometryVisibility::Retained {
                return true;
            }
            object.as_shape_paint_container().is_some_and(|container| {
                container.shape_paints().iter().any(|paint| {
                    paint
                        .with(|paint| {
                            paint
                                .as_shape_paint_behavior()
                                .is_some_and(|paint| paint.should_draw())
                        })
                        .unwrap_or(false)
                })
            })
        })
        .unwrap_or(false)
}

fn collect_geometry(
    artboard: &RuntimeArtboardInstanceHandle,
    artboard_to_root: NativeMat2D,
    path_prefix: &[RuntimeGeometryHitPathSegment],
    occurrence: &[RuntimeGeometryHitOccurrence],
    visibility: GeometryVisibility,
    query: Option<Vec2D>,
    output: &mut Vec<RuntimeGeometryHit>,
) {
    let Some(artboard_global_id) = source_artboard_global_id(artboard) else {
        return;
    };
    for drawable in drawable_chain(artboard, visibility) {
        let Some(handle) = drawable.authored_handle() else {
            continue;
        };
        let Some(local_id) = local_id_for_handle(artboard, &handle) else {
            continue;
        };
        let mut path = path_prefix.to_vec();
        path.push(RuntimeGeometryHitPathSegment {
            artboard_global_id,
            local_id,
        });

        if let Some((child, host_transform)) = handle
            .with(|object| {
                let nested = object.as_nested_artboard()?;
                let host = object.as_artboard_host()?;
                Some((
                    nested.artboard_instance_default()?,
                    host.world_transform_for_artboard(artboard.downgrade()),
                ))
            })
            .flatten()
        {
            collect_geometry(
                &child,
                artboard_to_root * host_transform,
                &path,
                occurrence,
                visibility,
                query,
                output,
            );
            continue;
        }

        if handle.is_type_of(ArtboardComponentList::TYPE_KEY) {
            let indices = handle
                .with_downcast_mut::<ArtboardComponentList, _>(|list| {
                    if visibility == GeometryVisibility::Visible {
                        list.ensure_ordered_list_indices();
                        list.ordered_list_indices().to_vec()
                    } else {
                        (0..list.artboard_count())
                            .filter_map(|index| i32::try_from(index).ok())
                            .collect()
                    }
                })
                .unwrap_or_default();
            for item_index in indices {
                let Some((child, item, child_transform)) = handle
                    .with_downcast::<ArtboardComponentList, _>(|list| {
                        let child = list.artboard_instance(item_index)?;
                        let item = list.list_item(item_index)?;
                        let transform = list.draw_transform_for_index(item_index)?;
                        Some((child, item, transform))
                    })
                    .flatten()
                else {
                    continue;
                };
                let Ok(item_index) = usize::try_from(item_index) else {
                    continue;
                };
                let mut child_occurrence = occurrence.to_vec();
                child_occurrence.push(RuntimeGeometryHitOccurrence {
                    artboard_global_id,
                    host_local_id: local_id,
                    item_index,
                    occurrence_identity: retained_occurrence_identity(&item),
                });
                collect_geometry(
                    &child,
                    artboard_to_root * child_transform,
                    &path,
                    &child_occurrence,
                    visibility,
                    query,
                    output,
                );
            }
            continue;
        }

        if !authored_geometry_is_catalogued(&handle, visibility) {
            continue;
        }

        let Some((local_bounds, world)) = handle
            .with(|object| {
                Some((
                    object.semantic_provider_local_bounds()?,
                    *object.as_world_transform_component()?.world_transform(),
                ))
            })
            .flatten()
        else {
            continue;
        };
        let root_transform = artboard_to_root * world;
        let Some(bounds) = semantic_bounds(local_bounds, root_transform) else {
            continue;
        };
        if ![bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
            .into_iter()
            .all(f32::is_finite)
        {
            continue;
        }
        if let Some(point) = query {
            if !bounds.contains(point) {
                continue;
            }
            let mut inverse = NativeMat2D::identity();
            if !artboard_to_root.invert(&mut inverse) {
                continue;
            }
            let local_point = inverse * render_to_native_point(point);
            let hit = handle
                .with_mut(|object| object.component_hit_test_point(&local_point, false, true))
                .flatten()
                .unwrap_or(false);
            if !hit {
                continue;
            }
        }
        output.push(RuntimeGeometryHit {
            path,
            occurrence: occurrence.to_vec(),
            bounds,
            text_value: None,
        });
    }
}

impl ArtboardInstance {
    fn geometry_with_bounds(
        &mut self,
        visibility: GeometryVisibility,
        query: Option<Vec2D>,
    ) -> Vec<RuntimeGeometryHit> {
        self.update_components();
        let mut output = Vec::new();
        collect_geometry(
            &self.native_handle(),
            NativeMat2D::identity(),
            &[],
            &[],
            visibility,
            query,
            &mut output,
        );
        output
    }

    pub fn hit_test_segments_with_bounds(&mut self, point: Vec2D) -> Vec<RuntimeGeometryHit> {
        if !point.x.is_finite() || !point.y.is_finite() {
            return Vec::new();
        }
        let mut hits = self.geometry_with_bounds(GeometryVisibility::Visible, Some(point));
        hits.reverse();
        hits
    }

    pub fn visible_geometry_with_bounds(&mut self) -> Vec<RuntimeGeometryHit> {
        self.geometry_with_bounds(GeometryVisibility::Visible, None)
    }

    pub fn retained_geometry_with_bounds(&mut self) -> Vec<RuntimeGeometryHit> {
        self.geometry_with_bounds(GeometryVisibility::Retained, None)
    }
}

fn collect_semantic_text(
    artboard: &RuntimeArtboardInstanceHandle,
    artboard_to_root: NativeMat2D,
    path_prefix: &[RuntimeGeometryHitPathSegment],
    occurrence: &[RuntimeGeometryHitOccurrence],
    output: &mut Vec<RuntimeSemanticTextHit>,
) {
    let Some(artboard_global_id) = source_artboard_global_id(artboard) else {
        return;
    };
    for drawable in drawable_chain(artboard, GeometryVisibility::Visible) {
        let Some(handle) = drawable.authored_handle() else {
            continue;
        };
        let Some(local_id) = local_id_for_handle(artboard, &handle) else {
            continue;
        };
        let mut path = path_prefix.to_vec();
        path.push(RuntimeGeometryHitPathSegment {
            artboard_global_id,
            local_id,
        });

        if let Some((child, host_transform)) = handle
            .with(|object| {
                let nested = object.as_nested_artboard()?;
                let host = object.as_artboard_host()?;
                Some((
                    nested.artboard_instance_default()?,
                    host.world_transform_for_artboard(artboard.downgrade()),
                ))
            })
            .flatten()
        {
            collect_semantic_text(
                &child,
                artboard_to_root * host_transform,
                &path,
                occurrence,
                output,
            );
            continue;
        }

        if handle.is_type_of(ArtboardComponentList::TYPE_KEY) {
            let indices = handle
                .with_downcast_mut::<ArtboardComponentList, _>(|list| {
                    list.ensure_ordered_list_indices();
                    list.ordered_list_indices().to_vec()
                })
                .unwrap_or_default();
            for item_index in indices {
                let Some((child, item, child_transform)) = handle
                    .with_downcast::<ArtboardComponentList, _>(|list| {
                        Some((
                            list.artboard_instance(item_index)?,
                            list.list_item(item_index)?,
                            list.draw_transform_for_index(item_index)?,
                        ))
                    })
                    .flatten()
                else {
                    continue;
                };
                let Ok(item_index) = usize::try_from(item_index) else {
                    continue;
                };
                let mut child_occurrence = occurrence.to_vec();
                child_occurrence.push(RuntimeGeometryHitOccurrence {
                    artboard_global_id,
                    host_local_id: local_id,
                    item_index,
                    occurrence_identity: retained_occurrence_identity(&item),
                });
                collect_semantic_text(
                    &child,
                    artboard_to_root * child_transform,
                    &path,
                    &child_occurrence,
                    output,
                );
            }
            continue;
        }

        let Some((local_bounds, world, value)) = handle
            .with(|object| {
                let text = object.as_text()?;
                let render_opacity = text.base.render_opacity();
                if !render_opacity.is_finite() || render_opacity <= 0.0 {
                    return None;
                }
                Some((
                    text.local_bounds(),
                    *object.as_world_transform_component()?.world_transform(),
                    text.settled_text_value(),
                ))
            })
            .flatten()
        else {
            continue;
        };
        let bounds = transform_bounds(local_bounds, artboard_to_root * world);
        if ![bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
            .into_iter()
            .all(f32::is_finite)
        {
            continue;
        }
        output.push(RuntimeSemanticTextHit {
            path,
            occurrence: occurrence.to_vec(),
            bounds,
            value,
        });
    }
}

impl ArtboardInstance {
    pub fn semantic_text_with_bounds(&mut self) -> Vec<RuntimeSemanticTextHit> {
        self.update_components();
        let mut output = Vec::new();
        collect_semantic_text(
            &self.native_handle(),
            NativeMat2D::identity(),
            &[],
            &[],
            &mut output,
        );
        output
    }
}

struct TextQuery {
    value: String,
    shape: FullyShapedText,
    world: NativeMat2D,
}

fn build_text_query(handle: &CoreHandle) -> Option<TextQuery> {
    handle
        .with_downcast_mut::<Text, _>(|text| {
            if text.have_modifiers()
                || !matches!(text.overflow(), TextOverflow::Visible | TextOverflow::Fit)
            {
                return None;
            }
            let world = *text.shape_world_transform();
            if !world.values().iter().all(|value| value.is_finite())
                || !text.effective_width().is_finite()
                || !text.effective_height().is_finite()
                || !text.base.paragraph_spacing().is_finite()
            {
                return None;
            }
            let mut styled = StyledText::default();
            if !text.make_styled(&mut styled, true, 1.0) {
                return None;
            }
            let value = styled
                .unichars()
                .iter()
                .map(|value| char::from_u32(*value).unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect::<String>();
            let mut code_points = styled.unichars().to_vec();
            let mut runs = Vec::new();
            styled.swap_runs(&mut runs);
            if runs.is_empty()
                || runs.iter().any(|run| {
                    run.font.is_none()
                        || !run.size.is_finite()
                        || !run.line_height.is_finite()
                        || !run.letter_spacing.is_finite()
                })
            {
                return None;
            }
            let mut shape = FullyShapedText::default();
            shape.shape(
                &mut code_points,
                &mut runs,
                text.effective_sizing(),
                text.effective_width(),
                text.effective_height(),
                text.align(),
                text.wrap(),
                text.text_origin(),
                text.overflow(),
                text.base.paragraph_spacing(),
            );
            if (!value.is_empty() && shape.ordered_lines().is_empty())
                || ![
                    shape.bounds().left(),
                    shape.bounds().top(),
                    shape.bounds().right(),
                    shape.bounds().bottom(),
                ]
                .into_iter()
                .all(f32::is_finite)
            {
                return None;
            }
            Some(TextQuery {
                value,
                shape,
                world,
            })
        })
        .flatten()
}

fn byte_to_code_point(value: &str, byte_offset: usize) -> Option<u32> {
    value.is_char_boundary(byte_offset).then_some(())?;
    u32::try_from(value[..byte_offset].chars().count()).ok()
}

fn code_point_to_byte(value: &str, code_point: u32) -> Option<usize> {
    let code_point = usize::try_from(code_point).ok()?;
    if code_point == value.chars().count() {
        return Some(value.len());
    }
    value
        .char_indices()
        .nth(code_point)
        .map(|(offset, _)| offset)
}

impl ArtboardInstance {
    pub fn text_caret(&mut self, local_id: usize, byte_offset: usize) -> Option<CaretGeometry> {
        self.update_components();
        let query = build_text_query(&self.object_handle(local_id)?)?;
        let code_point = byte_to_code_point(&query.value, byte_offset)?;
        let position = CursorPosition::at_index(code_point, &query.shape);
        let visual = position.visual_position(&query.shape);
        if !visual.found() {
            return None;
        }
        let top = query.world * NativeVec2D::new(visual.x(), visual.top());
        let bottom = query.world * NativeVec2D::new(visual.x(), visual.bottom());
        if ![top.x, top.y, bottom.x, bottom.y]
            .into_iter()
            .all(f32::is_finite)
        {
            return None;
        }
        Some(CaretGeometry {
            top: Vec2D::new(top.x, top.y),
            bottom: Vec2D::new(bottom.x, bottom.y),
        })
    }

    pub fn text_hit(&mut self, local_id: usize, point: Vec2D) -> Option<usize> {
        if !point.x.is_finite() || !point.y.is_finite() {
            return None;
        }
        self.update_components();
        let query = build_text_query(&self.object_handle(local_id)?)?;
        let mut inverse = NativeMat2D::identity();
        if !query.world.invert(&mut inverse) {
            return None;
        }
        let local = inverse * render_to_native_point(point);
        if !local.x.is_finite() || !local.y.is_finite() {
            return None;
        }
        let position = CursorPosition::from_translation(local, &query.shape);
        code_point_to_byte(&query.value, position.code_point_index())
    }

    pub fn text_selection_rects(&mut self, local_id: usize, range: Range<usize>) -> Vec<Aabb> {
        if range.start >= range.end {
            return Vec::new();
        }
        self.update_components();
        let Some(query) = self
            .object_handle(local_id)
            .as_ref()
            .and_then(build_text_query)
        else {
            return Vec::new();
        };
        let Some(start) = byte_to_code_point(&query.value, range.start) else {
            return Vec::new();
        };
        let Some(end) = byte_to_code_point(&query.value, range.end) else {
            return Vec::new();
        };
        let mut cursor = Cursor::new(
            CursorPosition::unresolved(start),
            CursorPosition::unresolved(end),
        );
        cursor.resolve_line_positions(&query.shape);
        let mut rectangles = Vec::new();
        cursor.selection_rects(&mut rectangles, &query.shape);
        let rectangles = rectangles
            .into_iter()
            .map(|bounds| transform_bounds(bounds, query.world))
            .collect::<Vec<_>>();
        rectangles
            .iter()
            .all(|bounds| {
                [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
                    .into_iter()
                    .all(f32::is_finite)
            })
            .then_some(rectangles)
            .unwrap_or_default()
    }
}

fn resolve_occurrence_artboard(
    root: &RuntimeArtboardInstanceHandle,
    occurrence: &[RuntimeArtboardOccurrenceSegment],
) -> Option<RuntimeArtboardInstanceHandle> {
    let mut artboard = root.clone();
    for segment in occurrence {
        artboard = match *segment {
            RuntimeArtboardOccurrenceSegment::NestedArtboard { host_local_id } => artboard
                .with_artboard(|artboard| {
                    artboard
                        .base
                        .resolve_handle(u32::try_from(host_local_id).ok()?)?
                        .with(|object| object.as_nested_artboard()?.artboard_instance_default())
                        .flatten()
                })?,
            RuntimeArtboardOccurrenceSegment::ComponentListItem {
                host_local_id,
                item_index,
                occurrence_identity,
            } => artboard.with_artboard(|artboard| {
                let list = artboard
                    .base
                    .resolve_handle(u32::try_from(host_local_id).ok()?)?;
                let item_index = i32::try_from(item_index).ok()?;
                let (item, child) = list
                    .with_downcast::<ArtboardComponentList, _>(|list| {
                        Some((
                            list.list_item(item_index)?,
                            list.artboard_instance(item_index)?,
                        ))
                    })
                    .flatten()?;
                (retained_occurrence_identity(&item) == occurrence_identity).then_some(child)
            })?,
        };
    }
    (!occurrence.is_empty()).then_some(artboard)
}

fn occurrence_machine(
    root: &RuntimeArtboardInstanceHandle,
    occurrence: &[RuntimeArtboardOccurrenceSegment],
    state_machine_index: usize,
) -> Option<RuntimeStateMachineInstanceHandle> {
    let (last, prefix) = occurrence.split_last()?;
    let parent = if prefix.is_empty() {
        root.clone()
    } else {
        resolve_occurrence_artboard(root, prefix)?
    };
    match *last {
        RuntimeArtboardOccurrenceSegment::NestedArtboard { host_local_id } => {
            parent.with_artboard(|artboard| {
                let nested = artboard
                    .base
                    .resolve_handle(u32::try_from(host_local_id).ok()?)?;
                nested.with(|object| {
                    let nested = object.as_nested_artboard()?;
                    nested.nested_animations().iter().find_map(|animation| {
                        animation
                            .with_downcast::<NestedStateMachine, _>(|machine| {
                                (machine.base.animation_id() as usize == state_machine_index)
                                    .then(|| machine.state_machine_instance())
                                    .flatten()
                            })
                            .flatten()
                    })
                })?
            })
        }
        RuntimeArtboardOccurrenceSegment::ComponentListItem {
            host_local_id,
            item_index,
            occurrence_identity,
        } => parent.with_artboard(|artboard| {
            let list = artboard
                .base
                .resolve_handle(u32::try_from(host_local_id).ok()?)?;
            let item_index = i32::try_from(item_index).ok()?;
            let (item, child, machine) = list
                .with_downcast::<ArtboardComponentList, _>(|list| {
                    Some((
                        list.list_item(item_index)?,
                        list.artboard_instance(item_index)?,
                        list.state_machine_instance(item_index)?,
                    ))
                })
                .flatten()?;
            if retained_occurrence_identity(&item) != occurrence_identity {
                return None;
            }
            let expected =
                child.with_artboard(|child| child.state_machine_handle_at(state_machine_index))?;
            machine
                .with_instance(|instance| instance.state_machine() == expected)
                .then_some(machine)
        }),
    }
}

fn machine_input_kind(
    machine: &RuntimeStateMachineInstanceHandle,
    input_index: usize,
) -> Option<StateMachineInputKind> {
    let input_index = u32::try_from(input_index).ok()?;
    machine.with_instance(|machine| {
        if machine.bool_input(input_index).is_some() {
            Some(StateMachineInputKind::Bool)
        } else if machine.number_input(input_index).is_some() {
            Some(StateMachineInputKind::Number)
        } else if machine.trigger_input(input_index).is_some() {
            Some(StateMachineInputKind::Trigger)
        } else {
            None
        }
    })
}

impl ArtboardInstance {
    pub fn occurrence_state_machine_input(
        &self,
        occurrence: &[RuntimeArtboardOccurrenceSegment],
        state_machine_index: usize,
        name: &str,
    ) -> Option<(usize, StateMachineInputKind)> {
        let machine = occurrence_machine(&self.native_handle(), occurrence, state_machine_index)?;
        let input_index = machine.with_instance(|instance| {
            (0..instance.input_count()).find(|&index| {
                instance
                    .input(index)
                    .is_some_and(|input| input.name() == name)
            })
        })?;
        Some((input_index, machine_input_kind(&machine, input_index)?))
    }

    pub fn set_occurrence_state_machine_bool(
        &mut self,
        occurrence: &[RuntimeArtboardOccurrenceSegment],
        state_machine_index: usize,
        input_index: usize,
        value: bool,
    ) -> Option<bool> {
        let machine = occurrence_machine(&self.native_handle(), occurrence, state_machine_index)?;
        let input_index = u32::try_from(input_index).ok()?;
        machine.with_instance_mut(|machine| {
            let input = machine.bool_input_mut(input_index)?;
            let changed = input.value() != value;
            input.set_value(value);
            Some(changed)
        })
    }

    pub fn occurrence_view_model_boolean(
        &self,
        occurrence: &[RuntimeArtboardOccurrenceSegment],
        source_path: &[u32],
    ) -> Option<bool> {
        let artboard = resolve_occurrence_artboard(&self.native_handle(), occurrence)?;
        let context = artboard.with_artboard(|artboard| artboard.base.data_context())?;
        let property =
            context.with_context(|context| context.get_view_model_property(source_path))?;
        property.with_downcast::<ViewModelInstanceBoolean, _>(ViewModelInstanceBoolean::value)
    }
}

impl ArtboardInstance {
    /// The translated runtime has no retained semantic-geometry revision
    /// authority. Hashing surrounding values would invent one, so hosts fail
    /// closed until the canonical owner exposes an exact revision.
    pub fn try_semantic_geometry_revision(&self) -> Option<SemanticGeometryRevision> {
        None
    }

    /// Register editor-supplied intrinsic dimensions on the exact ImageAsset.
    pub fn register_image_dimensions(
        &mut self,
        asset_global_id: u32,
        width: u32,
        height: u32,
    ) -> Result<(), RuntimeImageDimensionConflict> {
        let asset = self.native_file().with_file(|file| {
            file.assets()
                .iter()
                .find(|asset| {
                    u32::try_from(asset.identity_key().1) == Ok(asset_global_id)
                        && asset
                            .with(|asset| asset.as_any().is::<ImageAsset>())
                            .unwrap_or(false)
                })
                .cloned()
        });
        let Some(asset) = asset else {
            return Ok(());
        };
        let previous = asset
            .with_downcast::<ImageAsset, _>(|asset| (asset.base.width(), asset.base.height()))
            .unwrap_or_default();
        let actual = (width, height);
        if previous.0 > 0.0 && previous.1 > 0.0 && previous != (width as f32, height as f32) {
            return Err(RuntimeImageDimensionConflict {
                asset_global: asset_global_id,
                expected: (previous.0 as u32, previous.1 as u32),
                actual,
            });
        }
        CoreRegistry::set_double_handle(
            &asset,
            i32::from(DrawableAssetBase::WIDTH_PROPERTY_KEY),
            width as f32,
        );
        CoreRegistry::set_double_handle(
            &asset,
            i32::from(DrawableAssetBase::HEIGHT_PROPERTY_KEY),
            height as f32,
        );
        Ok(())
    }
}
