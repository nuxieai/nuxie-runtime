use super::{
    records::LiveSlot,
    transport::{FlightPaint, child, mark_draw_order_dirty, read, size, write},
    validate_move,
};
use crate::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle,
    css_layout::{hosted_frame, paint_order, record},
    source::{
        artboard_component_list::ArtboardComponentList,
        component_dirt::ComponentDirt,
        drawable::RuntimeDrawableOccurrence,
        generated::{
            core_registry::CoreRegistry, nested_artboard_layout_base::NestedArtboardLayoutBase,
            node_base::NodeBase, transform_component_base::TransformComponentBase,
            world_transform_component_base::WorldTransformComponentBase,
        },
        layout_component::{FlightShadow, LayoutComponent},
        math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
        nested_artboard::NestedArtboard,
        shapes::paint::color::color_argb,
    },
};
use anyhow::{Result, ensure};
use std::collections::HashMap;
use taffy::style::Position;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SizeMode {
    #[default]
    Scale,
    Layout,
}
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub transform: Mat2D,
    pub size: Vec2D,
    pub opacity: f32,
}
/// A component's pose in root drawing coordinates. Artboard's existing draw
/// projection includes frame origins, self transforms, full list-row matrices
/// and nested layout placement. Endpoints and the flight parent share this path.
fn draw_pose(component: &CoreHandle) -> Result<(Mat2D, f32, f32)> {
    let (board, world, opacity, child_opacity) = component
        .with(|o| {
            let transform = o.as_world_transform_component()?;
            let board = if o.as_artboard().is_some() {
                Some(component.clone())
            } else {
                o.as_component().and_then(|c| c.artboard_handle())
            };
            Some((
                board,
                if o.as_artboard().is_some() {
                    Mat2D::identity()
                } else {
                    transform.world_transform()
                },
                o.as_transform_component()?.render_opacity(),
                o.world_transform_child_opacity().unwrap_or(1.0),
            ))
        })
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("pose has no transform"))?;
    let matrix = if let Some(board) = board {
        board
            .with_downcast_mut::<Artboard, _>(|board| {
                let origin = board.semantic_root_transform(world * Vec2D::new(0.0, 0.0));
                let x = board.semantic_root_transform(world * Vec2D::new(1.0, 0.0)) - origin;
                let y = board.semantic_root_transform(world * Vec2D::new(0.0, 1.0)) - origin;
                Mat2D::new(x.x, x.y, y.x, y.y, origin.x, origin.y)
            })
            .ok_or_else(|| anyhow::anyhow!("pose artboard retired"))?
    } else {
        world
    };
    Ok((matrix, opacity, child_opacity))
}
// Size sampling deliberately does not enter draw_pose. All frame sizes must
// be applied and laid out before any endpoint transform is sampled.
fn slot_size(host: &CoreHandle) -> Result<Vec2D> {
    let frozen_size = read(host, |s| {
        s.live_move
            .as_ref()
            .and_then(|state| state.size)
            .or_else(|| s.natural_size_for_layout())
    })?;
    Ok(child(host)?
        .map(|b| size(host, &b))
        .or(frozen_size)
        .unwrap_or_default())
}
pub fn slot_pose(host: &CoreHandle) -> Result<Pose> {
    let (transform, opacity, _) = draw_pose(host)?;
    Ok(Pose {
        transform,
        size: slot_size(host)?,
        opacity,
    })
}

