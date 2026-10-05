//! Literal port of yoga_grid_virtual_contributions_test.cpp at 160085c6.
//! Taffy is the approved layout engine; contributions and realized nodes must agree.
use taffy::{
    geometry::MinMax,
    prelude::*,
    style::{GridPlacement, GridTemplateComponent, MaxTrackSizingFunction, MinTrackSizingFunction},
    tree::DetailedLayoutInfo,
};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    A,
    P,
    PC,
    F,
}
use Kind::*;
type Track = (Kind, f32, Kind, f32);
const ROW_TRACKS: &[&[Track]] = &[
    &[
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, P, 40_f32),
        (P, 80_f32, P, 120_f32),
    ],
    &[],
    &[
        (A, 0_f32, F, 2_f32),
        (PC, 10_f32, PC, 10_f32),
        (P, 80_f32, P, 80_f32),
    ],
    &[(A, 0_f32, A, 0_f32)],
    &[(A, 0_f32, PC, 10_f32), (A, 0_f32, A, 0_f32)],
    &[(P, 0_f32, P, 0_f32)],
    &[(P, 40_f32, P, 40_f32)],
    &[(P, 0_f32, PC, 10_f32)],
    &[(A, 0_f32, A, 0_f32), (A, 0_f32, F, 0.5_f32)],
    &[
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, F, 1_f32),
    ],
    &[
        (A, 0_f32, F, 1_f32),
        (P, 120_f32, P, 80_f32),
        (A, 0_f32, P, 40_f32),
    ],
    &[(PC, 25_f32, PC, 25_f32)],
    &[(A, 0_f32, P, 80_f32)],
    &[(A, 0_f32, F, 1_f32), (A, 0_f32, P, 80_f32)],
    &[(P, 80_f32, F, 2_f32)],
    &[
        (P, 0_f32, P, 0_f32),
        (A, 0_f32, A, 0_f32),
        (PC, 50_f32, PC, 50_f32),
    ],
    &[
        (A, 0_f32, F, 1_f32),
        (A, 0_f32, F, 0.5_f32),
        (P, 80_f32, P, 0_f32),
    ],
    &[(PC, 10_f32, PC, 10_f32)],
    &[
        (PC, 25_f32, PC, 50_f32),
        (P, 0_f32, P, 120_f32),
        (A, 0_f32, F, 0.5_f32),
    ],
    &[(P, 120_f32, P, 120_f32)],
    &[(A, 0_f32, F, 1_f32), (PC, 25_f32, P, 80_f32)],
    &[(A, 0_f32, F, 0.5_f32), (A, 0_f32, PC, 10_f32)],
    &[
        (A, 0_f32, A, 0_f32),
        (PC, 10_f32, PC, 10_f32),
        (A, 0_f32, PC, 25_f32),
    ],
    &[(P, 0_f32, P, 0_f32), (PC, 25_f32, F, 0.5_f32)],
    &[
        (P, 80_f32, PC, 50_f32),
        (P, 80_f32, P, 80_f32),
        (A, 0_f32, P, 120_f32),
    ],
    &[
        (P, 0_f32, A, 0_f32),
        (P, 0_f32, PC, 25_f32),
        (PC, 25_f32, PC, 50_f32),
    ],
    &[(A, 0_f32, PC, 10_f32), (PC, 25_f32, A, 0_f32)],
    &[
        (A, 0_f32, F, 2_f32),
        (A, 0_f32, PC, 25_f32),
        (A, 0_f32, A, 0_f32),
    ],
    &[(A, 0_f32, F, 0.5_f32), (P, 40_f32, P, 40_f32)],
    &[(A, 0_f32, F, 2_f32), (P, 80_f32, PC, 50_f32)],
    &[
        (P, 80_f32, F, 1_f32),
        (P, 80_f32, F, 2_f32),
        (P, 40_f32, P, 40_f32),
    ],
    &[(A, 0_f32, P, 0_f32)],
    &[(PC, 50_f32, A, 0_f32), (A, 0_f32, F, 1_f32)],
    &[
        (PC, 25_f32, PC, 25_f32),
        (PC, 50_f32, P, 80_f32),
        (A, 0_f32, A, 0_f32),
    ],
    &[
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, PC, 10_f32),
    ],
    &[(A, 0_f32, F, 2_f32)],
    &[
        (P, 80_f32, F, 0.5_f32),
        (P, 40_f32, P, 0_f32),
        (P, 120_f32, PC, 50_f32),
    ],
    &[(A, 0_f32, F, 0.5_f32), (PC, 10_f32, P, 120_f32)],
    &[(PC, 25_f32, PC, 50_f32)],
    &[(PC, 10_f32, A, 0_f32)],
    &[
        (A, 0_f32, P, 120_f32),
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, F, 1_f32),
    ],
    &[
        (A, 0_f32, P, 120_f32),
        (P, 120_f32, P, 120_f32),
        (PC, 25_f32, PC, 25_f32),
    ],
    &[
        (A, 0_f32, A, 0_f32),
        (PC, 25_f32, PC, 10_f32),
        (A, 0_f32, A, 0_f32),
    ],
    &[(A, 0_f32, A, 0_f32), (A, 0_f32, P, 120_f32)],
    &[
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, F, 2_f32),
        (P, 120_f32, P, 120_f32),
    ],
    &[
        (P, 80_f32, PC, 10_f32),
        (A, 0_f32, F, 0.5_f32),
        (A, 0_f32, F, 0.5_f32),
    ],
    &[(A, 0_f32, A, 0_f32), (A, 0_f32, PC, 10_f32)],
    &[(PC, 50_f32, PC, 50_f32), (A, 0_f32, F, 2_f32)],
    &[
        (PC, 10_f32, PC, 10_f32),
        (PC, 50_f32, F, 1_f32),
        (P, 0_f32, P, 0_f32),
    ],
    &[
        (P, 40_f32, P, 40_f32),
        (PC, 25_f32, PC, 10_f32),
        (A, 0_f32, F, 2_f32),
    ],
    &[
        (A, 0_f32, F, 2_f32),
        (PC, 10_f32, A, 0_f32),
        (A, 0_f32, PC, 10_f32),
    ],
    &[
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, F, 1_f32),
        (PC, 50_f32, PC, 50_f32),
    ],
    &[(A, 0_f32, P, 80_f32), (A, 0_f32, A, 0_f32)],
    &[
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, F, 0.5_f32),
        (A, 0_f32, PC, 50_f32),
    ],
    &[(PC, 50_f32, P, 120_f32), (A, 0_f32, F, 2_f32)],
    &[(A, 0_f32, A, 0_f32), (A, 0_f32, F, 1_f32)],
    &[(P, 0_f32, F, 0.5_f32)],
    &[(A, 0_f32, F, 0.5_f32), (A, 0_f32, F, 0.5_f32)],
    &[(P, 120_f32, F, 2_f32), (A, 0_f32, P, 120_f32)],
    &[
        (A, 0_f32, F, 2_f32),
        (P, 120_f32, F, 0.5_f32),
        (A, 0_f32, P, 80_f32),
    ],
    &[
        (PC, 10_f32, PC, 10_f32),
        (A, 0_f32, F, 2_f32),
        (PC, 50_f32, F, 0.5_f32),
    ],
];
const COLUMN_TRACKS: &[&[Track]] = &[
    &[(A, 0_f32, P, 0_f32), (PC, 10_f32, F, 0.5_f32)],
    &[(P, 80_f32, F, 1_f32), (A, 0_f32, F, 1_f32)],
    &[(P, 40_f32, P, 40_f32), (A, 0_f32, F, 0.5_f32)],
    &[
        (A, 0_f32, F, 1_f32),
        (A, 0_f32, F, 1_f32),
        (P, 40_f32, F, 0.5_f32),
    ],
    &[(A, 0_f32, P, 80_f32), (A, 0_f32, P, 80_f32)],
    &[
        (A, 0_f32, P, 120_f32),
        (A, 0_f32, F, 2_f32),
        (P, 120_f32, A, 0_f32),
    ],
    &[
        (A, 0_f32, P, 80_f32),
        (P, 40_f32, PC, 50_f32),
        (PC, 25_f32, PC, 25_f32),
    ],
    &[
        (A, 0_f32, F, 1_f32),
        (P, 0_f32, P, 0_f32),
        (A, 0_f32, A, 0_f32),
    ],
    &[
        (P, 40_f32, P, 80_f32),
        (A, 0_f32, A, 0_f32),
        (A, 0_f32, P, 40_f32),
    ],
    &[(A, 0_f32, P, 0_f32), (A, 0_f32, F, 0.5_f32)],
    &[(P, 0_f32, F, 0.5_f32), (PC, 25_f32, F, 1_f32)],
    &[(PC, 50_f32, P, 120_f32), (A, 0_f32, A, 0_f32)],
    &[(A, 0_f32, A, 0_f32), (A, 0_f32, A, 0_f32)],
    &[(A, 0_f32, F, 2_f32), (PC, 10_f32, PC, 10_f32)],
    &[
        (A, 0_f32, P, 120_f32),
        (P, 120_f32, P, 120_f32),
        (A, 0_f32, F, 2_f32),
    ],
    &[(PC, 50_f32, PC, 50_f32), (A, 0_f32, F, 1_f32)],
    &[
        (P, 0_f32, P, 120_f32),
        (A, 0_f32, F, 2_f32),
        (A, 0_f32, F, 1_f32),
    ],
    &[
        (A, 0_f32, P, 80_f32),
        (P, 0_f32, F, 0.5_f32),
        (P, 120_f32, P, 120_f32),
    ],
    &[(P, 80_f32, A, 0_f32), (P, 0_f32, F, 0.5_f32)],
];

