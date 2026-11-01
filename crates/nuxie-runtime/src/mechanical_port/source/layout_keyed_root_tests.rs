//! UNIV-5930: a keyframe may write an artboard root's `clip`, `width`,
//! `height`, `fractionalWidth` or `fractionalHeight`, as upstream's defs let
//! a Rive file key them. Upstream's `clipChanged` and size callbacks
//! (`layout_component.cpp`) mark the root's layout node dirty, and
//! `clipChanged` adds path dirt, synchronously. The port must do the same
//! without borrowing the root it is writing. A box below the root, and the
//! binding route, keep today's behavior.
use crate::live_move_fixture::{V, obj, paint_header};
use crate::source::{
    animation::keyframe_bool::KeyFrameBool,
    component_dirt::ComponentDirt,
    generated::{
        animation::keyframe_bool_base::KeyFrameBoolBase, core_registry::CoreRegistry,
        layout_component_base::LayoutComponentBase,
    },
    layout_component::LayoutComponent,
    nested_artboard::NestedArtboard,
};
use crate::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

const FRAME: f32 = 1.0 / 60.0;
const CLIP: u16 = LayoutComponentBase::CLIP_PROPERTY_KEY;
const WIDTH: u16 = LayoutComponentBase::WIDTH_PROPERTY_KEY;
const HEIGHT: u16 = LayoutComponentBase::HEIGHT_PROPERTY_KEY;
const FRACTIONAL_WIDTH: u16 = LayoutComponentBase::FRACTIONAL_WIDTH_PROPERTY_KEY;
const FRACTIONAL_HEIGHT: u16 = LayoutComponentBase::FRACTIONAL_HEIGHT_PROPERTY_KEY;

/// Child's root is its object 0 and its box its object 1.
const ROOT: u64 = 0;
const BOX: u64 = 1;

/// Writes the animation `name` keying `object`: each key is one property
/// with one keyframe at frame 0, a `KeyFrameBool` for a bool value and a
/// `KeyFrameDouble` otherwise.
fn animation(b: &mut Vec<u8>, name: &str, object: u64, keys: &[(u16, V<'_>)]) {
    use V::*;
    obj(
        b,
        "LinearAnimation",
        &[
            ("name", S(name)),
            ("fps", U(60)),
            ("duration", U(60)),
            ("loopValue", U(0)),
        ],
    );
    obj(b, "KeyedObject", &[("objectId", U(object))]);
    for &(key, value) in keys {
        obj(b, "KeyedProperty", &[("propertyKey", U(u64::from(key)))]);
        let kind = if matches!(value, B(_)) {
            "KeyFrameBool"
        } else {
            "KeyFrameDouble"
        };
        obj(b, kind, &[("frame", U(0)), ("value", value)]);
    }
}

/// A Rive-made file. Its default artboard "Page", 300 by 300, nests the
/// artboard "Child" in its host "host". Child is 200 by 100 and clips, and
/// holds the box "Box", 50 by 40, which clips too. Each of Child's
/// animations keys one property, so no other write's dirt can stand in for
/// its own: "clip off" keys its root's `clip` false, "width" its root's
/// width 150, "height" its root's height 80, "fractional width" its root's
/// fractional width 2, "fractional height" its root's fractional height 3,
/// "box clip" its box's `clip` false and "box width" its box's width 60.
fn bytes() -> Vec<u8> {
    use V::*;
    let mut b = paint_header();
    obj(&mut b, "Backboard", &[]);
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Page")),
            ("width", F(300.0)),
            ("height", F(300.0)),
            ("clip", B(true)),
        ],
    );
    obj(
        &mut b,
        "NestedArtboard",
        &[
            ("name", S("host")),
            ("parentId", U(0)),
            ("artboardId", U(1)),
        ],
    );
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Child")),
            ("width", F(200.0)),
            ("height", F(100.0)),
            ("clip", B(true)),
        ],
    );
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("Box")),
            ("parentId", U(ROOT)),
            ("styleId", U(BOX + 1)),
            ("width", F(50.0)),
            ("height", F(40.0)),
            ("clip", B(true)),
        ],
    );
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(BOX))]);
    animation(&mut b, "clip off", ROOT, &[(CLIP, B(false))]);
    animation(&mut b, "width", ROOT, &[(WIDTH, F(150.0))]);
    animation(&mut b, "height", ROOT, &[(HEIGHT, F(80.0))]);
    animation(
        &mut b,
        "fractional width",
        ROOT,
        &[(FRACTIONAL_WIDTH, F(2.0))],
    );
    animation(
        &mut b,
        "fractional height",
        ROOT,
        &[(FRACTIONAL_HEIGHT, F(3.0))],
    );
    animation(&mut b, "box clip", BOX, &[(CLIP, B(false))]);
    animation(&mut b, "box width", BOX, &[(WIDTH, F(60.0))]);
    b
}