/// The follower sizes the flight slot's own box, through the one route its
/// host kind already sizes a unit by:
/// - a CSS host: width and height in its item record, like a binding;
/// - a native layout host: `instanceWidth` and `instanceHeight` in points,
///   which upstream's `StyleOverrider` pushes into the unit;
/// - a plain host: the unit artboard's own size, as `NestedArtboardLeaf`
///   sizes its artboard upstream.
fn set_flight_box(flight: &CoreHandle, instance: &RuntimeArtboardInstanceHandle, size: Vec2D) {
    use crate::css_layout::keys::{HEIGHT_KIND, WIDTH_KIND};
    use crate::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase as Host;
    let (width, height) = (size.x.max(0.0), size.y.max(0.0));
    if !flight.is_type_of(Host::TYPE_KEY) {
        instance.set_size(width, height);
        return;
    }
    let record = instance.with_artboard(|unit| unit.css_record()).and(
        flight
            .with(|o| {
                o.as_container_component()
                    .and_then(|host| crate::css_layout::record::find_in(host.children()))
            })
            .flatten(),
    );
    if let Some(record) = record {
        for (kind, value) in [(WIDTH_KIND, width), (HEIGHT_KIND, height)] {
            // A length-percentage: the length, with no percentage part.
            CoreRegistry::set_uint_handle(&record, i32::from(kind), 1);
            CoreRegistry::set_double_handle(&record, i32::from(kind + 1), value);
            CoreRegistry::set_double_handle(&record, i32::from(kind + 2), 0.0);
        }
        return;
    }
    for (key, value) in [
        (Host::INSTANCE_WIDTH_SCALE_TYPE_PROPERTY_KEY, 0),
        (Host::INSTANCE_HEIGHT_SCALE_TYPE_PROPERTY_KEY, 0),
        (Host::INSTANCE_WIDTH_UNITS_VALUE_PROPERTY_KEY, 1),
        (Host::INSTANCE_HEIGHT_UNITS_VALUE_PROPERTY_KEY, 1),
    ] {
        CoreRegistry::set_uint_handle(flight, i32::from(key), value);
    }
    CoreRegistry::set_double_handle(flight, i32::from(Host::INSTANCE_WIDTH_PROPERTY_KEY), width);
    CoreRegistry::set_double_handle(
        flight,
        i32::from(Host::INSTANCE_HEIGHT_PROPERTY_KEY),
        height,
    );
}

/// One read of the keyed controls, shared by size, value and pose projection.
pub(super) struct FlightFrame {
    progress: f32,
    path: Vec2D,
    fade: f32,
    blended_size: Vec2D,
    /// The corners the flying unit draws in place of its own, when the
    /// controls carry all four (docs/live-moves.md).
    corners: Option<[f32; 4]>,
    /// The shadow the flying unit draws in place of its own outer shadows,
    /// when the controls carry one (docs/live-moves.md).
    shadow: Option<FlightShadow>,
}
pub(super) struct ProjectedPose {
    matrix: Mat2D,
    opacity: f32,
    /// Scale mode's stretch of the flying unit, (1, 1) in Layout mode.
    stretch: Vec2D,
}
#[derive(Clone)]
pub struct Flight {
    pub flight: CoreHandle,
    pub from: CoreHandle,
    pub to: CoreHandle,
    pub size_mode: SizeMode,
    controls: CoreHandle,
    departure_size: Vec2D,
    pub(super) values: super::values::FlightValues,
    pub(super) landing: CoreHandle,
    pub(super) board: CoreHandle,
    from_board: CoreHandle,
    to_board: CoreHandle,
    from_pose: std::cell::Cell<Pose>,
    to_pose: std::cell::Cell<Pose>,
}
impl Flight {
    pub fn new(
        flight: CoreHandle,
        from: CoreHandle,
        to: CoreHandle,
        size_mode: SizeMode,
    ) -> Result<Self> {
        validate_move(&from, &flight)?;
        validate_move(&from, &to)?;
        Self::along_path(flight, from, to, size_mode)
    }
    // Pose endpoints need not hold the same unit. Transport is validated
    // independently against the named flight and landing slots by the plan.
    pub(super) fn along_path(
        flight: CoreHandle,
        from: CoreHandle,
        to: CoreHandle,
        size_mode: SizeMode,
    ) -> Result<Self> {
        let board = read(&flight, NestedArtboard::parent_artboard_handle)?
            .ok_or_else(|| anyhow::anyhow!("flight has no parent"))?;
        let controls = board
            .with_downcast::<Artboard, _>(|b| {
                b.objects()
                    .iter()
                    .flatten()
                    .find(|h| {
                        h.with_downcast::<LiveSlot, _>(|r| r.parent().as_ref() == Some(&flight))
                            .unwrap_or(false)
                    })
                    .cloned()
            })
            .flatten()
            .ok_or_else(|| anyhow::anyhow!("flight has no LiveSlot record"))?;
        let from_pose = slot_pose(&from)?;
        let to_pose = slot_pose(&to)?;
        let from_board = read(&from, NestedArtboard::parent_artboard_handle)?.expect("slot row");
        let to_board = read(&to, NestedArtboard::parent_artboard_handle)?.expect("slot row");
        let departure_size = from_pose.size;
        Ok(Self {
            landing: to.clone(),
            board,
            from_board,
            to_board,
            from_pose: std::cell::Cell::new(from_pose),
            to_pose: std::cell::Cell::new(to_pose),
            flight,
            from,
            to,
            size_mode,
            controls,
            departure_size,
            values: super::values::FlightValues::default(),
        })
    }
    pub(super) fn use_departure(&mut self, holder: &CoreHandle) -> Result<()> {
        self.departure_size = child(holder)?
            .map(|b| size(holder, &b))
            .ok_or_else(|| anyhow::anyhow!("empty departure holder"))?;
        Ok(())
    }
    pub fn controls(&self) -> CoreHandle {
        self.controls.clone()
    }
    /// Takeoff validation uses the already settled departure scene and makes
    /// no writes. Frame execution uses the separate size and pose phases below.
    pub(super) fn validate(&self) -> Result<()> {
        self.project_pose(&self.size_frame(None)?, None).map(|_| ())
    }

