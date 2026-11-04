//! The cull every draw of an Android MSAA frame records, on the stand-in
//! driver of vulkan_allocation_failure_test.rs, so it runs on any host
//! ([UNIV-5966](https://universe.basis.dev/issue/UNIV-5966)).
//!
//! A dynamic fill (stencilDynamicMidpointFans) sets cull with
//! vkCmdSetCullMode for each of its passes and ends on its reset pass, FRONT.
//! Upstream Vulkan then binds the next stroke's pipeline, whose cull is static
//! BACK, and relies on that bind to put BACK back. The Vulkan spec says it
//! does. The Android emulator's driver (gfxstream over MoltenVK 1.4.0) does
//! not: after a dynamic FRONT, a static BACK pipeline culls as FRONT, so the
//! stroke's body is culled and only slivers at its caps and joins draw. A
//! static NONE pipeline there draws unculled, and a pipeline that takes cull
//! as dynamic state draws with the cull set after its bind (play-2's nocull
//! and setcull switches).
//!
//! The test drives the frame's batches through the binder `submitDrawList`
//! uses, in the frame's order, and checks the cull in effect at each draw on
//! two driver models: the spec's, and that emulator's. Every draw must get its
//! own cull on both, and every pipeline that takes cull as dynamic state must
//! have it set after its bind before it draws.

use super::draw_pipeline_vulkan_decl::PipelineProps;
use super::msaa_strokes_pipeline_test::{draw_props, render_pass_options};
use super::pipeline_manager_vulkan_decl::{PipelineManagerVulkan, ShaderCompilationMode};
use super::render_context_vulkan_decl::{ColorRampPipeline, DriverWorkarounds, TessellatePipeline};
use super::render_context_vulkan_impl::{PipelineBinder, bindResourceTexturePipeline};
use super::vkutil_decl::vkCullMode;
use super::vulkan_allocation_failure_test::{
    FakeContext, PIPELINE_DRIVER, RecordedCommand, reset_driver,
};
use crate::mechanical_port::hairline_batch_test::null_device_platform_features;
use crate::mechanical_port::msaa_strokes_test::{
    BORDER_COLOR, STROKE_COLOR, android_frame, has, is_stroke,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DSDynamicFillPipelineStates, DrawContents, DrawType, IAABB, InterlockMode,
    drawTypeHasPipelineDynamicState,
};
use crate::mechanical_port::source::renderer::src::gpu_cpp::{get_cull_face, get_pipeline_state};
use ash::vk::{self, Handle};
use nuxie_render_api::StrokePosition;
use std::collections::BTreeMap;
use std::sync::Arc;

const FILL_COLOR: u32 = 0x8066_9933;

/// Which cull a draw gets from the commands recorded before it.
#[derive(Clone, Copy, Debug)]
enum DriverModel {
    /// Vulkan's rule: binding a pipeline applies its static state; dynamic
    /// state applies only while a pipeline that lists it is bound, and must
    /// have been set since the last bind of a pipeline with it static.
    Spec,
    /// What the emulator did in play-2: once vkCmdSetCullMode has been
    /// recorded, a pipeline with static cull other than NONE culls with the
    /// last cull set, not its own.
    Emulator,
}

struct BoundPipeline {
    static_cull: vk::CullModeFlags,
    dynamic_cull: bool,
}

