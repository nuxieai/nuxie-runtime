//! CSS mode (docs/CSS_LAYOUT.md): a box with a `LayoutCssStyle` record is
//! laid out by its CSS, and a box without one by the CSS its Rive modes
//! translate to. The oracle is CSS arithmetic, as a browser computes it.
use super::layout_measured_content_tests::Riv;
use crate::source::{
    generated::{
        artboard_base::ArtboardBase,
        backboard_base::BackboardBase,
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        layout::{
            layout_component_style_base::LayoutComponentStyleBase,
            layout_sizing_style_base::LayoutSizingStyleBase,
        },
        layout_component_base::LayoutComponentBase,
        node_base::NodeBase,
        shapes::{
            paint::{
                feather_base::FeatherBase, fill_base::FillBase,
                gradient_stop_base::GradientStopBase, linear_gradient_base::LinearGradientBase,
                solid_color_base::SolidColorBase, stroke_base::StrokeBase,
            },
            parametric_path_base::ParametricPathBase,
            rectangle_base::RectangleBase,
            shape_base::ShapeBase,
        },
    },
    layout::{
        layout_enums::LayoutScaleType,
        layout_style_applier::{YGFlexDirection, YGUnit},
    },
    layout_component::LayoutComponent,
};
use crate::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

const FRAME: f32 = 1.0 / 60.0;

/// The record type and the keys these tests write (defs/nuxie/layout_css_style.json).
const LAYOUT_CSS_STYLE: u16 = 61000;
const DISPLAY: u16 = 61000;
const BOX_SIZING: u16 = 61001;
const POSITION: u16 = 61002;
const FLEX_GROW: u16 = 61008;
const FLEX_SHRINK: u16 = 61009;
const WIDTH_KIND: u16 = 61021;
const WIDTH: u16 = 61022;
const WIDTH_PERCENT: u16 = 61023;
const HEIGHT_KIND: u16 = 61024;
const HEIGHT: u16 = 61025;
const MARGIN_TOP: u16 = 61040;
const MARGIN_BOTTOM: u16 = 61046;
const MARGIN_LEFT: u16 = 61049;
const INSET_TOP_KIND: u16 = 61051;
const INSET_TOP: u16 = 61052;
const INSET_LEFT_KIND: u16 = 61060;
const INSET_LEFT: u16 = 61061;
const PADDING_TOP: u16 = 61063;
const PADDING_RIGHT: u16 = 61065;
const PADDING_BOTTOM: u16 = 61067;
const PADDING_LEFT: u16 = 61069;
const BORDER_TOP: u16 = 61071;
const BORDER_RIGHT: u16 = 61072;
const BORDER_BOTTOM: u16 = 61073;
const BORDER_LEFT: u16 = 61074;
const COLUMN_GAP: u16 = 61077;

const FLEX: f32 = 1.0;
const NONE: f32 = 3.0;
const LENGTH: f32 = 1.0;

/// One property: a uint for enums and kinds, a float for numbers.
#[derive(Clone, Copy)]
enum P {
    U(u16, u64),
    F(u16, f32),
}
use P::{F, U};

fn width(value: f32) -> [P; 2] {
    [U(WIDTH_KIND, 1), F(WIDTH, value)]
}
fn height(value: f32) -> [P; 2] {
    [U(HEIGHT_KIND, 1), F(HEIGHT, value)]
}
fn padding(value: f32) -> [P; 4] {
    [
        F(PADDING_TOP, value),
        F(PADDING_RIGHT, value),
        F(PADDING_BOTTOM, value),
        F(PADDING_LEFT, value),
    ]
}

/// A layout box: CSS mode when `css` is Some, translated otherwise.
#[derive(Clone, Default)]
struct B {
    name: &'static str,
    css: Option<Vec<P>>,
    /// Translated mode: `LayoutComponent` keys (width, height, fractions).
    component: Vec<P>,
    /// Transform and opacity properties on the box itself.
    component_keys: Vec<(u16, Vec<(u32, f32)>)>,
    component_bind: Option<u16>,
    component_bind_flags: u64,
    /// A LayerMask on this box, referring to another box by name.
    mask_source: Option<&'static str>,
    /// Translated mode: `LayoutComponentStyle` keys.
    style: Vec<P>,
    children: Vec<B>,
    /// Drawing content: rectangles, each a Shape with one Rectangle path.
    content: Vec<Rect>,
    /// A ScrollConstraint on this box (its content box), with its direction
    /// (0 horizontal, 1 vertical); its parent is the viewport.
    scroll: Option<u64>,
    /// Participants: a rectangle shape hosting a LayoutParticipant and,
    /// when Some, a LayoutCssStyle on the host.
    participants: Vec<(Rect, Option<Vec<P>>)>,
    /// Rive GridTracks on this box: (collection, track type, value).
    tracks: Vec<(u64, u64, f32)>,
    /// Images of the file's one 100 by 30 image asset, by name.
    images: Vec<&'static str>,
    /// The box's own paints, in draw order.
    paints: Vec<Paint>,
    /// Shapes of several rectangle paths: the shape's name and each path's
    /// x position and size.
    multi_path: Vec<(&'static str, Vec<(f32, (f32, f32))>)>,
    /// A data binding from a view model number to this record property.
    bind: Option<u16>,
    /// The view model number's index for `bind`.
    bind_source: u8,
    /// A BoxPaint record and the values it writes.
    box_paint: Option<Vec<P>>,
    /// A data binding from the view model number to a BoxPaint property.
    box_paint_bind: Option<u16>,
    /// BoxPaint properties keyed by the file's one animation: (key, frames
    /// and values at 60 frames a second, linear).
    box_paint_keys: Vec<(u16, Vec<(u32, f32)>)>,
    /// An empty Text beside each grouped content rectangle, in its group.
    text_in_groups: bool,
    /// LayoutCssStyle properties keyed by the file's one animation, as
    /// `box_paint_keys`.
    css_keys: Vec<(u16, Vec<(u32, f32)>)>,
    /// Children written after every other box, as runtime materialization
    /// (a text input's hint and value) appends them.
    late_children: Vec<B>,
    /// A pointer click listener on this box, in the file's one state
    /// machine, firing an event of this name.
    listener: Option<&'static str>,
    /// A pointer drag listener on this box, firing an event of this name.
    drag_listener: Option<&'static str>,
    /// A pointer enter listener on this box, firing an event of this name.
    enter_listener: Option<&'static str>,
    /// A pointer down listener on this box, firing an event of this name.
    down_listener: Option<&'static str>,
    /// A nested artboard as this box's content, of the file's second
    /// artboard: 200 by 40, a translated 180 by 40 box holding a field, with
    /// a state machine.
    nested_field: bool,
    /// As `nested_field`, the nested artboard holding a translated
    /// horizontal scroll view instead: a 200 by 40 viewport scrolling a 400
    /// point list.
    nested_scroller: bool,
    /// A text input as this box's content, focusable through its FocusData.
    text_input: bool,
    /// The text input's text.
    text: &'static str,
    /// The ScrollConstraint's threshold, the distance a press moves before
    /// it scrolls.
    scroll_threshold: f32,
    scroll_properties: Vec<P>,
    scroll_keys: Vec<(u16, Vec<(u32, f32)>)>,
    scroll_bind: Option<(u16, u64)>,
    /// A ScrollBarConstraint on this box, the thumb, with its direction,
    /// for the file's first ScrollConstraint; the box's parent is the track.
    scroll_bar: Option<u64>,
    scroll_bar_auto_size: bool,
}

/// A paint on a box, with its CSS additions (docs/CSS_LAYOUT.md, 4).
#[derive(Clone, Copy)]
enum Paint {
    /// A Fill with a SolidColor.
    Fill(u32),
    /// A Stroke with a SolidColor, unaffected by the world transform.
    WorldStroke(u32),
    /// A Stroke with a SolidColor and a BoxBorder over `sides`.
    Border(u32, u64),
    /// As `Border`, with the BoxBorder's style (1 dashed, 2 dotted).
    StyledBorder(u32, u64, u64),
    /// A shadow: a Fill with a SolidColor and a local Feather of
    /// `strength`, whose BoxShadow has `spread`.
    Shadow {
        color: u32,
        strength: f32,
        spread: f32,
    },
    /// A Fill with a two-stop LinearGradient whose BoxGradient has `angle`.
    Gradient(f32),
    /// An inset shadow: as `Shadow`, its Feather inner.
    InsetShadow {
        color: u32,
        strength: f32,
        spread: f32,
    },
}

const BOX_BORDER: u16 = 61001;
const BOX_GRADIENT: u16 = 61002;
const BOX_SHADOW: u16 = 61003;
const OVERFLOW_X: u16 = 61003;
const OVERFLOW_Y: u16 = 61004;
const DIRECTION: u16 = 61005;
const BORDER_SIDES: u16 = 61083;
const GRADIENT_ANGLE: u16 = 61084;
const SHADOW_SPREAD: u16 = 61096;
const BOX_PAINT: u16 = 61004;
const PAINT_WIDTH: u16 = 61097;
const PAINT_HEIGHT: u16 = 61098;
const PAINT_WIDTH_PERCENT: u16 = 61101;
const PAINT_HEIGHT_PERCENT: u16 = 61102;

/// BoxPaint values for a size in points alone: both percentages 0.
fn paint_points(width: f32, height: f32) -> Vec<P> {
    vec![
        F(PAINT_WIDTH, width),
        F(PAINT_WIDTH_PERCENT, 0.0),
        F(PAINT_HEIGHT, height),
        F(PAINT_HEIGHT_PERCENT, 0.0),
    ]
}

/// A rectangle shape named `name` (its path is `name` + " path"), authored
/// at `size`, directly in its box or inside a plain group.
#[derive(Clone, Copy)]
struct Rect {
    name: &'static str,
    size: (f32, f32),
    grouped: bool,
    /// The path's own horizontal scale inside its shape.
    path_scale_x: f32,
}

fn css(name: &'static str, props: impl IntoIterator<Item = P>, children: Vec<B>) -> B {
    B {
        name,
        css: Some(props.into_iter().collect()),
        children,
        ..B::default()
    }
}

/// Every unit a translated style leaves undefined is points, as the Rive
/// editor writes them.
fn point_units() -> Vec<P> {
    [
        LayoutComponentStyleBase::MARGIN_LEFT_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::MARGIN_RIGHT_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::MARGIN_TOP_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::MARGIN_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::PADDING_LEFT_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::PADDING_RIGHT_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::PADDING_TOP_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::PADDING_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::GAP_HORIZONTAL_UNITS_VALUE_PROPERTY_KEY,
        LayoutComponentStyleBase::GAP_VERTICAL_UNITS_VALUE_PROPERTY_KEY,
    ]
    .into_iter()
    .map(|key| U(key, YGUnit::Point as u64))
    .collect()
}

struct Scene {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    factory: PersistentFactory<RecordingFactory>,
}

enum Entry<'a> {
    Box(&'a B, u64),
    /// A CSS record of a type with its properties.
    Record(u16, Vec<P>, u64),
    Fill(u64),
    Stroke(u64),
    Solid(u32, u64),
    /// A Feather of a strength, inner when true, on a paint.
    Feather(f32, bool, u64),
    LinearGradient(u64),
    Stop(u32, f32, u64),
    NamedShape(&'static str, u64),
    Bind(u16, u64, u8),
    /// A record property keyed by the animation: the record's object index.
    Keys(u64, u16, &'a [(u32, f32)]),
    PathAt(f32, (f32, f32), u64),
    Group(u64),
    Shape(&'a Rect, u64),
    Path(&'a Rect, u64),
    /// A ScrollConstraint on a box: its direction and threshold.
    Scroll(u64, u64, &'a B),
    /// A ScrollBarConstraint on a box with its direction.
    ScrollBar(u64, u64, bool),
    Participant(u64),
    Track((u64, u64, f32), u64),
    Image(&'static str, u64),
    /// A Text with no runs.
    EmptyText(u64),
    /// A TextInput named "Field" with its text, and the FocusData and text
    /// style in it.
    TextInput(u64, &'static str),
    FocusData(u64),
    TextStyle(u64),
    /// An Event of a name, in the artboard.
    Event(&'static str),
    /// A NestedArtboard of the second artboard, and its NestedStateMachine.
    Nested(u64),
    NestedMachine(u64),
}

/// A box's (or the artboard's) paints and their CSS records.
fn push_paints(paints: &[Paint], index: u64, entries: &mut Vec<Entry<'_>>) {
    for paint in paints {
        let paint_index = slots(entries) + 1;
        match *paint {
            Paint::WorldStroke(color) => {
                entries.push(Entry::Record(
                    StrokeBase::TYPE_KEY,
                    vec![
                        F(StrokeBase::THICKNESS_PROPERTY_KEY, 4.0),
                        U(StrokeBase::TRANSFORM_AFFECTS_STROKE_PROPERTY_KEY, 0),
                    ],
                    index,
                ));
                entries.push(Entry::Solid(color, paint_index));
            }
            Paint::Fill(color) => {
                entries.push(Entry::Fill(index));
                entries.push(Entry::Solid(color, paint_index));
            }
            Paint::Border(color, sides) => {
                entries.push(Entry::Stroke(index));
                entries.push(Entry::Solid(color, paint_index));
                entries.push(Entry::Record(
                    BOX_BORDER,
                    vec![U(BORDER_SIDES, sides)],
                    paint_index,
                ));
            }
            Paint::StyledBorder(color, sides, style) => {
                entries.push(Entry::Stroke(index));
                entries.push(Entry::Solid(color, paint_index));
                entries.push(Entry::Record(
                    BOX_BORDER,
                    vec![
                        U(BORDER_SIDES, sides),
                        U(crate::css_layout::keys::BORDER_STYLE, style),
                    ],
                    paint_index,
                ));
            }
            Paint::Shadow {
                color,
                strength,
                spread,
            }
            | Paint::InsetShadow {
                color,
                strength,
                spread,
            } => {
                let inner = matches!(paint, Paint::InsetShadow { .. });
                entries.push(Entry::Fill(index));
                entries.push(Entry::Solid(color, paint_index));
                entries.push(Entry::Feather(strength, inner, paint_index));
                entries.push(Entry::Record(
                    BOX_SHADOW,
                    vec![F(SHADOW_SPREAD, spread)],
                    paint_index + 2,
                ));
            }
            Paint::Gradient(angle) => {
                entries.push(Entry::Fill(index));
                entries.push(Entry::LinearGradient(paint_index));
                entries.push(Entry::Stop(0xff00_00ff, 0.0, paint_index + 1));
                entries.push(Entry::Stop(0xffff_0000, 1.0, paint_index + 1));
                entries.push(Entry::Record(
                    BOX_GRADIENT,
                    vec![F(GRADIENT_ANGLE, angle)],
                    paint_index + 1,
                ));
            }
        }
    }
}

/// The artboard object slots `entries` take: a data binding takes none.
fn slots(entries: &[Entry]) -> u64 {
    entries
        .iter()
        .filter(|entry| !matches!(entry, Entry::Bind(..) | Entry::Keys(..)))
        .count() as u64
}

fn write(riv: &mut Riv, props: &[P]) {
    for prop in props {
        match *prop {
            U(key, value) => riv.uint(key, value),
            F(key, value) => riv.float(key, value),
        };
    }
}

/// Imports one artboard of `size` holding `root`, or None when the file is
/// refused. `artboard_css` puts a record on the artboard itself.
fn try_scene(size: (f32, f32), artboard_css: Option<Vec<P>>, root: B) -> Option<Scene> {
    try_scene_painted(size, artboard_css, &[], root)
}

/// As [`try_scene`], with the artboard's own paints.
fn try_scene_painted(
    size: (f32, f32),
    artboard_css: Option<Vec<P>>,
    artboard_paints: &[Paint],
    root: B,
) -> Option<Scene> {
    try_scene_spec(
        size,
        ArtboardSpec {
            css: artboard_css,
            paints: artboard_paints.to_vec(),
            ..ArtboardSpec::default()
        },
        Some(root),
    )
}

/// The artboard itself: its record, paints, direct content and origin.
#[derive(Default)]
struct ArtboardSpec {
    css: Option<Vec<P>>,
    /// The artboard's own LayoutComponentStyle keys.
    style: Vec<P>,
    paints: Vec<Paint>,
    content: Vec<Rect>,
    origin_x: f32,
    /// The artboard's BoxPaint and the values it writes.
    box_paint: Option<Vec<P>>,
    /// The artboard does not clip its content.
    unclipped: bool,
}

/// Imports one artboard of `size` described by `spec`, holding `root` when
/// there is one, or None when the file is refused.
fn try_scene_spec(size: (f32, f32), spec: ArtboardSpec, root: Option<B>) -> Option<Scene> {
    let artboard_css = spec.css.clone();
    let artboard_paints = spec.paints.as_slice();
    fn walk<'a>(
        node: &'a B,
        parent: u64,
        entries: &mut Vec<Entry<'a>>,
        late: &mut Vec<(&'a B, u64)>,
    ) {
        let index = slots(entries) + 1;
        entries.push(Entry::Box(node, parent));
        if let Some(key) = node.component_bind {
            entries.push(Entry::Bind(key, node.component_bind_flags, 0));
        }
        for (key, keys) in &node.component_keys {
            entries.push(Entry::Keys(index, *key, keys));
        }
        if let Some(props) = &node.css {
            let record = slots(entries) + 1;
            entries.push(Entry::Record(LAYOUT_CSS_STYLE, props.clone(), index));
            if let Some(key) = node.bind {
                entries.push(Entry::Bind(key, 0, node.bind_source));
            }
            for (key, keys) in &node.css_keys {
                entries.push(Entry::Keys(record, *key, keys));
            }
        }
        if let Some(props) = &node.box_paint {
            let record = slots(entries) + 1;
            entries.push(Entry::Record(BOX_PAINT, props.clone(), index));
            if let Some(key) = node.box_paint_bind {
                entries.push(Entry::Bind(key, 0, 0));
            }
            for (key, keys) in &node.box_paint_keys {
                entries.push(Entry::Keys(record, *key, keys));
            }
        }
        push_paints(&node.paints, index, entries);
        if let Some(direction) = node.scroll {
            entries.push(Entry::Scroll(index, direction, node));
            let scroll = slots(entries);
            for (key, keys) in &node.scroll_keys {
                entries.push(Entry::Keys(scroll, *key, keys));
            }
            if let Some((key, flags)) = node.scroll_bind {
                entries.push(Entry::Bind(key, flags, 0));
            }
        }
        if let Some(direction) = node.scroll_bar {
            entries.push(Entry::ScrollBar(
                index,
                direction,
                node.scroll_bar_auto_size,
            ));
        }
        for track in &node.tracks {
            entries.push(Entry::Track(*track, index));
        }
        for image in &node.images {
            entries.push(Entry::Image(image, index));
        }
        for (rect, props) in &node.participants {
            entries.push(Entry::Shape(rect, index));
            let host = slots(entries);
            entries.push(Entry::Path(rect, host));
            entries.push(Entry::Participant(host));
            if let Some(props) = props {
                entries.push(Entry::Record(LAYOUT_CSS_STYLE, props.clone(), host));
            }
        }
        for (name, paths) in &node.multi_path {
            entries.push(Entry::NamedShape(name, index));
            let shape = slots(entries);
            for &(x, size) in paths {
                entries.push(Entry::PathAt(x, size, shape));
            }
        }
        for rect in &node.content {
            let mut parent = index;
            if rect.grouped {
                entries.push(Entry::Group(index));
                parent = slots(entries);
            }
            entries.push(Entry::Shape(rect, parent));
            entries.push(Entry::Path(rect, slots(entries)));
            if rect.grouped && node.text_in_groups {
                entries.push(Entry::EmptyText(parent));
            }
        }
        if node.nested_field || node.nested_scroller {
            entries.push(Entry::Nested(index));
            entries.push(Entry::NestedMachine(slots(entries)));
        }
        if node.text_input {
            entries.push(Entry::TextInput(index, node.text));
            let input = slots(entries);
            entries.push(Entry::FocusData(input));
            entries.push(Entry::TextStyle(input));
        }
        for child in &node.children {
            walk(child, index, entries, late);
        }
        late.extend(node.late_children.iter().map(|child| (child, index)));
    }
    let mut entries = Vec::new();
    if let Some(props) = &artboard_css {
        entries.push(Entry::Record(LAYOUT_CSS_STYLE, props.clone(), 0));
    }
    if let Some(props) = &spec.box_paint {
        entries.push(Entry::Record(BOX_PAINT, props.clone(), 0));
    }
    push_paints(artboard_paints, 0, &mut entries);
    for rect in &spec.content {
        entries.push(Entry::Shape(rect, 0));
        entries.push(Entry::Path(rect, slots(&entries)));
    }
    if let Some(root) = &root {
        let mut late = Vec::new();
        walk(root, 0, &mut entries, &mut late);
        while !late.is_empty() {
            for (child, parent) in std::mem::take(&mut late) {
                walk(child, parent, &mut entries, &mut late);
            }
        }
    }
    // Resolve mask sources after every box has its object index, including
    // sources written after their masked subtree.
    let mut masks = Vec::new();
    let mut box_ids = Vec::new();
    let mut index = 0u64;
    for entry in &entries {
        if matches!(entry, Entry::Bind(..) | Entry::Keys(..)) {
            continue;
        }
        index += 1;
        if let Entry::Box(node, _) = entry {
            box_ids.push((node.name, index));
            if let Some(source) = node.mask_source {
                masks.push((index, source));
            }
        }
    }
    for (parent, source) in masks {
        use crate::source::generated::layer_mask_base::LayerMaskBase;
        let source = box_ids
            .iter()
            .find(|(name, _)| *name == source)
            .expect("the mask source box")
            .1;
        entries.push(Entry::Record(
            LayerMaskBase::TYPE_KEY,
            vec![U(LayerMaskBase::SOURCE_ID_PROPERTY_KEY, source)],
            parent,
        ));
    }
    // Each listened box's object and its event's, which follows the boxes.
    let mut listened = Vec::new();
    let mut index = 0u64;
    for entry in &entries {
        if matches!(entry, Entry::Bind(..) | Entry::Keys(..)) {
            continue;
        }
        index += 1;
        if let Entry::Box(node, _) = entry {
            use crate::source::listener_type::ListenerType;
            for (kind, event) in [
                (ListenerType::Click, node.listener),
                (ListenerType::Drag, node.drag_listener),
                (ListenerType::Enter, node.enter_listener),
                (ListenerType::Down, node.down_listener),
            ] {
                if let Some(event) = event {
                    listened.push((index, kind, event));
                }
            }
        }
    }
    // The first ScrollConstraint's object, which a scroll bar refers to.
    let mut first_scroll = 0u64;
    let mut index = 0u64;
    for entry in &entries {
        if matches!(entry, Entry::Bind(..) | Entry::Keys(..)) {
            continue;
        }
        index += 1;
        if matches!(entry, Entry::Scroll(..)) {
            first_scroll = index;
            break;
        }
    }
    let mut listeners = Vec::new();
    for (target, kind, event) in listened {
        entries.push(Entry::Event(event));
        listeners.push((target, kind, slots(&entries)));
    }
    // Styles follow every other object: the artboard's, then one per box.
    let first_style = slots(&entries) + 1;
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    let binds = entries.iter().any(|entry| matches!(entry, Entry::Bind(..)));
    if binds {
        // One view model with number properties: paths [0, index].
        use crate::source::generated::viewmodel::{
            viewmodel_base::ViewModelBase, viewmodel_component_base::ViewModelComponentBase,
            viewmodel_property_number_base::ViewModelPropertyNumberBase,
        };
        riv.object(ViewModelBase::TYPE_KEY)
            .string(ViewModelComponentBase::NAME_PROPERTY_KEY, "Model")
            .end();
        let last_source = entries
            .iter()
            .filter_map(|entry| match entry {
                Entry::Bind(_, _, source) => Some(*source),
                _ => None,
            })
            .max()
            .unwrap();
        for source in 0..=last_source {
            riv.object(ViewModelPropertyNumberBase::TYPE_KEY)
                .string(
                    ViewModelComponentBase::NAME_PROPERTY_KEY,
                    if source == 0 { "inset" } else { "display" },
                )
                .end();
        }
    }
    let pictured = entries
        .iter()
        .any(|entry| matches!(entry, Entry::Image(..)));
    if pictured {
        // Its asset has no bytes; an image takes the asset's size.
        use crate::source::generated::assets::{
            drawable_asset_base::DrawableAssetBase, file_asset_base::FileAssetBase,
            image_asset_base::ImageAssetBase,
        };
        riv.object(ImageAssetBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, "picture")
            .uint(FileAssetBase::ASSET_ID_PROPERTY_KEY, 1)
            .float(DrawableAssetBase::WIDTH_PROPERTY_KEY, 100.0)
            .float(DrawableAssetBase::HEIGHT_PROPERTY_KEY, 30.0)
            .end();
    }
    // A text input shapes its text in the file's one font, after any image.
    let font_asset = u64::from(pictured);
    let nested = entries
        .iter()
        .any(|entry| matches!(entry, Entry::Nested(..)));
    let nested_scroller = entries
        .iter()
        .any(|entry| matches!(entry, Entry::Box(node, _) if node.nested_scroller));
    if nested
        || entries
            .iter()
            .any(|entry| matches!(entry, Entry::TextInput(..)))
    {
        use crate::source::generated::assets::{
            file_asset_base::FileAssetBase, file_asset_contents_base::FileAssetContentsBase,
            font_asset_base::FontAssetBase,
        };
        riv.object(FontAssetBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, "font")
            .uint(FileAssetBase::ASSET_ID_PROPERTY_KEY, 2)
            .end();
        riv.object(FileAssetContentsBase::TYPE_KEY)
            .byte_field(
                FileAssetContentsBase::BYTES_PROPERTY_KEY,
                include_bytes!("../../../../../fixtures/fonts/roboto-input.ttf"),
            )
            .end();
    }
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Artboard")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, size.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, size.1)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, first_style);
    if spec.origin_x != 0.0 {
        riv.float(ArtboardBase::ORIGIN_X_PROPERTY_KEY, spec.origin_x);
    }
    if spec.unclipped {
        riv.uint(LayoutComponentBase::CLIP_PROPERTY_KEY, 0);
    }
    if binds {
        riv.uint(ArtboardBase::VIEW_MODEL_ID_PROPERTY_KEY, 0);
    }
    riv.end();
    let mut boxes = Vec::new();
    for entry in &entries {
        match entry {
            Entry::Box(node, parent) => {
                riv.object(LayoutComponentBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, node.name)
                    .uint(
                        LayoutComponentBase::STYLE_ID_PROPERTY_KEY,
                        first_style + 1 + boxes.len() as u64,
                    );
                write(&mut riv, &node.component);
                riv.end();
                boxes.push(*node);
            }
            Entry::Record(kind, props, parent) => {
                riv.object(*kind)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent);
                write(&mut riv, props);
                riv.end();
            }
            Entry::Keys(..) => {}
            Entry::Bind(key, flags, source) => {
                use crate::source::generated::data_bind::{
                    data_bind_base::DataBindBase, data_bind_context_base::DataBindContextBase,
                };
                riv.object(DataBindContextBase::TYPE_KEY)
                    .uint(DataBindBase::PROPERTY_KEY_PROPERTY_KEY, u64::from(*key))
                    .uint(DataBindBase::FLAGS_PROPERTY_KEY, *flags)
                    .byte_field(
                        DataBindContextBase::SOURCE_PATH_IDS_PROPERTY_KEY,
                        &[0, *source],
                    )
                    .end();
            }
            Entry::NamedShape(name, parent) => {
                riv.object(ShapeBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, name)
                    .end();
            }
            Entry::PathAt(x, size, parent) => {
                riv.object(RectangleBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .float(
                        crate::source::generated::node_base::NodeBase::X_PROPERTY_KEY,
                        *x,
                    )
                    .float(ParametricPathBase::WIDTH_PROPERTY_KEY, size.0)
                    .float(ParametricPathBase::HEIGHT_PROPERTY_KEY, size.1)
                    .end();
            }
            Entry::Fill(parent) => {
                riv.object(FillBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .end();
            }
            Entry::Stroke(parent) => {
                riv.object(StrokeBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .float(StrokeBase::THICKNESS_PROPERTY_KEY, 4.0)
                    .end();
            }
            Entry::Solid(color, parent) => {
                riv.object(SolidColorBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .color(SolidColorBase::COLOR_VALUE_PROPERTY_KEY, *color)
                    .end();
            }
            Entry::Feather(strength, inner, parent) => {
                riv.object(FeatherBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(FeatherBase::SPACE_VALUE_PROPERTY_KEY, 1)
                    .float(FeatherBase::STRENGTH_PROPERTY_KEY, *strength);
                if *inner {
                    riv.uint(FeatherBase::INNER_PROPERTY_KEY, 1);
                }
                riv.end();
            }
            Entry::LinearGradient(parent) => {
                riv.object(LinearGradientBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .end();
            }
            Entry::Stop(color, position, parent) => {
                riv.object(GradientStopBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .color(GradientStopBase::COLOR_VALUE_PROPERTY_KEY, *color)
                    .float(GradientStopBase::POSITION_PROPERTY_KEY, *position)
                    .end();
            }
            Entry::Scroll(parent, direction, node) => {
                use crate::source::generated::constraints::{
                    draggable_constraint_base::DraggableConstraintBase,
                    scrolling::scroll_constraint_base::ScrollConstraintBase,
                };
                riv.object(ScrollConstraintBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(
                        DraggableConstraintBase::DIRECTION_VALUE_PROPERTY_KEY,
                        *direction,
                    );
                if node.scroll_threshold != 0.0 {
                    riv.float(
                        ScrollConstraintBase::THRESHOLD_PROPERTY_KEY,
                        node.scroll_threshold,
                    );
                }
                for property in &node.scroll_properties {
                    match *property {
                        U(key, value) => {
                            riv.uint(key, value);
                        }
                        F(key, value) => {
                            riv.float(key, value);
                        }
                    }
                }
                riv.end();
            }
            Entry::ScrollBar(parent, direction, auto_size) => {
                use crate::source::generated::constraints::{
                    draggable_constraint_base::DraggableConstraintBase,
                    scrolling::scroll_bar_constraint_base::ScrollBarConstraintBase,
                };
                riv.object(ScrollBarConstraintBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(
                        DraggableConstraintBase::DIRECTION_VALUE_PROPERTY_KEY,
                        *direction,
                    )
                    .uint(
                        ScrollBarConstraintBase::SCROLL_CONSTRAINT_ID_PROPERTY_KEY,
                        first_scroll,
                    )
                    .uint(
                        ScrollBarConstraintBase::AUTO_SIZE_PROPERTY_KEY,
                        u64::from(*auto_size),
                    )
                    .end();
            }
            Entry::Image(name, parent) => {
                use crate::source::generated::shapes::image_base::ImageBase;
                riv.object(ImageBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, name)
                    .uint(ImageBase::ASSET_ID_PROPERTY_KEY, 0)
                    .float(ImageBase::ORIGIN_X_PROPERTY_KEY, 0.0)
                    .float(ImageBase::ORIGIN_Y_PROPERTY_KEY, 0.0)
                    .end();
            }
            Entry::Track((collection, kind, value), parent) => {
                use crate::source::generated::layout::grid_track_base::GridTrackBase;
                riv.object(GridTrackBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(GridTrackBase::COLLECTION_PROPERTY_KEY, *collection)
                    .uint(GridTrackBase::TRACK_TYPE_PROPERTY_KEY, *kind)
                    .float(GridTrackBase::TRACK_VALUE_PROPERTY_KEY, *value)
                    .end();
            }
            Entry::Participant(parent) => {
                riv.object(crate::source::layout::layout_participant::LayoutParticipant::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .end();
            }
            Entry::Group(parent) => {
                riv.object(NodeBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .end();
            }
            Entry::EmptyText(parent) => {
                riv.object(crate::source::generated::text::text_base::TextBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, "Note")
                    .end();
            }
            Entry::TextInput(parent, text) => {
                use crate::source::generated::text::text_input_base::TextInputBase;
                riv.object(TextInputBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, "Field")
                    .string(TextInputBase::TEXT_PROPERTY_KEY, text)
                    .end();
            }
            Entry::FocusData(parent) => {
                riv.object(crate::source::generated::focus_data_base::FocusDataBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .end();
            }
            Entry::TextStyle(parent) => {
                use crate::source::generated::text::{
                    text_style_base::TextStyleBase, text_style_paint_base::TextStylePaintBase,
                };
                riv.object(TextStylePaintBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(TextStyleBase::FONT_ASSET_ID_PROPERTY_KEY, font_asset)
                    .float(TextStyleBase::FONT_SIZE_PROPERTY_KEY, 16.0)
                    .end();
            }
            Entry::Nested(parent) => {
                use crate::source::generated::nested_artboard_base::NestedArtboardBase;
                riv.object(NestedArtboardBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, "Nested")
                    .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
                    .end();
            }
            Entry::NestedMachine(parent) => {
                use crate::source::generated::{
                    animation::nested_state_machine_base::NestedStateMachineBase,
                    nested_animation_base::NestedAnimationBase,
                };
                riv.object(NestedStateMachineBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY, 0)
                    .end();
            }
            Entry::Event(name) => {
                riv.object(crate::source::generated::event_base::EventBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
                    .string(ComponentBase::NAME_PROPERTY_KEY, name)
                    .end();
            }
            Entry::Shape(rect, parent) => {
                riv.object(ShapeBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(ComponentBase::NAME_PROPERTY_KEY, rect.name)
                    .end();
            }
            Entry::Path(rect, parent) => {
                riv.object(RectangleBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .string(
                        ComponentBase::NAME_PROPERTY_KEY,
                        &format!("{} path", rect.name),
                    )
                    .float(ParametricPathBase::WIDTH_PROPERTY_KEY, rect.size.0)
                    .float(ParametricPathBase::HEIGHT_PROPERTY_KEY, rect.size.1)
                    .float(
                        crate::source::generated::transform_component_base::TransformComponentBase::SCALE_X_PROPERTY_KEY,
                        rect.path_scale_x,
                    )
                    .end();
            }
        }
    }
    let style = |riv: &mut Riv, parent: u64, props: &[P]| {
        riv.object(LayoutComponentStyleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent);
        write(riv, &point_units());
        write(riv, props);
        riv.end();
    };
    style(&mut riv, 0, &spec.style);
    let mut index = 0u64;
    for entry in &entries {
        if matches!(entry, Entry::Bind(..) | Entry::Keys(..)) {
            continue;
        }
        index += 1;
        if let Entry::Box(node, _) = entry {
            style(&mut riv, index, &node.style);
        }
    }
    // One animation, a second at 60 frames, keying every Keys entry.
    let keyed = entries
        .iter()
        .filter_map(|entry| match entry {
            Entry::Keys(object, key, keys) => Some((*object, *key, *keys)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !keyed.is_empty() {
        use crate::source::generated::animation::{
            interpolating_keyframe_base::InterpolatingKeyFrameBase,
            keyed_object_base::KeyedObjectBase, keyed_property_base::KeyedPropertyBase,
            keyframe_base::KeyFrameBase, keyframe_double_base::KeyFrameDoubleBase,
            linear_animation_base::LinearAnimationBase,
        };
        riv.object(LinearAnimationBase::TYPE_KEY)
            .uint(LinearAnimationBase::FPS_PROPERTY_KEY, 60)
            .uint(LinearAnimationBase::DURATION_PROPERTY_KEY, 60)
            .end();
        for (object, key, keys) in keyed {
            riv.object(KeyedObjectBase::TYPE_KEY)
                .uint(KeyedObjectBase::OBJECT_ID_PROPERTY_KEY, object)
                .end();
            riv.object(KeyedPropertyBase::TYPE_KEY)
                .uint(KeyedPropertyBase::PROPERTY_KEY_PROPERTY_KEY, u64::from(key))
                .end();
            for &(frame, value) in keys {
                riv.object(KeyFrameDoubleBase::TYPE_KEY)
                    .uint(KeyFrameBase::FRAME_PROPERTY_KEY, u64::from(frame))
                    .float(KeyFrameDoubleBase::VALUE_PROPERTY_KEY, value)
                    .uint(
                        InterpolatingKeyFrameBase::INTERPOLATION_TYPE_PROPERTY_KEY,
                        1,
                    )
                    .end();
            }
        }
    }
    // One state machine with each box's listeners, each firing its event.
    if !listeners.is_empty() {
        use crate::source::generated::animation::{
            any_state_base::AnyStateBase, entry_state_base::EntryStateBase,
            exit_state_base::ExitStateBase, listener_fire_event_base::ListenerFireEventBase,
            listener_types::listener_input_type_base::ListenerInputTypeBase,
            state_machine_base::StateMachineBase, state_machine_layer_base::StateMachineLayerBase,
            state_machine_listener_base::StateMachineListenerBase,
        };
        riv.object(StateMachineBase::TYPE_KEY).end();
        for (target, kind, event) in listeners {
            riv.object(StateMachineListenerBase::TYPE_KEY)
                .uint(StateMachineListenerBase::TARGET_ID_PROPERTY_KEY, target)
                .end();
            riv.object(ListenerInputTypeBase::TYPE_KEY)
                .uint(
                    ListenerInputTypeBase::LISTENER_TYPE_VALUE_PROPERTY_KEY,
                    kind as u64,
                )
                .end();
            riv.object(ListenerFireEventBase::TYPE_KEY)
                .uint(ListenerFireEventBase::EVENT_ID_PROPERTY_KEY, event)
                .end();
        }
        riv.object(StateMachineLayerBase::TYPE_KEY).end();
        for state in [
            EntryStateBase::TYPE_KEY,
            AnyStateBase::TYPE_KEY,
            ExitStateBase::TYPE_KEY,
        ] {
            riv.object(state).end();
        }
    }
    if nested_scroller {
        // The nested artboard: 0 the artboard, 1 its viewport, 2 the list,
        // 3 the list's ScrollConstraint, 4 to 6 the three styles.
        use crate::source::generated::{
            animation::{
                any_state_base::AnyStateBase, entry_state_base::EntryStateBase,
                exit_state_base::ExitStateBase, state_machine_base::StateMachineBase,
                state_machine_layer_base::StateMachineLayerBase,
            },
            constraints::{
                draggable_constraint_base::DraggableConstraintBase,
                scrolling::scroll_constraint_base::ScrollConstraintBase,
            },
        };
        riv.object(ArtboardBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Nested")
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 4)
            .end();
        for (index, (name, width)) in [("Nested viewport", 200.0), ("Nested list", 400.0)]
            .into_iter()
            .enumerate()
        {
            riv.object(LayoutComponentBase::TYPE_KEY)
                .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, index as u64)
                .string(ComponentBase::NAME_PROPERTY_KEY, name)
                .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, width)
                .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0)
                .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 5 + index as u64)
                .end();
        }
        riv.object(ScrollConstraintBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
            .uint(DraggableConstraintBase::DIRECTION_VALUE_PROPERTY_KEY, 0)
            .end();
        // The viewport stacks its list in a column, so the list keeps its
        // width rather than shrinking along a row.
        let column = [U(
            LayoutComponentStyleBase::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
            YGFlexDirection::Column as u64,
        )];
        style(&mut riv, 0, &[]);
        style(&mut riv, 1, &column);
        style(&mut riv, 2, &[]);
        riv.object(StateMachineBase::TYPE_KEY).end();
        riv.object(StateMachineLayerBase::TYPE_KEY).end();
        for state in [
            EntryStateBase::TYPE_KEY,
            AnyStateBase::TYPE_KEY,
            ExitStateBase::TYPE_KEY,
        ] {
            riv.object(state).end();
        }
    } else if nested {
        // The nested artboard: 0 the artboard, 1 its box, 2 the field, 3 its
        // FocusData, 4 its text style, 5 and 6 the two styles.
        use crate::source::generated::{
            animation::{
                any_state_base::AnyStateBase, entry_state_base::EntryStateBase,
                exit_state_base::ExitStateBase, state_machine_base::StateMachineBase,
                state_machine_layer_base::StateMachineLayerBase,
            },
            focus_data_base::FocusDataBase,
            text::{
                text_input_base::TextInputBase, text_style_base::TextStyleBase,
                text_style_paint_base::TextStylePaintBase,
            },
        };
        riv.object(ArtboardBase::TYPE_KEY)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Nested")
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 5)
            .end();
        riv.object(LayoutComponentBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Nested box")
            .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 180.0)
            .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0)
            .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 6)
            .end();
        riv.object(TextInputBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
            .string(ComponentBase::NAME_PROPERTY_KEY, "Nested field")
            .string(TextInputBase::TEXT_PROPERTY_KEY, "hello world")
            .end();
        riv.object(FocusDataBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
            .end();
        riv.object(TextStylePaintBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
            .uint(TextStyleBase::FONT_ASSET_ID_PROPERTY_KEY, font_asset)
            .float(TextStyleBase::FONT_SIZE_PROPERTY_KEY, 16.0)
            .end();
        style(&mut riv, 0, &[]);
        style(&mut riv, 1, &[]);
        riv.object(StateMachineBase::TYPE_KEY).end();
        riv.object(StateMachineLayerBase::TYPE_KEY).end();
        for state in [
            EntryStateBase::TYPE_KEY,
            AnyStateBase::TYPE_KEY,
            ExitStateBase::TYPE_KEY,
        ] {
            riv.object(state).end();
        }
    }
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )?;
    let artboard = file.with_file(File::artboard_default)?;
    let scene = Scene {
        _file: file,
        artboard,
        factory,
    };
    scene.advance(3);
    Some(scene)
}

fn scene(size: (f32, f32), root: B) -> Scene {
    try_scene(size, None, root).expect("the scene imports")
}

impl Scene {
    fn named(&self, name: &str) -> CoreHandle {
        self.artboard
            .with_artboard(|artboard| artboard.find_all_handles::<LayoutComponent>())
            .into_iter()
            .find(|handle| {
                handle
                    .with(|object| {
                        object
                            .as_component()
                            .is_some_and(|component| component.base.name() == name)
                    })
                    .unwrap_or(false)
            })
            .unwrap_or_else(|| panic!("{name}"))
    }
    /// A drawing node's world translation, by name.
    fn world_origin(&self, name: &str) -> (f32, f32) {
        let node = self
            .artboard
            .with_artboard(|artboard| artboard.find_handle::<crate::source::node::Node>(name))
            .unwrap_or_else(|| panic!("{name}"));
        node.with(|object| {
            let world = object
                .as_world_transform_component()
                .expect("a transform")
                .world_transform();
            (world.tx(), world.ty())
        })
        .unwrap()
    }
    /// Binds a new instance of the file's view model and returns its number.
    fn bind_number(&self) -> CoreHandle {
        self.bind_numbers().remove(0)
    }
    /// Binds a new instance and returns all of its numbers.
    fn bind_numbers(&self) -> Vec<CoreHandle> {
        let artboard = self.artboard.core_handle();
        let instance = self
            ._file
            .with_file(|file| file.create_view_model_instance_for_artboard(artboard))
            .expect("a view model instance");
        self.artboard
            .bind_view_model_instance(Some(instance.clone()));
        instance
            .with_downcast::<crate::source::viewmodel::viewmodel_instance::ViewModelInstance, _>(
                |instance: &crate::source::viewmodel::viewmodel_instance::ViewModelInstance| {
                    instance.property_values().to_vec()
                },
            )
            .expect("the numbers")
    }
    /// The stream lines one draw of the artboard records.
    fn draw_lines(&self) -> Vec<String> {
        let before = self.factory.borrow().stream().lines().count();
        let mut renderer = self.factory.borrow().make_renderer();
        self.artboard.draw(&mut renderer);
        let stream = self.factory.borrow().stream();
        stream.lines().skip(before).map(str::to_owned).collect()
    }
    /// Every stream line recorded so far.
    fn stream_lines(&self) -> Vec<String> {
        self.factory
            .borrow()
            .stream()
            .lines()
            .map(str::to_owned)
            .collect()
    }
    /// A box's special CSS paints: each path's bounds and its clip's.
    fn css_paints(
        &self,
        name: &str,
    ) -> Vec<(
        crate::source::math::aabb::Aabb,
        Option<crate::source::math::aabb::Aabb>,
    )> {
        self.named(name)
            .with_downcast::<LayoutComponent, _>(|layout| {
                let paths = layout.render_paths_handle().expect("render paths");
                let paths = paths.borrow();
                paths
                    .css_paints
                    .iter()
                    .map(|paint| {
                        (
                            paint.path.raw_path().bounds(),
                            paint.clip.as_ref().map(|clip| clip.raw_path().bounds()),
                        )
                    })
                    .collect()
            })
            .unwrap()
    }
    /// The first child of `type_key` under any of `name`'s paints.
    fn paint_child(&self, name: &str, type_key: u16) -> CoreHandle {
        fn find(handle: &CoreHandle, type_key: u16) -> Option<CoreHandle> {
            if handle.is_type_of(type_key) {
                return Some(handle.clone());
            }
            let children = handle
                .with(|object| {
                    object
                        .as_container_component()
                        .map(|container| container.children().to_vec())
                })
                .flatten()
                .unwrap_or_default();
            children.iter().find_map(|child| find(child, type_key))
        }
        let paints = self
            .named(name)
            .with_downcast::<LayoutComponent, _>(|layout| layout.paints.shape_paints().to_vec())
            .unwrap();
        paints
            .iter()
            .find_map(|paint| find(paint, type_key))
            .unwrap_or_else(|| panic!("{name} has a {type_key}"))
    }
    fn advance(&self, frames: usize) {
        for _ in 0..frames {
            self.artboard.advance_default(FRAME);
        }
    }
    /// The file's one state machine, started.
    fn machine(&self) -> crate::RuntimeStateMachineInstanceHandle {
        let machine = self
            .artboard
            .state_machine_at(0)
            .expect("the state machine");
        machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        self.advance(1);
        machine.with_instance_mut(|machine| machine.advance_seconds(0.0));
        machine
    }
    /// The file's one scroll constraint's vertical offset.
    fn scroll_offset_y(&self) -> f32 {
        use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
        self.artboard
            .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
            .into_iter()
            .next()
            .expect("a scroll constraint")
            .with_downcast(ScrollConstraint::offset_y)
            .expect("a live scroll constraint")
    }
    /// Whether the text input named "Field" has focus.
    fn field_is_focused(&self) -> bool {
        self.artboard
            .with_artboard(|artboard| {
                artboard.find_handle::<crate::source::text::text_input::TextInput>("Field")
            })
            .expect("the field")
            .with_downcast(crate::source::text::text_input::TextInput::is_focused)
            .expect("a live field")
    }
    /// A box's solved left, top, width and height, relative to its parent.
    fn bounds(&self, name: &str) -> [f32; 4] {
        let bounds = self
            .named(name)
            .with_downcast::<LayoutComponent, _>(|layout| layout.layout_bounds())
            .expect(name);
        [bounds.left(), bounds.top(), bounds.width(), bounds.height()]
    }
    /// A rectangle's current path size, which layout writes when it
    /// resizes the shape.
    fn path_size(&self, name: &str) -> (f32, f32) {
        let path = format!("{name} path");
        self.artboard
            .with_artboard(|artboard| {
                artboard.find_all_handles::<crate::source::shapes::rectangle::Rectangle>()
            })
            .into_iter()
            .find_map(|handle| {
                handle
                    .with(|object| {
                        let path_object = object.as_parametric_path()?;
                        (object.as_component()?.base.name() == path)
                            .then(|| (path_object.width(), path_object.height()))
                    })
                    .flatten()
            })
            .unwrap_or_else(|| panic!("{path}"))
    }
    /// The scroll constraint's offset range: (max x, max y), both <= 0.
    fn max_offsets(&self) -> (f32, f32) {
        use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
        let scroll = self
            .artboard
            .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
            .into_iter()
            .next()
            .expect("a scroll constraint");
        scroll
            .with_downcast::<ScrollConstraint, _>(|scroll| {
                (scroll.max_offset_x(), scroll.max_offset_y())
            })
            .unwrap()
    }
    /// A box's background and clip paths, in its own and world space.
    fn paths(
        &self,
        name: &str,
    ) -> (
        Vec<crate::source::math::vec2d::Vec2D>,
        Option<crate::source::math::aabb::Aabb>,
        bool,
    ) {
        self.named(name)
            .with_downcast::<LayoutComponent, _>(|layout| {
                let paths = layout.render_paths_handle().expect("render paths");
                let paths = paths.borrow();
                let clips = layout.clips();
                (
                    paths.background.points().to_vec(),
                    clips.then(|| paths.clip.raw_path().bounds()),
                    clips,
                )
            })
            .unwrap()
    }
    fn record(&self, name: &str) -> CoreHandle {
        let owner = self.named(name);
        owner
            .with(|object| {
                object.as_container_component().and_then(|container| {
                    container
                        .children()
                        .iter()
                        .find(|child| child.is_type_of(LAYOUT_CSS_STYLE))
                        .cloned()
                })
            })
            .flatten()
            .unwrap_or_else(|| panic!("{name} has a record"))
    }
}

/// `display: flex` with padding and a gap: the second item grows into what
/// the first leaves.
#[test]
fn css_flex_row_lays_out_by_its_record() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [
                [U(DISPLAY, 1), U(BOX_SIZING, 1)].as_slice(),
                &width(400.0),
                &height(100.0),
                &padding(10.0),
                &[F(COLUMN_GAP, 20.0)],
            ]
            .concat(),
            vec![
                css("A", [width(50.0), height(20.0)].concat(), vec![]),
                css(
                    "B",
                    [[F(FLEX_GROW, 1.0)].as_slice(), &height(20.0)].concat(),
                    vec![],
                ),
            ],
        ),
    );
    assert_eq!(f.bounds("Row"), [0.0, 0.0, 400.0, 100.0]);
    // 400 - 2 * 10 padding - 50 - 20 gap = 310.
    assert_eq!(f.bounds("A"), [10.0, 10.0, 50.0, 20.0]);
    assert_eq!(f.bounds("B"), [80.0, 10.0, 310.0, 20.0]);
}

/// `box-sizing` is content-box unless the record says border-box.
#[test]
fn css_width_excludes_padding_and_border_by_default() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Box",
            [
                [U(DISPLAY, FLEX as u64)].as_slice(),
                &width(100.0),
                &height(50.0),
                &padding(10.0),
                &[
                    F(BORDER_TOP, 2.0),
                    F(BORDER_RIGHT, 2.0),
                    F(BORDER_BOTTOM, 2.0),
                    F(BORDER_LEFT, 2.0),
                ],
            ]
            .concat(),
            vec![css("Child", [width(10.0), height(10.0)].concat(), vec![])],
        ),
    );
    // 100 + 2 * (10 + 2) by 50 + 2 * (10 + 2).
    assert_eq!(f.bounds("Box"), [0.0, 0.0, 124.0, 74.0]);
    assert_eq!(f.bounds("Child"), [12.0, 12.0, 10.0, 10.0]);
}

/// `display: block` (the record's initial display): children stack at the
/// full width, and adjacent vertical margins collapse.
#[test]
fn css_block_flow_stacks_and_collapses_margins() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Column",
            width(200.0),
            vec![
                css(
                    "A",
                    [[F(MARGIN_BOTTOM, 10.0)].as_slice(), &height(30.0)].concat(),
                    vec![],
                ),
                css(
                    "B",
                    [[F(MARGIN_TOP, 20.0)].as_slice(), &height(40.0)].concat(),
                    vec![],
                ),
            ],
        ),
    );
    assert_eq!(f.bounds("A"), [0.0, 0.0, 200.0, 30.0]);
    // max(10, 20) between them.
    assert_eq!(f.bounds("B"), [0.0, 50.0, 200.0, 40.0]);
    assert_eq!(f.bounds("Column"), [0.0, 0.0, 200.0, 90.0]);
}

/// flex-shrink comes from the record: 0 keeps its width, the initial 1
/// gives up the overflow.
#[test]
fn css_flex_shrink_comes_from_the_record() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
            vec![
                css(
                    "Keep",
                    [[F(FLEX_SHRINK, 0.0)].as_slice(), &width(200.0)].concat(),
                    vec![],
                ),
                css("Give", width(200.0), vec![]),
            ],
        ),
    );
    assert_eq!(f.bounds("Keep")[2], 200.0);
    assert_eq!(f.bounds("Give"), [200.0, 0.0, 100.0, 40.0]);
}

/// Percentages resolve against the parent, and an absolute box is placed by
/// its insets inside its positioned parent.
#[test]
fn css_percentages_and_absolute_insets() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [
                [U(DISPLAY, 1), U(POSITION, 1)].as_slice(),
                &width(400.0),
                &height(200.0),
            ]
            .concat(),
            vec![
                css(
                    "Half",
                    [
                        U(WIDTH_KIND, 1),
                        F(WIDTH_PERCENT, 50.0),
                        U(HEIGHT_KIND, 1),
                        F(HEIGHT, 10.0),
                    ],
                    vec![],
                ),
                css(
                    "Pinned",
                    [
                        [
                            U(POSITION, 2),
                            U(INSET_LEFT_KIND, 1),
                            F(INSET_LEFT, 10.0),
                            U(INSET_TOP_KIND, 1),
                            F(INSET_TOP, 20.0),
                        ]
                        .as_slice(),
                        &width(30.0),
                        &height(30.0),
                    ]
                    .concat(),
                    vec![],
                ),
            ],
        ),
    );
    assert_eq!(f.bounds("Half"), [0.0, 0.0, 200.0, 10.0]);
    assert_eq!(f.bounds("Pinned"), [10.0, 20.0, 30.0, 30.0]);
}

/// `display: none` takes the box out of flow and hides it.
#[test]
fn css_display_none_hides_the_box() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
            vec![
                css(
                    "Gone",
                    [[U(DISPLAY, NONE as u64)].as_slice(), &width(100.0)].concat(),
                    vec![],
                ),
                css("Next", width(50.0), vec![]),
            ],
        ),
    );
    assert_eq!(f.bounds("Next")[0], 0.0);
    let hidden = f
        .named("Gone")
        .with_downcast::<LayoutComponent, _>(LayoutComponent::is_collapsed)
        .unwrap();
    assert!(hidden);
}

fn bound_records_under_hidden_box(nested: bool, cloned: bool) {
    use crate::source::generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase;
    let set_number = |number: &CoreHandle, value: f32| {
        assert!(CoreRegistry::set_double_handle(
            number,
            ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
            value
        ));
    };
    for display in [NONE, FLEX] {
        let children = vec![
            B {
                bind: Some(MARGIN_LEFT),
                ..css(
                    "Child",
                    [
                        [F(MARGIN_LEFT, 30.0)].as_slice(),
                        &width(50.0),
                        &height(20.0),
                    ]
                    .concat(),
                    vec![],
                )
            },
            B {
                name: "Ordinary",
                component_bind: Some(BOX_ROTATION),
                component: vec![
                    F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 10.0),
                    F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 10.0),
                ],
                ..B::default()
            },
        ];
        let children = if nested {
            vec![css(
                "Middle",
                [[U(DISPLAY, 1)].as_slice(), &width(260.0), &height(50.0)].concat(),
                children,
            )]
        } else {
            children
        };
        let mut f = scene(
            (300.0, 100.0),
            B {
                bind: Some(DISPLAY),
                bind_source: 1,
                ..css(
                    "Parent",
                    [
                        [U(DISPLAY, display as u64)].as_slice(),
                        &width(260.0),
                        &height(50.0),
                    ]
                    .concat(),
                    children,
                )
            },
        );
        if cloned {
            f.artboard = f.artboard.instance().expect("an artboard instance");
            f.advance(3);
        }
        let numbers = f.bind_numbers();
        set_number(&numbers[0], 30.0);
        set_number(&numbers[1], display);
        f.advance(3);
        let ordinary = f.named("Ordinary");
        let rotation = || CoreRegistry::get_double_handle(&ordinary, BOX_ROTATION.into()).unwrap();
        if display == FLEX {
            assert_eq!(f.bounds("Child")[0], 30.0);
            assert_eq!(rotation(), 30.0);
        }
        // Change the margin while hidden, then repeat after showing once.
        for margin in [10.0, 20.0] {
            set_number(&numbers[1], NONE);
            f.advance(3);
            assert!(
                ordinary
                    .with(|object| object.component_is_collapsed())
                    .unwrap()
            );
            let parked = rotation();
            set_number(&numbers[0], margin);
            f.advance(3);
            assert_eq!(
                CoreRegistry::get_double_handle(&f.record("Child"), MARGIN_LEFT.into()),
                Some(margin)
            );
            assert_eq!(rotation(), parked);
            for name in ["Parent", "Child"] {
                assert!(
                    !f.record(name)
                        .with(|object| object.component_is_collapsed())
                        .unwrap()
                );
            }
            set_number(&numbers[1], FLEX);
            f.advance(3);
            assert_eq!(f.bounds("Child")[0], margin);
            assert_eq!(rotation(), margin);
            assert!(
                !ordinary
                    .with(|object| object.component_is_collapsed())
                    .unwrap()
            );
        }
    }
}

/// CSS bindings keep applying while hidden; ordinary bindings wait for expansion.
#[test]
fn css_bound_records_survive_parent_collapse() {
    bound_records_under_hidden_box(false, false);
}

#[test]
fn css_bound_records_survive_grandparent_collapse() {
    bound_records_under_hidden_box(true, false);
}

#[test]
fn css_bound_records_survive_instance_initialization() {
    bound_records_under_hidden_box(true, true);
}

#[test]
fn css_participant_records_survive_host_collapse() {
    let f = scene(
        (300.0, 100.0),
        B {
            participants: vec![(
                Rect {
                    name: "Item",
                    size: (50.0, 20.0),
                    grouped: false,
                    path_scale_x: 1.0,
                },
                Some(
                    [
                        [U(DISPLAY, NONE as u64)].as_slice(),
                        &width(50.0),
                        &height(20.0),
                    ]
                    .concat(),
                ),
            )],
            ..css(
                "Parent",
                [[U(DISPLAY, FLEX as u64)].as_slice(), &height(50.0)].concat(),
                vec![],
            )
        },
    );
    let host = f
        .artboard
        .with_artboard(|artboard| {
            artboard.find_handle::<crate::source::shapes::shape::Shape>("Item")
        })
        .expect("a participant host");
    let record = host
        .with(|object| {
            object
                .as_container_component()
                .unwrap()
                .children()
                .iter()
                .find(|child| child.is_type_of(LAYOUT_CSS_STYLE))
                .cloned()
        })
        .flatten()
        .expect("a CSS record");
    let collapsed = |owner: &CoreHandle| {
        owner
            .with(|object| object.component_is_collapsed())
            .unwrap()
    };
    assert!(collapsed(&host));
    assert!(!collapsed(&record));
    for display in [FLEX, NONE, FLEX] {
        assert!(CoreRegistry::set_uint_handle(
            &record,
            DISPLAY.into(),
            display as u32
        ));
        f.advance(3);
        assert_eq!(collapsed(&host), display == NONE);
        assert!(!collapsed(&record));
    }
}

/// A record on the artboard sets its container properties; the artboard
/// keeps its own size.
#[test]
fn css_artboard_record_sets_container_properties() {
    let f = try_scene(
        (400.0, 300.0),
        // Its width is an item property, so the artboard ignores it.
        Some(
            [
                [U(DISPLAY, 1), U(WIDTH_KIND, LENGTH as u64), F(WIDTH, 10.0)].as_slice(),
                &padding(15.0),
            ]
            .concat(),
        ),
        css("Child", [width(50.0), height(20.0)].concat(), vec![]),
    )
    .expect("the scene imports");
    assert_eq!(f.bounds("Child"), [15.0, 15.0, 50.0, 20.0]);
    let artboard = f
        .artboard
        .with_artboard(|artboard| artboard.layout_bounds());
    assert_eq!((artboard.width(), artboard.height()), (400.0, 300.0));
}

/// A record value written at run time, as animation and binding write it,
/// lays the box out again.
#[test]
fn css_record_writes_relayout() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
            vec![css("A", width(50.0), vec![])],
        ),
    );
    assert_eq!(f.bounds("A")[2], 50.0);
    assert!(CoreRegistry::set_double_handle(
        &f.record("A"),
        WIDTH.into(),
        120.0
    ));
    assert_eq!(
        CoreRegistry::get_double_handle(&f.record("A"), WIDTH.into()),
        Some(120.0)
    );
    f.advance(2);
    assert_eq!(f.bounds("A")[2], 120.0);
    assert!(CoreRegistry::set_uint_handle(
        &f.record("A"),
        DISPLAY.into(),
        3
    ));
    f.advance(2);
    assert!(
        f.named("A")
            .with_downcast::<LayoutComponent, _>(LayoutComponent::is_collapsed)
            .unwrap()
    );
}

/// Refused at load (docs/CSS_LAYOUT.md, 1.4): the import fails.
#[test]
fn css_refused_forms_fail_the_import() {
    let refused =
        |props: Vec<P>| try_scene((400.0, 300.0), None, css("Box", props, vec![])).is_none();
    // A mixed calc(): 10px + 50%.
    assert!(refused(vec![
        U(WIDTH_KIND, 1),
        F(WIDTH, 10.0),
        F(WIDTH_PERCENT, 50.0)
    ]));
    // display 9 is outside 0 to 4.
    assert!(refused(vec![U(DISPLAY, 9)]));
    // A size kind outside 0 to 5.
    assert!(refused(vec![U(WIDTH_KIND, 6)]));
    // Mixed parts are not read for an auto width, so they are not refused.
    assert!(!refused(vec![F(WIDTH, 10.0), F(WIDTH_PERCENT, 50.0)]));
    assert!(!refused(vec![U(WIDTH_KIND, 1), F(WIDTH, 10.0)]));
    // Two records on one box.
    assert!(try_two_records().is_none());
}

/// Two records under one box.
fn try_two_records() -> Option<Scene> {
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 4)
        .end();
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 5)
        .end();
    for _ in 0..2 {
        riv.object(LAYOUT_CSS_STYLE)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1)
            .end();
    }
    for parent in [0, 1] {
        riv.object(LayoutComponentStyleBase::TYPE_KEY)
            .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, parent)
            .end();
    }
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )?;
    let artboard = file.with_file(File::artboard_default)?;
    Some(Scene {
        _file: file,
        artboard,
        factory,
    })
}

/// Translated mode: Fill along the main axis is `flex: <fraction> 1 0%`, so
/// Fill siblings share the row by fraction whatever their content. Rive's
/// own Fill sized each to its content first (here 180 and 120).
#[test]
fn translated_fill_shares_the_row_by_fraction() {
    let fill = |name: &'static str, children: Vec<B>| B {
        name,
        component: vec![F(LayoutComponentBase::FRACTIONAL_WIDTH_PROPERTY_KEY, 1.0)],
        style: vec![
            U(
                LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                LayoutScaleType::Fill as u64,
            ),
            U(
                LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
                LayoutScaleType::Fill as u64,
            ),
        ],
        children,
        ..B::default()
    };
    let content = B {
        name: "Content",
        component: vec![
            F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 60.0),
            F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 20.0),
        ],
        ..B::default()
    };
    let f = scene(
        (300.0, 100.0),
        B {
            name: "Row",
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 300.0),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0),
            ],
            style: vec![U(
                LayoutComponentStyleBase::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
                YGFlexDirection::Row as u64,
            )],
            children: vec![fill("Wide", vec![content]), fill("Empty", vec![])],
            ..B::default()
        },
    );
    assert_eq!(f.bounds("Wide")[2], 150.0);
    assert_eq!(f.bounds("Empty"), [150.0, 0.0, 150.0, 100.0]);
}

/// CSS mode: a shape is content at its natural size, as an `<svg width>` is.
/// Its box hugs it, keeps it as its automatic minimum, and never resizes it.
#[test]
fn css_shape_is_content_at_its_natural_size() {
    let rect = |name| Rect {
        name,
        size: (80.0, 30.0),
        grouped: false,
        path_scale_x: 1.0,
    };
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(100.0), &height(40.0)].concat(),
            vec![
                B {
                    content: vec![rect("Icon")],
                    ..css("Hug", [], vec![])
                },
                css("Wide", width(100.0), vec![]),
            ],
        ),
    );
    // Hug's basis is 80 and its minimum 80; Wide gives up the overflow.
    assert_eq!(f.bounds("Hug"), [0.0, 0.0, 80.0, 40.0]);
    assert_eq!(f.bounds("Wide"), [80.0, 0.0, 20.0, 40.0]);
    assert_eq!(f.path_size("Icon"), (80.0, 30.0));

    // A box wider than its shape leaves the shape at its own size.
    let f = scene(
        (400.0, 300.0),
        B {
            content: vec![rect("Icon")],
            ..css("Box", [width(200.0), height(100.0)].concat(), vec![])
        },
    );
    assert_eq!(f.bounds("Box"), [0.0, 0.0, 200.0, 100.0]);
    assert_eq!(f.path_size("Icon"), (80.0, 30.0));
}

/// Translated mode: a shape the box resizes is the box's paint, so it adds
/// nothing to the box's automatic minimum. The Fixed box shrinks with its
/// sibling in proportion to their bases (80 and 100 sharing an 80 overflow).
/// A grouped shape is content and keeps the minimum.
#[test]
fn translated_paint_shape_adds_no_minimum() {
    let row = |grouped: bool| {
        let fixed = |name: &'static str, width: f32, content: Vec<Rect>| B {
            name,
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, width),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0),
            ],
            content,
            ..B::default()
        };
        scene(
            (400.0, 300.0),
            B {
                name: "Row",
                component: vec![
                    F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0),
                    F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0),
                ],
                style: vec![U(
                    LayoutComponentStyleBase::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
                    YGFlexDirection::Row as u64,
                )],
                children: vec![
                    fixed(
                        "Painted",
                        80.0,
                        vec![Rect {
                            name: "Background",
                            size: (80.0, 40.0),
                            grouped,
                            path_scale_x: 1.0,
                        }],
                    ),
                    fixed("Sibling", 100.0, vec![]),
                ],
                ..B::default()
            },
        )
    };
    let close = |a: f32, b: f32| (a - b).abs() < 1e-3;
    let f = row(false);
    let painted = f.bounds("Painted")[2];
    assert!(close(painted, 80.0 - 80.0 * 80.0 / 180.0), "{painted}");
    assert!(close(f.bounds("Sibling")[2], 100.0 - 80.0 * 100.0 / 180.0));
    // Layout resized the background to its box.
    assert!(close(f.path_size("Background").0, painted));

    let f = row(true);
    assert_eq!(f.bounds("Painted")[2], 80.0);
    assert_eq!(f.bounds("Sibling")[2], 20.0);
}

/// CSS mode clips by `overflow`, at the padding box, and not by Rive's clip
/// (on by default, but CSS mode ignores it). The background has CSS radii.
#[test]
fn css_overflow_clips_at_the_padding_box() {
    const OVERFLOW_X: u16 = 61003;
    const OVERFLOW_Y: u16 = 61004;
    const RADIUS_TL_PERCENT: u16 = 61079;
    let border = [
        F(BORDER_TOP, 4.0),
        F(BORDER_RIGHT, 4.0),
        F(BORDER_BOTTOM, 4.0),
        F(BORDER_LEFT, 4.0),
    ];
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [[U(DISPLAY, 1)].as_slice(), &width(400.0), &height(300.0)].concat(),
            vec![
                css(
                    "Clipped",
                    [
                        [U(BOX_SIZING, 1), U(OVERFLOW_Y, 2)].as_slice(),
                        &width(100.0),
                        &height(60.0),
                        &border,
                    ]
                    .concat(),
                    vec![],
                ),
                css(
                    "Pill",
                    [
                        [
                            U(OVERFLOW_X, 1),
                            U(OVERFLOW_Y, 1),
                            F(RADIUS_TL_PERCENT, 50.0),
                        ]
                        .as_slice(),
                        &width(200.0),
                        &height(100.0),
                    ]
                    .concat(),
                    vec![],
                ),
                css("Visible", [width(10.0), height(10.0)].concat(), vec![]),
            ],
        ),
    );
    // overflow-y hidden makes overflow-x auto: both axes clip, inside the
    // 4-point border.
    let (_, clip, clips) = f.paths("Clipped");
    assert!(clips);
    let clip = clip.unwrap();
    assert_eq!(
        [clip.left(), clip.top(), clip.right(), clip.bottom()],
        [4.0, 4.0, 96.0, 56.0]
    );
    // 50% of 200 by 100 at the top left: a 100 by 50 ellipse, so the path
    // starts 100 along the top edge.
    let (background, _, clips) = f.paths("Pill");
    assert!(clips);
    assert_eq!(
        background[0],
        crate::source::math::vec2d::Vec2D::new(100.0, 0.0)
    );
    let visible = f
        .named("Visible")
        .with_downcast::<LayoutComponent, _>(LayoutComponent::clips)
        .unwrap();
    assert!(!visible, "Rive's default clip is ignored in CSS mode");
}

/// A CSS viewport's scroll range is its scrollable overflow less its padding
/// box: a 120-point block list in an 80-point view stops at 40.
#[test]
fn css_scroll_range_comes_from_scrollable_overflow() {
    const OVERFLOW_Y: u16 = 61004;
    let item = |name| css(name, height(60.0), vec![]);
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [[U(OVERFLOW_Y, 3)].as_slice(), &width(200.0), &height(80.0)].concat(),
            vec![B {
                scroll: Some(1),
                ..css("List", [], vec![item("One"), item("Two")])
            }],
        ),
    );
    assert_eq!(f.bounds("List"), [0.0, 0.0, 200.0, 120.0]);
    assert_eq!(f.max_offsets().1, -40.0);
}

/// CSS `position`: a static box is not a containing block, so an absolute
/// grandchild is placed against the positioned frame, and its bounds stay
/// relative to its own parent.
#[test]
fn css_absolute_box_skips_a_static_parent() {
    const MARGIN_LEFT: u16 = 61049;
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [[U(POSITION, 1)].as_slice(), &width(300.0), &height(200.0)].concat(),
            vec![css(
                "Static",
                [
                    [F(MARGIN_LEFT, 50.0)].as_slice(),
                    &width(100.0),
                    &height(40.0),
                ]
                .concat(),
                vec![css(
                    "Pinned",
                    [
                        [
                            U(POSITION, 2),
                            U(INSET_LEFT_KIND, 1),
                            F(INSET_LEFT, 10.0),
                            U(INSET_TOP_KIND, 1),
                            F(INSET_TOP, 10.0),
                        ]
                        .as_slice(),
                        &width(30.0),
                        &height(30.0),
                    ]
                    .concat(),
                    vec![],
                )],
            )],
        ),
    );
    assert_eq!(f.bounds("Static"), [50.0, 0.0, 100.0, 40.0]);
    // (10, 10) in the frame is (-40, 10) in the static box.
    assert_eq!(f.bounds("Pinned"), [-40.0, 10.0, 30.0, 30.0]);
}

/// A participant's host carries the record: the participant takes its item
/// properties (size, margins) and ignores its Rive sizing.
#[test]
fn css_record_on_a_participant_host() {
    const MARGIN_LEFT: u16 = 61049;
    let f = scene(
        (400.0, 300.0),
        B {
            participants: vec![(
                Rect {
                    name: "Icon",
                    size: (80.0, 30.0),
                    grouped: false,
                    path_scale_x: 1.0,
                },
                Some(
                    [
                        [F(MARGIN_LEFT, 10.0)].as_slice(),
                        &width(50.0),
                        &height(20.0),
                    ]
                    .concat(),
                ),
            )],
            ..css(
                "Row",
                [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
                vec![],
            )
        },
    );
    let participant = f
        .artboard
        .with_artboard(|artboard| {
            artboard
                .find_all_handles::<crate::source::layout::layout_participant::LayoutParticipant>()
        })
        .into_iter()
        .next()
        .expect("a participant");
    let bounds = participant
        .with_downcast::<crate::source::layout::layout_participant::LayoutParticipant, _>(
            |participant| participant.layout_bounds(),
        )
        .unwrap();
    assert_eq!(
        [bounds.left(), bounds.top(), bounds.width(), bounds.height()],
        [10.0, 0.0, 50.0, 20.0]
    );
}

/// Translated mode: Rive reads the authored flexBasis only for Fill. The
/// editor writes 0 points on every style, so a Fixed 50% box beside a Fill
/// keeps its 50% (computed_values_test), and the Fill takes the rest.
#[test]
fn translated_fixed_box_ignores_the_authored_flex_basis() {
    let basis = [
        U(
            LayoutComponentStyleBase::FLEX_BASIS_UNITS_VALUE_PROPERTY_KEY,
            YGUnit::Point as u64,
        ),
        F(LayoutComponentStyleBase::FLEX_BASIS_PROPERTY_KEY, 0.0),
    ];
    let row_style = |extra: &[P]| {
        [
            [U(
                LayoutComponentStyleBase::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
                YGFlexDirection::Row as u64,
            )]
            .as_slice(),
            &basis,
            extra,
        ]
        .concat()
    };
    let f = scene(
        (400.0, 100.0),
        B {
            name: "Row",
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 400.0),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0),
            ],
            style: row_style(&[]),
            children: vec![
                B {
                    name: "Half",
                    component: vec![
                        F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 50.0),
                        F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0),
                    ],
                    style: [
                        basis.as_slice(),
                        &[U(
                            LayoutSizingStyleBase::WIDTH_UNITS_VALUE_PROPERTY_KEY,
                            YGUnit::Percent as u64,
                        )],
                    ]
                    .concat(),
                    ..B::default()
                },
                B {
                    name: "Rest",
                    component: vec![F(LayoutComponentBase::FRACTIONAL_WIDTH_PROPERTY_KEY, 1.0)],
                    style: [
                        basis.as_slice(),
                        &[U(
                            LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                            LayoutScaleType::Fill as u64,
                        )],
                    ]
                    .concat(),
                    ..B::default()
                },
            ],
            ..B::default()
        },
    );
    assert_eq!(f.bounds("Half")[2], 200.0);
    assert_eq!(f.bounds("Rest"), [200.0, 0.0, 200.0, 0.0]);
}

/// A participant host's record with `display: none` hides the host, as a
/// participant's own display none does, and takes it out of flow.
#[test]
fn css_display_none_on_a_participant_hides_its_host() {
    let f = scene(
        (400.0, 300.0),
        B {
            participants: vec![
                (
                    Rect {
                        name: "Gone",
                        size: (80.0, 30.0),
                        grouped: false,
                        path_scale_x: 1.0,
                    },
                    Some([[U(DISPLAY, NONE as u64)].as_slice(), &width(50.0)].concat()),
                ),
                (
                    Rect {
                        name: "Kept",
                        size: (80.0, 30.0),
                        grouped: false,
                        path_scale_x: 1.0,
                    },
                    Some(width(40.0).to_vec()),
                ),
            ],
            ..css(
                "Row",
                [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
                vec![],
            )
        },
    );
    let shape = |name: &str| {
        f.artboard
            .with_artboard(|artboard| {
                artboard.find_all_handles::<crate::source::shapes::shape::Shape>()
            })
            .into_iter()
            .find(|handle| {
                handle
                    .with(|object| {
                        object
                            .as_component()
                            .is_some_and(|component| component.base.name() == name)
                    })
                    .unwrap_or(false)
            })
            .unwrap()
    };
    let collapsed = |name: &str| {
        shape(name)
            .with(|object| object.as_component().unwrap().is_collapsed())
            .unwrap()
    };
    assert!(collapsed("Gone"));
    assert!(!collapsed("Kept"));
    // The kept participant starts the row: the hidden one is out of flow.
    let kept = f
        .artboard
        .with_artboard(|artboard| {
            artboard
                .find_all_handles::<crate::source::layout::layout_participant::LayoutParticipant>()
        })
        .into_iter()
        .map(|handle| {
            handle
                .with_downcast::<crate::source::layout::layout_participant::LayoutParticipant, _>(
                    |participant| participant.layout_bounds(),
                )
                .unwrap()
        })
        .find(|bounds| bounds.width() == 40.0)
        .expect("the kept participant");
    assert_eq!(kept.left(), 0.0);
}

/// `display: grid` in CSS mode lays out with Rive's own GridTrack records:
/// columns of 100 points and 1fr in a 300-point box.
#[test]
fn css_grid_uses_rive_grid_tracks() {
    const POINTS: u64 = 1;
    const FR: u64 = 3;
    let f = scene(
        (400.0, 300.0),
        B {
            tracks: vec![(0, POINTS, 100.0), (0, FR, 1.0)],
            ..css(
                "Grid",
                [[U(DISPLAY, 2)].as_slice(), &width(300.0), &height(40.0)].concat(),
                vec![css("First", [], vec![]), css("Second", [], vec![])],
            )
        },
    );
    assert_eq!(f.bounds("First"), [0.0, 0.0, 100.0, 40.0]);
    assert_eq!(f.bounds("Second"), [100.0, 0.0, 200.0, 40.0]);
}

/// CSS mode: an image is content at its natural size, as an `<img>` is: a
/// box hugs it, and layout does not scale it to a wider box.
#[test]
fn css_image_is_content_at_its_natural_size() {
    let image_scale = |f: &Scene| {
        f.artboard
            .with_artboard(|artboard| {
                artboard.find_all_handles::<crate::source::shapes::image::Image>()
            })
            .into_iter()
            .next()
            .expect("an image")
            .with(|object| {
                let node = object.as_node().expect("image node");
                (node.base.scale_x(), node.base.scale_y())
            })
            .unwrap()
    };
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(80.0)].concat(),
            vec![B {
                images: vec!["Picture"],
                ..css("Hug", [], vec![])
            }],
        ),
    );
    // The box hugs the image's 100-point width; it stretches to the row's
    // 80-point height, and the image keeps its own scale.
    assert_eq!(f.bounds("Hug"), [0.0, 0.0, 100.0, 80.0]);
    assert_eq!(image_scale(&f), (1.0, 1.0));
}

fn border_box(props: &[P]) -> Vec<P> {
    [
        [
            U(BOX_SIZING, 1),
            F(BORDER_TOP, 4.0),
            F(BORDER_RIGHT, 4.0),
            F(BORDER_BOTTOM, 4.0),
            F(BORDER_LEFT, 4.0),
        ]
        .as_slice(),
        &width(100.0),
        &height(50.0),
        props,
    ]
    .concat()
}

fn aabb(left: f32, top: f32, right: f32, bottom: f32) -> crate::source::math::aabb::Aabb {
    crate::source::math::aabb::Aabb::new(left, top, right, bottom)
}

/// CSS's overflow clips a box's content, not the box: its background and
/// border draw before its padding-box clip, and its children inside it.
#[test]
fn css_box_paints_itself_before_clipping_its_children() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Fill(0xff11_2233), Paint::Border(0xff44_5566, 15)],
            ..css(
                "Clip",
                border_box(&[U(OVERFLOW_X, 2), U(OVERFLOW_Y, 2)]),
                vec![B {
                    paints: vec![Paint::Fill(0xff77_8899)],
                    ..css("Child", [width(200.0), height(200.0)].concat(), vec![])
                }],
            )
        },
    );
    let lines = f.draw_lines();
    let draw = |color: &str| {
        lines
            .iter()
            .position(|line| line.starts_with("drawPath") && line.contains(color))
            .unwrap_or_else(|| panic!("a draw with {color}:\n{}", lines.join("\n")))
    };
    let clip = lines
        .iter()
        .rposition(|line| line.starts_with("clipPath"))
        .expect("the box's clip");
    assert!(draw("color=0xff112233") < clip, "{}", lines.join("\n"));
    assert!(draw("color=0xff445566") < clip, "{}", lines.join("\n"));
    assert!(clip < draw("color=0xff778899"), "{}", lines.join("\n"));
}

/// A BoxBorder's sides written at run time redraw its ring.
#[test]
fn css_border_sides_written_at_run_time_redraw_the_border() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Border(0xff44_5566, 15)],
            ..css("Edge", border_box(&[]), vec![])
        },
    );
    assert_eq!(f.css_paints("Edge")[0].1, None);
    assert!(CoreRegistry::set_uint_handle(
        &f.paint_child("Edge", BOX_BORDER),
        BORDER_SIDES.into(),
        1
    ));
    f.advance(2);
    // Only the top side: its wedge from the outer to the inner corners.
    assert_eq!(f.css_paints("Edge")[0].1, Some(aabb(0.0, 0.0, 100.0, 4.0)));
}

/// A BoxShadow's spread written at run time regrows the shadow.
#[test]
fn css_shadow_spread_written_at_run_time_regrows_the_shadow() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Shadow {
                color: 0x8000_0000,
                strength: 0.0,
                spread: 4.0,
            }],
            ..css("Lifted", border_box(&[]), vec![])
        },
    );
    assert_eq!(f.css_paints("Lifted")[0].0, aabb(-4.0, -4.0, 104.0, 54.0));
    assert!(CoreRegistry::set_double_handle(
        &f.paint_child("Lifted", BOX_SHADOW),
        SHADOW_SPREAD.into(),
        20.0
    ));
    f.advance(2);
    assert_eq!(f.css_paints("Lifted")[0].0, aabb(-20.0, -20.0, 120.0, 70.0));
}

/// World-space offsets leave outer and inset clips at the box, including
/// under rotation and scale. A singular transform paints nothing.
#[test]
fn css_world_shadow_offset_keeps_its_clip_at_the_box() {
    use crate::source::math::mat2d::Mat2D;
    for inset in [false, true] {
        let shadow = if inset {
            Paint::InsetShadow {
                color: 0xff00_0000,
                strength: 0.0,
                spread: 0.0,
            }
        } else {
            Paint::Shadow {
                color: 0xff00_0000,
                strength: 0.0,
                spread: 0.0,
            }
        };
        let f = scene(
            (400.0, 300.0),
            B {
                paints: vec![shadow],
                ..css("Lifted", border_box(&[]), vec![])
            },
        );
        let feather = f.paint_child("Lifted", FeatherBase::TYPE_KEY);
        assert!(CoreRegistry::set_uint_handle(
            &feather,
            FeatherBase::SPACE_VALUE_PROPERTY_KEY.into(),
            0
        ));
        for (key, value) in [
            (FeatherBase::OFFSET_X_PROPERTY_KEY, 10.0),
            (FeatherBase::OFFSET_Y_PROPERTY_KEY, 12.0),
        ] {
            assert!(CoreRegistry::set_double_handle(&feather, key.into(), value));
        }
        f.advance(2);
        let owner = f.named("Lifted");
        let paint = owner
            .with_downcast::<LayoutComponent, _>(|layout| layout.paints.shape_paints()[0].clone())
            .unwrap();
        let draw = |world| {
            let before = f.factory.borrow().stream().lines().count();
            let mut renderer = f.factory.borrow().make_renderer();
            assert_eq!(
                owner.with_downcast_mut::<LayoutComponent, _>(|layout| {
                    layout.draw_css_paint_with(&mut renderer, &paint, world, None)
                }),
                Some(true)
            );
            f.stream_lines()
                .into_iter()
                .skip(before)
                .filter(|line| {
                    matches!(line.as_str(), "save" | "restore")
                        || line.starts_with("transform ")
                        || line.starts_with("clipPath")
                        || line.starts_with("drawPath")
                })
                .collect::<Vec<_>>()
        };
        let lines = draw(Mat2D::new(0.0, 2.0, -4.0, 0.0, 50.0, 70.0));
        assert_eq!(lines[0], "save");
        assert_eq!(lines[1], "transform matrix=[0,2,-4,0,50,70]");
        assert!(lines[2].starts_with("clipPath"), "{lines:?}");
        let corners = if inset {
            ["(4,4)", "(96,4)", "(96,46)", "(4,46)"]
        } else {
            ["(0,0)", "(100,0)", "(100,50)", "(0,50)"]
        };
        for corner in corners {
            assert!(lines[2].contains(corner), "{lines:?}");
        }
        assert_eq!(lines[3], "transform matrix=[1,0,0,1,6,-2.5]");
        assert!(lines[4].starts_with("drawPath"), "{lines:?}");
        assert_eq!(lines[5], "restore");
        assert!(draw(Mat2D::from_scale(0.0, 2.0)).is_empty());
    }
}

/// A world-space offset grows in box coordinates when the box shrinks.
/// Its outer clip and inset outline must still cover the visible shadow.
#[test]
fn css_world_shadow_downscaled_offset_expands_its_regions_and_bounds() {
    use crate::source::{
        generated::transform_component_base::TransformComponentBase, math::mat2d::Mat2D,
    };
    for inset in [false, true] {
        let shadow = if inset {
            Paint::InsetShadow {
                color: 0xff00_0000,
                strength: 0.0,
                spread: 5.0,
            }
        } else {
            Paint::Shadow {
                color: 0xff00_0000,
                strength: 0.0,
                spread: 5.0,
            }
        };
        let f = scene(
            (400.0, 300.0),
            B {
                paints: vec![shadow],
                ..css("Lifted", [width(100.0), height(60.0)].concat(), vec![])
            },
        );
        let owner = f.named("Lifted");
        let feather = f.paint_child("Lifted", FeatherBase::TYPE_KEY);
        assert!(CoreRegistry::set_uint_handle(
            &feather,
            FeatherBase::SPACE_VALUE_PROPERTY_KEY.into(),
            0
        ));
        assert!(CoreRegistry::set_double_handle(
            &feather,
            FeatherBase::OFFSET_X_PROPERTY_KEY.into(),
            10.0
        ));
        for key in [
            TransformComponentBase::SCALE_X_PROPERTY_KEY,
            TransformComponentBase::SCALE_Y_PROPERTY_KEY,
        ] {
            assert!(CoreRegistry::set_double_handle(&owner, key.into(), 0.1));
        }
        f.advance(2);
        let lines = f
            .draw_lines()
            .into_iter()
            .filter(|line| !line.starts_with("make"))
            .collect::<Vec<_>>();
        let draw = draw_at(&lines, 0xff00_0000);
        let offset = lines[draw - 1]
            .strip_prefix("transform matrix=[1,0,0,1,")
            .and_then(|line| line.strip_suffix(",0]"))
            .expect("the shadow's local translation")
            .parse::<f32>()
            .unwrap();
        assert!((offset - 100.0).abs() < 0.001);
        let clip = lines[..draw]
            .iter()
            .rposition(|line| line.starts_with("clipPath"))
            .expect("the stationary shadow clip");
        assert!(clip < draw - 1, "{lines:?}");
        let (path, clip) = f.css_paints("Lifted")[0];
        let world = Mat2D::from_scale(0.1, 0.1);
        let drawn = world.map_bounding_box(path.offset(offset, 0.0));
        let clip = world.map_bounding_box(clip.expect("the shadow clip"));
        if inset {
            // The translated outline must cover the entire padding box.
            assert!(drawn.left() <= 0.0 && drawn.right() >= 10.0, "{drawn:?}");
            assert_eq!(clip, aabb(0.0, 0.0, 10.0, 6.0));
        } else {
            // 0.1 * (100 + 5) + 10, measured in the parent's space.
            assert!((drawn.right() - 20.5).abs() < 0.001, "{drawn:?}");
            assert!(clip.right() >= drawn.right(), "{clip:?}");
            let mut bounds = aabb(0.0, 0.0, 0.0, 0.0);
            owner.with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.painted_world_bounds(&mut bounds)
            });
            assert!((bounds.right() - 20.5).abs() < 0.001, "{bounds:?}");
            // Bounds can also be requested in unscaled box coordinates.
            owner.with_downcast::<LayoutComponent, _>(|layout| {
                layout.css_painted_bounds(
                    aabb(0.0, 0.0, 100.0, 60.0),
                    &Mat2D::identity(),
                    &mut bounds,
                )
            });
            assert!((bounds.right() - 205.0).abs() < 0.001, "{bounds:?}");
        }
    }
}

/// A shadow's Feather strength written at run time widens the region the
/// shadow may draw in, which reaches 1.5 times the strength past the box.
#[test]
fn css_shadow_feather_written_at_run_time_widens_its_reach() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Shadow {
                color: 0x8000_0000,
                strength: 10.0,
                spread: 0.0,
            }],
            ..css("Lifted", border_box(&[]), vec![])
        },
    );
    // 1.5 times 10, plus a margin of 2.
    assert_eq!(
        f.css_paints("Lifted")[0].1,
        Some(aabb(-17.0, -17.0, 117.0, 67.0))
    );
    assert!(CoreRegistry::set_double_handle(
        &f.paint_child("Lifted", FeatherBase::TYPE_KEY),
        FeatherBase::STRENGTH_PROPERTY_KEY.into(),
        20.0
    ));
    f.advance(2);
    assert_eq!(
        f.css_paints("Lifted")[0].1,
        Some(aabb(-32.0, -32.0, 132.0, 82.0))
    );
}

/// A BoxGradient's angle written at run time moves its gradient line.
#[test]
fn css_gradient_angle_written_at_run_time_moves_the_gradient() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Gradient(180.0)],
            ..css("Painted", border_box(&[]), vec![])
        },
    );
    let last_gradient = || {
        let lines = f.stream_lines();
        let line = lines
            .iter()
            .rev()
            .find(|line| line.starts_with("makeLinearGradient"))
            .expect("a linear gradient")
            .clone();
        let numbers = |label: &str| -> (f32, f32) {
            let rest = &line[line.find(label).expect(label) + label.len()..];
            let (x, rest) = rest.split_once(',').unwrap();
            let y = rest.split_once(')').unwrap().0;
            (x.parse().unwrap(), y.parse().unwrap())
        };
        let (start, end) = (numbers("start=("), numbers("end=("));
        [start.0, start.1, end.0, end.1].map(|value| (value * 1000.0).round() / 1000.0)
    };
    // To the bottom, down the middle of the padding box: the 100 by 50
    // box less its 4-point border.
    assert_eq!(last_gradient(), [50.0, 4.0, 50.0, 46.0]);
    assert!(CoreRegistry::set_double_handle(
        &f.paint_child("Painted", BOX_GRADIENT),
        GRADIENT_ANGLE.into(),
        90.0
    ));
    f.advance(2);
    // To the right, across the middle.
    assert_eq!(last_gradient(), [4.0, 25.0, 96.0, 25.0]);
}

/// A CSS box whose overflow becomes hidden at run time clips its
/// children, though it has no paints of its own.
#[test]
fn css_overflow_written_at_run_time_clips() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [width(100.0), height(50.0)].concat(),
            vec![B {
                paints: vec![Paint::Fill(0xff77_8899)],
                ..css("Child", [width(200.0), height(200.0)].concat(), vec![])
            }],
        ),
    );
    let clips = |lines: Vec<String>| {
        lines
            .iter()
            .filter(|line| line.starts_with("clipPath"))
            .count()
    };
    let before = clips(f.draw_lines());
    assert!(CoreRegistry::set_uint_handle(
        &f.record("Frame"),
        OVERFLOW_X.into(),
        2
    ));
    f.advance(2);
    assert_eq!(clips(f.draw_lines()), before + 1);
}

/// A direction written at run time reaches the boxes that inherit it.
#[test]
fn css_direction_written_at_run_time_cascades() {
    const COLUMN: u64 = 2;
    const RTL: u32 = 2;
    let f = scene(
        (400.0, 300.0),
        css(
            "Outer",
            [
                [U(DISPLAY, 1), U(61006, COLUMN)].as_slice(),
                &width(200.0),
                &height(100.0),
            ]
            .concat(),
            vec![css(
                "Inner",
                [[U(DISPLAY, 1)].as_slice(), &width(200.0), &height(20.0)].concat(),
                vec![css("Dot", [width(20.0), height(20.0)].concat(), vec![])],
            )],
        ),
    );
    assert_eq!(f.bounds("Dot")[0], 0.0);
    assert!(CoreRegistry::set_uint_handle(
        &f.record("Outer"),
        DIRECTION.into(),
        RTL
    ));
    f.advance(2);
    assert_eq!(f.bounds("Dot")[0], 180.0);
}

/// A participant host's record decides whether it shows: a visible record
/// shows it though its Rive display is none.
#[test]
fn css_record_shows_a_participant_whose_rive_display_is_none() {
    const DISPLAY_VALUE: u16 = 596;
    let f = scene(
        (400.0, 300.0),
        B {
            participants: vec![(
                Rect {
                    name: "Icon",
                    size: (80.0, 30.0),
                    grouped: false,
                    path_scale_x: 1.0,
                },
                Some([width(50.0), height(20.0)].concat()),
            )],
            ..css(
                "Row",
                [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(40.0)].concat(),
                vec![],
            )
        },
    );
    let participant = f
        .artboard
        .with_artboard(|artboard| {
            artboard
                .find_all_handles::<crate::source::layout::layout_participant::LayoutParticipant>()
        })
        .into_iter()
        .next()
        .expect("a participant");
    assert!(CoreRegistry::set_uint_handle(
        &participant,
        DISPLAY_VALUE.into(),
        1
    ));
    f.advance(2);
    let bounds = participant
        .with_downcast::<crate::source::layout::layout_participant::LayoutParticipant, _>(
            |participant| participant.layout_bounds(),
        )
        .unwrap();
    assert_eq!((bounds.width(), bounds.height()), (50.0, 20.0));
}

/// Hit testing follows the clip's axes: `overflow-x: clip` with a visible
/// y axis passes points below the box and stops points beside it.
#[test]
fn css_hit_test_follows_the_clipped_axes() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Strip",
            [[U(OVERFLOW_X, 1)].as_slice(), &width(100.0), &height(50.0)].concat(),
            vec![css("Tall", [width(100.0), height(200.0)].concat(), vec![])],
        ),
    );
    let hits = |x: f32, y: f32| {
        f.named("Strip")
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.hit_test_point(&crate::source::math::vec2d::Vec2D::new(x, y), true, false)
            })
            .unwrap()
    };
    assert!(hits(50.0, 25.0));
    assert!(hits(50.0, 150.0));
    assert!(!hits(150.0, 25.0));
}

/// A CSS box measures its shape content with the path's own transforms:
/// an 80-point rectangle path scaled by 2 is 160 wide.
#[test]
fn css_shape_content_is_measured_with_its_path_transform() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(400.0), &height(100.0)].concat(),
            vec![B {
                content: vec![Rect {
                    name: "Wide",
                    size: (80.0, 30.0),
                    grouped: false,
                    path_scale_x: 2.0,
                }],
                ..css("Leaf", [], vec![])
            }],
        ),
    );
    assert_eq!(f.bounds("Leaf")[2], 160.0);
}

/// A CSS box's flex direction written at run time re-translates its Rive
/// children, whose Fill follows their parent's main axis.
#[test]
fn css_flex_direction_written_at_run_time_retranslates_rive_children() {
    const FLEX_DIRECTION: u16 = 61006;
    const COLUMN: u32 = 2;
    let f = scene(
        (300.0, 100.0),
        css(
            "Flow",
            [[U(DISPLAY, 1)].as_slice(), &width(300.0), &height(100.0)].concat(),
            vec![
                B {
                    name: "Grow",
                    component: vec![
                        F(LayoutComponentBase::FRACTIONAL_WIDTH_PROPERTY_KEY, 1.0),
                        F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 20.0),
                    ],
                    style: vec![U(
                        LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                        LayoutScaleType::Fill as u64,
                    )],
                    ..B::default()
                },
                B {
                    name: "Fixed",
                    component: vec![
                        F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0),
                        F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 20.0),
                    ],
                    ..B::default()
                },
            ],
        ),
    );
    assert_eq!(f.bounds("Grow"), [0.0, 0.0, 200.0, 20.0]);
    assert!(CoreRegistry::set_uint_handle(
        &f.record("Flow"),
        FLEX_DIRECTION.into(),
        COLUMN
    ));
    f.advance(2);
    // Its Fill width is now across the column: it stretches, and it keeps
    // its fixed height along the column.
    assert_eq!(f.bounds("Grow"), [0.0, 0.0, 300.0, 20.0]);
}

/// A CSS leaf keeps its shape content's CSS size as its automatic minimum:
/// an 80-point path scaled by 2 holds a 160-point box in a 100-point row.
#[test]
fn css_shape_content_minimum_uses_its_path_transform() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(100.0), &height(100.0)].concat(),
            vec![B {
                content: vec![Rect {
                    name: "Wide",
                    size: (80.0, 30.0),
                    grouped: false,
                    path_scale_x: 2.0,
                }],
                ..css("Leaf", [], vec![])
            }],
        ),
    );
    assert_eq!(f.bounds("Leaf")[2], 160.0);
}

/// A CSS leaf measures a shape's paths together: rectangles of 80 points
/// centred at x 0 and x 120 span 200.
#[test]
fn css_shape_paths_are_measured_together() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(400.0), &height(100.0)].concat(),
            vec![B {
                multi_path: vec![("Pair", vec![(0.0, (80.0, 30.0)), (120.0, (80.0, 30.0))])],
                ..css("Leaf", [], vec![])
            }],
        ),
    );
    assert_eq!(f.bounds("Leaf")[2], 200.0);
}

/// A CSS grid reads its GridTracks whatever the Rive style's layout type:
/// a style left as a stack does not turn the grid into one cell.
#[test]
fn css_grid_ignores_a_rive_stack_layout_type() {
    const POINTS: u64 = 1;
    const STACK: u64 = 2;
    let f = scene(
        (400.0, 300.0),
        B {
            tracks: vec![(0, POINTS, 100.0), (0, POINTS, 200.0)],
            style: vec![U(
                LayoutComponentStyleBase::LAYOUT_TYPE_VALUE_PROPERTY_KEY,
                STACK,
            )],
            ..css(
                "Grid",
                [[U(DISPLAY, 2)].as_slice(), &width(300.0), &height(40.0)].concat(),
                vec![css("First", [], vec![]), css("Second", [], vec![])],
            )
        },
    );
    assert_eq!(f.bounds("First"), [0.0, 0.0, 100.0, 40.0]);
    assert_eq!(f.bounds("Second"), [100.0, 0.0, 200.0, 40.0]);
}

/// A rounded per-side border reaches into its corners: the top side's
/// region follows each corner's join line until the corner's curve ends,
/// so the curved ring inside the padding rectangle is drawn.
#[test]
fn css_rounded_side_border_reaches_into_its_corners() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Border(0xff44_5566, 1)],
            style: vec![F(
                LayoutComponentStyleBase::CORNER_RADIUS_TL_PROPERTY_KEY,
                40.0,
            )],
            ..css(
                "Edge",
                [
                    [
                        U(BOX_SIZING, 1),
                        F(BORDER_TOP, 4.0),
                        F(BORDER_RIGHT, 4.0),
                        F(BORDER_BOTTOM, 4.0),
                        F(BORDER_LEFT, 4.0),
                    ]
                    .as_slice(),
                    &width(100.0),
                    &height(100.0),
                ]
                .concat(),
                vec![],
            )
        },
    );
    // Each join line runs from an outer corner through the inner corner
    // to the edge of the corner's 40-point curve.
    assert_eq!(f.css_paints("Edge")[0].1, Some(aabb(0.0, 0.0, 100.0, 40.0)));
}

/// Hit testing through a CSS box's clip uses its padding box and inner
/// radii, as its children are drawn; the box itself keeps its border box.
#[test]
fn css_clip_hit_test_uses_the_padding_box() {
    let f = scene(
        (400.0, 300.0),
        B {
            style: vec![F(
                LayoutComponentStyleBase::CORNER_RADIUS_TL_PROPERTY_KEY,
                30.0,
            )],
            ..css(
                "Frame",
                [
                    [
                        U(BOX_SIZING, 1),
                        U(OVERFLOW_X, 2),
                        U(OVERFLOW_Y, 2),
                        F(BORDER_TOP, 10.0),
                        F(BORDER_RIGHT, 10.0),
                        F(BORDER_BOTTOM, 10.0),
                        F(BORDER_LEFT, 10.0),
                    ]
                    .as_slice(),
                    &width(100.0),
                    &height(100.0),
                ]
                .concat(),
                vec![],
            )
        },
    );
    let hits = |x: f32, y: f32, primary: bool| {
        f.named("Frame")
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.hit_test_point(&crate::source::math::vec2d::Vec2D::new(x, y), true, primary)
            })
            .unwrap()
    };
    // In the border: clipped for children, part of the box itself.
    assert!(!hits(5.0, 50.0, false));
    assert!(hits(5.0, 50.0, true));
    // Inside the padding rectangle but outside its 20-point inner corner.
    assert!(!hits(12.0, 12.0, false));
    assert!(hits(50.0, 50.0, false));
}

/// A CSS box's overflow clips its children's rendered semantic bounds: a
/// child wholly outside an `overflow: hidden` box has none.
#[test]
fn css_overflow_clips_semantic_bounds() {
    const COLUMN: u64 = 2;
    let fixed = |name: &'static str, h: f32| {
        css(
            name,
            [[F(FLEX_SHRINK, 0.0)].as_slice(), &width(100.0), &height(h)].concat(),
            vec![],
        )
    };
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [
                [U(DISPLAY, 1), U(61006, COLUMN), U(OVERFLOW_Y, 2)].as_slice(),
                &width(100.0),
                &height(50.0),
            ]
            .concat(),
            vec![fixed("In", 50.0), fixed("Out", 20.0)],
        ),
    );
    use crate::source::semantic::semantic_provider::rendered_geometry_bounds;
    let inside = rendered_geometry_bounds(Some(&f.named("In")));
    assert_eq!(
        (inside.min_x, inside.min_y, inside.max_x, inside.max_y),
        (0.0, 0.0, 100.0, 50.0)
    );
    assert!(rendered_geometry_bounds(Some(&f.named("Out"))).is_empty_or_nan());
}

/// A shadow's spread counts towards its box's painted bounds.
#[test]
fn css_shadow_spread_is_in_the_painted_bounds() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Shadow {
                color: 0x8000_0000,
                strength: 0.0,
                spread: 30.0,
            }],
            ..css("Lifted", border_box(&[]), vec![])
        },
    );
    let mut bounds = crate::source::math::aabb::Aabb::default();
    f.named("Lifted")
        .with_downcast_mut::<LayoutComponent, _>(|layout| layout.painted_world_bounds(&mut bounds))
        .unwrap();
    assert_eq!(bounds, aabb(-30.0, -30.0, 130.0, 80.0));
}

const PADDING_PROPS: [P; 4] = [
    F(PADDING_TOP, 6.0),
    F(PADDING_RIGHT, 30.0),
    F(PADDING_BOTTOM, 4.0),
    F(PADDING_LEFT, 10.0),
];

/// A padded CSS leaf with content authored at (0, 0): a shape, a shape
/// in a group and an image. Its 2-point border and its padding place the
/// content box at (12, 8).
fn padded_leaf(extra: &[P], bind: Option<u16>) -> Scene {
    let rect = |name, grouped| Rect {
        name,
        size: (40.0, 20.0),
        grouped,
        path_scale_x: 1.0,
    };
    scene(
        (400.0, 300.0),
        B {
            content: vec![rect("Dot", false), rect("Grouped", true)],
            images: vec!["Picture"],
            bind,
            ..css(
                "Pad",
                [
                    [
                        F(BORDER_TOP, 2.0),
                        F(BORDER_RIGHT, 2.0),
                        F(BORDER_BOTTOM, 2.0),
                        F(BORDER_LEFT, 2.0),
                    ]
                    .as_slice(),
                    &PADDING_PROPS,
                    extra,
                ]
                .concat(),
                vec![],
            )
        },
    )
}

/// A CSS leaf places its content at its content box's origin, past its
/// border and padding, as a browser does.
#[test]
fn css_leaf_places_its_content_at_the_content_box() {
    let f = padded_leaf(&[], None);
    for name in ["Dot", "Grouped", "Picture"] {
        assert_eq!(f.world_origin(name), (12.0, 8.0), "{name}");
    }
}

/// One anonymous item lets each display apply its own alignment to text.
#[test]
fn css_leaf_text_alignment_uses_its_display() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    // display, align-items, flex-direction, justify-content, align-content,
    // padding and the fraction of the free block space before the text.
    for (display, align, direction, justify, content, padded, fraction) in [
        (0, 0, 0, 0, 0, false, 0.0),
        (1, 4, 0, 0, 0, false, 0.0),
        (1, 8, 0, 0, 0, false, 0.5),
        (1, 5, 0, 0, 0, false, 1.0),
        (1, 0, 2, 5, 0, false, 0.5),
        (0, 0, 0, 0, 5, false, 0.5),
        (2, 8, 0, 0, 0, false, 0.5),
        (2, 0, 0, 0, 5, false, 0.5),
        (1, 8, 0, 0, 0, true, 0.5),
    ] {
        let f = CssTextLeaf::new(
            "Hello wonderful\nworld",
            &[
                (DISPLAY, display),
                (61013, align),
                (61006, direction),
                (61018, justify),
                (61017, content),
                (WIDTH_KIND, 1),
                (HEIGHT_KIND, 1),
            ],
            &[(WIDTH, 338.0), (HEIGHT, 120.0)],
        );
        if padded {
            CssTextLeaf::write(
                &f.record,
                &[],
                &[
                    (PADDING_TOP, 10.0),
                    (PADDING_BOTTOM, 10.0),
                    (PADDING_LEFT, 12.0),
                    (PADDING_RIGHT, 12.0),
                ],
            );
        }
        let measured = f.measure(Some(338.0));
        assert!(measured.y > 36.0 && measured.y < 120.0);
        f.solve();
        near(
            f.first_line_position().y,
            (if padded { 10.0 } else { 0.0 }) + (120.0 - measured.y) * fraction,
        );
        if fraction != 0.0 {
            near(f.text_size().y, measured.y);
        }
        // The retained child must be invalidated when alignment changes.
        CssTextLeaf::write(&f.record, &[(61013, 4), (61018, 0), (61017, 0)], &[]);
        f.solve();
        near(f.offset().y, if padded { 10.0 } else { 0.0 });
    }
}

#[test]
fn css_leaf_row_text_keeps_its_intrinsic_width_before_text_align() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    use crate::source::text::text::Text;
    for (display, direction, justify, fraction, stretched) in [
        (1, 0, 0, 0.0, false),
        (1, 0, 5, 0.5, false),
        (0, 0, 0, 0.5, true),
        (1, 2, 0, 0.5, true),
        (2, 0, 0, 0.5, true),
    ] {
        let f = CssTextLeaf::new(
            "Short label",
            &[
                (DISPLAY, display),
                (61006, direction),
                (61018, justify),
                (WIDTH_KIND, 1),
            ],
            &[(WIDTH, 338.0)],
        );
        f.text
            .with_downcast_mut::<Text, _>(|text| {
                text.base.set_align_value_value(2);
            })
            .unwrap();
        let line = f.measure(None).x;
        assert!(line < 338.0);
        f.solve();
        let size = f.text_size();
        near(size.x, if stretched { 338.0 } else { line });
        near(f.first_line_position().x, (338.0 - line) * fraction);
    }
}

#[test]
fn css_leaf_grid_item_size_changes_propagate_without_a_box_resize() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    const JUSTIFY_ITEMS: u16 = 61015;
    for borrowed in [false, true] {
        let f = CssTextLeaf::new(
            "Hello",
            &[
                (DISPLAY, 2),
                (JUSTIFY_ITEMS, 10),
                (WIDTH_KIND, 1),
                (HEIGHT_KIND, 1),
            ],
            &[(WIDTH, 300.0), (HEIGHT, 100.0)],
        );
        let line = f.measure(None).x;
        assert!(line > 0.0 && line < 300.0);
        let outer = f.solve();
        near(f.text_size().x, 300.0);
        for justify in [8, 10, 8] {
            CssTextLeaf::write(&f.record, &[(JUSTIFY_ITEMS, justify)], &[]);
            LayoutComponent::sync_style_occurrence(&f.label);
            LayoutComponent::calculate_layout_occurrence(&f.root, f32::NAN, f32::NAN);
            if borrowed {
                f.label
                    .with_downcast_mut::<LayoutComponent, _>(|layout| {
                        assert!(!layout.animates());
                        layout.update_layout_bounds(false);
                        assert!(!layout.layout_data.has_new_layout);
                    })
                    .unwrap();
            } else {
                LayoutComponent::update_layout_bounds_occurrence(&f.label, false);
            }
            let next = f
                .label
                .with_downcast::<LayoutComponent, _>(|layout| layout.layout_data.solved_layout)
                .unwrap();
            assert_eq!(next, outer);
            near(f.text_size().x, if justify == 8 { line } else { 300.0 });
            near(
                f.offset().x,
                if justify == 8 {
                    (300.0 - line) / 2.0
                } else {
                    0.0
                },
            );
        }
    }
}

#[test]
fn css_leaf_grid_item_snap_survives_an_animated_advance() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    use crate::source::layout::layout_enums::LayoutStyleInterpolation;
    const JUSTIFY_ITEMS: u16 = 61015;
    for borrowed in [false, true] {
        for in_flight in [false, true] {
            let f = CssTextLeaf::new(
                "Hello",
                &[
                    (DISPLAY, 2),
                    (JUSTIFY_ITEMS, 10),
                    (WIDTH_KIND, 1),
                    (HEIGHT_KIND, 1),
                ],
                &[(WIDTH, 300.0), (HEIGHT, 100.0)],
            );
            let line = f.measure(None).x;
            let outer = f.solve();
            enable_leaf_layout_interpolation(&f.label);
            f.label
                .with_downcast_mut::<LayoutComponent, _>(|layout| {
                    layout.set_inherited_interpolation(LayoutStyleInterpolation::Linear, None, 1.0);
                })
                .unwrap();
            if in_flight {
                CssTextLeaf::write(&f.record, &[(JUSTIFY_ITEMS, 8)], &[]);
                retarget_css_leaf(&f, borrowed);
                advance_leaf_layout(&f.label, 0.5, borrowed);
                advance_leaf_layout(&f.label, 0.0, borrowed);
                near(f.text_size().x, (300.0 + line) / 2.0);
            }
            // Snap an item-only change with either an idle animation record
            // or a clock already in flight. Later advances must not undo it.
            let alignments = if in_flight { [10, 8] } else { [8, 10] };
            for justify in alignments {
                CssTextLeaf::write(&f.record, &[(JUSTIFY_ITEMS, justify)], &[]);
                LayoutComponent::sync_style_occurrence(&f.label);
                LayoutComponent::calculate_layout_occurrence(&f.root, f32::NAN, f32::NAN);
                if borrowed {
                    f.label
                        .with_downcast_mut::<LayoutComponent, _>(|layout| {
                            layout.update_layout_bounds(false);
                        })
                        .unwrap();
                } else {
                    LayoutComponent::update_layout_bounds_occurrence(&f.label, false);
                }
                let expected_width = if justify == 8 { line } else { 300.0 };
                let expected_left = (300.0 - expected_width) / 2.0;
                near(f.text_size().x, expected_width);
                near(f.offset().x, expected_left);
                for elapsed in [0.0, 0.5, 0.0, 1.0, 0.0] {
                    advance_leaf_layout(&f.label, elapsed, borrowed);
                    near(f.text_size().x, expected_width);
                    near(f.offset().x, expected_left);
                }
                assert_eq!(
                    f.label
                        .with_downcast::<LayoutComponent, _>(|layout| {
                            layout.layout_data.solved_layout
                        })
                        .unwrap(),
                    outer
                );
            }
        }
    }
}

fn enable_leaf_layout_interpolation(owner: &CoreHandle) {
    use crate::source::layout::layout_enums::{LayoutAnimationStyle, LayoutStyleInterpolation};
    owner
        .with_downcast_mut::<LayoutComponent, _>(|layout| {
            layout.with_style_mut(|style| {
                style
                    .base
                    .set_animation_style_type_value(LayoutAnimationStyle::Custom as u8);
                style
                    .base
                    .set_interpolation_type_value(LayoutStyleInterpolation::Linear as u8);
                style.base.set_interpolation_time_value(1.0);
            });
        })
        .unwrap();
}

fn retarget_css_leaf(f: &super::layout_fit_content_tests::CssTextLeaf, borrowed: bool) {
    LayoutComponent::sync_style_occurrence(&f.label);
    LayoutComponent::sync_style_occurrence(&f.root);
    LayoutComponent::calculate_layout_occurrence(&f.root, f32::NAN, f32::NAN);
    if borrowed {
        f.label
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.update_layout_bounds(true);
            })
            .unwrap();
    }
    LayoutComponent::update_layout_bounds_occurrence(&f.root, true);
}

fn advance_leaf_layout(owner: &CoreHandle, elapsed: f32, borrowed: bool) {
    use crate::source::advance_flags::AdvanceFlags;
    let flags = AdvanceFlags::NEW_FRAME | AdvanceFlags::ANIMATE;
    if borrowed {
        owner
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.advance_component(elapsed, flags);
            })
            .unwrap();
    } else {
        LayoutComponent::advance_component_occurrence(owner, elapsed, flags);
    }
}

#[test]
fn css_leaf_text_width_follows_layout_interpolation() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    for borrowed in [false, true] {
        let f = CssTextLeaf::new("Hello", &[(WIDTH_KIND, 1)], &[(WIDTH, 100.0)]);
        f.solve();
        enable_leaf_layout_interpolation(&f.label);
        let origin = f.offset();
        CssTextLeaf::write(&f.record, &[], &[(WIDTH, 200.0)]);
        retarget_css_leaf(&f, borrowed);
        near(f.text_size().x, 100.0);
        // Interpolation evaluates the current clock before advancing it.
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.text_size().x, 150.0);
        assert_eq!(f.offset(), origin);
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.text_size().x, 200.0);
        assert_eq!(f.offset(), origin);
    }
}

#[test]
fn css_leaf_centered_text_position_follows_layout_interpolation() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    for borrowed in [false, true] {
        let f = CssTextLeaf::new(
            "Hello",
            &[(DISPLAY, 1), (61013, 8), (WIDTH_KIND, 1), (HEIGHT_KIND, 1)],
            &[(WIDTH, 300.0), (HEIGHT, 100.0)],
        );
        let text_height = f.measure(None).y;
        f.solve();
        enable_leaf_layout_interpolation(&f.label);
        let old_top = (100.0 - text_height) / 2.0;
        let new_top = (200.0 - text_height) / 2.0;
        near(f.offset().y, old_top);
        CssTextLeaf::write(&f.record, &[], &[(HEIGHT, 200.0)]);
        retarget_css_leaf(&f, borrowed);
        near(f.offset().y, old_top);
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.first_line_position().y, (old_top + new_top) / 2.0);
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.first_line_position().y, new_top);
    }
}

#[test]
fn css_leaf_text_alignment_interpolates_without_a_box_resize() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    for borrowed in [false, true] {
        let f = CssTextLeaf::new(
            "Hello",
            &[(DISPLAY, 1), (61013, 4), (WIDTH_KIND, 1), (HEIGHT_KIND, 1)],
            &[(WIDTH, 300.0), (HEIGHT, 100.0)],
        );
        let text_height = f.measure(None).y;
        f.solve();
        enable_leaf_layout_interpolation(&f.label);
        CssTextLeaf::write(&f.record, &[(61013, 8)], &[]);
        retarget_css_leaf(&f, borrowed);
        near(f.offset().y, 0.0);
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.offset().y, (100.0 - text_height) / 4.0);
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.offset().y, (100.0 - text_height) / 2.0);
    }
}

#[test]
fn css_leaf_text_layout_retarget_keeps_smoothing_continuous() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    for borrowed in [false, true] {
        let f = CssTextLeaf::new(
            "Hello",
            &[(61017, 5), (WIDTH_KIND, 1), (HEIGHT_KIND, 1)],
            &[(WIDTH, 100.0), (HEIGHT, 100.0)],
        );
        let text_height = f.measure(None).y;
        f.solve();
        enable_leaf_layout_interpolation(&f.label);
        CssTextLeaf::write(&f.record, &[], &[(WIDTH, 200.0), (HEIGHT, 200.0)]);
        retarget_css_leaf(&f, borrowed);
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        for target in [300.0, 400.0] {
            let size = f.text_size();
            let offset = f.offset();
            CssTextLeaf::write(&f.record, &[], &[(WIDTH, target), (HEIGHT, target)]);
            retarget_css_leaf(&f, borrowed);
            assert_eq!(f.text_size(), size);
            assert_eq!(f.offset(), offset);
            // The second retarget also exercises copying the smoothing data.
            advance_leaf_layout(&f.label, 0.0, borrowed);
            near(f.text_size().x, size.x);
            near(f.offset().y, offset.y);
            advance_leaf_layout(&f.label, 0.25, borrowed);
            advance_leaf_layout(&f.label, 0.0, borrowed);
            let bounds = f
                .label
                .with_downcast::<LayoutComponent, _>(|layout| layout.layout_bounds())
                .unwrap();
            near(f.text_size().x, bounds.width());
            near(f.offset().y, (bounds.height() - text_height) / 2.0);
        }
        advance_leaf_layout(&f.label, 1.0, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.text_size().x, 400.0);
        near(f.offset().y, (400.0 - text_height) / 2.0);
    }
}

#[test]
fn translated_leaf_text_keeps_layout_interpolation() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    for borrowed in [false, true] {
        let f = CssTextLeaf::new("Hello", &[], &[]);
        f.label
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout
                    .layout_data
                    .appliers
                    .as_mut()
                    .unwrap()
                    .retain(|applier| applier != &f.record);
                layout.with_style_mut(|style| {
                    style.base.set_intrinsically_sized_value_value(false);
                    style
                        .base
                        .set_layout_width_scale_type_value(LayoutScaleType::Fixed as u8);
                    style
                        .base
                        .set_layout_height_scale_type_value(LayoutScaleType::Fixed as u8);
                    style.base.set_width_units_value_value(YGUnit::Point as u8);
                    style.base.set_height_units_value_value(YGUnit::Point as u8);
                });
                layout.base.set_width_value(100.0);
                layout.base.set_height_value(100.0);
            })
            .unwrap();
        f.solve();
        enable_leaf_layout_interpolation(&f.label);
        f.label
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.base.set_width_value(200.0);
                layout.base.set_height_value(200.0);
            })
            .unwrap();
        retarget_css_leaf(&f, borrowed);
        near(f.text_size().x, 100.0);
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.text_size().x, 150.0);
        near(f.text_size().y, 150.0);
        near(f.offset().x, 0.0);
        near(f.offset().y, 0.0);
        advance_leaf_layout(&f.label, 0.5, borrowed);
        advance_leaf_layout(&f.label, 0.0, borrowed);
        near(f.text_size().x, 200.0);
        near(f.text_size().y, 200.0);
    }
}

/// Inline content in a block ignores justify-items, while a grid's
/// anonymous item follows it and a flex item's width stays intrinsic.
#[test]
fn css_leaf_text_justify_items_applies_only_to_grid() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    use crate::source::text::text::Text;
    const JUSTIFY_ITEMS: u16 = 61015;
    for (display, justify) in [(0, 8), (0, 3), (4, 8), (1, 8), (2, 8), (0, 0)] {
        let f = CssTextLeaf::new(
            "Hello",
            &[
                (DISPLAY, display),
                (JUSTIFY_ITEMS, justify),
                (WIDTH_KIND, 1),
            ],
            &[(WIDTH, 300.0)],
        );
        let line = f.measure(None).x;
        assert!(line > 0.0 && line < 300.0);
        let block = matches!(display, 0 | 4);
        for centered_text in [false, true] {
            f.text
                .with_downcast_mut::<Text, _>(|text| {
                    text.base
                        .set_align_value_value(if centered_text { 2 } else { 0 });
                })
                .unwrap();
            f.solve();
            near(f.text_size().x, if block { 300.0 } else { line });
            near(
                f.offset().x,
                if display == 2 {
                    (300.0 - line) / 2.0
                } else {
                    0.0
                },
            );
            near(
                f.first_line_position().x,
                if display == 2 || (block && centered_text) {
                    (300.0 - line) / 2.0
                } else {
                    0.0
                },
            );
        }
    }
}

#[test]
fn translated_leaf_text_keeps_its_authored_position_and_box_size() {
    use super::layout_fit_content_tests::{CssTextLeaf, near};
    use crate::source::layout::layout_measure_mode::LayoutMeasureMode;
    use crate::source::text::text::Text;
    let f = CssTextLeaf::new(
        "Short label",
        &[(WIDTH_KIND, 1), (HEIGHT_KIND, 1)],
        &[(WIDTH, 338.0), (HEIGHT, 120.0)],
    );
    // Remove only the CSS applier to exercise the translated leaf path.
    f.label
        .with_downcast_mut::<LayoutComponent, _>(|layout| {
            layout
                .layout_data
                .appliers
                .as_mut()
                .unwrap()
                .retain(|applier| applier != &f.record);
            layout.with_style_mut(|style| {
                style.base.set_intrinsically_sized_value_value(false);
                style
                    .base
                    .set_layout_width_scale_type_value(LayoutScaleType::Fixed as u8);
                style
                    .base
                    .set_layout_height_scale_type_value(LayoutScaleType::Fixed as u8);
                style.base.set_width_units_value_value(YGUnit::Point as u8);
                style.base.set_height_units_value_value(YGUnit::Point as u8);
            });
            layout.base.set_width_value(338.0);
            layout.base.set_height_value(120.0);
        })
        .unwrap();
    f.text
        .with_downcast_mut::<Text, _>(|text| {
            text.base.set_align_value_value(2);
        })
        .unwrap();
    f.solve();
    near(f.offset().x, 0.0);
    near(f.offset().y, 0.0);
    near(f.text_size().x, 338.0);
    let measured = f
        .text
        .with_downcast_mut::<Text, _>(|text| {
            text.measure_layout(
                338.0,
                LayoutMeasureMode::Exactly,
                f32::NAN,
                LayoutMeasureMode::Undefined,
            )
        })
        .unwrap();
    assert!(measured.y > 0.0);
}

/// Padding and border written at run time move the content with them.
#[test]
fn css_leaf_content_follows_padding_written_at_run_time() {
    let f = padded_leaf(&[], None);
    assert!(CoreRegistry::set_double_handle(
        &f.record("Pad"),
        PADDING_LEFT.into(),
        20.0
    ));
    assert!(CoreRegistry::set_double_handle(
        &f.record("Pad"),
        BORDER_TOP.into(),
        5.0
    ));
    f.advance(2);
    for name in ["Dot", "Grouped", "Picture"] {
        assert_eq!(f.world_origin(name), (22.0, 11.0), "{name}");
    }
}

/// Bound padding moves the content as its view model value changes.
#[test]
fn css_leaf_content_follows_bound_padding() {
    use crate::source::generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase;
    let f = padded_leaf(&[], Some(PADDING_LEFT));
    let number = f.bind_number();
    assert!(CoreRegistry::set_double_handle(
        &number,
        ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
        25.0
    ));
    f.advance(3);
    assert_eq!(
        CoreRegistry::get_double_handle(&f.record("Pad"), PADDING_LEFT.into()),
        Some(25.0)
    );
    assert_eq!(f.world_origin("Dot"), (27.0, 8.0));
}

/// Right to left, the content box is the same physical rectangle: the
/// content starts past the left border and padding.
#[test]
fn css_leaf_content_box_is_physical_right_to_left() {
    const RTL: u64 = 2;
    let f = padded_leaf(&[U(DIRECTION, RTL)], None);
    assert_eq!(f.world_origin("Dot"), (12.0, 8.0));
}

/// A translated box leaves its content where it was authored, as Rive does.
#[test]
fn translated_box_leaves_its_content_where_authored() {
    let f = scene(
        (400.0, 300.0),
        B {
            name: "Pad",
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 50.0),
            ],
            style: vec![
                F(LayoutComponentStyleBase::PADDING_LEFT_PROPERTY_KEY, 10.0),
                F(LayoutComponentStyleBase::PADDING_TOP_PROPERTY_KEY, 6.0),
            ],
            content: vec![Rect {
                name: "Dot",
                size: (40.0, 20.0),
                grouped: false,
                path_scale_x: 1.0,
            }],
            ..B::default()
        },
    );
    assert_eq!(f.world_origin("Dot"), (0.0, 0.0));
}

/// An absolute box placed against an ancestor above its static parent is
/// republished when that containing block resizes, and when a static
/// ancestor between them moves, though its parent's layout holds.
#[test]
fn css_hoisted_box_follows_its_containing_block_and_static_ancestors() {
    const MARGIN_LEFT: u16 = 61049;
    const INSET_RIGHT_KIND: u16 = 61054;
    const INSET_RIGHT: u16 = 61055;
    let f = scene(
        (500.0, 300.0),
        css(
            "Frame",
            [[U(POSITION, 1)].as_slice(), &width(300.0), &height(200.0)].concat(),
            vec![css(
                "Outer",
                [
                    [F(MARGIN_LEFT, 50.0)].as_slice(),
                    &width(200.0),
                    &height(60.0),
                ]
                .concat(),
                vec![css(
                    "Static",
                    [width(100.0), height(40.0)].concat(),
                    vec![css(
                        "Pinned",
                        [
                            [
                                U(POSITION, 2),
                                U(INSET_RIGHT_KIND, 1),
                                F(INSET_RIGHT, 10.0),
                                U(INSET_TOP_KIND, 1),
                                F(INSET_TOP, 0.0),
                            ]
                            .as_slice(),
                            &width(30.0),
                            &height(10.0),
                        ]
                        .concat(),
                        vec![],
                    )],
                )],
            )],
        ),
    );
    // 300 - 10 - 30 = 260 in the frame, 210 in the static box at x 50.
    assert_eq!(f.bounds("Pinned"), [210.0, 0.0, 30.0, 10.0]);
    assert!(CoreRegistry::set_double_handle(
        &f.record("Frame"),
        WIDTH.into(),
        400.0
    ));
    f.advance(2);
    assert_eq!(f.bounds("Pinned"), [310.0, 0.0, 30.0, 10.0]);
    assert!(CoreRegistry::set_double_handle(
        &f.record("Outer"),
        MARGIN_LEFT.into(),
        80.0
    ));
    f.advance(2);
    assert_eq!(f.bounds("Static")[0], 0.0);
    assert_eq!(f.bounds("Pinned"), [280.0, 0.0, 30.0, 10.0]);
}

/// A record on the artboard clips the artboard's content at its padding
/// box when its overflow is not visible.
#[test]
fn css_artboard_overflow_clips_its_content() {
    let clip_points = |overflow: u64| {
        let f = try_scene(
            (400.0, 300.0),
            Some(vec![
                U(OVERFLOW_X, overflow),
                U(OVERFLOW_Y, overflow),
                F(BORDER_TOP, 10.0),
                F(BORDER_RIGHT, 10.0),
                F(BORDER_BOTTOM, 10.0),
                F(BORDER_LEFT, 10.0),
            ]),
            B {
                paints: vec![Paint::Fill(0xff77_8899)],
                ..css("Child", [width(500.0), height(500.0)].concat(), vec![])
            },
        )
        .expect("the scene imports");
        f.draw_lines()
            .into_iter()
            .filter(|line| line.starts_with("clipPath"))
            .collect::<Vec<_>>()
    };
    let visible = clip_points(0);
    let hidden = clip_points(2);
    assert_eq!(hidden.len(), visible.len() + 1, "{hidden:?}");
    assert!(
        hidden
            .last()
            .unwrap()
            .contains("(10,10),(390,10),(390,290),(10,290)"),
        "{hidden:?}"
    );
}

/// A CSS box inherits its parent's direction through its record, whatever
/// its Rive style's direction says.
#[test]
fn css_direction_inherits_past_a_legacy_style_direction() {
    const RTL: u64 = 2;
    const LTR: u64 = 1;
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [
                [U(DISPLAY, 1), U(DIRECTION, RTL)].as_slice(),
                &width(200.0),
                &height(100.0),
            ]
            .concat(),
            vec![B {
                style: vec![U(
                    LayoutComponentStyleBase::DIRECTION_VALUE_PROPERTY_KEY,
                    LTR,
                )],
                ..css(
                    "Middle",
                    [[U(DISPLAY, 1)].as_slice(), &width(200.0), &height(20.0)].concat(),
                    vec![css("Dot", [width(20.0), height(20.0)].concat(), vec![])],
                )
            }],
        ),
    );
    assert_eq!(f.bounds("Dot")[0], 180.0);
}

/// A shadow Feather made inner at run time redraws as an inset shadow,
/// inside the padding box.
#[test]
fn css_shadow_feather_made_inner_at_run_time_draws_inset() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Shadow {
                color: 0x8000_0000,
                strength: 10.0,
                spread: 0.0,
            }],
            ..css("Lifted", border_box(&[]), vec![])
        },
    );
    assert_eq!(
        f.css_paints("Lifted")[0].1,
        Some(aabb(-17.0, -17.0, 117.0, 67.0))
    );
    assert!(CoreRegistry::set_bool_handle(
        &f.paint_child("Lifted", FeatherBase::TYPE_KEY),
        FeatherBase::INNER_PROPERTY_KEY.into(),
        true
    ));
    f.advance(2);
    // Inset: clipped to the padding box inside the 4-point border.
    assert_eq!(
        f.css_paints("Lifted")[0].1,
        Some(aabb(4.0, 4.0, 96.0, 46.0))
    );
}

/// CSS content keeps its natural size past its box: an image overflowing
/// its fixed content box adds to its scroll viewport's range.
#[test]
fn css_content_overflow_reaches_the_scroll_range() {
    const ALL: u64 = 2;
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [
                [U(OVERFLOW_X, 3), U(OVERFLOW_Y, 3)].as_slice(),
                &width(50.0),
                &height(20.0),
            ]
            .concat(),
            vec![B {
                scroll: Some(ALL),
                images: vec!["Picture"],
                ..css("Content", [width(50.0), height(20.0)].concat(), vec![])
            }],
        ),
    );
    // The 100 by 30 image overflows the 50 by 20 box by 50 and 10.
    assert_eq!(f.max_offsets(), (-50.0, -10.0));
}

/// An artboard record's BoxBorder draws its CSS ring, from the record's
/// border widths, not the Stroke's own thickness.
#[test]
fn css_artboard_paints_with_css_geometry() {
    let f = try_scene_painted(
        (400.0, 300.0),
        Some(vec![
            F(BORDER_TOP, 10.0),
            F(BORDER_RIGHT, 10.0),
            F(BORDER_BOTTOM, 10.0),
            F(BORDER_LEFT, 10.0),
        ]),
        &[Paint::Border(0xff44_5566, 15)],
        css("Child", [width(50.0), height(50.0)].concat(), vec![]),
    )
    .expect("the scene imports");
    let lines = f.draw_lines();
    let border = lines
        .iter()
        .find(|line| line.starts_with("drawPath") && line.contains("color=0xff445566"))
        .unwrap_or_else(|| panic!("{}", lines.join("\n")));
    // A filled ring: the border box, and the padding box as its hole.
    assert!(border.contains("style=fill"), "{border}");
    assert!(border.contains("(10,10)"), "{border}");
}

/// A CSS artboard's overflow clips its children's semantic bounds too.
#[test]
fn css_artboard_overflow_clips_semantic_bounds() {
    const MARGIN_LEFT: u16 = 61049;
    let f = try_scene(
        (400.0, 300.0),
        Some(vec![
            U(OVERFLOW_X, 2),
            U(OVERFLOW_Y, 2),
            F(BORDER_TOP, 10.0),
            F(BORDER_LEFT, 10.0),
        ]),
        css(
            "Corner",
            [
                [F(MARGIN_TOP, -10.0), F(MARGIN_LEFT, -10.0)].as_slice(),
                &width(5.0),
                &height(5.0),
            ]
            .concat(),
            vec![],
        ),
    )
    .expect("the scene imports");
    assert_eq!(f.bounds("Corner")[..2], [0.0, 0.0]);
    use crate::source::semantic::semantic_provider::rendered_geometry_bounds;
    assert!(rendered_geometry_bounds(Some(&f.named("Corner"))).is_empty_or_nan());
}

/// A BoxBorder draws its ring whatever its Stroke's thickness, which the
/// contract ignores: a zero-thickness Stroke still draws the border.
#[test]
fn css_box_border_ignores_its_stroke_thickness() {
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Border(0xff44_5566, 15)],
            ..css("Edge", border_box(&[]), vec![])
        },
    );
    let stroke = f
        .named("Edge")
        .with_downcast::<LayoutComponent, _>(|layout| layout.paints.shape_paints()[0].clone())
        .unwrap();
    assert!(CoreRegistry::set_double_handle(
        &stroke,
        StrokeBase::THICKNESS_PROPERTY_KEY.into(),
        0.0
    ));
    f.advance(2);
    let lines = f.draw_lines();
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("drawPath") && line.contains("color=0xff445566")),
        "{}",
        lines.join("\n")
    );
}

/// A zero percentage is not a zero length: against an indefinite height,
/// `flex-basis: 0%` (what `flex: 1` writes) is the content's size and
/// `max-height: 0%` is none, where 0px would collapse the box.
#[test]
fn css_zero_percentages_resolve_against_an_indefinite_size() {
    const COLUMN: u64 = 2;
    const FLEX_BASIS_KIND: u16 = 61010;
    const MIN_HEIGHT_KIND: u16 = 61030;
    const MAX_HEIGHT_KIND: u16 = 61036;
    const PERCENTAGE: u64 = 7;
    const LIMIT_PERCENTAGE: u64 = 2;
    let leaf = |name, props: Vec<P>| B {
        images: vec!["Picture"],
        ..css(name, props, vec![])
    };
    let f = scene(
        (400.0, 300.0),
        css(
            "Column",
            [[U(DISPLAY, 1), U(61006, COLUMN)].as_slice(), &width(200.0)].concat(),
            vec![
                leaf(
                    "Basis",
                    vec![
                        F(FLEX_GROW, 1.0),
                        U(FLEX_BASIS_KIND, PERCENTAGE),
                        U(MIN_HEIGHT_KIND, 1),
                    ],
                ),
                leaf("Capped", vec![U(MAX_HEIGHT_KIND, LIMIT_PERCENTAGE)]),
            ],
        ),
    );
    assert_eq!(f.bounds("Basis")[3], 30.0);
    assert_eq!(f.bounds("Capped")[3], 30.0);
}

/// `flex: 2` beside `height: 80px` in a column: its `0%` basis against an
/// indefinite height is its content's size, here none, so the height does
/// not stand in for it. Chrome 156 gives 0 in an auto-height column and
/// grows it to 140 in a 200-point one.
#[test]
fn css_flex_basis_percentage_ignores_the_height_against_an_indefinite_column() {
    const COLUMN: u64 = 2;
    const FLEX_BASIS_KIND: u16 = 61010;
    const PERCENTAGE: u64 = 7;
    let column = |height_props: Vec<P>, painted: bool| {
        scene(
            (400.0, 400.0),
            css(
                "Column",
                [
                    [U(DISPLAY, 1), U(61006, COLUMN)].as_slice(),
                    &width(200.0),
                    &height_props,
                ]
                .concat(),
                vec![
                    css(
                        "Before",
                        [[F(FLEX_SHRINK, 0.0)].as_slice(), &height(30.0)].concat(),
                        vec![],
                    ),
                    B {
                        paints: if painted {
                            vec![Paint::Fill(0xffcc_3333)]
                        } else {
                            vec![]
                        },
                        ..css(
                            "Grow",
                            [
                                [F(FLEX_GROW, 2.0), U(FLEX_BASIS_KIND, PERCENTAGE)].as_slice(),
                                &height(80.0),
                            ]
                            .concat(),
                            vec![],
                        )
                    },
                    css(
                        "After",
                        [[F(FLEX_SHRINK, 0.0)].as_slice(), &height(30.0)].concat(),
                        vec![],
                    ),
                ],
            ),
        )
    };
    // A background is no content: painted or not, the item is the same.
    for painted in [false, true] {
        let auto = column(vec![], painted);
        assert_eq!(
            auto.bounds("Grow"),
            [0.0, 30.0, 200.0, 0.0],
            "painted {painted}"
        );
        assert_eq!(auto.bounds("After")[1], 30.0, "painted {painted}");
        assert_eq!(auto.bounds("Column")[3], 60.0, "painted {painted}");
        let definite = column(height(200.0).to_vec(), painted);
        assert_eq!(
            definite.bounds("Grow"),
            [0.0, 30.0, 200.0, 140.0],
            "painted {painted}"
        );
    }
}

/// A block parent measures its column with max-content available height.
/// The preferred item height must not replace an unresolved percentage basis.
#[test]
fn css_flex_basis_percentage_in_a_column_under_a_block() {
    for direction in [2, 3] {
        for column_width in [false, true] {
            let mut props = vec![U(DISPLAY, 1), U(61006, direction)];
            if column_width {
                props.extend(width(200.0));
            }
            let f = scene(
                (360.0, 640.0),
                css(
                    "Block",
                    vec![],
                    vec![css(
                        "Column",
                        props,
                        vec![css(
                            "Grow",
                            [&[F(FLEX_GROW, 2.0), U(61010, 7)][..], &height(80.0)].concat(),
                            vec![],
                        )],
                    )],
                ),
            );
            assert_eq!(
                f.bounds("Grow")[3],
                0.0,
                "direction {direction}, width {column_width}"
            );
        }
    }
}

/// Chrome 156: intrinsic single-line columns use each item's hypothetical
/// main size, including the automatic minimum, whatever its flex basis.
#[test]
fn css_intrinsic_column_uses_hypothetical_item_heights() {
    const FLEX_BASIS_KIND: u16 = 61010;
    const FLEX_BASIS: u16 = 61011;
    const MIN_HEIGHT_KIND: u16 = 61030;
    const MIN_HEIGHT: u16 = 61031;
    const MAX_HEIGHT_KIND: u16 = 61036;
    const MAX_HEIGHT: u16 = 61037;
    let flex = vec![F(FLEX_GROW, 2.0), U(FLEX_BASIS_KIND, 7)];
    let zero_basis = vec![U(FLEX_BASIS_KIND, 1)];
    let thirty_basis = vec![
        F(FLEX_GROW, 1.0),
        U(FLEX_BASIS_KIND, 1),
        F(FLEX_BASIS, 30.0),
    ];
    let cases = [
        ("auto basis, taller content", vec![], 120.0, 80.0),
        ("auto basis, shorter content", vec![], 50.0, 80.0),
        (
            "percentage basis, taller content",
            flex.clone(),
            120.0,
            120.0,
        ),
        (
            "percentage basis, shorter content",
            flex.clone(),
            50.0,
            50.0,
        ),
        ("percentage basis, empty content", flex.clone(), 0.0, 0.0),
        (
            "zero basis, taller content",
            zero_basis.clone(),
            120.0,
            80.0,
        ),
        ("zero basis, shorter content", zero_basis, 50.0, 50.0),
        (
            "percentage basis, zero minimum",
            [flex.clone(), vec![U(MIN_HEIGHT_KIND, 1)]].concat(),
            120.0,
            120.0,
        ),
        (
            "100 point basis, empty content",
            vec![U(FLEX_BASIS_KIND, 1), F(FLEX_BASIS, 100.0)],
            0.0,
            100.0,
        ),
        (
            "100 point basis, taller content",
            vec![U(FLEX_BASIS_KIND, 1), F(FLEX_BASIS, 100.0)],
            120.0,
            100.0,
        ),
        (
            "30 point basis, empty content",
            thirty_basis.clone(),
            0.0,
            30.0,
        ),
        ("30 point basis, taller content", thirty_basis, 120.0, 80.0),
        (
            "auto basis, maximum",
            vec![U(MAX_HEIGHT_KIND, 1), F(MAX_HEIGHT, 60.0)],
            120.0,
            60.0,
        ),
        (
            "auto basis, minimum",
            vec![U(MIN_HEIGHT_KIND, 1), F(MIN_HEIGHT, 100.0)],
            0.0,
            100.0,
        ),
        (
            "percentage basis, hidden overflow",
            [flex, vec![U(OVERFLOW_X, 2), U(OVERFLOW_Y, 2)]].concat(),
            120.0,
            120.0,
        ),
    ];
    let mut failures = Vec::new();
    for direction in [2, 3] {
        // Auto height under a block parent takes the max-content path. The
        // keywords also exercise explicit min-content and max-content requests.
        for column_height_kind in [0, 2, 3] {
            for (name, item_props, content_height, expected) in &cases {
                let f = scene(
                    (360.0, 640.0),
                    css(
                        "Block",
                        vec![],
                        vec![css(
                            "Column",
                            [
                                &[
                                    U(DISPLAY, 1),
                                    U(61006, direction),
                                    U(HEIGHT_KIND, column_height_kind),
                                ][..],
                                &width(200.0),
                            ]
                            .concat(),
                            vec![css(
                                "Item",
                                [height(80.0).as_slice(), item_props].concat(),
                                vec![css("Content", height(*content_height).to_vec(), vec![])],
                            )],
                        )],
                    ),
                );
                let actual = [f.bounds("Item")[3], f.bounds("Column")[3]];
                if actual != [*expected; 2] {
                    failures.push(format!("{name}, direction {direction}, height kind {column_height_kind}: {actual:?}, expected {expected}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Outer hypothetical sizes include margins; the container adds gaps and
/// its own padding and border once, without counting overflowing content.
#[test]
fn css_intrinsic_column_sums_outer_hypothetical_sizes_and_gaps() {
    for direction in [2, 3] {
        for column_height_kind in [0, 2, 3] {
            let f = scene(
                (360.0, 640.0),
                css(
                    "Block",
                    vec![],
                    vec![css(
                        "Column",
                        [
                            &[
                                U(DISPLAY, 1),
                                U(61006, direction),
                                U(HEIGHT_KIND, column_height_kind),
                                F(61075, 9.0),
                                F(PADDING_TOP, 4.0),
                                F(PADDING_BOTTOM, 6.0),
                                F(BORDER_TOP, 2.0),
                                F(BORDER_BOTTOM, 3.0),
                            ][..],
                            &width(200.0),
                        ]
                        .concat(),
                        vec![
                            css(
                                "Fixed",
                                [
                                    height(80.0).as_slice(),
                                    &[F(MARGIN_TOP, 7.0), F(MARGIN_BOTTOM, 11.0)],
                                ]
                                .concat(),
                                vec![css("Overflowing content", height(120.0).to_vec(), vec![])],
                            ),
                            css(
                                "Flexible",
                                vec![
                                    F(FLEX_GROW, 2.0),
                                    U(61010, 7),
                                    F(MARGIN_TOP, -3.0),
                                    F(MARGIN_BOTTOM, 5.0),
                                ],
                                vec![css("Content", height(50.0).to_vec(), vec![])],
                            ),
                        ],
                    )],
                ),
            );
            assert_eq!(
                [
                    f.bounds("Fixed")[3],
                    f.bounds("Flexible")[3],
                    f.bounds("Column")[3]
                ],
                [80.0, 50.0, 174.0],
                "direction {direction}, height kind {column_height_kind}",
            );
        }
    }
}

/// A plain Taffy tree isolates intrinsic main sizing from runtime measurement.
#[test]
fn css_flex_basis_taffy_intrinsic_columns_use_content() {
    use taffy::prelude::*;
    for direction in [FlexDirection::Column, FlexDirection::ColumnReverse] {
        for constraint in [AvailableSpace::MinContent, AvailableSpace::MaxContent] {
            for basis in [
                Dimension::percent(0.0),
                Dimension::length(0.0),
                Dimension::content(),
            ] {
                for (content_height, min_height, max_height, expected) in [
                    (0.0, None, None, 0.0),
                    (50.0, None, None, 50.0),
                    (0.0, None, Some(40.0), 0.0),
                    (0.0, Some(20.0), None, 20.0),
                ] {
                    let mut tree = TaffyTree::<f32>::new();
                    tree.disable_rounding();
                    let item = tree
                        .new_leaf_with_context(
                            Style {
                                size: Size {
                                    width: length(110.0),
                                    height: length(80.0),
                                },
                                min_size: Size {
                                    width: auto(),
                                    height: min_height.map(length).unwrap_or(auto()),
                                },
                                max_size: Size {
                                    width: auto(),
                                    height: max_height.map(length).unwrap_or(auto()),
                                },
                                flex_basis: basis,
                                flex_grow: 2.0,
                                ..Style::default()
                            },
                            content_height,
                        )
                        .unwrap();
                    let column = tree
                        .new_with_children(
                            Style {
                                flex_direction: direction,
                                size: Size {
                                    width: length(200.0),
                                    height: auto(),
                                },
                                ..Style::default()
                            },
                            &[item],
                        )
                        .unwrap();
                    tree.compute_layout_with_measure(
                        column,
                        Size {
                            width: AvailableSpace::Definite(200.0),
                            height: constraint,
                        },
                        |inputs, _, content, style| {
                            taffy::compute_leaf_layout(
                                inputs,
                                style,
                                |_, _| 0.0,
                                |known, _| {
                                    known.unwrap_or(Size {
                                        width: 0.0,
                                        height: content.copied().unwrap_or(0.0),
                                    })
                                },
                            )
                        },
                    )
                    .unwrap();
                    assert_eq!(
                        tree.layout(item).unwrap().size.height,
                        expected,
                        "{direction:?}, {constraint:?}, {basis:?}, content {content_height}, min {min_height:?}, max {max_height:?}"
                    );
                    assert_eq!(tree.layout(column).unwrap().size.height, expected);
                }
            }
        }
    }
}

/// The compiler writes a block viewport and a full-size block content box.
/// Chrome 156 gives the reverse column 144 points, including the empty
/// flexible item's 44 points of vertical margins.
#[test]
fn css_flex_basis_percentage_in_a_reverse_column_under_a_block_scroller() {
    let f = try_scene(
        (360.0, 640.0),
        Some(vec![]),
        overflow_viewport(
            "a",
            [height(160.0).as_slice(), &[U(OVERFLOW_Y, 4)]].concat(),
            vec![
                css(
                    "b",
                    [
                        &[
                            U(DISPLAY, 1),
                            U(61006, 3),
                            U(61018, 2),
                            U(61013, 2),
                            F(PADDING_TOP, 14.0),
                            F(PADDING_RIGHT, 8.0),
                            F(PADDING_BOTTOM, 18.0),
                            F(PADDING_LEFT, 8.0),
                            F(BORDER_TOP, 1.0),
                            F(BORDER_LEFT, 8.0),
                        ][..],
                        &width(250.0),
                        &height(380.0),
                    ]
                    .concat(),
                    vec![
                        css(
                            "c",
                            [
                                &[
                                    U(BOX_SIZING, 1),
                                    F(PADDING_TOP, 14.0),
                                    F(PADDING_RIGHT, 14.0),
                                    F(PADDING_BOTTOM, 14.0),
                                    F(PADDING_LEFT, 14.0),
                                    F(BORDER_BOTTOM, 6.0),
                                ][..],
                                &height(80.0),
                                &width(170.0),
                            ]
                            .concat(),
                            vec![],
                        ),
                        css(
                            "d",
                            [
                                &[
                                    U(WIDTH_KIND, 7),
                                    F(WIDTH_PERCENT, 90.0),
                                    F(MARGIN_TOP + 1, 3.0),
                                    F(61043 + 1, 3.0),
                                    F(MARGIN_BOTTOM + 1, 3.0),
                                    F(MARGIN_LEFT + 1, 3.0),
                                ][..],
                                &height(10.0),
                            ]
                            .concat(),
                            vec![],
                        ),
                    ],
                ),
                css(
                    "e",
                    vec![
                        U(DISPLAY, 1),
                        U(61006, 3),
                        F(MARGIN_TOP, 18.0),
                        F(61043, -4.0),
                        F(MARGIN_BOTTOM, 18.0),
                        F(MARGIN_LEFT, -4.0),
                    ],
                    vec![
                        css(
                            "f",
                            [height(10.0).as_slice(), &width(80.0), &padding(10.0)].concat(),
                            vec![],
                        ),
                        css(
                            "g",
                            [
                                &[
                                    U(BOX_SIZING, 1),
                                    F(BORDER_TOP, 1.0),
                                    F(BORDER_RIGHT, 1.0),
                                    F(BORDER_BOTTOM, 1.0),
                                    F(BORDER_LEFT, 1.0),
                                ][..],
                                &height(70.0),
                                &width(180.0),
                            ]
                            .concat(),
                            vec![],
                        ),
                        B {
                            paints: vec![Paint::Fill(0xffcc_3333)],
                            ..css(
                                "h",
                                [
                                    &[
                                        F(FLEX_GROW, 2.0),
                                        U(61010, 7),
                                        F(MARGIN_TOP, 22.0),
                                        F(61043, 6.0),
                                        F(MARGIN_BOTTOM, 22.0),
                                        F(MARGIN_LEFT, 6.0),
                                    ][..],
                                    &height(80.0),
                                    &width(110.0),
                                ]
                                .concat(),
                                vec![],
                            )
                        },
                    ],
                ),
            ],
        ),
    )
    .expect("the scene imports");
    let e = f.bounds("e");
    let world_bounds = |name| {
        let [x, y, width, height] = f.bounds(name);
        [e[0] + x, e[1] + y, width, height]
    };
    assert_eq!(
        [world_bounds("h"), world_bounds("f"), world_bounds("g"), e],
        [
            [2.0, 453.0, 110.0, 0.0],
            [-4.0, 545.0, 100.0, 30.0],
            [-4.0, 475.0, 180.0, 70.0],
            [-4.0, 431.0, 368.0, 144.0],
        ],
        "scroll offsets {:?}",
        f.max_offsets(),
    );
    assert_chrome_range(&f, (4.0, 433.0));
}

/// A CSS artboard without layout children places its content at its
/// content box's origin, as a CSS leaf box does.
#[test]
fn css_leaf_artboard_places_its_content() {
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![
                F(BORDER_TOP, 2.0),
                F(BORDER_LEFT, 2.0),
                F(PADDING_TOP, 10.0),
                F(PADDING_LEFT, 10.0),
            ]),
            content: vec![Rect {
                name: "Dot",
                size: (40.0, 20.0),
                grouped: false,
                path_scale_x: 1.0,
            }],
            ..ArtboardSpec::default()
        },
        None,
    )
    .expect("the scene imports");
    assert_eq!(f.world_origin("Dot"), (12.0, 12.0));
}

/// A CSS artboard's background follows its CSS radii, as a box's does.
#[test]
fn css_artboard_background_is_rounded() {
    const RADIUS_PERCENT: [u16; 4] = [61079, 61080, 61081, 61082];
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(RADIUS_PERCENT.map(|key| F(key, 50.0)).to_vec()),
            paints: vec![Paint::Fill(0xff11_2233)],
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let lines = f.draw_lines();
    let background = lines
        .iter()
        .find(|line| line.starts_with("drawPath") && line.contains("color=0xff112233"))
        .unwrap_or_else(|| panic!("{}", lines.join("\n")));
    assert!(background.contains("cubic"), "{background}");
}

/// A CSS artboard's BoxGradient covers its background where it is drawn,
/// past the artboard's origin.
#[test]
fn css_artboard_gradient_follows_its_origin() {
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![]),
            paints: vec![Paint::Gradient(90.0)],
            origin_x: 0.5,
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let lines = f.stream_lines();
    let gradient = lines
        .iter()
        .rev()
        .find(|line| line.starts_with("makeLinearGradient"))
        .expect("a gradient");
    // To the right across the background, which runs from -200 to 200.
    assert_eq!(
        gradient_ends(gradient),
        ((-200.0, 150.0), (200.0, 150.0)),
        "{gradient}"
    );
}

/// A BoxGradient follows its box while layout interpolation resizes it:
/// every frame of the resize places the gradient over that frame's box,
/// though only the box's world transform is dirty between its ends.
#[test]
fn css_box_gradient_follows_an_interpolated_resize() {
    use crate::source::generated::layout::layout_component_style_base::LayoutComponentStyleBase as Style;
    const CUSTOM: u64 = 2;
    const LINEAR: u64 = 1;
    let f = frame(vec![B {
        paints: vec![Paint::Gradient(90.0)],
        style: vec![
            U(Style::ANIMATION_STYLE_TYPE_PROPERTY_KEY, CUSTOM),
            U(Style::INTERPOLATION_TYPE_PROPERTY_KEY, LINEAR),
            F(Style::INTERPOLATION_TIME_PROPERTY_KEY, 1.0),
        ],
        ..css("Card", sized(&[]), vec![])
    }]);
    assert!(CoreRegistry::set_double_handle(
        &f.record("Card"),
        WIDTH.into(),
        200.0
    ));
    f.advance(30);
    let width_now = f.bounds("Card")[2];
    assert!(width_now > 110.0 && width_now < 190.0, "{width_now}");
    f.draw_lines();
    let lines = f.stream_lines();
    let gradient = lines
        .iter()
        .rev()
        .find(|line| line.starts_with("makeLinearGradient"))
        .expect("a gradient");
    // To the right across the box as it is now.
    let ((start_x, _), (end_x, _)) = gradient_ends(gradient);
    assert_eq!(start_x, 0.0, "{gradient}");
    assert!((end_x - width_now).abs() < 0.01, "{width_now}: {gradient}");
}

/// A recorded gradient's start and end, to a hundredth.
fn gradient_ends(gradient: &str) -> ((f32, f32), (f32, f32)) {
    let point = |label: &str| -> (f32, f32) {
        let rest = &gradient[gradient.find(label).expect(label) + label.len()..];
        let (x, rest) = rest.split_once(',').unwrap();
        let y = rest.split_once(')').unwrap().0;
        let round = |value: &str| (value.parse::<f32>().unwrap() * 100.0).round() / 100.0;
        (round(x), round(y))
    };
    (point("start=("), point("end=("))
}

/// A CSS grid's `justify-items` reaches its Rive children: a Hug child
/// with an automatic `justify-self` is centred in a centred grid.
#[test]
fn css_grid_justify_items_reaches_rive_children() {
    const POINTS: u64 = 1;
    const JUSTIFY_ITEMS: u16 = 61015;
    const CENTER: u64 = 8;
    let f = scene(
        (400.0, 300.0),
        B {
            tracks: vec![(0, POINTS, 200.0)],
            ..css(
                "Grid",
                [
                    [U(DISPLAY, 2), U(JUSTIFY_ITEMS, CENTER)].as_slice(),
                    &width(200.0),
                    &height(40.0),
                ]
                .concat(),
                vec![B {
                    name: "Hug",
                    style: vec![
                        U(
                            LayoutComponentStyleBase::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY,
                            1,
                        ),
                        U(
                            LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                            LayoutScaleType::Hug as u64,
                        ),
                        U(
                            LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
                            LayoutScaleType::Hug as u64,
                        ),
                    ],
                    content: vec![Rect {
                        name: "Dot",
                        size: (40.0, 20.0),
                        grouped: false,
                        path_scale_x: 1.0,
                    }],
                    ..B::default()
                }],
            )
        },
    );
    assert_eq!(f.bounds("Hug")[..3], [80.0, 0.0, 40.0]);
}

/// A CSS leaf artboard keeps its content at its content box when it
/// resizes or its origin moves.
#[test]
fn css_leaf_artboard_content_follows_its_size_and_origin() {
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![F(BORDER_LEFT, 2.0), F(PADDING_LEFT, 10.0)]),
            content: vec![Rect {
                name: "Dot",
                size: (40.0, 20.0),
                grouped: false,
                path_scale_x: 1.0,
            }],
            origin_x: 0.5,
            ..ArtboardSpec::default()
        },
        None,
    )
    .expect("the scene imports");
    // The background runs from -200: the content box starts 12 past it.
    assert_eq!(f.world_origin("Dot").0, -188.0);
    let artboard = f.artboard.core_handle();
    assert!(CoreRegistry::set_double_handle(
        &artboard,
        LayoutComponentBase::WIDTH_PROPERTY_KEY.into(),
        600.0
    ));
    f.advance(2);
    assert_eq!(f.world_origin("Dot").0, -288.0);
    assert!(CoreRegistry::set_double_handle(
        &artboard,
        ArtboardBase::ORIGIN_X_PROPERTY_KEY.into(),
        0.0
    ));
    f.advance(2);
    assert_eq!(f.world_origin("Dot").0, 12.0);
}

/// A CSS grid's `justify-items` written at run time moves its Rive Hug
/// children.
#[test]
fn css_grid_justify_items_written_at_run_time_moves_rive_children() {
    const POINTS: u64 = 1;
    const JUSTIFY_ITEMS: u16 = 61015;
    const CENTER: u32 = 8;
    let f = scene(
        (400.0, 300.0),
        B {
            tracks: vec![(0, POINTS, 200.0)],
            ..css(
                "Grid",
                [[U(DISPLAY, 2)].as_slice(), &width(200.0), &height(40.0)].concat(),
                vec![B {
                    name: "Hug",
                    style: vec![
                        U(
                            LayoutComponentStyleBase::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY,
                            1,
                        ),
                        U(
                            LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                            LayoutScaleType::Hug as u64,
                        ),
                        U(
                            LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
                            LayoutScaleType::Hug as u64,
                        ),
                    ],
                    content: vec![Rect {
                        name: "Dot",
                        size: (40.0, 20.0),
                        grouped: false,
                        path_scale_x: 1.0,
                    }],
                    ..B::default()
                }],
            )
        },
    );
    assert_eq!(f.bounds("Hug")[0], 0.0);
    assert!(CoreRegistry::set_uint_handle(
        &f.record("Grid"),
        JUSTIFY_ITEMS.into(),
        CENTER
    ));
    f.advance(2);
    assert_eq!(f.bounds("Hug")[0], 80.0);
}

/// A CSS artboard is its own size: the Rive style's maximum does not
/// shrink it.
#[test]
fn css_artboard_ignores_legacy_limits() {
    let f = try_scene_spec(
        (200.0, 100.0),
        ArtboardSpec {
            css: Some(vec![]),
            style: vec![
                U(
                    LayoutSizingStyleBase::MAX_WIDTH_UNITS_VALUE_PROPERTY_KEY,
                    YGUnit::Point as u64,
                ),
                F(LayoutSizingStyleBase::MAX_WIDTH_PROPERTY_KEY, 80.0),
            ],
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let artboard = f
        .artboard
        .with_artboard(|artboard| artboard.layout_bounds());
    assert_eq!(artboard.width(), 200.0);
}

/// A CSS artboard whose Rive style hugs (the runtime marks a hugging style
/// intrinsically sized) takes its content's size when nothing bounds it.
/// That is how a hosted content-height presentation, such as a sheet,
/// measures itself.
#[test]
fn css_artboard_that_hugs_takes_its_content_size() {
    let f = try_scene_spec(
        (200.0, 100.0),
        ArtboardSpec {
            css: Some(vec![]),
            style: vec![
                U(
                    LayoutSizingStyleBase::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,
                    LayoutScaleType::Hug as u64,
                ),
                U(
                    LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
                    LayoutScaleType::Hug as u64,
                ),
                U(
                    LayoutComponentStyleBase::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY,
                    1,
                ),
            ],
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let bounds = f
        .artboard
        .with_artboard(|artboard| artboard.layout_bounds());
    assert_eq!((bounds.width(), bounds.height()), (10.0, 10.0));
}

/// A hosted CSS artboard keeps its stored Hug height without
/// intrinsicallySized, as component artboards are authored.
#[test]
fn css_hosted_artboard_keeps_its_stored_hug_height() {
    use crate::source::{
        generated::{
            nested_artboard_base::NestedArtboardBase,
            nested_artboard_layout_base::NestedArtboardLayoutBase,
        },
        nested_artboard_layout::NestedArtboardLayout,
    };
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    // 0 outer artboard, 1 its CSS record, 2 the host, 3 its style.
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 400.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 300.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 3)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(NestedArtboardLayoutBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 1)
        .float(NestedArtboardLayoutBase::INSTANCE_WIDTH_PROPERTY_KEY, 200.0)
        .uint(
            NestedArtboardLayoutBase::INSTANCE_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
            LayoutScaleType::Hug as u64,
        )
        .end();
    riv.object(LayoutComponentStyleBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    // 0 component artboard, 1 its record, 2 child, 3 child's record,
    // 4 and 5 their styles.
    riv.object(ArtboardBase::TYPE_KEY)
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 200.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 4)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .end();
    riv.object(LayoutComponentBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, 5)
        .end();
    riv.object(LAYOUT_CSS_STYLE)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
        .uint(WIDTH_KIND, 1)
        .float(WIDTH, 10.0)
        .uint(HEIGHT_KIND, 1)
        .float(HEIGHT, 10.0)
        .end();
    riv.object(LayoutComponentStyleBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0)
        .uint(
            LayoutSizingStyleBase::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
            LayoutScaleType::Hug as u64,
        )
        .bool(
            LayoutComponentStyleBase::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY,
            false,
        )
        .end();
    riv.object(LayoutComponentStyleBase::TYPE_KEY)
        .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 2)
        .end();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &riv.bytes(),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("the scene imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("outer artboard");
    for _ in 0..3 {
        artboard.advance_default(FRAME);
    }
    let hosts =
        artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboardLayout>());
    assert_eq!(hosts.len(), 1);
    let bounds = hosts[0]
        .with_downcast::<NestedArtboardLayout, _>(NestedArtboardLayout::layout_bounds)
        .expect("hosted artboard");
    assert_eq!((bounds.width(), bounds.height()), (200.0, 10.0));
}

/// A CSS artboard keeps its own clip for hits: content past the artboard
/// is not hit though the record's overflow is visible.
#[test]
fn css_artboard_keeps_its_own_clip_for_hits() {
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![]),
            ..ArtboardSpec::default()
        },
        Some(css("Wide", [width(500.0), height(100.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let clips = f.artboard.with_artboard(|artboard| artboard.clip());
    assert!(clips, "artboards clip by default");
    let hits = |x: f32, y: f32| {
        f.named("Wide")
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.hit_test_point(&crate::source::math::vec2d::Vec2D::new(x, y), true, true)
            })
            .unwrap()
    };
    assert!(hits(100.0, 50.0));
    assert!(!hits(450.0, 50.0));
}

/// Right to left, a CSS viewport's content starts at its right edge and
/// scrolls toward its left: dragging right reveals the left side, up to
/// the content's left edge.
#[test]
fn css_rtl_viewport_scrolls_toward_its_left() {
    const RTL: u64 = 2;
    const HORIZONTAL: u64 = 0;
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [
                [U(DISPLAY, 1), U(DIRECTION, RTL), U(OVERFLOW_X, 3)].as_slice(),
                &width(100.0),
                &height(20.0),
            ]
            .concat(),
            vec![B {
                scroll: Some(HORIZONTAL),
                ..css(
                    "Content",
                    [
                        [F(FLEX_SHRINK, 0.0)].as_slice(),
                        &width(300.0),
                        &height(20.0),
                    ]
                    .concat(),
                    vec![css("Item", [width(300.0), height(20.0)].concat(), vec![])],
                )
            }],
        ),
    );
    assert_eq!(f.world_origin("Item").0, -200.0);
    let drag = |dx: f32| {
        use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
        let scroll = f
            .artboard
            .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
            .into_iter()
            .next()
            .expect("a scroll constraint");
        scroll
            .with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                scroll.drag_view(crate::source::math::vec2d::Vec2D::new(dx, 0.0), 0.0, false)
            })
            .unwrap();
        f.advance(2);
    };
    drag(50.0);
    assert_eq!(f.world_origin("Item").0, -150.0);
    drag(500.0);
    assert_eq!(f.world_origin("Item").0, 0.0);
}

/// A CSS artboard with rounded corners and its own clip draws its children
/// through the rounded clip, so they are not hit in the cut corners.
#[test]
fn css_artboard_rounded_clip_stops_hits_in_its_corners() {
    const RADIUS_PERCENT: [u16; 4] = [61079, 61080, 61081, 61082];
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(RADIUS_PERCENT.map(|key| F(key, 50.0)).to_vec()),
            ..ArtboardSpec::default()
        },
        Some(css("Wide", [width(400.0), height(300.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let hits = |x: f32, y: f32| {
        f.named("Wide")
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout.hit_test_point(&crate::source::math::vec2d::Vec2D::new(x, y), true, true)
            })
            .unwrap()
    };
    assert!(hits(200.0, 150.0));
    // The top left corner is cut away by the 200 by 150 elliptical radius.
    assert!(!hits(5.0, 5.0));
}

/// A box painted at a BoxPaint's size: `width: 200px; height: 100px` with a
/// 100 by 40 BoxPaint draws its background centred, from (50, 30).
fn painted_row(paint: Option<Vec<P>>, extra: impl FnOnce(B) -> B) -> Scene {
    scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(400.0), &height(100.0)].concat(),
            vec![
                extra(B {
                    paints: vec![Paint::Fill(0xff11_2233)],
                    box_paint: paint,
                    ..css(
                        "Painted",
                        [
                            [F(FLEX_SHRINK, 0.0)].as_slice(),
                            &width(200.0),
                            &height(100.0),
                        ]
                        .concat(),
                        vec![],
                    )
                }),
                css(
                    "Sibling",
                    [
                        [F(FLEX_SHRINK, 0.0)].as_slice(),
                        &width(50.0),
                        &height(50.0),
                    ]
                    .concat(),
                    vec![],
                ),
            ],
        ),
    )
}

fn background_bounds(f: &Scene, name: &str) -> crate::source::math::aabb::Aabb {
    let (points, _, _) = f.paths(name);
    let mut bounds = crate::source::math::aabb::Aabb::new(
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    );
    for point in points {
        bounds.expand(aabb(point.x, point.y, point.x, point.y));
    }
    bounds
}

/// A BoxPaint paints the box at its own size, centred on the border box,
/// while layout keeps the box's size.
#[test]
fn css_box_paint_sizes_the_painted_box() {
    let f = painted_row(Some(paint_points(100.0, 40.0)), |b| b);
    assert_eq!(f.bounds("Painted"), [0.0, 0.0, 200.0, 100.0]);
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(50.0, 30.0, 150.0, 70.0)
    );
}

/// A BoxPaint keyed by an animation repaints the box, centred, and nothing
/// reflows: the box and its sibling keep their places.
#[test]
fn css_box_paint_keyed_by_an_animation_does_not_reflow() {
    let f = painted_row(Some(paint_points(100.0, 40.0)), |b| B {
        box_paint_keys: vec![
            (PAINT_WIDTH, vec![(0, 100.0), (60, 200.0)]),
            (PAINT_HEIGHT, vec![(0, 40.0), (60, 80.0)]),
        ],
        ..b
    });
    let mut instance =
        crate::ArtboardInstance::from_native_handle(f._file.clone(), 0, f.artboard.clone());
    let mut animation = instance
        .linear_animation_instance(0)
        .expect("the animation");
    instance
        .apply_linear_animation_instance_at(&mut animation, 0.5, 1.0)
        .unwrap();
    f.advance(2);
    // Halfway: 150 by 60, centred on the 200 by 100 box.
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(25.0, 20.0, 175.0, 80.0)
    );
    assert_eq!(f.bounds("Painted"), [0.0, 0.0, 200.0, 100.0]);
    assert_eq!(f.bounds("Sibling"), [200.0, 0.0, 50.0, 50.0]);
}

/// A BoxPaint size bound to a view model follows its value.
#[test]
fn css_box_paint_follows_bound_values() {
    use crate::source::generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase;
    let f = painted_row(Some(paint_points(100.0, 40.0)), |b| B {
        box_paint_bind: Some(PAINT_WIDTH),
        ..b
    });
    let number = f.bind_number();
    assert!(CoreRegistry::set_double_handle(
        &number,
        ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
        120.0
    ));
    f.advance(3);
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(40.0, 30.0, 160.0, 70.0)
    );
}

/// Applies the file's one animation at `seconds` and settles the scene.
fn animate(f: &Scene, seconds: f32) {
    let mut instance =
        crate::ArtboardInstance::from_native_handle(f._file.clone(), 0, f.artboard.clone());
    let mut animation = instance
        .linear_animation_instance(0)
        .expect("the animation");
    instance
        .apply_linear_animation_instance_at(&mut animation, seconds, 1.0)
        .unwrap();
    f.advance(2);
}

/// Writes `value` to the scene's bound view model number and settles it.
fn set_bound(f: &Scene, value: f32) {
    use crate::source::generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase;
    let number = f.bind_number();
    assert!(CoreRegistry::set_double_handle(
        &number,
        ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
        value
    ));
    f.advance(3);
}

/// A BoxPaint with no values paints the border box, and follows it when
/// layout resizes the box: each axis is 0 points plus 100%.
#[test]
fn css_box_paint_at_rest_follows_the_border_box() {
    let f = painted_row(Some(vec![]), |b| B {
        bind: Some(WIDTH),
        ..b
    });
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(0.0, 0.0, 200.0, 100.0)
    );
    set_bound(&f, 300.0);
    assert_eq!(f.bounds("Painted"), [0.0, 0.0, 300.0, 100.0]);
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(0.0, 0.0, 300.0, 100.0)
    );
}

/// One axis keyed in points while the other follows the box's size.
#[test]
fn css_box_paint_keyed_axis_beside_a_following_axis() {
    let f = painted_row(Some(vec![F(PAINT_HEIGHT_PERCENT, 0.0)]), |b| B {
        bind: Some(WIDTH),
        box_paint_keys: vec![(PAINT_HEIGHT, vec![(0, 40.0), (60, 80.0)])],
        ..b
    });
    animate(&f, 0.5);
    // 60 high, and as wide as the 200 point box.
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(0.0, 20.0, 200.0, 80.0)
    );
    set_bound(&f, 300.0);
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(0.0, 20.0, 300.0, 80.0)
    );
}

/// A timeline from (0 points, 100%) to (120 points, 0%) interpolates both
/// parts, as CSS interpolates a mixed length-percentage.
#[test]
fn css_box_paint_interpolates_points_and_percent() {
    let f = painted_row(Some(vec![]), |b| B {
        box_paint_keys: vec![
            (PAINT_WIDTH, vec![(0, 0.0), (60, 120.0)]),
            (PAINT_WIDTH_PERCENT, vec![(0, 100.0), (60, 0.0)]),
        ],
        ..b
    });
    animate(&f, 0.0);
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(0.0, 0.0, 200.0, 100.0)
    );
    // Halfway: 60 points plus 50% of 200.
    animate(&f, 0.5);
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(20.0, 0.0, 180.0, 100.0)
    );
    animate(&f, 1.0);
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(40.0, 0.0, 160.0, 100.0)
    );
}

/// A BoxPaint percentage bound to a view model follows its value, and a
/// negative result paints nothing wide.
#[test]
fn css_box_paint_follows_a_bound_percentage() {
    let f = painted_row(Some(vec![F(PAINT_WIDTH, -150.0)]), |b| B {
        box_paint_bind: Some(PAINT_WIDTH_PERCENT),
        ..b
    });
    set_bound(&f, 125.0);
    // -150 points plus 125% of 200.
    assert_eq!(
        background_bounds(&f, "Painted"),
        aabb(50.0, 0.0, 150.0, 100.0)
    );
    set_bound(&f, 50.0);
    // -50 counts as 0: whatever is left of the path lies on the centre.
    let (points, _, _) = f.paths("Painted");
    assert!(points.iter().all(|point| point.x == 100.0), "{points:?}");
}

/// A shadow grows from the BoxPaint's rectangle, and a gradient covers it.
#[test]
fn css_box_paint_carries_its_shadow_and_gradient() {
    let f = painted_row(Some(paint_points(100.0, 40.0)), |b| B {
        paints: vec![
            Paint::Shadow {
                color: 0x8000_0000,
                strength: 0.0,
                spread: 4.0,
            },
            Paint::Gradient(90.0),
        ],
        ..b
    });
    assert_eq!(f.css_paints("Painted")[0].0, aabb(46.0, 26.0, 154.0, 74.0));
    let lines = f.stream_lines();
    let gradient = lines
        .iter()
        .rev()
        .find(|line| line.starts_with("makeLinearGradient"))
        .expect("a gradient");
    assert_eq!(
        gradient_ends(gradient),
        ((50.0, 50.0), (150.0, 50.0)),
        "{gradient}"
    );
}

/// A CSS artboard's BoxPaint moves its background, not its own clip.
#[test]
fn css_artboard_box_paint_keeps_its_clip() {
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![]),
            paints: vec![Paint::Fill(0xff11_2233)],
            box_paint: Some(paint_points(100.0, 50.0)),
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let lines = f.draw_lines();
    let clip = lines
        .iter()
        .find(|line| line.starts_with("clipPath"))
        .unwrap_or_else(|| panic!("{}", lines.join("\n")));
    // The artboard's bounds; its background starts at -0.
    assert!(clip.contains("(400,-0),(400,300),(-0,300)"), "{clip}");
    let background = lines
        .iter()
        .find(|line| line.starts_with("drawPath") && line.contains("color=0xff112233"))
        .unwrap_or_else(|| panic!("{}", lines.join("\n")));
    assert!(
        background.contains("(150,125),(250,125),(250,175),(150,175)"),
        "{background}"
    );
}

/// World-space strokes use the same painted rectangle as fills. Without
/// a BoxPaint both still use the frame rectangle.
#[test]
fn css_artboard_world_stroke_uses_its_box_paint_rectangle() {
    for box_paint in [None, Some(paint_points(100.0, 50.0))] {
        let painted = box_paint.is_some();
        let f = try_scene_spec(
            (400.0, 300.0),
            ArtboardSpec {
                css: Some(vec![]),
                paints: vec![Paint::Fill(0xff11_2233), Paint::WorldStroke(0xff44_5566)],
                box_paint,
                ..ArtboardSpec::default()
            },
            Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
        )
        .expect("the scene imports");
        let lines = f.draw_lines();
        let fill = &lines[draw_at(&lines, 0xff11_2233)];
        let stroke = &lines[draw_at(&lines, 0xff44_5566)];
        let corners = if painted {
            ["(150,125)", "(250,125)", "(250,175)", "(150,175)"]
        } else {
            ["(400,0)", "(400,300)", "(0,300)", "(0,0)"]
        };
        // The fill may retain negative zero at the artboard's origin.
        for line in [fill, stroke] {
            let normalized = line.replace("-0", "0");
            for corner in corners {
                assert!(normalized.contains(corner), "{line}");
            }
        }
        assert!(stroke.contains("style=stroke"), "{stroke}");
    }
}

const Z_INDEX_KIND: u16 = 61099;
const Z_INDEX: u16 = 61100;

/// Where a draw of `color` sits in `lines`, the order paint happens in.
fn draw_at(lines: &[String], color: u32) -> usize {
    let color = format!("color=0x{color:08x}");
    lines
        .iter()
        .position(|line| line.starts_with("drawPath") && line.contains(&color))
        .unwrap_or_else(|| panic!("a draw with {color}:\n{}", lines.join("\n")))
}

/// A painted CSS box: a Fill of `color` on a box with `props`.
fn painted(name: &'static str, color: u32, props: Vec<P>) -> B {
    B {
        paints: vec![Paint::Fill(color)],
        ..css(name, props, vec![])
    }
}

fn sized(extra: &[P]) -> Vec<P> {
    [extra, &width(100.0), &height(50.0)].concat()
}

/// The frame every paint order test draws in: a positioned flex column.
fn frame(children: Vec<B>) -> Scene {
    scene(
        (400.0, 300.0),
        css(
            "Frame",
            [
                [U(POSITION, 1), U(DISPLAY, 1), U(61006, 2)].as_slice(),
                &width(200.0),
                &height(200.0),
            ]
            .concat(),
            children,
        ),
    )
}

/// CSS paints a box's children in tree order: a later sibling draws on
/// top of an earlier one (Rive draws the first child on top).
#[test]
fn css_later_siblings_paint_on_top() {
    let f = frame(vec![
        painted("A", 0xff00_0001, sized(&[])),
        painted("B", 0xff00_0002, sized(&[])),
    ]);
    let lines = f.draw_lines();
    assert!(draw_at(&lines, 0xff00_0001) < draw_at(&lines, 0xff00_0002));
}

/// An early `position: absolute; inset: 0` background stays below what
/// follows it when that is positioned too (tree order among positioned
/// boxes), or when it has a negative z-index.
#[test]
fn css_early_absolute_background_stays_below() {
    let inset = |kind: u16| [U(kind, 1), F(kind + 1, 0.0)];
    let background = |extra: &[P]| {
        painted(
            "Background",
            0xff00_0001,
            [
                [U(POSITION, 2)].as_slice(),
                &inset(INSET_TOP_KIND),
                &inset(61054),
                &inset(61057),
                &inset(INSET_LEFT_KIND),
                extra,
            ]
            .concat(),
        )
    };
    let f = frame(vec![
        background(&[]),
        painted("Content", 0xff00_0002, sized(&[U(POSITION, 1)])),
    ]);
    let lines = f.draw_lines();
    assert!(draw_at(&lines, 0xff00_0001) < draw_at(&lines, 0xff00_0002));
    let f = frame(vec![
        background(&[U(Z_INDEX_KIND, 1), F(Z_INDEX, -1.0)]),
        painted("Content", 0xff00_0002, sized(&[])),
    ]);
    let lines = f.draw_lines();
    assert!(draw_at(&lines, 0xff00_0001) < draw_at(&lines, 0xff00_0002));
}

/// Positioned boxes paint over in-flow ones, whatever their tree order,
/// and in-flow boxes paint in tree order.
#[test]
fn css_positioned_boxes_paint_over_in_flow_ones() {
    let f = frame(vec![
        painted("Raised", 0xff00_0001, sized(&[U(POSITION, 1)])),
        painted("Flat", 0xff00_0002, sized(&[])),
        painted("Later", 0xff00_0003, sized(&[])),
    ]);
    let lines = f.draw_lines();
    let mut colors = [1u32, 2, 3];
    colors.sort_by_key(|color| draw_at(&lines, 0xff00_0000 + color));
    assert_eq!(colors, [2, 3, 1]);
}

/// z-index orders positioned boxes: negative below the in-flow content,
/// positive above it, ascending, and a z-index written at run time
/// reorders them.
#[test]
fn css_z_index_orders_positioned_boxes() {
    let z = |value: f32| [U(POSITION, 1), U(Z_INDEX_KIND, 1), F(Z_INDEX, value)];
    let f = frame(vec![
        painted("Below", 0xff00_0001, sized(&z(-1.0))),
        painted("Flow", 0xff00_0002, sized(&[])),
        painted("Two", 0xff00_0003, sized(&z(2.0))),
        painted("One", 0xff00_0004, sized(&z(1.0))),
    ]);
    let order = |f: &Scene| {
        let lines = f.draw_lines();
        let mut colors = [1u32, 2, 3, 4];
        colors.sort_by_key(|color| draw_at(&lines, 0xff00_0000 + color));
        colors
    };
    assert_eq!(order(&f), [1, 2, 4, 3]);
    assert!(CoreRegistry::set_double_handle(
        &f.record("Below"),
        Z_INDEX.into(),
        3.0
    ));
    f.advance(2);
    assert_eq!(order(&f), [2, 4, 3, 1]);
}

/// Hits follow the paint order: the topmost painted box comes first in
/// the order the state machine hit-tests in.
#[test]
fn css_hit_order_follows_the_paint_order() {
    let f = frame(vec![
        painted("A", 0xff00_0001, sized(&[])),
        painted("B", 0xff00_0002, sized(&[])),
    ]);
    // The draw list from its top, as StateMachineInstance sorts its hits.
    let top_first = f.draw_names_top_first();
    let at = |name: &str| top_first.iter().position(|n| n == name).unwrap();
    assert!(at("B") < at("A"), "{top_first:?}");
}

/// Paint order and the proxy order used by state machine hits agree with
/// host geometry hits at an overlap point. The expectation is bottom first.
fn assert_stacking_order(f: &Scene, expected: &[(&str, u32)], point: (f32, f32)) {
    let lines = f.draw_lines();
    let painted_at = |(name, color): (&str, u32)| {
        let opacity = f
            .named(name)
            .with(|object| object.world_transform_child_opacity())
            .flatten()
            .expect("the box's render opacity");
        draw_at(
            &lines,
            crate::source::shapes::paint::color::color_modulate_opacity(color, opacity),
        )
    };
    for pair in expected.windows(2) {
        assert!(
            painted_at(pair[0]) < painted_at(pair[1]),
            "{} should paint below {}: {}",
            pair[0].0,
            pair[1].0,
            lines.join("\n")
        );
    }
    let top_first: Vec<_> = expected.iter().rev().map(|(name, _)| *name).collect();
    let proxies = f.artboard.with_artboard(|artboard| {
        artboard
            .drawables()
            .iter()
            .filter_map(|item| {
                let crate::source::drawable::RuntimeDrawableOccurrence::RuntimeProxy(proxy) = item
                else {
                    return None;
                };
                proxy
                    .borrow()
                    .owner_handle()
                    .with(|owner| {
                        owner
                            .as_component()
                            .map(|component| component.name().to_owned())
                    })
                    .flatten()
            })
            .filter(|name| top_first.contains(&name.as_str()))
            .collect::<Vec<_>>()
    });
    assert_eq!(proxies, top_first, "state machine proxy order");
    let hits = f
        .host()
        .hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(point.0, point.1));
    let hits: Vec<_> = f
        .geometry_names(&hits)
        .into_iter()
        .filter(|name| top_first.contains(&name.as_str()))
        .collect();
    assert_eq!(hits, top_first, "host geometry hit order");
}

const CARD: (&str, u32) = ("Card", 0xffcc_3333);
const LATER: (&str, u32) = ("Later", 0xff33_33cc);
const BADGE: (&str, u32) = ("Badge", 0xff33_cc33);
const FLOW: (&str, u32) = ("Flow", 0xff99_9999);
const BOX_OPACITY: u16 = 18;
const BOX_ROTATION: u16 = 15;
const BOX_SCALE_X: u16 = 16;

fn stacking_scene(children: Vec<B>, flex: bool) -> Scene {
    try_scene(
        (400.0, 300.0),
        Some(vec![]),
        css(
            "Frame",
            [
                vec![U(POSITION, 1), U(DISPLAY, u64::from(flex)), U(61006, 2)],
                width(200.0).to_vec(),
                height(240.0).to_vec(),
            ]
            .concat(),
            children,
        ),
    )
    .expect("the scene imports")
}

fn stacking_box((name, color): (&'static str, u32), extra: &[P]) -> B {
    painted(
        name,
        color,
        [extra, &width(200.0), &height(100.0), &[F(FLEX_SHRINK, 0.0)]].concat(),
    )
}

fn stacking_z(z: f32) -> [P; 2] {
    [U(Z_INDEX_KIND, 1), F(Z_INDEX, z)]
}

/// Chrome's card and overlapping sibling, with the badge positioned against
/// the frame when the card is static. Wrappers contribute no paint or clip.
fn stacking_badge(card: B, later: &[P], badge_z: Option<f32>, wrappers: bool) -> Scene {
    let mut badge = painted(
        BADGE.0,
        BADGE.1,
        [
            vec![
                U(POSITION, 2),
                U(INSET_TOP_KIND, 1),
                F(INSET_TOP, 80.0),
                U(INSET_LEFT_KIND, 1),
                F(INSET_LEFT, 10.0),
            ],
            width(60.0).to_vec(),
            height(40.0).to_vec(),
            badge_z.map(|z| stacking_z(z).to_vec()).unwrap_or_default(),
        ]
        .concat(),
    );
    if wrappers {
        badge = css("Outer", vec![], vec![css("Inner", vec![], vec![badge])]);
    }
    stacking_scene(
        vec![
            B {
                children: vec![badge],
                ..card
            },
            stacking_box(LATER, &[later, &[F(MARGIN_TOP, -30.0)]].concat()),
        ],
        false,
    )
}

#[test]
fn css_stacking_badge_escapes_static_card() {
    let f = stacking_badge(stacking_box(CARD, &[]), &[], Some(10.0), false);
    assert_stacking_order(&f, &[CARD, LATER, BADGE], (30.0, 90.0));
}

#[test]
fn css_stacking_auto_badge_escapes_static_card() {
    let f = stacking_badge(stacking_box(CARD, &[]), &[], None, false);
    assert_stacking_order(&f, &[CARD, LATER, BADGE], (30.0, 90.0));
}

#[test]
fn css_stacking_badge_escapes_above_positive_sibling() {
    let f = stacking_badge(
        stacking_box(CARD, &[]),
        &[&[U(POSITION, 1)][..], &stacking_z(5.0)].concat(),
        Some(10.0),
        false,
    );
    assert_stacking_order(&f, &[CARD, LATER, BADGE], (30.0, 90.0));
}

#[test]
fn css_stacking_badge_escapes_two_static_wrappers() {
    let f = stacking_badge(
        stacking_box(CARD, &[]),
        &[&[U(POSITION, 1)][..], &stacking_z(5.0)].concat(),
        Some(10.0),
        true,
    );
    assert_stacking_order(&f, &[CARD, LATER, BADGE], (30.0, 90.0));
}

fn stacking_negative(card: B, flex_item: bool, flow: bool) -> Scene {
    let negative = painted(
        BADGE.0,
        BADGE.1,
        [
            vec![
                U(POSITION, 1),
                U(INSET_TOP_KIND, 1),
                F(INSET_TOP, 10.0),
                U(INSET_LEFT_KIND, 1),
                F(INSET_LEFT, 10.0),
            ],
            stacking_z(-1.0).to_vec(),
            width(60.0).to_vec(),
            height(40.0).to_vec(),
        ]
        .concat(),
    );
    let mut children = vec![negative];
    if flow {
        children.push(painted(
            FLOW.0,
            FLOW.1,
            [
                vec![F(MARGIN_TOP, -20.0), F(61049, 30.0)],
                width(60.0).to_vec(),
                height(40.0).to_vec(),
            ]
            .concat(),
        ));
    }
    stacking_scene(vec![B { children, ..card }], flex_item)
}

#[test]
fn css_stacking_negative_stays_above_context_background() {
    let f = stacking_negative(
        stacking_box(CARD, &[&[U(POSITION, 1)][..], &stacking_z(0.0)].concat()),
        false,
        true,
    );
    assert_stacking_order(&f, &[CARD, BADGE], (15.0, 25.0));
    assert_stacking_order(&f, &[CARD, BADGE, FLOW], (50.0, 35.0));
}

#[test]
fn css_stacking_negative_escapes_behind_static_background() {
    let f = stacking_negative(stacking_box(CARD, &[]), false, false);
    assert_stacking_order(&f, &[BADGE, CARD], (15.0, 25.0));
}

#[test]
fn css_stacking_negative_escapes_behind_relative_auto_background() {
    let f = stacking_negative(stacking_box(CARD, &[U(POSITION, 1)]), false, false);
    assert_stacking_order(&f, &[BADGE, CARD], (15.0, 25.0));
}

#[test]
fn css_stacking_negative_stays_in_opacity_context() {
    let f = stacking_negative(
        B {
            component: vec![F(BOX_OPACITY, 0.99)],
            ..stacking_box(CARD, &[])
        },
        false,
        false,
    );
    assert_stacking_order(&f, &[CARD, BADGE], (15.0, 25.0));
}

#[test]
fn css_stacking_negative_stays_in_transform_context() {
    let f = stacking_negative(
        B {
            component_keys: vec![(BOX_ROTATION, vec![(0, 0.0)])],
            ..stacking_box(CARD, &[])
        },
        false,
        false,
    );
    assert_stacking_order(&f, &[CARD, BADGE], (15.0, 25.0));
}

#[test]
fn css_stacking_negative_stays_in_flex_item_zero_context() {
    let f = stacking_negative(stacking_box(CARD, &stacking_z(0.0)), true, false);
    assert_stacking_order(&f, &[CARD, BADGE], (15.0, 25.0));
}

#[test]
fn css_stacking_negative_flex_item_escapes_behind_absolute_parent() {
    let child = painted(
        BADGE.0,
        BADGE.1,
        [&stacking_z(-1.0)[..], &width(60.0), &height(40.0)].concat(),
    );
    let f = stacking_scene(
        vec![B {
            children: vec![child],
            ..stacking_box(CARD, &[U(POSITION, 2), U(DISPLAY, 1)])
        }],
        false,
    );
    assert_stacking_order(&f, &[BADGE, CARD], (15.0, 25.0));
}

fn assert_badge_is_contained(card: B) {
    let f = stacking_badge(
        card,
        &[&[U(POSITION, 1)][..], &stacking_z(5.0)].concat(),
        Some(10.0),
        false,
    );
    assert_stacking_order(&f, &[CARD, BADGE, LATER], (30.0, 90.0));
}

#[test]
fn css_stacking_opacity_contains_positive_badge() {
    assert_badge_is_contained(B {
        component: vec![F(BOX_OPACITY, 0.99)],
        ..stacking_box(CARD, &[])
    });
}

#[test]
fn css_stacking_keyed_rotation_contains_positive_badge() {
    assert_badge_is_contained(B {
        component_keys: vec![(BOX_ROTATION, vec![(0, 0.0)])],
        ..stacking_box(CARD, &[])
    });
}

#[test]
fn css_stacking_keyed_translation_contains_positive_badge() {
    assert_badge_is_contained(B {
        component_keys: vec![(NodeBase::X_PROPERTY_KEY, vec![(0, 0.0)])],
        ..stacking_box(CARD, &[])
    });
}

fn stacking_siblings(first: B, later: B, flex: bool) -> Scene {
    let mut later = later;
    later.css.as_mut().unwrap().push(F(MARGIN_TOP, -30.0));
    stacking_scene(vec![first, later], flex)
}

#[test]
fn css_stacking_later_opacity_paints_above_relative() {
    let f = stacking_siblings(
        stacking_box(CARD, &[U(POSITION, 1)]),
        B {
            component: vec![F(BOX_OPACITY, 0.99)],
            ..stacking_box(LATER, &[])
        },
        false,
    );
    assert_stacking_order(&f, &[CARD, LATER], (30.0, 90.0));
}

#[test]
fn css_stacking_relative_paints_above_later_plain_box() {
    let f = stacking_siblings(
        stacking_box(CARD, &[U(POSITION, 1)]),
        stacking_box(LATER, &[]),
        false,
    );
    assert_stacking_order(&f, &[LATER, CARD], (30.0, 90.0));
}

fn assert_keyed_transform_paints_above_relative(key: u16, identity: f32) {
    let f = stacking_siblings(
        stacking_box(CARD, &[U(POSITION, 1)]),
        B {
            component_keys: vec![(key, vec![(0, identity), (60, identity)])],
            ..stacking_box(LATER, &[])
        },
        false,
    );
    // The track is enough to establish a context before it is played and
    // after its end value holds at identity.
    assert_stacking_order(&f, &[CARD, LATER], (30.0, 90.0));
    animate(&f, 1.0);
    assert_stacking_order(&f, &[CARD, LATER], (30.0, 90.0));
}

#[test]
fn css_stacking_later_keyed_rotation_paints_above_relative() {
    assert_keyed_transform_paints_above_relative(BOX_ROTATION, 0.0);
}

#[test]
fn css_stacking_later_keyed_translation_paints_above_relative() {
    assert_keyed_transform_paints_above_relative(NodeBase::X_PROPERTY_KEY, 0.0);
    assert_keyed_transform_paints_above_relative(NodeBase::Y_PROPERTY_KEY, 0.0);
}

#[test]
fn css_stacking_later_keyed_scale_paints_above_relative() {
    assert_keyed_transform_paints_above_relative(BOX_SCALE_X, 1.0);
    assert_keyed_transform_paints_above_relative(17, 1.0);
}

#[test]
fn css_stacking_bound_transform_paints_above_relative() {
    let f = stacking_siblings(
        stacking_box(CARD, &[U(POSITION, 1)]),
        B {
            component_bind: Some(BOX_ROTATION),
            ..stacking_box(LATER, &[])
        },
        false,
    );
    set_bound(&f, 0.0);
    assert_stacking_order(&f, &[CARD, LATER], (30.0, 90.0));
    let counter = stacking_sort_count(&f);
    set_bound(&f, 0.01);
    set_bound(&f, 0.0);
    assert_stacking_order(&f, &[CARD, LATER], (30.0, 90.0));
    assert_eq!(stacking_sort_count(&f), counter);
}

#[test]
fn css_stacking_earlier_opacity_paints_above_later_flow() {
    let f = stacking_siblings(
        B {
            component: vec![F(BOX_OPACITY, 0.99)],
            ..stacking_box(CARD, &[])
        },
        stacking_box(LATER, &[]),
        false,
    );
    assert_stacking_order(&f, &[LATER, CARD], (30.0, 90.0));
}

#[test]
fn css_stacking_earlier_opacity_flex_item_paints_above_later_flow() {
    let f = stacking_siblings(
        B {
            component: vec![F(BOX_OPACITY, 0.99)],
            ..stacking_box(CARD, &[])
        },
        stacking_box(LATER, &[]),
        true,
    );
    assert_stacking_order(&f, &[LATER, CARD], (30.0, 90.0));
}

fn stacking_sort_count(f: &Scene) -> u8 {
    f.artboard
        .with_artboard(|artboard| artboard.dirty_handle().draw_order_change_counter())
}

#[test]
fn css_stacking_opacity_resorts_only_when_crossing_one() {
    let f = stacking_siblings(
        stacking_box(CARD, &[U(POSITION, 1)]),
        B {
            component_keys: vec![(BOX_OPACITY, vec![(0, 1.0), (30, 0.5), (60, 1.0)])],
            ..stacking_box(LATER, &[])
        },
        false,
    );
    assert_stacking_order(&f, &[LATER, CARD], (30.0, 90.0));
    let start = stacking_sort_count(&f);
    animate(&f, 0.25);
    assert_stacking_order(&f, &[CARD, LATER], (30.0, 90.0));
    assert_eq!(stacking_sort_count(&f), start.wrapping_add(1));
    animate(&f, 0.5);
    assert_eq!(stacking_sort_count(&f), start.wrapping_add(1));
    animate(&f, 1.0);
    assert_stacking_order(&f, &[LATER, CARD], (30.0, 90.0));
    assert_eq!(stacking_sort_count(&f), start.wrapping_add(2));
    animate(&f, 1.0);
    assert_eq!(stacking_sort_count(&f), start.wrapping_add(2));
}

#[test]
fn css_stacking_script_transform_resorts_only_at_identity() {
    for (key, identity, changed) in [
        (13, 0.0, 1.0),
        (14, 0.0, 1.0),
        (15, 0.0, 0.01),
        (16, 1.0, 1.01),
        (17, 1.0, 1.01),
    ] {
        let f = stacking_siblings(
            stacking_box(CARD, &[U(POSITION, 1)]),
            stacking_box(LATER, &[]),
            false,
        );
        assert_stacking_order(&f, &[LATER, CARD], (30.0, 90.0));
        let start = stacking_sort_count(&f);
        for value in [changed, changed * 1.01, identity] {
            assert!(CoreRegistry::set_double_handle(
                &f.named(LATER.0),
                key,
                value
            ));
            f.advance(3);
            let expected = if value == identity {
                [LATER, CARD]
            } else {
                [CARD, LATER]
            };
            assert_stacking_order(&f, &expected, (30.0, 90.0));
        }
        assert_eq!(stacking_sort_count(&f), start.wrapping_add(2));
    }
}

#[test]
fn css_stacking_overflow_toggle_rebuilds_escape_boundary() {
    let f = stacking_badge(
        stacking_box(CARD, &[U(POSITION, 1)]),
        &[&[U(POSITION, 1)][..], &stacking_z(5.0)].concat(),
        Some(10.0),
        false,
    );
    assert_stacking_order(&f, &[CARD, LATER, BADGE], (30.0, 90.0));
    let start = stacking_sort_count(&f);
    assert!(CoreRegistry::set_uint_handle(
        &f.record(CARD.0),
        OVERFLOW_X.into(),
        2
    ));
    f.advance(3);
    assert_stacking_order(&f, &[CARD, BADGE, LATER], (30.0, 90.0));
    assert_eq!(stacking_sort_count(&f), start.wrapping_add(1));
    assert!(CoreRegistry::set_uint_handle(
        &f.record(CARD.0),
        OVERFLOW_X.into(),
        0
    ));
    f.advance(3);
    assert_stacking_order(&f, &[CARD, LATER, BADGE], (30.0, 90.0));
    assert_eq!(stacking_sort_count(&f), start.wrapping_add(2));
}

#[test]
fn css_stacking_masked_card_keeps_its_subtree_and_mask_bracket() {
    use crate::source::{
        drawable::{Drawable, RuntimeDrawableOccurrence},
        layer_mask::LayerMask,
    };
    let badge = painted(
        BADGE.0,
        BADGE.1,
        [
            vec![
                U(POSITION, 2),
                U(INSET_TOP_KIND, 1),
                F(INSET_TOP, 80.0),
                U(INSET_LEFT_KIND, 1),
                F(INSET_LEFT, 10.0),
            ],
            stacking_z(10.0).to_vec(),
            width(60.0).to_vec(),
            height(40.0).to_vec(),
        ]
        .concat(),
    );
    let f = stacking_scene(
        vec![
            B {
                mask_source: Some("MaskSource"),
                children: vec![badge],
                ..stacking_box(CARD, &[])
            },
            stacking_box(LATER, &[F(MARGIN_TOP, -30.0)]),
            painted(
                "MaskSource",
                0xffff_ffff,
                [
                    vec![U(POSITION, 2), U(INSET_LEFT_KIND, 1), F(INSET_LEFT, 300.0)],
                    width(200.0).to_vec(),
                    height(100.0).to_vec(),
                ]
                .concat(),
            ),
        ],
        false,
    );
    assert!(
        !f.named(CARD.0)
            .with_downcast::<LayoutComponent, _>(LayoutComponent::clips)
            .unwrap()
    );
    let (mask, items, mut current) = f.artboard.with_artboard(|artboard| {
        (
            artboard.all_layer_masks()[0].clone(),
            artboard.drawables().to_vec(),
            artboard.first_drawable(),
        )
    });
    let members: Vec<_> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.layer_masks().contains(&mask))
        .map(|(index, _)| index)
        .collect();
    // Both boxes' own drawables and paint proxies stay in one range, with
    // the unrelated later sibling outside it.
    assert_eq!(members.len(), 4);
    assert_eq!(members.last().unwrap() - members[0] + 1, members.len());
    let badge = RuntimeDrawableOccurrence::authored(f.named(BADGE.0));
    let later = RuntimeDrawableOccurrence::authored(f.named(LATER.0));
    assert!(
        items.iter().position(|item| item == &later).unwrap()
            < items.iter().position(|item| item == &badge).unwrap()
    );
    let mut chain = Vec::new();
    while let Some(item) = current {
        current = item.with(Drawable::prev_drawable).flatten();
        chain.push(item);
    }
    let start = chain
        .iter()
        .position(|item| item.is_mask_start())
        .expect("mask start spliced");
    let end = chain
        .iter()
        .position(|item| item.is_mask_end())
        .expect("mask end spliced");
    assert_eq!(end - start - 1, members.len());
    assert!(
        chain[start + 1..end]
            .iter()
            .all(|item| item.layer_masks().contains(&mask))
    );
    let paired_end = chain[start]
        .layer_mask_marker()
        .unwrap()
        .borrow()
        .paired_end
        .as_ref()
        .and_then(|end| end.upgrade())
        .expect("paired mask end");
    assert!(paired_end == chain[end]);
    assert!(
        mask.with_downcast::<LayerMask, _>(LayerMask::source_start)
            .flatten()
            .is_some()
    );
}

#[test]
fn css_stacking_source_only_transform_binding_allows_badge_to_escape() {
    let f = stacking_badge(
        B {
            component_bind: Some(BOX_ROTATION),
            component_bind_flags: u64::from(crate::source::data_bind::data_bind::TO_SOURCE),
            ..stacking_box(CARD, &[])
        },
        &[&[U(POSITION, 1)][..], &stacking_z(5.0)].concat(),
        Some(10.0),
        false,
    );
    assert_stacking_order(&f, &[CARD, LATER, BADGE], (30.0, 90.0));
    set_bound(&f, 0.5);
    assert_eq!(
        f.named(CARD.0)
            .with_downcast::<LayoutComponent, _>(LayoutComponent::rotation),
        Some(0.0)
    );
    assert_stacking_order(&f, &[CARD, LATER, BADGE], (30.0, 90.0));
}

#[test]
fn css_stacking_clipping_card_keeps_its_positive_badge_atomic() {
    assert_badge_is_contained(stacking_box(
        CARD,
        &[U(POSITION, 1), U(OVERFLOW_X, 2), U(OVERFLOW_Y, 2)],
    ));
}

/// A box's own paint stays at the bottom of its stack whatever order its
/// drawables arrive in. A retained remount (a late font or catalog
/// registration re-creating the text inside a box) appends the re-created
/// children after the box's own drawable and proxy; the box's fills,
/// strokes, borders and shadows must still paint below them, and its clip
/// still close above them, as a cold load orders them.
#[test]
fn css_box_paint_stays_below_children_appended_after_it() {
    use crate::css_layout::paint_order;
    use crate::source::drawable::RuntimeDrawableOccurrence;
    let f = frame(vec![B {
        children: vec![painted("Phrase", 0xff00_0002, sized(&[]))],
        ..painted("Detail", 0xff00_0001, sized(&[]))
    }]);
    let name = |item: &RuntimeDrawableOccurrence| {
        let (owner, proxy) = match item {
            RuntimeDrawableOccurrence::Authored(handle) => (handle.clone(), ""),
            RuntimeDrawableOccurrence::RuntimeProxy(proxy) => {
                (proxy.borrow().owner_handle(), " proxy")
            }
        };
        let owner = owner
            .with(|object| object.as_component().map(|c| c.name().to_owned()))
            .flatten()
            .unwrap_or_default();
        format!("{owner}{proxy}")
    };
    let root = f.artboard.core_handle();
    let (cold, file_order) = f.artboard.with_artboard(|artboard| {
        let file_order = artboard
            .objects()
            .iter()
            .enumerate()
            .filter_map(|(index, object)| object.clone().map(|object| (object, index)))
            .collect::<std::collections::HashMap<_, _>>();
        (artboard.drawables().to_vec(), file_order)
    });
    let names = |items: &[RuntimeDrawableOccurrence]| items.iter().map(name).collect::<Vec<_>>();
    let cold_names = names(&cold);
    let at = |names: &[String], wanted: &str| {
        names
            .iter()
            .position(|name| name == wanted)
            .unwrap_or_else(|| panic!("{wanted} in {names:?}"))
    };
    // The cold load: the box's clip closes above its child, its paint
    // (the proxy) sits below it.
    assert!(
        at(&cold_names, "Detail") < at(&cold_names, "Phrase"),
        "{cold_names:?}"
    );
    assert!(
        at(&cold_names, "Phrase") < at(&cold_names, "Detail proxy"),
        "{cold_names:?}"
    );
    // The remount: Phrase's drawables arrive last, after Detail's own.
    let (phrase, mut remount): (Vec<_>, Vec<_>) = cold
        .iter()
        .cloned()
        .partition(|item| name(item).starts_with("Phrase"));
    remount.extend(phrase);
    let ordered = paint_order::order(
        &root,
        paint_order::container_of(&root),
        remount,
        &file_order,
    );
    assert_eq!(names(&ordered), cold_names);
}

/// The same on a cold mount: a box whose children's objects come after
/// another box's, as a text input's materialized hint and value do, reaches
/// the draw list with its own drawable and proxy before its children. Its
/// background still paints below their ink.
#[test]
fn css_box_paint_stays_below_children_mounted_after_it() {
    let f = frame(vec![
        B {
            late_children: vec![painted("Phrase", 0xff00_0002, sized(&[]))],
            ..painted("Detail", 0xff00_0001, sized(&[]))
        },
        painted("Other", 0xff00_0003, sized(&[])),
    ]);
    let top_first = f.draw_names_top_first();
    let at = |name: &str| top_first.iter().position(|n| n == name).unwrap();
    assert!(at("Detail") < at("Phrase"), "{top_first:?}");
    let lines = f.draw_lines();
    assert!(
        draw_at(&lines, 0xff00_0001) < draw_at(&lines, 0xff00_0002),
        "{}",
        lines.join("\n")
    );
}

impl Scene {
    fn host(&self) -> crate::ArtboardInstance {
        crate::ArtboardInstance::from_native_handle(self._file.clone(), 0, self.artboard.clone())
    }
    /// The object names a geometry list's last path segments point at.
    fn geometry_names(&self, hits: &[crate::RuntimeGeometryHit]) -> Vec<String> {
        hits.iter()
            .map(|hit| {
                let local_id = hit.path.last().expect("a path").local_id;
                self.artboard
                    .with_artboard(|artboard| artboard.objects()[local_id].clone())
                    .and_then(|handle| {
                        handle
                            .with(|object| object.as_component().map(|c| c.name().to_owned()))
                            .flatten()
                    })
                    .unwrap_or_default()
            })
            .collect()
    }
}

/// A CSS box that paints itself is geometry: visible with its border box,
/// retained, and hit; a box without paint is not.
#[test]
fn css_painted_box_is_catalogued_geometry() {
    let f = frame(vec![
        painted("Card", 0xff00_0001, sized(&[])),
        css("Plain", sized(&[]), vec![]),
    ]);
    let mut host = f.host();
    let visible = host.visible_geometry_with_bounds();
    let names = f.geometry_names(&visible);
    assert_eq!(names, ["Card"], "{names:?}");
    let bounds = visible[0].bounds;
    assert_eq!(
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y),
        (0.0, 0.0, 100.0, 50.0)
    );
    let retained = host.retained_geometry_with_bounds();
    assert!(f.geometry_names(&retained).contains(&"Card".to_owned()));
    let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(10.0, 10.0));
    assert_eq!(f.geometry_names(&hits), ["Card"]);
}

/// Hits follow the CSS painting order: of two painted boxes over each
/// other, the later one is on top and hit first.
#[test]
fn css_painted_box_hits_follow_the_paint_order() {
    let inset = |kind: u16| [U(kind, 1), F(kind + 1, 0.0)];
    let layer = |name, color| {
        painted(
            name,
            color,
            [
                [U(POSITION, 2)].as_slice(),
                &inset(INSET_TOP_KIND),
                &inset(61054),
                &inset(61057),
                &inset(INSET_LEFT_KIND),
            ]
            .concat(),
        )
    };
    let f = frame(vec![
        layer("Lower", 0xff00_0001),
        layer("Upper", 0xff00_0002),
    ]);
    let hits = f
        .host()
        .hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(10.0, 10.0));
    assert_eq!(f.geometry_names(&hits), ["Upper", "Lower"]);
}

/// A painted box outside its ancestor's clip is not visible geometry.
#[test]
fn css_painted_box_outside_a_clip_is_not_visible() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Frame",
            [
                [
                    U(DISPLAY, 1),
                    U(61006, 2),
                    U(OVERFLOW_X, 2),
                    U(OVERFLOW_Y, 2),
                ]
                .as_slice(),
                &width(100.0),
                &height(50.0),
            ]
            .concat(),
            vec![
                css(
                    "Spacer",
                    [
                        [F(FLEX_SHRINK, 0.0)].as_slice(),
                        &width(100.0),
                        &height(60.0),
                    ]
                    .concat(),
                    vec![],
                ),
                painted(
                    "Out",
                    0xff00_0001,
                    [
                        [F(FLEX_SHRINK, 0.0)].as_slice(),
                        &width(100.0),
                        &height(20.0),
                    ]
                    .concat(),
                ),
            ],
        ),
    );
    let visible = f.host().visible_geometry_with_bounds();
    assert!(
        f.geometry_names(&visible).is_empty(),
        "{:?}",
        f.geometry_names(&visible)
    );
}

/// A CSS artboard that paints itself is geometry too.
#[test]
fn css_painted_artboard_is_catalogued_geometry() {
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![]),
            paints: vec![Paint::Fill(0xff11_2233)],
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let visible = f.host().visible_geometry_with_bounds();
    assert_eq!(f.geometry_names(&visible), ["Artboard"]);
    let bounds = visible[0].bounds;
    assert_eq!(
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y),
        (0.0, 0.0, 400.0, 300.0)
    );
}

impl Scene {
    /// Any component named `name`.
    fn object(&self, name: &str) -> CoreHandle {
        self.artboard
            .with_artboard(|artboard| artboard.objects().to_vec())
            .into_iter()
            .flatten()
            .find(|handle: &CoreHandle| {
                handle
                    .with(|object| object.as_component().is_some_and(|c| c.name() == name))
                    .unwrap_or(false)
            })
            .unwrap_or_else(|| panic!("an object named {name}"))
    }
    /// The artboard-local id of the component named `name`.
    fn local_id(&self, name: &str) -> usize {
        let wanted = self.object(name);
        self.artboard
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .position(|handle| handle.as_ref() == Some(&wanted))
            })
            .unwrap_or_else(|| panic!("{name} has a local id"))
    }
    /// The names of the drawables' components, the top of the draw list
    /// first, as hits are tested.
    fn draw_names_top_first(&self) -> Vec<String> {
        self.artboard.with_artboard(|artboard| {
            let mut top = artboard.first_drawable();
            while let Some(previous) = top.as_ref().and_then(|drawable| {
                drawable
                    .with(|drawable| drawable.prev.as_ref().and_then(|prev| prev.upgrade()))
                    .flatten()
            }) {
                top = Some(previous);
            }
            let mut names = Vec::new();
            while let Some(drawable) = top {
                if let Some(name) = drawable.authored_handle().and_then(|handle| {
                    handle
                        .with(|object| object.as_component().map(|c| c.name().to_owned()))
                        .flatten()
                }) {
                    names.push(name);
                }
                top = drawable
                    .with(|drawable| drawable.next.as_ref().and_then(|next| next.upgrade()))
                    .flatten();
            }
            names
        })
    }
}

/// A positioned participant paints above a later static sibling, as a
/// positioned box does, and a run-time position write sorts again.
#[test]
fn css_positioned_participant_paints_above_static_siblings() {
    let with_icon = |position: u64| {
        scene(
            (400.0, 300.0),
            B {
                participants: vec![(
                    Rect {
                        name: "Icon",
                        size: (20.0, 20.0),
                        grouped: false,
                        path_scale_x: 1.0,
                    },
                    Some(
                        [
                            [U(POSITION, position)].as_slice(),
                            &width(20.0),
                            &height(20.0),
                        ]
                        .concat(),
                    ),
                )],
                ..css(
                    "Frame",
                    [
                        [U(POSITION, 1), U(DISPLAY, 1), U(61006, 2)].as_slice(),
                        &width(200.0),
                        &height(200.0),
                    ]
                    .concat(),
                    vec![painted("Later", 0xff00_0001, sized(&[]))],
                )
            },
        )
    };
    let above = |f: &Scene| {
        let names = f.draw_names_top_first();
        let at = |name: &str| names.iter().position(|n| n == name).unwrap();
        at("Icon") < at("Later")
    };
    assert!(!above(&with_icon(0)));
    assert!(above(&with_icon(1)));
    let f = with_icon(0);
    let record = f
        .object("Icon")
        .with(|object| {
            object
                .as_container_component()
                .and_then(|container| crate::css_layout::record::find_in(container.children()))
        })
        .flatten()
        .expect("the icon's record");
    assert!(CoreRegistry::set_uint_handle(&record, POSITION.into(), 1));
    f.advance(2);
    assert!(above(&f), "{:?}", f.draw_names_top_first());
}

/// A box with a smaller BoxPaint is selected by visible geometry only where
/// it paints, while a tap (interaction hit testing) takes its whole border
/// box (docs/CSS_LAYOUT.md, 4.5).
#[test]
fn css_box_paint_selects_its_rectangle_and_takes_taps_in_its_border_box() {
    let f = painted_row(Some(paint_points(100.0, 40.0)), |b| b);
    let mut host = f.host();
    let visible = host.visible_geometry_with_bounds();
    let bounds = visible[f
        .geometry_names(&visible)
        .iter()
        .position(|n| n == "Painted")
        .unwrap()]
    .bounds;
    assert_eq!(
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y),
        (50.0, 30.0, 150.0, 70.0)
    );
    let mut selected = |x: f32, y: f32| {
        let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(x, y));
        f.geometry_names(&hits).contains(&"Painted".to_owned())
    };
    assert!(!selected(10.0, 10.0));
    assert!(selected(100.0, 50.0));
    assert!(tapped(&f, "Painted", (10.0, 10.0)));
}

/// Whether a tap at `point` (in the artboard) hits `name` itself, as
/// interaction hit testing asks.
fn tapped(f: &Scene, name: &str, point: (f32, f32)) -> bool {
    f.object(name)
        .with_mut(|object| {
            object.component_hit_test_point(
                &crate::source::math::vec2d::Vec2D::new(point.0, point.1),
                false,
                true,
            )
        })
        .flatten()
        .unwrap_or(false)
}

/// A CSS artboard's BoxPaint leaves its own clip, so a child outside the
/// painted rectangle stays semantically visible.
#[test]
fn css_artboard_box_paint_keeps_semantic_visibility() {
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![]),
            paints: vec![Paint::Fill(0xff11_2233)],
            box_paint: Some(paint_points(100.0, 50.0)),
            ..ArtboardSpec::default()
        },
        Some(css("Corner", [width(5.0), height(5.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    use crate::source::semantic::semantic_provider::rendered_geometry_bounds;
    let bounds = rendered_geometry_bounds(Some(&f.named("Corner")));
    assert!(!bounds.is_empty_or_nan(), "{bounds:?}");
}

/// A CSS artboard's visible geometry is its painted rectangle, and it is
/// selected there only.
#[test]
fn css_artboard_box_paint_is_its_visible_geometry() {
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![]),
            paints: vec![Paint::Fill(0xff11_2233)],
            box_paint: Some(paint_points(100.0, 50.0)),
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let mut host = f.host();
    let visible = host.visible_geometry_with_bounds();
    assert_eq!(f.geometry_names(&visible), ["Artboard"]);
    let bounds = visible[0].bounds;
    assert_eq!(
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y),
        (150.0, 125.0, 250.0, 175.0)
    );
    // Selected where it paints, not elsewhere in its bounds.
    let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(200.0, 150.0));
    assert_eq!(f.geometry_names(&hits), ["Artboard"]);
    let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(390.0, 290.0));
    assert!(f.geometry_names(&hits).is_empty());
}

/// Painted boxes clipped away by a CSS artboard's padding-box clip, or
/// lying in a rounded clip's cut corner, are not visible geometry.
#[test]
fn css_painted_boxes_follow_rendered_clips() {
    const MARGIN_LEFT: u16 = 61049;
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![
                U(OVERFLOW_X, 2),
                U(OVERFLOW_Y, 2),
                F(BORDER_TOP, 10.0),
                F(BORDER_LEFT, 10.0),
            ]),
            ..ArtboardSpec::default()
        },
        Some(B {
            children: vec![
                painted(
                    "Hidden",
                    0xff00_0001,
                    [
                        [F(MARGIN_TOP, -10.0), F(MARGIN_LEFT, -10.0)].as_slice(),
                        &width(5.0),
                        &height(5.0),
                    ]
                    .concat(),
                ),
                css(
                    "Round",
                    [
                        [U(OVERFLOW_X, 2), U(OVERFLOW_Y, 2)].as_slice(),
                        &[61079, 61080, 61081, 61082].map(|key| F(key, 50.0)),
                        &width(100.0),
                        &height(100.0),
                    ]
                    .concat(),
                    vec![painted("Cut", 0xff00_0002, sized_to(10.0, 10.0))],
                ),
            ],
            ..css("Column", [U(DISPLAY, 1), U(61006, 2)], vec![])
        }),
    )
    .expect("the scene imports");
    assert_eq!(f.bounds("Cut")[..2], [0.0, 0.0]);
    let visible = f.host().visible_geometry_with_bounds();
    let names = f.geometry_names(&visible);
    assert!(
        !names.contains(&"Hidden".to_owned()) && !names.contains(&"Cut".to_owned()),
        "{names:?}"
    );
}

fn sized_to(width_value: f32, height_value: f32) -> Vec<P> {
    [width(width_value), height(height_value)].concat()
}

/// A box painted only by a BoxBorder is visible geometry whatever its
/// Stroke's thickness, as it draws; on a CSS artboard too.
#[test]
fn css_box_border_alone_is_catalogued_geometry() {
    let zero_thickness = |f: &Scene, owner: CoreHandle| {
        let stroke = owner
            .with(|object| {
                let paints = match object.as_artboard() {
                    Some(artboard) => artboard.base.base.shape_paint_container().shape_paints(),
                    None => object.as_shape_paint_container()?.shape_paints(),
                };
                paints.first().cloned()
            })
            .flatten()
            .unwrap();
        assert!(CoreRegistry::set_double_handle(
            &stroke,
            StrokeBase::THICKNESS_PROPERTY_KEY.into(),
            0.0
        ));
        f.advance(2);
    };
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Border(0xff44_5566, 15)],
            ..css("Edge", border_box(&[]), vec![])
        },
    );
    zero_thickness(&f, f.named("Edge"));
    let visible = f.host().visible_geometry_with_bounds();
    assert_eq!(f.geometry_names(&visible), ["Edge"]);
    let f = try_scene_spec(
        (400.0, 300.0),
        ArtboardSpec {
            css: Some(vec![F(BORDER_TOP, 4.0)]),
            paints: vec![Paint::Border(0xff44_5566, 15)],
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    zero_thickness(&f, f.object("Artboard"));
    let visible = f.host().visible_geometry_with_bounds();
    assert_eq!(f.geometry_names(&visible), ["Artboard"]);
}

/// The host's scrolled layout bounds move content as an RTL viewport draws
/// it: a drag to the right moves it right.
#[test]
fn css_rtl_scroll_projection_follows_the_drawn_content() {
    const RTL: u64 = 2;
    const HORIZONTAL: u64 = 0;
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [
                [U(DISPLAY, 1), U(DIRECTION, RTL), U(OVERFLOW_X, 3)].as_slice(),
                &width(100.0),
                &height(20.0),
            ]
            .concat(),
            vec![B {
                scroll: Some(HORIZONTAL),
                ..css(
                    "Content",
                    [
                        [F(FLEX_SHRINK, 0.0)].as_slice(),
                        &width(300.0),
                        &height(20.0),
                    ]
                    .concat(),
                    vec![css("Item", [width(300.0), height(20.0)].concat(), vec![])],
                )
            }],
        ),
    );
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    let scroll = f
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
        .into_iter()
        .next()
        .expect("a scroll constraint");
    scroll
        .with_downcast_mut::<ScrollConstraint, _>(|scroll| {
            scroll.drag_view(
                crate::source::math::vec2d::Vec2D::new(50.0, 0.0),
                0.0,
                false,
            )
        })
        .unwrap();
    f.advance(2);
    assert_eq!(f.world_origin("Item").0, -150.0);
    let projected = f
        .host()
        .scrolled_layout_bounds(f.local_id("Item"))
        .expect("scrolled bounds");
    // Item sits at its content box's origin, moved right by the drag.
    assert_eq!(projected.x, 50.0);
}

/// A grouped shape keeps its geometry beside a sibling with no natural
/// size of its own: an empty Text in its group adds nothing.
#[test]
fn css_group_keeps_its_shape_beside_an_unmeasured_sibling() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Row",
            [[U(DISPLAY, 1)].as_slice(), &width(200.0), &height(40.0)].concat(),
            vec![B {
                content: vec![Rect {
                    name: "Icon",
                    size: (80.0, 30.0),
                    grouped: true,
                    path_scale_x: 1.0,
                }],
                text_in_groups: true,
                ..css("Hug", [], vec![])
            }],
        ),
    );
    assert_eq!(f.bounds("Hug"), [0.0, 0.0, 80.0, 40.0]);
}

/// Indexed scrolling in an RTL CSS viewport counts from its start, the
/// right edge: index 2 brings the third item, the leftmost, to the
/// viewport.
#[test]
fn css_rtl_index_scrolling_counts_from_the_start() {
    const RTL: u64 = 2;
    const HORIZONTAL: u64 = 0;
    let item = |name: &'static str| {
        css(
            name,
            [
                [F(FLEX_SHRINK, 0.0)].as_slice(),
                &width(100.0),
                &height(20.0),
            ]
            .concat(),
            vec![],
        )
    };
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [
                [U(DISPLAY, 1), U(DIRECTION, RTL), U(OVERFLOW_X, 3)].as_slice(),
                &width(100.0),
                &height(20.0),
            ]
            .concat(),
            vec![B {
                scroll: Some(HORIZONTAL),
                ..css(
                    "Content",
                    [
                        [U(DISPLAY, 1), F(FLEX_SHRINK, 0.0)].as_slice(),
                        &width(300.0),
                        &height(20.0),
                    ]
                    .concat(),
                    vec![item("A"), item("B"), item("C")],
                )
            }],
        ),
    );
    assert_eq!(f.world_origin("A").0, 0.0);
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    let scroll = f
        .artboard
        .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
        .into_iter()
        .next()
        .expect("a scroll constraint");
    scroll
        .with_downcast_mut::<ScrollConstraint, _>(|scroll| scroll.set_scroll_index(2.0))
        .unwrap();
    f.advance(2);
    assert_eq!(f.world_origin("C").0, 0.0);
    let index = scroll
        .with_downcast::<ScrollConstraint, _>(|scroll| (scroll.offset_x(), scroll.scroll_index()))
        .unwrap();
    assert_eq!(index, (-200.0, 2.0));
}

fn css_axis_scroll(f: &Scene) -> CoreHandle {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    f.artboard
        .with_artboard(|a| a.find_all_handles::<ScrollConstraint>())[0]
        .clone()
}

fn css_axis_row(
    rtl: bool,
    viewport: f32,
    widths: &[f32],
    gap: f32,
    pad: f32,
    viewport_padding: bool,
    snap: bool,
    bind: Option<(u16, u64)>,
) -> Scene {
    let names = ["A", "B", "C", "D"];
    let items = widths
        .iter()
        .enumerate()
        .map(|(i, w)| {
            css(
                names[i],
                [[F(FLEX_SHRINK, 0.0)].as_slice(), &width(*w), &height(20.0)].concat(),
                vec![],
            )
        })
        .collect();
    let padding = [F(PADDING_LEFT, pad), F(PADDING_RIGHT, pad)];
    scene(
        (1000.0, 300.0),
        css(
            "Viewport",
            [
                vec![
                    U(DISPLAY, 1),
                    U(DIRECTION, if rtl { 2 } else { 1 }),
                    U(OVERFLOW_X, 3),
                ],
                width(viewport).to_vec(),
                height(20.0).to_vec(),
                if viewport_padding {
                    padding.to_vec()
                } else {
                    vec![]
                },
            ]
            .concat(),
            vec![B {
                scroll: Some(0),
                scroll_properties: vec![U(724, u64::from(snap))],
                scroll_bind: bind,
                ..css(
                    "Content",
                    [
                        vec![
                            U(DISPLAY, 1),
                            U(WIDTH_KIND, 3),
                            F(FLEX_SHRINK, 0.0),
                            F(COLUMN_GAP, gap),
                        ],
                        height(20.0).to_vec(),
                        if viewport_padding {
                            vec![]
                        } else {
                            padding.to_vec()
                        },
                    ]
                    .concat(),
                    items,
                )
            }],
        ),
    )
}

#[test]
fn css_scroll_index_round_trips_normal_and_reversed_axes() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for horizontal in [false, true] {
        for rtl in [false, true] {
            for reverse in [false, true] {
                let f = scene(
                    (400.0, 400.0),
                    css(
                        "Viewport",
                        [
                            vec![
                                U(DISPLAY, 1),
                                U(DIRECTION, if rtl { 2 } else { 1 }),
                                U(if horizontal { OVERFLOW_X } else { OVERFLOW_Y }, 3),
                            ],
                            width(100.0).to_vec(),
                            height(100.0).to_vec(),
                        ]
                        .concat(),
                        vec![B {
                            scroll: Some(if horizontal { 0 } else { 1 }),
                            ..css(
                                "Content",
                                [
                                    vec![
                                        U(DISPLAY, 1),
                                        F(FLEX_SHRINK, 0.0),
                                        U(
                                            61006,
                                            if horizontal {
                                                u64::from(reverse)
                                            } else {
                                                2 + u64::from(reverse)
                                            },
                                        ),
                                    ],
                                    width(if horizontal { 300.0 } else { 100.0 }).to_vec(),
                                    height(if horizontal { 100.0 } else { 300.0 }).to_vec(),
                                ]
                                .concat(),
                                ["A", "B", "C"]
                                    .into_iter()
                                    .map(|name| {
                                        css(
                                            name,
                                            [
                                                vec![F(FLEX_SHRINK, 0.0)],
                                                width(100.0).to_vec(),
                                                height(100.0).to_vec(),
                                            ]
                                            .concat(),
                                            vec![],
                                        )
                                    })
                                    .collect(),
                            )
                        }],
                    ),
                );
                let scroll = css_axis_scroll(&f);
                for index in [0.0, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0] {
                    scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                        scroll.set_scroll_index(index)
                    });
                    f.advance(2);
                    let expected = -100.0 * if reverse { 2.0 - index } else { index };
                    assert_eq!(
                        scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                            if horizontal {
                                scroll.offset_x()
                            } else {
                                scroll.offset_y()
                            },
                            scroll.scroll_index(),
                        )),
                        Some((expected, index)),
                        "horizontal={horizontal}, rtl={rtl}, reverse={reverse}, index={index}"
                    );
                    if index.fract() == 0.0 {
                        let origin = f.world_origin(["A", "B", "C"][index as usize]);
                        assert_eq!(if horizontal { origin.0 } else { origin.1 }, 0.0);
                    }
                }
            }
        }
    }
}

#[test]
fn css_scroll_index_round_trips_every_collapsed_item_gap_pattern() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for rtl in [false, true] {
        for collapsed in 0u32..16 {
            let f = css_axis_row(rtl, 100.0, &[100.0; 4], 0.0, 0.0, false, false, None);
            for (index, name) in ["A", "B", "C", "D"].into_iter().enumerate() {
                if collapsed & (1 << index) != 0 {
                    assert!(CoreRegistry::set_uint_handle(
                        &f.record(name),
                        DISPLAY.into(),
                        3
                    ));
                }
            }
            f.advance(3);
            let scroll = css_axis_scroll(&f);
            let visible = (0..4)
                .filter(|index| collapsed & (1 << index) == 0)
                .collect::<Vec<_>>();
            if visible.is_empty() {
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index),
                    Some(0.0)
                );
                scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_scroll_index(0.0));
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index),
                    Some(0.0)
                );
                continue;
            }
            let mut samples = vec![
                (50.0, visible[0] as f32),
                (
                    -100.0 * visible.len() as f32,
                    *visible.last().unwrap() as f32,
                ),
            ];
            for (slot, index) in visible.iter().enumerate() {
                samples.push((-100.0 * slot as f32, *index as f32));
                if let Some(next) = visible.get(slot + 1) {
                    for fraction in [0.25, 0.5, 0.75] {
                        samples.push((
                            -100.0 * (slot as f32 + fraction),
                            *index as f32 + (*next - *index) as f32 * fraction,
                        ));
                    }
                }
            }
            for (offset, expected) in samples {
                scroll.with_downcast_mut::<ScrollConstraint, _>(|s| {
                    s.set_authored_scroll_offset_x(offset)
                });
                let read = scroll
                    .with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index)
                    .unwrap();
                assert_eq!(
                    read, expected,
                    "rtl={rtl}, collapsed={collapsed:04b}, offset={offset}"
                );
                scroll.with_downcast_mut::<ScrollConstraint, _>(|s| {
                    s.set_authored_scroll_offset_x(if read == visible[0] as f32 {
                        -100.0 * visible.len() as f32
                    } else {
                        0.0
                    });
                });
                assert!(CoreRegistry::set_double_handle(&scroll, 763, read));
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index),
                    Some(read),
                    "rtl={rtl}, collapsed={collapsed:04b}, index={read}"
                );
                // Exercise the setter even when the registry skips an equal write.
                scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_scroll_index(read));
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index),
                    Some(read),
                    "rtl={rtl}, collapsed={collapsed:04b}, index={read}"
                );
            }
        }
    }
}

fn css_padded_auto_thumb_scene(
    horizontal: bool,
    rtl: bool,
    border_box: bool,
    percent_padding: bool,
    with_content: bool,
) -> Scene {
    let thumb_padding = if percent_padding {
        vec![
            F(PADDING_LEFT + 1, 5.0),
            F(PADDING_RIGHT + 1, 5.0),
            F(PADDING_TOP + 1, 5.0),
            F(PADDING_BOTTOM + 1, 5.0),
        ]
    } else {
        vec![
            F(PADDING_LEFT, 10.0),
            F(PADDING_RIGHT, 10.0),
            F(PADDING_TOP, 4.0),
            F(PADDING_BOTTOM, 6.0),
        ]
    };
    let f = scene(
        (1000.0, 1000.0),
        css(
            "Viewport",
            [
                vec![
                    U(DISPLAY, 1),
                    U(POSITION, 1),
                    U(DIRECTION, if rtl { 2 } else { 1 }),
                    U(if horizontal { OVERFLOW_X } else { OVERFLOW_Y }, 3),
                ],
                width(300.0).to_vec(),
                height(300.0).to_vec(),
            ]
            .concat(),
            vec![
                B {
                    scroll: Some(if horizontal { 0 } else { 1 }),
                    ..css(
                        "Content",
                        [
                            vec![F(FLEX_SHRINK, 0.0)],
                            width(900.0).to_vec(),
                            height(900.0).to_vec(),
                        ]
                        .concat(),
                        vec![],
                    )
                },
                css(
                    "Track",
                    [
                        vec![
                            U(POSITION, 2),
                            U(INSET_LEFT_KIND, 1),
                            F(INSET_LEFT, 0.0),
                            U(INSET_TOP_KIND, 1),
                            F(INSET_TOP, 0.0),
                        ],
                        width(if horizontal { 300.0 } else { 60.0 }).to_vec(),
                        height(if horizontal { 60.0 } else { 300.0 }).to_vec(),
                    ]
                    .concat(),
                    vec![B {
                        scroll_bar: Some(if horizontal { 0 } else { 1 }),
                        scroll_bar_auto_size: true,
                        content: if with_content {
                            vec![Rect {
                                name: "Thumb content",
                                size: (10.0, 10.0),
                                grouped: false,
                                path_scale_x: 1.0,
                            }]
                        } else {
                            vec![]
                        },
                        ..css(
                            "Thumb",
                            [
                                vec![
                                    U(BOX_SIZING, u64::from(border_box)),
                                    F(BORDER_LEFT, 3.0),
                                    F(BORDER_RIGHT, 7.0),
                                    F(BORDER_TOP, 2.0),
                                    F(BORDER_BOTTOM, 3.0),
                                ],
                                thumb_padding,
                                width(80.0).to_vec(),
                                height(80.0).to_vec(),
                            ]
                            .concat(),
                            vec![],
                        )
                    }],
                ),
            ],
        ),
    );
    f.advance(4);
    f
}

#[test]
fn css_auto_thumb_forces_only_the_scrolling_axis_outer_size() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for horizontal in [false, true] {
        for rtl in [false, true] {
            for border_box in [false, true] {
                for with_content in [false, true] {
                    let f = css_padded_auto_thumb_scene(
                        horizontal,
                        rtl,
                        border_box,
                        false,
                        with_content,
                    );
                    let bounds = f.bounds("Thumb");
                    assert_eq!(
                        if horizontal { bounds[2] } else { bounds[3] },
                        100.0,
                        "horizontal={horizontal}, rtl={rtl}, border_box={border_box}"
                    );
                    let authored_other = if border_box {
                        80.0
                    } else if horizontal {
                        95.0
                    } else {
                        110.0
                    };
                    assert_eq!(
                        if horizontal { bounds[3] } else { bounds[2] },
                        authored_other
                    );
                    let origin = f.world_origin("Thumb");
                    assert_eq!(
                        if horizontal { origin.0 } else { origin.1 },
                        if horizontal && rtl { 200.0 } else { 0.0 }
                    );
                    css_axis_scroll(&f).with_downcast_mut::<ScrollConstraint, _>(|s| {
                        if horizontal {
                            s.set_authored_scroll_offset_x(-300.0);
                        } else {
                            s.set_authored_scroll_offset_y(-300.0);
                        }
                    });
                    f.advance(4);
                    let origin = f.world_origin("Thumb");
                    assert_eq!(if horizontal { origin.0 } else { origin.1 }, 100.0);
                    let bounds = f.bounds("Thumb");
                    assert_eq!(if horizontal { bounds[2] } else { bounds[3] }, 100.0);
                }
            }
        }
    }
}

#[test]
fn css_auto_thumb_outer_size_resolves_percentage_padding_after_track_resize() {
    for horizontal in [false, true] {
        for rtl in [false, true] {
            for border_box in [false, true] {
                for with_content in [false, true] {
                    let f = css_padded_auto_thumb_scene(
                        horizontal,
                        rtl,
                        border_box,
                        true,
                        with_content,
                    );
                    let bounds = f.bounds("Thumb");
                    assert_eq!(if horizontal { bounds[2] } else { bounds[3] }, 100.0);
                    assert!(CoreRegistry::set_double_handle(
                        &f.record("Track"),
                        WIDTH.into(),
                        if horizontal { 450.0 } else { 90.0 }
                    ));
                    f.advance(6);
                    let expected = if horizontal { 150.0 } else { 100.0 };
                    let bounds = f.bounds("Thumb");
                    assert_eq!(
                        if horizontal { bounds[2] } else { bounds[3] },
                        expected,
                        "horizontal={horizontal}, rtl={rtl}, border_box={border_box}"
                    );
                    let origin = f.world_origin("Thumb");
                    assert_eq!(
                        if horizontal { origin.0 } else { origin.1 },
                        if horizontal && rtl { 300.0 } else { 0.0 }
                    );
                }
            }
        }
    }
}

#[test]
fn css_auto_thumb_border_box_stays_forced_during_leaf_interpolation() {
    for horizontal in [false, true] {
        for rtl in [false, true] {
            for border_box in [false, true] {
                for borrowed in [false, true] {
                    let f = css_padded_auto_thumb_scene(horizontal, rtl, border_box, true, true);
                    let thumb = f.named("Thumb");
                    // Stretch the anonymous item along the scrolling axis.
                    assert!(CoreRegistry::set_uint_handle(
                        &f.record("Thumb"),
                        DISPLAY.into(),
                        1
                    ));
                    assert!(CoreRegistry::set_uint_handle(
                        &f.record("Thumb"),
                        61006,
                        if horizontal { 2 } else { 0 }
                    ));
                    f.advance(4);
                    enable_leaf_layout_interpolation(&thumb);
                    assert!(CoreRegistry::set_double_handle(
                        &f.record("Track"),
                        if horizontal {
                            WIDTH.into()
                        } else {
                            HEIGHT.into()
                        },
                        450.0
                    ));
                    if !horizontal {
                        assert!(CoreRegistry::set_double_handle(
                            &f.record("Track"),
                            WIDTH.into(),
                            90.0
                        ));
                    }
                    f.advance(4);
                    for elapsed in [0.0, 0.25, 0.0, 0.25, 0.0, 0.5, 0.0] {
                        advance_leaf_layout(&thumb, elapsed, borrowed);
                        let bounds = f.bounds("Thumb");
                        assert_eq!(
                            if horizontal { bounds[2] } else { bounds[3] },
                            150.0,
                            "horizontal={horizontal}, rtl={rtl}, border_box={border_box}, borrowed={borrowed}"
                        );
                        let forced = thumb
                            .with_downcast::<LayoutComponent, _>(|thumb| {
                                if horizontal {
                                    thumb.forced_width()
                                } else {
                                    thumb.forced_height()
                                }
                            })
                            .unwrap();
                        assert_eq!(forced, 150.0);
                    }
                }
            }
        }
    }
}

#[test]
fn css_scroll_bar_uses_track_content_box_for_thumb_size_and_input() {
    use crate::source::constraints::scrolling::{
        scroll_bar_constraint::ScrollBarConstraint, scroll_constraint::ScrollConstraint,
    };
    use crate::source::math::vec2d::Vec2D;
    for horizontal in [false, true] {
        for rtl in [false, true] {
            let f = css_padded_auto_thumb_scene(horizontal, rtl, false, false, true);
            let record = f.record("Track");
            for (key, value) in [
                (PADDING_LEFT, 10.0),
                (PADDING_RIGHT, 14.0),
                (PADDING_TOP, 4.0),
                (PADDING_BOTTOM, 6.0),
                (BORDER_LEFT, 3.0),
                (BORDER_RIGHT, 9.0),
                (BORDER_TOP, 2.0),
                (BORDER_BOTTOM, 3.0),
            ] {
                assert!(CoreRegistry::set_double_handle(&record, key.into(), value));
            }
            f.advance(6);
            let bar = f
                .artboard
                .with_artboard(|a| a.find_all_handles::<ScrollBarConstraint>())[0]
                .clone();
            let scroll = css_axis_scroll(&f);
            let bounds = f.bounds("Thumb");
            assert_eq!(if horizontal { bounds[2] } else { bounds[3] }, 100.0);
            let start = if horizontal { 13.0 } else { 6.0 };
            let origin = f.world_origin("Thumb");
            assert_eq!(
                if horizontal { origin.0 } else { origin.1 },
                start + if horizontal && rtl { 200.0 } else { 0.0 }
            );
            let click = start + if horizontal && rtl { 225.0 } else { 75.0 };
            ScrollBarConstraint::hit_track(
                &bar,
                if horizontal {
                    Vec2D::new(click, 30.0)
                } else {
                    Vec2D::new(30.0, click)
                },
            );
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(|scroll| if horizontal {
                    scroll.offset_x()
                } else {
                    scroll.offset_y()
                }),
                Some(-225.0)
            );
            ScrollBarConstraint::drag_thumb(
                &bar,
                if horizontal {
                    Vec2D::new(if rtl { -25.0 } else { 25.0 }, 0.0)
                } else {
                    Vec2D::new(0.0, 25.0)
                },
                0.1,
            );
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(|scroll| if horizontal {
                    scroll.offset_x()
                } else {
                    scroll.offset_y()
                }),
                Some(-300.0)
            );
            f.advance(4);
            let origin = f.world_origin("Thumb");
            assert_eq!(if horizontal { origin.0 } else { origin.1 }, start + 100.0);
            assert!(CoreRegistry::set_bool_handle(&bar, 734, false));
            assert_eq!(
                bar.with_downcast::<ScrollBarConstraint, _>(|bar| {
                    if horizontal {
                        bar.computed_thumb_width()
                    } else {
                        bar.computed_thumb_height()
                    }
                }),
                Some(100.0)
            );
        }
    }
}

#[test]
fn css_fractional_index_uses_solved_item_steps_and_css_gaps() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for rtl in [false, true] {
        let f = css_axis_row(rtl, 60.0, &[60.0; 4], 20.0, 0.0, false, false, None);
        let scroll = css_axis_scroll(&f);
        scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_scroll_index(0.5));
        f.advance(2);
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index())),
            Some((-40.0, 0.5))
        );
    }
}

#[test]
fn css_scroll_index_interpolates_an_oversized_single_item() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for rtl in [false, true] {
        for pad in [0.0, 24.0] {
            for viewport_padding in [false, true] {
                let f = css_axis_row(
                    rtl,
                    100.0,
                    &[300.0],
                    0.0,
                    pad,
                    viewport_padding,
                    false,
                    None,
                );
                let scroll = css_axis_scroll(&f);
                let expected = -(pad + 150.0);
                assert!(CoreRegistry::set_double_handle(&scroll, 763, 0.5));
                assert_eq!(
                    scroll
                        .with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index())),
                    Some((expected, 0.5)),
                    "rtl={rtl}, padding={pad}, viewport_padding={viewport_padding}"
                );
                // Read from an authored offset independently of the index setter.
                scroll.with_downcast_mut::<ScrollConstraint, _>(|s| {
                    s.set_authored_scroll_offset_x(expected)
                });
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index),
                    Some(0.5)
                );
                let end = scroll
                    .with_downcast::<ScrollConstraint, _>(ScrollConstraint::max_offset_x)
                    .unwrap();
                if pad == 0.0 {
                    assert_eq!(end, -200.0);
                }
                // Upstream targets the far end at and beyond the item count.
                for index in [1.0, 2.0] {
                    scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_scroll_index(index));
                    let (offset, read) = scroll
                        .with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index()))
                        .unwrap();
                    assert_eq!(offset, end);
                    assert!((read - (-end - pad) / 300.0).abs() < 0.00001);
                }
            }
        }
    }
}

#[test]
fn css_scroll_index_interpolates_through_an_oversized_final_item_to_the_end() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for rtl in [false, true] {
        let f = css_axis_row(
            rtl,
            100.0,
            &[100.0, 100.0, 300.0],
            12.0,
            0.0,
            false,
            false,
            None,
        );
        let scroll = css_axis_scroll(&f);
        for (index, offset) in [
            (0.0, 0.0),
            (1.0, -112.0),
            (2.0, -224.0),
            (2.25, -302.0),
            (2.5, -380.0),
        ] {
            assert!(CoreRegistry::set_double_handle(&scroll, 763, index));
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index())),
                Some((offset, index)),
                "rtl={rtl}, index={index}"
            );
            scroll.with_downcast_mut::<ScrollConstraint, _>(|s| {
                s.set_authored_scroll_offset_x(offset)
            });
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index),
                Some(index)
            );
        }
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::max_offset_x),
            Some(-424.0)
        );
        let end_index = 2.0 + 200.0 / 312.0;
        scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_authored_scroll_offset_x(-424.0));
        let read = scroll
            .with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index)
            .unwrap();
        assert!((read - end_index).abs() < 0.00001);
        for index in [end_index, 2.75, 3.0, 4.0] {
            scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_scroll_index(index));
            let (offset, read) = scroll
                .with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index()))
                .unwrap();
            assert!((offset + 424.0).abs() < 0.0001);
            assert!((read - end_index).abs() < 0.00001);
        }
    }
}

#[test]
fn css_rtl_snap_uses_logical_starts_for_mixed_width_items() {
    use crate::source::constraints::scrolling::{
        elastic_scroll_physics::ElasticScrollPhysics, scroll_constraint::ScrollConstraint,
    };
    for rtl in [false, true] {
        let f = css_axis_row(
            rtl,
            60.0,
            &[60.0, 140.0, 60.0, 60.0],
            20.0,
            0.0,
            false,
            true,
            None,
        );
        let scroll = css_axis_scroll(&f);
        let physics = scroll
            .insert_sibling(ElasticScrollPhysics::default())
            .unwrap();
        scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_physics(physics));
        for (release, expected) in [(-50.0, -80.0), (-200.0, -240.0)] {
            scroll.with_downcast_mut::<ScrollConstraint, _>(|s| {
                s.stop_physics();
                s.set_authored_scroll_offset_x(release);
                s.prime_physics();
                s.run_physics();
            });
            f.advance(240);
            let offset = scroll
                .with_downcast::<ScrollConstraint, _>(ScrollConstraint::offset_x)
                .unwrap();
            assert!((offset - expected).abs() < 0.01, "rtl={rtl}: {offset}");
        }
    }
}

#[test]
fn css_mandatory_snap_settles_at_load_from_the_scroll_origin() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for rtl in [false, true] {
        for viewport_padding in [false, true] {
            let f = css_axis_row(
                rtl,
                100.0,
                &[100.0; 3],
                0.0,
                24.0,
                viewport_padding,
                true,
                None,
            );
            assert_eq!(
                css_axis_scroll(&f)
                    .with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index())),
                Some((-24.0, 0.0))
            );
        }
    }
}

#[test]
fn css_scroll_index_interpolates_clamped_snap_positions_and_writes_back() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    use crate::source::data_bind::data_bind::TWO_WAY;
    use crate::source::generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase;
    for rtl in [false, true] {
        let f = css_axis_row(
            rtl,
            402.0,
            &[300.0; 3],
            12.0,
            0.0,
            false,
            true,
            Some((763, u64::from(TWO_WAY))),
        );
        let scroll = css_axis_scroll(&f);
        for (offset, index) in [(0.0, 0.0), (-312.0, 1.0), (-522.0, 2.0), (-417.0, 1.5)] {
            scroll.with_downcast_mut::<ScrollConstraint, _>(|s| {
                s.set_authored_scroll_offset_x(offset)
            });
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index),
                Some(index)
            );
        }
        assert!(CoreRegistry::set_double_handle(&scroll, 763, 2.0));
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index())),
            Some((-522.0, 2.0))
        );
        scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_authored_scroll_offset_x(-417.0));
        let number = f.bind_number();
        // A new two-way binding first publishes the current target value.
        f.advance(3);
        assert_eq!(
            CoreRegistry::get_double_handle(
                &number,
                ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into()
            ),
            Some(1.5)
        );
        CoreRegistry::set_double_handle(
            &number,
            ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
            2.0,
        );
        f.advance(3);
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index())),
            Some((-522.0, 2.0))
        );
        assert_eq!(
            CoreRegistry::get_double_handle(
                &number,
                ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into()
            ),
            Some(2.0)
        );
    }
}

#[test]
fn css_scroll_offset_keyframes_and_native_properties_keep_authored_coordinates() {
    use crate::source::{
        animation::keyframe_double::KeyFrameDouble,
        constraints::scrolling::scroll_constraint::ScrollConstraint, data_bind::data_bind::TWO_WAY,
        generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
    };
    for horizontal in [false, true] {
        for rtl in [false, true] {
            let key = if horizontal { 759 } else { 760 };
            let f = scene(
                (1000.0, 1000.0),
                css(
                    "Viewport",
                    [
                        vec![
                            U(DISPLAY, 1),
                            U(DIRECTION, if rtl { 2 } else { 1 }),
                            U(if horizontal { OVERFLOW_X } else { OVERFLOW_Y }, 3),
                        ],
                        width(100.0).to_vec(),
                        height(100.0).to_vec(),
                    ]
                    .concat(),
                    vec![B {
                        scroll: Some(if horizontal { 0 } else { 1 }),
                        scroll_properties: vec![F(key, -25.0)],
                        scroll_keys: vec![(key, vec![(0, 0.0), (60, -100.0)])],
                        scroll_bind: Some((key, u64::from(TWO_WAY))),
                        ..css(
                            "Content",
                            [
                                vec![
                                    U(DISPLAY, if horizontal { 1 } else { 0 }),
                                    F(FLEX_SHRINK, 0.0),
                                ],
                                width(if horizontal { 1000.0 } else { 100.0 }).to_vec(),
                                height(if horizontal { 100.0 } else { 1000.0 }).to_vec(),
                            ]
                            .concat(),
                            ["First", "Second", "Tail"]
                                .into_iter()
                                .enumerate()
                                .map(|(i, name)| {
                                    css(
                                        name,
                                        [
                                            vec![F(FLEX_SHRINK, 0.0)],
                                            width(if horizontal && i == 2 { 800.0 } else { 100.0 })
                                                .to_vec(),
                                            height(if !horizontal && i == 2 {
                                                800.0
                                            } else {
                                                100.0
                                            })
                                            .to_vec(),
                                        ]
                                        .concat(),
                                        vec![],
                                    )
                                })
                                .collect(),
                        )
                    }],
                ),
            );
            let scroll = css_axis_scroll(&f);
            assert_eq!(
                CoreRegistry::get_double_handle(&scroll, key.into()),
                Some(-25.0)
            );
            let number = f.bind_number();
            f.advance(3);
            assert_eq!(
                CoreRegistry::get_double_handle(
                    &number,
                    ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into()
                ),
                Some(-25.0)
            );
            animate(&f, 1.0);
            assert_eq!(
                CoreRegistry::get_double_handle(&scroll, key.into()),
                Some(-100.0)
            );
            let origin = f.world_origin("Second");
            assert_eq!(if horizontal { origin.0 } else { origin.1 }, 0.0);
            assert_eq!(
                CoreRegistry::get_double_handle(
                    &number,
                    ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into()
                ),
                Some(-100.0)
            );
            let mut frame = KeyFrameDouble::default();
            frame.base.set_value_value(-200.0);
            assert!(frame.apply(&scroll, key.into(), 0.5, None));
            assert_eq!(
                CoreRegistry::get_double_handle(&scroll, key.into()),
                Some(-150.0)
            );
            // Native and script property access share the raw registry path.
            assert!(CoreRegistry::set_double_handle(&scroll, key.into(), -100.0));
            assert_eq!(
                CoreRegistry::get_double_handle(&scroll, key.into()),
                Some(-100.0)
            );
            f.advance(3);
            for offset in [0.0, -100.0] {
                assert!(CoreRegistry::set_double_handle(
                    &number,
                    ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
                    offset
                ));
                f.advance(3);
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(|scroll| if horizontal {
                        scroll.offset_x()
                    } else {
                        scroll.offset_y()
                    }),
                    Some(offset)
                );
                assert_eq!(
                    CoreRegistry::get_double_handle(&scroll, key.into()),
                    Some(offset)
                );
                assert_eq!(
                    CoreRegistry::get_double_handle(
                        &number,
                        ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into()
                    ),
                    Some(offset)
                );
            }
            let origin = f.world_origin("Second");
            assert_eq!(if horizontal { origin.0 } else { origin.1 }, 0.0);
        }
    }
}

#[test]
fn css_focus_reveals_items_inside_the_bordered_padding_box() {
    use crate::source::{
        constraints::scrolling::scroll_constraint::ScrollConstraint, focus_data::FocusData,
    };
    for horizontal in [false, true] {
        for rtl in [false, true] {
            let f = scene(
                (400.0, 400.0),
                css(
                    "Viewport",
                    [
                        vec![
                            U(DISPLAY, 1),
                            U(BOX_SIZING, 1),
                            U(DIRECTION, if rtl { 2 } else { 1 }),
                            U(if horizontal { OVERFLOW_X } else { OVERFLOW_Y }, 3),
                            F(BORDER_LEFT, 10.0),
                            F(BORDER_RIGHT, 10.0),
                            F(BORDER_TOP, 10.0),
                            F(BORDER_BOTTOM, 10.0),
                        ],
                        width(100.0).to_vec(),
                        height(100.0).to_vec(),
                    ]
                    .concat(),
                    vec![B {
                        scroll: Some(if horizontal { 0 } else { 1 }),
                        ..css(
                            "Content",
                            [
                                vec![
                                    U(DISPLAY, if horizontal { 1 } else { 0 }),
                                    F(FLEX_SHRINK, 0.0),
                                ],
                                width(if horizontal { 400.0 } else { 80.0 }).to_vec(),
                                height(if horizontal { 80.0 } else { 400.0 }).to_vec(),
                            ]
                            .concat(),
                            vec![
                                css(
                                    "First",
                                    [
                                        vec![F(FLEX_SHRINK, 0.0)],
                                        width(80.0).to_vec(),
                                        height(80.0).to_vec(),
                                    ]
                                    .concat(),
                                    vec![],
                                ),
                                css(
                                    "Target",
                                    [
                                        vec![F(FLEX_SHRINK, 0.0)],
                                        width(80.0).to_vec(),
                                        height(80.0).to_vec(),
                                    ]
                                    .concat(),
                                    // Keep the outer reveal separate from TextInput's caret scroll.
                                    vec![B {
                                        text_input: true,
                                        text: "focus",
                                        ..css(
                                            "Field box",
                                            [width(80.0), height(80.0)].concat(),
                                            vec![],
                                        )
                                    }],
                                ),
                                css(
                                    "Tail",
                                    [
                                        vec![F(FLEX_SHRINK, 0.0)],
                                        width(if horizontal { 240.0 } else { 80.0 }).to_vec(),
                                        height(if horizontal { 80.0 } else { 240.0 }).to_vec(),
                                    ]
                                    .concat(),
                                    vec![],
                                ),
                            ],
                        )
                    }],
                ),
            );
            let scroll = css_axis_scroll(&f);
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                    scroll.viewport_width(),
                    scroll.viewport_height()
                )),
                Some((80.0, 80.0))
            );
            let before = f.world_origin("Target");
            let before = if horizontal { before.0 } else { before.1 };
            assert_eq!(before, if horizontal && rtl { -70.0 } else { 90.0 });
            let focus = f
                .artboard
                .with_artboard(|a| a.find_all_handles::<FocusData>())[0]
                .clone();
            let machine = f.machine();
            machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
            // Inspect the reveal before any field paint or layout update.
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(|scroll| if horizontal {
                    scroll.authored_scroll_offset_x()
                } else {
                    scroll.authored_scroll_offset_y()
                }),
                Some(-80.0),
                "horizontal={horizontal}, rtl={rtl}"
            );
            f.advance(3);
            assert!(f.field_is_focused());
            let after = f.world_origin("Target");
            let after = if horizontal { after.0 } else { after.1 };
            assert_eq!(after, 10.0, "horizontal={horizontal}, rtl={rtl}");
            assert_eq!((after - before).abs(), 80.0);
            let bounds = f.bounds("Target");
            assert_eq!(after + if horizontal { bounds[2] } else { bounds[3] }, 90.0);
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(|scroll| if horizontal {
                    scroll.offset_x()
                } else {
                    scroll.offset_y()
                }),
                Some(-80.0)
            );
            // Focus reveal uses the padding-box scroll range.
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(|scroll| if horizontal {
                    scroll.max_offset_x()
                } else {
                    scroll.max_offset_y()
                }),
                Some(-320.0)
            );
        }
    }
}

fn css_offset_binding_scene(
    horizontal: bool,
    rtl: bool,
    virtualize: bool,
    flags: u32,
    viewport: f32,
) -> Scene {
    scene(
        (800.0, 800.0),
        css(
            "Viewport",
            [
                vec![
                    U(DISPLAY, 1),
                    U(DIRECTION, if rtl { 2 } else { 1 }),
                    U(if horizontal { OVERFLOW_X } else { OVERFLOW_Y }, 3),
                ],
                width(viewport).to_vec(),
                height(viewport).to_vec(),
            ]
            .concat(),
            vec![B {
                scroll: Some(if horizontal { 0 } else { 1 }),
                scroll_properties: vec![
                    F(if horizontal { 759 } else { 760 }, -400.0),
                    U(850, u64::from(virtualize)),
                ],
                scroll_bind: Some((if horizontal { 759 } else { 760 }, u64::from(flags))),
                ..css(
                    "Content",
                    [
                        vec![
                            U(DISPLAY, 1),
                            U(61006, if horizontal { 0 } else { 2 }),
                            F(FLEX_SHRINK, 0.0),
                        ],
                        width(if horizontal { 600.0 } else { 100.0 }).to_vec(),
                        height(if horizontal { 100.0 } else { 600.0 }).to_vec(),
                    ]
                    .concat(),
                    vec![css(
                        "Item",
                        [
                            vec![F(FLEX_SHRINK, 0.0)],
                            width(if horizontal { 600.0 } else { 100.0 }).to_vec(),
                            height(if horizontal { 100.0 } else { 600.0 }).to_vec(),
                        ]
                        .concat(),
                        vec![],
                    )],
                )
            }],
        ),
    )
}

#[test]
fn css_viewport_loaded_virtualized_keeps_native_scroll_mode() {
    use crate::source::{
        constraints::scrolling::scroll_constraint::ScrollConstraint,
        data_bind::data_bind::{TO_SOURCE, TWO_WAY},
        generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
    };
    for horizontal in [false, true] {
        for rtl in [false, true] {
            for flags in [TO_SOURCE, TWO_WAY] {
                let f = css_offset_binding_scene(horizontal, rtl, true, flags, 100.0);
                let scroll = css_axis_scroll(&f);
                let number = f.bind_number();
                let read = || {
                    CoreRegistry::get_double_handle(
                        &number,
                        ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
                    )
                };
                f.advance(4);
                assert_eq!(read(), Some(-400.0));
                assert!(CoreRegistry::set_bool_handle(&scroll, 850, false));
                assert!(CoreRegistry::set_bool_handle(&scroll, 851, true));
                assert_eq!(CoreRegistry::get_bool_handle(&scroll, 850), Some(true));
                assert_eq!(CoreRegistry::get_bool_handle(&scroll, 851), Some(false));
                for name in ["Content", "Item"] {
                    assert!(CoreRegistry::set_double_handle(
                        &f.record(name),
                        if horizontal {
                            WIDTH.into()
                        } else {
                            HEIGHT.into()
                        },
                        200.0
                    ));
                }
                f.advance(4);
                let expected = -400.0;
                // Native RTL includes the content's negative x in viewport_width.
                let viewport_extent = if horizontal && rtl { 200.0 } else { 100.0 };
                let maximum = if horizontal && rtl { 0.0 } else { -100.0 };
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(|scroll| if horizontal {
                        (scroll.viewport_width(), scroll.content_width())
                    } else {
                        (scroll.viewport_height(), scroll.content_height())
                    }),
                    Some((viewport_extent, 200.0)),
                    "horizontal={horizontal}, rtl={rtl}, flags={flags}"
                );
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                        scroll.virtualize(),
                        if horizontal {
                            scroll.max_offset_x()
                        } else {
                            scroll.max_offset_y()
                        },
                        if horizontal {
                            scroll.authored_scroll_offset_x()
                        } else {
                            scroll.authored_scroll_offset_y()
                        },
                    )),
                    Some((true, maximum, expected)),
                    "horizontal={horizontal}, rtl={rtl}, flags={flags}"
                );
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(
                        ScrollConstraint::has_css_scroll_axis
                    ),
                    Some(false)
                );
                assert_eq!(read(), Some(expected));
                assert_eq!(
                    CoreRegistry::get_double_handle(&scroll, if horizontal { 759 } else { 760 }),
                    Some(-400.0)
                );
                let offset_key = if horizontal { 759 } else { 760 };
                let (target, key) = if flags == TWO_WAY {
                    (
                        &number,
                        ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
                    )
                } else {
                    (&scroll, offset_key)
                };
                assert!(CoreRegistry::set_double_handle(target, key, -100.0));
                f.advance(4);
                // Main's native setters retain the raw value even when range is zero.
                assert_eq!(
                    CoreRegistry::get_double_handle(&scroll, offset_key),
                    Some(-100.0)
                );
                assert_eq!(read(), Some(-100.0));
                assert_eq!(
                    scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                        scroll.virtualize(),
                        scroll.has_css_scroll_axis(),
                        if horizontal {
                            scroll.clamped_offset_x()
                        } else {
                            scroll.clamped_offset_y()
                        },
                    )),
                    Some((true, false, maximum)),
                    "horizontal={horizontal}, rtl={rtl}, flags={flags}"
                );
            }
        }
    }
}

#[test]
fn css_scroll_mode_ignores_late_virtualize_and_infinite_writes() {
    use crate::source::{
        constraints::scrolling::{
            elastic_scroll_physics::ElasticScrollPhysics, scroll_constraint::ScrollConstraint,
        },
        data_bind::data_bind::TO_SOURCE,
        generated::{
            core_registry::{CoreField, CoreRegistryObject},
            viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
        },
    };
    const ROW_GAP: u16 = 61075;
    for horizontal in [false, true] {
        for rtl in [false, true] {
            let f = scene(
                (500.0, 500.0),
                css(
                    "Viewport",
                    [
                        vec![
                            U(DISPLAY, 1),
                            U(DIRECTION, if rtl { 2 } else { 1 }),
                            U(if horizontal { OVERFLOW_X } else { OVERFLOW_Y }, 3),
                        ],
                        width(if horizontal { 60.0 } else { 20.0 }).to_vec(),
                        height(if horizontal { 20.0 } else { 60.0 }).to_vec(),
                    ]
                    .concat(),
                    vec![B {
                        scroll: Some(if horizontal { 0 } else { 1 }),
                        scroll_properties: vec![U(724, 1)],
                        scroll_bind: Some((
                            if horizontal { 759 } else { 760 },
                            u64::from(TO_SOURCE),
                        )),
                        ..css(
                            "Content",
                            [
                                vec![
                                    U(DISPLAY, 1),
                                    U(61006, if horizontal { 0 } else { 2 }),
                                    F(FLEX_SHRINK, 0.0),
                                    F(if horizontal { COLUMN_GAP } else { ROW_GAP }, 20.0),
                                ],
                                width(if horizontal { 380.0 } else { 20.0 }).to_vec(),
                                height(if horizontal { 20.0 } else { 380.0 }).to_vec(),
                            ]
                            .concat(),
                            [60.0, 140.0, 60.0, 60.0]
                                .into_iter()
                                .enumerate()
                                .map(|(index, size)| {
                                    css(
                                        ["A", "B", "C", "D"][index],
                                        [
                                            vec![F(FLEX_SHRINK, 0.0)],
                                            width(if horizontal { size } else { 20.0 }).to_vec(),
                                            height(if horizontal { 20.0 } else { size }).to_vec(),
                                        ]
                                        .concat(),
                                        vec![],
                                    )
                                })
                                .collect(),
                        )
                    }],
                ),
            );
            let scroll = css_axis_scroll(&f);
            let number = f.bind_number();
            f.advance(4);
            scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| scroll.set_scroll_index(1.5));
            f.advance(4);
            let expected = -160.0;
            for path in 0..3 {
                for (key, field) in [
                    (850, CoreField::ScrollConstraintVirtualize),
                    (851, CoreField::ScrollConstraintInfinite),
                ] {
                    for value in [true, false] {
                        match path {
                            0 => {
                                scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                                    CoreRegistry::set_bool(scroll, key, value);
                                });
                            }
                            1 => {
                                scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                                    CoreRegistryObject::set_bool(scroll, field, value);
                                });
                            }
                            _ => {
                                assert!(CoreRegistry::set_bool_handle(&scroll, key, value));
                            }
                        }
                        f.advance(2);
                        assert_eq!(CoreRegistry::get_bool_handle(&scroll, key), Some(false));
                        assert_eq!(
                            scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                                scroll.has_css_scroll_axis(),
                                if horizontal {
                                    scroll.offset_x()
                                } else {
                                    scroll.offset_y()
                                },
                                scroll.scroll_index(),
                            )),
                            Some((true, -160.0, 1.5))
                        );
                        assert_eq!(
                            CoreRegistry::get_double_handle(
                                &number,
                                ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into()
                            ),
                            Some(expected)
                        );
                    }
                }
            }
            scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| scroll.set_scroll_index(2.0));
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_index),
                Some(2.0)
            );
            let physics = scroll
                .insert_sibling(ElasticScrollPhysics::default())
                .unwrap();
            scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| scroll.set_physics(physics));
            for (release, snap, index) in [(-50.0, -80.0, 1.0), (-200.0, -240.0, 2.0)] {
                scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                    scroll.stop_physics();
                    if horizontal {
                        scroll.set_authored_scroll_offset_x(release);
                    } else {
                        scroll.set_authored_scroll_offset_y(release);
                    }
                    scroll.prime_physics();
                    scroll.run_physics();
                });
                f.advance(240);
                let (offset, read_index) = scroll
                    .with_downcast::<ScrollConstraint, _>(|scroll| {
                        (
                            if horizontal {
                                scroll.offset_x()
                            } else {
                                scroll.offset_y()
                            },
                            scroll.scroll_index(),
                        )
                    })
                    .unwrap();
                assert!((offset - snap).abs() < 0.01);
                assert!((read_index - index).abs() < 0.001);
                let bound = CoreRegistry::get_double_handle(
                    &number,
                    ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
                )
                .unwrap();
                assert!((bound - snap).abs() < 0.01);
            }
        }
    }
}

#[test]
fn translated_scroll_mode_keeps_upstream_flag_toggles() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    let f = translated_scroll_view(B::default());
    let scroll = css_axis_scroll(&f);
    assert!(CoreRegistry::set_double_handle(&scroll, 760, -40.0));
    for (virtualize, infinite) in [(true, false), (true, true), (true, false), (false, false)] {
        assert!(CoreRegistry::set_bool_handle(&scroll, 850, virtualize));
        assert!(CoreRegistry::set_bool_handle(&scroll, 851, infinite));
        f.advance(4);
        assert_eq!(
            CoreRegistry::get_bool_handle(&scroll, 850),
            Some(virtualize)
        );
        assert_eq!(CoreRegistry::get_bool_handle(&scroll, 851), Some(infinite));
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                scroll.has_css_scroll_axis(),
                scroll.virtual_layout().is_some(),
                scroll.authored_scroll_offset_y(),
            )),
            Some((false, virtualize, -40.0))
        );
    }
}

fn translated_offset_binding_scene(flags: u32) -> Scene {
    let sized = |w, h| {
        vec![
            F(LayoutComponentBase::WIDTH_PROPERTY_KEY, w),
            F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, h),
        ]
    };
    scene(
        (400.0, 300.0),
        B {
            name: "Viewport",
            component: sized(200.0, 80.0),
            children: vec![B {
                name: "List",
                component: sized(200.0, 240.0),
                scroll: Some(1),
                scroll_bind: Some((760, u64::from(flags))),
                children: vec![
                    B {
                        name: "Row",
                        component: sized(200.0, 40.0),
                        ..B::default()
                    },
                    B {
                        name: "Tail",
                        component: sized(200.0, 200.0),
                        ..B::default()
                    },
                ],
                ..B::default()
            }],
            ..B::default()
        },
    )
}

fn scroll_offset_binding(f: &Scene) -> CoreHandle {
    use crate::source::data_bind::data_bind_container::DataBindContainerOwner;
    let scroll = css_axis_scroll(f);
    DataBindContainerOwner::Authored(f.artboard.core_handle())
        .data_binds()
        .into_iter()
        .find(|bind| {
            bind.with(|owner| {
                let bind = owner.as_data_bind().unwrap();
                bind.target().as_ref() == Some(&scroll) && matches!(bind.property_key(), 759 | 760)
            }) == Some(true)
        })
        .expect("an offset binding")
}

#[test]
fn css_and_translated_offset_bindings_use_raw_values_and_push_delivery() {
    use crate::source::{
        constraints::scrolling::scroll_constraint::ScrollConstraint,
        data_bind::{
            data_bind::{BINDINGS, BINDINGS_TARGET, TWO_WAY},
            data_bind_container::DataBindContainerOwner,
        },
        generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
    };
    for css in [false, true] {
        for viewport in [0.0, 100.0] {
            let f = if css {
                css_offset_binding_scene(false, false, false, TWO_WAY, viewport)
            } else {
                let f = translated_offset_binding_scene(TWO_WAY);
                assert!(CoreRegistry::set_double_handle(
                    &f.named("Viewport"),
                    LayoutComponentBase::HEIGHT_PROPERTY_KEY.into(),
                    viewport
                ));
                f.advance(3);
                f
            };
            let scroll = css_axis_scroll(&f);
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::has_css_scroll_axis),
                Some(css)
            );
            assert!(CoreRegistry::set_double_handle(&scroll, 760, 0.0));
            let number = f.bind_number();
            f.advance(4);
            let binding = scroll_offset_binding(&f);
            let container = DataBindContainerOwner::Authored(f.artboard.core_handle());
            let source_key = ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into();
            let read = || CoreRegistry::get_double_handle(&number, source_key);
            binding
                .with(|owner| {
                    let bind = owner.as_data_bind().unwrap();
                    assert!(bind.target_supports_push());
                    assert!(!bind.in_persisting_list());
                    assert_eq!(bind.dirt(), 0);
                })
                .unwrap();
            assert_eq!(read(), Some(0.0));
            assert!(CoreRegistry::set_double_handle(&scroll, 760, -40.0));
            assert_eq!(read(), Some(0.0));
            // The offset setter queues target dirt immediately, before layout.
            binding
                .with(|owner| {
                    let bind = owner.as_data_bind().unwrap();
                    assert_eq!(bind.dirt(), BINDINGS_TARGET);
                    assert!(bind.in_dirty_list());
                })
                .unwrap();
            container.update_data_binds(true);
            assert_eq!(read(), Some(-40.0));
            for offset in [-100.0, -700.0] {
                assert!(CoreRegistry::set_double_handle(&number, source_key, offset));
                binding
                    .with(|owner| {
                        let bind = owner.as_data_bind().unwrap();
                        assert_eq!(bind.dirt(), BINDINGS);
                        assert!(bind.in_dirty_list());
                    })
                    .unwrap();
                container.update_data_binds(true);
                // Raw writes neither defer for zero size nor clamp to the range.
                assert_eq!(CoreRegistry::get_double_handle(&scroll, 760), Some(offset));
                assert_eq!(f.scroll_offset_y(), offset);
                assert_eq!(read(), Some(offset));
            }
            if css {
                assert!(CoreRegistry::set_double_handle(
                    &f.record("Viewport"),
                    HEIGHT.into(),
                    100.0
                ));
            } else {
                assert!(CoreRegistry::set_double_handle(
                    &f.named("Viewport"),
                    LayoutComponentBase::HEIGHT_PROPERTY_KEY.into(),
                    100.0
                ));
            }
            f.advance(4);
            assert_eq!(
                scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::has_css_scroll_axis),
                Some(css)
            );
            assert_eq!(CoreRegistry::get_double_handle(&scroll, 760), Some(-700.0));
            assert_eq!(read(), Some(-700.0));
        }
    }
}

#[test]
fn translated_offset_bindings_follow_upstream_mid_frame_push_order() {
    use crate::source::{
        data_bind::{
            data_bind::{BINDINGS, BINDINGS_TARGET, TWO_WAY},
            data_bind_container::DataBindContainerOwner,
        },
        generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
    };
    use std::cell::RefCell;
    thread_local! {
        static EVENTS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    }
    fn binding_changed() {
        EVENTS.with(|events| events.borrow_mut().push("binding"));
    }
    let f = translated_offset_binding_scene(TWO_WAY);
    let number = f.bind_number();
    f.advance(4);
    let scroll = css_axis_scroll(&f);
    let binding = scroll_offset_binding(&f);
    let container = DataBindContainerOwner::Authored(f.artboard.core_handle());
    binding.with_mut(|owner| {
        owner
            .as_data_bind_mut()
            .unwrap()
            .set_changed_callback(binding_changed)
    });
    let source_key = ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into();
    // Upstream data_bind.cpp keeps offsets push-driven. data_bind_container.cpp
    // drains target dirt before dependents, and does not read a stale target
    // over a source-only write. Check both passes without advancing layout.
    assert!(CoreRegistry::set_double_handle(&scroll, 760, -40.0));
    EVENTS.with(|events| assert_eq!(*events.borrow(), ["binding"]));
    assert_eq!(
        CoreRegistry::get_double_handle(&number, source_key),
        Some(0.0)
    );
    binding.with(|owner| {
        let bind = owner.as_data_bind().unwrap();
        assert_eq!(bind.dirt(), BINDINGS_TARGET);
        assert!(bind.in_dirty_list());
        assert!(!bind.in_persisting_list());
    });
    container.update_data_binds(true);
    assert_eq!(
        CoreRegistry::get_double_handle(&number, source_key),
        Some(-40.0)
    );
    assert_eq!(
        binding.with(|owner| owner.as_data_bind().unwrap().dirt()),
        Some(0)
    );
    EVENTS.with(|events| events.borrow_mut().clear());
    assert!(CoreRegistry::set_double_handle(&number, source_key, -80.0));
    EVENTS.with(|events| assert_eq!(*events.borrow(), ["binding"]));
    assert_eq!(CoreRegistry::get_double_handle(&scroll, 760), Some(-40.0));
    binding.with(|owner| {
        let bind = owner.as_data_bind().unwrap();
        assert_eq!(bind.dirt(), BINDINGS);
        assert!(bind.in_dirty_list());
    });
    container.update_data_binds(true);
    assert_eq!(CoreRegistry::get_double_handle(&scroll, 760), Some(-80.0));
    assert_eq!(
        CoreRegistry::get_double_handle(&number, source_key),
        Some(-80.0)
    );
    EVENTS.with(|events| assert_eq!(*events.borrow(), ["binding"]));
}

#[test]
fn translated_offset_bindings_keep_authored_values_and_notification_order() {
    use crate::source::{
        constraints::scrolling::scroll_constraint::ScrollConstraint,
        data_bind::data_bind::{DataBind, TO_SOURCE, TWO_WAY},
        generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase,
    };
    use std::cell::RefCell;
    thread_local! {
        static EVENTS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    }
    fn raw_changed() {
        EVENTS.with(|events| events.borrow_mut().push("raw"));
    }
    fn source_changed() {
        EVENTS.with(|events| events.borrow_mut().push("source"));
    }
    for flags in [TO_SOURCE, TWO_WAY] {
        let f = translated_offset_binding_scene(flags);
        let scroll = css_axis_scroll(&f);
        let number = f.bind_number();
        f.advance(4);
        let read = || {
            CoreRegistry::get_double_handle(
                &number,
                ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
            )
        };
        assert_eq!(read(), Some(0.0));
        // Explicit raw observers measure publication independently of binding delivery.
        let raw = scroll.insert_sibling(DataBind::new(0, 760, 0)).unwrap();
        let source = number
            .insert_sibling(DataBind::new(
                0,
                ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
                0,
            ))
            .unwrap();
        let raw_observers = scroll.ensure_property_observers().unwrap();
        let source_observers = number.ensure_property_observers().unwrap();
        for (observer, target, callback, observers) in [
            (&raw, &scroll, raw_changed as fn(), &raw_observers),
            (&source, &number, source_changed as fn(), &source_observers),
        ] {
            observer.with_mut(|owner| {
                let bind = owner.as_data_bind_mut().unwrap();
                bind.set_target(Some(target.clone()));
                bind.set_changed_callback(callback);
                observers.add(bind);
            });
        }
        for (previous, value) in [(0.0, -40.0), (-40.0, -80.0)] {
            for observer in [&raw, &source] {
                observer.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_dirt(0));
            }
            EVENTS.with(|events| events.borrow_mut().clear());
            assert!(CoreRegistry::set_double_handle(&scroll, 760, value));
            EVENTS.with(|events| assert_eq!(*events.borrow(), ["raw"]));
            assert_eq!(read(), Some(previous));
            f.advance(4);
            assert_eq!(read(), Some(value));
            assert_eq!(f.scroll_offset_y(), value);
            EVENTS.with(|events| assert_eq!(*events.borrow(), ["raw", "source"]));
            // Equal writes and unchanged offsets produce no further notifications.
            assert!(CoreRegistry::set_double_handle(&scroll, 760, value));
            f.advance(4);
            EVENTS.with(|events| assert_eq!(*events.borrow(), ["raw", "source"]));
        }
        for observer in [&raw, &source] {
            observer.with_mut(|owner| owner.as_data_bind_mut().unwrap().set_dirt(0));
        }
        EVENTS.with(|events| events.borrow_mut().clear());
        // A native offset remains authored even outside a smaller scroll range.
        assert!(CoreRegistry::set_double_handle(
            &f.named("Viewport"),
            LayoutComponentBase::HEIGHT_PROPERTY_KEY.into(),
            220.0
        ));
        f.advance(4);
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                scroll.max_offset_y(),
                scroll.authored_scroll_offset_y(),
            )),
            Some((-20.0, -80.0))
        );
        assert_eq!(read(), Some(-80.0));
        EVENTS.with(|events| assert!(events.borrow().is_empty()));
        if flags == TWO_WAY {
            assert!(CoreRegistry::set_double_handle(
                &number,
                ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY.into(),
                -120.0
            ));
            f.advance(4);
            assert_eq!(CoreRegistry::get_double_handle(&scroll, 760), Some(-120.0));
            assert_eq!(read(), Some(-120.0));
            EVENTS.with(|events| assert_eq!(*events.borrow(), ["source", "raw"]));
        }
        for (observer, observers) in [(&raw, &raw_observers), (&source, &source_observers)] {
            observer.with_mut(|owner| observers.remove(owner.as_data_bind_mut().unwrap()));
        }
    }
}

#[test]
fn translated_bound_offsets_keep_rive_sign() {
    let f = translated_scroll_view(B::default());
    let scroll = css_axis_scroll(&f);
    CoreRegistry::set_double_handle(&scroll, 760, -40.0);
    f.advance(2);
    assert_eq!(CoreRegistry::get_double_handle(&scroll, 760), Some(-40.0));
    assert_eq!(f.scroll_offset_y(), -40.0);
}

fn css_rtl_thumb_scene(track_rtl: bool) -> Scene {
    scene(
        (1000.0, 300.0),
        css(
            "Viewport",
            [
                vec![
                    U(DISPLAY, 1),
                    U(POSITION, 1),
                    U(DIRECTION, 2),
                    U(OVERFLOW_X, 3),
                ],
                width(300.0).to_vec(),
                height(40.0).to_vec(),
            ]
            .concat(),
            vec![
                B {
                    scroll: Some(0),
                    ..css(
                        "Content",
                        [
                            vec![U(DISPLAY, 1), F(FLEX_SHRINK, 0.0)],
                            width(900.0).to_vec(),
                            height(20.0).to_vec(),
                        ]
                        .concat(),
                        vec![css("Item", [width(900.0), height(20.0)].concat(), vec![])],
                    )
                },
                css(
                    "Track",
                    [
                        vec![
                            U(POSITION, 2),
                            U(DIRECTION, if track_rtl { 0 } else { 1 }),
                            U(INSET_LEFT_KIND, 1),
                            F(INSET_LEFT, 0.0),
                            U(INSET_TOP_KIND, 1),
                            F(INSET_TOP, 30.0),
                        ],
                        width(300.0).to_vec(),
                        height(10.0).to_vec(),
                    ]
                    .concat(),
                    vec![B {
                        scroll_bar: Some(0),
                        ..css("Thumb", [width(100.0), height(10.0)].concat(), vec![])
                    }],
                ),
            ],
        ),
    )
}

#[test]
fn css_rtl_thumb_mirrors_progress_against_its_solved_position() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for track_rtl in [false, true] {
        let f = css_rtl_thumb_scene(track_rtl);
        assert_eq!(f.world_origin("Thumb"), (200.0, 30.0));
        assert_eq!(f.bounds("Thumb")[2], 100.0);
        css_axis_scroll(&f)
            .with_downcast_mut::<ScrollConstraint, _>(|s| s.set_authored_scroll_offset_x(-300.0));
        f.advance(2);
        assert_eq!(f.world_origin("Thumb"), (100.0, 30.0));
    }
}

#[test]
fn css_rtl_thumb_track_click_and_drag_follow_the_mirrored_axis() {
    use crate::source::constraints::scrolling::{
        scroll_bar_constraint::ScrollBarConstraint, scroll_constraint::ScrollConstraint,
    };
    use crate::source::math::vec2d::Vec2D;
    for track_rtl in [false, true] {
        let f = css_rtl_thumb_scene(track_rtl);
        let bar = f
            .artboard
            .with_artboard(|a| a.find_all_handles::<ScrollBarConstraint>())[0]
            .clone();
        ScrollBarConstraint::hit_track(&bar, Vec2D::new(150.0, 35.0));
        f.advance(2);
        assert_eq!(
            css_axis_scroll(&f).with_downcast::<ScrollConstraint, _>(ScrollConstraint::offset_x),
            Some(-450.0)
        );
        ScrollBarConstraint::drag_thumb(&bar, Vec2D::new(50.0, 0.0), 0.1);
        f.advance(2);
        assert_eq!(
            css_axis_scroll(&f).with_downcast::<ScrollConstraint, _>(ScrollConstraint::offset_x),
            Some(-300.0)
        );
        assert_eq!(f.world_origin("Thumb").0, 100.0);
        ScrollBarConstraint::drag_thumb(&bar, Vec2D::new(-50.0, 0.0), 0.2);
        assert_eq!(
            css_axis_scroll(&f).with_downcast::<ScrollConstraint, _>(ScrollConstraint::offset_x),
            Some(-450.0)
        );
    }
}

#[test]
fn css_rtl_focus_scrolls_offscreen_items_in_logical_offsets() {
    use crate::source::{
        constraints::scrolling::scroll_constraint::ScrollConstraint, focus_data::FocusData,
    };
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [
                vec![U(DISPLAY, 1), U(DIRECTION, 2), U(OVERFLOW_X, 3)],
                width(100.0).to_vec(),
                height(40.0).to_vec(),
            ]
            .concat(),
            vec![B {
                scroll: Some(0),
                ..css(
                    "Content",
                    [
                        vec![U(DISPLAY, 1), F(FLEX_SHRINK, 0.0)],
                        width(300.0).to_vec(),
                        height(40.0).to_vec(),
                    ]
                    .concat(),
                    vec![
                        css(
                            "A",
                            [
                                vec![F(FLEX_SHRINK, 0.0)],
                                width(100.0).to_vec(),
                                height(40.0).to_vec(),
                            ]
                            .concat(),
                            vec![],
                        ),
                        css(
                            "B",
                            [
                                vec![F(FLEX_SHRINK, 0.0)],
                                width(100.0).to_vec(),
                                height(40.0).to_vec(),
                            ]
                            .concat(),
                            vec![],
                        ),
                        B {
                            text_input: true,
                            text: "focus",
                            ..css(
                                "C",
                                [
                                    vec![F(FLEX_SHRINK, 0.0)],
                                    width(100.0).to_vec(),
                                    height(40.0).to_vec(),
                                ]
                                .concat(),
                                vec![],
                            )
                        },
                    ],
                )
            }],
        ),
    );
    let scroll = css_axis_scroll(&f);
    assert_eq!(
        scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::viewport_width),
        Some(100.0)
    );
    assert_eq!(f.world_origin("C").0, -200.0);
    let focus = f
        .artboard
        .with_artboard(|a| a.find_all_handles::<FocusData>())[0]
        .clone();
    let machine = f.machine();
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    f.advance(2);
    assert!(f.field_is_focused());
    assert_eq!(
        scroll.with_downcast::<ScrollConstraint, _>(ScrollConstraint::offset_x),
        Some(-200.0)
    );
    assert_eq!(f.world_origin("C").0, 0.0);
}

#[test]
fn css_fractional_index_resolves_percentage_gaps_against_the_content_box() {
    use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for rtl in [false, true] {
        let f = scene(
            (1000.0, 300.0),
            css(
                "Viewport",
                [
                    vec![
                        U(DISPLAY, 1),
                        U(DIRECTION, if rtl { 2 } else { 1 }),
                        U(OVERFLOW_X, 3),
                    ],
                    width(100.0).to_vec(),
                    height(20.0).to_vec(),
                ]
                .concat(),
                vec![B {
                    scroll: Some(0),
                    ..css(
                        "Content",
                        [
                            vec![U(DISPLAY, 1), F(FLEX_SHRINK, 0.0), F(COLUMN_GAP + 1, 10.0)],
                            width(400.0).to_vec(),
                            height(20.0).to_vec(),
                        ]
                        .concat(),
                        ["A", "B", "C", "D"]
                            .into_iter()
                            .map(|name| {
                                css(
                                    name,
                                    [
                                        vec![F(FLEX_SHRINK, 0.0)],
                                        width(60.0).to_vec(),
                                        height(20.0).to_vec(),
                                    ]
                                    .concat(),
                                    vec![],
                                )
                            })
                            .collect(),
                    )
                }],
            ),
        );
        let scroll = css_axis_scroll(&f);
        scroll.with_downcast_mut::<ScrollConstraint, _>(|s| s.set_scroll_index(0.5));
        f.advance(2);
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|s| (s.offset_x(), s.scroll_index())),
            Some((-50.0, 0.5))
        );
    }
}

#[test]
fn css_rtl_thumb_overscroll_shrink_stays_at_the_mirrored_edge() {
    use crate::source::constraints::scrolling::{
        elastic_scroll_physics::ElasticScrollPhysics, scroll_bar_constraint::ScrollBarConstraint,
        scroll_constraint::ScrollConstraint,
    };
    for track_rtl in [false, true] {
        let f = css_rtl_thumb_scene(track_rtl);
        let bar = f
            .artboard
            .with_artboard(|a| a.find_all_handles::<ScrollBarConstraint>())[0]
            .clone();
        bar.with_downcast_mut::<ScrollBarConstraint, _>(|bar| bar.base.set_auto_size_value(true));
        let scroll = css_axis_scroll(&f);
        let physics = scroll
            .insert_sibling(ElasticScrollPhysics::default())
            .unwrap();
        scroll.with_downcast_mut::<ScrollConstraint, _>(|s| {
            s.set_physics(physics);
            s.prime_physics();
        });
        for offset in [60.0, -660.0] {
            scroll.with_downcast_mut::<ScrollConstraint, _>(|s| {
                s.set_authored_scroll_offset_x(offset)
            });
            f.advance(4);
            let left = f.world_origin("Thumb").0;
            let width = f.bounds("Thumb")[2];
            assert!(width < 100.0 && width > 0.0, "{width}");
            if offset > 0.0 {
                assert!((left + width - 300.0).abs() < 0.01, "{left}, {width}");
            } else {
                assert!(left.abs() < 0.01, "{left}");
            }
        }
    }
}

/// A CSS viewport's scroll bar thumb shows its visible share of the
/// scrollable extent, LTR or RTL: 100 of 300 points.
#[test]
fn css_scroll_bar_ratio_follows_the_scrollable_extent() {
    const RTL: u64 = 2;
    const HORIZONTAL: u64 = 0;
    let ratio = |direction: u64, content_width: f32| {
        let f = scene(
            (400.0, 300.0),
            css(
                "Viewport",
                [
                    [U(DISPLAY, 1), U(DIRECTION, direction), U(OVERFLOW_X, 3)].as_slice(),
                    &width(100.0),
                    &height(20.0),
                ]
                .concat(),
                vec![B {
                    scroll: Some(HORIZONTAL),
                    ..css(
                        "Content",
                        [
                            [U(DISPLAY, 1), F(FLEX_SHRINK, 0.0)].as_slice(),
                            &width(content_width),
                            &height(20.0),
                        ]
                        .concat(),
                        vec![css(
                            "Item",
                            [
                                [F(FLEX_SHRINK, 0.0)].as_slice(),
                                &width(300.0),
                                &height(20.0),
                            ]
                            .concat(),
                            vec![],
                        )],
                    )
                }],
            ),
        );
        use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
        f.artboard
            .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
            .into_iter()
            .next()
            .expect("a scroll constraint")
            .with_downcast::<ScrollConstraint, _>(ScrollConstraint::visible_width_ratio)
            .unwrap()
    };
    let close = |value: f32| (value - 1.0 / 3.0).abs() < 1e-5;
    // A 100-point content box overflowed by its 300-point item.
    assert!(close(ratio(1, 100.0)), "{}", ratio(1, 100.0));
    // An RTL content box of 300 points, placed at -200.
    assert!(close(ratio(RTL, 300.0)), "{}", ratio(RTL, 300.0));
}

/// An inset shadow is bounded by its clip, the padding box: a box just
/// outside its parent's clip is not visible geometry.
#[test]
fn css_inset_shadow_bounds_stay_in_its_padding_box() {
    const MARGIN_LEFT: u16 = 61049;
    let f = scene(
        (400.0, 300.0),
        css(
            "Clip",
            [
                [U(OVERFLOW_X, 2), U(OVERFLOW_Y, 2)].as_slice(),
                &width(100.0),
                &height(100.0),
            ]
            .concat(),
            vec![B {
                paints: vec![Paint::Shadow {
                    color: 0x8000_0000,
                    strength: 0.0,
                    spread: 0.0,
                }],
                ..css(
                    "Outside",
                    [
                        [F(MARGIN_LEFT, 101.0)].as_slice(),
                        &width(10.0),
                        &height(10.0),
                    ]
                    .concat(),
                    vec![],
                )
            }],
        ),
    );
    assert!(CoreRegistry::set_bool_handle(
        &f.paint_child("Outside", FeatherBase::TYPE_KEY),
        FeatherBase::INNER_PROPERTY_KEY.into(),
        true
    ));
    f.advance(2);
    assert_eq!(f.bounds("Outside")[..2], [101.0, 0.0]);
    let visible = f.host().visible_geometry_with_bounds();
    let names = f.geometry_names(&visible);
    assert!(!names.contains(&"Outside".to_owned()), "{names:?}");
}

/// A translated child that layout places absolutely is positioned in its
/// CSS parent's painting order: it paints above a later static sibling.
#[test]
fn css_translated_absolute_child_paints_above_static_siblings() {
    let f = frame(vec![
        B {
            name: "Red",
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 50.0),
            ],
            style: vec![U(
                LayoutComponentStyleBase::POSITION_TYPE_VALUE_PROPERTY_KEY,
                2,
            )],
            paints: vec![Paint::Fill(0xff00_0001)],
            ..B::default()
        },
        painted("Blue", 0xff00_0002, sized(&[])),
    ]);
    let names = f.draw_names_top_first();
    let at = |name: &str| names.iter().position(|n| n == name).unwrap();
    assert!(at("Red") < at("Blue"), "{names:?}");
}

#[test]
fn css_virtualized_scroll_keeps_native_metrics_and_content_origin() {
    use crate::source::{
        constraints::scrolling::scroll_constraint::ScrollConstraint,
        generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        math::vec2d::Vec2D,
    };
    const MARGIN_LEFT: u16 = 61049;
    for horizontal in [false, true] {
        let f = scene(
            (500.0, 500.0),
            css(
                "Viewport",
                [
                    vec![
                        U(DISPLAY, 1),
                        U(if horizontal { OVERFLOW_X } else { OVERFLOW_Y }, 3),
                    ],
                    width(100.0).to_vec(),
                    height(100.0).to_vec(),
                ]
                .concat(),
                vec![B {
                    scroll: Some(if horizontal { 0 } else { 1 }),
                    scroll_properties: vec![U(ScrollConstraintBase::VIRTUALIZE_PROPERTY_KEY, 1)],
                    ..css(
                        "Content",
                        [
                            vec![
                                U(DISPLAY, 1),
                                U(61006, if horizontal { 0 } else { 2 }),
                                F(FLEX_SHRINK, 0.0),
                                F(if horizontal { MARGIN_LEFT } else { MARGIN_TOP }, 20.0),
                            ],
                            width(if horizontal { 400.0 } else { 100.0 }).to_vec(),
                            height(if horizontal { 100.0 } else { 400.0 }).to_vec(),
                        ]
                        .concat(),
                        (0..4)
                            .map(|index| {
                                css(
                                    if index == 3 { "Last" } else { "Row" },
                                    [
                                        vec![F(FLEX_SHRINK, 0.0)],
                                        width(100.0).to_vec(),
                                        height(100.0).to_vec(),
                                    ]
                                    .concat(),
                                    vec![],
                                )
                            })
                            .collect(),
                    )
                }],
            ),
        );
        let scroll = css_axis_scroll(&f);
        let content = f.bounds("Content");
        assert_eq!(if horizontal { content[0] } else { content[1] }, 20.0);
        let last = f.world_origin("Last");
        assert_eq!(if horizontal { last.0 } else { last.1 }, 320.0);
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                scroll.virtualize(),
                if horizontal {
                    scroll.content_width()
                } else {
                    scroll.content_height()
                },
            )),
            Some((true, 400.0)),
        );
        // Scrolling to the end must include the content's 20-point inset.
        scroll.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
            scroll.scroll_by(if horizontal {
                Vec2D::new(-1000.0, 0.0)
            } else {
                Vec2D::new(0.0, -1000.0)
            })
        });
        f.advance(3);
        let last = f.world_origin("Last");
        let size = f.bounds("Last");
        assert_eq!(
            if horizontal {
                last.0 + size[2]
            } else {
                last.1 + size[3]
            },
            100.0,
            "horizontal={horizontal}",
        );
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                if horizontal {
                    scroll.viewport_width()
                } else {
                    scroll.viewport_height()
                },
                if horizontal {
                    scroll.max_offset_x()
                } else {
                    scroll.max_offset_y()
                },
                if horizontal {
                    scroll.offset_x()
                } else {
                    scroll.offset_y()
                },
            )),
            Some((80.0, -320.0, -320.0)),
        );
        assert_eq!(
            scroll.with_downcast::<ScrollConstraint, _>(|scroll| (
                scroll.has_css_scroll_axis(),
                scroll.css_scrollport(),
            )),
            Some((false, None))
        );
    }
}

/// The orientation helpers virtualized scrolling reads follow a CSS box's
/// record, not its retained Rive style: its main axis from
/// `flex-direction`, its wrapping from `flex-wrap`.
#[test]
fn css_virtualized_scroll_axis_follows_the_css_record() {
    use crate::source::generated::layout::layout_component_style_base::LayoutComponentStyleBase as Style;
    let axis = |props: &[P], style: Vec<P>| {
        let f = frame(vec![B {
            style,
            ..css(
                "List",
                [
                    [U(DISPLAY, 1)].as_slice(),
                    props,
                    &width(200.0),
                    &height(100.0),
                ]
                .concat(),
                vec![],
            )
        }]);
        f.named("List")
            .with_downcast::<LayoutComponent, _>(|layout| {
                (
                    layout.main_axis_is_row(),
                    layout.main_axis_is_column(),
                    layout.wraps_lines(),
                )
            })
            .unwrap()
    };
    // A CSS row, then a CSS column, beside the default Rive row.
    assert_eq!(axis(&[], vec![]), (true, false, false));
    assert_eq!(axis(&[U(61006, 2)], vec![]), (false, true, false));
    // CSS wrap beside a Rive nowrap, and CSS nowrap beside a Rive row wrap.
    assert_eq!(axis(&[U(61007, 1)], vec![]), (true, false, true));
    let rive_row_wrap = vec![
        U(Style::FLEX_DIRECTION_VALUE_PROPERTY_KEY, 2),
        U(Style::FLEX_WRAP_VALUE_PROPERTY_KEY, 1),
    ];
    assert_eq!(axis(&[], rive_row_wrap), (true, false, false));
}

/// A box's painted bounds measure each of its paths with the paint that
/// draws it. A blurred inset shadow, drawn only inside the padding box,
/// grows neither its own bounds past the box nor the background's; a
/// BoxBorder fills its ring, so its Stroke's thickness adds nothing.
#[test]
fn css_painted_bounds_measure_each_path_with_its_own_paint() {
    const MARGIN_LEFT: u16 = 61049;
    let at = |props: &[P]| {
        [
            [F(MARGIN_TOP, 50.0), F(MARGIN_LEFT, 50.0)].as_slice(),
            props,
        ]
        .concat()
    };
    let bounds = |f: &Scene| {
        let visible = f.host().visible_geometry_with_bounds();
        let bounds = visible[0].bounds;
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y)
    };
    let inset = scene(
        (400.0, 300.0),
        B {
            paints: vec![
                Paint::Fill(0xff11_2233),
                Paint::InsetShadow {
                    color: 0x8000_0000,
                    strength: 20.0,
                    spread: 0.0,
                },
            ],
            ..css("Card", at(&[width(100.0), height(50.0)].concat()), vec![])
        },
    );
    assert_eq!(bounds(&inset), (50.0, 50.0, 150.0, 100.0));
    // The border ring fills the border box; its Stroke is 4 points thick.
    let border = scene(
        (400.0, 300.0),
        B {
            paints: vec![Paint::Border(0xff44_5566, 15)],
            ..css("Card", at(&border_box(&[])), vec![])
        },
    );
    assert_eq!(bounds(&border), (50.0, 50.0, 150.0, 100.0));
}

/// A hidden shadow adds nothing to its box's painted bounds: the visible
/// geometry is the background alone.
#[test]
fn css_hidden_shadow_adds_no_painted_bounds() {
    const MARGIN_LEFT: u16 = 61049;
    let f = scene(
        (400.0, 300.0),
        B {
            paints: vec![
                Paint::Shadow {
                    color: 0x8000_0000,
                    strength: 0.0,
                    spread: 20.0,
                },
                Paint::Fill(0xff11_2233),
            ],
            ..css(
                "Card",
                [
                    [F(MARGIN_TOP, 50.0), F(MARGIN_LEFT, 50.0)].as_slice(),
                    &width(100.0),
                    &height(50.0),
                ]
                .concat(),
                vec![],
            )
        },
    );
    let bounds = |f: &Scene| {
        let visible = f.host().visible_geometry_with_bounds();
        let bounds = visible[0].bounds;
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y)
    };
    assert_eq!(bounds(&f), (30.0, 30.0, 170.0, 120.0));
    let shadow = f
        .named("Card")
        .with_downcast::<LayoutComponent, _>(|layout| layout.paints.shape_paints()[0].clone())
        .unwrap();
    assert!(CoreRegistry::set_bool_handle(
        &shadow,
        crate::source::generated::shapes::paint::shape_paint_base::ShapePaintBase::IS_VISIBLE_PROPERTY_KEY
            .into(),
        false
    ));
    f.advance(2);
    assert_eq!(bounds(&f), (50.0, 50.0, 150.0, 100.0));
}

const RADIUS_PERCENTS: [u16; 4] = [61079, 61080, 61081, 61082];

/// A painted CSS box of `width` by `height` with `percent` radii on every
/// corner, and the names hit at `point`.
fn rounded_hits(
    width_value: f32,
    height_value: f32,
    percent: f32,
    point: (f32, f32),
) -> Vec<String> {
    let f = frame(vec![painted(
        "Round",
        0xff00_0001,
        [
            RADIUS_PERCENTS.map(|key| F(key, percent)).as_slice(),
            &width(width_value),
            &height(height_value),
        ]
        .concat(),
    )]);
    let hits = f
        .host()
        .hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(point.0, point.1));
    f.geometry_names(&hits)
}

/// A rounded CSS box is hit in its rounded border box, as a browser hits
/// it: a tap in a cut-off corner misses, one by an edge hits.
#[test]
fn css_rounded_box_is_hit_in_its_rounded_border_box() {
    assert!(rounded_hits(100.0, 100.0, 50.0, (5.0, 5.0)).is_empty());
    assert_eq!(rounded_hits(100.0, 100.0, 50.0, (50.0, 2.0)), ["Round"]);
    assert_eq!(rounded_hits(100.0, 100.0, 50.0, (50.0, 50.0)), ["Round"]);
}

/// Percent radii on a box that is not square are elliptical for hits too:
/// 50% of 200 by 100 misses a point a 50-point circle would take.
#[test]
fn css_elliptical_radii_shape_hits() {
    assert!(rounded_hits(200.0, 100.0, 50.0, (30.0, 10.0)).is_empty());
    assert_eq!(rounded_hits(200.0, 100.0, 50.0, (100.0, 3.0)), ["Round"]);
    assert_eq!(rounded_hits(200.0, 100.0, 50.0, (3.0, 50.0)), ["Round"]);
}

/// A child passes hits through its parent's rounded clip as it is drawn:
/// in the clipped-off corner it is not hit.
#[test]
fn css_children_are_hit_through_a_rounded_clip() {
    let f = frame(vec![css(
        "Clip",
        [
            [U(OVERFLOW_X, 2), U(OVERFLOW_Y, 2)].as_slice(),
            RADIUS_PERCENTS.map(|key| F(key, 50.0)).as_slice(),
            &width(100.0),
            &height(100.0),
        ]
        .concat(),
        vec![painted("Inside", 0xff00_0002, sized_to(100.0, 100.0))],
    )]);
    let names = |x: f32, y: f32| {
        let hits = f
            .host()
            .hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(x, y));
        f.geometry_names(&hits)
    };
    assert!(names(5.0, 5.0).is_empty(), "{:?}", names(5.0, 5.0));
    assert_eq!(names(50.0, 50.0), ["Inside"]);
}

/// A card with no background whose only paint is its top border is
/// selected only on that side: a point in its unpainted middle selects
/// nothing, square or rounded, while a tap there still takes the card.
#[test]
fn css_patterned_border_selection_stays_in_its_visible_side() {
    for radius in [0.0, 10.0] {
        let f = frame(vec![B {
            paints: vec![Paint::Border(0xff00_0001, 1)],
            style: vec![F(
                LayoutComponentStyleBase::CORNER_RADIUS_TL_PROPERTY_KEY,
                radius,
            )],
            ..css(
                "Card",
                [
                    [
                        U(BOX_SIZING, 1),
                        F(BORDER_TOP, 4.0),
                        F(BORDER_RIGHT, 6.0),
                        F(BORDER_BOTTOM, 8.0),
                        F(BORDER_LEFT, 2.0),
                    ]
                    .as_slice(),
                    &width(68.0),
                    &height(47.0),
                ]
                .concat(),
                vec![],
            )
        }]);
        let mut host = f.host();
        let visible = host.visible_geometry_with_bounds();
        assert_eq!(f.geometry_names(&visible), ["Card"], "radius {radius}");
        // The top side's ring: 4 deep, and down into its rounded corners.
        let bounds = visible[0].bounds;
        assert_eq!((bounds.min_x, bounds.min_y, bounds.max_x), (0.0, 0.0, 68.0));
        assert!(
            bounds.max_y <= radius.max(4.0) + 1.0e-3,
            "radius {radius}: {bounds:?}"
        );
        let mut selected = |x: f32, y: f32| {
            let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(x, y));
            f.geometry_names(&hits)
        };
        assert_eq!(selected(34.0, 2.0), ["Card"], "radius {radius}");
        assert!(selected(30.0, 33.0).is_empty(), "radius {radius}");
        assert!(tapped(&f, "Card", (30.0, 33.0)), "radius {radius}");
    }
}

/// A dashed border is selected only on its dashes, as it paints: the gap
/// between the first two dashes of a 4-point dashed top (8-point dashes,
/// fitted 3.5-point gaps) selects nothing, while a tap there still takes
/// the box.
#[test]
fn css_dashed_border_selection_skips_its_gaps() {
    let f = frame(vec![B {
        paints: vec![Paint::StyledBorder(0xff00_0001, 1, 1)],
        ..css(
            "Card",
            [
                [U(BOX_SIZING, 1), F(BORDER_TOP, 4.0)].as_slice(),
                &width(100.0),
                &height(40.0),
            ]
            .concat(),
            vec![],
        )
    }]);
    let mut host = f.host();
    let mut selected = |x: f32, y: f32| {
        let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(x, y));
        f.geometry_names(&hits)
    };
    assert_eq!(selected(4.0, 2.0), ["Card"], "on a dash");
    assert!(selected(10.0, 2.0).is_empty(), "in a gap");
    assert!(tapped(&f, "Card", (10.0, 2.0)));
}

/// The front of two overlapping CSS viewports, its corners cut 14 points
/// round and holding only an image, takes no tap in a cut corner, and its
/// image is not hit there through the rounded clip; both are hit inside.
#[test]
fn css_rounded_viewport_passes_corner_hits_through() {
    let f = frame(vec![
        painted(
            "Back",
            0xff00_0001,
            [[U(POSITION, 2)].as_slice(), &width(100.0), &height(100.0)].concat(),
        ),
        B {
            images: vec!["Picture"],
            ..css(
                "Front",
                [
                    [U(POSITION, 2), U(OVERFLOW_X, 3), U(OVERFLOW_Y, 3)].as_slice(),
                    RADIUS_PERCENTS.map(|key| F(key, 14.0)).as_slice(),
                    &width(100.0),
                    &height(100.0),
                ]
                .concat(),
                vec![],
            )
        },
    ]);
    assert!(!tapped(&f, "Front", (2.0, 2.0)));
    assert!(!tapped(&f, "Picture", (2.0, 2.0)));
    assert!(tapped(&f, "Front", (14.0, 2.0)));
    assert!(tapped(&f, "Picture", (14.0, 2.0)));
    assert!(tapped(&f, "Back", (2.0, 2.0)));
}

/// A viewport's radius animated from 0 to 40% clips its children's hits as
/// it is at each frame: its image is hit in the corner at the start and
/// not at the end.
#[test]
fn css_animated_radius_clips_child_hits_live() {
    let f = frame(vec![B {
        images: vec!["Picture"],
        css_keys: RADIUS_PERCENTS
            .iter()
            .map(|&key| (key, vec![(0, 0.0), (60, 40.0)]))
            .collect(),
        ..css(
            "Front",
            [
                [U(OVERFLOW_X, 3), U(OVERFLOW_Y, 3)].as_slice(),
                &width(100.0),
                &height(100.0),
            ]
            .concat(),
            vec![],
        )
    }]);
    animate(&f, 0.0);
    assert!(tapped(&f, "Picture", (2.0, 2.0)));
    animate(&f, 1.0);
    assert!(!tapped(&f, "Picture", (2.0, 2.0)));
    assert!(tapped(&f, "Picture", (50.0, 20.0)));
}

/// `display: none` hides a CSS artboard's own paint, as it hides a box's:
/// no fill, border or shadow draws, whatever its BoxPaint's size and
/// whether or not it clips, and it is no visible geometry. A hidden
/// component (data-show hides with display: none) paints nothing.
#[test]
fn css_hidden_artboard_paints_none_of_its_own_paints() {
    let paints = vec![
        Paint::Shadow {
            color: 0xff77_8899,
            strength: 4.0,
            spread: 2.0,
        },
        Paint::Fill(0xff11_2233),
        Paint::Border(0xff44_5566, 15),
    ];
    let record = vec![
        U(DISPLAY, 3),
        F(BORDER_TOP, 4.0),
        F(BORDER_RIGHT, 4.0),
        F(BORDER_BOTTOM, 4.0),
        F(BORDER_LEFT, 4.0),
    ];
    for unclipped in [false, true] {
        let f = try_scene_spec(
            (100.0, 100.0),
            ArtboardSpec {
                css: Some(record.clone()),
                paints: paints.clone(),
                box_paint: Some(paint_points(40.0, 40.0)),
                unclipped,
                ..ArtboardSpec::default()
            },
            Some(B {
                paints: paints.clone(),
                box_paint: Some(paint_points(40.0, 40.0)),
                ..css("Hidden", record.clone(), vec![])
            }),
        )
        .expect("the scene imports");
        let lines = f.draw_lines();
        for color in ["0xff778899", "0xff112233", "0xff445566"] {
            assert!(
                !lines
                    .iter()
                    .any(|line| line.starts_with("drawPath") && line.contains(color)),
                "unclipped {unclipped}: a draw with {color}:\n{}",
                lines.join("\n")
            );
        }
        let visible = f.host().visible_geometry_with_bounds();
        assert!(
            f.geometry_names(&visible).is_empty(),
            "unclipped {unclipped}: {:?}",
            f.geometry_names(&visible)
        );
    }
}

/// A rounded CSS artboard's own clip bounds where its larger BoxPaint can
/// be selected: not in a clipped-off corner, as it draws.
#[test]
fn css_artboard_box_paint_selection_stays_in_its_rounded_clip() {
    let f = try_scene_spec(
        (100.0, 100.0),
        ArtboardSpec {
            css: Some(RADIUS_PERCENTS.map(|key| F(key, 50.0)).to_vec()),
            paints: vec![Paint::Fill(0xff11_2233)],
            box_paint: Some(paint_points(200.0, 200.0)),
            ..ArtboardSpec::default()
        },
        Some(css("Child", [width(10.0), height(10.0)].concat(), vec![])),
    )
    .expect("the scene imports");
    let mut host = f.host();
    let mut selected = |x: f32, y: f32| {
        let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(x, y));
        f.geometry_names(&hits)
    };
    assert!(selected(5.0, 5.0).is_empty());
    assert_eq!(selected(50.0, 50.0), ["Artboard"]);
}

/// Clicks the primary button at a point: the names of the events the click
/// fires, in the order its listeners fire them, and whether a listener under
/// the point answers the button beforehand.
fn click(
    machine: &crate::RuntimeStateMachineInstanceHandle,
    x: f32,
    y: f32,
) -> (bool, Vec<String>) {
    use crate::source::{math::vec2d::Vec2D, pointer_button::PointerButton};
    let point = Vec2D::new(x, y);
    let listens = machine
        .with_instance_mut(|machine| machine.listens_to_button_at(point, PointerButton::Primary));
    let before = machine.with_instance(|machine| machine.reported_event_count());
    machine.with_instance_mut(|machine| {
        machine.pointer_down(point, 0, PointerButton::Primary);
        machine.pointer_up(point, 0, PointerButton::Primary);
    });
    let fired = machine.with_instance(|machine| {
        (before..machine.reported_event_count())
            .filter_map(|index| machine.reported_event_at(index).event)
            .map(|event| {
                event
                    .with(|event| {
                        event
                            .as_component()
                            .map(|event| event.base.name().to_owned())
                    })
                    .flatten()
                    .expect("a named event")
            })
            .collect()
    });
    (listens, fired)
}

/// A pointer event bubbles from the topmost hit to its CSS ancestors, as a
/// browser's does (docs/CSS_LAYOUT.md, 4.5). A text input is an opaque hit,
/// and a click on it reaches the listeners of the boxes it is in, deepest
/// first, and still focuses the field.
#[test]
fn css_box_listeners_hear_a_click_on_their_text_input() {
    let f = scene(
        (200.0, 200.0),
        B {
            listener: Some("outer clicked"),
            ..css(
                "Outer",
                [width(200.0), height(100.0)].concat(),
                vec![B {
                    listener: Some("field clicked"),
                    text_input: true,
                    ..css("Input box", [width(180.0), height(40.0)].concat(), vec![])
                }],
            )
        },
    );
    let machine = f.machine();
    assert_eq!(
        click(&machine, 195.0, 80.0),
        (true, vec!["outer clicked".to_owned()]),
        "beside the field"
    );
    assert!(!f.field_is_focused());
    assert_eq!(
        click(&machine, 90.0, 20.0),
        (
            true,
            vec!["field clicked".to_owned(), "outer clicked".to_owned()]
        ),
        "on the field"
    );
    assert!(f.field_is_focused());
}

/// An opaque hit still blocks what is behind it that it is not inside: the
/// box under a text input's box, its sibling, hears nothing there, while the
/// box both are in hears the click.
#[test]
fn css_text_input_blocks_a_sibling_box_behind_it() {
    let f = scene(
        (200.0, 200.0),
        B {
            listener: Some("row clicked"),
            ..css(
                "Row",
                [[U(POSITION, 1)].as_slice(), &width(200.0), &height(100.0)].concat(),
                vec![
                    B {
                        listener: Some("behind clicked"),
                        ..css("Behind", [width(190.0), height(40.0)].concat(), vec![])
                    },
                    B {
                        text_input: true,
                        ..css(
                            "Front",
                            [
                                [
                                    U(POSITION, 2),
                                    U(INSET_LEFT_KIND, 1),
                                    F(INSET_LEFT, 0.0),
                                    U(INSET_TOP_KIND, 1),
                                    F(INSET_TOP, 0.0),
                                ]
                                .as_slice(),
                                &width(180.0),
                                &height(40.0),
                            ]
                            .concat(),
                            vec![],
                        )
                    },
                ],
            )
        },
    );
    let machine = f.machine();
    assert_eq!(
        click(&machine, 185.0, 20.0),
        (
            true,
            vec!["behind clicked".to_owned(), "row clicked".to_owned()]
        ),
        "beside the field"
    );
    assert_eq!(
        click(&machine, 90.0, 20.0),
        (true, vec!["row clicked".to_owned()]),
        "on the field"
    );
    assert!(f.field_is_focused());
}

/// Translated (Rive) content keeps Rive's hit testing, as upstream has it:
/// an opaque child blocks its parents' listeners.
#[test]
fn translated_text_input_blocks_its_parents_listeners() {
    let sized = |width: f32, height: f32| {
        vec![
            F(LayoutComponentBase::WIDTH_PROPERTY_KEY, width),
            F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, height),
        ]
    };
    let f = scene(
        (200.0, 200.0),
        B {
            name: "Outer",
            component: sized(200.0, 100.0),
            listener: Some("outer clicked"),
            children: vec![B {
                name: "Input box",
                component: sized(180.0, 40.0),
                listener: Some("field clicked"),
                text_input: true,
                ..B::default()
            }],
            ..B::default()
        },
    );
    let machine = f.machine();
    assert_eq!(
        click(&machine, 195.0, 80.0),
        (true, vec!["outer clicked".to_owned()]),
        "beside the field"
    );
    assert_eq!(click(&machine, 90.0, 20.0), (false, vec![]), "on the field");
    assert!(f.field_is_focused());
}

/// Presses at `from`, moves through `path` and releases at its end: the
/// names of the events the gesture fires.
fn press_and_drag(
    machine: &crate::RuntimeStateMachineInstanceHandle,
    from: (f32, f32),
    path: &[(f32, f32)],
) -> Vec<String> {
    use crate::source::{math::vec2d::Vec2D, pointer_button::PointerButton};
    let before = machine.with_instance(|machine| machine.reported_event_count());
    machine.with_instance_mut(|machine| {
        let start = Vec2D::new(from.0, from.1);
        machine.pointer_move(start, 0.0, 0);
        machine.pointer_down(start, 0, PointerButton::Primary);
        let mut last = start;
        for (step, &(x, y)) in path.iter().enumerate() {
            last = Vec2D::new(x, y);
            machine.pointer_move(last, (step + 1) as f32 / 60.0, 0);
        }
        machine.pointer_up(last, 0, PointerButton::Primary);
    });
    machine.with_instance(|machine| {
        (before..machine.reported_event_count())
            .filter_map(|index| machine.reported_event_at(index).event)
            .map(|event| {
                event
                    .with(|event| {
                        event
                            .as_component()
                            .map(|event| event.base.name().to_owned())
                    })
                    .flatten()
                    .expect("a named event")
            })
            .collect()
    })
}

impl Scene {
    /// The offset of the nested artboard's ScrollConstraint.
    fn nested_scroll_offset(&self) -> (f32, f32) {
        use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
        let nested = self
            .artboard
            .with_artboard(|artboard| {
                artboard.find_handle::<crate::source::nested_artboard::NestedArtboard>("Nested")
            })
            .expect("the nested artboard");
        let instance = nested
            .with(|nested| nested.nested_artboard_instance_handle())
            .flatten()
            .expect("its instance");
        instance
            .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
            .into_iter()
            .next()
            .expect("its scroll constraint")
            .with_downcast(|scroll: &ScrollConstraint| (scroll.offset_x(), scroll.offset_y()))
            .expect("a live scroll constraint")
    }
    /// The offset of the ScrollConstraint on the box named `content`.
    fn scroll_offset(&self, content: &str) -> (f32, f32) {
        use crate::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
        let content = self.named(content);
        self.artboard
            .with_artboard(|artboard| artboard.find_all_handles::<ScrollConstraint>())
            .into_iter()
            .find(|scroll| {
                scroll
                    .with(|object| object.component_parent_handle())
                    .flatten()
                    .as_ref()
                    == Some(&content)
            })
            .expect("a scroll constraint on the box")
            .with_downcast(|scroll: &ScrollConstraint| (scroll.offset_x(), scroll.offset_y()))
            .expect("a live scroll constraint")
    }
    /// The text input named "Field"'s selected text.
    fn field_selection(&self) -> String {
        self.artboard
            .with_artboard(|artboard| {
                artboard.find_handle::<crate::source::text::text_input::TextInput>("Field")
            })
            .expect("the field")
            .with_downcast(crate::source::text::text_input::TextInput::selected_text)
            .flatten()
            .unwrap_or_default()
    }
}

/// A CSS scroll box as the compiler writes one, with a 10 point threshold
/// (docs/CSS_LAYOUT.md, 3): a 200 by 80 viewport scrolling a list
/// vertically, whose first 40 point row holds `item`, above a tall tail.
/// The row keeps the list from being a field's own scroller, which is its
/// grandparent's (upstream 0dd067f1).
fn css_scroll_view(listener: Option<&'static str>, item: B) -> Scene {
    scene(
        (400.0, 300.0),
        B {
            listener,
            ..css(
                "Viewport",
                [
                    [U(POSITION, 1), U(OVERFLOW_Y, 3)].as_slice(),
                    &width(200.0),
                    &height(80.0),
                ]
                .concat(),
                vec![B {
                    scroll: Some(1),
                    scroll_threshold: 10.0,
                    ..css(
                        "List",
                        [],
                        vec![
                            css("Row", height(40.0), vec![item]),
                            css("Tail", height(200.0), vec![]),
                        ],
                    )
                }],
            )
        },
    )
}

/// The same scroll view translated from Rive boxes, whose ScrollConstraint
/// keeps Rive's default threshold, 0.
fn translated_scroll_view(item: B) -> Scene {
    let sized = |width: f32, height: f32| {
        vec![
            F(LayoutComponentBase::WIDTH_PROPERTY_KEY, width),
            F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, height),
        ]
    };
    scene(
        (400.0, 300.0),
        B {
            name: "Viewport",
            component: sized(200.0, 80.0),
            children: vec![B {
                name: "List",
                component: sized(200.0, 240.0),
                scroll: Some(1),
                children: vec![
                    B {
                        name: "Row",
                        component: sized(200.0, 40.0),
                        children: vec![item],
                        ..B::default()
                    },
                    B {
                        name: "Tail",
                        component: sized(200.0, 200.0),
                        ..B::default()
                    },
                ],
                ..B::default()
            }],
            ..B::default()
        },
    )
}

/// A 180 by 40 box holding a field, CSS or translated.
fn field_box(css_mode: bool, listener: Option<&'static str>, text: &'static str) -> B {
    let mut field = if css_mode {
        css("Input box", [width(180.0), height(40.0)].concat(), vec![])
    } else {
        B {
            name: "Input box",
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 180.0),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0),
            ],
            ..B::default()
        }
    };
    field.listener = listener;
    field.text_input = true;
    field.text = text;
    field
}

/// A drag goes to the innermost drag handler under the pointer
/// (docs/CSS_LAYOUT.md, 4.5). An unfocused field is not one: a tap focuses
/// it on release and fires the clicks around it, while a drag that starts on
/// it scrolls the CSS scroll box around it, focuses nothing and fires no
/// click.
#[test]
fn css_drag_on_an_unfocused_field_scrolls_its_scroll_box() {
    let f = css_scroll_view(
        Some("viewport clicked"),
        field_box(true, Some("field clicked"), "hello world"),
    );
    let machine = f.machine();
    assert_eq!(
        press_and_drag(&machine, (90.0, 20.0), &[]),
        ["field clicked", "viewport clicked"],
        "a tap"
    );
    assert!(f.field_is_focused());
    machine.with_instance_mut(|machine| machine.clear_focus());
    assert_eq!(
        press_and_drag(&machine, (90.0, 20.0), &[(90.0, 17.0), (90.0, 15.0)]),
        ["field clicked", "viewport clicked"],
        "a tap that moves 5 points"
    );
    assert!(f.field_is_focused());
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0));
    machine.with_instance_mut(|machine| machine.clear_focus());
    assert_eq!(
        press_and_drag(
            &machine,
            (90.0, 35.0),
            &[(90.0, 25.0), (90.0, 15.0), (90.0, 5.0)]
        ),
        Vec::<String>::new(),
        "a drag"
    );
    assert_eq!(f.scroll_offset("List"), (0.0, -30.0));
    assert!(!f.field_is_focused());
}

/// A button is no drag handler: a drag that starts on it scrolls the box
/// around it, and cancels its click once it does. Below the 10 point
/// threshold the press stays a tap.
#[test]
fn css_drag_on_a_button_scrolls_and_cancels_its_click() {
    let button = || B {
        listener: Some("button clicked"),
        ..css("Button", [width(180.0), height(40.0)].concat(), vec![])
    };
    let f = css_scroll_view(None, button());
    let machine = f.machine();
    assert_eq!(
        press_and_drag(&machine, (90.0, 20.0), &[(90.0, 15.0)]),
        ["button clicked"],
        "a press that moves 5 points"
    );
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0));
    let f = css_scroll_view(None, button());
    let machine = f.machine();
    assert_eq!(
        press_and_drag(&machine, (90.0, 30.0), &[(90.0, 15.0)]),
        Vec::<String>::new(),
        "a press that moves 15 points"
    );
    assert_eq!(f.scroll_offset("List"), (0.0, -15.0));
}

/// A focused field takes the drag to select, so the scroll box around it
/// does not scroll.
#[test]
fn css_drag_in_a_focused_field_selects_and_does_not_scroll() {
    let f = css_scroll_view(None, field_box(true, None, "hello world"));
    let machine = f.machine();
    press_and_drag(&machine, (90.0, 20.0), &[]);
    assert!(f.field_is_focused());
    assert_eq!(f.field_selection(), "");
    press_and_drag(
        &machine,
        (1.0, 35.0),
        &[(30.0, 25.0), (60.0, 15.0), (70.0, 5.0)],
    );
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0));
    assert!(!f.field_selection().is_empty());
}

/// A scroll bar's thumb takes its own drag: dragging it 15 points down a
/// 60 point track range scrolls the 160 point range by 40, and the content
/// around it does not also drag.
#[test]
fn css_scroll_bar_thumb_drags_only_the_thumb() {
    let f = scene(
        (400.0, 300.0),
        css(
            "Viewport",
            [
                [U(POSITION, 1), U(OVERFLOW_Y, 3)].as_slice(),
                &width(200.0),
                &height(80.0),
            ]
            .concat(),
            vec![
                B {
                    scroll: Some(1),
                    scroll_threshold: 10.0,
                    ..css("List", [], vec![css("Tail", height(240.0), vec![])])
                },
                css(
                    "Track",
                    [
                        [
                            U(POSITION, 2),
                            U(INSET_LEFT_KIND, 1),
                            F(INSET_LEFT, 190.0),
                            U(INSET_TOP_KIND, 1),
                            F(INSET_TOP, 0.0),
                        ]
                        .as_slice(),
                        &width(10.0),
                        &height(80.0),
                    ]
                    .concat(),
                    vec![B {
                        scroll_bar: Some(1),
                        ..css("Thumb", [width(10.0), height(20.0)].concat(), vec![])
                    }],
                ),
            ],
        ),
    );
    let machine = f.machine();
    press_and_drag(&machine, (195.0, 5.0), &[(195.0, 20.0)]);
    assert_eq!(f.scroll_offset("List"), (0.0, -40.0));
}

/// Nested scroll boxes: the inner one takes a drag along its axis before
/// the outer one, and a drag across it goes to the outer one.
#[test]
fn css_inner_scroll_box_takes_its_drag_before_the_outer_one() {
    let nested = |inner_direction: u64| {
        let inner_list = if inner_direction == 0 {
            [width(400.0), height(40.0)].concat()
        } else {
            [width(200.0), height(100.0)].concat()
        };
        scene(
            (400.0, 300.0),
            css(
                "Viewport",
                [[U(OVERFLOW_Y, 3)].as_slice(), &width(200.0), &height(80.0)].concat(),
                vec![B {
                    scroll: Some(1),
                    scroll_threshold: 10.0,
                    ..css(
                        "List",
                        [],
                        vec![
                            css(
                                "Inner",
                                [
                                    [U(
                                        if inner_direction == 0 {
                                            OVERFLOW_X
                                        } else {
                                            OVERFLOW_Y
                                        },
                                        3,
                                    )]
                                    .as_slice(),
                                    &width(200.0),
                                    &height(40.0),
                                ]
                                .concat(),
                                vec![B {
                                    scroll: Some(inner_direction),
                                    scroll_threshold: 10.0,
                                    ..css("Inner list", inner_list, vec![])
                                }],
                            ),
                            css("Tail", height(200.0), vec![]),
                        ],
                    )
                }],
            ),
        )
    };
    let down = [(100.0, 25.0), (100.0, 15.0), (100.0, 5.0)];
    let f = nested(1);
    press_and_drag(&f.machine(), (100.0, 35.0), &down);
    assert_eq!(f.scroll_offset("Inner list"), (0.0, -30.0), "the inner box");
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0), "the outer box");
    let f = nested(0);
    press_and_drag(&f.machine(), (100.0, 35.0), &down);
    assert_eq!(
        f.scroll_offset("Inner list"),
        (0.0, 0.0),
        "across the inner box"
    );
    assert_eq!(f.scroll_offset("List"), (0.0, -30.0));
    let f = nested(0);
    press_and_drag(
        &f.machine(),
        (150.0, 20.0),
        &[(140.0, 20.0), (130.0, 20.0), (120.0, 20.0)],
    );
    assert_eq!(
        f.scroll_offset("Inner list"),
        (-30.0, 0.0),
        "along the inner box"
    );
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0));
}

/// A box's drag listener is a drag handler too: a focused field inside it
/// keeps its selecting drag, while a drag beside the field fires it.
#[test]
fn css_drag_listener_leaves_a_focused_fields_drag_alone() {
    let f = scene(
        (400.0, 300.0),
        B {
            drag_listener: Some("card dragged"),
            ..css(
                "Card",
                [width(200.0), height(40.0)].concat(),
                vec![field_box(true, None, "hello world")],
            )
        },
    );
    let machine = f.machine();
    press_and_drag(&machine, (90.0, 20.0), &[]);
    assert!(f.field_is_focused());
    assert_eq!(
        press_and_drag(&machine, (1.0, 20.0), &[(30.0, 20.0), (60.0, 20.0)]),
        Vec::<String>::new(),
        "a drag in the field"
    );
    assert!(!f.field_selection().is_empty());
    assert_eq!(
        press_and_drag(&machine, (190.0, 20.0), &[(195.0, 20.0)]),
        ["card dragged"],
        "a drag beside the field"
    );
}

/// An event bubbles to the CSS boxes a hit is in whatever their own
/// geometry: a field placed outside its box still fires the box's click.
#[test]
fn css_box_hears_a_click_on_a_field_outside_it() {
    let f = scene(
        (400.0, 300.0),
        B {
            listener: Some("card clicked"),
            ..css(
                "Card",
                [[U(POSITION, 1)].as_slice(), &width(100.0), &height(40.0)].concat(),
                vec![B {
                    text_input: true,
                    ..css(
                        "Input box",
                        [
                            [
                                U(POSITION, 2),
                                U(INSET_LEFT_KIND, 1),
                                F(INSET_LEFT, 120.0),
                                U(INSET_TOP_KIND, 1),
                                F(INSET_TOP, 0.0),
                            ]
                            .as_slice(),
                            &width(80.0),
                            &height(40.0),
                        ]
                        .concat(),
                        vec![],
                    )
                }],
            )
        },
    );
    let machine = f.machine();
    assert_eq!(
        click(&machine, 160.0, 20.0),
        (true, vec!["card clicked".to_owned()])
    );
    assert!(f.field_is_focused());
}

/// Translated content keeps Rive's gestures, as runtime main has them for
/// a scroll view: a drag on an unfocused field selects in it and focuses it
/// without scrolling, a drag on a button scrolls and still clicks it on
/// release, and a drag in a focused field selects without scrolling.
#[test]
fn translated_scroll_view_keeps_rives_gestures() {
    let f = translated_scroll_view(field_box(false, None, "hello world"));
    let machine = f.machine();
    press_and_drag(
        &machine,
        (1.0, 35.0),
        &[(30.0, 25.0), (60.0, 15.0), (70.0, 5.0)],
    );
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0), "an unfocused field");
    assert!(f.field_is_focused());
    assert!(!f.field_selection().is_empty());
    press_and_drag(
        &machine,
        (1.0, 35.0),
        &[(30.0, 25.0), (60.0, 15.0), (70.0, 5.0)],
    );
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0), "a focused field");
    let f = translated_scroll_view(B {
        name: "Button",
        component: vec![
            F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 180.0),
            F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0),
        ],
        listener: Some("button clicked"),
        ..B::default()
    });
    let machine = f.machine();
    assert_eq!(
        press_and_drag(&machine, (90.0, 30.0), &[(90.0, 15.0)]),
        ["button clicked"],
        "a button"
    );
    assert_eq!(f.scroll_offset("List"), (0.0, -15.0));
}

/// A drag a nested artboard's handler takes is not the host's: a field in
/// one, which takes its press, keeps the drag from the CSS scroll box
/// around the nested artboard, while a drag beside the field scrolls it.
#[test]
fn css_scroll_box_leaves_a_drag_a_nested_artboard_takes() {
    let holder = || B {
        nested_field: true,
        ..css("Holder", [width(200.0), height(40.0)].concat(), vec![])
    };
    let f = css_scroll_view(Some("viewport clicked"), holder());
    press_and_drag(
        &f.machine(),
        (90.0, 35.0),
        &[(90.0, 25.0), (90.0, 15.0), (90.0, 5.0)],
    );
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0), "on the nested field");
    let f = css_scroll_view(Some("viewport clicked"), holder());
    press_and_drag(
        &f.machine(),
        (190.0, 35.0),
        &[(190.0, 25.0), (190.0, 15.0), (190.0, 5.0)],
    );
    assert_eq!(f.scroll_offset("List"), (0.0, -30.0), "beside it");
}

/// A CSS box is hovered and pressed through a child outside it, as a
/// browser's ancestors are: a child that only clicks keeps its box's enter
/// listener hovered as the pointer moves over it, and a child that answers
/// no button still lets the box answer the host's button query.
#[test]
fn css_box_is_hovered_and_pressed_through_a_child_outside_it() {
    let outside = |child: B| {
        scene(
            (400.0, 300.0),
            B {
                listener: Some("card clicked"),
                enter_listener: Some("card entered"),
                ..css(
                    "Card",
                    [[U(POSITION, 1)].as_slice(), &width(100.0), &height(40.0)].concat(),
                    vec![child],
                )
            },
        )
    };
    let placed = |name: &'static str| {
        css(
            name,
            [
                [
                    U(POSITION, 2),
                    U(INSET_LEFT_KIND, 1),
                    F(INSET_LEFT, 120.0),
                    U(INSET_TOP_KIND, 1),
                    F(INSET_TOP, 0.0),
                ]
                .as_slice(),
                &width(80.0),
                &height(40.0),
            ]
            .concat(),
            vec![],
        )
    };
    let f = outside(B {
        listener: Some("pop clicked"),
        ..placed("Pop")
    });
    let machine = f.machine();
    let moved = |x: f32| {
        use crate::source::math::vec2d::Vec2D;
        let before = machine.with_instance(|machine| machine.reported_event_count());
        machine.with_instance_mut(|machine| machine.pointer_move(Vec2D::new(x, 20.0), 0.0, 0));
        machine.with_instance(|machine| machine.reported_event_count()) - before
    };
    assert_eq!(moved(160.0), 1, "the card's enter");
    assert_eq!(moved(165.0), 0, "still hovered");
    assert_eq!(
        click(&machine, 165.0, 20.0),
        (
            true,
            vec!["pop clicked".to_owned(), "card clicked".to_owned()]
        )
    );
    let f = outside(B {
        enter_listener: Some("badge entered"),
        ..placed("Badge")
    });
    let machine = f.machine();
    assert!(machine.with_instance_mut(|machine| {
        machine.listens_to_button_at(
            crate::source::math::vec2d::Vec2D::new(160.0, 20.0),
            crate::source::pointer_button::PointerButton::Primary,
        )
    }));
}

/// Once a host's CSS scroll box takes a drag, the drag handlers inside a
/// nested artboard under it let it go: a drag that scrolls the box
/// vertically does not also scroll a nested horizontal list when it turns
/// sideways.
#[test]
fn css_scroll_box_takes_a_drag_from_a_nested_artboard_under_it() {
    let f = css_scroll_view(
        None,
        B {
            nested_scroller: true,
            ..css("Holder", [width(200.0), height(40.0)].concat(), vec![])
        },
    );
    press_and_drag(
        &f.machine(),
        (100.0, 35.0),
        &[(100.0, 25.0), (100.0, 15.0), (60.0, 15.0), (20.0, 15.0)],
    );
    assert_eq!(f.scroll_offset("List"), (0.0, -20.0), "the box");
    assert_eq!(f.nested_scroll_offset(), (0.0, 0.0), "the nested list");
    let f = css_scroll_view(
        None,
        B {
            nested_scroller: true,
            ..css("Holder", [width(200.0), height(40.0)].concat(), vec![])
        },
    );
    press_and_drag(&f.machine(), (100.0, 20.0), &[(60.0, 20.0), (20.0, 20.0)]);
    assert_eq!(f.scroll_offset("List"), (0.0, 0.0), "a sideways drag");
    assert!(f.nested_scroll_offset().0 < 0.0, "scrolls the nested list");
}

/// A field's tap belongs to its own pointer: a second finger that starts a
/// scroll elsewhere does not cancel the first finger's tap on the field.
#[test]
fn css_field_tap_survives_another_pointers_scroll() {
    use crate::source::{math::vec2d::Vec2D, pointer_button::PointerButton};
    let f = css_scroll_view(None, field_box(true, None, "hello world"));
    let machine = f.machine();
    machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(90.0, 20.0), 0, PointerButton::Primary);
        machine.pointer_move(Vec2D::new(100.0, 60.0), 0.0, 1);
        machine.pointer_down(Vec2D::new(100.0, 60.0), 1, PointerButton::Primary);
        machine.pointer_move(Vec2D::new(100.0, 45.0), 1.0 / 60.0, 1);
    });
    assert_eq!(
        f.scroll_offset("List"),
        (0.0, -15.0),
        "the second finger scrolls"
    );
    machine.with_instance_mut(|machine| {
        machine.pointer_up(Vec2D::new(90.0, 20.0), 0, PointerButton::Primary);
    });
    assert!(f.field_is_focused());
}

/// A press bubbles from a child outside its box both ways: a child that
/// hears only the press going down still carries the release to its box's
/// click.
#[test]
fn css_box_clicks_through_a_child_that_hears_only_the_press() {
    let f = scene(
        (400.0, 300.0),
        B {
            listener: Some("card clicked"),
            ..css(
                "Card",
                [[U(POSITION, 1)].as_slice(), &width(100.0), &height(40.0)].concat(),
                vec![B {
                    down_listener: Some("pop pressed"),
                    ..css(
                        "Pop",
                        [
                            [
                                U(POSITION, 2),
                                U(INSET_LEFT_KIND, 1),
                                F(INSET_LEFT, 120.0),
                                U(INSET_TOP_KIND, 1),
                                F(INSET_TOP, 0.0),
                            ]
                            .as_slice(),
                            &width(80.0),
                            &height(40.0),
                        ]
                        .concat(),
                        vec![],
                    )
                }],
            )
        },
    );
    assert_eq!(
        press_and_drag(&f.machine(), (160.0, 20.0), &[]),
        ["pop pressed", "card clicked"]
    );
}

// The compiler puts container layout on the movable content box and padding
// on its block viewport. Percentages keep the viewport's content-box basis.
fn overflow_viewport(name: &'static str, props: Vec<P>, children: Vec<B>) -> B {
    let container_key = |key| {
        matches!(
            key,
            DISPLAY
                | DIRECTION
                | 61006
                | 61007
                | 61013
                | 61015
                | 61017
                | 61018
                | 61075
                | 61076
                | 61077
                | 61078
        )
    };
    let key = |p: &P| match *p {
        U(key, _) | F(key, _) => key,
    };
    let mut content: Vec<P> = props
        .iter()
        .copied()
        .filter(|p| container_key(key(p)))
        .collect();
    content.extend([
        U(WIDTH_KIND, 7),
        F(WIDTH_PERCENT, 100.0),
        U(HEIGHT_KIND, 7),
        F(61026, 100.0),
    ]);
    let viewport = props
        .into_iter()
        .filter(|p| !container_key(key(p)) || key(p) == DIRECTION)
        .collect::<Vec<_>>();
    css(
        name,
        viewport,
        vec![B {
            scroll: Some(2),
            ..css("Scroll content", content, children)
        }],
    )
}

fn assert_chrome_range(f: &Scene, expected: (f32, f32)) {
    let offsets = f.max_offsets();
    // Chrome exposes integer scroll dimensions; Taffy keeps fractional layout.
    assert!(
        (-offsets.0 - expected.0).abs() < 0.5,
        "horizontal: {offsets:?}, Chrome {expected:?}"
    );
    assert!(
        (-offsets.1 - expected.1).abs() < 0.5,
        "vertical: {offsets:?}, Chrome {expected:?}"
    );
}

#[test]
fn css_scroll_overflow_flex_row_includes_final_margin_and_padding() {
    let f = scene(
        (400.0, 300.0),
        overflow_viewport(
            "Viewport",
            [
                width(80.0).to_vec(),
                height(100.0).to_vec(),
                vec![
                    U(DISPLAY, 1),
                    U(OVERFLOW_X, 3),
                    F(PADDING_LEFT, 5.0),
                    F(PADDING_RIGHT, 11.0),
                ],
            ]
            .concat(),
            vec![css(
                "Item",
                [
                    width(120.0).to_vec(),
                    height(20.0).to_vec(),
                    vec![F(FLEX_SHRINK, 0.0), F(61043, 17.0)],
                ]
                .concat(),
                vec![],
            )],
        ),
    );
    assert_eq!(f.max_offsets(), (-57.0, 0.0));
}

#[test]
fn css_scroll_overflow_flex_column_and_grid_include_final_margin_and_padding() {
    for display in [1, 2] {
        let f = scene(
            (400.0, 300.0),
            overflow_viewport(
                "Viewport",
                [
                    width(100.0).to_vec(),
                    height(80.0).to_vec(),
                    vec![
                        U(DISPLAY, display),
                        U(61006, 2),
                        U(OVERFLOW_Y, 3),
                        F(PADDING_TOP, 5.0),
                        F(PADDING_BOTTOM, 11.0),
                    ],
                ]
                .concat(),
                vec![css(
                    "Item",
                    [
                        height(120.0).to_vec(),
                        vec![F(FLEX_SHRINK, 0.0), F(MARGIN_BOTTOM, 17.0)],
                    ]
                    .concat(),
                    vec![],
                )],
            ),
        );
        // The content box stays at the viewport's content-box size.
        assert_eq!(f.bounds("Scroll content")[2..], [100.0, 80.0]);
        assert_eq!(f.max_offsets(), (0.0, -57.0), "display {display}");
    }
}

#[test]
fn css_scroll_overflow_taffy_fork_counts_the_last_items_margin() {
    use taffy::prelude::*;
    use taffy::{Overflow, Point};

    // Guards the local Taffy fork: stock 0.15.0 gives 40 for each of these
    // trees, dropping the item's 17-point bottom margin from the range.
    for display in [Display::Flex, Display::Grid, Display::Block] {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        tree.disable_rounding();
        let item = tree
            .new_leaf(Style {
                size: Size {
                    width: length(20.0),
                    height: length(120.0),
                },
                flex_shrink: 0.0,
                margin: Rect {
                    bottom: length(17.0),
                    ..Rect::zero()
                },
                ..Style::default()
            })
            .unwrap();
        let root = tree
            .new_with_children(
                Style {
                    display,
                    flex_direction: FlexDirection::Column,
                    size: Size {
                        width: length(100.0),
                        height: length(96.0),
                    },
                    overflow: Point {
                        x: Overflow::Scroll,
                        y: Overflow::Scroll,
                    },
                    padding: Rect {
                        top: length(5.0),
                        bottom: length(11.0),
                        ..Rect::zero()
                    },
                    ..Style::default()
                },
                &[item],
            )
            .unwrap();
        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
        assert_eq!(
            tree.layout(root).unwrap().scroll_height(),
            57.0,
            "the Taffy fork's margin patch is missing for {display:?}"
        );
    }
}

#[test]
fn css_scroll_overflow_descendants_do_not_gain_the_items_margin() {
    // Chrome reads 120: the descendant's 200 points, not the item's margin
    // added after them. No padding, which Chrome adds after in-flow items only.
    for display in [0, 1, 2] {
        let f = scene(
            (400.0, 300.0),
            overflow_viewport(
                "Viewport",
                [
                    width(100.0).to_vec(),
                    height(80.0).to_vec(),
                    vec![U(DISPLAY, display), U(61006, 2), U(OVERFLOW_Y, 3)],
                ]
                .concat(),
                vec![css(
                    "Item",
                    [
                        height(120.0).to_vec(),
                        vec![
                            U(DISPLAY, 1),
                            U(61006, 2),
                            F(FLEX_SHRINK, 0.0),
                            F(MARGIN_BOTTOM, 17.0),
                        ],
                    ]
                    .concat(),
                    vec![css(
                        "Descendant",
                        [
                            width(20.0).to_vec(),
                            height(200.0).to_vec(),
                            vec![F(FLEX_SHRINK, 0.0)],
                        ]
                        .concat(),
                        vec![],
                    )],
                )],
            ),
        );
        assert_eq!(f.max_offsets(), (0.0, -120.0), "display {display}");
    }
}

#[test]
fn css_scroll_overflow_c045_flex_item_margin_box() {
    for (w, x) in [(360.0, 132.0), (402.0, 128.0), (520.0, 116.0)] {
        let f = scene(
            (w, 800.0),
            css(
                "a",
                vec![
                    U(WIDTH_KIND, 7),
                    F(WIDTH_PERCENT, 10.0),
                    U(BOX_SIZING, 1),
                    F(PADDING_TOP, 16.0),
                ],
                vec![
                    css(
                        "b",
                        [
                            width(20.0).to_vec(),
                            height(90.0).to_vec(),
                            if w < 400.0 {
                                padding(24.0).to_vec()
                            } else {
                                vec![
                                    F(PADDING_TOP, 14.0),
                                    F(PADDING_BOTTOM, 14.0),
                                    F(61066, 5.0),
                                    F(61070, 5.0),
                                ]
                            },
                        ]
                        .concat(),
                        vec![],
                    ),
                    css(
                        "c",
                        [U(DISPLAY, 4)],
                        vec![css(
                            "d",
                            vec![
                                U(DISPLAY, if w < 400.0 { 3 } else { 0 }),
                                U(WIDTH_KIND, 7),
                                F(WIDTH_PERCENT, 30.0),
                            ],
                            vec![],
                        )],
                    ),
                    css(
                        "e",
                        [
                            width(170.0).to_vec(),
                            vec![
                                U(HEIGHT_KIND, 7),
                                F(61026, 40.0),
                                U(61030, 1),
                                F(61031, 80.0),
                                U(BOX_SIZING, 1),
                            ],
                        ]
                        .concat(),
                        vec![],
                    ),
                    overflow_viewport(
                        "f",
                        [
                            height(120.0).to_vec(),
                            vec![
                                U(DISPLAY, 1),
                                U(BOX_SIZING, 1),
                                F(MARGIN_TOP, 2.0),
                                F(61043, -8.0),
                                F(MARGIN_BOTTOM, -8.0),
                                F(MARGIN_LEFT, -8.0),
                                U(OVERFLOW_Y, 4),
                            ],
                        ]
                        .concat(),
                        vec![css(
                            "g",
                            [
                                width(150.0).to_vec(),
                                height(260.0).to_vec(),
                                vec![
                                    F(PADDING_TOP, 2.0),
                                    F(PADDING_BOTTOM, 2.0),
                                    F(PADDING_LEFT, 12.0),
                                    F(PADDING_RIGHT, 12.0),
                                    F(BORDER_TOP, 3.0),
                                    F(BORDER_RIGHT, 3.0),
                                    F(BORDER_BOTTOM, 3.0),
                                    F(BORDER_LEFT, 3.0),
                                    F(61041, 3.0),
                                    F(61043, 2.0),
                                    F(MARGIN_BOTTOM, 24.0),
                                    F(MARGIN_LEFT, 2.0),
                                    F(FLEX_SHRINK, 0.0),
                                ],
                            ]
                            .concat(),
                            vec![],
                        )],
                    ),
                ],
            ),
        );
        assert_chrome_range(&f, (x, 176.0));
    }
}

#[test]
fn css_scroll_overflow_c147_grid_final_margin() {
    for (w, y) in [(360.0, 594.0), (402.0, 494.0), (520.0, 494.0)] {
        // A page box stands in for the body, so `a` keeps its 380 points
        // when the page is narrower, as it does in Chrome.
        let f = scene(
            (w, 800.0),
            css(
                "Page",
                vec![],
                vec![overflow_viewport(
                    "a",
                    [
                        width(380.0).to_vec(),
                        height(60.0).to_vec(),
                        vec![
                            U(DISPLAY, 2),
                            F(61075, 8.0),
                            F(COLUMN_GAP, 2.0),
                            U(POSITION, 1),
                            U(OVERFLOW_Y, 3),
                        ],
                    ]
                    .concat(),
                    vec![
                        css(
                            "b",
                            [
                                U(DISPLAY, 1),
                                U(61006, 2),
                                U(61018, 9),
                                F(61075, 16.0),
                                F(COLUMN_GAP, 4.0),
                                U(POSITION, 1),
                                U(INSET_TOP_KIND, 1),
                                F(INSET_TOP, -10.0),
                                U(61057, 1),
                                F(61058, 30.0),
                            ],
                            vec![css(
                                "c",
                                [
                                    width(70.0).to_vec(),
                                    height(50.0).to_vec(),
                                    vec![
                                        U(61014, 5),
                                        F(MARGIN_TOP, -2.0),
                                        F(61043, 8.0),
                                        F(MARGIN_BOTTOM, 10.0),
                                        F(MARGIN_LEFT, 6.0),
                                    ],
                                ]
                                .concat(),
                                vec![],
                            )],
                        ),
                        css(
                            "d",
                            [
                                width(110.0).to_vec(),
                                height(if w < 380.0 { 140.0 } else { 40.0 }).to_vec(),
                                vec![U(61014, 2)],
                            ]
                            .concat(),
                            vec![],
                        ),
                        css(
                            "e",
                            [
                                width(180.0).to_vec(),
                                height(220.0).to_vec(),
                                vec![
                                    F(PADDING_TOP, 12.0),
                                    F(PADDING_RIGHT, 16.0),
                                    F(PADDING_LEFT, 16.0),
                                    F(61068, 6.0),
                                    F(MARGIN_TOP, 22.0),
                                    F(61043, 22.0),
                                    F(MARGIN_BOTTOM, 22.0),
                                    F(MARGIN_LEFT, 22.0),
                                    U(POSITION, 1),
                                    U(INSET_LEFT_KIND, 1),
                                    F(INSET_LEFT, 0.0),
                                    U(61054, 1),
                                    F(61055, 20.0),
                                ],
                            ]
                            .concat(),
                            vec![],
                        ),
                        css(
                            "f",
                            [
                                width(60.0).to_vec(),
                                height(90.0).to_vec(),
                                vec![
                                    F(61064, 5.0),
                                    F(PADDING_RIGHT, 2.0),
                                    F(PADDING_LEFT, 2.0),
                                    F(PADDING_BOTTOM, 20.0),
                                    F(MARGIN_TOP, 2.0),
                                    F(MARGIN_BOTTOM, 2.0),
                                    F(61043, 10.0),
                                    F(MARGIN_LEFT, 10.0),
                                ],
                            ]
                            .concat(),
                            vec![],
                        ),
                    ],
                )],
            ),
        );
        assert_chrome_range(&f, (0.0, y));
    }
}

#[test]
fn css_scroll_overflow_c297_flex_margin_and_viewport_padding() {
    for (w, x) in [(360.0, 41.0), (402.0, 12.0), (520.0, 0.0)] {
        let f = scene(
            (w, 800.0),
            overflow_viewport(
                "a",
                [
                    height(160.0).to_vec(),
                    padding(8.0).to_vec(),
                    vec![
                        U(DISPLAY, 1),
                        U(WIDTH_KIND, 7),
                        F(WIDTH_PERCENT, 70.0),
                        U(BOX_SIZING, 1),
                        F(BORDER_TOP, 7.0),
                        U(OVERFLOW_Y, 4),
                    ],
                ]
                .concat(),
                vec![
                    css(
                        "b",
                        [
                            width(50.0).to_vec(),
                            height(50.0).to_vec(),
                            padding(16.0).to_vec(),
                            vec![F(FLEX_SHRINK, 0.0), U(BOX_SIZING, 1)],
                            if w >= 500.0 {
                                vec![
                                    F(MARGIN_TOP, 20.0),
                                    F(MARGIN_BOTTOM, 20.0),
                                    F(MARGIN_LEFT, 14.0),
                                    F(61043, 14.0),
                                ]
                            } else {
                                vec![]
                            },
                        ]
                        .concat(),
                        vec![],
                    ),
                    css(
                        "c",
                        [
                            width(160.0).to_vec(),
                            height(400.0).to_vec(),
                            vec![
                                U(61030, 1),
                                F(61031, 220.0),
                                F(BORDER_TOP, 2.0),
                                F(BORDER_RIGHT, 3.0),
                                F(BORDER_BOTTOM, 5.0),
                                F(MARGIN_TOP, 14.0),
                                F(MARGIN_BOTTOM, 14.0),
                                F(MARGIN_LEFT, 20.0),
                                F(61043, 20.0),
                                F(FLEX_SHRINK, 0.0),
                            ],
                        ]
                        .concat(),
                        vec![],
                    ),
                    css(
                        "d",
                        [
                            height(60.0).to_vec(),
                            vec![
                                U(WIDTH_KIND, 7),
                                F(WIDTH_PERCENT, 70.0),
                                U(BOX_SIZING, 1),
                                F(PADDING_TOP, 8.0),
                                F(PADDING_BOTTOM, 8.0),
                                F(PADDING_LEFT, 12.0),
                                F(PADDING_RIGHT, 12.0),
                            ],
                            if w >= 500.0 {
                                vec![
                                    F(MARGIN_TOP, 8.0),
                                    F(MARGIN_BOTTOM, 8.0),
                                    F(MARGIN_LEFT, 8.0),
                                    F(61043, 8.0),
                                ]
                            } else {
                                vec![]
                            },
                        ]
                        .concat(),
                        vec![],
                    ),
                ],
            ),
        );
        assert_chrome_range(&f, (x, 298.0));
    }
}

#[test]
fn translated_scroll_overflow_keeps_content_bounds_range() {
    let f = scene(
        (400.0, 300.0),
        B {
            name: "Viewport",
            component: vec![
                F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0),
                F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 80.0),
            ],
            style: vec![F(
                LayoutComponentStyleBase::PADDING_BOTTOM_PROPERTY_KEY,
                11.0,
            )],
            children: vec![B {
                name: "Content",
                scroll: Some(1),
                component: vec![
                    F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0),
                    F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 120.0),
                ],
                children: vec![B {
                    name: "Overflow",
                    component: vec![
                        F(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0),
                        F(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 300.0),
                    ],
                    style: vec![F(
                        LayoutComponentStyleBase::MARGIN_BOTTOM_PROPERTY_KEY,
                        17.0,
                    )],
                    ..B::default()
                }],
                ..B::default()
            }],
            ..B::default()
        },
    );
    assert_eq!(f.max_offsets(), (0.0, -51.0));
}

#[test]
fn css_scroll_overflow_rtl_includes_end_margin_and_padding() {
    let f = scene(
        (400.0, 300.0),
        overflow_viewport(
            "Viewport",
            [
                width(80.0).to_vec(),
                height(100.0).to_vec(),
                vec![
                    U(DISPLAY, 1),
                    U(DIRECTION, 2),
                    U(OVERFLOW_X, 3),
                    F(PADDING_RIGHT, 5.0),
                    F(PADDING_LEFT, 11.0),
                ],
            ]
            .concat(),
            vec![css(
                "Item",
                [
                    width(120.0).to_vec(),
                    height(20.0).to_vec(),
                    vec![F(FLEX_SHRINK, 0.0), F(MARGIN_LEFT, 17.0)],
                ]
                .concat(),
                vec![],
            )],
        ),
    );
    assert_eq!(f.max_offsets(), (-57.0, 0.0));
}

// A block scroller with 5 and 11 points of padding, a 120-point item and a
// 17-point end margin: Chrome 156 reads 57 for each of these.
fn block_scroller(items: Vec<B>) -> Scene {
    scene(
        (400.0, 300.0),
        overflow_viewport(
            "Viewport",
            [
                width(100.0).to_vec(),
                height(80.0).to_vec(),
                vec![
                    U(DISPLAY, 0),
                    U(OVERFLOW_Y, 3),
                    F(PADDING_TOP, 5.0),
                    F(PADDING_BOTTOM, 11.0),
                ],
            ]
            .concat(),
            items,
        ),
    )
}

#[test]
fn css_scroll_overflow_block_child_includes_its_end_margin() {
    let f = block_scroller(vec![css(
        "Item",
        [height(120.0).to_vec(), vec![F(MARGIN_BOTTOM, 17.0)]].concat(),
        vec![],
    )]);
    assert_eq!(f.max_offsets(), (0.0, -57.0));
}

#[test]
fn css_scroll_overflow_block_includes_a_margin_collapsed_through_a_child() {
    let f = block_scroller(vec![css(
        "Wrapper",
        vec![],
        vec![css(
            "Item",
            [height(120.0).to_vec(), vec![F(MARGIN_BOTTOM, 17.0)]].concat(),
            vec![],
        )],
    )]);
    assert_eq!(f.max_offsets(), (0.0, -57.0));
    // Padding stops the collapse; the margin then sits inside the wrapper.
    let f = block_scroller(vec![css(
        "Wrapper",
        vec![F(PADDING_BOTTOM, 1.0)],
        vec![css(
            "Item",
            [height(120.0).to_vec(), vec![F(MARGIN_BOTTOM, 17.0)]].concat(),
            vec![],
        )],
    )]);
    assert_eq!(f.max_offsets(), (0.0, -58.0));
}

#[test]
fn css_scroll_overflow_empty_last_item_keeps_its_margin() {
    let items = || {
        vec![
            css(
                "Item",
                [height(120.0).to_vec(), vec![F(FLEX_SHRINK, 0.0)]].concat(),
                vec![],
            ),
            css(
                "Spacer",
                vec![F(FLEX_SHRINK, 0.0), F(MARGIN_BOTTOM, 17.0)],
                vec![],
            ),
        ]
    };
    assert_eq!(block_scroller(items()).max_offsets(), (0.0, -57.0));
    let f = scene(
        (400.0, 300.0),
        overflow_viewport(
            "Viewport",
            [
                width(100.0).to_vec(),
                height(80.0).to_vec(),
                vec![
                    U(DISPLAY, 1),
                    U(61006, 2),
                    U(OVERFLOW_Y, 3),
                    F(PADDING_TOP, 5.0),
                    F(PADDING_BOTTOM, 11.0),
                ],
            ]
            .concat(),
            items(),
        ),
    );
    assert_eq!(f.max_offsets(), (0.0, -57.0));
}

#[test]
fn css_scroll_overflow_block_row_includes_the_end_margin() {
    let f = scene(
        (400.0, 300.0),
        overflow_viewport(
            "Viewport",
            [
                width(80.0).to_vec(),
                height(100.0).to_vec(),
                vec![
                    U(DISPLAY, 0),
                    U(OVERFLOW_X, 3),
                    F(PADDING_LEFT, 5.0),
                    F(PADDING_RIGHT, 11.0),
                ],
            ]
            .concat(),
            vec![css(
                "Item",
                [
                    width(120.0).to_vec(),
                    height(20.0).to_vec(),
                    vec![F(61043, 17.0)],
                ]
                .concat(),
                vec![],
            )],
        ),
    );
    assert_eq!(f.max_offsets(), (-57.0, 0.0));
}

// A 100 by 80 scroller without padding, as in Chrome 156's measurements.
fn plain_scroller(display: u64, items: Vec<B>) -> Scene {
    scene(
        (400.0, 300.0),
        overflow_viewport(
            "Viewport",
            [
                width(100.0).to_vec(),
                height(80.0).to_vec(),
                vec![U(DISPLAY, display), U(61006, 2), U(OVERFLOW_Y, 3)],
            ]
            .concat(),
            items,
        ),
    )
}

/// Chrome counts a relative item's margin box at its in-flow position and
/// its shifted border box on its own: 120 points, `top: 30px` and a 17-point
/// bottom margin read 70, and `top: -30px` reads 57.
#[test]
fn css_scroll_overflow_relative_items_count_their_margin_before_the_shift() {
    for (display, top, range) in [
        (1, 30.0, 70.0),
        (2, 30.0, 70.0),
        (0, 30.0, 70.0),
        (1, -30.0, 57.0),
    ] {
        let f = plain_scroller(
            display,
            vec![css(
                "Item",
                [
                    height(120.0).to_vec(),
                    vec![
                        F(FLEX_SHRINK, 0.0),
                        U(POSITION, 1),
                        U(INSET_TOP_KIND, 1),
                        F(INSET_TOP, top),
                        F(MARGIN_BOTTOM, 17.0),
                    ],
                ]
                .concat(),
                vec![],
            )],
        );
        assert_eq!(
            f.max_offsets(),
            (0.0, -range),
            "display {display}, top {top}"
        );
    }
}

/// An empty block's margins collapse with its neighbours', so its own
/// margin box does not count. Chrome 156 reads these after a 120-point block.
#[test]
fn css_scroll_overflow_self_collapsing_spacers_count_their_collapsed_margin() {
    let spacer = |top: f32, bottom: f32| {
        css(
            "Spacer",
            vec![
                F(FLEX_SHRINK, 0.0),
                F(MARGIN_TOP, top),
                F(MARGIN_BOTTOM, bottom),
            ],
            vec![],
        )
    };
    let block = |name: &'static str, height_value: f32, margin_top: f32| {
        css(
            name,
            [
                height(height_value).to_vec(),
                vec![F(FLEX_SHRINK, 0.0), F(MARGIN_TOP, margin_top)],
            ]
            .concat(),
            vec![],
        )
    };
    for (display, items, range) in [
        // Between blocks: the last block ends at 138.
        (
            0,
            vec![
                block("Item", 120.0, 0.0),
                spacer(10.0, 17.0),
                block("Last", 1.0, 0.0),
            ],
            58.0,
        ),
        // Last: the flow ends after the collapsed 17 or 30.
        (0, vec![block("Item", 120.0, 0.0), spacer(10.0, 17.0)], 57.0),
        (0, vec![block("Item", 120.0, 0.0), spacer(30.0, 17.0)], 70.0),
        // Collapsing with the following block's 20-point top margin.
        (
            0,
            vec![
                block("Item", 120.0, 0.0),
                spacer(10.0, 17.0),
                block("Last", 1.0, 20.0),
            ],
            61.0,
        ),
        // Flex items' margins do not collapse.
        (1, vec![block("Item", 120.0, 0.0), spacer(10.0, 17.0)], 67.0),
    ] {
        let f = plain_scroller(display, items);
        assert_eq!(
            f.max_offsets(),
            (0.0, -range),
            "display {display}, range {range}"
        );
    }
}