    pub(super) fn size_frame(
        &self,
        active: Option<&std::collections::HashSet<CoreHandle>>,
    ) -> Result<FlightFrame> {
        // Sleeping endpoints retain their last settled size without entering
        // their artboard, just as pose sampling retains their last transform.
        let a = if active.is_none_or(|a| a.contains(&self.from_board)) {
            slot_size(&self.from)?
        } else {
            self.from_pose.get().size
        };
        let b = if active.is_none_or(|a| a.contains(&self.to_board)) {
            slot_size(&self.to)?
        } else {
            self.to_pose.get().size
        };
        let (progress, path_x, path_y, fade, corners, shadow) = self
            .controls
            .with_downcast::<LiveSlot, _>(|r| {
                (
                    r.values.progress,
                    r.values.path_x,
                    r.values.path_y,
                    r.values.fade,
                    r.values.corners,
                    r.values.shadow,
                )
            })
            .ok_or_else(|| anyhow::anyhow!("flight controls retired"))?;
        ensure!(
            progress.is_finite()
                && path_x.is_finite()
                && path_y.is_finite()
                && fade.is_finite()
                && corners.iter().all(|corner| corner.is_finite())
                && shadow.iter().all(|value| value.is_finite()),
            "non-finite flight controls"
        );
        let t = progress.clamp(0.0, 1.0);
        let mix = |a: f32, b: f32| a + (b - a) * t;
        let blended_size = Vec2D::new(mix(a.x, b.x), mix(a.y, b.y));
        ensure!(
            blended_size.x.is_finite() && blended_size.y.is_finite(),
            "non-finite flight size"
        );
        if self.size_mode == SizeMode::Scale {
            ensure!(
                self.departure_size.x > 0.0 && self.departure_size.y > 0.0,
                "scale mode needs a nonzero box"
            );
        }
        Ok(FlightFrame {
            progress: t,
            path: Vec2D::new(path_x, path_y),
            fade,
            blended_size,
            // A corner below 0 (-1 by default) carries no corners: the unit
            // draws its own.
            corners: corners
                .iter()
                .all(|corner| *corner >= 0.0)
                .then_some(corners),
            shadow: flight_shadow(shadow),
        })
    }

    pub(super) fn apply_size(&self, frame: &FlightFrame) -> Result<()> {
        // Blended values can feed text/layout bindings. Apply them before the
        // layout barrier too, using the same progress that the pose will use.
        self.values.follow(frame.progress);
        let Some(instance) = child(&self.flight)? else {
            return Ok(());
        };
        let size = if self.size_mode == SizeMode::Layout {
            frame.blended_size
        } else {
            self.departure_size
        };
        set_flight_box(&self.flight, &instance, size);
        // The corners and the shadow mark paths, not layout; transport clears
        // them.
        instance.with_artboard_mut(|unit| {
            unit.set_flight_corners(frame.corners);
            unit.set_flight_shadow(frame.shadow);
        });
        Ok(())
    }

