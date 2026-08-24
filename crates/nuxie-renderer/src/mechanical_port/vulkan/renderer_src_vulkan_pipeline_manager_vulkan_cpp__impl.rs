//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/pipeline_manager_vulkan.cpp`, together with the exact
//! type-specialized behavior inherited from pinned
//! `renderer/include/rive/renderer/async_pipeline_manager.hpp`.

#![allow(non_snake_case)]

use super::draw_pipeline_layout_vulkan_decl::DrawPipelineLayoutVulkan;
use super::draw_pipeline_vulkan_decl::{DrawPipelineOptions, DrawPipelineVulkan, PipelineProps};
use super::draw_shader_vulkan_decl::{DrawShaderVulkan, DrawShaderVulkanType};
use super::pipeline_manager_vulkan_decl::{
    CompletedJob, JobParams, PLSBackingType, PipelineCreateType, PipelineManagerVulkan,
    PipelineStatus, ShaderCompilationMode, MAX_SAMPLER_PERMUTATIONS,
};
use super::render_pass_vulkan_decl::{
    RenderPassOptionsVulkan, RenderPassVulkan, RENDER_PASS_OPTIONS_LAYOUT_MASK,
    RENDER_PASS_OPTION_COUNT,
};
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::{
    ImageFilter, ImageSampler, ImageWrap,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    kVertexShaderFeaturesMask, DrawContents, DrawType, InterlockMode, LoadAction, PlatformFeatures,
    ShaderFeatures, ShaderMiscFlags, UbershaderFeaturesMaskFor,
    DRAW_CONTENTS_FOR_MSAA_PIPELINE_STATE,
};
use crate::mechanical_port::source::renderer::src::gpu_cpp::{
    get_stencil_info, ForEachUbershaderPermutation, ShaderUniqueKey,
};
use ash::vk;
use nuxie_render_api::BlendMode;
use std::pin::Pin;
use std::sync::Arc;

const FLUSH_UNIFORM_BUFFER_IDX: u32 = 0;
const PATH_BUFFER_IDX: u32 = 2;
const PAINT_BUFFER_IDX: u32 = 3;
const PAINT_AUX_BUFFER_IDX: u32 = 4;
const CONTOUR_BUFFER_IDX: u32 = 5;
const COVERAGE_BUFFER_IDX: u32 = 6;
const TESS_VERTEX_TEXTURE_IDX: u32 = 7;
const GRAD_TEXTURE_IDX: u32 = 8;
const GAUSSIAN_INTEGRAL_TEXTURE_IDX: u32 = 9;
const FEATHER_ATLAS_TEXTURE_IDX: u32 = 10;
const IMAGE_TEXTURE_IDX: u32 = 11;

fn vk_sampler_mipmap_mode(option: ImageFilter) -> vk::SamplerMipmapMode {
    if option == ImageFilter::nearest || option == ImageFilter::bilinear {
        vk::SamplerMipmapMode::NEAREST
    } else {
        panic!("RIVE_UNREACHABLE in vk_sampler_mipmap_mode")
    }
}

fn vk_sampler_address_mode(option: ImageWrap) -> vk::SamplerAddressMode {
    if option == ImageWrap::clamp {
        vk::SamplerAddressMode::CLAMP_TO_EDGE
    } else if option == ImageWrap::repeat {
        vk::SamplerAddressMode::REPEAT
    } else if option == ImageWrap::mirror {
        vk::SamplerAddressMode::MIRRORED_REPEAT
    } else {
        panic!("RIVE_UNREACHABLE in vk_sampler_address_mode")
    }
}

fn vk_filter(option: ImageFilter) -> vk::Filter {
    if option == ImageFilter::bilinear {
        vk::Filter::LINEAR
    } else if option == ImageFilter::nearest {
        vk::Filter::NEAREST
    } else {
        panic!("RIVE_UNREACHABLE in vk_filter")
    }
}

#[cold]
#[track_caller]
fn vk_check<T>(result: Result<T, vk::Result>) -> T {
    match result {
        Ok(value) => value,
        Err(result) => super::vkutil_impl::vk_abort(result, file!(), line!()),
    }
}

