//! Nuxie: the scroll regions of a presented scene, for a host's capture of
//! a frame. Upstream's semantics carry no scroll facts; a host there asks a
//! state machine `wants_scroll` in process. Our hosts read a frame through
//! an immutable capture on their own thread, so the facts a native scroll
//! view would give them ride in it: where each scroll viewport is, whether
//! it can still move each way, and which region a node is in.
//!
//! Nothing is computed anew. A region is a `ScrollConstraint`'s viewport,
//! its box mapped to root space as node bounds are (`semantic_bounds`), and
//! its moves are the constraint's own clamp test. CSS and translated
//! scrollers are both upstream's `ScrollConstraint`, so both are listed.
use std::collections::{HashMap, HashSet};

use crate::mechanical_port::source::{
    artboard::Artboard,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::CoreHandle,
    math::vec2d::Vec2D,
    semantic::{semantic_provider::root_transform_aabb, semantic_snapshot::Bounds},
};

/// At most this many regions in one capture; more fail it as a whole.
pub const MAX_SCROLL_REGIONS: usize = 256;

/// `ScrollRegion::can_scroll` bits. Directions are on screen, also in a
/// right-to-left viewport, whose start is its right edge.
pub mod can_scroll {
    /// The region can move toward its top, showing content above.
    pub const TOP: u32 = 1 << 0;
    /// The region can move toward its bottom, showing content below.
    pub const BOTTOM: u32 = 1 << 1;
    /// The region can move toward its left, showing content to the left.
    pub const LEFT: u32 = 1 << 2;
    /// The region can move toward its right, showing content to the right.
    pub const RIGHT: u32 = 1 << 3;
}

/// One scroll viewport on screen.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollRegion {
    /// The viewport's padding box in root space, the space node bounds are
    /// in. Not clipped by the regions around it; `parent` names them.
    pub bounds: Bounds,
    /// `can_scroll` bits, each the constraint's clamp test for a one point
    /// move that way, so it agrees with what a drag would do.
    pub can_scroll: u32,
    /// The region this one is inside, an index before this one's, or -1.
    pub parent: i32,
}

/// A scene has more than [`MAX_SCROLL_REGIONS`] regions, or more than the
/// limit a caller asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooManyScrollRegions;

/// The scroll regions of a scene, in tree order: each after the region it
/// is inside.
#[derive(Default)]
pub struct ScrollRegions {
    regions: Vec<ScrollRegion>,
    /// Each region's viewport, to its index. Bounded as `regions` is.
    viewports: HashMap<CoreHandle, usize>,
}

impl ScrollRegions {
    /// The regions under `root`, an artboard, its nested artboards and the
    /// rows its lists show included. A region is listed when its constraint
    /// is interactive and its viewport is neither collapsed nor empty.
    pub fn collect(root: &CoreHandle) -> Result<Self, TooManyScrollRegions> {
        Self::collect_at_most(root, MAX_SCROLL_REGIONS)
    }

    /// As [`Self::collect`], failing past `limit` regions.
    pub fn collect_at_most(root: &CoreHandle, limit: usize) -> Result<Self, TooManyScrollRegions> {
        let mut found = Self::default();
        let mut pending = vec![root.clone()];
        // Each object of the scene is visited once, so the walk is bounded
        // by the scene; a collapsed component's subtree is not entered.
        let mut visited = HashSet::new();
        while let Some(handle) = pending.pop() {
            if !visited.insert(handle.clone()) {
                continue;
            }
            let Some(children) = handle
                .with(|object| {
                    if object.component_is_collapsed() {
                        return None;
                    }
                    let mut children = object
                        .as_container_component()
                        .map(|container| container.children().to_vec())
                        .unwrap_or_default();
                    // A host's artboards, with the same row membership as
                    // advancing, layout and hits use.
                    children.extend(
                        crate::hosted_scene::active_instances(object)
                            .map(|instance| instance.core_handle()),
                    );
                    Some(children)
                })
                .flatten()
            else {
                continue;
            };
            // A content box's scroll constraints are its children. Listing
            // them before walking into the box puts every region after the
            // region it is inside, so its parent is already known.
            for child in &children {
                let Some((viewport, bounds, can_scroll)) = measure(child) else {
                    continue;
                };
                if found.regions.len() == limit {
                    return Err(TooManyScrollRegions);
                }
                let parent = found.region_of(&viewport);
                found
                    .viewports
                    .entry(viewport)
                    .or_insert(found.regions.len());
                found.regions.push(ScrollRegion {
                    bounds,
                    can_scroll,
                    parent,
                });
            }
            pending.extend(children.into_iter().rev());
        }
        Ok(found)
    }