fn tracks(values: &[Track]) -> Vec<GridTemplateComponent<String>> {
    values
        .iter()
        .map(|&(a, x, b, y)| {
            GridTemplateComponent::Single(MinMax {
                min: match a {
                    P => MinTrackSizingFunction::length(x),
                    PC => MinTrackSizingFunction::percent(x / 100.0),
                    _ => MinTrackSizingFunction::auto(),
                },
                max: match b {
                    P => MaxTrackSizingFunction::length(y),
                    PC => MaxTrackSizingFunction::percent(y / 100.0),
                    F => MaxTrackSizingFunction::fr(y),
                    A => MaxTrackSizingFunction::auto(),
                },
            })
        })
        .collect()
}
fn yoga_style() -> Style {
    Style {
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        align_content: Some(AlignContent::START),
        justify_content: Some(JustifyContent::START),
        align_items: Some(AlignItems::STRETCH),
        justify_items: Some(JustifyItems::STRETCH),
        ..Default::default()
    }
}
fn grid_style(width: f32) -> Style {
    Style {
        display: Display::Grid,
        size: Size {
            width: length(width),
            height: auto(),
        },
        ..yoga_style()
    }
}
fn tree() -> TaffyTree<()> {
    let mut tree = TaffyTree::new();
    tree.disable_rounding();
    tree
}
fn calculate(t: &mut TaffyTree<()>, root: NodeId) {
    t.compute_layout(
        root,
        Size {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
    )
    .unwrap();
}
fn lines(t: &TaffyTree<()>, node: NodeId, rows: bool) -> Vec<f32> {
    match t.detailed_layout_info(node) {
        DetailedLayoutInfo::Grid(info) => {
            if rows {
                info.rows.line_offsets.clone()
            } else {
                info.columns.line_offsets.clone()
            }
        }
        _ => Vec::new(),
    }
}
fn approx(actual: f32, expected: f32, margin: f32) {
    let delta = (f64::from(actual) - f64::from(expected)).abs();
    assert!(
        delta <= f64::from(margin)
            || delta <= f64::from(100.0 * f32::EPSILON) * f64::from(expected).abs(),
        "{actual} != {expected}"
    );
}
fn check_same(expected: &[f32], actual: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (&e, &a)) in expected.iter().zip(actual).enumerate() {
        eprintln!("value {index}: actual {a}, expected {e}");
        approx(a, e, 0.01);
    }
}
#[derive(Clone, Copy, Debug)]
enum Sizing {
    Fixed,
    Hug,
    FlexParent,
}
struct Grid<'a> {
    rows: Option<&'a [Track]>,
    columns: Option<&'a [Track]>,
    column_count: usize,
    widths: Vec<f32>,
    heights: Vec<f32>,
    gap: f32,
    sizing: Sizing,
    vertical: bool,
}
impl Default for Grid<'_> {
    fn default() -> Self {
        Self {
            rows: None,
            columns: None,
            column_count: 1,
            widths: vec![],
            heights: vec![],
            gap: 0.0,
            sizing: Sizing::Fixed,
            vertical: true,
        }
    }
}
fn solve(g: &Grid<'_>, realized: &[bool]) -> Vec<f32> {
    let mut t = tree();
    let mut s = Style {
        display: Display::Grid,
        gap: Size {
            width: length(g.gap),
            height: length(g.gap),
        },
        ..yoga_style()
    };
    if let Some(rows) = g.rows {
        s.grid_template_rows = tracks(rows);
    }
    if let Some(columns) = g.columns {
        s.grid_template_columns = tracks(columns);
    }
    let space = if g.vertical { 400.0 } else { 300.0 };
    let across = if g.vertical { 300.0 } else { 400.0 };
    if g.vertical {
        s.size.width = length(across);
    } else {
        s.size.height = length(across);
    }
    if matches!(g.sizing, Sizing::Fixed) {
        if g.vertical {
            s.size.height = length(space);
        } else {
            s.size.width = length(space);
        }
    }
    let mut row_sizes = vec![0.0_f32; (g.widths.len() + g.column_count - 1) / g.column_count];
    let mut column_sizes = vec![0.0_f32; g.column_count];
    let mut children = Vec::new();
    for i in 0..g.widths.len() {
        let row = i / g.column_count;
        let column = i % g.column_count;
        row_sizes[row] = row_sizes[row].max(g.heights[i]);
        column_sizes[column] = column_sizes[column].max(g.widths[i]);
        if !realized[i] {
            continue;
        }
        let item = Style {
            size: Size {
                width: length(g.widths[i]),
                height: length(g.heights[i]),
            },
            grid_row: Line {
                start: GridPlacement::from_line_index((row + 1) as i32),
                end: GridPlacement::Auto,
            },
            grid_column: Line {
                start: GridPlacement::from_line_index((column + 1) as i32),
                end: GridPlacement::Auto,
            },
            ..yoga_style()
        };
        children.push(t.new_leaf(item).unwrap());
    }
    let node = t.new_with_children(s, &children).unwrap();
    if realized.iter().any(|&r| !r) {
        t.set_grid_virtual_contributions(node, &row_sizes, &column_sizes)
            .unwrap();
    }
    let root = if matches!(g.sizing, Sizing::FlexParent) {
        t.new_with_children(
            Style {
                flex_direction: if g.vertical {
                    FlexDirection::Column
                } else {
                    FlexDirection::Row
                },
                ..yoga_style()
            },
            &[node],
        )
        .unwrap()
    } else {
        node
    };
    calculate(&mut t, root);
    let mut out = lines(&t, node, true);
    out.push(-1.0);
    out.extend(lines(&t, node, false));
    let size = t.layout(node).unwrap().size;
    out.extend([size.width, size.height]);
    out
}
fn check_grid(g: &Grid<'_>) {
    let count = g.widths.len();
    let expected = solve(g, &vec![true; count]);
    let mut some = vec![false; count];
    for i in count / 3..(count / 3 + g.column_count).min(count) {
        some[i] = true;
    }
    eprintln!("some realized");
    check_same(&expected, &solve(g, &some));
    eprintln!("none realized");
    check_same(&expected, &solve(g, &vec![false; count]));
}
const ITEM_SIZES: [f32; 8] = [150.0, 50.0, 80.0, 200.0, 20.0, 100.0, 80.0, 150.0];
#[test]
fn virtual_contributions_size_rows_as_items_would() {
    for (index, rows) in ROW_TRACKS.iter().enumerate() {
        for sizing in [Sizing::Fixed, Sizing::Hug, Sizing::FlexParent] {
            eprintln!("row case {index} sizing {sizing:?}");
            let count = rows.len() + 3;
            let g = Grid {
                rows: Some(rows),
                gap: (index % 2) as f32 * 10.0,
                sizing,
                widths: vec![100.0; count],
                heights: (0..count).map(|i| ITEM_SIZES[(index + i) % 8]).collect(),
                ..Default::default()
            };
            check_grid(&g);
        }
    }
}
#[test]
fn virtual_contributions_size_columns_as_items_would() {
    for (index, columns) in COLUMN_TRACKS.iter().enumerate() {
        for sizing in [Sizing::Fixed, Sizing::Hug, Sizing::FlexParent] {
            eprintln!("column case {index} sizing {sizing:?}");
            let count = columns.len() * 3;
            let g = Grid {
                columns: Some(columns),
                column_count: columns.len(),
                gap: (index % 2) as f32 * 10.0,
                sizing,
                vertical: false,
                widths: (0..count).map(|i| ITEM_SIZES[(index + i) % 8]).collect(),
                heights: vec![50.0; count],
                ..Default::default()
            };
            check_grid(&g);
        }
    }
}
#[test]
fn virtual_contributions_extend_implicit_grid() {
    let mut t = tree();
    let node = t.new_leaf(grid_style(200.0)).unwrap();
    assert!(
        t.set_grid_virtual_contributions(node, &[10.0, 20.0, 30.0], &[50.0, 60.0])
            .unwrap()
    );
    assert!(
        !t.set_grid_virtual_contributions(node, &[10.0, 20.0, 30.0], &[50.0, 60.0])
            .unwrap()
    );
    calculate(&mut t, node);
    assert_eq!(lines(&t, node, true).len(), 4);
    assert_eq!(lines(&t, node, false).len(), 3);
    approx(t.layout(node).unwrap().size.height, 60.0, 0.0);
    assert!(t.set_grid_virtual_contributions(node, &[], &[]).unwrap());
    calculate(&mut t, node);
    approx(t.layout(node).unwrap().size.height, 0.0, 0.0);
}
#[test]
fn virtual_sizes_reused_only_while_inputs_hold() {
    // Each source SECTION starts from a fresh node.
    for section in 0..4 {
        let mut t = tree();
        let mut style = grid_style(300.0);
        style.grid_template_columns = tracks(&[(F, 1.0, F, 1.0), (A, 0.0, A, 0.0)]);
        let mut item_style = yoga_style();
        item_style.size.height = length(40.0);
        item_style.grid_row.start = GridPlacement::from_line_index(1);
        item_style.grid_column.start = GridPlacement::from_line_index(1);
        let item = t.new_leaf(item_style).unwrap();
        let node = t.new_with_children(style, &[item]).unwrap();
        let mut rows = vec![40.0, 50.0, 60.0, 70.0, 80.0];
        let columns = [100.0, 90.0];
        t.set_grid_virtual_contributions(node, &rows, &columns)
            .unwrap();
        calculate(&mut t, node);
        let before_rows = lines(&t, node, true);
        let before_columns = lines(&t, node, false);
        assert_eq!(before_rows, vec![0.0, 40.0, 90.0, 150.0, 220.0, 300.0]);
        match section {
            0 => {
                let mut s = t.style(item).unwrap().clone();
                s.grid_row.start = GridPlacement::from_line_index(4);
                t.set_style(item, s).unwrap();
                calculate(&mut t, node);
                assert_eq!(lines(&t, node, true), before_rows);
                assert_eq!(lines(&t, node, false), before_columns);
                approx(t.layout(item).unwrap().location.y, 150.0, 0.0);
            }
            1 => {
                rows[1] = 100.0;
                t.set_grid_virtual_contributions(node, &rows, &columns)
                    .unwrap();
                calculate(&mut t, node);
                assert_eq!(
                    lines(&t, node, true),
                    vec![0.0, 40.0, 140.0, 200.0, 270.0, 350.0]
                );
            }
            2 => {
                let mut s = t.style(node).unwrap().clone();
                s.gap.height = length(10.0);
                t.set_style(node, s).unwrap();
                calculate(&mut t, node);
                assert_eq!(
                    lines(&t, node, true),
                    vec![0.0, 50.0, 110.0, 180.0, 260.0, 340.0]
                );
            }
            _ => {
                let mut s = t.style(node).unwrap().clone();
                s.size.width = length(400.0);
                t.set_style(node, s).unwrap();
                calculate(&mut t, node);
                assert_eq!(lines(&t, node, false), vec![0.0, 310.0, 400.0]);
            }
        }
    }
}
#[test]
fn virtual_contributions_copy_with_node() {
    let mut t = tree();
    let node = t.new_leaf(grid_style(100.0)).unwrap();
    t.set_grid_virtual_contributions(node, &[10.0, 20.0], &[])
        .unwrap();
    calculate(&mut t, node);
    // This one-node tree clone deep-copies the node and its retained contribution vectors.
    let mut cloned = t.clone();
    t.set_grid_virtual_contributions(node, &[50.0, 50.0], &[])
        .unwrap();
    calculate(&mut t, node);
    let mut s = cloned.style(node).unwrap().clone();
    s.size.width = length(101.0);
    cloned.set_style(node, s).unwrap();
    calculate(&mut cloned, node);
    approx(t.layout(node).unwrap().size.height, 100.0, 0.0);
    approx(cloned.layout(node).unwrap().size.height, 30.0, 0.0);
}
#[test]
fn virtual_sizes_follow_child_into_implicit_grid_before() {
    let mut t = tree();
    let mut s = grid_style(300.0);
    s.size.height = length(100.0);
    let mut item_style = yoga_style();
    item_style.grid_column.start = GridPlacement::from_line_index(3);
    let item = t.new_leaf(item_style).unwrap();
    let node = t.new_with_children(s, &[item]).unwrap();
    t.set_grid_virtual_contributions(node, &[], &[100.0, 10.0])
        .unwrap();
    calculate(&mut t, node);
    let mut s = t.style(item).unwrap().clone();
    s.grid_column.start = GridPlacement::Auto;
    s.grid_column.end = GridPlacement::from_line_index(1);
    t.set_style(item, s).unwrap();
    calculate(&mut t, node);
    assert_eq!(lines(&t, node, false), vec![0.0, 0.0, 100.0, 110.0]);
}
#[test]
fn nonfinite_virtual_contributions_count_as_zero() {
    let mut t = tree();
    let node = t.new_leaf(grid_style(50.0)).unwrap();
    assert!(
        t.set_grid_virtual_contributions(node, &[f32::NAN, 30.0], &[])
            .unwrap()
    );
    assert!(
        !t.set_grid_virtual_contributions(node, &[f32::NAN, 30.0], &[])
            .unwrap()
    );
    calculate(&mut t, node);
    assert_eq!(lines(&t, node, true), vec![0.0, 0.0, 30.0]);
    t.set_grid_virtual_contributions(node, &[f32::INFINITY, 30.0], &[])
        .unwrap();
    calculate(&mut t, node);
    assert_eq!(lines(&t, node, true), vec![0.0, 0.0, 30.0]);
    approx(t.layout(node).unwrap().size.height, 30.0, 0.0);
}
#[test]
fn grid_records_content_distribution_gap() {
    let mut t = tree();
    let mut s = grid_style(100.0);
    s.size.height = length(300.0);
    s.align_content = Some(AlignContent::SPACE_BETWEEN);
    s.gap.height = length(10.0);
    let node = t.new_leaf(s).unwrap();
    t.set_grid_virtual_contributions(node, &[50.0, 50.0], &[])
        .unwrap();
    calculate(&mut t, node);
    let DetailedLayoutInfo::Grid(info) = t.detailed_layout_info(node) else {
        panic!("grid layout");
    };
    approx(info.rows.gap, 200.0, 0.0);
    assert_eq!(lines(&t, node, true), vec![0.0, 250.0, 300.0]);
}

