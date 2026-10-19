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
    /// A data binding from the file's one view model number to this record
    /// property.
    bind: Option<u16>,
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
}

/// A paint on a box, with its CSS additions (docs/CSS_LAYOUT.md, 4).
#[derive(Clone, Copy)]
enum Paint {
    /// A Fill with a SolidColor.
    Fill(u32),
    /// A Stroke with a SolidColor and a BoxBorder over `sides`.
    Border(u32, u64),
    /// A shadow: a Fill with a SolidColor and a local Feather of
    /// `strength`, whose BoxShadow has `spread`.
    Shadow {
        color: u32,
        strength: f32,
        spread: f32,
    },
    /// A Fill with a two-stop LinearGradient whose BoxGradient has `angle`.
    Gradient(f32),
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
    Feather(f32, u64),
    LinearGradient(u64),
    Stop(u32, f32, u64),
    NamedShape(&'static str, u64),
    Bind(u16),
    /// A record property keyed by the animation: the record's object index.
    Keys(u64, u16, &'a [(u32, f32)]),
    PathAt(f32, (f32, f32), u64),
    Group(u64),
    Shape(&'a Rect, u64),
    Path(&'a Rect, u64),
    Scroll(u64, u64),
    Participant(u64),
    Track((u64, u64, f32), u64),
    Image(&'static str, u64),
    /// A Text with no runs.
    EmptyText(u64),
}

/// A box's (or the artboard's) paints and their CSS records.
fn push_paints(paints: &[Paint], index: u64, entries: &mut Vec<Entry<'_>>) {
    for paint in paints {
        let paint_index = slots(entries) + 1;
        match *paint {
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
            Paint::Shadow {
                color,
                strength,
                spread,
            } => {
                entries.push(Entry::Fill(index));
                entries.push(Entry::Solid(color, paint_index));
                entries.push(Entry::Feather(strength, paint_index));
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
}

/// Imports one artboard of `size` described by `spec`, holding `root` when
/// there is one, or None when the file is refused.
fn try_scene_spec(size: (f32, f32), spec: ArtboardSpec, root: Option<B>) -> Option<Scene> {
    let artboard_css = spec.css.clone();
    let artboard_paints = spec.paints.as_slice();
    fn walk<'a>(node: &'a B, parent: u64, entries: &mut Vec<Entry<'a>>) {
        let index = slots(entries) + 1;
        entries.push(Entry::Box(node, parent));
        if let Some(props) = &node.css {
            let record = slots(entries) + 1;
            entries.push(Entry::Record(LAYOUT_CSS_STYLE, props.clone(), index));
            if let Some(key) = node.bind {
                entries.push(Entry::Bind(key));
            }
            for (key, keys) in &node.css_keys {
                entries.push(Entry::Keys(record, *key, keys));
            }
        }
        if let Some(props) = &node.box_paint {
            let record = slots(entries) + 1;
            entries.push(Entry::Record(BOX_PAINT, props.clone(), index));
            if let Some(key) = node.box_paint_bind {
                entries.push(Entry::Bind(key));
            }
            for (key, keys) in &node.box_paint_keys {
                entries.push(Entry::Keys(record, *key, keys));
            }
        }
        push_paints(&node.paints, index, entries);
        if let Some(direction) = node.scroll {
            entries.push(Entry::Scroll(index, direction));
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
        for child in &node.children {
            walk(child, index, entries);
        }
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
        walk(root, 0, &mut entries);
    }
    // Styles follow every other object: the artboard's, then one per box.
    let first_style = slots(&entries) + 1;
    let mut riv = Riv::default();
    riv.object(BackboardBase::TYPE_KEY).end();
    let binds = entries.iter().any(|entry| matches!(entry, Entry::Bind(..)));
    if binds {
        // One view model with one number property: path [0, 0].
        use crate::source::generated::viewmodel::{
            viewmodel_base::ViewModelBase, viewmodel_component_base::ViewModelComponentBase,
            viewmodel_property_number_base::ViewModelPropertyNumberBase,
        };
        riv.object(ViewModelBase::TYPE_KEY)
            .string(ViewModelComponentBase::NAME_PROPERTY_KEY, "Model")
            .end();
        riv.object(ViewModelPropertyNumberBase::TYPE_KEY)
            .string(ViewModelComponentBase::NAME_PROPERTY_KEY, "inset")
            .end();
    }
    if entries
        .iter()
        .any(|entry| matches!(entry, Entry::Image(..)))
    {
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
    riv.object(ArtboardBase::TYPE_KEY)
        .string(ComponentBase::NAME_PROPERTY_KEY, "Artboard")
        .float(LayoutComponentBase::WIDTH_PROPERTY_KEY, size.0)
        .float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, size.1)
        .uint(LayoutComponentBase::STYLE_ID_PROPERTY_KEY, first_style);
    if spec.origin_x != 0.0 {
        riv.float(ArtboardBase::ORIGIN_X_PROPERTY_KEY, spec.origin_x);
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
            Entry::Bind(key) => {
                use crate::source::generated::data_bind::{
                    data_bind_base::DataBindBase, data_bind_context_base::DataBindContextBase,
                };
                riv.object(DataBindContextBase::TYPE_KEY)
                    .uint(DataBindBase::PROPERTY_KEY_PROPERTY_KEY, u64::from(*key))
                    .uint(DataBindBase::FLAGS_PROPERTY_KEY, 0)
                    .byte_field(DataBindContextBase::SOURCE_PATH_IDS_PROPERTY_KEY, &[0, 0])
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
            Entry::Feather(strength, parent) => {
                riv.object(FeatherBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(FeatherBase::SPACE_VALUE_PROPERTY_KEY, 1)
                    .float(FeatherBase::STRENGTH_PROPERTY_KEY, *strength)
                    .end();
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
            Entry::Scroll(parent, direction) => {
                use crate::source::generated::constraints::draggable_constraint_base::DraggableConstraintBase;
                riv.object(crate::source::generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase::TYPE_KEY)
                    .uint(ComponentBase::PARENT_ID_PROPERTY_KEY, *parent)
                    .uint(DraggableConstraintBase::DIRECTION_VALUE_PROPERTY_KEY, *direction)
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
                    instance.property_values()[0].clone()
                },
            )
            .expect("the number")
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
/// still hit anywhere in its bounds.
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
    let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(390.0, 290.0));
    assert_eq!(f.geometry_names(&hits), ["Artboard"]);
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
        let bounds = visible[0].bounds;
        assert!(bounds.max_y <= 4.0 + 1.0e-3, "radius {radius}: {bounds:?}");
        let mut selected = |x: f32, y: f32| {
            let hits = host.hit_test_segments_with_bounds(nuxie_render_api::Vec2D::new(x, y));
            f.geometry_names(&hits)
        };
        assert_eq!(selected(34.0, 2.0), ["Card"], "radius {radius}");
        assert!(selected(30.0, 33.0).is_empty(), "radius {radius}");
        assert!(tapped(&f, "Card", (30.0, 33.0)), "radius {radius}");
    }
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
