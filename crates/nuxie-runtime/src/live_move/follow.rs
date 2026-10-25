use super::{
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
    },
};
use anyhow::{Result, ensure};
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
        })
    }

    pub(super) fn apply_size(&self, frame: &FlightFrame) -> Result<()> {
        // Blended values can feed text/layout bindings. Apply them before the
        // layout barrier too, using the same progress that the pose will use.
        self.values.follow(frame.progress);
        if let Some(instance) = child(&self.flight)? {
            let size = if self.size_mode == SizeMode::Layout {
                frame.blended_size
            } else {
                self.departure_size
            };
            instance.set_size(size.x, size.y);
        }
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
        if self.size_mode == SizeMode::Scale {
            // Scaling is only a pose override: it can use the settled endpoint
            // sizes without changing the departure box or triggering layout.
            let size = Vec2D::new(mix(a.size.x, b.size.x), mix(a.size.y, b.size.y));
            blended.set_scale_x(blended.scale_x() * size.x / self.departure_size.x);
            blended.set_scale_y(blended.scale_y() * size.y / self.departure_size.y);
        }
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
        Ok(())
    }
}