    pub(super) fn project_pose(
        &self,
        frame: &FlightFrame,
        active: Option<&std::collections::HashSet<CoreHandle>>,
    ) -> Result<ProjectedPose> {
        if active.is_none_or(|a| a.contains(&self.from_board)) {
            self.from_pose.set(slot_pose(&self.from)?);
        }
        if active.is_none_or(|a| a.contains(&self.to_board)) {
            self.to_pose.set(slot_pose(&self.to)?);
        }
        let a = self.from_pose.get();
        let b = self.to_pose.get();
        let t = frame.progress;
        let mix = |a: f32, b: f32| a + (b - a) * t;
        let (ac, bc) = (a.transform.decompose(), b.transform.decompose());
        let mut blended = ac;
        let angle = (bc.rotation() - ac.rotation() + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        blended.set_rotation(ac.rotation() + angle * t);
        blended.set_scale_x(mix(ac.scale_x(), bc.scale_x()));
        blended.set_scale_y(mix(ac.scale_y(), bc.scale_y()));
        blended.set_skew(mix(ac.skew(), bc.skew()));
        blended.set_x(mix(ac.x(), bc.x()) + frame.path.x);
        blended.set_y(mix(ac.y(), bc.y()) + frame.path.y);
        let stretch = if self.size_mode == SizeMode::Scale {
            // Scaling stretches the flying unit under its own transform: it can
            // use the settled endpoint sizes without changing the departure box
            // or triggering layout. The slot's pose keeps the blended placement.
            let size = Vec2D::new(mix(a.size.x, b.size.x), mix(a.size.y, b.size.y));
            Vec2D::new(
                size.x / self.departure_size.x,
                size.y / self.departure_size.y,
            )
        } else {
            Vec2D::new(1.0, 1.0)
        };
        let parent = self.flight.with(|o| o.component_parent_handle()).flatten();
        let (parent_world, _, parent_opacity) = parent
            .as_ref()
            .map(draw_pose)
            .transpose()?
            .unwrap_or((Mat2D::identity(), 1.0, 1.0));
        let mut inverse = Mat2D::identity();
        ensure!(parent_world.invert(&mut inverse), "singular flight parent");
        let opacity = mix(a.opacity, b.opacity) * frame.fade;
        ensure!(
            parent_opacity != 0.0 || opacity == 0.0,
            "opaque flight under transparent parent"
        );
        let matrix = inverse * Mat2D::compose(&blended);
        ensure!(
            (0..6).all(|i| matrix[i].is_finite()) && opacity.is_finite(),
            "non-finite flight pose"
        );
        Ok(ProjectedPose {
            matrix,
            opacity: if parent_opacity == 0.0 {
                0.0
            } else {
                opacity / parent_opacity
            },
            stretch,
        })
    }

    pub(super) fn apply_pose(&self, pose: ProjectedPose) -> Result<()> {
        // These are transform/opacity overrides only. Size and value writes
        // belong to apply_size, so projection cannot invalidate the layout.
        let matrix = pose.matrix;
        write(&self.flight, |s| {
            s.live_move_mut().local_override = Some(matrix);
            s.base.base.mark_transform_dirty();
            s.base.base.base.add_dirt(ComponentDirt::COMPONENTS, false);
        });
        let local = matrix.decompose();
        for (key, value) in [
            (NodeBase::X_PROPERTY_KEY, local.x()),
            (NodeBase::Y_PROPERTY_KEY, local.y()),
            (
                TransformComponentBase::ROTATION_PROPERTY_KEY,
                local.rotation(),
            ),
            (
                TransformComponentBase::SCALE_X_PROPERTY_KEY,
                local.scale_x(),
            ),
            (
                TransformComponentBase::SCALE_Y_PROPERTY_KEY,
                local.scale_y(),
            ),
            (
                WorldTransformComponentBase::OPACITY_PROPERTY_KEY,
                pose.opacity,
            ),
        ] {
            CoreRegistry::set_double_handle(&self.flight, i32::from(key), value);
        }
        if let Some(unit) = child(&self.flight)? {
            unit.with_artboard_mut(|unit| unit.set_flight_stretch(pose.stretch));
        }
        Ok(())
    }

    /// The matched element's resting slot on the upper Screen, the board
    /// that holds this flight's slot: `to` going forward, `from` on Back.
    pub(super) fn anchor(&self) -> Option<CoreHandle> {
        if self.to_board == self.board {
            Some(self.to.clone())
        } else if self.from_board == self.board {
            Some(self.from.clone())
        } else {
            None
        }
    }

    /// Reads the anchor's stacking context, through the frame at rest in it.
    /// When it differs from the one the placement holds, it stores it and
    /// marks the Screen's draw order dirty, so the list is sorted with it
    /// before `read_cover` walks it. True when it changed. A flight with no
    /// anchor ends its placement.
    pub(super) fn read_context(&self) -> Result<bool> {
        let Some(anchor) = self.anchor() else {
            if write(&self.flight, |s| s.live_move_mut().paint.take()).is_some() {
                mark_draw_order_dirty(&self.flight);
            }
            return Ok(false);
        };
        let context = anchor_context(&anchor);
        let changed = write(&self.flight, |s| match s.live_move_mut().paint.as_mut() {
            Some(paint) if paint.context != context => {
                paint.context = context;
                true
            }
            _ => false,
        });
        if changed {
            mark_draw_order_dirty(&self.flight);
        }
        Ok(changed)
    }

    /// Finds what covers the anchor: walking the Screen's sorted list from
    /// the anchor toward the top, the first drawable that draws something
    /// and covers the anchor's resting box. Stores it in the placement, and
    /// marks the draw order dirty only when it changed.
    pub(super) fn read_cover(&self) -> Result<()> {
        let Some(anchor) = self.anchor() else {
            return Ok(());
        };
        let list = self
            .board
            .with_downcast::<Artboard, _>(|board| board.drawables().to_vec())
            .unwrap_or_default();
        let at = list.iter().position(
            |item| matches!(item, RuntimeDrawableOccurrence::Authored(slot) if *slot == anchor),
        );
        let mut rest = Resting::new(&self.board);
        let coverer = at.and_then(|at| {
            let anchor_box = rest.slot_box(&anchor);
            list[..at]
                .iter()
                .rev()
                .find(|item| {
                    draws_something(item)
                        && match (anchor_box, rest.drawable_box(item)) {
                            (Some(a), Some(b)) => overlaps(a, b),
                            _ => true,
                        }
                })
                .cloned()
        });
        let changed = write(&self.flight, |s| {
            let paint = s.live_move_mut().paint.as_mut()?;
            let same = paint.anchor == anchor
                && match (&paint.coverer, &coverer) {
                    (Some(a), Some(b)) => paint_order::same(a, b),
                    (None, None) => true,
                    _ => false,
                };
            paint.anchor = anchor;
            paint.coverer = coverer;
            Some(!same)
        });
        if changed == Some(true) {
            mark_draw_order_dirty(&self.flight);
        }
        Ok(())
    }

    /// The placement take-off seeds on this flight's slot, before any frame
    /// is drawn: its anchor and the anchor's context, with no coverer yet.
    pub(super) fn seed_placement(&self) -> Option<CoreHandle> {
        let anchor = self.anchor()?;
        let context = anchor_context(&anchor);
        write(&self.flight, |s| {
            s.live_move_mut().paint = Some(FlightPaint {
                anchor: anchor.clone(),
                coverer: None,
                context,
            })
        });
        mark_draw_order_dirty(&self.flight);
        Some(anchor)
    }
}

/// The shadow seven controls carry. An alpha below 0 (-1 by default) carries
/// none: the unit draws its own. Alpha and channels round to 8 bits.
fn flight_shadow([alpha, red, green, blue, blur, x, y]: [f32; 7]) -> Option<FlightShadow> {
    let byte = |value: f32| value.clamp(0.0, 255.0).round() as i32;
    (alpha >= 0.0).then(|| FlightShadow {
        color: color_argb(byte(alpha * 255.0), byte(red), byte(green), byte(blue)),
        blur: blur.max(0.0),
        offset: Vec2D::new(x, y),
    })
}

/// Whether a slot is a stacking context in its Screen's paint order, as the
/// sort reads it: it has a CSS record, and its own opacity or the frame at
/// rest in it makes one (css_layout/hosted_frame.rs).
fn anchor_context(anchor: &CoreHandle) -> bool {
    anchor
        .with(|o| {
            o.as_container_component()
                .and_then(|slot| record::find_in(slot.children()))
                .is_some()
                && hosted_frame::slot_makes_context(o)
        })
        .unwrap_or(false)
}

/// The two boxes overlap by more than 0.01 on both axes: touching edges do
/// not cover.
fn overlaps(a: Aabb, b: Aabb) -> bool {
    a.left().max(b.left()) + 0.01 < a.right().min(b.right())
        && a.top().max(b.top()) + 0.01 < a.bottom().min(b.bottom())
}

/// Whether a drawable can cover a flight: it will draw, and it is not a
/// box's own layout drawable (whose draw only restores), a box's proxy with
/// no shape paint, a flight slot, or a slot whose unit is away.
fn draws_something(item: &RuntimeDrawableOccurrence) -> bool {
    if !item.will_draw() {
        return false;
    }
    match item {
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => {
            let owner = proxy.borrow().owner_handle();
            owner
                .with(|o| {
                    o.as_layout_component()
                        .is_none_or(|layout| layout.has_shape_paints())
                })
                .unwrap_or(false)
        }
        RuntimeDrawableOccurrence::Authored(handle) => handle
            .with(|o| {
                if o.as_layout_component().is_some() {
                    return false;
                }
                let Some(slot) = o.as_nested_artboard() else {
                    return true;
                };
                let flight = slot.children().iter().any(|child| {
                    child
                        .with_downcast::<LiveSlot, _>(|r| r.values.role != 0)
                        .unwrap_or(false)
                });
                let away = slot
                    .live_move
                    .as_ref()
                    .is_some_and(|state| state.away_instance.is_some());
                !flight && !away
            })
            .unwrap_or(false),
    }
}

/// The bounds of the box (0, 0, `size`) under `transform`, from its corners.
fn transformed_box(transform: Mat2D, size: Vec2D) -> Aabb {
    Aabb::from_points(&[
        transform * Vec2D::new(0.0, 0.0),
        transform * Vec2D::new(size.x, 0.0),
        transform * Vec2D::new(0.0, size.y),
        transform * Vec2D::new(size.x, size.y),
    ])
}

/// Resting boxes on one Screen, in its root's frame. A box is its laid-out
/// box composed up to the Screen root, each level's own transform applied
/// as it is now, or at identity (its rest) when a timeline keys it. A slot's
/// box is its layout box under the transform of the frame at rest in it.
/// None means the drawable counts as covering whatever its geometry: a level
/// on its chain holds keyed layout or follows the flow beside a sibling that
/// does, the Screen root's own layout is keyed, or a level is not CSS (a
/// translated layout or a plain node).
struct Resting {
    board: CoreHandle,
    /// The Screen root's own resting transform: identity, or None when a
    /// timeline keys its layout (its record, or a grid track among its
    /// children), so that every box on the Screen counts as covering. Only
    /// its own mark counts. The root's box is the Screen's, so keyed layout
    /// inside it moves nothing through the root.
    root: Option<Mat2D>,
    worlds: HashMap<CoreHandle, Option<Mat2D>>,
    /// `holds_keyed_layout`'s answer for each box it has read.
    held: HashMap<CoreHandle, bool>,
}
impl Resting {
    fn new(board: &CoreHandle) -> Self {
        let keyed = css_record(board).is_some_and(|record| record::is_layout_keyed(&record));
        Self {
            board: board.clone(),
            root: (!keyed).then(Mat2D::identity),
            worlds: HashMap::new(),
            held: HashMap::new(),
        }
    }