impl PipelineManagerVulkan {
    pub(crate) fn new(
        vk: Arc<VulkanContext>,
        mode: ShaderCompilationMode,
        nullTextureView: vk::ImageView,
    ) -> Pin<Box<Self>> {
        let linearInfo = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .min_lod(0.0)
            .max_lod(0.0);
        let linearSampler = vk_check(unsafe { vk.m_ashDevice.create_sampler(&linearInfo, None) });
        let mut imageSamplers = [vk::Sampler::null(); MAX_SAMPLER_PERMUTATIONS];
        for (i, sampler) in imageSamplers.iter_mut().enumerate() {
            let wrapX = ImageSampler::GetWrapXOptionFromKey(i as u8);
            let wrapY = ImageSampler::GetWrapYOptionFromKey(i as u8);
            let filter = ImageSampler::GetFilterOptionFromKey(i as u8);
            let minMagFilter = vk_filter(filter);
            let info = vk::SamplerCreateInfo::default()
                .mag_filter(minMagFilter)
                .min_filter(minMagFilter)
                .mipmap_mode(vk_sampler_mipmap_mode(filter))
                .address_mode_u(vk_sampler_address_mode(wrapX))
                .address_mode_v(vk_sampler_address_mode(wrapY))
                .min_lod(0.0)
                .max_lod(vk::LOD_CLAMP_NONE);
            *sampler = vk_check(unsafe { vk.m_ashDevice.create_sampler(&info, None) });
        }

        let vertexFragment = vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT;
        let perFlushBindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(FLUSH_UNIFORM_BUFFER_IDX)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vertexFragment),
            vk::DescriptorSetLayoutBinding::default()
                .binding(PATH_BUFFER_IDX)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
            vk::DescriptorSetLayoutBinding::default()
                .binding(PAINT_BUFFER_IDX)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vertexFragment),
            vk::DescriptorSetLayoutBinding::default()
                .binding(PAINT_AUX_BUFFER_IDX)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vertexFragment),
            vk::DescriptorSetLayoutBinding::default()
                .binding(CONTOUR_BUFFER_IDX)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
            vk::DescriptorSetLayoutBinding::default()
                .binding(COVERAGE_BUFFER_IDX)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            vk::DescriptorSetLayoutBinding::default()
                .binding(TESS_VERTEX_TEXTURE_IDX)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX),
            vk::DescriptorSetLayoutBinding {
                binding: GRAD_TEXTURE_IDX,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: std::ptr::from_ref(&linearSampler),
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: GAUSSIAN_INTEGRAL_TEXTURE_IDX,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vertexFragment,
                p_immutable_samplers: std::ptr::from_ref(&linearSampler),
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: FEATHER_ATLAS_TEXTURE_IDX,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                p_immutable_samplers: std::ptr::from_ref(&linearSampler),
                ..Default::default()
            },
        ];
        let perFlushInfo = vk::DescriptorSetLayoutCreateInfo::default().bindings(&perFlushBindings);
        let perFlushDescriptorSetLayout = vk_check(unsafe {
            vk.m_ashDevice
                .create_descriptor_set_layout(&perFlushInfo, None)
        });
        let perDrawBindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(IMAGE_TEXTURE_IDX)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let perDrawInfo = vk::DescriptorSetLayoutCreateInfo::default().bindings(&perDrawBindings);
        let perDrawDescriptorSetLayout = vk_check(unsafe {
            vk.m_ashDevice
                .create_descriptor_set_layout(&perDrawInfo, None)
        });
        let emptyInfo = vk::DescriptorSetLayoutCreateInfo::default();
        let emptyDescriptorSetLayout = vk_check(unsafe {
            vk.m_ashDevice
                .create_descriptor_set_layout(&emptyInfo, None)
        });
        let staticPoolSizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
        }];
        let staticPoolInfo = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(2)
            .pool_sizes(&staticPoolSizes);
        let staticDescriptorPool =
            vk_check(unsafe { vk.m_ashDevice.create_descriptor_pool(&staticPoolInfo, None) });
        let nullLayouts = [perDrawDescriptorSetLayout];
        let nullInfo = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(staticDescriptorPool)
            .set_layouts(&nullLayouts);
        let nullImageDescriptorSet =
            vk_check(unsafe { vk.m_ashDevice.allocate_descriptor_sets(&nullInfo) })[0];
        vk.updateImageDescriptorSets(
            nullImageDescriptorSet,
            vk::WriteDescriptorSet::default()
                .dst_binding(IMAGE_TEXTURE_IDX)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER),
            &[vk::DescriptorImageInfo::default()
                .sampler(imageSamplers[ImageSampler::LINEAR_CLAMP_SAMPLER_KEY as usize])
                .image_view(nullTextureView)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)],
        );
        Box::pin(Self {
            m_state: Default::default(),
            m_mode: mode,
            m_jobThread: Default::default(),
            m_newJobCV: Default::default(),
            m_jobCompleteCV: Default::default(),
            m_sharedObjectReadyCV: Default::default(),
            m_vk: vk,
            m_featherAtlasFormat: vk::Format::R16_SFLOAT,
            m_linearSampler: linearSampler,
            m_imageSamplers: imageSamplers,
            m_perFlushDescriptorSetLayout: perFlushDescriptorSetLayout,
            m_perDrawDescriptorSetLayout: perDrawDescriptorSetLayout,
            m_emptyDescriptorSetLayout: emptyDescriptorSetLayout,
            m_staticDescriptorPool: staticDescriptorPool,
            m_nullImageDescriptorSet: nullImageDescriptorSet,
            m_pin: std::marker::PhantomPinned,
        })
    }
}

