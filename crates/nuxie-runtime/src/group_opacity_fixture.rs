//! Hand-built files for an element that fades as one picture, shared by the
//! renderer's pixel and canvas tests and the runtime's hit tests. A scene is a
//! root artboard and the unit artboards its hosts name, each written in the
//! shape a test names: the compiler's shape for a box root, an empty record,
//! or no CSS record at all.
#![allow(dead_code)]
use crate::live_move_fixture::{V, css_at, css_size, obj, paint_header};

const FONT: &[u8] = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");

/// What a record says about overflow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    /// Hidden on the x axis alone, which counts as no clip for a picture's box.
    HiddenXOnly,
}

/// An outer `box-shadow`, reaching `blur + spread` points past its box.
#[derive(Clone, Copy, Debug)]
pub struct BoxShadow {
    pub color: u32,
    pub blur: f32,
    pub spread: f32,
}

/// A CSS box placed with absolute insets.
#[derive(Clone, Debug, Default)]
pub struct Block {
    pub at: (f32, f32),
    pub size: (f32, f32),
    pub color: Option<u32>,
    pub overflow: Overflow,
    pub shadow: Option<BoxShadow>,
    pub items: Vec<Item>,
}

/// A host of a unit. With `layout` it is a layout-sized host whose item record
/// carries `overflow`. Without it, a plain host at `at`.
#[derive(Clone, Debug)]
pub struct Host {
    pub name: &'static str,
    pub unit: usize,
    pub at: (f32, f32),
    pub opacity: f32,
    pub layout: Option<(f32, f32)>,
    pub overflow: Overflow,
}

impl Host {
    pub fn plain(name: &'static str, unit: usize, at: (f32, f32), opacity: f32) -> Self {
        Self {
            name,
            unit,
            at,
            opacity,
            layout: None,
            overflow: Overflow::Visible,
        }
    }
}

/// One entry of a draw list, in file order.
#[derive(Clone, Debug)]
pub enum Item {
    Block(Block),
    /// The file's picture at its natural size, its top left at `at`.
    Photo {
        at: (f32, f32),
    },
    /// A text title in the file's font, its top left at `at`.
    Title {
        at: (f32, f32),
        size: (f32, f32),
        text: &'static str,
        color: u32,
    },
    Host(Host),
    /// A Rive rectangle, with no CSS of its own. In a Rive artboard the first
    /// one written draws on top.
    Shape {
        at: (f32, f32),
        size: (f32, f32),
        color: u32,
    },
}

/// An artboard: the root, or a unit a host names.
#[derive(Clone, Debug)]
pub struct Unit {
    pub size: (f32, f32),
    /// Writes a `LayoutCssStyle` record. Without one the artboard is Rive-made.
    pub css: bool,
    pub overflow: Overflow,
    pub frame_clip: bool,
    pub background: Option<u32>,
    pub radius: f32,
    /// A solid border on every side: its width and color.
    pub border: Option<(f32, u32)>,
    pub shadow: Option<BoxShadow>,
    /// Carries an enabled `BitmapCache`.
    pub cached: bool,
    pub items: Vec<Item>,
}

impl Unit {
    /// A CSS artboard with an empty record and no frame clip.
    pub fn css(width: f32, height: f32) -> Self {
        Self {
            size: (width, height),
            css: true,
            overflow: Overflow::Visible,
            frame_clip: false,
            background: None,
            radius: 0.0,
            border: None,
            shadow: None,
            cached: false,
            items: Vec::new(),
        }
    }
    /// The compiler's shape for a box root: `clip: false` and a record whose
    /// overflow is hidden on both axes.
    pub fn compiled(width: f32, height: f32) -> Self {
        Self {
            overflow: Overflow::Hidden,
            ..Self::css(width, height)
        }
    }
    /// A Rive-made artboard: no CSS record, and a frame clip as Rive writes.
    pub fn rive(width: f32, height: f32) -> Self {
        Self {
            css: false,
            frame_clip: true,
            ..Self::css(width, height)
        }
    }
}

/// A file: `root` is its default artboard, and a host's `unit` indexes `units`.
#[derive(Clone, Debug)]
pub struct Scene {
    pub root: Unit,
    pub units: Vec<Unit>,
    /// PNG bytes for `Item::Photo`.
    pub photo: Option<Vec<u8>>,
}