// Source-derived boundary regressions: pinned Yoga's ResolvedAutoPlacement
// extends the grid with int32_t contribution counts, and its explicit grid
// lines remain int32_t. Neither nodeless tracks nor realized cells truncate at
// the former Taffy u16 track-count / i16 line boundaries.
#[test]
fn virtual_contributions_preserve_large_nodeless_track_counts() {
    for rows in [true, false] {
        for count in [65_536usize, 65_537] {
            let mut t = tree();
            let node = t
                .new_leaf(Style {
                    display: Display::Grid,
                    ..yoga_style()
                })
                .unwrap();
            let contributions = vec![1.0; count];
            let (row_sizes, column_sizes): (&[f32], &[f32]) = if rows {
                (&contributions, &[1.0])
            } else {
                (&[1.0], &contributions)
            };
            t.set_grid_virtual_contributions(node, row_sizes, column_sizes)
                .unwrap();
            calculate(&mut t, node);
            let offsets = lines(&t, node, rows);
            assert_eq!(offsets.len(), count + 1);
            for (index, offset) in offsets.into_iter().enumerate() {
                assert_eq!(offset, index as f32);
            }
            let size = t.layout(node).unwrap().size;
            assert_eq!(if rows { size.height } else { size.width }, count as f32);
        }
    }
}