    pub fn regions(&self) -> &[ScrollRegion] {
        &self.regions
    }

    /// The innermost region whose viewport clips `component`: the nearest
    /// region viewport among its ancestors, across the artboards hosting
    /// it. -1 when it is in none.
    pub fn region_of(&self, component: &CoreHandle) -> i32 {
        if self.viewports.is_empty() {
            return -1;
        }
        let mut current = parent_of(component);
        // Each ancestor is visited once; a cycle ends the walk.
        let mut visited = HashSet::new();
        while let Some(ancestor) = current {
            if !visited.insert(ancestor.clone()) {
                break;
            }
            if let Some(&index) = self.viewports.get(&ancestor) {
                return i32::try_from(index).unwrap_or(-1);
            }
            current = parent_of(&ancestor);
        }
        -1
    }
}

fn parent_of(component: &CoreHandle) -> Option<CoreHandle> {
    component
        .with(|object| {
            object
                .component_parent_handle()
                .or_else(|| object.as_artboard().and_then(Artboard::host))
        })
        .flatten()
}

/// A listed constraint's viewport, its box in root space and its
/// `can_scroll` bits.
fn measure(constraint: &CoreHandle) -> Option<(CoreHandle, Bounds, u32)> {
    let (viewport, local, world, artboard, bits) = constraint
        .with(|object| {
            let scroll = object.as_scroll_constraint()?;
            if !scroll.interactive() || !scroll.has_layout_parent() {
                return None;
            }
            let viewport = scroll.viewport_handle()?;
            let (box_bounds, world, artboard, collapsed) = viewport
                .with(|viewport| {
                    let layout = viewport.as_layout_component()?;
                    Some((
                        layout.local_bounds(),
                        layout.shape_world_transform(),
                        layout.artboard_handle(),
                        viewport.component_is_collapsed(),
                    ))
                })
                .flatten()?;
            if collapsed {
                return None;
            }
            // A CSS viewport scrolls inside its padding box; a translated
            // one has no border, so its box is its padding box.
            let local = scroll.css_scrollport().unwrap_or(box_bounds);
            Some((viewport, local, world, artboard, can_scroll_bits(scroll)))
        })
        .flatten()?;
    let world = world.map_bounding_box(local);
    let bounds = Bounds {
        min_x: world.min_x,
        min_y: world.min_y,
        max_x: world.max_x,
        max_y: world.max_y,
    };
    if bounds.is_empty_or_nan() {
        return None;
    }
    let bounds = artboard.map_or(bounds, |artboard| root_transform_aabb(&artboard, bounds));
    Some((viewport, bounds, bits))
}

fn can_scroll_bits(scroll: &ScrollConstraint) -> u32 {
    [
        (Vec2D::new(0.0, 1.0), can_scroll::TOP),
        (Vec2D::new(0.0, -1.0), can_scroll::BOTTOM),
        (Vec2D::new(1.0, 0.0), can_scroll::LEFT),
        (Vec2D::new(-1.0, 0.0), can_scroll::RIGHT),
    ]
    .into_iter()
    .filter(|&(movement, _)| scroll.can_move_content(movement))
    .fold(0, |bits, (_, bit)| bits | bit)
}
