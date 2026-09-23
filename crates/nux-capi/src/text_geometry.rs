//! Frame-qualified geometry for native editors over text runs and input occurrences.

use super::*;
use nuxie::runtime::{layout_component::LayoutComponent, text::text::Text};

/// Copied geometry; never contains text, glyphs, or pointers into the scene.
/// Matrices use [a, b, c, d, tx, ty]: x' = a*x + c*y + tx.
/// `world_transform` maps the text's local layout box into artboard space.
/// `content_transform` additionally includes runtime text fitting/alignment.
/// Bounds are the local layout box, including authored origin/baseline offset;
/// they are not the ink bounds and must use `world_transform`.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NuxTextRunGeometry {
    pub struct_size: u32,
    pub render_revision: u64,
    pub world_transform: [f32; 6],
    pub content_transform: [f32; 6],
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    /// Canonical 0/1. Zero means there is no LayoutComponent ancestor.
    pub has_layout_ancestor: u32,
    /// Nearest LayoutComponent ancestor, excluding the text itself. All fields
    /// below are zero when absent. This is a structural relationship, not a
    /// semantic field association or an axis-aligned world bounding box.
    pub layout_ancestor_transform: [f32; 6],
    pub layout_ancestor_min_x: f32,
    pub layout_ancestor_min_y: f32,
    pub layout_ancestor_max_x: f32,
    pub layout_ancestor_max_y: f32,
    /// Canonical 0/1. Empty or unshaped text has no first baseline.
    pub has_first_baseline: u32,
    /// First logical line's baseline y in shaped-content coordinates. Map
    /// through content_transform; zero when has_first_baseline is zero.
    pub first_baseline: f32,
}

pub const NUX_TEXT_RUN_GEOMETRY_MIN_SIZE: usize =
    std::mem::offset_of!(NuxTextRunGeometry, first_baseline) + std::mem::size_of::<f32>();

/// Native TextInput geometry for an exact presented semantic occurrence.
/// The transform maps input-local coordinates into the root artboard, including
/// nested placement. Bounds describe shaped text, NOT the field container;
/// When present, layout_ancestor_* supplies the affine field layout box.
/// Semantic bounds describe an axis-aligned interaction box, not local layout.
/// Contains no editable text, glyph identifiers, or native selection state.
/// With a host-owned content ScrollConstraint, geometry is the unscrolled
/// editing basis and layout_ancestor_* is its stationary viewport. Native
/// editors apply their own content offset; feeding it back here would scroll
/// the UIKit control a second time.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NuxTextInputGeometry {
    pub struct_size: u32,
    pub render_revision: u64,
    pub world_transform: [f32; 6],
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub has_first_baseline: u32,
    pub first_baseline: f32,
    pub obscured: u32,
    pub multiline: u32,
    /// Nearest layout ancestor, in its own local coordinates. The transform
    /// includes nested occurrence placement; semantic bounds are only an AABB.
    pub has_layout_ancestor: u32,
    pub layout_ancestor_transform: [f32; 6],
    pub layout_ancestor_min_x: f32,
    pub layout_ancestor_min_y: f32,
    pub layout_ancestor_max_x: f32,
    pub layout_ancestor_max_y: f32,
}

pub const NUX_TEXT_INPUT_GEOMETRY_MIN_SIZE: usize =
    std::mem::offset_of!(NuxTextInputGeometry, layout_ancestor_max_y) + std::mem::size_of::<f32>();

// Match TextInput::on_added_clean's upstream input -> layout -> content
// relationship. Never search arbitrary ancestors and scroll an enclosing page.
fn input_content_scroll(
    input: &nuxie::runtime::core::CoreHandle,
) -> Option<nuxie::runtime::core::CoreHandle> {
    use nuxie::runtime::text::text_input::TextInput;
    let layout = input.with_downcast::<TextInput, _>(|input| input.base.parent_handle())??;
    layout
        .with(|value| value.component_parent_handle())??
        .with(|parent| {
            let constraints = parent.as_transform_component()?.constraints();
            let mut scrolls = constraints.iter().filter(|constraint| {
                constraint
                    .with(|value| value.as_scroll_constraint().is_some())
                    .unwrap_or(false)
            });
            let scroll = scrolls.next()?.clone();
            // An ambiguous authored container must not be guessed at.
            scrolls.next().is_none().then_some(scroll)
        })?
}

