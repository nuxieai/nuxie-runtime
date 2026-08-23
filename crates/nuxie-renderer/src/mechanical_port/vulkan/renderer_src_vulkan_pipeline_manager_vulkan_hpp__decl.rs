//! Complete mechanical declaration translation of
//! `renderer/src/vulkan/pipeline_manager_vulkan.hpp`.

#![allow(non_snake_case)]

use super::draw_pipeline_layout_vulkan_decl::DrawPipelineLayoutVulkan;
use super::draw_pipeline_vulkan_decl::{DrawPipelineVulkan, PipelineProps};
use super::draw_shader_vulkan_decl::DrawShaderVulkan;
use super::render_pass_vulkan_decl::{RenderPassOptionsVulkan, RenderPassVulkan};
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    InterlockMode, LoadAction, PlatformFeatures,
};
use ash::vk;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

pub(crate) const MAX_SAMPLER_PERMUTATIONS: usize = 18;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShaderCompilationMode {
    allowAsynchronous = 0,
    alwaysSynchronous = 1,
    onlyUbershaders = 2,
}

impl ShaderCompilationMode {
    pub(crate) const standard: Self = Self::allowAsynchronous;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineCreateType {
    sync,
    r#async,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineStatus {
    notReady,
    ready,
    errored,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PLSBackingType {
    inputAttachment,
    storageTexture,
}

pub(super) struct JobParams {
    pub(super) props: PipelineProps,
    pub(super) key: u64,
    pub(super) platformFeatures: *const PlatformFeatures,
}

unsafe impl Send for JobParams {}

pub(super) struct CompletedJob {
    pub(super) key: u64,
    pub(super) program: Box<DrawPipelineVulkan>,
}

pub(super) struct PipelineManagerState {
    pub(super) m_vertexShaderMap: HashMap<u32, Option<Box<DrawShaderVulkan>>>,
    pub(super) m_fragmentShaderMap: HashMap<u32, Option<Box<DrawShaderVulkan>>>,
    pub(super) m_pipelines: HashMap<u64, Option<Box<DrawPipelineVulkan>>>,
    pub(super) m_jobQueue: VecDeque<JobParams>,
    pub(super) m_completedJobs: Vec<CompletedJob>,
    pub(super) m_isDone: bool,
    pub(super) m_activePipelineCreationCount: u32,
    pub(super) m_currentThreadPipelineKey: Option<u64>,
    pub(super) m_renderPasses: HashMap<u32, Option<Box<RenderPassVulkan>>>,
    pub(super) m_drawPipelineLayouts: HashMap<u32, Option<Box<DrawPipelineLayoutVulkan>>>,
}

impl Default for PipelineManagerState {
    fn default() -> Self {
        Self {
            m_vertexShaderMap: HashMap::new(),
            m_fragmentShaderMap: HashMap::new(),
            m_pipelines: HashMap::new(),
            m_jobQueue: VecDeque::new(),
            m_completedJobs: Vec::new(),
            m_isDone: false,
            m_activePipelineCreationCount: 0,
            m_currentThreadPipelineKey: None,
            m_renderPasses: HashMap::new(),
            m_drawPipelineLayouts: HashMap::new(),
        }
    }
}

pub(crate) struct PipelineManagerVulkan {
    // AsyncPipelineManager<DrawPipelineVulkan> fields.
    pub(super) m_state: Mutex<PipelineManagerState>,
    pub(super) m_mode: ShaderCompilationMode,
    pub(super) m_jobThread: Mutex<Option<JoinHandle<()>>>,
    pub(super) m_newJobCV: Condvar,
    pub(super) m_jobCompleteCV: Condvar,
    pub(super) m_sharedObjectReadyCV: Condvar,

    // PipelineManagerVulkan fields, in source order.
    pub(crate) m_vk: Arc<VulkanContext>,
    pub(crate) m_featherAtlasFormat: vk::Format,
    pub(crate) m_linearSampler: vk::Sampler,
    pub(crate) m_imageSamplers: [vk::Sampler; MAX_SAMPLER_PERMUTATIONS],
    pub(crate) m_perFlushDescriptorSetLayout: vk::DescriptorSetLayout,
    pub(crate) m_perDrawDescriptorSetLayout: vk::DescriptorSetLayout,
    pub(crate) m_emptyDescriptorSetLayout: vk::DescriptorSetLayout,
    pub(crate) m_staticDescriptorPool: vk::DescriptorPool,
    pub(crate) m_nullImageDescriptorSet: vk::DescriptorSet,
}

// The source permits the manager and its Vulkan caches on its one owned
// compilation thread. Every mutable map/queue is under m_state, the raw
// layout pointers target stable Boxes, and Drop joins that thread first.
unsafe impl Send for PipelineManagerVulkan {}
unsafe impl Sync for PipelineManagerVulkan {}

impl PipelineManagerVulkan {
    pub(crate) fn vendorID(&self) -> u32 {
        self.m_vk.physicalDeviceProperties().vendor_id
    }
    pub(crate) fn featherAtlasFormat(&self) -> vk::Format {
        self.m_featherAtlasFormat
    }
    pub(crate) fn vulkanContext(&self) -> &VulkanContext {
        &self.m_vk
    }
    pub(crate) fn perFlushDescriptorSetLayout(&self) -> vk::DescriptorSetLayout {
        self.m_perFlushDescriptorSetLayout
    }
    pub(crate) fn perDrawDescriptorSetLayout(&self) -> vk::DescriptorSetLayout {
        self.m_perDrawDescriptorSetLayout
    }
    pub(crate) fn emptyDescriptorSetLayout(&self) -> vk::DescriptorSetLayout {
        self.m_emptyDescriptorSetLayout
    }
    pub(crate) fn linearSampler(&self) -> vk::Sampler {
        self.m_linearSampler
    }
    pub(crate) fn imageSampler(&self, i: u32) -> vk::Sampler {
        self.m_imageSamplers[i as usize]
    }
    pub(crate) fn nullImageDescriptorSet(&self) -> vk::DescriptorSet {
        self.m_nullImageDescriptorSet
    }
    pub(crate) fn plsBackingType(&self, interlockMode: InterlockMode) -> PLSBackingType {
        if interlockMode == InterlockMode::clockwise {
            assert!(self.m_vk.features.fragmentShaderPixelInterlock);
            PLSBackingType::storageTexture
        } else {
            PLSBackingType::inputAttachment
        }
    }
    pub(crate) fn getDrawPipelineLayoutSynchronous(
        &self,
        interlockMode: InterlockMode,
        renderPassOptions: RenderPassOptionsVulkan,
    ) -> &DrawPipelineLayoutVulkan {
        super::pipeline_manager_vulkan_impl::getDrawPipelineLayoutSynchronous(
            self,
            interlockMode,
            renderPassOptions,
        )
    }
    pub(crate) fn getRenderPassSynchronous(
        &self,
        interlockMode: InterlockMode,
        renderPassOptions: RenderPassOptionsVulkan,
        renderTargetFormat: vk::Format,
        colorLoadAction: LoadAction,
    ) -> &RenderPassVulkan {
        super::pipeline_manager_vulkan_impl::getRenderPassSynchronous(
            self,
            interlockMode,
            renderPassOptions,
            renderTargetFormat,
            colorLoadAction,
        )
    }
    pub(crate) fn tryGetPipeline(
        &self,
        props: &PipelineProps,
        platformFeatures: &PlatformFeatures,
    ) -> Option<&DrawPipelineVulkan> {
        super::pipeline_manager_vulkan_impl::tryGetPipeline(self, props, platformFeatures)
    }
    pub(crate) fn clearCache(&self) {
        super::pipeline_manager_vulkan_impl::clearCache(self)
    }
    pub(crate) fn waitForAllBackgroundPipelineCreation(&self) {
        super::pipeline_manager_vulkan_impl::waitForAllBackgroundPipelineCreation(self)
    }
    pub(crate) fn queueUbershaderPipelineCreation(
        &self,
        interlockMode: InterlockMode,
        renderTargetFormat: vk::Format,
        renderTargetUsage: vk::ImageUsageFlags,
        colorLoadAction: LoadAction,
        platformFeatures: &PlatformFeatures,
    ) {
        super::pipeline_manager_vulkan_impl::queueUbershaderPipelineCreation(
            self,
            interlockMode,
            renderTargetFormat,
            renderTargetUsage,
            colorLoadAction,
            platformFeatures,
        )
    }
}
