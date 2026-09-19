//! Frame-qualified geometry for native editors over exact-name root text runs.

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
                return NuxStatus::InvalidArgument;
            }
            if visited.len() > 16_384 {
                return NuxStatus::LimitExceeded;
            }
            if let Some((transform, bounds)) =
                current.with_downcast::<LayoutComponent, _>(|layout| {
                    (
                        *layout.shape_world_transform().values(),
                        layout.local_bounds(),
                    )
                })
            {
                value.has_layout_ancestor = 1;
                value.layout_ancestor_transform = transform;
                value.layout_ancestor_min_x = bounds.min_x;
                value.layout_ancestor_min_y = bounds.min_y;
                value.layout_ancestor_max_x = bounds.max_x;
                value.layout_ancestor_max_y = bounds.max_y;
                break;
            }
            ancestor = current
                .with(|object| {
                    object
                        .as_component()
                        .and_then(|component| component.parent_handle())
                })
                .flatten();
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