struct Scene {
    /// The artboard one frame advances: the page, or Child alone.
    top: RuntimeArtboardInstanceHandle,
    /// Child, nested in the page's host or instanced alone.
    child: RuntimeArtboardInstanceHandle,
    _file: RuntimeFileHandle,
    _factory: PersistentFactory<RecordingFactory>,
}

impl Scene {
    fn import() -> (RuntimeFileHandle, PersistentFactory<RecordingFactory>) {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes(),
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            None,
            None,
            None,
        )
        .expect("the keyed root fixture imports");
        (file, factory)
    }

    /// Child as the page's host mounts it, settled.
    fn nested() -> Self {
        let (file, factory) = Self::import();
        let page = file.with_file(File::artboard_default).expect("the page");
        let host = page
            .with_artboard(|page| page.find_handle::<NestedArtboard>("host"))
            .expect("the page's host");
        let child = host
            .with(|host| host.as_nested_artboard()?.artboard_instance_default())
            .flatten()
            .expect("Child, nested in the host");
        let scene = Self {
            top: page,
            child,
            _file: file,
            _factory: factory,
        };
        scene.settle();
        scene
    }

    /// Child instanced alone, with no host, settled.
    fn plain() -> Self {
        let (file, factory) = Self::import();
        let child = file
            .with_file(|file| file.artboard_named("Child"))
            .expect("Child alone");
        let scene = Self {
            top: child.clone(),
            child,
            _file: file,
            _factory: factory,
        };
        scene.settle();
        scene
    }

    fn advance(&self) {
        self.top.advance_default(FRAME);
    }

    fn settle(&self) {
        self.advance();
        self.advance();
    }

    /// Applies Child's animation `name` at its one key, as a playing
    /// animation does.
    fn play(&self, name: &str) {
        self.child
            .animation_named(name)
            .unwrap_or_else(|| panic!("Child's animation {name}"))
            .apply(1.0);
    }

    fn root(&self) -> CoreHandle {
        self.child.core_handle()
    }

    fn named_box(&self) -> CoreHandle {
        self.child
            .with_artboard(|child| child.find_handle::<LayoutComponent>("Box"))
            .expect("Child's box")
    }

    /// Whether Child holds `layout` for its next layout pass, and whether
    /// `layout`'s node waits for its tree's next solve.
    fn layout_dirty(&self, layout: &CoreHandle) -> (bool, bool) {
        let marked = self
            .child
            .with_artboard(|child| child.layout_is_dirty(layout));
        let queued = layout
            .with(|object| {
                object
                    .as_layout_component()
                    .expect("a layout component")
                    .layout_data
                    .is_dirty()
            })
            .expect("a live layout component");
        (marked, queued)
    }

    /// Child's root `clip`, read as the draw reads it to clip the root's
    /// paint (`Artboard::draw_handle`).
    fn root_clips(&self) -> bool {
        self.child.with_artboard(|child| child.clip())
    }

    /// Child's root size, as its last layout solved it.
    fn root_layout_size(&self) -> (f32, f32) {
        self.child
            .with_artboard(|child| (child.layout_width(), child.layout_height()))
    }
}

fn path_dirty(component: &CoreHandle) -> bool {
    component
        .with(|object| {
            object
                .as_component()
                .expect("a component")
                .has_dirt(ComponentDirt::PATH)
        })
        .expect("a live component")
}

fn bool_value(object: &CoreHandle, key: u16) -> bool {
    CoreRegistry::get_bool_handle(object, key.into()).expect("a bool property")
}

fn double_value(object: &CoreHandle, key: u16) -> f32 {
    CoreRegistry::get_double_handle(object, key.into()).expect("a double property")
}

