//! The persistent Taffy tree of one layout root.
//!
//! Every LayoutComponent and LayoutParticipant under a layout root owns one
//! stable node in that root's tree. Style appliers write the owner's
//! [`LayoutData`](super::layout_data::LayoutData) style and queue the node;
//! a solve copies only queued, changed styles into the tree, reconciles
//! children only where topology changed, and lets Taffy's per-node caches
//! carry every untouched subtree across passes.
//!
//! The queue lives outside the tree, so a measure callback that runs script
//! mid-solve (and dirties layout) only appends to it; the next solve picks
//! the change up.
use crate::mechanical_port::source::{
    core::CoreHandle, layout::layout_measure_mode::LayoutMeasureMode, math::vec2d::Vec2D,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use taffy::{
    DetailedGridTracksInfo, LengthPercentage, LengthPercentageAuto, TraversePartialTree,
    prelude::{
        AvailableSpace, Dimension, Display, GridPlacement, Line, NodeId, Size, Style,
        TaffyGridLine, TaffyTree,
    },
    style::{ExpandedDimension, ExpandedLengthPercentage, ExpandedLengthPercentageAuto},
};

/// Nodes queued for a tree by their owners since its last solve.
#[derive(Default)]
struct DirtyQueue(RefCell<Vec<NodeId>>);

/// An owner's stable node: the tree identity (its queue) and the node id.
#[derive(Clone)]
pub(crate) struct LayoutNodeLink {
    queue: Rc<DirtyQueue>,
    id: NodeId,
}

impl PartialEq for LayoutNodeLink {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Rc::ptr_eq(&self.queue, &other.queue)
    }
}