impl Scene {
    /// A CSS root of `size` with one plain host of `unit` at `at`.
    pub fn hosted(size: (f32, f32), unit: Unit, at: (f32, f32), opacity: f32) -> Self {
        Self {
            root: Unit {
                items: vec![Item::Host(Host::plain("host", 0, at, opacity))],
                ..Unit::css(size.0, size.1)
            },
            units: vec![unit],
            photo: None,
        }
    }
    fn has_title(&self) -> bool {
        fn holds(items: &[Item]) -> bool {
            items.iter().any(|item| match item {
                Item::Title { .. } => true,
                Item::Block(block) => holds(&block.items),
                _ => false,
            })
        }
        std::iter::once(&self.root)
            .chain(&self.units)
            .any(|unit| holds(&unit.items))
    }
    pub fn bytes(&self) -> Vec<u8> {
        use V::*;
        let mut b = paint_header();
        obj(&mut b, "Backboard", &[]);
        let mut assets = Assets::default();
        let mut next = 0;
        if let Some(photo) = &self.photo {
            obj(&mut b, "ImageAsset", &[("assetId", U(next))]);
            obj(&mut b, "FileAssetContents", &[("bytes", Bytes(photo))]);
            assets.photo = next;
            next += 1;
        }
        if self.has_title() {
            obj(&mut b, "FontAsset", &[("assetId", U(next))]);
            obj(&mut b, "FileAssetContents", &[("bytes", Bytes(FONT))]);
            assets.font = next;
        }
        for (index, unit) in std::iter::once(&self.root).chain(&self.units).enumerate() {
            write_unit(&mut b, &format!("artboard {index}"), unit, assets);
        }
        b
    }
}

#[derive(Clone, Copy, Default)]
struct Assets {
    photo: u64,
    font: u64,
}

/// Writes one artboard's objects, numbering them as the file does.
struct Board<'a> {
    b: &'a mut Vec<u8>,
    next: u64,
    assets: Assets,
}