/// Synchronize a native editor's content displacement with the input's existing
/// ScrollConstraint. Positive offsets move content left/up, in artboard-local
/// content units (not device pixels). The field viewport remains fixed.
/// This changes presentation only: no text, binding, cursor, or response write.
/// A changed offset invalidates the capture; step/present before another write.
/// Returns NotFound for legacy fields or inputs without a content constraint.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_text_input_content_offset_set(
    player: *mut NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
    node_id: u32,
    name: NuxStringView,
    offset_x: f32,
    offset_y: f32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if !offset_x.is_finite() || !offset_y.is_finite() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            semantic_snapshot::with_presented_field_property(
                player,
                snapshot,
                node_id,
                name,
                true,
                |occurrence, input| {
                    let Some(scroll) = input_content_scroll(input) else {
                        return NuxStatus::NotFound;
                    };
                    scroll
                        .with_mut(|value| {
                            let Some(scroll) = value.as_scroll_constraint_mut() else {
                                return NuxStatus::NotFound;
                            };
                            let x = if scroll.base.constrains_horizontal() {
                                -offset_x
                            } else {
                                0.0
                            };
                            let y = if scroll.base.constrains_vertical() {
                                -offset_y
                            } else {
                                0.0
                            };
                            if scroll.authored_scroll_offset_x() == x
                                && scroll.authored_scroll_offset_y() == y
                            {
                                return NuxStatus::Ok;
                            }
                            if let Err(status) = occurrence.invalidate_render() {
                                return status;
                            }
                            scroll.stop_physics();
                            scroll.set_authored_scroll_offset_x(x);
                            scroll.set_authored_scroll_offset_y(y);
                            NuxStatus::Ok
                        })
                        .unwrap_or(NuxStatus::NotFound)
                },
            )
        }
    })
}

fn nearest_layout_geometry(
    owner: &nuxie::runtime::core::CoreHandle,
) -> Result<Option<([f32; 6], [f32; 4])>, NuxStatus> {
    let mut visited = std::collections::HashSet::from([owner.identity_key()]);
    let mut ancestor = owner
        .with(|object| {
            object
                .as_component()
                .and_then(|component| component.parent_handle())
        })
        .flatten();
    while let Some(current) = ancestor {
        if !visited.insert(current.identity_key()) {
            return Err(NuxStatus::InvalidArgument);
        }
        if visited.len() > 16_384 {
            return Err(NuxStatus::LimitExceeded);
        }
        if let Some(value) = current.with_downcast::<LayoutComponent, _>(|layout| {
            let bounds = layout.local_bounds();
            (
                *layout.shape_world_transform().values(),
                [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y],
            )
        }) {
            return Ok(Some(value));
        }
        ancestor = current
            .with(|object| {
                object
                    .as_component()
                    .and_then(|component| component.parent_handle())
            })
            .flatten();
    }
    Ok(None)
}