impl LayoutNodeLink {
    pub(crate) fn id(&self) -> NodeId {
        self.id
    }
    pub(crate) fn enqueue(&self) {
        self.queue.0.borrow_mut().push(self.id);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LayoutNodeKind {
    Layout,
    Participant,
}

pub(crate) struct LayoutNodeContext {
    pub(crate) owner: CoreHandle,
    pub(crate) kind: LayoutNodeKind,
    /// Measured through the owner's intrinsic size (text, images, scripted
    /// layouts, intrinsically sized layouts without layout children).
    pub(crate) measured: bool,
    /// Taffy-only grid items standing in for a virtualized grid's
    /// unrealized items; appended after the realized children.
    pub(crate) virtual_items: Vec<NodeId>,
}

pub(crate) struct LayoutTree {
    pub(crate) taffy: TaffyTree<LayoutNodeContext>,
    queue: Rc<DirtyQueue>,
    pub(crate) root: NodeId,
    /// Available size of the last root style resolution (NaN: unbounded).
    pub(crate) root_available: (f32, f32),
}

impl LayoutTree {
    pub(crate) fn new(root_owner: CoreHandle, kind: LayoutNodeKind) -> Self {
        let mut taffy = TaffyTree::new();
        taffy.disable_rounding();
        let root = taffy
            .new_leaf_with_context(
                Style::default(),
                LayoutNodeContext {
                    owner: root_owner,
                    kind,
                    measured: false,
                    virtual_items: Vec::new(),
                },
            )
            .expect("new layout root");
        Self {
            taffy,
            queue: Rc::default(),
            root,
            root_available: (f32::NAN, f32::NAN),
        }
    }

    pub(crate) fn link(&self, id: NodeId) -> LayoutNodeLink {
        LayoutNodeLink {
            queue: self.queue.clone(),
            id,
        }
    }

    /// Whether `link` addresses a live node of this tree owned by `owner`.
    pub(crate) fn owns(&self, link: &LayoutNodeLink, owner: &CoreHandle) -> bool {
        Rc::ptr_eq(&link.queue, &self.queue)
            && self
                .taffy
                .get_node_context(link.id)
                .is_some_and(|context| &context.owner == owner)
    }

    pub(crate) fn new_node(&mut self, owner: CoreHandle, kind: LayoutNodeKind) -> NodeId {
        self.taffy
            .new_leaf_with_context(
                Style::default(),
                LayoutNodeContext {
                    owner,
                    kind,
                    measured: false,
                    virtual_items: Vec::new(),
                },
            )
            .expect("new layout node")
    }

    pub(crate) fn take_queue(&self) -> Vec<NodeId> {
        std::mem::take(&mut *self.queue.0.borrow_mut())
    }

    /// Replace `node`'s style only if it differs; returns whether it did.
    pub(crate) fn sync_style(&mut self, node: NodeId, style: &Style) -> bool {
        if self.taffy.style(node).expect("live layout node") == style {
            return false;
        }
        self.taffy
            .set_style(node, style.clone())
            .expect("live layout node");
        true
    }

    /// Drop every removal candidate that no longer has a parent, together
    /// with the subtree it still holds. Returns the owners whose node was
    /// dropped so they can be unlinked.
    pub(crate) fn sweep(&mut self, candidates: Vec<NodeId>) -> Vec<(CoreHandle, LayoutNodeLink)> {
        let mut dropped = Vec::new();
        let mut removed = HashSet::new();
        let mut pending = candidates;
        while let Some(node) = pending.pop() {
            if node == self.root || removed.contains(&node) || self.taffy.parent(node).is_some() {
                continue;
            }
            pending.extend(self.taffy.child_ids(node).collect::<Vec<_>>());
            if let Some(context) = self.taffy.get_node_context(node) {
                dropped.push((context.owner.clone(), self.link(node)));
            }
            self.taffy.remove(node).expect("live layout node");
            removed.insert(node);
        }
        dropped
    }

    /// Rebuild a grid container's stand-ins for unrealized items (see
    /// [`virtual_item_styles`]), reusing existing nodes.
    pub(crate) fn sync_virtual_items(
        &mut self,
        container: NodeId,
        rows: &[f32],
        columns: &[f32],
    ) -> Vec<NodeId> {
        let styles = virtual_item_styles(rows, columns);
        let mut items = std::mem::take(
            &mut self
                .taffy
                .get_node_context_mut(container)
                .expect("live grid container")
                .virtual_items,
        );
        for extra in items.drain(styles.len().min(items.len())..) {
            let _ = self.taffy.remove(extra);
        }
        for (index, style) in styles.into_iter().enumerate() {
            if let Some(&item) = items.get(index) {
                self.sync_style(item, &style);
            } else {
                items.push(self.taffy.new_leaf(style).expect("new virtual grid item"));
            }
        }
        self.taffy
            .get_node_context_mut(container)
            .expect("live grid container")
            .virtual_items = items.clone();
        items
    }
}

/// Stock Taffy's per-axis grid track limit (`taffy::compute::grid`'s
/// private `MAX_GRID_TRACKS`). Rive's Yoga grid accepted any `int32` line.
pub const MAX_GRID_TRACKS: usize = 10_000;

/// Taffy-only grid items that stand in for a virtualized grid's unrealized
/// items. Each nonzero contribution becomes a fixed-size item in its track
/// (and on line one of the other axis), so the track sizes exactly as it
/// would around the realized item. The last track always gets one, so the
/// implicit grid still extends over every virtual track. Stock Taffy clamps
/// a grid to [`MAX_GRID_TRACKS`] tracks per axis (CSS "overlarge grids"), so
/// tracks past that limit are not represented.
pub fn virtual_item_styles(rows: &[f32], columns: &[f32]) -> Vec<Style> {
    let mut styles = Vec::new();
    for (contributions, rows_axis) in [(rows, true), (columns, false)] {
        let count = contributions.len().min(MAX_GRID_TRACKS);
        for (index, &size) in contributions.iter().enumerate().take(count) {
            let size = if size.is_finite() { size } else { 0.0 };
            if size <= 0.0 && index + 1 != count {
                continue;
            }
            let track = GridPlacement::from_line_index(index as i16 + 1);
            let first = GridPlacement::from_line_index(1);
            let (row, column, width, height) = if rows_axis {
                (track, first, 0.0, size.max(0.0))
            } else {
                (first, track, size.max(0.0), 0.0)
            };
            styles.push(Style {
                size: Size {
                    width: Dimension::length(width),
                    height: Dimension::length(height),
                },
                grid_row: Line {
                    start: row,
                    end: GridPlacement::Auto,
                },
                grid_column: Line {
                    start: column,
                    end: GridPlacement::Auto,
                },
                ..Style::default()
            });
        }
    }
    styles
}

/// Map a Taffy measurement request onto Rive's intrinsic measure protocol.
/// A known axis is measured exactly at its content-box size (Taffy's Definite
/// available space already excludes padding and border). CSS min- and
/// max-content both ask for the content's own size: Rive's measurers clamp
/// to any bound they are given, so they report intrinsic size only unbounded.
pub fn measure_axis(known: Option<f32>, available: AvailableSpace) -> (f32, LayoutMeasureMode) {
    let bounded = |value: f32| {
        if value.is_nan() {
            value
        } else {
            value.max(0.0)
        }
    };
    if let Some(outer) = known {
        let inner = match available {
            AvailableSpace::Definite(value) => value,
            _ => outer,
        };
        return (bounded(inner), LayoutMeasureMode::Exactly);
    }
    match available {
        AvailableSpace::Definite(value) => (bounded(value), LayoutMeasureMode::AtMost),
        AvailableSpace::MinContent | AvailableSpace::MaxContent => {
            (f32::NAN, LayoutMeasureMode::Undefined)
        }
    }
}

/// Resolve width limits even during ContentSize, where Taffy defers these
/// clamps. Height must already reflect the final content-box width.
pub(crate) fn content_width_limits(
    style: &Style,
    parent: taffy::geometry::Size<Option<f32>>,
) -> (Option<f32>, Option<f32>) {
    use taffy::util::{MaybeMath, MaybeResolve, ResolveOrZero};
    let calc = |_: *const (), _: f32| 0.0;
    let padding_border = style.padding.resolve_or_zero(parent.width, calc)
        + style.border.resolve_or_zero(parent.width, calc);
    let inset = if style.box_sizing == taffy::BoxSizing::BorderBox {
        padding_border.horizontal_axis_sum()
    } else {
        0.0
    } + if style.overflow.y == taffy::Overflow::Scroll {
        style.scrollbar_width
    } else {
        0.0
    };
    let content = |value: Option<f32>| value.map(|value| (value - inset).max(0.0));
    (
        content(
            style
                .min_size
                .maybe_resolve(parent, calc)
                .maybe_apply_aspect_ratio(style.aspect_ratio)
                .width,
        ),
        content(style.max_size.width.maybe_resolve(parent.width, calc)),
    )
}

/// Answer a bounded intrinsic inline-size offer using CSS fit-content:
/// min(max-content, max(min-content, stretch-fit)). Taffy passes content-box
/// offers here; width limits are resolved from the same style and box sizing.
/// A host without a meaningful minimum keeps its existing measurement protocol.
/// Exact and unbounded axes, including hugUnbounded, do not take this path.
pub(crate) fn measure_fit_content(
    width: (f32, LayoutMeasureMode),
    height: (f32, LayoutMeasureMode),
    limits: (Option<f32>, Option<f32>),
    supports_minimum: bool,
    minimum: impl FnOnce() -> Option<f32>,
    mut measure: impl FnMut(f32, LayoutMeasureMode, f32, LayoutMeasureMode) -> Vec2D,
) -> Vec2D {
    if supports_minimum && width.1 == LayoutMeasureMode::AtMost && width.0.is_finite() {
        let maximum = measure(
            f32::NAN,
            LayoutMeasureMode::Undefined,
            f32::NAN,
            LayoutMeasureMode::Undefined,
        );
        if let Some(minimum) = minimum().filter(|minimum| minimum.is_finite()) {
            if maximum.x.is_finite() {
                let inline = maximum.x.min(minimum.max(width.0));
                let mut wrapping_width = inline;
                if let Some(maximum) = limits.1 {
                    wrapping_width = wrapping_width.min(maximum);
                }
                if let Some(minimum) = limits.0 {
                    wrapping_width = wrapping_width.max(minimum);
                }
                let mut measured = measure(
                    wrapping_width,
                    LayoutMeasureMode::Exactly,
                    height.0,
                    height.1,
                );
                // Upstream text returns the widest wrapped line even for an
                // exact offer. The CSS box keeps the chosen inline size.
                // Preserve the intrinsic flex basis: ContentSize intentionally
                // ignores min/max on its main axis. Taffy clamps the box later;
                // its block size must already use that final wrapping width.
                measured.x = inline;
                return measured;
            }
        }
    }
    measure(width.0, width.1, height.0, height.1)
}

fn resolve_percent(value: ExpandedLengthPercentageAuto, owner: f32) -> Option<f32> {
    match value {
        ExpandedLengthPercentageAuto::Length(value) => Some(value),
        ExpandedLengthPercentageAuto::Percent(value) if !owner.is_nan() => Some(value * owner),
        _ => None,
    }
}

/// The style a layout root solves with. A Rive layout root without an
/// authored size or maximum on an axis fills the space its host offers
/// (less margins); otherwise it keeps its authored sizing.
pub fn root_style(style: &Style, available: (f32, f32)) -> Style {
    let mut root = style.clone();
    let margin = |start: LengthPercentageAuto, end: LengthPercentageAuto| {
        resolve_percent(start.expand(), available.0).unwrap_or(0.0)
            + resolve_percent(end.expand(), available.0).unwrap_or(0.0)
    };
    for (owner, size, min, max, margin) in [
        (
            available.0,
            &mut root.size.width,
            style.min_size.width,
            style.max_size.width,
            margin(style.margin.left, style.margin.right),
        ),
        (
            available.1,
            &mut root.size.height,
            style.min_size.height,
            style.max_size.height,
            margin(style.margin.top, style.margin.bottom),
        ),
    ] {
        if owner.is_nan() {
            continue;
        }
        let sized = match size.expand() {
            ExpandedDimension::Length(value) => value >= 0.0,
            ExpandedDimension::Percent(value) => value >= 0.0,
            _ => false,
        };
        let pinned = !min.is_auto() && min == max;
        if sized || pinned || resolve_percent(max.expand(), owner).is_some() {
            continue;
        }
        *size = Dimension::length((owner - margin).max(0.0));
    }
    root
}

/// Solved grid lines (each track start, then the last track's end) and the
/// realized gap between adjacent tracks, relative to the container's border
/// box. With fewer than two tracks the authored gap is reported.
pub fn grid_lines(info: &DetailedGridTracksInfo, authored_gap: f32) -> (Vec<f32>, f32) {
    let positions = &info.positions;
    let mut lines: Vec<f32> = positions.iter().map(|track| track.start).collect();
    match positions.last() {
        Some(last) => lines.push(last.end),
        None => lines.push(info.empty_axis_line.unwrap_or(0.0)),
    }
    let gap = if positions.len() >= 2 {
        positions[1].start - positions[0].end
    } else {
        authored_gap
    };
    (lines, gap)
}

/// An authored gap resolved against the container's content size.
pub(crate) fn resolve_gap(gap: LengthPercentage, content: f32) -> f32 {
    match gap.expand() {
        ExpandedLengthPercentage::Length(value) => value,
        ExpandedLengthPercentage::Percent(value) if !content.is_nan() => value * content,
        _ => 0.0,
    }
}

pub(crate) fn is_grid(style: &Style) -> bool {
    style.display == Display::Grid
}

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::{
        geometry::Rect,
        prelude::TaffyMaxContent,
        style_helpers::length,
        tree::{LayoutInput, RunMode, SizingMode},
    };

    #[test]
    fn measure_axis_maps_taffy_requests_onto_rive_modes() {
        use AvailableSpace::{Definite, MaxContent, MinContent};
        assert!(measure_axis(Some(4.0), Definite(0.0)) == (0.0, LayoutMeasureMode::Exactly));
        assert!(measure_axis(Some(4.0), Definite(-4.0)) == (0.0, LayoutMeasureMode::Exactly));
        assert!(measure_axis(Some(4.0), MaxContent) == (4.0, LayoutMeasureMode::Exactly));
        assert!(measure_axis(None, Definite(-4.0)) == (0.0, LayoutMeasureMode::AtMost));
        assert!(measure_axis(None, Definite(f32::NAN)).0.is_nan());
        for intrinsic in [MinContent, MaxContent] {
            let (size, mode) = measure_axis(None, intrinsic);
            assert!(size.is_nan());
            assert!(mode == LayoutMeasureMode::Undefined);
        }
    }

    #[test]
    fn padded_leaf_measures_its_content_box_and_keeps_its_outer_box() {
        let style: Style = Style {
            padding: Rect {
                left: length(10.0),
                right: length(10.0),
                top: length(2.0),
                bottom: length(2.0),
            },
            ..Style::default()
        };
        for (known, expected_inner, expected_outer) in [
            (
                Size {
                    width: Some(100.0),
                    height: None,
                },
                (80.0, LayoutMeasureMode::Exactly),
                Size {
                    width: 100.0,
                    height: 12.0,
                },
            ),
            (
                Size {
                    width: None,
                    height: Some(80.0),
                },
                (76.0, LayoutMeasureMode::Exactly),
                Size {
                    width: 28.0,
                    height: 80.0,
                },
            ),
        ] {
            let mut calls = 0;
            let output = taffy::compute_leaf_layout(
                LayoutInput {
                    run_mode: RunMode::ComputeSize,
                    sizing_mode: SizingMode::InherentSize,
                    known_dimensions: known,
                    available_space: Size {
                        width: AvailableSpace::MaxContent,
                        height: AvailableSpace::MaxContent,
                    },
                    ..LayoutInput::HIDDEN
                },
                &style,
                |_, _| 0.0,
                |known, available| {
                    calls += 1;
                    let width = measure_axis(known.width, available.width);
                    let height = measure_axis(known.height, available.height);
                    assert!((if known.width.is_some() { width } else { height }) == expected_inner);
                    Size {
                        width: 8.0,
                        height: 8.0,
                    }
                },
            );
            assert_eq!(calls, 1);
            assert_eq!(output.size, expected_outer);
        }
    }

    #[test]
    fn layout_root_without_authored_size_fills_the_offered_space() {
        let style = Style {
            margin: Rect {
                left: LengthPercentageAuto::length(5.0),
                right: LengthPercentageAuto::percent(0.1),
                top: LengthPercentageAuto::length(0.0),
                bottom: LengthPercentageAuto::length(0.0),
            },
            ..Style::default()
        };
        let root = root_style(&style, (200.0, f32::NAN));
        assert_eq!(root.size.width, Dimension::length(175.0));
        assert!(root.size.height.is_auto());
        let sized = Style {
            size: Size {
                width: Dimension::length(50.0),
                height: Dimension::auto(),
            },
            max_size: Size {
                width: LengthPercentageAuto::auto(),
                height: LengthPercentageAuto::length(30.0),
            },
            ..Style::default()
        };
        assert_eq!(root_style(&sized, (200.0, 100.0)), sized);
    }

    fn grid_with_virtual_items(rows: &[f32], realized: &[(i16, f32)]) -> (TaffyTree<()>, NodeId) {
        let mut taffy = TaffyTree::<()>::new();
        let mut children = Vec::new();
        for &(row, height) in realized {
            children.push(
                taffy
                    .new_leaf(Style {
                        size: Size {
                            width: length(10.0),
                            height: length(height),
                        },
                        grid_row: Line {
                            start: GridPlacement::from_line_index(row),
                            end: GridPlacement::Auto,
                        },
                        grid_column: Line {
                            start: GridPlacement::from_line_index(1),
                            end: GridPlacement::Auto,
                        },
                        ..Style::default()
                    })
                    .unwrap(),
            );
        }
        for style in virtual_item_styles(rows, &[]) {
            children.push(taffy.new_leaf(style).unwrap());
        }
        let grid = taffy
            .new_with_children(
                Style {
                    display: Display::Grid,
                    ..Style::default()
                },
                &children,
            )
            .unwrap();
        (taffy, grid)
    }

    #[test]
    fn virtual_grid_items_size_tracks_as_realized_items_would() {
        let rows = [30.0, 0.0, 50.0, 20.0];
        let (mut all, all_grid) = grid_with_virtual_items(&[], &[(1, 30.0), (3, 50.0), (4, 20.0)]);
        all.compute_layout(all_grid, Size::MAX_CONTENT).unwrap();
        let (mut some, some_grid) = grid_with_virtual_items(&rows, &[(3, 50.0)]);
        some.compute_layout(some_grid, Size::MAX_CONTENT).unwrap();
        let lines = |taffy: &TaffyTree<()>, node| match taffy.detailed_layout_info(node) {
            taffy::tree::DetailedLayoutInfo::Grid(info) => grid_lines(&info.rows, 0.0).0,
            _ => unreachable!(),
        };
        assert_eq!(lines(&all, all_grid), lines(&some, some_grid));
        assert_eq!(lines(&some, some_grid), vec![0.0, 30.0, 30.0, 80.0, 100.0]);
        assert_eq!(
            all.layout(all_grid).unwrap().size,
            some.layout(some_grid).unwrap().size
        );
    }
}
