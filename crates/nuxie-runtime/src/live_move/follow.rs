use super::{
    affine::{Affine, mix},
    records::LiveSlot,
    transport::{child, read, size, write},
    validate_move,
};
use crate::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle,
    source::{
        component_dirt::ComponentDirt,
        generated::{
            core_registry::CoreRegistry, node_base::NodeBase,
            transform_component_base::TransformComponentBase,
            world_transform_component_base::WorldTransformComponentBase,
        },
        math::{mat2d::Mat2D, vec2d::Vec2D},
        nested_artboard::NestedArtboard,
        text::text::Text,
    },
};
use anyhow::{Result, ensure};
use std::{cell::Cell, rc::Rc};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SizeMode {
    #[default]
    Scale,
    Layout,
    /// Our layer: the two versions of a text that changes its look fly by
    /// the lines they draw, each scaled evenly (docs/live-moves.md).
    Lines,
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
}
pub(super) struct ProjectedPose {
    matrix: Mat2D,
    opacity: f32,
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
    /// Our layer: the text pair this flight flies with in lines mode.
    lines: Option<Rc<LinePair>>,
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
            lines: None,
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
    /// The mode this flight flies in now. A lines-mode flight without a
    /// partner, or whose pair met a degenerate frame, flies in Scale mode.
    pub fn effective_size_mode(&self) -> SizeMode {
        match (self.size_mode, &self.lines) {
            (SizeMode::Lines, Some(pair)) if !pair.failed.get() => SizeMode::Lines,
            (SizeMode::Lines, _) => SizeMode::Scale,
            (mode, _) => mode,
        }
    }
    fn ensure_scale_box(&self) -> Result<()> {
        ensure!(
            self.departure_size.x > 0.0 && self.departure_size.y > 0.0,
            "scale mode needs a nonzero box"
        );
        Ok(())
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
        let (progress, path_x, path_y, fade) = self
            .controls
            .with_downcast::<LiveSlot, _>(|r| {
                (
                    r.values.progress,
                    r.values.path_x,
                    r.values.path_y,
                    r.values.fade,
                )
            })
            .ok_or_else(|| anyhow::anyhow!("flight controls retired"))?;
        ensure!(
            progress.is_finite() && path_x.is_finite() && path_y.is_finite() && fade.is_finite(),
            "non-finite flight controls"
        );
        let t = progress.clamp(0.0, 1.0);
        let blended_size = Vec2D::new(mix(a.x, b.x, t), mix(a.y, b.y, t));
        ensure!(
            blended_size.x.is_finite() && blended_size.y.is_finite(),
            "non-finite flight size"
        );
        if self.effective_size_mode() == SizeMode::Scale {
            self.ensure_scale_box()?;
        }
        Ok(FlightFrame {
            progress: t,
            path: Vec2D::new(path_x, path_y),
            fade,
            blended_size,
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
        Ok(())
    }

    /// Rectangles are read live only in frames (`active` is set). Takeoff
    /// validation keeps the ones preflight read before transport.
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
        let drawn = match self.lines_pose(&a, &b, frame, active) {
            Some(drawn) => drawn,
            None => self.box_pose(&a, &b, frame)?,
        };
        let parent = self.flight.with(|o| o.component_parent_handle()).flatten();
        let (parent_world, _, parent_opacity) = parent
            .as_ref()
            .map(draw_pose)
            .transpose()?
            .unwrap_or((Mat2D::identity(), 1.0, 1.0));
        let mut inverse = Mat2D::identity();
        ensure!(parent_world.invert(&mut inverse), "singular flight parent");
        let opacity = (a.opacity + (b.opacity - a.opacity) * t) * frame.fade;
        ensure!(
            parent_opacity != 0.0 || opacity == 0.0,
            "opaque flight under transparent parent"
        );
        let matrix = inverse * drawn;
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
        })
    }

    /// Scale and Layout modes blend the two end poses exactly. Scale mode
    /// then scales the departure box to the blended box, in the unit's space.
    fn box_pose(&self, a: &Pose, b: &Pose, frame: &FlightFrame) -> Result<Mat2D> {
        let t = frame.progress;
        let mut blended = Affine::blend(
            &Affine::decompose(&a.transform),
            &Affine::decompose(&b.transform),
            t,
        );
        blended.x += frame.path.x;
        blended.y += frame.path.y;
        if self.effective_size_mode() != SizeMode::Scale {
            return Ok(blended.compose());
        }
        // Scaling is only a pose override: it can use the settled endpoint
        // sizes without changing the departure box or triggering layout.
        self.ensure_scale_box()?;
        let size = Vec2D::new(mix(a.size.x, b.size.x, t), mix(a.size.y, b.size.y, t));
        Ok(blended.compose()
            * Mat2D::from_scale(
                size.x / self.departure_size.x,
                size.y / self.departure_size.y,
            ))
    }

    /// Our layer: lines mode (docs/live-moves.md). The line frame
    /// `F_e = D_e x Translate(c_e) x Scale(h_e)` of each end blends exactly,
    /// and this version draws at `F(t) x Scale(1 / h_v) x Translate(-c_v)`.
    /// None when this flight flies in another mode, or when this frame's
    /// blend is degenerate, which sends the pair to Scale mode for the rest
    /// of the flight.
    fn lines_pose(
        &self,
        a: &Pose,
        b: &Pose,
        frame: &FlightFrame,
        active: Option<&std::collections::HashSet<CoreHandle>>,
    ) -> Option<Mat2D> {
        if self.effective_size_mode() != SizeMode::Lines {
            return None;
        }
        let pair = self.lines.as_ref()?;
        // A sleeping board keeps its last settled rectangles.
        if active.is_some_and(|active| active.contains(&self.board)) {
            pair.refresh();
        }
        let rects = pair.rects.get();
        let mut blended = Affine::blend(
            &Affine::decompose(&(a.transform * rects[0].frame())),
            &Affine::decompose(&(b.transform * rects[1].frame())),
            frame.progress,
        );
        blended.x += frame.path.x;
        blended.y += frame.path.y;
        let lines = blended.compose();
        let drawn = rects.map(|rect| lines * rect.unframe());
        if !usable(&lines) || !drawn.iter().all(usable) {
            pair.failed.set(true);
            return None;
        }
        Some(drawn[usize::from(pair.hosts[1] == self.flight)])
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
        Ok(())
    }
}