impl Drop for PipelineManagerVulkan {
    fn drop(&mut self) {
        shutdownBackgroundThread(self);
        unsafe {
            self.m_vk
                .m_ashDevice
                .destroy_descriptor_set_layout(self.m_perFlushDescriptorSetLayout, None);
            self.m_vk
                .m_ashDevice
                .destroy_descriptor_set_layout(self.m_perDrawDescriptorSetLayout, None);
            self.m_vk
                .m_ashDevice
                .destroy_descriptor_set_layout(self.m_emptyDescriptorSetLayout, None);
            self.m_vk
                .m_ashDevice
                .destroy_descriptor_pool(self.m_staticDescriptorPool, None);
            for sampler in self.m_imageSamplers {
                self.m_vk.m_ashDevice.destroy_sampler(sampler, None);
            }
            self.m_vk
                .m_ashDevice
                .destroy_sampler(self.m_linearSampler, None);
        }
    }
}

pub(crate) fn getDrawPipelineLayoutSynchronous(
    manager: &PipelineManagerVulkan,
    interlockMode: InterlockMode,
    mut renderPassOptions: RenderPassOptionsVulkan,
) -> &DrawPipelineLayoutVulkan {
    renderPassOptions = renderPassOptions & RENDER_PASS_OPTIONS_LAYOUT_MASK;
    let key = ((interlockMode as u32) << RENDER_PASS_OPTION_COUNT) | renderPassOptions.0;
    assert_eq!(key >> RENDER_PASS_OPTION_COUNT, interlockMode as u32);
    loop {
        let mut state = manager.m_state.lock().unwrap();
        match state.m_drawPipelineLayouts.get(&key) {
            Some(Some(value)) => {
                let ptr = std::ptr::from_ref(value.as_ref());
                drop(state);
                return unsafe { &*ptr };
            }
            Some(None) => {
                state = manager.m_sharedObjectReadyCV.wait(state).unwrap();
                drop(state);
            }
            None => {
                state.m_drawPipelineLayouts.insert(key, None);
                drop(state);
                let object = Box::new(DrawPipelineLayoutVulkan::new(
                    &manager.m_vk,
                    interlockMode,
                    renderPassOptions,
                    manager.plsBackingType(interlockMode) == PLSBackingType::storageTexture,
                    manager.m_perFlushDescriptorSetLayout,
                    manager.m_perDrawDescriptorSetLayout,
                ));
                let ptr = std::ptr::from_ref(object.as_ref());
                let mut state = manager.m_state.lock().unwrap();
                assert!(state.m_drawPipelineLayouts.get(&key).unwrap().is_none());
                state.m_drawPipelineLayouts.insert(key, Some(object));
                manager.m_sharedObjectReadyCV.notify_all();
                drop(state);
                return unsafe { &*ptr };
            }
        }
    }
}

fn getVertexShaderSynchronous(
    manager: &PipelineManagerVulkan,
    drawType: DrawType,
    mut shaderFeatures: ShaderFeatures,
    interlockMode: InterlockMode,
) -> &DrawShaderVulkan {
    shaderFeatures.0 &= kVertexShaderFeaturesMask.0;
    let key = ShaderUniqueKey(
        drawType,
        shaderFeatures,
        interlockMode,
        ShaderMiscFlags::none,
    );
    loop {
        let mut state = manager.m_state.lock().unwrap();
        match state.m_vertexShaderMap.get(&key) {
            Some(Some(value)) => {
                let ptr = std::ptr::from_ref(value.as_ref());
                drop(state);
                return unsafe { &*ptr };
            }
            Some(None) => {
                state = manager.m_sharedObjectReadyCV.wait(state).unwrap();
                drop(state);
            }
            None => {
                state.m_vertexShaderMap.insert(key, None);
                drop(state);
                let object = Box::new(DrawShaderVulkan::new(
                    DrawShaderVulkanType::vertex,
                    &manager.m_vk,
                    drawType,
                    shaderFeatures,
                    interlockMode,
                    ShaderMiscFlags::none,
                ));
                let ptr = std::ptr::from_ref(object.as_ref());
                let mut state = manager.m_state.lock().unwrap();
                state.m_vertexShaderMap.insert(key, Some(object));
                manager.m_sharedObjectReadyCV.notify_all();
                drop(state);
                return unsafe { &*ptr };
            }
        }
    }
}