#[test]
fn realized_virtual_item_preserves_large_grid_cell_indices() {
    const COUNT: usize = 65_537;
    const INDEX: usize = COUNT - 1;
    for rows in [true, false] {
        let mut t = tree();
        let placement = GridPlacement::from_line_index((INDEX + 1) as i32);
        let item = t
            .new_leaf(Style {
                size: Size {
                    width: length(1.0),
                    height: length(1.0),
                },
                grid_row: Line {
                    start: if rows {
                        placement.clone()
                    } else {
                        GridPlacement::from_line_index(1)
                    },
                    end: GridPlacement::Auto,
                },
                grid_column: Line {
                    start: if rows {
                        GridPlacement::from_line_index(1)
                    } else {
                        placement
                    },
                    end: GridPlacement::Auto,
                },
                ..yoga_style()
            })
            .unwrap();
        let node = t
            .new_with_children(
                Style {
                    display: Display::Grid,
                    ..yoga_style()
                },
                &[item],
            )
            .unwrap();
        let contributions = vec![1.0; COUNT];
        let (row_sizes, column_sizes): (&[f32], &[f32]) = if rows {
            (&contributions, &[1.0])
        } else {
            (&[1.0], &contributions)
        };
        t.set_grid_virtual_contributions(node, row_sizes, column_sizes)
            .unwrap();
        calculate(&mut t, node);
        let offsets = lines(&t, node, rows);
        assert_eq!(offsets.len(), COUNT + 1);
        assert_eq!(offsets[INDEX], INDEX as f32);
        assert_eq!(offsets[COUNT], COUNT as f32);
        let item_layout = t.layout(item).unwrap();
        assert_eq!(
            if rows {
                item_layout.location.y
            } else {
                item_layout.location.x
            },
            INDEX as f32
        );
        let size = t.layout(node).unwrap().size;
        assert_eq!(if rows { size.height } else { size.width }, COUNT as f32);
    }
}
