//! Island frames through the C edge, on CSS pages written by hand here.
//!
//! Oracle: each fixture's CSS. Every expected number below is worked out
//! from the boxes' sizes, insets, borders, paddings, opacities, transforms
//! and scroll offsets as the fixture writes them.
use super::*;
use nuxie_schema::definition_by_name;
use std::cell::{Cell, RefCell};
use std::ffi::c_void;

/// One property value as the file writes it.
#[derive(Clone, Copy)]
enum V {
    U(u64),
    F(f32),
    S(&'static str),
}

type Props = Vec<(&'static str, V)>;

fn var(b: &mut Vec<u8>, mut n: u64) {
    while n >= 128 {
        b.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    b.push(n as u8);
}

/// The key of `kind`'s property `name`, its own or an ancestor's.
fn key(kind: &str, name: &str) -> u16 {
    let definition = definition_by_name(kind).unwrap_or_else(|| panic!("type {kind}"));
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(definition_by_name)
        .flat_map(|definition| definition.properties)
        .find(|property| property.name == name)
        .unwrap_or_else(|| panic!("{kind}.{name}"))
        .key
        .int
}

/// One object of `kind` with `props`, then its terminator.
fn obj(b: &mut Vec<u8>, kind: &str, props: &[(&str, V)]) {
    let kind_key = definition_by_name(kind)
        .unwrap_or_else(|| panic!("type {kind}"))
        .type_key
        .int;
    var(b, kind_key.into());
    for (name, value) in props {
        var(b, key(kind, name).into());
        match value {
            V::U(n) => var(b, *n),
            V::F(v) => b.extend(v.to_le_bytes()),
            V::S(s) => {
                var(b, s.len() as u64);
                b.extend(s.as_bytes());
            }
        }
    }
    var(b, 0);
}

/// What an item is.
enum Kind {
    /// A CSS box: a `LayoutComponent` and its record.
    Box,
    /// A used component of the artboard at this index, sized by its record.
    Used(u64),
    /// A keyed list whose rows are the artboard at this index.
    Rows(u64),
    /// A text, which is not a box.
    Text,
    /// A text input with this text, named "editable".
    Input(&'static str),
    /// A closed path shape with these corners and a clipping shape on the
    /// parent that clips with it.
    Clip([(f32, f32); 4]),
}

/// One item of an artboard.
struct Item {
    name: &'static str,
    kind: Kind,
    /// Its CSS record.
    css: Props,
    /// Its own properties: opacity, rotation, scale.
    own: Props,
    /// The direction of the scroll constraint on it: 1 vertical.
    scroll: Option<(u64, f32)>,
    /// The role of the semantic data on it.
    role: Option<u64>,
    children: Vec<Item>,
}

fn item(name: &'static str, kind: Kind, css: Props, children: Vec<Item>) -> Item {
    Item {
        name,
        kind,
        css,
        own: Vec::new(),
        scroll: None,
        role: None,
        children,
    }
}

fn boxed(name: &'static str, css: Props, children: Vec<Item>) -> Item {
    item(name, Kind::Box, css, children)
}

impl Item {
    fn own(mut self, own: Props) -> Self {
        self.own = own;
        self
    }
}

fn size(width: f32, height: f32) -> Props {
    use V::*;
    vec![
        ("widthKind", U(1)),
        ("width", F(width)),
        ("heightKind", U(1)),
        ("height", F(height)),
    ]
}

/// Absolute, at (`left`, `top`) in its containing block.
fn at(left: f32, top: f32, width: f32, height: f32) -> Props {
    use V::*;
    let mut css = vec![
        ("position", U(2)),
        ("insetLeftKind", U(1)),
        ("insetLeft", F(left)),
        ("insetTopKind", U(1)),
        ("insetTop", F(top)),
    ];
    css.extend(size(width, height));
    css
}

/// Relative, filling a 400 by 600 page from its top left.
fn wrapper(more: Props) -> Props {
    let mut css = vec![("position", V::U(1))];
    css.extend(size(400.0, 600.0));
    css.extend(more);
    css
}

/// The field: `css`, then `box-sizing: border-box`, a 1 point border, and
/// padding 8 above and below and 12 at each side.
fn field_box(css: Props, children: Vec<Item>) -> Item {
    use V::*;
    let mut css = css;
    css.extend([
        ("boxSizing", U(1)),
        ("borderTop", F(1.0)),
        ("borderRight", F(1.0)),
        ("borderBottom", F(1.0)),
        ("borderLeft", F(1.0)),
        ("paddingTop", F(8.0)),
        ("paddingRight", F(12.0)),
        ("paddingBottom", F(8.0)),
        ("paddingLeft", F(12.0)),
    ]);
    boxed(FIELD, css, children)
}

/// The field at rest: 200 by 44 at (16, 100) in its containing block.
fn field() -> Item {
    field_box(at(16.0, 100.0, 200.0, 44.0), Vec::new())
}

const FIELD: &str = "Field";

/// One artboard: its size, its own properties, its root record and items.
struct Artboard {
    name: &'static str,
    size: (f32, f32),
    own: Props,
    items: Vec<Item>,
}

fn artboard(name: &'static str, size: (f32, f32), items: Vec<Item>) -> Artboard {
    Artboard {
        name,
        size,
        own: Vec::new(),
        items,
    }
}

/// One artboard's objects, its local ids counted as they are written.
struct Board {
    b: Vec<u8>,
    count: usize,
    style_base: usize,
    styled: Vec<usize>,
    ids: Vec<(&'static str, usize)>,
}

impl Board {
    fn add(&mut self, kind: &str, props: &[(&str, V)]) -> usize {
        obj(&mut self.b, kind, props);
        self.count += 1;
        self.count - 1
    }

    fn next_style(&mut self, owner: usize) -> u64 {
        self.styled.push(owner);
        (self.style_base + self.styled.len() - 1) as u64
    }

    fn item(&mut self, parent: usize, item: &Item) {
        use V::*;
        let id = self.count;
        self.ids.push((item.name, id));
        let named = [("name", S(item.name)), ("parentId", U(parent as u64))];
        match item.kind {
            Kind::Box => {
                let style = self.next_style(id);
                let mut props = named.to_vec();
                props.push(("styleId", U(style)));
                props.extend(item.own.iter().copied());
                self.add("LayoutComponent", &props);
                self.record(id, &item.css);
            }
            Kind::Used(artboard) => {
                let mut props = named.to_vec();
                props.push(("artboardId", U(artboard)));
                props.extend(item.own.iter().copied());
                self.add("NestedArtboardLayout", &props);
                self.record(id, &item.css);
            }
            Kind::Rows(artboard) => {
                let mut props = named.to_vec();
                props.push(("rowArtboardId", U(artboard)));
                self.add("ArtboardComponentList", &props);
            }
            Kind::Text => {
                self.add("Text", &named);
            }
            Kind::Input(text) => {
                self.add(
                    "TextInput",
                    &[
                        ("name", S("editable")),
                        ("parentId", U(parent as u64)),
                        ("text", S(text)),
                    ],
                );
                self.add(
                    "TextStylePaint",
                    &[("parentId", U(id as u64)), ("fontSize", F(16.0))],
                );
            }
            Kind::Clip(corners) => {
                self.add("Shape", &named);
                let path = self.add(
                    "PointsPath",
                    &[("parentId", U(id as u64)), ("isClosed", U(1))],
                );
                for (x, y) in corners {
                    self.add(
                        "StraightVertex",
                        &[("parentId", U(path as u64)), ("x", F(x)), ("y", F(y))],
                    );
                }
                self.add(
                    "ClippingShape",
                    &[("parentId", U(parent as u64)), ("sourceId", U(id as u64))],
                );
            }
        }
        if let Some((direction, offset)) = item.scroll {
            self.add(
                "ScrollConstraint",
                &[
                    ("parentId", U(id as u64)),
                    ("directionValue", U(direction)),
                    ("scrollOffsetY", F(offset)),
                ],
            );
        }
        if let Some(role) = item.role {
            self.add(
                "SemanticData",
                &[("parentId", U(id as u64)), ("role", U(role))],
            );
        }
        for child in &item.children {
            self.item(id, child);
        }
    }

    fn record(&mut self, owner: usize, css: &Props) {
        let mut record = vec![("parentId", V::U(owner as u64))];
        record.extend(css.iter().copied());
        self.add("LayoutCssStyle", &record);
    }
}

fn board(artboard: &Artboard, style_base: usize) -> Board {
    use V::*;
    let mut board = Board {
        b: Vec::new(),
        count: 0,
        style_base,
        styled: Vec::new(),
        ids: Vec::new(),
    };
    let style = board.next_style(0);
    let mut props = vec![
        ("name", S(artboard.name)),
        ("width", F(artboard.size.0)),
        ("height", F(artboard.size.1)),
        ("styleId", U(style)),
        ("clip", U(0)),
    ];
    props.extend(artboard.own.iter().copied());
    board.add("Artboard", &props);
    board.record(0, &Vec::new());
    for item in &artboard.items {
        board.item(0, item);
    }
    board
}

/// A file of `artboards`, and the local ids of each one's items by name.
fn file(artboards: &[Artboard]) -> (Vec<u8>, Vec<Vec<(&'static str, usize)>>) {
    let mut b = b"RIVE".to_vec();
    // 7.3: a box composes its own rotation and scale with its layout.
    for n in [7, 3, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    let mut ids = Vec::new();
    for artboard in artboards {
        // The styles follow every other object; count those first.
        let objects = board(artboard, 0).count;
        let mut written = board(artboard, objects);
        for owner in written.styled.clone() {
            written.add("LayoutComponentStyle", &[("parentId", V::U(owner as u64))]);
        }
        b.extend(written.b);
        ids.push(written.ids);
    }
    (b, ids)
}

fn id_of(ids: &[(&'static str, usize)], name: &str) -> usize {
    ids.iter()
        .find(|(item, _)| *item == name)
        .unwrap_or_else(|| panic!("no item {name}"))
        .1
}

fn sized<T>() -> u32 {
    std::mem::size_of::<T>() as u32
}

/// A root object by its local id.
fn root_object(local_id: usize) -> NuxSceneObject {
    NuxSceneObject {
        struct_size: sized::<NuxSceneObject>(),
        local_id,
        path: ptr::null(),
        path_len: 0,
        row_keys: ptr::null(),
        row_key_count: 0,
    }
}

/// Row operations a driver makes in the next step's first round.
#[derive(Default)]
struct Rows {
    list: Cell<usize>,
    mount: RefCell<Vec<&'static str>>,
    remove: RefCell<Vec<&'static str>>,
}

fn bytes(key: &'static str) -> NuxByteView {
    NuxByteView {
        data: key.as_ptr(),
        len: key.len(),
    }
}

unsafe extern "C" fn drive_rows(
    context: *mut c_void,
    round: *const NuxPlayerDriverRound,
    writer: *mut NuxSceneWriter,
) -> NuxStatus {
    let rows = unsafe { &*context.cast::<Rows>() };
    if unsafe { (*round).index } != 0 {
        return NuxStatus::Ok;
    }
    let list = root_object(rows.list.get());
    for key in rows.mount.take() {
        let status =
            unsafe { nux_scene_writer_mount_row(writer, &list, bytes(key), ptr::null_mut()) };
        if status != NuxStatus::Ok {
            return status;
        }
    }
    for key in rows.remove.take() {
        let status =
            unsafe { nux_scene_writer_remove_row(writer, &list, bytes(key), ptr::null_mut()) };
        if status != NuxStatus::Ok {
            return status;
        }
    }
    NuxStatus::Ok
}

/// A static player on artboard 0 of a file, with a row driver.
struct Played {
    file: *mut NuxFile,
    instance: *mut NuxArtboardInstance,
    player: *mut NuxPlayer,
    rows: Box<Rows>,
    ids: Vec<Vec<(&'static str, usize)>>,
}

impl Played {
    fn new(artboards: &[Artboard]) -> Self {
        let (bytes, ids) = file(artboards);
        let mut file = ptr::null_mut();
        let mut instance = ptr::null_mut();
        let mut player = ptr::null_mut();
        let rows = Box::new(Rows::default());
        unsafe {
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            let driver = NuxPlayerDriver {
                struct_size: sized::<NuxPlayerDriver>(),
                context: (&raw const *rows).cast_mut().cast(),
                drive: Some(drive_rows),
                release: None,
            };
            assert_eq!(nux_player_driver_set(player, &driver), NuxStatus::Ok);
        }
        Self {
            file,
            instance,
            player,
            rows,
            ids,
        }
    }

    /// The local id of the item `name` in artboard `index`.
    fn id(&self, index: usize, name: &str) -> usize {
        id_of(&self.ids[index], name)
    }

    /// Steps until the scene settles, presenting each, and returns the last.
    fn settle(&self) -> Step {
        for _ in 0..4 {
            self.step().present(self.player);
        }
        self.step()
    }

    fn step(&self) -> Step {
        let step = NuxPlayerStep {
            elapsed_seconds: 1.0 / 60.0,
            ..NuxPlayerStep::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(self.player, &step, &mut result) },
            NuxStatus::Ok
        );
        Step(result)
    }

    fn read(
        &self,
        step: &Step,
        boxes: &[NuxSceneObject],
    ) -> Result<Vec<NuxIslandFrame>, NuxStatus> {
        let mut out = vec![
            NuxIslandFrame {
                struct_size: sized::<NuxIslandFrame>(),
                ..Default::default()
            };
            boxes.len()
        ];
        let status = unsafe {
            nux_player_island_frames(
                self.player,
                step.0,
                boxes.as_ptr(),
                boxes.len(),
                out.as_mut_ptr(),
            )
        };
        if status == NuxStatus::Ok {
            Ok(out)
        } else {
            Err(status)
        }
    }

    /// The island frame of the item `name` of the page.
    fn frame(&self, step: &Step, name: &str) -> NuxIslandFrame {
        self.read(step, &[root_object(self.id(0, name))])
            .unwrap_or_else(|status| panic!("{name}: {status:?}"))[0]
    }
}

impl Drop for Played {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(nux_player_free(self.player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(self.instance), NuxStatus::Ok);
            assert_eq!(nux_file_free(self.file), NuxStatus::Ok);
        }
    }
}

struct Step(*mut NuxPlayerStepResult);

impl Step {
    fn present(&self, player: *mut NuxPlayer) {
        let mut scheduling = NuxPlayerSchedulingInfo {
            struct_size: sized::<NuxPlayerSchedulingInfo>(),
            ..Default::default()
        };
        unsafe {
            assert_eq!(
                nux_player_step_result_scheduling(self.0, &mut scheduling),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_acknowledge_presented(player, scheduling.render_revision),
                NuxStatus::Ok
            );
        }
    }

    fn revision(&self) -> u64 {
        let mut scheduling = NuxPlayerSchedulingInfo {
            struct_size: sized::<NuxPlayerSchedulingInfo>(),
            ..Default::default()
        };
        assert_eq!(
            unsafe { nux_player_step_result_scheduling(self.0, &mut scheduling) },
            NuxStatus::Ok
        );
        scheduling.render_revision
    }
}

impl Drop for Step {
    fn drop(&mut self) {
        assert_eq!(
            unsafe { nux_player_step_result_free(self.0) },
            NuxStatus::Ok
        );
    }
}

fn page(items: Vec<Item>) -> Artboard {
    artboard("Page", (400.0, 600.0), items)
}

fn translation(frame: &NuxIslandFrame) -> (f32, f32) {
    (frame.world_transform[4], frame.world_transform[5])
}

fn close(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < 1.0e-4
}

/// A 200 by 44 box at (16, 100) with a 1 point border and padding 8 by
/// 12: its frame is its border box, (0, 0) to (200, 44), placed at
/// (16, 100); its content box is 1 + 12 = 13 in from each side and
/// 1 + 8 = 9 in from top and bottom: (13, 9) to (187, 35).
#[test]
fn a_box_at_rest_reads_its_frame_and_content_box() {
    let scene = Played::new(&[page(vec![field()])]);
    let step = scene.settle();
    let frame = scene.frame(&step, FIELD);
    assert_eq!(frame.struct_size, sized::<NuxIslandFrame>());
    assert_eq!(frame.render_revision, step.revision());
    assert_eq!(frame.world_transform, [1.0, 0.0, 0.0, 1.0, 16.0, 100.0]);
    assert_eq!(
        [
            frame.frame_min_x,
            frame.frame_min_y,
            frame.frame_max_x,
            frame.frame_max_y
        ],
        [0.0, 0.0, 200.0, 44.0]
    );
    assert_eq!(
        [
            frame.content_min_x,
            frame.content_min_y,
            frame.content_max_x,
            frame.content_max_y
        ],
        [13.0, 9.0, 187.0, 35.0]
    );
    assert_eq!(frame.visible, 1);
    assert_eq!(frame.opacity, 1.0);
    assert_eq!((frame.scale_x, frame.scale_y), (1.0, 1.0));
    assert_eq!(frame.clip, 0);
    assert_eq!(frame.placeable, 1);
}

/// A "Fade" box at 0.5 that clips both axes holds its fade as a picture;
/// a "Parent" at 0.5 inside it hands its own down. The field reads
/// 0.5 * 0.5 = 0.25.
fn faded(field_opacity: f32, inner_opacity: f32) -> Played {
    use V::*;
    let mut field = field();
    field.own = vec![("opacity", F(field_opacity))];
    field.children =
        vec![boxed("Words", size(10.0, 10.0), Vec::new()).own(vec![("opacity", F(inner_opacity))])];
    Played::new(&[page(vec![
        boxed(
            "Fade",
            wrapper(vec![("overflowX", U(2)), ("overflowY", U(2))]),
            vec![boxed("Parent", wrapper(Vec::new()), vec![field]).own(vec![("opacity", F(0.5))])],
        )
        .own(vec![("opacity", F(0.5))]),
    ])])
}

#[test]
fn opacity_is_the_render_opacity_times_each_held_fade_above() {
    let scene = faded(1.0, 1.0);
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(frame.opacity, 0.25);
    assert_eq!(frame.visible, 1);

    let scene = faded(1.0, 0.0);
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(frame.opacity, 0.25, "a child at 0 leaves the box as it is");
    assert_eq!(frame.visible, 1);

    let scene = faded(0.0, 1.0);
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(frame.visible, 0, "the box's own opacity 0");
}

/// A field that clips both axes at 0.5 holds its own fade: its render
/// opacity is 1 and its draw applies the 0.5.
#[test]
fn a_box_that_holds_its_own_fade_reads_it() {
    use V::*;
    let mut field = field();
    field.css.extend([("overflowX", U(2)), ("overflowY", U(2))]);
    field.own = vec![("opacity", F(0.5))];
    let scene = Played::new(&[page(vec![field])]);
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(frame.opacity, 0.5);
    assert_eq!(frame.visible, 1);
}

/// A used "Card" at 0.5 holds the field. The card is a CSS artboard, so it
/// holds its host's fade for its picture: 0.5. Under a "Fade" box at 0.5
/// that clips both axes and holds its own, the field reads 0.5 * 0.5.
#[test]
fn a_fade_held_past_the_box_s_artboard_is_read() {
    use V::*;
    let card = || {
        artboard(
            "Card",
            (200.0, 60.0),
            vec![field_box(size(100.0, 44.0), Vec::new())],
        )
    };
    let used = || {
        item("Used", Kind::Used(1), size(200.0, 60.0), Vec::new()).own(vec![("opacity", F(0.5))])
    };
    let read = |scene: &Played| {
        let path = [NuxScenePathSegment {
            artboard_global_id: PAGE_GLOBAL_ID,
            local_id: scene.id(0, "Used"),
        }];
        let field = NuxSceneObject {
            struct_size: sized::<NuxSceneObject>(),
            local_id: scene.id(1, FIELD),
            path: path.as_ptr(),
            path_len: path.len(),
            row_keys: std::ptr::null(),
            row_key_count: 0,
        };
        scene.read(&scene.settle(), &[field]).expect("read")[0]
    };

    let scene = Played::new(&[page(vec![used()]), card()]);
    let frame = read(&scene);
    assert_eq!(frame.opacity, 0.5);
    assert_eq!(frame.visible, 1);

    let scene = Played::new(&[
        page(vec![
            boxed(
                "Fade",
                wrapper(vec![("overflowX", U(2)), ("overflowY", U(2))]),
                vec![used()],
            )
            .own(vec![("opacity", F(0.5))]),
        ]),
        card(),
    ]);
    assert_eq!(read(&scene).opacity, 0.25);
}

/// A "View" 300 by 100 at the page's corner clips both axes. The field,
/// which clips its own overflow, sits at (16, 99): its 1 point top border,
/// y 99 to 100, is inside the view, and its padding box, from y 100, is
/// not. Its own clip cuts what is inside it, not the field, so the field
/// shows, clipped to the view's (0, 0) to (300, 100).
#[test]
fn a_box_s_own_clip_does_not_hide_it() {
    use V::*;
    let mut field = field_box(at(16.0, 99.0, 200.0, 44.0), Vec::new());
    field.css.extend([("overflowX", U(2)), ("overflowY", U(2))]);
    let mut view = vec![("position", U(1))];
    view.extend(size(300.0, 100.0));
    view.extend([("overflowX", U(2)), ("overflowY", U(2))]);
    let scene = Played::new(&[page(vec![boxed("View", view, vec![field])])]);
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(frame.visible, 1);
    assert_eq!(frame.clip, 1);
    assert_eq!(
        [
            frame.clip_min_x,
            frame.clip_min_y,
            frame.clip_max_x,
            frame.clip_max_y
        ],
        [0.0, 0.0, 300.0, 100.0]
    );
}

#[test]
fn display_none_on_the_box_or_an_ancestor_hides_it() {
    use V::*;
    let mut hidden = field();
    hidden.css.push(("display", U(3)));
    let scene = Played::new(&[page(vec![hidden])]);
    assert_eq!(scene.frame(&scene.settle(), FIELD).visible, 0);

    let scene = Played::new(&[page(vec![boxed(
        "Parent",
        wrapper(vec![("display", U(3))]),
        vec![field()],
    )])]);
    assert_eq!(scene.frame(&scene.settle(), FIELD).visible, 0);
}

/// A clip whose path the read cannot work out, here a corner at infinity,
/// fails the read with `RuntimeError` and writes nothing, as the semantic
/// snapshot fails on it.
#[test]
fn a_clip_path_that_cannot_be_worked_out_fails_the_read() {
    let corners = [
        (0.0, 0.0),
        (f32::INFINITY, 0.0),
        (300.0, 300.0),
        (0.0, 300.0),
    ];
    let scene = Played::new(&[page(vec![boxed(
        "Parent",
        wrapper(Vec::new()),
        vec![
            item("Clipper", Kind::Clip(corners), Vec::new(), Vec::new()),
            field(),
        ],
    )])]);
    let step = scene.settle();
    assert_eq!(
        scene.read(&step, &[root_object(scene.id(0, FIELD))]),
        Err(NuxStatus::RuntimeError)
    );
}

/// The field inside a "Parent" box with these own transform properties.
fn transformed(own: Props) -> NuxIslandFrame {
    let scene = Played::new(&[page(vec![
        boxed("Parent", wrapper(Vec::new()), vec![field()]).own(own),
    ])]);
    scene.frame(&scene.settle(), FIELD)
}

#[test]
fn scale_is_read_and_a_turn_or_a_mirror_is_not_placeable() {
    use V::*;
    let frame = transformed(vec![("scaleX", F(2.0))]);
    assert_eq!((frame.scale_x, frame.scale_y), (2.0, 1.0));
    assert_eq!(frame.placeable, 1);

    let frame = transformed(vec![("scaleX", F(0.0))]);
    assert_eq!((frame.scale_x, frame.scale_y), (0.0, 1.0));
    assert_eq!(frame.placeable, 1, "the host hides a frame at zero scale");

    let frame = transformed(vec![("rotation", F(std::f32::consts::FRAC_PI_6))]);
    assert_eq!(frame.placeable, 0, "a 30 degree turn");

    let frame = transformed(vec![("scaleX", F(-1.0))]);
    assert_eq!(frame.placeable, 0, "a mirror");
}

/// A 300 by 200 viewport at (0, 50) that scrolls (`overflow-y: scroll`,
/// which clips both axes) a column: a 100 point gap, then `inner`, then
/// 500 points more. Its content is scrolled down by `scrolled`.
fn scrolling(scrolled: f32, viewport: Props, inner: Item) -> Played {
    use V::*;
    let mut css = at(0.0, 50.0, 300.0, 200.0);
    css.push(("overflowY", U(3)));
    css.extend(viewport);
    let mut content = boxed(
        "Content",
        Vec::new(),
        vec![
            boxed("Gap", size(300.0, 100.0), Vec::new()),
            inner,
            boxed("Tail", size(300.0, 500.0), Vec::new()),
        ],
    );
    content.scroll = Some((1, -scrolled));
    Played::new(&[page(vec![boxed("Viewport", css, vec![content])])])
}

/// The field in the flow: 200 by 44.
fn flow_field() -> Item {
    field_box(size(200.0, 44.0), Vec::new())
}

#[test]
fn a_scroll_viewport_moves_and_clips_the_box() {
    use V::*;
    // At rest the field sits under the gap: 50 + 100 = 150.
    let scene = scrolling(0.0, Vec::new(), flow_field());
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(translation(&frame), (0.0, 150.0));
    assert_eq!(frame.clip, 1);
    assert_eq!(
        [
            frame.clip_min_x,
            frame.clip_min_y,
            frame.clip_max_x,
            frame.clip_max_y
        ],
        [0.0, 50.0, 300.0, 250.0]
    );
    assert_eq!(frame.visible, 1);
    assert_eq!(frame.placeable, 1);

    let scene = scrolling(40.0, Vec::new(), flow_field());
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(translation(&frame), (0.0, 110.0), "150 - 40");
    assert_eq!(
        [
            frame.clip_min_x,
            frame.clip_min_y,
            frame.clip_max_x,
            frame.clip_max_y
        ],
        [0.0, 50.0, 300.0, 250.0]
    );
    assert_eq!(frame.visible, 1);

    // Scrolled by 200, the field spans -50 to -6, above the viewport's top.
    let scene = scrolling(200.0, Vec::new(), flow_field());
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(translation(&frame), (0.0, -50.0));
    assert_eq!(frame.visible, 0);

    let scene = scrolling(
        0.0,
        vec![
            ("radiusTopLeftPercent", F(10.0)),
            ("radiusTopRightPercent", F(10.0)),
            ("radiusBottomRightPercent", F(10.0)),
            ("radiusBottomLeftPercent", F(10.0)),
        ],
        flow_field(),
    );
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(frame.clip, 2, "a rounded viewport");
    assert_eq!(frame.placeable, 0);
    assert_eq!(frame.visible, 1);

    // An "Inner" box 150 by 100 under the gap clips too: (0, 150) to
    // (150, 250), which meets the viewport's (0, 50) to (300, 250) there.
    let mut inner = size(150.0, 100.0);
    inner.extend([("overflowX", U(2)), ("overflowY", U(2))]);
    let scene = scrolling(0.0, Vec::new(), boxed("Inner", inner, vec![flow_field()]));
    let frame = scene.frame(&scene.settle(), FIELD);
    assert_eq!(frame.clip, 1);
    assert_eq!(
        [
            frame.clip_min_x,
            frame.clip_min_y,
            frame.clip_max_x,
            frame.clip_max_y
        ],
        [0.0, 150.0, 150.0, 250.0]
    );
    assert_eq!(frame.placeable, 1);
}

/// The page's file-global artboard id: the Backboard is object 0.
const PAGE_GLOBAL_ID: u32 = 1;

/// A page whose column holds a used "Card" (200 by 60), then a keyed list
/// of cards. A card holds an "Inset" box 200 by 60 (border-box) padded 8
/// from the top and 16 from the left, and in it the field, 100 by 44, so
/// the field sits at (16, 8) in the card.
///
/// The field is in normal flow on purpose. Nothing in the card is
/// positioned, so an absolute field would be placed against the initial
/// containing block, as in a browser, and every card's field would land in
/// the same place.
fn addressed() -> Played {
    let card = artboard(
        "Card",
        (200.0, 60.0),
        vec![boxed(
            "Inset",
            [
                size(200.0, 60.0),
                vec![
                    ("boxSizing", V::U(1)),
                    ("paddingTop", V::F(8.0)),
                    ("paddingLeft", V::F(16.0)),
                ],
            ]
            .concat(),
            vec![field_box(size(100.0, 44.0), Vec::new())],
        )],
    );
    let scene = Played::new(&[
        page(vec![
            item("Used", Kind::Used(1), size(200.0, 60.0), Vec::new()),
            item("Cards", Kind::Rows(1), Vec::new(), Vec::new()),
        ]),
        card,
    ]);
    scene.rows.list.set(scene.id(0, "Cards"));
    scene.rows.mount.replace(vec!["a", "b"]);
    scene
}

#[test]
fn each_occurrence_of_a_box_reads_its_own_frame() {
    let scene = addressed();
    let step = scene.settle();
    let field = scene.id(1, FIELD);
    let used = [NuxScenePathSegment {
        artboard_global_id: PAGE_GLOBAL_ID,
        local_id: scene.id(0, "Used"),
    }];
    let list = [NuxScenePathSegment {
        artboard_global_id: PAGE_GLOBAL_ID,
        local_id: scene.id(0, "Cards"),
    }];
    let row = |key| {
        [NuxSceneRowKey {
            artboard_global_id: PAGE_GLOBAL_ID,
            host_local_id: scene.id(0, "Cards"),
            key: bytes(key),
        }]
    };
    let (row_a, row_b) = (row("a"), row("b"));
    let inside = |path: &[NuxScenePathSegment], rows: &[NuxSceneRowKey]| NuxSceneObject {
        struct_size: sized::<NuxSceneObject>(),
        local_id: field,
        path: path.as_ptr(),
        path_len: path.len(),
        row_keys: rows.as_ptr(),
        row_key_count: rows.len(),
    };
    let frames = scene
        .read(
            &step,
            &[
                inside(&used, &[]),
                inside(&list, &row_a),
                inside(&list, &row_b),
            ],
        )
        .expect("three occurrences");
    // The used card fills the column's first 60 points, and each row
    // follows: the field is 16 in and 8 down in each.
    assert_eq!(translation(&frames[0]), (16.0, 8.0));
    assert_eq!(translation(&frames[1]), (16.0, 68.0));
    assert_eq!(translation(&frames[2]), (16.0, 128.0));
    for frame in &frames {
        assert_eq!(
            [
                frame.frame_min_x,
                frame.frame_min_y,
                frame.frame_max_x,
                frame.frame_max_y
            ],
            [0.0, 0.0, 100.0, 44.0]
        );
        assert_eq!(frame.visible, 1);
    }

    // Removing a row changes the scene: the step before it is stale.
    scene.rows.remove.replace(vec!["a"]);
    let before = step;
    let step = scene.settle();
    assert_eq!(
        scene.read(&before, &[inside(&list, &row_b)]),
        Err(NuxStatus::HandleMismatch)
    );
    assert_eq!(
        translation(&scene.read(&step, &[inside(&list, &row_b)]).expect("row b")[0]),
        (16.0, 68.0),
        "row b moves up into row a's place"
    );
    let sentinel = NuxIslandFrame {
        struct_size: sized::<NuxIslandFrame>(),
        opacity: 7.0,
        ..Default::default()
    };
    let mut out = [sentinel];
    let gone = [inside(&list, &row_a)];
    assert_eq!(
        unsafe {
            nux_player_island_frames(scene.player, step.0, gone.as_ptr(), 1, out.as_mut_ptr())
        },
        NuxStatus::NotFound
    );
    assert_eq!(out, [sentinel], "a failed read writes nothing");
}

#[test]
fn the_read_keeps_its_contract() {
    let scene = Played::new(&[page(vec![
        field(),
        item("Words", Kind::Text, Vec::new(), Vec::new()),
    ])]);
    let step = scene.settle();
    let field = root_object(scene.id(0, FIELD));
    let words = root_object(scene.id(0, "Words"));
    let missing = root_object(9_999);
    let sentinel = NuxIslandFrame {
        struct_size: sized::<NuxIslandFrame>(),
        opacity: 7.0,
        ..Default::default()
    };
    let read = |step: &Step, boxes: &[NuxSceneObject], out: &mut [NuxIslandFrame]| unsafe {
        nux_player_island_frames(
            scene.player,
            step.0,
            boxes.as_ptr(),
            boxes.len(),
            out.as_mut_ptr(),
        )
    };

    assert_eq!(read(&step, &[], &mut []), NuxStatus::Ok, "count 0");
    let mut out = [sentinel];
    assert_eq!(read(&step, &[words], &mut out), NuxStatus::InvalidArgument);
    assert_eq!(read(&step, &[missing], &mut out), NuxStatus::NotFound);
    assert_eq!(out, [sentinel]);

    let mut short = [NuxIslandFrame {
        struct_size: sized::<NuxIslandFrame>() - 4,
        ..sentinel
    }];
    assert_eq!(
        read(&step, &[field], &mut short),
        NuxStatus::InvalidStructSize
    );
    assert_eq!(short[0].opacity, 7.0);

    // Both arrays step by their struct's size, so a box record of an
    // older, shorter size, or an out record of a longer one, is refused.
    let mut out = [sentinel; 2];
    let older = NuxSceneObject {
        struct_size: 16,
        ..field
    };
    assert_eq!(
        read(&step, &[older, field], &mut out),
        NuxStatus::InvalidStructSize
    );
    let mut long = [NuxIslandFrame {
        struct_size: sized::<NuxIslandFrame>() + 8,
        ..sentinel
    }];
    assert_eq!(
        read(&step, &[field], &mut long),
        NuxStatus::InvalidStructSize
    );
    assert_eq!(out, [sentinel; 2]);
    assert_eq!(long[0].opacity, 7.0);

    let mut out = [sentinel; 3];
    assert_eq!(
        read(&step, &[field, words, field], &mut out),
        NuxStatus::InvalidArgument
    );
    assert_eq!(out, [sentinel; 3], "one bad box writes nothing");
    assert_eq!(
        read(&step, &[field, field, missing], &mut out),
        NuxStatus::NotFound
    );
    assert_eq!(out, [sentinel; 3]);
    assert_eq!(read(&step, &[field, field], &mut out[..2]), NuxStatus::Ok);
    assert_eq!(out[0], out[1]);
}

/// A page drawn at half size: `world_transform` maps the field's frame to
/// the points `nux_player_text_input_geometry` gives for the layout box
/// around a text input in it.
#[test]
fn world_maps_the_frame_where_text_input_geometry_does() {
    use V::*;
    let mut field = field_box(
        at(16.0, 100.0, 200.0, 44.0),
        vec![item("Input", Kind::Input("hello"), Vec::new(), Vec::new())],
    );
    field.role = Some(u64::from(NUX_SEMANTIC_ROLE_TEXT_FIELD));
    let mut half = page(vec![field]);
    half.own = vec![("scaleX", F(0.5)), ("scaleY", F(0.5))];
    let scene = Played::new(&[half]);
    let step = scene.settle();
    step.present(scene.player);
    let frame = scene.frame(&step, FIELD);
    assert_eq!(frame.world_transform, [0.5, 0.0, 0.0, 0.5, 8.0, 50.0]);
    assert_eq!((frame.scale_x, frame.scale_y), (0.5, 0.5));
    let mut geometry = NuxTextInputGeometry {
        struct_size: sized::<NuxTextInputGeometry>(),
        ..Default::default()
    };
    unsafe {
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            nux_player_semantic_snapshot(scene.player, &mut snapshot),
            NuxStatus::Ok
        );
        let mut info = NuxSemanticSnapshotInfo {
            struct_size: sized::<NuxSemanticSnapshotInfo>(),
            ..Default::default()
        };
        assert_eq!(
            nux_semantic_snapshot_info(snapshot, &mut info),
            NuxStatus::Ok
        );
        let mut node = NuxSemanticNodeView {
            struct_size: sized::<NuxSemanticNodeView>(),
            ..Default::default()
        };
        for index in 0..info.node_count {
            assert_eq!(
                nux_semantic_snapshot_node(snapshot, index, &mut node),
                NuxStatus::Ok
            );
            if node.role == NUX_SEMANTIC_ROLE_TEXT_FIELD {
                break;
            }
        }
        assert_eq!(node.role, NUX_SEMANTIC_ROLE_TEXT_FIELD);
        let name = "editable";
        assert_eq!(
            nux_player_text_input_geometry(
                scene.player,
                snapshot,
                node.id,
                NuxStringView {
                    data: name.as_ptr().cast(),
                    len: name.len(),
                },
                &mut geometry,
            ),
            NuxStatus::Ok
        );
        assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
    }
    assert_eq!(geometry.has_layout_ancestor, 1);
    assert_eq!(
        [
            geometry.layout_ancestor_min_x,
            geometry.layout_ancestor_min_y,
            geometry.layout_ancestor_max_x,
            geometry.layout_ancestor_max_y
        ],
        [
            frame.frame_min_x,
            frame.frame_min_y,
            frame.frame_max_x,
            frame.frame_max_y
        ]
    );
    let map =
        |m: [f32; 6], (x, y): (f32, f32)| (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]);
    for corner in [(0.0, 0.0), (200.0, 0.0), (0.0, 44.0), (200.0, 44.0)] {
        let island = map(frame.world_transform, corner);
        let input = map(geometry.layout_ancestor_transform, corner);
        assert!(
            close(island.0, input.0) && close(island.1, input.1),
            "{corner:?}: {island:?} != {input:?}"
        );
    }
}