    /// A drawable's resting box: its own slot's, or that of the CSS box it
    /// paints for (its owner, or the nearest box above its component).
    fn drawable_box(&mut self, item: &RuntimeDrawableOccurrence) -> Option<Aabb> {
        let mut component = match item {
            RuntimeDrawableOccurrence::Authored(handle) => handle.clone(),
            RuntimeDrawableOccurrence::RuntimeProxy(proxy) => proxy.borrow().owner_handle(),
        };
        if component
            .with(|o| o.as_nested_artboard().is_some())
            .unwrap_or(false)
        {
            return self.slot_box(&component);
        }
        loop {
            if component == self.board {
                let root = self.root?;
                let size = self.board.with_downcast::<Artboard, _>(|b| {
                    Vec2D::new(b.layout_width(), b.layout_height())
                })?;
                return Some(transformed_box(root, size));
            }
            match component.with(|o| o.as_layout_component().map(|l| l.is_css_mode())) {
                Some(Some(true)) => break,
                Some(Some(false)) => return None,
                _ => {}
            }
            component = component.with(|o| o.component_parent_handle()).flatten()?;
        }
        let world = self.world(&component)?;
        let size = component
            .with(|o| {
                o.as_layout_component().map(|l| {
                    let bounds = l.layout_bounds();
                    Vec2D::new(bounds.width(), bounds.height())
                })
            })
            .flatten()?;
        Some(transformed_box(world, size))
    }

