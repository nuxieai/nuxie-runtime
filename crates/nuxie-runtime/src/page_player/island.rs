//! Where a host places a platform view over a box: the box's frame and
//! content box in the root artboard's space, whether it shows, how
//! opaque it is, how it is scaled and what clips it, read from the
//! settled scene.
//!
//! A box under a copy that flies in place is read from the flight's pose:
//! the copy's matrix and opacity, and the clips the flight draws the copy
//! under: those inside the copy, the flight's own zoom clip, and the upper
//! Screen's own and its page's, since the copy draws above its ancestors'
//! clips (css_layout/lift.rs).
use super::driver::{SceneObject, scene_object_handle};
use crate::ArtboardInstance;
use crate::css_layout::{
    group_opacity::{held_box_fade, held_out_of},
    lift::OpacityRoot,
    paint::add_rounded_rect,
};
use crate::mechanical_port::source::{
    artboard::Artboard,
    core::CoreHandle,
    layout_component::LayoutComponent,
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    semantic::{
        semantic_provider::{
            IslandFlight, SemanticGeometryError, island_clip, island_clipped_away, node_has_area,
            source_ancestors_are_visible,
        },
        semantic_snapshot::Bounds,
    },
};

/// What clips a box, in the root artboard's space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IslandClip {
    /// Nothing above it clips it.
    None,
    /// Every clip above it is an axis-aligned rectangle; this is where
    /// they meet.
    Rect(Bounds),
    /// A clip above it is rotated, rounded or another shape.
    NotRect,
}

/// One box, as a host places a platform view over it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IslandFrame {
    /// Maps the box's own layout space into the root artboard's,
    /// nested placement, scroll offsets and the root's own transform
    /// included, and a flight's pose while the box flies in place.
    pub world: Mat2D,
    /// The border box, in the box's own layout space.
    pub frame: Bounds,
    /// The frame less its border widths and padding.
    pub content: Bounds,
    /// Whether it shows: authored visibility and every ancestor's,
    /// an opacity above zero, an area, and not clipped away.
    pub visible: bool,
    /// Its render opacity times every fade held out of it: its own, the
    /// boxes' above it, and each hosting artboard's (`held_out_of`). While
    /// it flies in place, the flight's opacity times the fades held from
    /// it up to the copy instead.
    pub opacity: f32,
    /// The x and y scale of `world`.
    pub scale: (f32, f32),
    pub clip: IslandClip,
    /// False when `world` rotates, skews or mirrors, or the clip is not a
    /// rectangle: a platform view cannot be laid over it.
    pub placeable: bool,
}

/// Why an island frame could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IslandReadError {
    /// The object names nothing.
    NotFound,
    /// The object is not a LayoutComponent.
    NotABox,
    /// A clip around the box is past the geometry budget or not a path, as
    /// a semantic capture fails on it.
    Geometry(SemanticGeometryError),
}

/// `object`'s island frame. NotFound when it names nothing; NotABox
/// when it names something that is not a LayoutComponent.
pub fn island_frame(
    artboard: &ArtboardInstance,
    object: &SceneObject,
) -> Result<IslandFrame, IslandReadError> {
    let handle = scene_object_handle(artboard, object).map_err(|_| IslandReadError::NotFound)?;
    frame_of(&handle)
}