fn getFragmentShaderSynchronous(
    manager: &PipelineManagerVulkan,
    drawType: DrawType,
    shaderFeatures: ShaderFeatures,
    interlockMode: InterlockMode,
    miscFlags: ShaderMiscFlags,
) -> &DrawShaderVulkan {
    let key = ShaderUniqueKey(drawType, shaderFeatures, interlockMode, miscFlags);
    loop {
        let mut state = manager.m_state.lock().unwrap();
        match state.m_fragmentShaderMap.get(&key) {
            Some(Some(value)) => {
                let ptr = std::ptr::from_ref(value.as_ref());
                drop(state);
                return unsafe { &*ptr };
            }
            Some(None) => {
                state = manager.m_sharedObjectReadyCV.wait(state).unwrap();
                drop(state);
            }
            None => {
                state.m_fragmentShaderMap.insert(key, None);
                drop(state);
                let object = Box::new(DrawShaderVulkan::new(
                    DrawShaderVulkanType::fragment,
                    &manager.m_vk,
                    drawType,
                    shaderFeatures,
                    interlockMode,
                    miscFlags,
                ));
                let ptr = std::ptr::from_ref(object.as_ref());
                let mut state = manager.m_state.lock().unwrap();
                state.m_fragmentShaderMap.insert(key, Some(object));
                manager.m_sharedObjectReadyCV.notify_all();
                drop(state);
                return unsafe { &*ptr };
            }
        }
    }
}

pub(crate) fn getRenderPassSynchronous(
    manager: &PipelineManagerVulkan,
    interlockMode: InterlockMode,
    renderPassOptions: RenderPassOptionsVulkan,
    renderTargetFormat: vk::Format,
    colorLoadAction: LoadAction,
) -> &RenderPassVulkan {
    let key = RenderPassVulkan::Key(
        interlockMode,
        renderPassOptions,
        renderTargetFormat,
        colorLoadAction,
    );
    loop {
        let mut state = manager.m_state.lock().unwrap();
        match state.m_renderPasses.get(&key) {
            Some(Some(value)) => {
                let ptr = std::ptr::from_ref(value.as_ref());
                drop(state);
                return unsafe { &*ptr };
            }
            Some(None) => {
                state = manager.m_sharedObjectReadyCV.wait(state).unwrap();
                drop(state);
            }
            None => {
                state.m_renderPasses.insert(key, None);
                drop(state);
                let layout =
                    manager.getDrawPipelineLayoutSynchronous(interlockMode, renderPassOptions);
                let object = Box::new(RenderPassVulkan::new(
                    &manager.m_vk,
                    layout,
                    interlockMode,
                    renderPassOptions,
                    renderTargetFormat,
                    colorLoadAction,
                    manager.plsBackingType(interlockMode) == PLSBackingType::storageTexture,
                ));
                let ptr = std::ptr::from_ref(object.as_ref());
                let mut state = manager.m_state.lock().unwrap();
                state.m_renderPasses.insert(key, Some(object));
                manager.m_sharedObjectReadyCV.notify_all();
                drop(state);
                return unsafe { &*ptr };
            }
        }
    }
}

fn createPipeline(
    manager: &PipelineManagerVulkan,
    createType: PipelineCreateType,
    key: u64,
    props: &PipelineProps,
    platformFeatures: &PlatformFeatures,
) -> Option<Box<DrawPipelineVulkan>> {
    if createType == PipelineCreateType::r#async {
        queueBackgroundJob(manager, key, *props, platformFeatures);
        return None;
    }
    let renderPass = manager.getRenderPassSynchronous(
        props.interlockMode,
        props.renderPassOptions,
        props.renderTargetFormat,
        props.colorLoadAction,
    );
    assert_eq!(createType, PipelineCreateType::sync);
    let pipelineLayout = renderPass.drawPipelineLayout().unwrap();
    let vertShader = getVertexShaderSynchronous(
        manager,
        props.drawType,
        props.shaderFeatures,
        props.interlockMode,
    );
    let fragShader = getFragmentShaderSynchronous(
        manager,
        props.drawType,
        props.shaderFeatures,
        props.interlockMode,
        props.shaderMiscFlags,
    );
    Some(Box::new(DrawPipelineVulkan::new(
        &manager.m_vk,
        pipelineLayout,
        props,
        renderPass.into(),
        platformFeatures,
        vertShader,
        fragShader,
        manager.vendorID(),
    )))
}