/// R1 and R5: `write` turns Child's clipping root off. The root reads false
/// at once, its layout node is dirty and its path too, as upstream's
/// `clipChanged` leaves them. The next advance lays it out and cleans it,
/// and the draw finds the root unclipped.
fn assert_root_unclips(scene: &Scene, write: impl FnOnce(&Scene)) {
    let root = scene.root();
    assert!(scene.root_clips(), "Child's root clips as written");
    assert_eq!(
        scene.layout_dirty(&root),
        (false, false),
        "Child is settled before the write"
    );
    assert!(!path_dirty(&root), "Child's root path is settled");
    write(scene);
    assert!(!bool_value(&root, CLIP), "the write turns clip off");
    assert_eq!(
        scene.layout_dirty(&root),
        (true, true),
        "the write marks the root's layout node dirty"
    );
    assert!(path_dirty(&root), "the write marks the root's path dirty");
    scene.advance();
    assert_eq!(
        scene.layout_dirty(&root),
        (false, false),
        "the next advance lays the root out"
    );
    assert!(
        !path_dirty(&root),
        "the next advance rebuilds the root path"
    );
    assert!(!scene.root_clips(), "the draw finds the root unclipped");
}

/// R1: a keyframe turns a nested artboard's clipping root off. Before
/// UNIV-5930 the write re-borrowed the root it was writing and panicked.
#[test]
fn a_keyframe_turns_a_nested_artboards_root_clip_off() {
    assert_root_unclips(&Scene::nested(), |scene| scene.play("clip off"));
}

/// R2: the same on an artboard instanced alone, with no host.
#[test]
fn a_keyframe_turns_a_plain_artboards_root_clip_off() {
    assert_root_unclips(&Scene::plain(), |scene| scene.play("clip off"));
}

/// R5 (guard): the binding route, `set_bool_handle`, leaves the nested root
/// as R1's keyframe does after one advance.
#[test]
fn a_binding_turns_a_nested_artboards_root_clip_off_as_a_keyframe_does() {
    assert_root_unclips(&Scene::nested(), |scene| {
        assert!(CoreRegistry::set_bool_handle(
            &scene.root(),
            CLIP.into(),
            false
        ));
    });
}

/// R3: a keyframe writes a nested artboard's root width. Its layout node is
/// dirty after the write, and the next advance lays the root out at 150.
#[test]
fn a_keyframe_resizes_a_nested_artboards_root_width() {
    let scene = Scene::nested();
    let root = scene.root();
    assert_eq!(scene.root_layout_size(), (200.0, 100.0), "as written");
    assert_eq!(scene.layout_dirty(&root), (false, false), "settled");
    scene.play("width");
    assert_eq!(double_value(&root, WIDTH), 150.0, "the keyed width");
    assert_eq!(
        scene.layout_dirty(&root),
        (true, true),
        "the write marks the root's layout node dirty"
    );
    scene.advance();
    assert_eq!(
        scene.layout_dirty(&root),
        (false, false),
        "the next advance lays the root out"
    );
    assert_eq!(
        scene.root_layout_size(),
        (150.0, 100.0),
        "at the keyed width"
    );
}

/// R3: a keyframe writes a nested artboard's root height, fractional width
/// or fractional height, each on its own settled root, so each write must
/// mark the root's layout node dirty itself. The next advance lays the root
/// out and keeps the keyed value, and the keyed height sets its height.
#[test]
fn a_keyframe_resizes_a_nested_artboards_root_height_and_fractions() {
    let cases: [(&str, u16, f32, Option<(f32, f32)>); 3] = [
        ("height", HEIGHT, 80.0, Some((200.0, 80.0))),
        ("fractional width", FRACTIONAL_WIDTH, 2.0, None),
        ("fractional height", FRACTIONAL_HEIGHT, 3.0, None),
    ];
    for (name, key, value, size) in cases {
        let scene = Scene::nested();
        let root = scene.root();
        assert_eq!(
            scene.root_layout_size(),
            (200.0, 100.0),
            "{name}: as written"
        );
        assert_eq!(scene.layout_dirty(&root), (false, false), "{name}: settled");
        scene.play(name);
        assert_eq!(double_value(&root, key), value, "{name}: the keyed value");
        assert_eq!(
            scene.layout_dirty(&root),
            (true, true),
            "{name}: the write marks the root's layout node dirty"
        );
        scene.advance();
        assert_eq!(
            scene.layout_dirty(&root),
            (false, false),
            "{name}: the next advance lays the root out"
        );
        assert_eq!(
            double_value(&root, key),
            value,
            "{name}: the root keeps the keyed value"
        );
        if let Some(size) = size {
            assert_eq!(scene.root_layout_size(), size, "{name}: at the keyed size");
        }
    }
}

