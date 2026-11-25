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
//!
//! A region is listed only where the runtime would take a drag on it. A
//! drag reaches a scroller through the root state machine's hit components:
//! a nested artboard's or list's hit component forwards it into the hosted
//! artboard's own state machine, and there the constraint's listener group
//! (`DraggableConstraint::listener_groups`) hit-tests the viewport's proxy
//! and hands the drag to `ViewportDraggableProxy`. Each gate on that path,
//! where the pointer path checks it, and where collection does:
//!
//! | Gate | Pointer path | Collection |
//! |---|---|---|
//! | Host collapsed, hidden or paused | `HitNestedArtboard` (`nested_refuses_pointers`), `HitComponentList` (`list_refuses_pointers`) | the same predicates, through `pointer_hosted_instances` |
//! | Host transform cannot be inverted (scaled to 0) | `nested_mapping`, `component_list_world_to_local` | the same, through `pointer_hosted_instances` |
//! | Hosted artboard has no state machine | `nested_state_machine`, `component_list_state_machine` | the same, through `pointer_hosted_instances` |
//! | Rows and their order | `component_list_indices` | the same, through `pointer_hosted_instances` |
//! | Host that forwards no pointers (a picture painted from an artboard) | no hit component | `pointer_hosted_instances` returns none |
//! | Component collapsed (a `display: none` box and what it holds) | `Drawable::is_hidden` (collapsed dirt) | the walk does not enter it (`component_is_collapsed`) |
//! | Viewport hidden | `RuntimeDrawableOccurrence::hit_test_point` refuses a hidden proxy | `RuntimeDrawableOccurrence::is_hidden` on the same proxy |
//! | Viewport transform cannot be inverted | `LayoutComponent::hit_test_point_with_origin` | restated: the viewport's world transform must invert |
//! | Viewport clipped away by the clips around it | the hit walks the same rendered clips (docs/CSS_LAYOUT.md, padding-box clip) | `semantic_is_fully_clipped`, the semantic layer's walk of those clips |
//! | Constraint not interactive | `ViewportDraggableProxy::takes_drag`, `drag`, `start_drag` (`interactive`) | restated: `interactive()` |
//! | Viewport not a layout box with a layout parent | `ScrollConstraint::draggables` needs the viewport's proxy | restated: `has_layout_parent`, a layout viewport |
//! | Box empty on screen | nothing to hit | rechecked on the final root-space box |
//! | Player sends its pointers to no state machine (a static or linear animation player) | `PagePlayer::operation` applies a step's pointers only in its `Playback::StateMachine` arm and answers every other playback with no hit | not here: the caller collects only for a state machine player (`nux_player_semantic_snapshot`, restated, since that arm binds the machine and has no predicate to share) |
//!
//! The constraint's own `wants_scroll` and `can_consume` answer for wheel
//! scrolls; a drag reads `can_move_content`'s test instead, which is what
//! `can_scroll` reports.
use std::collections::{HashMap, HashSet};

use crate::mechanical_port::source::animation::state_machine_instance::pointer_hosted_instances;
use crate::mechanical_port::source::{
    artboard::Artboard,
    core::CoreHandle,
    math::mat2d::Mat2D,
    math::vec2d::Vec2D,
    semantic::{
        semantic_provider::{root_transform_aabb, semantic_is_fully_clipped},
        semantic_snapshot::Bounds,
    },
};

/// At most this many regions in one capture; more fail it as a whole.
pub const MAX_SCROLL_REGIONS: usize = 256;

/// `ScrollRegion::can_scroll` bits. Directions are on screen: in a
/// right-to-left viewport, whose start is its right edge, and in a host
/// that turns or mirrors the scroller too.
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
    /// move, so it agrees with what a drag would do, named for the way the
    /// move points once mapped to root space.
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
    /// The regions under `root`, an artboard played by a state machine, its
    /// nested artboards and the rows its lists show included: each scroller
    /// a drag can reach, by the gates in the module's table.
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
            let Some(mut children) = handle
                .with(|object| {
                    if object.component_is_collapsed() {
                        return None;
                    }
                    Some(
                        object
                            .as_container_component()
                            .map(|container| container.children().to_vec())
                            .unwrap_or_default(),
                    )
                })
                .flatten()
            else {
                continue;
            };
            // A host's artboards that its hit component would pass a drag
            // into, by the pointer path's own gates.
            children.extend(
                pointer_hosted_instances(&handle)
                    .into_iter()
                    .map(|instance| instance.core_handle()),
            );
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