/// Our layer: the rectangle a text version's lines take up in its unit
/// space, the space its slot host draws it in (docs/live-moves.md).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct LineRect {
    center: Vec2D,
    height: f32,
}
impl LineRect {
    /// A finite center within a million points of the origin, and a height
    /// from 0.01 to 100,000 points.
    fn in_range(&self) -> bool {
        self.center.x.is_finite()
            && self.center.y.is_finite()
            && self.center.x.abs() <= 1_000_000.0
            && self.center.y.abs() <= 1_000_000.0
            && (0.01..=100_000.0).contains(&self.height)
    }
    /// Both in range, and neither height more than 10,000 times the other.
    fn pair_in_range(rects: &[Self; 2]) -> bool {
        let [a, b] = rects;
        a.in_range() && b.in_range() && a.height.max(b.height) <= 10_000.0 * a.height.min(b.height)
    }
    /// `Translate(c) x Scale(h)`: the frame in which these lines have height 1.
    fn frame(&self) -> Mat2D {
        Mat2D::new(
            self.height,
            0.0,
            0.0,
            self.height,
            self.center.x,
            self.center.y,
        )
    }
    /// `Scale(1 / h) x Translate(-c)`, the inverse of `frame`.
    fn unframe(&self) -> Mat2D {
        let scale = 1.0 / self.height;
        Mat2D::new(
            scale,
            0.0,
            0.0,
            scale,
            -self.center.x * scale,
            -self.center.y * scale,
        )
    }
}

/// Our layer: what a text's two versions share while they fly by their
/// lines. Index 0 is the outgoing version, 1 the incoming one.
pub(super) struct LinePair {
    hosts: [CoreHandle; 2],
    rects: Cell<[LineRect; 2]>,
    failed: Cell<bool>,
}
impl LinePair {
    /// Reads both flying units' rectangles after the layout barrier. One that
    /// can't be read, or fails the range check, keeps its last one that passed.
    fn refresh(&self) {
        let mut rects = self.rects.get();
        for (side, host) in self.hosts.iter().enumerate() {
            let Some(rect) = child(host).ok().flatten().and_then(|unit| unit_lines(&unit)) else {
                continue;
            };
            let mut next = rects;
            next[side] = rect;
            if LineRect::pair_in_range(&next) {
                rects = next;
            }
        }
        self.rects.set(rects);
    }
}