/// Read settled native TextInput geometry for the same presented field used by
/// field_string_copy/set. Stale/foreign snapshots are rejected. Does not focus,
/// advance, edit, or start a runtime selection session. Output changes only on
/// success. CustomPropertyString endpoints have no native input geometry.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_text_input_geometry(
    player: *const NuxPlayer,
    snapshot: *const NuxSemanticSnapshot,
    node_id: u32,
    name: NuxStringView,
    out_geometry: *mut NuxTextInputGeometry,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || unsafe {
        semantic_snapshot::with_presented_field_property(
            player,
            snapshot,
            node_id,
            name,
            false,
            |occurrence, input| {
                use nuxie::runtime::{
                    math::vec2d::Vec2D, semantic::semantic_provider::root_transform_point,
                    text::text_input::TextInput,
                };
                let Some((mut value, mut world, artboard)) = input
                    .with_downcast_mut::<TextInput, _>(|input| {
                        let world = *input.base.world_transform();
                        let artboard = input.base.artboard_handle();
                        let bounds = input.local_bounds();
                        let baseline = input
                            .raw_text_input()
                            .shape()
                            .ordered_lines()
                            .first()
                            .map(|line| line.y());
                        (
                            NuxTextInputGeometry {
                                struct_size: std::mem::size_of::<NuxTextInputGeometry>() as u32,
                                render_revision: occurrence.render_revision.get(),
                                min_x: bounds.min_x,
                                min_y: bounds.min_y,
                                max_x: bounds.max_x,
                                max_y: bounds.max_y,
                                has_first_baseline: u32::from(baseline.is_some()),
                                first_baseline: baseline.unwrap_or(0.0),
                                obscured: u32::from(input.base.obscured()),
                                multiline: u32::from(input.base.multiline()),
                                ..Default::default()
                            },
                            world,
                            artboard,
                        )
                    })
                else {
                    return NuxStatus::NotFound;
                };
                let Some(artboard) = artboard else {
                    return NuxStatus::NotFound;
                };
                let scroll_layout = input_content_scroll(input).and_then(|scroll| {
                    scroll.with(|value| {
                        let scroll = value.as_scroll_constraint()?;
                        Some((
                            scroll.content_handle()?,
                            scroll.viewport_handle()?,
                            scroll.clamped_offset_x() * scroll.base.strength(),
                            scroll.clamped_offset_y() * scroll.base.strength(),
                        ))
                    })?
                });
                let layout = if let Some((content, viewport, x, y)) = scroll_layout {
                    let Some(transform) = content
                        .with(|value| {
                            value
                                .as_transform_component()
                                .map(|value| *value.world_transform())
                        })
                        .flatten()
                    else {
                        return NuxStatus::NotFound;
                    };
                    let delta = transform * Vec2D::new(x, y) - transform * Vec2D::new(0.0, 0.0);
                    // Preserve all authored rotation/scale and nested placement.
                    world[4] -= delta.x;
                    world[5] -= delta.y;
                    Ok(viewport.with_downcast::<LayoutComponent, _>(|layout| {
                        let bounds = layout.local_bounds();
                        (
                            *layout.shape_world_transform().values(),
                            [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y],
                        )
                    }))
                } else {
                    nearest_layout_geometry(input)
                };
                let points = [
                    Vec2D::new(0.0, 0.0),
                    Vec2D::new(1.0, 0.0),
                    Vec2D::new(0.0, 1.0),
                ]
                .map(|point| root_transform_point(&artboard, world * point));
                let [Some(origin), Some(x), Some(y)] = points else {
                    return NuxStatus::NotFound;
                };
                value.world_transform = [
                    x.x - origin.x,
                    x.y - origin.y,
                    y.x - origin.x,
                    y.y - origin.y,
                    origin.x,
                    origin.y,
                ];
                match layout {
                    Ok(Some((transform, bounds))) => {
                        let [a, b, c, d, tx, ty] = transform;
                        let points = [(tx, ty), (tx + a, ty + b), (tx + c, ty + d)]
                            .map(|(x, y)| root_transform_point(&artboard, Vec2D::new(x, y)));
                        let [Some(origin), Some(x), Some(y)] = points else {
                            return NuxStatus::NotFound;
                        };
                        value.has_layout_ancestor = 1;
                        value.layout_ancestor_transform = [
                            x.x - origin.x,
                            x.y - origin.y,
                            y.x - origin.x,
                            y.y - origin.y,
                            origin.x,
                            origin.y,
                        ];
                        [
                            value.layout_ancestor_min_x,
                            value.layout_ancestor_min_y,
                            value.layout_ancestor_max_x,
                            value.layout_ancestor_max_y,
                        ] = bounds;
                    }
                    Ok(None) => {}
                    Err(status) => return status,
                }
                if !value
                    .world_transform
                    .iter()
                    .chain(&value.layout_ancestor_transform)
                    .chain(
                        [
                            value.min_x,
                            value.min_y,
                            value.max_x,
                            value.max_y,
                            value.first_baseline,
                            value.layout_ancestor_min_x,
                            value.layout_ancestor_min_y,
                            value.layout_ancestor_max_x,
                            value.layout_ancestor_max_y,
                        ]
                        .iter(),
                    )
                    .all(|value| value.is_finite())
                {
                    return NuxStatus::RuntimeError;
                }
                write_caller_struct(out_geometry, &value, NUX_TEXT_INPUT_GEOMETRY_MIN_SIZE)
                    .map_or_else(|status| status, |()| NuxStatus::Ok)
            },
        )
    })
}

