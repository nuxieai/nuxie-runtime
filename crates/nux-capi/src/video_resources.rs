//! Shared decoder admission and safe reclamation for platform SDK hosts.
use super::*;
use nuxie::video::{
    Video,
    resources::{Allocation, DecoderBudget, DecoderRequest, allocate},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NuxVideoDecoderBudget {
    pub struct_size: u32,
    pub max_players: u32,
    pub managed_players: u32,
    pub hardware_players: u32,
    pub managed_pixels_per_second: u64,
    pub software_pixels_per_second: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NuxVideoDecoderRequest {
    pub struct_size: u32,
    pub id: u64,
    pub pixels_per_second: u64,
    pub priority: u32,
    /// Visible=1, hardware supported=2, software supported=4, platform managed=8.
    pub flags: u32,
}

/// Allocate a host-measured budget across visible video occurrences. Higher
/// priorities win; IDs break ties. This does not create or mutate decoders.
/// Output has one value per input, in input order: hardware=0, software=1,
/// poster=2, platform-managed=3. IDs must be unique and count <= 65536.
/// Hardware/managed/software availability and pixel rates come from the host,
/// not the renderer. A platform-managed allocation makes no hardware claim.
/// The host closes denied/replaced decoders before opening admitted ones.
/// Arrays must have count readable/writable elements; output is unchanged on
/// validation failure. Empty arrays may be null. Each struct_size equals sizeof its declared type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_video_allocate_decoders(
    requests: *const NuxVideoDecoderRequest,
    count: usize,
    budget: *const NuxVideoDecoderBudget,
    out_allocations: *mut u32,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if budget.is_null() || (count > 0 && (requests.is_null() || out_allocations.is_null())) {
            return NuxStatus::NullArgument;
        }
        if count > 65_536 {
            return NuxStatus::LimitExceeded;
        }
        let budget = unsafe { *budget };
        if budget.struct_size as usize != size_of::<NuxVideoDecoderBudget>() {
            return NuxStatus::InvalidArgument;
        }
        if count == 0 {
            return NuxStatus::Ok;
        }
        let input = unsafe { std::slice::from_raw_parts(requests, count) };
        let mut ids = std::collections::BTreeSet::new();
        let mut decoded = Vec::with_capacity(count);
        for request in input {
            if request.struct_size as usize != size_of::<NuxVideoDecoderRequest>()
                || request.flags & !15 != 0
                || !ids.insert(request.id)
            {
                return NuxStatus::InvalidArgument;
            }
            decoded.push(DecoderRequest {
                id: request.id,
                priority: request.priority,
                visible: request.flags & 1 != 0,
                hardware_supported: request.flags & 2 != 0,
                software_supported: request.flags & 4 != 0,
                managed_supported: request.flags & 8 != 0,
                pixels_per_second: request.pixels_per_second,
            });
        }
        let choices: std::collections::BTreeMap<_, _> = allocate(
            &decoded,
            DecoderBudget {
                max_players: budget.max_players as usize,
                managed_players: budget.managed_players as usize,
                hardware_players: budget.hardware_players as usize,
                managed_pixels_per_second: budget.managed_pixels_per_second,
                software_pixels_per_second: budget.software_pixels_per_second,
            },
        )
        .into_iter()
        .collect();
        // Compute every result before touching caller memory (input may alias output).
        let output: Vec<_> = decoded
            .iter()
            .map(|request| match choices[&request.id] {
                Allocation::Hardware => 0,
                Allocation::Software => 1,
                Allocation::Poster => 2,
                Allocation::PlatformManaged => 3,
            })
            .collect();
        unsafe {
            std::ptr::copy_nonoverlapping(output.as_ptr(), out_allocations, count);
        }
        NuxStatus::Ok
    })
}