/// The unit's one Text's `lines_bounds`, mapped into unit space through the
/// steps `Artboard::draw_content_with_visitor_handle` applies: the Text's
/// `shape_world_transform`, the unit's self transform, then its frame-origin
/// offset. The unit's root transform is never read: during a flight it holds
/// the flight pose. None unless the unit has exactly one Text and it draws
/// a line.
fn unit_lines(unit: &RuntimeArtboardInstanceHandle) -> Option<LineRect> {
    let (texts, unit_space) = unit.with_artboard(|board| {
        let mut space = if board.frame_origin() {
            Mat2D::from_translate(
                board.layout_width() * board.origin_x(),
                board.layout_height() * board.origin_y(),
            )
        } else {
            Mat2D::identity()
        };
        if board.has_self_transform() {
            space = space * board.self_transform();
        }
        (board.find_all_handles::<Text>(), space)
    });
    let [text] = texts.as_slice() else {
        return None;
    };
    let (lines, shape) = text
        .with_downcast::<Text, _>(|text| Some((text.lines_bounds()?, text.shape_world_transform())))
        .flatten()?;
    let matrix = unit_space * shape;
    let corners = [
        Vec2D::new(lines.min_x, lines.min_y),
        Vec2D::new(lines.max_x, lines.min_y),
        Vec2D::new(lines.min_x, lines.max_y),
        Vec2D::new(lines.max_x, lines.max_y),
    ]
    .map(|corner| matrix * corner);
    let (mut min, mut max) = (corners[0], corners[0]);
    for corner in &corners[1..] {
        min = Vec2D::new(min.x.min(corner.x), min.y.min(corner.y));
        max = Vec2D::new(max.x.max(corner.x), max.y.max(corner.y));
    }
    Some(LineRect {
        center: Vec2D::new((min.x + max.x) * 0.5, (min.y + max.y) * 0.5),
        height: max.y - min.y,
    })
}

/// Finite and invertible, with a finite inverse.
fn usable(matrix: &Mat2D) -> bool {
    let mut inverse = Mat2D::identity();
    (0..6).all(|i| matrix[i].is_finite())
        && matrix.invert(&mut inverse)
        && (0..6).all(|i| inverse[i].is_finite())
}

/// Our layer: preflight for a text's two versions, before transport moves
/// anything. `units` are the outgoing and incoming versions in their own
/// resting slots. The pair flies by its lines only when both units have
/// exactly one Text with rectangles in range, both flights are on one board,
/// both end poses are invertible with the same handedness, and both line
/// frames and both versions' drawn matrices at both ends are finite and
/// invertible. Otherwise both versions fly in Scale mode.
pub(super) fn pair_lines(
    outgoing: &mut Flight,
    incoming: &mut Flight,
    units: [Option<RuntimeArtboardInstanceHandle>; 2],
) {
    let (a, b) = (
        outgoing.from_pose.get().transform,
        outgoing.to_pose.get().transform,
    );
    let rects = match units {
        [Some(out), Some(inc)] => unit_lines(&out).zip(unit_lines(&inc)),
        _ => None,
    }
    .map(|(out, inc)| [out, inc])
    .filter(|rects| {
        outgoing.board == incoming.board
            && usable(&a)
            && usable(&b)
            && (a.determinant() > 0.0) == (b.determinant() > 0.0)
            && LineRect::pair_in_range(rects)
            && [a * rects[0].frame(), b * rects[1].frame()]
                .iter()
                .all(|frame| usable(frame) && rects.iter().all(|v| usable(&(*frame * v.unframe()))))
    });
    let pair = rects.map(|rects| {
        Rc::new(LinePair {
            hosts: [outgoing.flight.clone(), incoming.flight.clone()],
            rects: Cell::new(rects),
            failed: Cell::new(false),
        })
    });
    let mode = if pair.is_some() {
        SizeMode::Lines
    } else {
        SizeMode::Scale
    };
    for flight in [outgoing, incoming] {
        flight.size_mode = mode;
        flight.lines = pair.clone();
    }
}