fn getPipelineStatus(pipeline: &DrawPipelineVulkan) -> PipelineStatus {
    if pipeline.m_vkPipeline == vk::Pipeline::null() {
        PipelineStatus::errored
    } else {
        PipelineStatus::ready
    }
}

fn processCompletedJobs(manager: &PipelineManagerVulkan, targetKey: Option<u64>) -> bool {
    loop {
        let completed = {
            let mut state = manager.m_state.lock().unwrap();
            state.m_completedJobs.pop()
        };
        let Some(completed) = completed else {
            return targetKey.is_none();
        };
        assert_ne!(
            getPipelineStatus(&completed.program),
            PipelineStatus::notReady
        );
        let key = completed.key;
        manager
            .m_state
            .lock()
            .unwrap()
            .m_pipelines
            .insert(key, Some(completed.program));
        if targetKey == Some(key) {
            return true;
        }
    }
}

fn queueBackgroundJob(
    manager: &PipelineManagerVulkan,
    key: u64,
    props: PipelineProps,
    platformFeatures: &PlatformFeatures,
) {
    {
        let mut thread = manager.m_jobThread.lock().unwrap();
        if thread.is_none() {
            // PipelineManagerVulkan is permanently pinned before this thread
            // can start, and Drop joins the thread before freeing the owner.
            let address = manager as *const PipelineManagerVulkan as usize;
            *thread = Some(std::thread::spawn(move || unsafe {
                backgroundShaderCompilationThread(&*(address as *const PipelineManagerVulkan));
            }));
        }
    }
    let mut state = manager.m_state.lock().unwrap();
    state.m_jobQueue.push_back(JobParams {
        props,
        key,
        platformFeatures: *platformFeatures,
    });
    manager.m_newJobCV.notify_one();
}

fn backgroundShaderCompilationThread(manager: &PipelineManagerVulkan) {
    loop {
        let nextJob = {
            let mut state = manager.m_state.lock().unwrap();
            while !state.m_isDone && state.m_jobQueue.is_empty() {
                state = manager.m_newJobCV.wait(state).unwrap();
            }
            if state.m_isDone {
                return;
            }
            let next = state.m_jobQueue.pop_front().unwrap();
            state.m_currentThreadPipelineKey = Some(next.key);
            state.m_activePipelineCreationCount += 1;
            next
        };
        let newPipeline = createPipeline(
            manager,
            PipelineCreateType::sync,
            nextJob.key,
            &nextJob.props,
            &nextJob.platformFeatures,
        )
        .unwrap();
        let mut state = manager.m_state.lock().unwrap();
        state.m_completedJobs.push(CompletedJob {
            key: nextJob.key,
            program: newPipeline,
        });
        state.m_currentThreadPipelineKey = None;
        state.m_activePipelineCreationCount -= 1;
        manager.m_jobCompleteCV.notify_all();
    }
}

fn shutdownBackgroundThread(manager: &PipelineManagerVulkan) {
    let handle = {
        let mut thread = manager.m_jobThread.lock().unwrap();
        if thread.is_none() {
            return;
        }
        manager.m_state.lock().unwrap().m_isDone = true;
        manager.m_newJobCV.notify_all();
        thread.take().unwrap()
    };
    handle.join().unwrap();
}

fn waitForPipelineForRender(manager: &PipelineManagerVulkan, key: u64) {
    {
        let state = manager.m_state.lock().unwrap();
        assert!(state.m_pipelines.contains_key(&key));
        if state.m_pipelines.get(&key).unwrap().is_some() {
            return;
        }
    }
    if processCompletedJobs(manager, Some(key)) {
        return;
    }
    {
        let mut state = manager.m_state.lock().unwrap();
        if let Some(index) = state.m_jobQueue.iter().position(|job| job.key == key) {
            let params = state.m_jobQueue.remove(index).unwrap();
            state.m_jobQueue.push_front(params);
        }
    }
    loop {
        {
            let mut state = manager.m_state.lock().unwrap();
            if state.m_completedJobs.is_empty() {
                state = manager.m_jobCompleteCV.wait(state).unwrap();
                drop(state);
            }
        }
        if processCompletedJobs(manager, Some(key)) {
            return;
        }
    }
}