/// `layout`'s width, as its last layout solved it.
fn solved_width(layout: &CoreHandle) -> f32 {
    layout
        .with(|object| {
            object
                .as_layout_component()
                .expect("a layout component")
                .layout_width()
        })
        .expect("a live layout component")
}

/// R4 and its borrowed form: `write` sets Child's settled box's `clip` to
/// `clip`. The box reads it at once, with its layout node and path dirty, as
/// upstream's `clipChanged` leaves them, and the next advance cleans both
/// and keeps its width.
fn assert_box_clip(scene: &Scene, clip: bool, write: impl FnOnce(&CoreHandle)) {
    let bx = scene.named_box();
    assert_eq!(bool_value(&bx, CLIP), !clip, "the box's clip before");
    assert_eq!(solved_width(&bx), 50.0, "as written");
    assert_eq!(scene.layout_dirty(&bx), (false, false), "settled");
    assert!(!path_dirty(&bx), "the box path is settled");
    write(&bx);
    assert_eq!(bool_value(&bx, CLIP), clip, "the keyed clip");
    assert_eq!(
        scene.layout_dirty(&bx),
        (true, true),
        "the write marks the box's layout node dirty"
    );
    assert!(path_dirty(&bx), "the write marks the box's path dirty");
    scene.advance();
    assert_eq!(
        scene.layout_dirty(&bx),
        (false, false),
        "the next advance lays the box out"
    );
    assert!(!path_dirty(&bx), "the next advance rebuilds the box path");
    assert_eq!(solved_width(&bx), 50.0, "the box keeps its width");
}

/// R4 (guard): keyframes write a box below a nested artboard's root, its
/// `clip` off or its width 60, each on its own settled box, as before
/// UNIV-5930. Each write leaves the box's layout node dirty itself, the
/// clip write its path too, and the next advance cleans it and lays it out.
#[test]
fn a_keyframe_writes_a_box_below_the_root_as_before() {
    let scene = Scene::nested();
    assert_box_clip(&scene, false, |_| scene.play("box clip"));

    let scene = Scene::nested();
    let bx = scene.named_box();
    assert_eq!(solved_width(&bx), 50.0, "as written");
    assert_eq!(scene.layout_dirty(&bx), (false, false), "settled");
    scene.play("box width");
    assert_eq!(double_value(&bx, WIDTH), 60.0, "the keyed width");
    assert_eq!(
        scene.layout_dirty(&bx),
        (true, true),
        "the width write marks the box's layout node dirty"
    );
    scene.advance();
    assert_eq!(
        scene.layout_dirty(&bx),
        (false, false),
        "the next advance lays the box out"
    );
    assert_eq!(solved_width(&bx), 60.0, "at the keyed width");
}

fn bool_keyframe(value: bool) -> KeyFrameBool {
    let mut frame = KeyFrameBool::default();
    CoreRegistry::set_bool(
        &mut frame,
        KeyFrameBoolBase::VALUE_PROPERTY_KEY.into(),
        value,
    );
    frame
}

/// R4 (guard) through the borrowed entry points: `KeyFrameBool::apply` and
/// `apply_interpolation` take a target their caller already holds and
/// finish inside that borrow, so a box's clip callback runs there, as before
/// UNIV-5930, rather than after a release that hasn't happened.
#[test]
fn a_borrowed_bool_keyframe_writes_a_box_below_the_root_as_before() {
    let scene = Scene::nested();
    let off = bool_keyframe(false);
    let on = bool_keyframe(true);
    assert_box_clip(&scene, false, |bx| {
        bx.with_mut(|object| off.apply(object, CLIP.into(), 1.0, None))
            .expect("a live box");
    });
    assert_box_clip(&scene, true, |bx| {
        bx.with_mut(|object| on.apply_interpolation(object, CLIP.into(), 0.5, &off, 1.0, None))
            .expect("a live box");
    });
}