/// The island frame of `handle`, as `island_frame` reads it.
fn frame_of(handle: &CoreHandle) -> Result<IslandFrame, IslandReadError> {
    let Some((world, frame, content, on)) = handle.with_downcast::<LayoutComponent, _>(|layout| {
        (
            layout.shape_world_transform(),
            layout.local_bounds(),
            layout.css_content_box(),
            layout.artboard_handle(),
        )
    }) else {
        return Err(IslandReadError::NotABox);
    };
    let on = on.ok_or(IslandReadError::NotFound)?;
    let render_opacity = handle
        .with(|object| {
            object
                .as_transform_component()
                .map(|transform| transform.render_opacity())
        })
        .flatten()
        .ok_or(IslandReadError::NotABox)?;
    let rest = to_root(&on, world).ok_or(IslandReadError::NotFound)?;
    let flight = match lifted_above(handle) {
        None => None,
        Some((lifted, held)) => match read_flight(&lifted, held) {
            Some(flight) => Some(flight),
            // A copy marked to fly whose lift can't be read or solved
            // draws nothing a platform view can be laid over.
            None => {
                return Ok(IslandFrame {
                    world: rest,
                    frame: bounds(frame),
                    content: bounds(content),
                    visible: false,
                    opacity: 0.0,
                    scale: scale_of(rest),
                    clip: IslandClip::NotRect,
                    placeable: false,
                });
            }
        },
    };
    // Read from the island box itself, never from inside it.
    let (world, opacity) = match &flight {
        None => (rest, render_opacity * held_out_of(handle)),
        Some(flight) => (flight.drawn.pose * rest, render_opacity * flight.opacity),
    };
    let drawn = flight.as_ref().map(|flight| &flight.drawn);
    let clip = island_clip(handle, drawn);
    let visible = source_ancestors_are_visible(handle, drawn.map(|drawn| &drawn.copy))
        && !island_clipped_away(handle, drawn).map_err(IslandReadError::Geometry)?
        && node_has_area(handle)
        && opacity > 0.0;
    let [a, b, c, d, _, _] = *world.values();
    let placeable = b == 0.0 && c == 0.0 && a >= 0.0 && d >= 0.0 && clip != IslandClip::NotRect;
    Ok(IslandFrame {
        world,
        frame: bounds(frame),
        content: bounds(content),
        visible,
        opacity,
        scale: scale_of(world),
        clip,
        placeable,
    })
}

fn bounds(aabb: Aabb) -> Bounds {
    Bounds {
        min_x: aabb.min_x,
        min_y: aabb.min_y,
        max_x: aabb.max_x,
        max_y: aabb.max_y,
    }
}

/// The x and y scale of `world`.
fn scale_of(world: Mat2D) -> (f32, f32) {
    let [a, b, c, d, _, _] = *world.values();
    if b == 0.0 && c == 0.0 {
        (a, d)
    } else {
        (a.hypot(b), c.hypot(d))
    }
}

/// `world`, in `board`'s drawing space, in the root artboard's space.
fn to_root(board: &CoreHandle, world: Mat2D) -> Option<Mat2D> {
    board.with_downcast_mut::<Artboard, _>(|board| {
        if let Some(root) = board.semantic_root_matrix() {
            return root * world;
        }
        // A host callback maps the root: only points can be read.
        let origin = board.semantic_root_transform(world * Vec2D::new(0.0, 0.0));
        let x = board.semantic_root_transform(world * Vec2D::new(1.0, 0.0)) - origin;
        let y = board.semantic_root_transform(world * Vec2D::new(0.0, 1.0)) - origin;
        Mat2D::new(x.x, x.y, y.x, y.y, origin.x, origin.y)
    })
}

/// A copy flying in place that a box is or sits under, as its lift draws
/// it. Read from the lift each Pose barrier writes (`InPlaceFlight::apply_pose`).
struct Flight {
    /// The copy, the pose and the clips the lift draws under.
    drawn: IslandFlight,
    /// The lift's opacity times the fades held from the box up to and
    /// including the copy.
    opacity: f32,
}

/// The copy flying in place that `node` is or sits under, across hosts,
/// and the fades held from `node` up to and including it: a flight marks
/// its copy's opacity root while it lifts it (css_layout/lift.rs), so the
/// copy renders without its ancestors' opacity.
fn lifted_above(node: &CoreHandle) -> Option<(CoreHandle, f32)> {
    let mut current = Some(node.clone());
    let mut seen = std::collections::HashSet::new();
    let mut held = 1.0;
    while let Some(handle) = current {
        if !seen.insert(handle.clone()) {
            return None;
        }
        let (lifted, fade, next) = handle.with(|object| {
            let lifted = object
                .as_transform_component()
                .is_some_and(|transform| transform.opacity_root != OpacityRoot::None);
            match object.as_artboard() {
                Some(artboard) => (lifted, artboard.held_fade(), artboard.host()),
                None => (
                    lifted,
                    object.as_layout_component().map_or(1.0, held_box_fade),
                    object.component_parent_handle(),
                ),
            }
        })?;
        held *= fade;
        if lifted {
            return Some((handle, held));
        }
        current = next;
    }
    None
}