pub(crate) fn tryGetPipeline<'a>(
    manager: &'a PipelineManagerVulkan,
    propsIn: &PipelineProps,
    platformFeatures: &PlatformFeatures,
) -> Option<&'a DrawPipelineVulkan> {
    let mut props = *propsIn;
    let ubershaderFeatures = UbershaderFeaturesMaskFor(
        props.shaderFeatures,
        props.drawType,
        props.interlockMode,
        props.shaderMiscFlags,
        platformFeatures,
    );
    let createType = match manager.m_mode {
        ShaderCompilationMode::allowAsynchronous => {
            if props.shaderFeatures == ubershaderFeatures {
                PipelineCreateType::sync
            } else {
                PipelineCreateType::r#async
            }
        }
        ShaderCompilationMode::onlyUbershaders => {
            props.shaderFeatures = ubershaderFeatures;
            PipelineCreateType::sync
        }
        ShaderCompilationMode::alwaysSynchronous => PipelineCreateType::sync,
    };
    let key = props.createKey(platformFeatures);

    #[cfg(feature = "with-rive-tools")]
    {
        use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType;
        if props.synthesizedFailureType == SynthesizedFailureType::ubershaderLoad
            && props.drawType != DrawType::renderPassResolve
        {
            return None;
        }
        if props.shaderFeatures == ubershaderFeatures {
            props.synthesizedFailureType = SynthesizedFailureType::none;
            if !manager
                .m_state
                .lock()
                .unwrap()
                .m_pipelines
                .contains_key(&key)
            {
                debug_assert!(isValidUbershaderPipelineProps(
                    manager,
                    &props,
                    platformFeatures
                ));
            }
        }
    }

    let missing = !manager
        .m_state
        .lock()
        .unwrap()
        .m_pipelines
        .contains_key(&key);
    if missing {
        let pipeline = createPipeline(manager, createType, key, &props, platformFeatures);
        manager
            .m_state
            .lock()
            .unwrap()
            .m_pipelines
            .insert(key, pipeline);
        if createType == PipelineCreateType::sync {
            let status = {
                let state = manager.m_state.lock().unwrap();
                getPipelineStatus(state.m_pipelines[&key].as_ref().unwrap())
            };
            if status != PipelineStatus::errored {
                return pipeline_ref(manager, key);
            }
            if props.shaderFeatures == ubershaderFeatures {
                #[cfg(not(feature = "with-rive-tools"))]
                debug_assert!(false, "Ubershader creation failed");
                return None;
            }
            assert_ne!(props.shaderFeatures, ubershaderFeatures);
        }
    }
    let is_none = manager.m_state.lock().unwrap().m_pipelines[&key].is_none();
    if is_none {
        if processCompletedJobs(manager, Some(key)) {
            debug_assert!(pipeline_ref(manager, key).is_some());
        } else if props.shaderFeatures == ubershaderFeatures {
            waitForPipelineForRender(manager, key);
        }
    }
    if let Some(pipeline) = pipeline_ref(manager, key) {
        match getPipelineStatus(pipeline) {
            PipelineStatus::ready => return Some(pipeline),
            PipelineStatus::notReady => {}
            PipelineStatus::errored => {}
        }
    }
    if props.shaderFeatures == ubershaderFeatures {
        debug_assert_eq!(
            pipeline_ref(manager, key).map(getPipelineStatus),
            Some(PipelineStatus::errored)
        );
        return None;
    }
    let mut ubershaderProps = props;
    ubershaderProps.shaderFeatures = ubershaderFeatures;
    tryGetPipeline(manager, &ubershaderProps, platformFeatures)
}

fn pipeline_ref(manager: &PipelineManagerVulkan, key: u64) -> Option<&DrawPipelineVulkan> {
    let state = manager.m_state.lock().unwrap();
    let ptr = state
        .m_pipelines
        .get(&key)
        .and_then(Option::as_ref)
        .map(|pipeline| std::ptr::from_ref(pipeline.as_ref()));
    drop(state);
    ptr.map(|ptr| unsafe { &*ptr })
}

fn queuePipelineIfNotFound(
    manager: &PipelineManagerVulkan,
    props: &PipelineProps,
    platformFeatures: &PlatformFeatures,
) -> bool {
    let key = props.createKey(platformFeatures);
    if manager
        .m_state
        .lock()
        .unwrap()
        .m_pipelines
        .contains_key(&key)
    {
        return false;
    }
    let pipeline = createPipeline(
        manager,
        PipelineCreateType::r#async,
        key,
        props,
        platformFeatures,
    );
    manager
        .m_state
        .lock()
        .unwrap()
        .m_pipelines
        .insert(key, pipeline);
    true
}

