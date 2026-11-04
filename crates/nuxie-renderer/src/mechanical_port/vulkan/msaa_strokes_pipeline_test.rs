//! Android's MSAA strokes through the Vulkan pipeline manager, on the stand-in
//! driver of vulkan_allocation_failure_test.rs, so it runs on any host.
//!
//! The stroke batches come from a real flush (`msaa_strokes_test`'s NULL
//! device). Each is turned into the PipelineProps `flush()` builds for it
//! (render_context_vulkan_impl_cpp__impl.rs, the per-batch tryGetPipeline
//! call), then asked for the way the draw asks: a stroke whose
//! tryGetPipeline returns None is skipped without a draw. The test checks the
//! pipeline is found, and that the driver was asked for the depth-stroke state
//! upstream's draw_pipeline_vulkan.cpp builds: the DS_POLAR_STROKE
//! specialization, depth test and write at GREATER, 4 samples, the
//! depth-stencil path shaders, and color writes only when the stroke draws
//! color.

use super::draw_pipeline_vulkan_decl::{DrawPipelineOptions, PipelineProps};
use super::pipeline_manager_vulkan_decl::{PipelineManagerVulkan, ShaderCompilationMode};
use super::render_pass_vulkan_decl::RenderPassOptionsVulkan;
use super::vulkan_allocation_failure_test::{
    CreatedPipeline, FakeContext, PIPELINE_DRIVER, reset_driver,
};
use super::vulkan_shaders_decl as spirv;
use crate::mechanical_port::hairline_batch_test::{
    CapturedBatch, CapturedFlush, null_device_platform_features,
};
use crate::mechanical_port::msaa_strokes_test::{android_stroke_frame, has, is_stroke};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawContents, DrawType, InterlockMode, LoadAction, ShaderFeatures, ShaderMiscFlags,
};
use ash::vk::{self, Handle};
use std::sync::Arc;

/// The render pass options `flush()` starts from for this flush.
pub(super) fn render_pass_options(flush: &CapturedFlush) -> RenderPassOptionsVulkan {
    let mut options = RenderPassOptionsVulkan::none;
    if flush.fixed_function_color_output {
        options |= RenderPassOptionsVulkan::fixedFunctionColorOutput;
    }
    if flush.msaa_sample_count > 1 {
        options |= RenderPassOptionsVulkan::msaa;
    }
    if flush.manually_resolved {
        options |= RenderPassOptionsVulkan::manuallyResolved;
    }
    if options.has(RenderPassOptionsVulkan::msaa)
        && flush.color_load_action == LoadAction::preserveRenderTarget
    {
        options |= RenderPassOptionsVulkan::msaaSeedFromOffscreenTexture;
    }
    options
}

/// The PipelineProps `flush()` passes to tryGetPipeline for this batch.
pub(super) fn draw_props(
    flush: &CapturedFlush,
    batch: &CapturedBatch,
    options: RenderPassOptionsVulkan,
    format: vk::Format,
) -> PipelineProps {
    let shader_features = if flush.interlock_mode == InterlockMode::atomics {
        flush.combined_shader_features
    } else {
        batch.shader_features
    };
    let mut misc = batch.shader_misc_flags;
    if options.has(RenderPassOptionsVulkan::msaa)
        && !misc.has(ShaderMiscFlags::fixedFunctionColorOutput)
        && batch.draw_type != DrawType::renderPassInitialize
    {
        misc |= ShaderMiscFlags::msaaDstRead;
    }
    PipelineProps {
        drawType: batch.draw_type,
        shaderFeatures: shader_features,
        interlockMode: flush.interlock_mode,
        shaderMiscFlags: misc,
        drawContents: batch.draw_contents,
        blendMode: batch.first_blend_mode,
        drawPipelineOptions: DrawPipelineOptions::none,
        renderPassOptions: options,
        renderTargetFormat: format,
        colorLoadAction: flush.color_load_action,
        #[cfg(feature = "with-rive-tools")]
        synthesizedFailureType: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType::none,
    }
}

fn created(handle: vk::Pipeline) -> Option<CreatedPipeline> {
    PIPELINE_DRIVER
        .lock()
        .unwrap()
        .pipelines
        .iter()
        .find(|p| p.handle == handle.as_raw())
        .cloned()
}

fn module_words(handle: u64) -> Vec<u32> {
    PIPELINE_DRIVER.lock().unwrap().modules[&handle].clone()
}

