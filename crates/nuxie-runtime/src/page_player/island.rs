//! Where a host places a platform view over a box: the box's frame and
//! content box in the root artboard's space, whether it shows, how
//! opaque it is, how it is scaled and what clips it, read from the
//! settled scene.
use super::driver::{SceneObject, scene_object_handle};
use crate::ArtboardInstance;
use crate::css_layout::{group_opacity::held_out_of, lift::OpacityRoot};
use crate::mechanical_port::source::{
    artboard::Artboard,
    core::CoreHandle,
    layout_component::LayoutComponent,
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    semantic::{
        semantic_provider::{
            SemanticGeometryError, island_clip, island_clipped_away, node_has_area,
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
    /// nested placement, scroll offsets and the root's own transform included.
    pub world: Mat2D,
    /// The border box, in the box's own layout space.
    pub frame: Bounds,
    /// The frame less its border widths and padding.
    pub content: Bounds,
    /// Whether it shows: authored visibility and every ancestor's,
    /// an opacity above zero, an area, and not clipped away.
    pub visible: bool,
    /// Its render opacity times every fade held out of it: its own, the
    /// boxes' above it, and each hosting artboard's (`held_out_of`).
    pub opacity: f32,
    /// The x and y scale of `world`.
    pub scale: (f32, f32),
    pub clip: IslandClip,
    /// False when `world` rotates, skews or mirrors, the clip is not a
    /// rectangle, or the box or anything above it flies in place: a
    /// platform view cannot be laid over it. A flight draws the box under
    /// its own pose, which `world` leaves out, for the frames it runs.
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
    // Read from the island box itself, never from inside it.
    let opacity = render_opacity * held_out_of(&handle);
    let world = on
        .with_downcast_mut::<Artboard, _>(|board| {
            if let Some(root) = board.semantic_root_matrix() {
                return root * world;
            }
            // A host callback maps the root: only points can be read.
            let origin = board.semantic_root_transform(world * Vec2D::new(0.0, 0.0));
            let x = board.semantic_root_transform(world * Vec2D::new(1.0, 0.0)) - origin;
            let y = board.semantic_root_transform(world * Vec2D::new(0.0, 1.0)) - origin;
            Mat2D::new(x.x, x.y, y.x, y.y, origin.x, origin.y)
        })
        .ok_or(IslandReadError::NotFound)?;
    let visible = source_ancestors_are_visible(&handle)
        && !island_clipped_away(&handle).map_err(IslandReadError::Geometry)?
        && node_has_area(&handle)
        && opacity > 0.0;
    let [a, b, c, d, _, _] = *world.values();
    let scale = if b == 0.0 && c == 0.0 {
        (a, d)
    } else {
        (a.hypot(b), c.hypot(d))
    };
    let clip = island_clip(&handle);
    let placeable = b == 0.0
        && c == 0.0
        && a >= 0.0
        && d >= 0.0
        && clip != IslandClip::NotRect
        && !flies_in_place(&handle);
    let bounds = |aabb: Aabb| Bounds {
        min_x: aabb.min_x,
        min_y: aabb.min_y,
        max_x: aabb.max_x,
        max_y: aabb.max_y,
    };
    Ok(IslandFrame {
        world,
        frame: bounds(frame),
        content: bounds(content),
        visible,
        opacity,
        scale,
        clip,
        placeable,
    })
}

/// Whether `node` or anything above it, across hosts, is a copy flying in
/// place: a flight marks its copy's opacity root while it lifts it
/// (css_layout/lift.rs), and draws it under the flight's pose.
fn flies_in_place(node: &CoreHandle) -> bool {
    let mut current = Some(node.clone());
    let mut seen = std::collections::HashSet::new();
    while let Some(handle) = current {
        if !seen.insert(handle.clone()) {
            return false;
        }
        let Some((lifted, next)) = handle.with(|object| {
            let lifted = object
                .as_transform_component()
                .is_some_and(|transform| transform.opacity_root != OpacityRoot::None);
            let next = match object.as_artboard() {
                Some(artboard) => artboard.host(),
                None => object.component_parent_handle(),
            };
            (lifted, next)
        }) else {
            return false;
        };
        if lifted {
            return true;
        }
        current = next;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flight marks its copy's opacity root while it lifts it, as
    /// `InPlace::mark` does: a box under the copy, or the copy, flies in
    /// place; a box beside it does not; and once the mark ends, none does.
    #[test]
    fn a_box_under_a_copy_flying_in_place_is_not_placeable() {
        use crate::css_layout::lift::set_opacity_root;
        use crate::group_opacity_fixture::{
            Block, Item, Page, Unit, bind_page, box_record, screen_row,
        };
        use crate::{File, RuntimeFactoryHandle};
        use nuxie_render_api::{PersistentFactory, RecordingFactory};

        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let block = |name, at, items| Block {
            name: Some(name),
            at,
            size: (100.0, 60.0),
            items,
            ..Block::default()
        };
        let file = File::import(
            &Page::one(
                (300.0, 300.0),
                Unit {
                    items: vec![
                        Item::Block(block(
                            "Copy",
                            (40.0, 40.0),
                            vec![Item::Block(block("Inside", (0.0, 0.0), Vec::new()))],
                        )),
                        Item::Block(block("Beside", (40.0, 160.0), Vec::new())),
                    ],
                    ..Unit::css(300.0, 300.0)
                },
            )
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
        let node = |name| {
            box_record(&screen, name)
                .with(|record| record.component_parent_handle())
                .flatten()
                .expect("the box")
        };
        let (copy, inside, beside) = (node("Copy"), node("Inside"), node("Beside"));
        assert!(!flies_in_place(&inside));

        set_opacity_root(&copy, OpacityRoot::Own);
        assert!(flies_in_place(&copy));
        assert!(flies_in_place(&inside));
        assert!(!flies_in_place(&beside));

        set_opacity_root(&copy, OpacityRoot::None);
        assert!(!flies_in_place(&inside));
    }
}