impl Board<'_> {
    fn add(&mut self, kind: &str, props: &[(&str, V<'_>)]) -> u64 {
        obj(self.b, kind, props);
        self.next += 1;
        self.next - 1
    }
    /// Paints in the order CSS stacks them: shadow, background, border.
    fn paints(
        &mut self,
        parent: u64,
        shadow: Option<BoxShadow>,
        color: Option<u32>,
        border: Option<u32>,
    ) {
        use V::*;
        if let Some(shadow) = shadow {
            let fill = self.add("Fill", &[("parentId", U(parent))]);
            self.add(
                "SolidColor",
                &[("parentId", U(fill)), ("colorValue", Color(shadow.color))],
            );
            let feather = self.add(
                "Feather",
                &[
                    ("parentId", U(fill)),
                    ("spaceValue", U(1)),
                    ("strength", F(shadow.blur)),
                ],
            );
            self.add(
                "BoxShadow",
                &[("parentId", U(feather)), ("spread", F(shadow.spread))],
            );
        }
        if let Some(color) = color {
            let fill = self.add("Fill", &[("parentId", U(parent))]);
            self.add(
                "SolidColor",
                &[("parentId", U(fill)), ("colorValue", Color(color))],
            );
        }
        if let Some(color) = border {
            let stroke = self.add("Stroke", &[("parentId", U(parent))]);
            self.add(
                "SolidColor",
                &[("parentId", U(stroke)), ("colorValue", Color(color))],
            );
            self.add("BoxBorder", &[("parentId", U(stroke))]);
        }
    }
    fn item(&mut self, parent: u64, css_parent: bool, item: &Item) {
        use V::*;
        match item {
            Item::Block(block) => {
                let style = self.next + 1;
                let id = self.add(
                    "LayoutComponent",
                    &[
                        ("name", S("box")),
                        ("parentId", U(parent)),
                        ("styleId", U(style)),
                    ],
                );
                self.add("LayoutComponentStyle", &[("parentId", U(id))]);
                let mut record = vec![("parentId", U(id))];
                record.extend(css_at(block.at.0, block.at.1, block.size.0, block.size.1));
                record.extend(overflow(block.overflow));
                self.add("LayoutCssStyle", &record);
                self.paints(id, block.shadow, block.color, None);
                for child in &block.items {
                    self.item(id, true, child);
                }
            }
            Item::Photo { at } => {
                let asset = self.assets.photo;
                self.add(
                    "Image",
                    &[
                        ("parentId", U(parent)),
                        ("assetId", U(asset)),
                        ("x", F(at.0)),
                        ("y", F(at.1)),
                        ("originX", F(0.0)),
                        ("originY", F(0.0)),
                    ],
                );
            }
            Item::Title {
                at,
                size,
                text,
                color,
            } => {
                let font = self.assets.font;
                let id = self.add(
                    "Text",
                    &[
                        ("parentId", U(parent)),
                        ("x", F(at.0)),
                        ("y", F(at.1)),
                        ("sizingValue", U(2)),
                        ("width", F(size.0)),
                        ("height", F(size.1)),
                        ("wrapValue", U(1)),
                    ],
                );
                let style = self.add(
                    "TextStylePaint",
                    &[
                        ("parentId", U(id)),
                        ("fontSize", F(32.0)),
                        ("fontAssetId", U(font)),
                        ("lineHeight", F(36.0)),
                    ],
                );
                let fill = self.add("Fill", &[("parentId", U(style))]);
                self.add(
                    "SolidColor",
                    &[("parentId", U(fill)), ("colorValue", Color(*color))],
                );
                self.add(
                    "TextValueRun",
                    &[
                        ("parentId", U(id)),
                        ("styleId", U(style)),
                        ("text", S(text)),
                    ],
                );
            }
            Item::Host(host) => {
                let mut props = vec![
                    ("name", S(host.name)),
                    ("parentId", U(parent)),
                    // The root is artboard 0, so unit n is artboard n + 1.
                    ("artboardId", U(host.unit as u64 + 1)),
                ];
                if host.opacity != 1.0 {
                    props.push(("opacity", F(host.opacity)));
                }
                match host.layout {
                    None => {
                        props.extend([("x", F(host.at.0)), ("y", F(host.at.1))]);
                        let id = self.add("NestedArtboard", &props);
                        if css_parent {
                            self.add("LayoutCssStyle", &[("parentId", U(id))]);
                        }
                    }
                    Some((width, height)) => {
                        let id = self.add("NestedArtboardLayout", &props);
                        let mut record = vec![("parentId", U(id))];
                        record.extend(css_size(width, height));
                        record.extend(overflow(host.overflow));
                        self.add("LayoutCssStyle", &record);
                    }
                }
            }
            Item::Shape { at, size, color } => {
                let id = self.add(
                    "Shape",
                    &[
                        ("parentId", U(parent)),
                        ("x", F(at.0 + size.0 / 2.0)),
                        ("y", F(at.1 + size.1 / 2.0)),
                    ],
                );
                self.add(
                    "Rectangle",
                    &[
                        ("parentId", U(id)),
                        ("width", F(size.0)),
                        ("height", F(size.1)),
                    ],
                );
                let fill = self.add("Fill", &[("parentId", U(id))]);
                self.add(
                    "SolidColor",
                    &[("parentId", U(fill)), ("colorValue", Color(*color))],
                );
            }
        }
    }
}

fn overflow(value: Overflow) -> Vec<(&'static str, V<'static>)> {
    match value {
        Overflow::Visible => Vec::new(),
        Overflow::Hidden => vec![("overflowX", V::U(2)), ("overflowY", V::U(2))],
        Overflow::HiddenXOnly => vec![("overflowX", V::U(2))],
    }
}

fn write_unit(b: &mut Vec<u8>, name: &str, unit: &Unit, assets: Assets) {
    use V::*;
    let mut board = Board { b, next: 0, assets };
    let mut props = vec![
        ("name", S(name)),
        ("width", F(unit.size.0)),
        ("height", F(unit.size.1)),
        ("clip", B(unit.frame_clip)),
    ];
    if unit.css {
        props.push(("styleId", U(2)));
    }
    board.add("Artboard", &props);
    if unit.css {
        let mut record = vec![("parentId", U(0))];
        record.extend(overflow(unit.overflow));
        if let Some((width, _)) = unit.border {
            record.extend([
                ("borderTop", F(width)),
                ("borderRight", F(width)),
                ("borderBottom", F(width)),
                ("borderLeft", F(width)),
            ]);
        }
        board.add("LayoutCssStyle", &record);
        let mut style = vec![("parentId", U(0))];
        if unit.radius > 0.0 {
            style.push(("cornerRadiusTL", F(unit.radius)));
        }
        board.add("LayoutComponentStyle", &style);
    }
    board.paints(
        0,
        unit.shadow,
        unit.background,
        unit.border.map(|(_, color)| color),
    );
    if unit.cached {
        board.add("BitmapCache", &[("parentId", U(0))]);
    }
    for item in &unit.items {
        board.item(0, unit.css, item);
    }
}