    /// A CSS slot's resting box: its layout box and size under the
    /// transform of the frame at rest in it, at stretch (1, 1).
    fn slot_box(&mut self, slot: &CoreHandle) -> Option<Aabb> {
        let world = self.world(slot)?;
        let rest = read(slot, hosted_frame::frame_at_rest).ok()?;
        let size = slot_size(slot).ok()?;
        Some(transformed_box(
            world * hosted_frame::rest_transform(&rest),
            size,
        ))
    }

    /// A level's resting world transform, from the Screen root down.
    fn world(&mut self, level: &CoreHandle) -> Option<Mat2D> {
        if *level == self.board {
            return self.root;
        }
        if let Some(world) = self.worlds.get(level) {
            return *world;
        }
        let world = self.compose(level);
        self.worlds.insert(level.clone(), world);
        world
    }

    fn compose(&mut self, level: &CoreHandle) -> Option<Mat2D> {
        let parent = level.with(|o| o.component_parent_handle()).flatten()?;
        let (local, record) = level
            .with(|o| {
                if let Some(layout) = o.as_layout_component() {
                    if !layout.is_css_mode() {
                        return None;
                    }
                    let own = if layout.transform_keyed() {
                        Mat2D::identity()
                    } else {
                        layout.build_own_transform()
                    };
                    return Some((
                        Mat2D::from_translation(layout.layout_translation()) * own,
                        layout.css_record()?,
                    ));
                }
                let slot = o.as_nested_artboard()?;
                // A plain host or a native layout host is not measured.
                if !level.is_type_of(NestedArtboardLayoutBase::TYPE_KEY) {
                    return None;
                }
                let record = record::find_in(slot.children())?;
                let node = o.as_transform_component()?.transform();
                // The slot's own placement (nested_artboard_layout.rs): its
                // layout box, and a pivot that stays put when it turns.
                let (mut base, pivot) = slot.live_move_layout_instance()?.with_artboard(|i| {
                    (
                        Vec2D::new(i.layout_x(), i.layout_y()) - i.origin(),
                        i.origin_is_pivot_only().then(|| i.origin_offset()),
                    )
                });
                if let Some(origin) = parent
                    .with(|parent| parent.as_artboard().map(Artboard::origin))
                    .flatten()
                {
                    base += origin;
                }
                let mut local = Mat2D::from_translation(base) * node;
                if let Some(p) = pivot
                    && (node[0] != 1.0 || node[1] != 0.0 || node[2] != 0.0 || node[3] != 1.0)
                {
                    local[4] += p.x - (node[0] * p.x + node[2] * p.y);
                    local[5] += p.y - (node[1] * p.x + node[3] * p.y);
                }
                Some((local, record))
            })
            .flatten()?;
        if self.holds_keyed_layout(level) || self.pushed_by_keyed_layout(level, &record, &parent) {
            return None;
        }
        Some(self.world(&parent)? * local)
    }