/// What the driver must have been asked for, by upstream's construction.
fn check_stroke_pipeline(
    what: &str,
    props: &PipelineProps,
    features: ShaderFeatures,
    pipeline: &CreatedPipeline,
    wrong: &mut Vec<String>,
) {
    let mut expect = |ok: bool, claim: String| {
        if !ok {
            wrong.push(format!("{what}: {claim}: {pipeline:?}"));
        }
    };
    expect(
        pipeline.specialization.get(16) == Some(&1),
        "DS_POLAR_STROKE (slot 16) is 1".into(),
    );
    expect(
        pipeline.specialization.get(17) == Some(&0),
        "DS_HAIRLINE_STROKE (slot 17) is 0".into(),
    );
    expect(
        pipeline.depth_test && pipeline.depth_write,
        "depth test and write on".into(),
    );
    expect(
        pipeline.depth_compare == vk::CompareOp::GREATER,
        "depth compare GREATER".into(),
    );
    expect(
        pipeline.samples == vk::SampleCountFlags::TYPE_4,
        "4 samples".into(),
    );
    let clip_update = props.drawContents.0 & DrawContents::clipUpdate.0 != 0;
    let color = pipeline
        .color_write_masks
        .first()
        .copied()
        .unwrap_or(vk::ColorComponentFlags::empty());
    if clip_update {
        expect(color.is_empty(), "a clip stroke writes no color".into());
        expect(pipeline.stencil_test, "a clip stroke writes stencil".into());
    } else {
        expect(
            color == vk::ColorComponentFlags::RGBA,
            "a colored stroke writes RGBA".into(),
        );
    }
    let vert = if features.0 & ShaderFeatures::ENABLE_CLIP_RECT.0 != 0 {
        spirv::draw_depthstencil_path_vert.read().unwrap().to_vec()
    } else {
        spirv::draw_depthstencil_path_noclipdistance_vert
            .read()
            .unwrap()
            .to_vec()
    };
    let frag = if props
        .renderPassOptions
        .has(RenderPassOptionsVulkan::fixedFunctionColorOutput)
        || props
            .shaderMiscFlags
            .has(ShaderMiscFlags::fixedFunctionColorOutput)
    {
        spirv::draw_depthstencil_path_fixedcolor_frag
            .read()
            .unwrap()
            .to_vec()
    } else if props.shaderMiscFlags.has(ShaderMiscFlags::msaaDstRead) {
        spirv::draw_depthstencil_path_msaa_frag
            .read()
            .unwrap()
            .to_vec()
    } else {
        spirv::draw_depthstencil_path_frag.read().unwrap().to_vec()
    };
    expect(
        module_words(pipeline.vertex_module) == vert,
        "vertex module is the depth-stencil path shader".into(),
    );
    expect(
        module_words(pipeline.fragment_module) == frag,
        "fragment module is the depth-stencil path shader".into(),
    );
}

#[test]
fn msaa_strokes_ask_for_build_and_find_their_depth_stroke_pipelines() {
    let _guard = reset_driver();
    let mut wrong = Vec::new();
    for supports_pipeline_dynamic_state in [true, false] {
        let (flush, batches) = android_stroke_frame(supports_pipeline_dynamic_state);
        let platform = null_device_platform_features(supports_pipeline_dynamic_state);
        let options = render_pass_options(&flush);
        let strokes: Vec<_> = batches.iter().filter(|b| is_stroke(b)).collect();
        assert!(
            strokes.iter().any(|b| !has(b, DrawContents::clipUpdate))
                && strokes.iter().any(|b| has(b, DrawContents::clipUpdate)),
            "a colored and a clip stroke batch: {batches:?}"
        );
        for format in [vk::Format::B8G8R8A8_UNORM, vk::Format::R8G8B8A8_UNORM] {
            let vk_context = FakeContext::new();
            let manager = PipelineManagerVulkan::make(
                Arc::clone(&vk_context),
                ShaderCompilationMode::standard,
                vk::ImageView::from_raw(0x3_0000),
            )
            .expect("pipeline manager on the stand-in driver");
            for batch in &strokes {
                let what = format!(
                    "dynamic state {supports_pipeline_dynamic_state}, {format:?}, {:?} {:?}",
                    batch.draw_type, batch.draw_contents
                );
                let props = draw_props(&flush, batch, options, format);
                // The first frame: a specialized pipeline compiles in the
                // background while the ubershader draws.
                let first = manager
                    .tryGetPipeline(&props, &platform)
                    .map(|p| p.m_vkPipeline);
                eprintln!("{what}: first ask {first:?}");
                let Some(first) = first.filter(|p| *p != vk::Pipeline::null()) else {
                    wrong.push(format!(
                        "{what}: first frame finds no pipeline; the draw skips it"
                    ));
                    continue;
                };
                match created(first) {
                    Some(pipeline) => {
                        let ubershader = pipeline.specialization.clone();
                        eprintln!("{what}: first-frame pipeline {pipeline:?}");
                        // The ubershader turns on every feature the draw allows.
                        let features = ShaderFeatures(ubershader.iter().take(10).enumerate().fold(
                            0,
                            |bits, (i, on)| if *on != 0 { bits | 1 << i } else { bits },
                        ));
                        check_stroke_pipeline(&what, &props, features, &pipeline, &mut wrong);
                    }
                    None => wrong.push(format!("{what}: {first:?} was never created")),
                }
                manager.waitForAllBackgroundPipelineCreation();
                let second = manager
                    .tryGetPipeline(&props, &platform)
                    .map(|p| p.m_vkPipeline);
                eprintln!("{what}: after background compilation {second:?}");
                let Some(second) = second.filter(|p| *p != vk::Pipeline::null()) else {
                    wrong.push(format!(
                        "{what}: later frames find no pipeline; the draw skips it"
                    ));
                    continue;
                };
                match created(second) {
                    Some(pipeline) => {
                        eprintln!("{what}: specialized pipeline {pipeline:?}");
                        check_stroke_pipeline(
                            &what,
                            &props,
                            props.shaderFeatures,
                            &pipeline,
                            &mut wrong,
                        );
                    }
                    None => wrong.push(format!("{what}: {second:?} was never created")),
                }
                // Each later frame finds that same pipeline and builds nothing new.
                let built = PIPELINE_DRIVER.lock().unwrap().pipelines.len();
                let third = manager
                    .tryGetPipeline(&props, &platform)
                    .map(|p| p.m_vkPipeline);
                if third != Some(second) {
                    wrong.push(format!(
                        "{what}: next frame finds {third:?}, not {second:?}"
                    ));
                }
                if PIPELINE_DRIVER.lock().unwrap().pipelines.len() != built {
                    wrong.push(format!("{what}: next frame built another pipeline"));
                }
            }
            drop(manager);
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