/// The flight of `lifted`, a copy marked to fly in place, from its lift in
/// its Screen's `away`; `held` is the fades held from the box up to it.
/// None when the lift is not there or the Screens can't be solved.
fn read_flight(lifted: &CoreHandle, held: f32) -> Option<Flight> {
    let row = lifted
        .with(|object| object.component_artboard_handle())
        .flatten()?;
    let lift = row
        .with_downcast::<Artboard, _>(|row| {
            row.lifts
                .borrow()
                .away
                .iter()
                .find(|lift| lift.0.borrow().element == *lifted)
                .cloned()
        })
        .flatten()?;
    let state = lift.0.borrow();
    let upper = state
        .anchor
        .with(|object| object.component_artboard_handle())
        .flatten()?;
    let drawn = to_root(&upper, Mat2D::identity())? * state.matrix;
    let mut from_row = Mat2D::identity();
    if !to_root(&state.row, Mat2D::identity())?.invert(&mut from_row) {
        return None;
    }
    // As `lift::apply_clip` draws it: the zoom's box under the copy's
    // world, in the copy's Screen's drawing space, under the lift.
    let zoom = match state.clip {
        None => None,
        Some(clip) => {
            let world = lifted
                .with(|object| {
                    object
                        .as_world_transform_component()
                        .map(|transform| transform.world_transform())
                })
                .flatten()?;
            let space = drawn * world * state.clip_space;
            let mut outline = RawPath::default();
            add_rounded_rect(&mut outline, clip.area(), clip.radii);
            Some(outline.morph(|point| space * point))
        }
    };
    Some(Flight {
        drawn: IslandFlight {
            copy: lifted.clone(),
            // U · M · L⁻¹: U and L the upper Screen's and the copy's
            // Screen's root matrices, M the lift's.
            pose: drawn * from_row,
            zoom,
            upper,
        },
        opacity: state.opacity * held,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css_layout::lift::{Lift, set_opacity_root};
    use crate::group_opacity_fixture::{
        Block, Item, Overflow, Page, Unit, bind_page, box_record, screen_row,
    };
    use crate::mechanical_port::source::artboard::FlightClip;
    use crate::{File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle};
    use nuxie_render_api::{PersistentFactory, RecordingFactory};

    const PAGE: Bounds = Bounds {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 300.0,
        max_y: 400.0,
    };

    /// A 300 by 400 page that clips, as the compiler writes one, holding one
    /// 300 by 400 CSS Screen.
    struct Scene {
        _factory: PersistentFactory<RecordingFactory>,
        _file: RuntimeFileHandle,
        page: RuntimeArtboardInstanceHandle,
        screen: RuntimeArtboardInstanceHandle,
        /// The copy that flies: "Copy", 200 by 60, resting at (0, 280).
        copy: CoreHandle,
        lift: Option<Lift>,
    }

    fn block(name: &'static str, at: (f32, f32), size: (f32, f32), items: Vec<Item>) -> Block {
        Block {
            name: Some(name),
            at,
            size,
            items,
            ..Block::default()
        }
    }

    impl Scene {
        fn new(items: Vec<Item>) -> Self {
            Self::on(
                true,
                Unit {
                    items,
                    ..Unit::css(300.0, 400.0)
                },
            )
        }

        /// A 300 by 400 page that clips when `clips` holds `screen`.
        fn on(clips: bool, screen: Unit) -> Self {
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let file = File::import(
                &Page {
                    clips,
                    ..Page::one((300.0, 400.0), screen)
                }
                .bytes(),
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None,
            )
            .unwrap();
            let page = file.with_file(File::artboard_default).unwrap();
            bind_page(&file, &page);
            let screen = screen_row(&page, "Screen");
            let node = |name| node(&screen, name);
            Self {
                copy: node("Copy"),
                _factory: factory,
                _file: file,
                page,
                screen,
                lift: None,
            }
        }

        /// Fixture A: a 50 by 50 "Frame" at (0, 0) that clips both axes at
        /// opacity 0.8 holds Copy at (0, 280), which holds Field at
        /// (20, 20); "Beside", 100 by 40, rests at (20, 160) in the Screen.
        fn framed() -> Self {
            let copy = block(
                "Copy",
                (0.0, 280.0),
                (200.0, 60.0),
                vec![Item::Block(block(
                    "Field",
                    (20.0, 20.0),
                    (100.0, 40.0),
                    Vec::new(),
                ))],
            );
            Self::new(vec![
                Item::Block(Block {
                    overflow: Overflow::Hidden,
                    opacity: Some(0.8),
                    ..block("Frame", (0.0, 0.0), (50.0, 50.0), vec![Item::Block(copy)])
                }),
                Item::Block(block("Beside", (20.0, 160.0), (100.0, 40.0), Vec::new())),
            ])
        }

        /// Fixture B: Copy at (0, 280) in the Screen, with `overflow`,
        /// holding Field at (`field_x`, 20).
        fn bare(overflow: Overflow, field_x: f32) -> Self {
            Self::new(vec![bare_copy(overflow, None, field_x)])
        }

        /// Lifts Copy as `InPlaceFlight::mark` does: its opacity root, and
        /// its lift in its Screen's `away`. The upper Screen is its own, so
        /// the lift's matrix is the pose in the page's space.
        fn take_off(&mut self) {
            self.take_off_as(OpacityRoot::Own);
        }

        /// `take_off` with the opacity root `root`: `Nothing` lifts it as
        /// a zoomed pair's version, without its own opacity.
        fn take_off_as(&mut self, root: OpacityRoot) {
            let row = self.row();
            let lift = Lift::new(row.clone(), self.copy.clone(), self.copy.clone());
            set_opacity_root(&self.copy, root);
            row.with_downcast::<Artboard, _>(|row| row.lifts.borrow_mut().away.push(lift.clone()));
            self.lift = Some(lift);
        }

        /// Writes the lift's pose as a Pose barrier does
        /// (`InPlaceFlight::apply_pose`), and its zoom clip, then settles.
        fn pose(&self, matrix: Mat2D, opacity: f32, clip: Option<FlightClip>) {
            let mut state = self.lift.as_ref().expect("a flight").0.borrow_mut();
            state.matrix = matrix;
            state.opacity = opacity;
            state.clip = clip;
            drop(state);
            self.page.advance_default(0.0);
        }

        /// Ends the flight as `InPlaceFlight::clear` does, then settles.
        fn land(&mut self) {
            let lift = self.lift.take().expect("a flight");
            set_opacity_root(&self.copy, OpacityRoot::None);
            self.row().with_downcast::<Artboard, _>(|row| {
                row.lifts
                    .borrow_mut()
                    .away
                    .retain(|away| !away.ptr_eq(&lift))
            });
            self.page.advance_default(0.0);
        }

        fn row(&self) -> CoreHandle {
            self.copy
                .with(|object| object.component_artboard_handle())
                .flatten()
                .expect("Copy's Screen")
        }

        fn read(&self, name: &str) -> IslandFrame {
            frame_of(&node(&self.screen, name)).expect("an island frame")
        }
    }

    /// Fixture B's Copy, 200 by 60 at (0, 280), with `overflow` and
    /// `opacity`, holding Field, 100 by 40 at (`field_x`, 20).
    fn bare_copy(overflow: Overflow, opacity: Option<f32>, field_x: f32) -> Item {
        Item::Block(Block {
            overflow,
            opacity,
            ..block(
                "Copy",
                (0.0, 280.0),
                (200.0, 60.0),
                vec![Item::Block(block(
                    "Field",
                    (field_x, 20.0),
                    (100.0, 40.0),
                    Vec::new(),
                ))],
            )
        })
    }

    fn node(screen: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
        box_record(screen, name)
            .with(|record| record.component_parent_handle())
            .flatten()
            .expect("the box")
    }

    fn translate(x: f32, y: f32) -> Mat2D {
        Mat2D::from_translation(Vec2D::new(x, y))
    }

    fn rect(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> IslandClip {
        IslandClip::Rect(Bounds {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    fn zoom(radius: f32) -> FlightClip {
        FlightClip {
            width: 200.0,
            height: 60.0,
            radii: [Vec2D::new(radius, radius); 4],
        }
    }

    fn assert_world(frame: &IslandFrame, expected: [f32; 6]) {
        let world = *frame.world.values();
        assert!(
            world
                .iter()
                .zip(expected)
                .all(|(a, b)| (a - b).abs() < 1e-4),
            "world {world:?}, expected {expected:?}"
        );
    }

    /// Field rests at (20, 300), cut away by the Frame's (0, 0) to
    /// (50, 50) at the Frame's 0.8. Flown up by 200 at 0.5 it draws at
    /// (20, 100) under the page's clip alone; Beside does not move; and once
    /// the flight lands, Field reads at rest again.
    #[test]
    fn a_box_under_a_copy_flying_in_place_reads_the_flights_pose() {
        let mut scene = Scene::framed();
        let at_rest = scene.read("Field");
        assert_world(&at_rest, [1.0, 0.0, 0.0, 1.0, 20.0, 300.0]);
        assert_eq!(at_rest.clip, rect(0.0, 0.0, 50.0, 50.0));
        assert!(!at_rest.visible);
        assert_eq!(at_rest.opacity, 0.8);
        assert!(at_rest.placeable);
        let beside = scene.read("Beside");
        assert_world(&beside, [1.0, 0.0, 0.0, 1.0, 20.0, 160.0]);
        assert_eq!(beside.clip, IslandClip::Rect(PAGE));
        assert!(beside.visible);
        assert_eq!(beside.opacity, 1.0);
        assert!(beside.placeable);

        scene.take_off();
        scene.pose(translate(0.0, -200.0), 0.5, None);
        let flying = scene.read("Field");
        assert_world(&flying, [1.0, 0.0, 0.0, 1.0, 20.0, 100.0]);
        assert_eq!(flying.scale, (1.0, 1.0));
        // 0.5, not 0.5 * 0.8: the Frame sits above the copy, and a copy
        // flying in place renders without its ancestors' opacity, which the
        // flight's own opacity stands in for.
        assert_eq!(flying.opacity, 0.5);
        assert_eq!(flying.clip, IslandClip::Rect(PAGE));
        assert!(flying.visible);
        assert!(flying.placeable);
        assert_eq!(scene.read("Beside"), beside);

        scene.land();
        assert_eq!(scene.read("Field"), at_rest);
    }

    /// Turned by 10 degrees the field can't take a platform view, though
    /// it still shows: its corners span about x -39 to 66 and y 99 to 156.
    #[test]
    fn a_box_in_a_rotated_flight_is_not_placeable() {
        let mut scene = Scene::framed();
        scene.take_off();
        scene.pose(
            translate(0.0, -200.0) * Mat2D::from_rotation(10f32.to_radians()),
            0.5,
            None,
        );
        let frame = scene.read("Field");
        assert!(!frame.placeable);
        assert!(frame.visible);
        assert_eq!(frame.clip, IslandClip::Rect(PAGE));
    }

    /// The zoom's 200 by 60 box in Copy's box, which rests at (0, 280),
    /// draws at (0, 80) to (200, 140) under the pose; the Frame's 50 by 50
    /// clip, above the copy, does not apply. Rounded, it is not a
    /// rectangle.
    #[test]
    fn a_box_in_a_flight_with_a_zoom_clip_is_cut_to_it() {
        let mut scene = Scene::framed();
        scene.take_off();
        scene.pose(translate(0.0, -200.0), 0.5, Some(zoom(0.0)));
        let frame = scene.read("Field");
        assert_eq!(frame.clip, rect(0.0, 80.0, 200.0, 140.0));
        assert!(frame.visible);
        assert!(frame.placeable);

        scene.pose(translate(0.0, -200.0), 0.5, Some(zoom(8.0)));
        let frame = scene.read("Field");
        assert_eq!(frame.clip, IslandClip::NotRect);
        assert!(!frame.placeable);
    }

    /// A field flying in from below the page's bottom edge reads the
    /// page's clip before, through and after the flight; it shows once it
    /// reaches the page.
    #[test]
    fn a_box_flying_in_from_off_the_page_keeps_the_pages_clip() {
        let mut scene = Scene::bare(Overflow::Visible, 20.0);
        let check = |frame: IslandFrame, y: f32, visible: bool| {
            assert_world(&frame, [1.0, 0.0, 0.0, 1.0, 20.0, y]);
            assert_eq!(frame.clip, IslandClip::Rect(PAGE));
            assert_eq!(frame.visible, visible, "visible at y {y}");
            assert_eq!(frame.opacity, 1.0);
            assert!(frame.placeable);
        };
        check(scene.read("Field"), 300.0, true);
        scene.take_off();
        for (dy, visible) in [(200.0, false), (60.0, true), (0.0, true)] {
            scene.pose(translate(0.0, dy), 1.0, None);
            check(scene.read("Field"), 300.0 + dy, visible);
        }
        scene.land();
        check(scene.read("Field"), 300.0, true);
    }

    /// Copy clips at its box, (0, 280) to (200, 340) at rest; Field at
    /// (150, 20) in it sticks out past its right edge. Flown up by 200, the
    /// copy's clip goes with it: (0, 80) to (200, 140).
    #[test]
    fn a_box_in_a_flight_is_cut_by_the_clips_inside_the_copy() {
        let mut scene = Scene::bare(Overflow::Hidden, 150.0);
        assert_eq!(scene.read("Field").clip, rect(0.0, 280.0, 200.0, 340.0));
        scene.take_off();
        scene.pose(translate(0.0, -200.0), 1.0, None);
        let frame = scene.read("Field");
        assert_world(&frame, [1.0, 0.0, 0.0, 1.0, 150.0, 100.0]);
        assert_eq!(frame.clip, rect(0.0, 80.0, 200.0, 140.0));
        assert!(frame.visible);
        assert!(frame.placeable);
    }

    /// The page does not clip; the Screen, written as the compiler writes
    /// a box root, clips its content at (0, 0) to (300, 400). The lift
    /// draws as the Screen's content, so a field flown below it is cut
    /// away.
    #[test]
    fn a_box_in_a_flight_is_cut_by_the_screens_own_clip() {
        let mut scene = Scene::on(
            false,
            Unit {
                items: vec![bare_copy(Overflow::Visible, None, 20.0)],
                ..Unit::compiled(300.0, 400.0)
            },
        );
        let check = |frame: IslandFrame, y: f32, visible: bool| {
            assert_world(&frame, [1.0, 0.0, 0.0, 1.0, 20.0, y]);
            assert_eq!(frame.clip, IslandClip::Rect(PAGE));
            assert_eq!(frame.visible, visible, "visible at y {y}");
            assert!(frame.placeable);
        };
        check(scene.read("Field"), 300.0, true);
        scene.take_off();
        scene.pose(translate(0.0, 60.0), 1.0, None);
        check(scene.read("Field"), 360.0, true);
        scene.pose(translate(0.0, 200.0), 1.0, None);
        check(scene.read("Field"), 500.0, false);
    }

    /// Copy rests at opacity 0, so Field does not show. Lifted as a zoomed
    /// version, which leaves the copy's own opacity out, it draws at the
    /// flight's 0.5.
    #[test]
    fn a_zoomed_copy_at_opacity_zero_shows_at_its_flights_opacity() {
        let mut scene = Scene::new(vec![bare_copy(Overflow::Visible, Some(0.0), 20.0)]);
        let at_rest = scene.read("Field");
        assert_eq!(at_rest.opacity, 0.0);
        assert!(!at_rest.visible);
        scene.take_off_as(OpacityRoot::Nothing);
        scene.pose(translate(0.0, -200.0), 0.5, Some(zoom(0.0)));
        let frame = scene.read("Field");
        assert_eq!(frame.opacity, 0.5);
        assert!(frame.visible);
        assert_eq!(frame.clip, rect(0.0, 80.0, 200.0, 140.0));
        assert!(frame.placeable);
    }

    /// The clips a flight draws a box under cut it together. Flown left by
    /// 210, Copy's clip spans x -210 to -10 and the page's 0 to 300: each
    /// meets Field, at x -60 to 40, and the two share nothing. A rounded
    /// zoom over a field flown to (20, 600), below the page, cuts it away
    /// too.
    #[test]
    fn a_box_cut_away_by_the_draws_clips_together_is_hidden() {
        let mut scene = Scene::bare(Overflow::Hidden, 150.0);
        scene.take_off();
        scene.pose(translate(-210.0, 0.0), 1.0, None);
        let frame = scene.read("Field");
        assert!(!frame.visible);
        assert!(matches!(frame.clip, IslandClip::Rect(_)));

        let mut scene = Scene::framed();
        scene.take_off();
        scene.pose(translate(0.0, 300.0), 0.5, Some(zoom(8.0)));
        let frame = scene.read("Field");
        assert!(!frame.visible);
        assert_eq!(frame.clip, IslandClip::NotRect);
        assert!(!frame.placeable);
    }
}