/// Walks the recorded commands and returns one line per draw whose cull is
/// wrong, or which draws with dynamic cull that was never set for it.
fn wrong_culls(
    model: DriverModel,
    commands: &[RecordedCommand],
    pipelines: &BTreeMap<u64, BoundPipeline>,
) -> Vec<String> {
    let mut wrong = Vec::new();
    let mut bound: Option<&BoundPipeline> = None;
    // The last vkCmdSetCullMode, and whether it was recorded after the last
    // bind of a pipeline with static cull (the spec's validity rule).
    let mut last_set: Option<vk::CullModeFlags> = None;
    let mut set_is_valid = false;
    for command in commands {
        match command {
            RecordedCommand::BindPipeline(handle) => {
                let pipeline = &pipelines[handle];
                if !pipeline.dynamic_cull {
                    set_is_valid = false;
                }
                bound = Some(pipeline);
            }
            RecordedCommand::SetCullMode(cull) => {
                last_set = Some(*cull);
                set_is_valid = true;
            }
            RecordedCommand::Draw {
                what,
                expected_cull,
            } => {
                let Some(pipeline) = bound else {
                    wrong.push(format!("{model:?}: {what} draws with no pipeline bound"));
                    continue;
                };
                let in_effect = if pipeline.dynamic_cull {
                    if !set_is_valid {
                        wrong.push(format!(
                            "{model:?}: {what} takes cull as dynamic state, but none was set after its bind"
                        ));
                        continue;
                    }
                    last_set.unwrap()
                } else {
                    match (model, last_set) {
                        (DriverModel::Emulator, Some(set))
                            if pipeline.static_cull != vk::CullModeFlags::NONE =>
                        {
                            set
                        }
                        _ => pipeline.static_cull,
                    }
                };
                if in_effect != *expected_cull {
                    wrong.push(format!(
                        "{model:?}: {what} culls {in_effect:?}, its draw needs {expected_cull:?}"
                    ));
                }
            }
        }
    }
    wrong
}

fn draw(what: String, expected_cull: vk::CullModeFlags) {
    PIPELINE_DRIVER
        .lock()
        .unwrap()
        .commands
        .push(RecordedCommand::Draw {
            what,
            expected_cull,
        });
}