pub(crate) fn clearCache(manager: Pin<&mut PipelineManagerVulkan>) {
    let manager = manager.as_ref().get_ref();
    let mut state = manager.m_state.lock().unwrap();
    state.m_jobQueue.clear();
    while state.m_activePipelineCreationCount > 0 {
        state = manager.m_jobCompleteCV.wait(state).unwrap();
    }
    state.m_completedJobs.clear();
    state.m_vertexShaderMap.clear();
    state.m_fragmentShaderMap.clear();
    state.m_pipelines.clear();
    state.m_drawPipelineLayouts.clear();
    state.m_renderPasses.clear();
}

pub(crate) fn waitForAllBackgroundPipelineCreation(manager: &PipelineManagerVulkan) {
    {
        let mut state = manager.m_state.lock().unwrap();
        while state.m_currentThreadPipelineKey.is_some() || !state.m_jobQueue.is_empty() {
            state = manager.m_jobCompleteCV.wait(state).unwrap();
        }
    }
    assert!(processCompletedJobs(manager, None));
}

fn bit_combinations_descending(mask: u32) -> impl Iterator<Item = u32> {
    let mut current = Some(mask);
    std::iter::from_fn(move || {
        let value = current?;
        current = if value == 0 {
            None
        } else {
            Some(value.wrapping_sub(1) & mask)
        };
        Some(value)
    })
}

fn get_relevant_blend_modes_for_pipeline_creation(
    interlockMode: InterlockMode,
    _drawType: DrawType,
    _miscFlags: ShaderMiscFlags,
    drawContents: DrawContents,
    platformFeatures: &PlatformFeatures,
) -> &'static [BlendMode] {
    const SRC_OVER_ONLY: &[BlendMode] = &[BlendMode::SrcOver];
    match interlockMode {
        InterlockMode::rasterOrdering
        | InterlockMode::atomics
        | InterlockMode::clockwise
        | InterlockMode::clockwiseAtomic => SRC_OVER_ONLY,
        InterlockMode::msaa => {
            assert!(
                drawContents.0 & DrawContents::opaquePaint.0 != 0
                    || !platformFeatures.supportsBlendAdvancedKHR
            );
            SRC_OVER_ONLY
        }
    }
}

fn forEachUbershaderPermutation(
    manager: &PipelineManagerVulkan,
    interlockMode: InterlockMode,
    renderTargetFormat: vk::Format,
    renderTargetUsage: vk::ImageUsageFlags,
    colorLoadAction: LoadAction,
    platformFeatures: &PlatformFeatures,
    mut func: impl FnMut(&PipelineProps) -> bool,
) {
    let _ = manager;
    ForEachUbershaderPermutation(
        interlockMode,
        platformFeatures,
        |drawType, shaderFeatures, shaderMiscFlags| {
            let mut props = PipelineProps {
                drawType,
                shaderFeatures,
                interlockMode,
                shaderMiscFlags,
                drawContents: DrawContents::none,
                blendMode: BlendMode::SrcOver,
                drawPipelineOptions: DrawPipelineOptions::none,
                renderPassOptions: RenderPassOptionsVulkan::none,
                renderTargetFormat,
                colorLoadAction,
                #[cfg(feature = "with-rive-tools")]
                synthesizedFailureType: crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType::none,
            };
            let validDrawContents = if interlockMode == InterlockMode::msaa {
                DRAW_CONTENTS_FOR_MSAA_PIPELINE_STATE.0
            } else {
                DrawContents::none.0
            };
            let mut fixedPassOptions = RenderPassOptionsVulkan::none;
            if interlockMode != InterlockMode::clockwiseAtomic
                && interlockMode != InterlockMode::msaa
                && shaderMiscFlags.has(ShaderMiscFlags::fixedFunctionColorOutput)
            {
                fixedPassOptions |= RenderPassOptionsVulkan::fixedFunctionColorOutput;
            }
            let mut validPassOptions = RenderPassOptionsVulkan::none;
            match interlockMode {
                InterlockMode::rasterOrdering => {
                    validPassOptions |= RenderPassOptionsVulkan::manuallyResolved
                        | RenderPassOptionsVulkan::rasterOrderingInterruptible
                        | RenderPassOptionsVulkan::rasterOrderingResume;
                }
                InterlockMode::atomics => {
                    if !renderTargetUsage.contains(vk::ImageUsageFlags::INPUT_ATTACHMENT)
                        && !shaderMiscFlags.has(ShaderMiscFlags::fixedFunctionColorOutput)
                    {
                        fixedPassOptions |=
                            RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer;
                    }
                }
                InterlockMode::clockwise => {}
                InterlockMode::clockwiseAtomic => {
                    if shaderMiscFlags.has(ShaderMiscFlags::fixedFunctionColorOutput) {
                        validPassOptions |= RenderPassOptionsVulkan::fixedFunctionColorOutput;
                    }
                }
                InterlockMode::msaa => {
                    validPassOptions |= RenderPassOptionsVulkan::manuallyResolved
                        | RenderPassOptionsVulkan::msaaSeedFromOffscreenTexture;
                    if shaderMiscFlags.has(ShaderMiscFlags::fixedFunctionColorOutput) {
                        validPassOptions |= RenderPassOptionsVulkan::fixedFunctionColorOutput;
                    }
                }
            }
            for drawContents in bit_combinations_descending(validDrawContents) {
                let drawContents = DrawContents(drawContents);
                if !get_stencil_info(interlockMode, drawType, drawContents).areDrawContentsValid {
                    continue;
                }
                props.drawContents = drawContents;
                for variable in bit_combinations_descending(validPassOptions.0) {
                    props.renderPassOptions = RenderPassOptionsVulkan(variable) | fixedPassOptions;
                    if props
                        .renderPassOptions
                        .has(RenderPassOptionsVulkan::manuallyResolved)
                        && (props
                            .renderPassOptions
                            .has(RenderPassOptionsVulkan::fixedFunctionColorOutput)
                            || props
                                .renderPassOptions
                                .has(RenderPassOptionsVulkan::rasterOrderingInterruptible))
                    {
                        continue;
                    }
                    if props
                        .renderPassOptions
                        .has(RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer)
                    {
                        if props
                            .renderPassOptions
                            .has(RenderPassOptionsVulkan::fixedFunctionColorOutput)
                        {
                            continue;
                        }
                        if drawType == DrawType::renderPassResolve
                            && !props
                                .shaderMiscFlags
                                .has(ShaderMiscFlags::coalescedResolveAndTransfer)
                        {
                            continue;
                        }
                    }
                    for &blendMode in get_relevant_blend_modes_for_pipeline_creation(
                        interlockMode,
                        drawType,
                        shaderMiscFlags,
                        props.drawContents,
                        platformFeatures,
                    ) {
                        props.blendMode = blendMode;
                        if !func(&props) {
                            return false;
                        }
                    }
                }
            }
            true
        },
    );
}