    /// Whether layout can read a keyed record for this box's size or place:
    /// its own record is keyed (a grid track or placement marks it), or a box
    /// in flow inside it holds keyed layout, as layout's own children go. A
    /// translated box has no record and counts by what it holds. A slot
    /// counts by both records, the host's here and its frame's as its child.
    /// Conservative on purpose: a fixed-height wrapper counts too, as
    /// covering is the safe side.
    fn holds_keyed_layout(&mut self, item: &CoreHandle) -> bool {
        if let Some(held) = self.held.get(item) {
            return *held;
        }
        let held = css_record(item).is_some_and(|record| record::is_layout_keyed(&record))
            || layout_children_in_flow(item)
                .iter()
                .any(|child| self.holds_keyed_layout(child));
        self.held.insert(item.clone(), held);
        held
    }

    /// Whether a sibling in flow that holds keyed layout can move `level`,
    /// because `level` follows the flow.
    fn pushed_by_keyed_layout(
        &mut self,
        level: &CoreHandle,
        own: &CoreHandle,
        parent: &CoreHandle,
    ) -> bool {
        record::with_values(own, |css| css.follows_the_flow()).unwrap_or(false)
            && layout_children_in_flow(parent)
                .iter()
                .any(|sibling| sibling != level && self.holds_keyed_layout(sibling))
    }
}

/// The CSS record layout reads `item`'s style from: a box's own, or for a
/// slot the one among its children.
fn css_record(item: &CoreHandle) -> Option<CoreHandle> {
    item.with(|o| match o.as_layout_component() {
        Some(layout) => layout.css_record(),
        None => o
            .as_container_component()
            .and_then(|c| record::find_in(c.children())),
    })
    .flatten()
}

/// What layout flows directly inside `item`. It reads layout's own list and
/// changes nothing: `layout_providers_occurrence` sees through plain nodes
/// and Solo, and the two reads `layout_node_owner_for` makes stand in for
/// `layout_node_for`, which takes a node's layout data.
/// - A list gives its rows, except a collapsed one, which has no node.
/// - A slot holds the one artboard its layout places: its frame, or while it
///   is empty the placeholder `remember` kept, never the frame in flight.
///   That placeholder holds nothing, so a flown frame's keys reach no one.
fn layout_children_in_flow(item: &CoreHandle) -> Vec<CoreHandle> {
    if let Some(instance) = layout_instance(item) {
        return vec![instance.core_handle()];
    }
    if item
        .with(|o| o.as_layout_component().is_none())
        .unwrap_or(true)
    {
        return Vec::new();
    }
    let mut children = Vec::new();
    for (child, _) in LayoutComponent::layout_providers_occurrence(item) {
        let rows = child.with_downcast::<ArtboardComponentList, _>(|list| {
            (0..list.item_count())
                .filter(|&row| !list.live_move_row_collapsed(row))
                .filter_map(|row| list.item(row))
                .map(|row| row.core_handle())
                .collect::<Vec<_>>()
        });
        match rows {
            Some(rows) => children.extend(rows),
            None => children.push(child),
        }
    }
    children.retain(in_flow);
    children
}

/// The artboard a slot's layout places, when `item` is a slot with one.
fn layout_instance(item: &CoreHandle) -> Option<RuntimeArtboardInstanceHandle> {
    item.with(|o| {
        o.as_nested_artboard()
            .and_then(NestedArtboard::live_move_layout_instance)
    })
    .flatten()
}

/// Whether layout flows `child`, by the position in the style it builds that
/// node from. Whatever sets the position writes it there: a box's or a row's
/// own record, a slot's child record for the frame it places, a translated
/// box's own style. A shape, text or picture has no such style here, so its
/// record says, and without one it holds nothing that could count.
fn in_flow(child: &CoreHandle) -> bool {
    let node = layout_instance(child).map_or_else(|| child.clone(), |i| i.core_handle());
    let absolute = node
        .with(|o| {
            o.as_layout_component()
                .map(|layout| layout.layout_style().position == Position::Absolute)
        })
        .flatten();
    match absolute {
        Some(absolute) => !absolute,
        None => css_record(child)
            .and_then(|record| record::with_values(&record, |css| css.in_flow()))
            .unwrap_or(false),
    }
}