/// After synchronously closing an occurrence's decoder, invalidate its old
/// callbacks and clear its frame while retaining position/settings/play intent.
/// blocked=1 selects its poster and resource suspension; blocked=0 releases only
/// that suspension. Other lifecycle suspensions remain. The returned generation
/// must be used by the replacement decoder. Do not call this on every frame:
/// call once per decoder retirement/admission transition, after draining commands.
/// A later ready observation seeks to the retained position before playing.
/// Failed/disposed occurrences cannot be revived. Output is unchanged on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_video_reclaim_decoder(
    player: *const NuxPlayer,
    component_id: usize,
    blocked: u32,
    out_generation: *mut u64,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        status(video::with_video(
            player,
            component_id,
            |video, occurrence| {
                if out_generation.is_null() {
                    return Err(NuxStatus::NullArgument);
                }
                if blocked > 1 {
                    return Err(NuxStatus::InvalidArgument);
                }
                let generation = video
                    .with_downcast_mut::<Video, _>(|video| {
                        let generation = video
                            .playback
                            .reclaim_decoder()
                            .map_err(|_| NuxStatus::RuntimeError)?;
                        video.clear_frame();
                        video.apply_allocation(if blocked == 1 {
                            Allocation::Poster
                        } else {
                            Allocation::PlatformManaged
                        });
                        Ok(generation)
                    })
                    .ok_or(NuxStatus::NotFound)??;
                occurrence.commit_runtime_change(true)?;
                unsafe {
                    *out_generation = generation;
                }
                Ok(())
            },
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reclamation_rejects_stale_callbacks_and_preserves_intent_and_lifecycle() {
        use nuxie::video::playback::{Command, DecoderAction, SuspensionReason};
        use nuxie_binary::{FixtureProperty as P, FixtureRecord as R, FixtureValue as V};
        let records = vec![
            R {
                type_key: 23,
                properties: vec![],
            },
            R {
                type_key: 60000,
                properties: vec![P {
                    key: 60000,
                    value: V::String("asset:clip".into()),
                }],
            },
            R {
                type_key: 1,
                properties: vec![],
            },
            R {
                type_key: 60001,
                properties: vec![
                    P {
                        key: 5,
                        value: V::Uint(0),
                    },
                    P {
                        key: 206,
                        value: V::Uint(0),
                    },
                ],
            },
        ];
        let bytes = nuxie_binary::encode_runtime_file(
            &nuxie_binary::RuntimeFile::from_fixture_records(records).unwrap(),
        )
        .unwrap();
        let (mut file, mut artboard, mut player, mut result) = (
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        unsafe {
            let capabilities = NuxVideoPlaybackCapabilities {
                playback_available: 1,
                ..Default::default()
            };
            assert_eq!(
                nux_file_import_with_video_capabilities(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &capabilities,
                    &mut file,
                    &mut result
                ),
                NuxStatus::Ok
            );
            nux_capi_result_free(result);
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut artboard),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(artboard, &mut player), NuxStatus::Ok);
            let old = video::with_video(player, 1, |handle, _| {
                Ok(handle
                    .with_downcast_mut::<Video, _>(|v| {
                        let generation = v.playback.generation();
                        v.playback.opened(generation, 10.0);
                        v.playback.enqueue(Command::Seek(2.0)).unwrap();
                        v.playback.enqueue(Command::Play).unwrap();
                        v.playback.drain_actions();
                        v.playback
                            .update_suspension(SuspensionReason::Background, true);
                        v.playback.generation()
                    })
                    .unwrap())
            })
            .unwrap();
            let mut generation = 999;
            assert_eq!(
                nux_player_video_reclaim_decoder(player, 1, 2, &mut generation),
                NuxStatus::InvalidArgument
            );
            assert_eq!(generation, 999);
            assert_eq!(
                nux_player_video_reclaim_decoder(player, 1, 1, &mut generation),
                NuxStatus::Ok
            );
            assert_eq!(generation, old + 1);
            video::with_video(player, 1, |handle, _| {
                Ok(handle
                    .with_downcast_mut::<Video, _>(|v| {
                        assert!(v.playback.wants_play());
                        assert_eq!(v.playback.position(), 2.0);
                        assert!(!v.has_video_frame());
                        assert!(v.playback.opened(old, 10.0).is_empty());
                        assert!(!v.playback.accept_frame(old, 2.0));
                    })
                    .unwrap())
            })
            .unwrap();
            assert_eq!(
                nux_player_video_reclaim_decoder(player, 1, 0, &mut generation),
                NuxStatus::Ok
            );
            video::with_video(player, 1, |handle, _| {
                Ok(handle
                    .with_downcast_mut::<Video, _>(|v| {
                        let actions = v.playback.opened(generation, 10.0);
                        assert!(actions.iter().any(
                            |a| matches!(a, DecoderAction::Seek { seconds, .. } if *seconds == 2.0)
                        ));
                        assert!(!actions.iter().any(|a| matches!(a, DecoderAction::Play)));
                        v.playback.accept_frame(generation, 2.0);
                        let resume = v
                            .playback
                            .update_suspension(SuspensionReason::Background, false);
                        assert!(resume.iter().any(|a| matches!(a, DecoderAction::Play)));
                    })
                    .unwrap())
            })
            .unwrap();
            nux_player_free(player);
            nux_artboard_instance_free(artboard);
            nux_file_free(file);
        }
    }

    #[test]
    fn admission_preserves_input_order_and_enforces_priority_visibility_and_cost() {
        let request = |id, priority, flags, pixels| NuxVideoDecoderRequest {
            struct_size: size_of::<NuxVideoDecoderRequest>() as u32,
            id,
            priority,
            flags,
            pixels_per_second: pixels,
        };
        let input = [
            request(9, 0, 15, 100),
            request(4, 10, 15, 100),
            request(3, 10, 9, 100),
            request(1, 99, 14, 1),
        ];
        let budget = NuxVideoDecoderBudget {
            max_players: 2,
            hardware_players: 1,
            managed_players: 1,
            struct_size: size_of::<NuxVideoDecoderBudget>() as u32,
            managed_pixels_per_second: 100,
            software_pixels_per_second: 100,
        };
        let mut output = [99; 4];
        assert_eq!(
            unsafe {
                nux_video_allocate_decoders(
                    input.as_ptr(),
                    input.len(),
                    &budget,
                    output.as_mut_ptr(),
                )
            },
            NuxStatus::Ok
        );
        assert_eq!(output, [2, 0, 3, 2]);
        let budget = NuxVideoDecoderBudget {
            max_players: 3,
            hardware_players: 0,
            managed_players: 0,
            software_pixels_per_second: 150,
            ..budget
        };
        assert_eq!(
            unsafe {
                nux_video_allocate_decoders(
                    input.as_ptr(),
                    input.len(),
                    &budget,
                    output.as_mut_ptr(),
                )
            },
            NuxStatus::Ok
        );
        assert_eq!(output, [2, 1, 2, 2]);
    }
    #[test]
    fn invalid_admission_never_writes_partial_output() {
        let budget = NuxVideoDecoderBudget {
            max_players: 1,
            hardware_players: 1,
            managed_players: 0,
            struct_size: size_of::<NuxVideoDecoderBudget>() as u32,
            managed_pixels_per_second: 0,
            software_pixels_per_second: 0,
        };
        let valid = NuxVideoDecoderRequest {
            struct_size: size_of::<NuxVideoDecoderRequest>() as u32,
            id: 1,
            priority: 0,
            flags: 3,
            pixels_per_second: 1,
        };
        let mut output = [77; 2];
        for invalid in [
            valid,
            NuxVideoDecoderRequest {
                id: 2,
                flags: 16,
                ..valid
            },
        ] {
            let input = [valid, invalid];
            assert_eq!(
                unsafe {
                    nux_video_allocate_decoders(input.as_ptr(), 2, &budget, output.as_mut_ptr())
                },
                NuxStatus::InvalidArgument
            );
            assert_eq!(output, [77; 2]);
        }
        assert_eq!(
            unsafe {
                nux_video_allocate_decoders(std::ptr::null(), 0, &budget, std::ptr::null_mut())
            },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_video_allocate_decoders(&valid, 65_537, &budget, output.as_mut_ptr()) },
            NuxStatus::LimitExceeded
        );
        assert_eq!(output, [77; 2]);
    }
}