#[test]
fn every_msaa_draw_after_a_dynamic_fill_gets_its_own_cull() {
    let _guard = reset_driver();
    let mut wrong = Vec::new();
    for supports_pipeline_dynamic_state in [true, false] {
        // A translucent fill, a stroke over it, a CSS border (a clip stroke and
        // a fill inside it), then another stroke.
        let (flush, batches) = android_frame(
            supports_pipeline_dynamic_state,
            &[
                (10.0, FILL_COLOR, None),
                (10.0, STROKE_COLOR, Some((4.0, StrokePosition::Center))),
                (130.0, BORDER_COLOR, Some((6.0, StrokePosition::Inside))),
                (40.0, STROKE_COLOR, Some((2.0, StrokePosition::Center))),
            ],
        );
        let label = format!("dynamic state {supports_pipeline_dynamic_state}");
        for batch in &batches {
            eprintln!("{label}:   {batch:?}");
        }
        assert_eq!(flush.interlock_mode, InterlockMode::depthStencil, "{label}");
        let dynamic_fill = batches
            .iter()
            .position(|b| drawTypeHasPipelineDynamicState(b.draw_type));
        if supports_pipeline_dynamic_state {
            let Some(dynamic_fill) = dynamic_fill else {
                panic!("{label}: no dynamic fill: {batches:?}");
            };
            // The flush batches the two colored strokes together ahead of the
            // fills, so the stroke that follows a fill is the border's clip.
            assert!(
                batches
                    .iter()
                    .skip(dynamic_fill + 1)
                    .any(|b| is_stroke(b) && has(b, DrawContents::clipUpdate)),
                "{label}: no stroke after a dynamic fill: {batches:?}"
            );
        } else {
            assert!(dynamic_fill.is_none(), "{label}: {batches:?}");
        }
        let platform = null_device_platform_features(supports_pipeline_dynamic_state);
        let options = render_pass_options(&flush);
        let format = vk::Format::R8G8B8A8_UNORM;
        let vk_context = FakeContext::new();
        let manager = PipelineManagerVulkan::make(
            Arc::clone(&vk_context),
            ShaderCompilationMode::standard,
            vk::ImageView::from_raw(0x3_0000),
        )
        .expect("pipeline manager on the stand-in driver");
        let props: Vec<PipelineProps> = batches
            .iter()
            .map(|b| draw_props(&flush, b, options, format))
            .collect();
        // Let the specialized pipelines finish, as on a later frame.
        for p in &props {
            let _ = manager.tryGetPipeline(p, &platform);
        }
        manager.waitForAllBackgroundPipelineCreation();
        PIPELINE_DRIVER.lock().unwrap().commands.clear();

        let mut binder = PipelineBinder::new(&vk_context);
        let command = vk::CommandBuffer::from_raw(0x7_0000);
        for (i, (batch, p)) in batches.iter().zip(&props).enumerate() {
            let what = format!(
                "{label}, batch {i} {:?} {:?}",
                batch.draw_type, batch.draw_contents
            );
            let Some(pipeline) = manager.tryGetPipeline(p, &platform) else {
                wrong.push(format!("{what}: no pipeline"));
                continue;
            };
            binder.bindDrawPipeline(
                command,
                pipeline.m_vkPipeline,
                IAABB::default(),
                batch.draw_type,
                flush.interlock_mode,
                &platform,
            );
            if drawTypeHasPipelineDynamicState(batch.draw_type) {
                // submitDrawList's dynamic fill: each pass sets its state, then draws.
                for pass in DSDynamicFillPipelineStates::PassDrawTypes {
                    let state = get_pipeline_state(
                        pass,
                        flush.interlock_mode,
                        p.shaderMiscFlags,
                        batch.draw_contents,
                        flush.fixed_function_color_output,
                        batch.first_blend_mode,
                        &platform,
                    );
                    binder.setDynamicState(command, &state);
                    draw(format!("{what} pass {pass:?}"), vkCullMode(state.cullFace));
                }
            } else {
                draw(what, vkCullMode(get_cull_face(batch.draw_type)));
            }
        }

        let driver = PIPELINE_DRIVER.lock().unwrap();
        let mut pipelines = BTreeMap::new();
        for created in &driver.pipelines {
            let culls = created
                .dynamic_states
                .iter()
                .filter(|s| **s == vk::DynamicState::CULL_MODE)
                .count();
            if culls > 1 {
                wrong.push(format!("{label}: CULL_MODE listed twice: {created:?}"));
            }
            pipelines.insert(
                created.handle,
                BoundPipeline {
                    static_cull: created.cull_mode,
                    dynamic_cull: culls > 0,
                },
            );
        }
        let commands = driver.commands.clone();
        drop(driver);
        for command in &commands {
            eprintln!("{label}:   {command:?}");
        }
        for model in [DriverModel::Spec, DriverModel::Emulator] {
            wrong.extend(
                wrong_culls(model, &commands, &pipelines)
                    .into_iter()
                    .map(|line| format!("{label}: {line}")),
            );
        }
        if !supports_pipeline_dynamic_state {
            // Without pipeline dynamic state nothing sets cull: every
            // pipeline keeps its static cull, as upstream builds it.
            let bound_dynamic_cull = commands.iter().any(|c| match c {
                RecordedCommand::BindPipeline(handle) => pipelines[handle].dynamic_cull,
                _ => false,
            });
            if bound_dynamic_cull
                || commands
                    .iter()
                    .any(|c| matches!(c, RecordedCommand::SetCullMode(_)))
            {
                wrong.push(format!("{label}: cull became dynamic: {commands:?}"));
            }
        }
        drop(manager);
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A frame that splits into logical flushes records each flush's color ramp
/// and tessellation passes into the same command buffer, after the previous
/// flush's draw pass. Records a dynamic fill's passes (the end of a first
/// flush) and then each of those pipelines' binds, as the next flush does, and
/// checks both draw with their baked BACK on both driver models.
#[test]
fn a_later_flush_color_ramp_and_tessellation_after_a_dynamic_fill_cull_back() {
    let _guard = reset_driver();
    let mut wrong = Vec::new();
    let workarounds = DriverWorkarounds {
        maxInstancesPerRenderPass: u32::MAX,
        avoidManualMSAAResolves: false,
        needsManualMSAAResolveAfterDstRead: false,
        avoidDstReadFromNonMRTRenderTarget: false,
    };
    for supports_pipeline_dynamic_state in [true, false] {
        let label = format!("dynamic state {supports_pipeline_dynamic_state}");
        let (flush, batches) =
            android_frame(supports_pipeline_dynamic_state, &[(10.0, FILL_COLOR, None)]);
        let platform = null_device_platform_features(supports_pipeline_dynamic_state);
        let options = render_pass_options(&flush);
        let vk_context = FakeContext::new();
        let manager = PipelineManagerVulkan::make(
            Arc::clone(&vk_context),
            ShaderCompilationMode::standard,
            vk::ImageView::from_raw(0x3_0000),
        )
        .expect("pipeline manager on the stand-in driver");
        let color_ramp =
            ColorRampPipeline::make(&manager, workarounds, supports_pipeline_dynamic_state)
                .expect("color ramp pipeline on the stand-in driver");
        let tessellate =
            TessellatePipeline::make(&manager, workarounds, supports_pipeline_dynamic_state)
                .expect("tessellation pipeline on the stand-in driver");
        let fill = batches
            .iter()
            .find(|b| b.ranges > 0 && b.draw_type != DrawType::clipReset)
            .unwrap_or_else(|| panic!("{label}: no fill: {batches:?}"));
        assert_eq!(
            drawTypeHasPipelineDynamicState(fill.draw_type),
            supports_pipeline_dynamic_state,
            "{label}: {fill:?}"
        );
        let props = draw_props(&flush, fill, options, vk::Format::R8G8B8A8_UNORM);
        let _ = manager.tryGetPipeline(&props, &platform);
        manager.waitForAllBackgroundPipelineCreation();
        PIPELINE_DRIVER.lock().unwrap().commands.clear();

        // The first flush's last draw batch.
        let command = vk::CommandBuffer::from_raw(0x7_0000);
        let mut binder = PipelineBinder::new(&vk_context);
        let fill_pipeline = manager
            .tryGetPipeline(&props, &platform)
            .expect("the fill's pipeline");
        binder.bindDrawPipeline(
            command,
            fill_pipeline.m_vkPipeline,
            IAABB::default(),
            fill.draw_type,
            flush.interlock_mode,
            &platform,
        );
        if drawTypeHasPipelineDynamicState(fill.draw_type) {
            for pass in DSDynamicFillPipelineStates::PassDrawTypes {
                let state = get_pipeline_state(
                    pass,
                    flush.interlock_mode,
                    props.shaderMiscFlags,
                    fill.draw_contents,
                    flush.fixed_function_color_output,
                    fill.first_blend_mode,
                    &platform,
                );
                binder.setDynamicState(command, &state);
                draw(
                    format!("{label}: fill pass {pass:?}"),
                    vkCullMode(state.cullFace),
                );
            }
        } else {
            draw(
                format!("{label}: fill {:?}", fill.draw_type),
                vkCullMode(get_cull_face(fill.draw_type)),
            );
        }
        // The next flush's resource passes.
        for (name, pipeline) in [
            (
                "color ramp",
                (color_ramp.m_renderPipeline, color_ramp.m_cullIsDynamic),
            ),
            (
                "tessellation",
                (tessellate.m_renderPipeline, tessellate.m_cullIsDynamic),
            ),
        ] {
            bindResourceTexturePipeline(&vk_context, command, pipeline.0, pipeline.1);
            draw(
                format!("{label}: next flush's {name}"),
                vk::CullModeFlags::BACK,
            );
        }

        let driver = PIPELINE_DRIVER.lock().unwrap();
        let pipelines: BTreeMap<u64, BoundPipeline> = driver
            .pipelines
            .iter()
            .map(|created| {
                (
                    created.handle,
                    BoundPipeline {
                        static_cull: created.cull_mode,
                        dynamic_cull: created
                            .dynamic_states
                            .contains(&vk::DynamicState::CULL_MODE),
                    },
                )
            })
            .collect();
        let commands = driver.commands.clone();
        drop(driver);
        for command in &commands {
            eprintln!("{label}:   {command:?}");
        }
        for (name, handle) in [
            ("color ramp", color_ramp.m_renderPipeline),
            ("tessellation", tessellate.m_renderPipeline),
        ] {
            let p = &pipelines[&handle.as_raw()];
            if p.static_cull != vk::CullModeFlags::BACK {
                wrong.push(format!(
                    "{label}: {name} bakes {:?}, not BACK",
                    p.static_cull
                ));
            }
            if p.dynamic_cull != supports_pipeline_dynamic_state {
                wrong.push(format!("{label}: {name} dynamic cull {}", p.dynamic_cull));
            }
        }
        for model in [DriverModel::Spec, DriverModel::Emulator] {
            wrong.extend(
                wrong_culls(model, &commands, &pipelines)
                    .into_iter()
                    .map(|line| format!("{label}: {line}")),
            );
        }
        if !supports_pipeline_dynamic_state
            && commands
                .iter()
                .any(|c| matches!(c, RecordedCommand::SetCullMode(_)))
        {
            wrong.push(format!("{label}: cull was set: {commands:?}"));
        }
        drop(color_ramp);
        drop(tessellate);
        drop(manager);
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