/// The content moves a scroller is asked about: one point each way in its
/// viewport's own space, down, up, right and left.
const LOCAL_MOVES: [Vec2D; 4] = [
    Vec2D { x: 0.0, y: 1.0 },
    Vec2D { x: 0.0, y: -1.0 },
    Vec2D { x: 1.0, y: 0.0 },
    Vec2D { x: -1.0, y: 0.0 },
];

/// A listed constraint's viewport, its box in root space and its
/// `can_scroll` bits.
fn measure(constraint: &CoreHandle) -> Option<(CoreHandle, Bounds, u32)> {
    let (viewport, local, world, artboard, moves) = constraint
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
            let moves = LOCAL_MOVES.map(|movement| scroll.can_move_content(movement));
            Some((viewport, local, world, artboard, moves))
        })
        .flatten()?;
    // The drag's target is the viewport's proxy: hidden, it is not hit.
    let hidden = viewport
        .with_mut(|viewport| {
            viewport
                .as_layout_component_mut()
                .and_then(|layout| layout.proxy())
        })
        .flatten()
        .is_none_or(|proxy| proxy.is_hidden());
    if hidden || !world.invert(&mut Mat2D::default()) {
        return None;
    }
    let world_box = world.map_bounding_box(local);
    let bounds = Bounds {
        min_x: world_box.min_x,
        min_y: world_box.min_y,
        max_x: world_box.max_x,
        max_y: world_box.max_y,
    };
    if bounds.is_empty_or_nan() {
        return None;
    }
    let bounds = artboard
        .as_ref()
        .map_or(bounds, |artboard| root_transform_aabb(artboard, bounds));
    // Empty on screen once mapped through every host, or clipped away by
    // the clips around it: no point of it can be hit.
    if bounds.is_empty_or_nan() || semantic_is_fully_clipped(&viewport) {
        return None;
    }
    // Each move the scroller allows, carried to the screen through the same
    // transforms as its box, so a scroller in a turned or mirrored host
    // names the way its content moves on screen.
    let to_root = |point: Vec2D| {
        artboard
            .as_ref()
            .and_then(|artboard| {
                artboard.with_downcast_mut::<Artboard, _>(|artboard| artboard.root_transform(point))
            })
            .unwrap_or(point)
    };
    let start = to_root(world * Vec2D::new(0.0, 0.0));
    let bits = LOCAL_MOVES
        .into_iter()
        .zip(moves)
        .filter(|&(_, can)| can)
        .fold(0, |bits, (movement, _)| {
            bits | screen_bit(to_root(world * movement) - start)
        });
    Some((viewport, bounds, bits))
}

/// The bit for content moving by `movement` on screen. Content moving down
/// shows what is above it, so a move down is TOP, up is BOTTOM, right is
/// LEFT and left is RIGHT. A move along a diagonal counts for the axis it
/// mostly points along, and an exact diagonal counts as vertical. A move
/// that vanishes or is not finite names no way.
fn screen_bit(movement: Vec2D) -> u32 {
    let (x, y) = (movement.x, movement.y);
    if !(x.is_finite() && y.is_finite()) || (x == 0.0 && y == 0.0) {
        0
    } else if y.abs() >= x.abs() {
        if y > 0.0 {
            can_scroll::TOP
        } else {
            can_scroll::BOTTOM
        }
    } else if x > 0.0 {
        can_scroll::LEFT
    } else {
        can_scroll::RIGHT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each axis, a diagonal leaning either way, an exact diagonal, and
    /// moves that name nothing.
    #[test]
    fn a_move_names_the_axis_it_mostly_points_along() {
        let bit = |x, y| screen_bit(Vec2D::new(x, y));
        assert_eq!(bit(0.0, 1.0), can_scroll::TOP);
        assert_eq!(bit(0.0, -1.0), can_scroll::BOTTOM);
        assert_eq!(bit(1.0, 0.0), can_scroll::LEFT);
        assert_eq!(bit(-1.0, 0.0), can_scroll::RIGHT);
        assert_eq!(bit(0.9, -0.3), can_scroll::LEFT);
        assert_eq!(bit(-0.3, -0.9), can_scroll::BOTTOM);
        assert_eq!(bit(-0.7, 0.7), can_scroll::TOP);
        assert_eq!(bit(0.0, 0.0), 0);
        assert_eq!(bit(f32::NAN, 1.0), 0);
    }
}