fn isValidUbershaderPipelineProps(
    manager: &PipelineManagerVulkan,
    props: &PipelineProps,
    platformFeatures: &PlatformFeatures,
) -> bool {
    let mut found = false;
    let currentKey = props.createKey(platformFeatures);
    forEachUbershaderPermutation(
        manager,
        props.interlockMode,
        props.renderTargetFormat,
        if props
            .renderPassOptions
            .has(RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer)
        {
            vk::ImageUsageFlags::empty()
        } else {
            vk::ImageUsageFlags::INPUT_ATTACHMENT
        },
        props.colorLoadAction,
        platformFeatures,
        |validProps| {
            if validProps.createKey(platformFeatures) == currentKey {
                found = true;
            }
            !found
        },
    );
    found
}

pub(crate) fn queueUbershaderPipelineCreation(
    manager: &PipelineManagerVulkan,
    interlockMode: InterlockMode,
    renderTargetFormat: vk::Format,
    renderTargetUsage: vk::ImageUsageFlags,
    colorLoadAction: LoadAction,
    platformFeatures: &PlatformFeatures,
) {
    forEachUbershaderPermutation(
        manager,
        interlockMode,
        renderTargetFormat,
        renderTargetUsage,
        colorLoadAction,
        platformFeatures,
        |props| {
            queuePipelineIfNotFound(manager, props, platformFeatures);
            true
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_sampler_option_mappings_are_preserved() {
        assert_eq!(vk_filter(ImageFilter::bilinear), vk::Filter::LINEAR);
        assert_eq!(vk_filter(ImageFilter::nearest), vk::Filter::NEAREST);
        assert_eq!(
            vk_sampler_address_mode(ImageWrap::mirror),
            vk::SamplerAddressMode::MIRRORED_REPEAT
        );
        assert_eq!(
            vk_sampler_mipmap_mode(ImageFilter::bilinear),
            vk::SamplerMipmapMode::NEAREST
        );
    }

    #[test]
    fn source_bit_combination_iteration_is_descending_and_includes_zero() {
        assert_eq!(
            bit_combinations_descending(0b1011).collect::<Vec<_>>(),
            vec![0b1011, 0b1010, 0b1001, 0b1000, 0b0011, 0b0010, 0b0001, 0]
        );
    }
}