/// Read a root text run's settled geometry from the state named by `step`.
/// The successful step must belong to this player's artboard occurrence and
/// still name its current render revision. A mutation requires another step;
/// stale/foreign results return HANDLE_MISMATCH. No presentation acknowledgement
/// is required: hosts can capture this beside pixels before publishing a frame.
/// Hosts must publish/discard both using the returned render revision.
/// Exact duplicate names return INVALID_ARGUMENT; absent names return NOT_FOUND.
/// Output is written only on success. This call does not advance or mutate text.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_text_run_geometry(
    player: *const NuxPlayer,
    step: *const NuxPlayerStepResult,
    name: NuxStringView,
    out_geometry: *mut NuxTextRunGeometry,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _player = enter_status_handle!(player, HandleKind::Player);
        let _step = enter_status_handle!(step, HandleKind::PlayerStepResult);
        let player = unsafe { &*player };
        let step = unsafe { &*step };
        let _occurrence = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        if let Err(status) = player
            .artboard
            .refresh_renderer_domain_invalidation()
            .and_then(|()| player.artboard.refresh_bound_view_model_invalidation())
        {
            player.artboard.poisoned.set(true);
            return status;
        }
        if step.status != NuxStatus::Ok
            || !step
                .occurrence
                .upgrade()
                .is_some_and(|owner| Rc::ptr_eq(&owner, &player.artboard))
            || step.scheduling.render_revision == 0
            || step.scheduling.render_revision != player.artboard.render_revision.get()
        {
            return NuxStatus::HandleMismatch;
        }
        if name.len > 4096 {
            return NuxStatus::LimitExceeded;
        }
        let name = match with_utf8_view(name, str::to_owned) {
            Ok(name) if !name.is_empty() => name,
            Ok(_) => return NuxStatus::InvalidArgument,
            Err(status) => return status,
        };
        let owner = match data_binding::root_text_owner(&player.artboard, &name) {
            Ok(owner) => owner,
            Err(status) => return status,
        };
        let Some(mut value) = owner.with_downcast::<Text, _>(|text| {
            let world = *text.base.world_transform();
            let content = world * text.internal_transform();
            let bounds = text.local_bounds();
            let first_baseline = text.first_baseline();
            NuxTextRunGeometry {
                struct_size: std::mem::size_of::<NuxTextRunGeometry>() as u32,
                render_revision: step.scheduling.render_revision,
                world_transform: *world.values(),
                content_transform: *content.values(),
                min_x: bounds.min_x,
                min_y: bounds.min_y,
                max_x: bounds.max_x,
                max_y: bounds.max_y,
                has_first_baseline: u32::from(first_baseline.is_some()),
                first_baseline: first_baseline.unwrap_or(0.0),
                ..Default::default()
            }
        }) else {
            return NuxStatus::NotFound;
        };
        match nearest_layout_geometry(&owner) {
            Ok(Some((transform, bounds))) => {
                value.has_layout_ancestor = 1;
                value.layout_ancestor_transform = transform;
                [
                    value.layout_ancestor_min_x,
                    value.layout_ancestor_min_y,
                    value.layout_ancestor_max_x,
                    value.layout_ancestor_max_y,
                ] = bounds;
            }
            Ok(None) => {}
            Err(status) => return status,
        }
        if !value
            .world_transform
            .iter()
            .chain(&value.content_transform)
            .chain([value.min_x, value.min_y, value.max_x, value.max_y].iter())
            .chain(std::iter::once(&value.first_baseline))
            .chain(&value.layout_ancestor_transform)
            .chain(
                [
                    value.layout_ancestor_min_x,
                    value.layout_ancestor_min_y,
                    value.layout_ancestor_max_x,
                    value.layout_ancestor_max_y,
                ]
                .iter(),
            )
            .all(|number| number.is_finite())
        {
            return NuxStatus::RuntimeError;
        }
        unsafe { write_caller_struct(out_geometry, &value, NUX_TEXT_RUN_GEOMETRY_MIN_SIZE) }
            .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie::runtime::text::text_value_run::TextValueRun;
    #[allow(dead_code)]
    mod fixture {
        include!("../tests/support/semantic_text.rs");
    }

    fn name(value: &str) -> NuxStringView {
        NuxStringView {
            data: value.as_ptr().cast(),
            len: value.len(),
        }
    }

    #[test]
    fn native_input_scroll_uses_existing_constraint_without_editing_value() {
        unsafe {
            let bytes = fixture::scrolling_native_input_artboard();
            let mut file = ptr::null_mut();
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            let mut instance = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            let mut player = ptr::null_mut();
            assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            let mut result = ptr::null_mut();
            let step = NuxPlayerStep {
                struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                ..Default::default()
            };
            assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
            let mut scheduling = NuxPlayerSchedulingInfo {
                struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_player_step_result_scheduling(result, &mut scheduling),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_acknowledge_presented(player, scheduling.render_revision),
                NuxStatus::Ok
            );
            let mut snapshot = ptr::null_mut();
            assert_eq!(
                nux_player_semantic_snapshot(player, &mut snapshot),
                NuxStatus::Ok
            );
            let mut info = NuxSemanticSnapshotInfo {
                struct_size: std::mem::size_of::<NuxSemanticSnapshotInfo>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_semantic_snapshot_info(snapshot, &mut info),
                NuxStatus::Ok
            );
            let mut node = NuxSemanticNodeView {
                struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
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
            let mut before = NuxTextInputGeometry {
                struct_size: std::mem::size_of::<NuxTextInputGeometry>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_player_text_input_geometry(
                    player,
                    snapshot,
                    node.id,
                    name("editable"),
                    &mut before
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                before.layout_ancestor_max_x, 100.0,
                "Editing uses the viewport, not the 500-unit content extent"
            );
            assert_eq!(
                nux_player_text_input_content_offset_set(
                    player,
                    snapshot,
                    node.id,
                    name("editable"),
                    f32::NAN,
                    0.0
                ),
                NuxStatus::InvalidArgument
            );
            assert_eq!(
                nux_player_text_input_content_offset_set(
                    player,
                    snapshot,
                    node.id,
                    name("editable"),
                    0.0,
                    0.0
                ),
                NuxStatus::Ok
            );
            // A no-op leaves this capture usable; a real offset retires it.
            assert_eq!(
                nux_player_text_input_content_offset_set(
                    player,
                    snapshot,
                    node.id,
                    name("editable"),
                    70.0,
                    20.0
                ),
                NuxStatus::Ok
            );
            assert_ne!(
                nux_player_text_input_content_offset_set(
                    player,
                    snapshot,
                    node.id,
                    name("editable"),
                    80.0,
                    0.0
                ),
                NuxStatus::Ok
            );
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
            assert_eq!(
                nux_player_step_result_scheduling(result, &mut scheduling),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_acknowledge_presented(player, scheduling.render_revision),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_semantic_snapshot(player, &mut snapshot),
                NuxStatus::Ok
            );
            let mut after = NuxTextInputGeometry {
                struct_size: std::mem::size_of::<NuxTextInputGeometry>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_player_text_input_geometry(
                    player,
                    snapshot,
                    node.id,
                    name("editable"),
                    &mut after
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                after.world_transform, before.world_transform,
                "Native editor placement must not feed back its own scroll displacement"
            );
            assert_eq!(
                after.layout_ancestor_transform,
                before.layout_ancestor_transform
            );
            assert_eq!(
                semantic_snapshot::with_presented_field_property(
                    player,
                    snapshot,
                    node.id,
                    name("editable"),
                    false,
                    |_, input| {
                        use nuxie::runtime::text::text_input::TextInput;
                        assert_eq!(
                            input.with_downcast::<TextInput, _>(|input| input
                                .base
                                .text()
                                .to_owned()),
                            Some("unchanged".into())
                        );
                        let scroll = input_content_scroll(input).unwrap();
                        scroll.with(|value| {
                            let scroll = value.as_scroll_constraint().unwrap();
                            assert_eq!(scroll.authored_scroll_offset_x(), -70.0);
                            assert_eq!(
                                scroll.authored_scroll_offset_y(),
                                0.0,
                                "The horizontal field ignores the other axis"
                            );
                        });
                        NuxStatus::Ok
                    }
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_text_input_content_offset_set(
                    player,
                    snapshot,
                    node.id,
                    name("editable"),
                    0.0,
                    0.0
                ),
                NuxStatus::Ok
            );
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            assert_eq!(nux_player_free(player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(instance), NuxStatus::Ok);
            assert_eq!(nux_file_free(file), NuxStatus::Ok);
        }
    }

    #[test]
    fn native_input_geometry_preserves_layout_box_and_affine_basis() {
        for (bytes, artboard_index, expected_layout, expected_origin) in [
            (
                fixture::nested_layout_native_input_artboard(),
                0,
                [0.0, 2.0, -3.0, 0.0, 24.0, 24.0],
                [-9.0, 38.0],
            ),
            (
                fixture::nested_layout_native_input_occurrence(),
                1,
                [0.0, 1.0, -1.5, 0.0, 212.0, 162.0],
                [195.5, 169.0],
            ),
        ] {
            unsafe {
                let mut file = ptr::null_mut();
                assert_eq!(
                    nux_file_import(
                        bytes.as_ptr(),
                        bytes.len(),
                        &NuxRenderCallbacks::default(),
                        &mut file
                    ),
                    NuxStatus::Ok
                );
                let mut instance = ptr::null_mut();
                assert_eq!(
                    nux_artboard_instance_new(file, artboard_index, &mut instance),
                    NuxStatus::Ok
                );
                let mut player = ptr::null_mut();
                assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
                assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
                let step = NuxPlayerStep {
                    struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                    ..Default::default()
                };
                let mut result = ptr::null_mut();
                assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
                let mut scheduling = NuxPlayerSchedulingInfo {
                    struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_player_step_result_scheduling(result, &mut scheduling),
                    NuxStatus::Ok
                );
                assert_eq!(
                    nux_player_acknowledge_presented(player, scheduling.render_revision),
                    NuxStatus::Ok
                );
                let mut snapshot = ptr::null_mut();
                assert_eq!(
                    nux_player_semantic_snapshot(player, &mut snapshot),
                    NuxStatus::Ok
                );
                let mut node = NuxSemanticNodeView {
                    struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
                    ..Default::default()
                };
                let mut info = NuxSemanticSnapshotInfo {
                    struct_size: std::mem::size_of::<NuxSemanticSnapshotInfo>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_semantic_snapshot_info(snapshot, &mut info),
                    NuxStatus::Ok
                );
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
                let mut geometry = NuxTextInputGeometry {
                    struct_size: std::mem::size_of::<NuxTextInputGeometry>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_player_text_input_geometry(
                        player,
                        snapshot,
                        node.id,
                        name("editable"),
                        &mut geometry
                    ),
                    NuxStatus::Ok
                );
                assert_eq!(geometry.has_layout_ancestor, 1);
                assert_eq!(
                    [
                        geometry.layout_ancestor_min_x,
                        geometry.layout_ancestor_min_y,
                        geometry.layout_ancestor_max_x,
                        geometry.layout_ancestor_max_y
                    ],
                    [0.0, 0.0, 100.0, 40.0]
                );
                for (actual, expected) in geometry
                    .layout_ancestor_transform
                    .into_iter()
                    .zip(expected_layout)
                {
                    assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
                }
                assert!((geometry.world_transform[4] - expected_origin[0]).abs() < 0.0001);
                assert!((geometry.world_transform[5] - expected_origin[1]).abs() < 0.0001);
                assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
                assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
                assert_eq!(nux_player_free(player), NuxStatus::Ok);
                assert_eq!(nux_artboard_instance_free(instance), NuxStatus::Ok);
                assert_eq!(nux_file_free(file), NuxStatus::Ok);
            }
        }
    }

    #[test]
    fn native_input_geometry_reports_shaped_baseline_for_plain_secure_and_empty_text() {
        for obscured in [false, true] {
            for text in ["aaa", "a\na", ""] {
                unsafe {
                    let bytes = fixture::shaped_native_input(obscured, text);
                    let mut file = ptr::null_mut();
                    assert_eq!(
                        nux_file_import(
                            bytes.as_ptr(),
                            bytes.len(),
                            &NuxRenderCallbacks::default(),
                            &mut file
                        ),
                        NuxStatus::Ok
                    );
                    let mut instance = ptr::null_mut();
                    assert_eq!(
                        nux_artboard_instance_new(file, 0, &mut instance),
                        NuxStatus::Ok
                    );
                    let mut player = ptr::null_mut();
                    assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
                    assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
                    let step = NuxPlayerStep {
                        struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                        ..Default::default()
                    };
                    let mut result = ptr::null_mut();
                    assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
                    let mut scheduling = NuxPlayerSchedulingInfo {
                        struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
                        ..Default::default()
                    };
                    assert_eq!(
                        nux_player_step_result_scheduling(result, &mut scheduling),
                        NuxStatus::Ok
                    );
                    assert_eq!(
                        nux_player_acknowledge_presented(player, scheduling.render_revision),
                        NuxStatus::Ok
                    );
                    let mut snapshot = ptr::null_mut();
                    assert_eq!(
                        nux_player_semantic_snapshot(player, &mut snapshot),
                        NuxStatus::Ok
                    );
                    let mut node = NuxSemanticNodeView {
                        struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
                        ..Default::default()
                    };
                    assert_eq!(
                        nux_semantic_snapshot_node(snapshot, 0, &mut node),
                        NuxStatus::Ok
                    );
                    let mut geometry = NuxTextInputGeometry {
                        struct_size: std::mem::size_of::<NuxTextInputGeometry>() as u32,
                        ..Default::default()
                    };
                    assert_eq!(
                        nux_player_text_input_geometry(
                            player,
                            snapshot,
                            node.id,
                            name("editable"),
                            &mut geometry
                        ),
                        NuxStatus::Ok
                    );
                    assert_eq!(geometry.world_transform, [1.0, 0.0, 0.0, 1.0, 12.0, 18.0]);
                    assert_eq!(geometry.has_first_baseline, 1);
                    // Independent font-table oracle: Roboto hhea ascent 1900,
                    // unitsPerEm 2048, authored font size 24, top origin.
                    assert!(
                        (geometry.first_baseline - 1900.0 * 24.0 / 2048.0).abs() < 0.001,
                        "baseline {} for {text:?}, secure {obscured}",
                        geometry.first_baseline
                    );
                    assert_eq!(geometry.obscured, u32::from(obscured));
                    assert_eq!(geometry.multiline, 1);
                    assert!(geometry.max_y > geometry.min_y);
                    assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
                    assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
                    assert_eq!(nux_player_free(player), NuxStatus::Ok);
                    assert_eq!(nux_artboard_instance_free(instance), NuxStatus::Ok);
                    assert_eq!(nux_file_free(file), NuxStatus::Ok);
                }
            }
        }
    }

    #[test]
    fn nearest_layout_owner_preserves_its_affine_basis_and_separate_text_offset() {
        unsafe {
            let bytes = fixture::nested_layout_text_artboard();
            let mut file = ptr::null_mut();
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            let mut instance = ptr::null_mut();
            let mut player = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
            let step = NuxPlayerStep {
                struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
            let mut geometry = NuxTextRunGeometry {
                struct_size: std::mem::size_of::<NuxTextRunGeometry>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("field/name"), &mut geometry),
                NuxStatus::Ok
            );
            assert_eq!(geometry.has_layout_ancestor, 1);
            for (actual, expected) in geometry
                .layout_ancestor_transform
                .into_iter()
                .zip([0.0, 2.0, -3.0, 0.0, 24.0, 24.0])
            {
                assert!(
                    (actual - expected).abs() < 0.0001,
                    "layout basis: {actual} != {expected}"
                );
            }
            assert_eq!(
                [
                    geometry.layout_ancestor_min_x,
                    geometry.layout_ancestor_min_y,
                    geometry.layout_ancestor_max_x,
                    geometry.layout_ancestor_max_y
                ],
                [0.0, 0.0, 100.0, 40.0]
            );
            assert!((geometry.world_transform[4] + 9.0).abs() < 0.0001);
            assert!((geometry.world_transform[5] - 38.0).abs() < 0.0001);
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            assert_eq!(nux_player_free(player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(instance), NuxStatus::Ok);
            assert_eq!(nux_file_free(file), NuxStatus::Ok);
        }
    }

    #[test]
    fn geometry_read_rejects_foreign_stale_ambiguous_and_expired_results() {
        unsafe {
            let bytes =
                fixture::transformed_compound_text_artboard([24.0, 24.0, 0.0, 1.0, 1.0, 0.0, 0.0]);
            let mut file = ptr::null_mut();
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            let mut instance = ptr::null_mut();
            let mut player = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(instance, &mut player), NuxStatus::Ok);
            let step = NuxPlayerStep {
                struct_size: std::mem::size_of::<NuxPlayerStep>() as u32,
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
            let mut geometry = NuxTextRunGeometry {
                struct_size: std::mem::size_of::<NuxTextRunGeometry>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("field/name"), &mut geometry),
                NuxStatus::Ok
            );
            assert_eq!(geometry.world_transform[4], 24.0);
            assert_eq!(
                geometry.has_layout_ancestor, 0,
                "a Shape parent is not a layout owner"
            );
            assert_eq!(geometry.layout_ancestor_transform, [0.0; 6]);
            assert_eq!(
                geometry.has_first_baseline, 0,
                "fontless text has no shaped baseline"
            );
            assert_eq!(geometry.first_baseline, 0.0);
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("absent"), &mut geometry),
                NuxStatus::NotFound
            );
            assert_eq!(
                nux_player_text_run_geometry(player, result, name(""), &mut geometry),
                NuxStatus::InvalidArgument
            );
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("field/name"), ptr::null_mut()),
                NuxStatus::NullArgument
            );
            geometry.struct_size = 4;
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("field/name"), &mut geometry),
                NuxStatus::InvalidStructSize
            );
            geometry.struct_size = std::mem::size_of::<NuxTextRunGeometry>() as u32;
            let addresses = (player as usize, result as usize);
            assert_eq!(
                std::thread::spawn(move || {
                    let mut output = NuxTextRunGeometry {
                        struct_size: std::mem::size_of::<NuxTextRunGeometry>() as u32,
                        ..Default::default()
                    };
                    nux_player_text_run_geometry(
                        addresses.0 as *const NuxPlayer,
                        addresses.1 as *const NuxPlayerStepResult,
                        name("field/name"),
                        &mut output,
                    )
                })
                .join()
                .unwrap(),
                NuxStatus::WrongThread
            );
            let mut other_instance = ptr::null_mut();
            let mut other = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut other_instance),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_new_static(other_instance, &mut other),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_text_run_geometry(other, result, name("field/name"), &mut geometry),
                NuxStatus::HandleMismatch
            );
            assert_eq!(nux_player_free(other), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(other_instance), NuxStatus::Ok);

            let replacement = b"changed private value";
            let mutation = NuxTextRunMutation {
                name: name("field/name"),
                text: NuxByteView {
                    data: replacement.as_ptr(),
                    len: replacement.len(),
                },
            };
            let batch = NuxTextRunMutationBatch {
                mutations: &mutation,
                mutation_count: 1,
                ..Default::default()
            };
            assert_eq!(
                nux_artboard_instance_set_text_runs(instance, &batch, ptr::null_mut()),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("field/name"), &mut geometry),
                NuxStatus::HandleMismatch
            );
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("field/name"), &mut geometry),
                NuxStatus::Ok
            );
            let artboard = (&(*player).artboard).instance.borrow().native_handle();
            let runs = artboard.with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .filter(|o| o.is_type_of(TextValueRun::TYPE_KEY))
                    .cloned()
                    .collect::<Vec<_>>()
            });
            struct NameOnlyCallbacks;
            impl nuxie::runtime::generated::component_base::ComponentBaseCallbacks for NameOnlyCallbacks {
                fn notify_property_changed(&mut self, _: u16) {}
            }
            runs[1].with_mut(|run| {
                run.as_component_mut()
                    .unwrap()
                    .set_name("field/name".to_owned(), &mut NameOnlyCallbacks)
            });
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("field/name"), &mut geometry),
                NuxStatus::InvalidArgument
            );
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            assert_eq!(
                nux_player_text_run_geometry(player, result, name("field/name"), &mut geometry),
                NuxStatus::HandleMismatch
            );
            assert_eq!(nux_player_free(player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(instance), NuxStatus::Ok);
            assert_eq!(nux_file_free(file), NuxStatus::Ok);
        }
    }
}
