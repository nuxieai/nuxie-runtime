// fdd555d6 texel_buffer_ring_gl_test.cpp, translated at the existing native
// recording-provider boundary. This exercises the real GL context flush and
// storage rings, but is not ANGLE/Mali execution or a rasterized-image test.

#[derive(Default)]
struct RingTrace {
    lifecycle: Option<GLContextLifecycleIngress>,
    next_name: GLuint,
    active_unit: GLenum,
    textures: std::collections::BTreeMap<GLenum, GLuint>,
    fence_pending: bool,
    fail_insert: bool,
    fence_calls: usize,
    queries: Vec<(GLuint, GLenum)>,
    events: Vec<String>,
}

struct RingTestProvider {
    canvas: CanvasTestProvider,
    trace: Rc<RefCell<RingTrace>>,
}

impl GLExecutionProvider for RingTestProvider {
    fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress) {
        self.trace.borrow_mut().lifecycle = Some(ingress.clone());
        self.canvas.installContextLifecycleIngress(ingress);
    }
    fn installFinalReleaseIngress(
        &mut self,
        ingress: GLFinalReleaseIngress,
    ) -> std::sync::Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
        self.canvas.installFinalReleaseIngress(ingress)
    }
    fn submit(&mut self, command: GLCommand) {
        {
            let mut trace = self.trace.borrow_mut();
            match &command {
                GLCommand::ActiveTexture(unit) => trace.active_unit = *unit,
                GLCommand::BindTexture(GL_TEXTURE_2D, texture) => {
                    let unit = trace.active_unit;
                    trace.textures.insert(unit, *texture);
                    trace.events.push(format!("bind:{texture}"));
                }
                GLCommand::DeleteTexture(texture) => trace.events.push(format!("delete:{texture}")),
                GLCommand::TexStorage2D { .. } => trace.events.push("storage".into()),
                GLCommand::TexSubImage2D { .. } => trace.events.push("upload".into()),
                GLCommand::Flush => trace.events.push("flush".into()),
                _ => {}
            }
        }
        self.canvas.submit(command);
    }
    fn generateObject(&mut self, kind: GLObjectKind) -> GLuint {
        let mut trace = self.trace.borrow_mut();
        trace.next_name += 1;
        let name = trace.next_name;
        if kind == GLObjectKind::Texture {
            trace.events.push(format!("generate:{name}"));
        }
        name
    }
    fn createProgram(&mut self) -> GLuint {
        self.generateObject(GLObjectKind::Program)
    }
    fn createShader(&mut self, kind: GLenum) -> GLuint {
        self.canvas.createShader(kind)
    }
    fn fenceSync(&mut self, condition: GLenum, flags: GLbitfield) -> GLsync {
        assert_eq!(condition, GL_SYNC_GPU_COMMANDS_COMPLETE);
        assert_eq!(flags, 0);
        let mut trace = self.trace.borrow_mut();
        trace.fence_calls += 1;
        trace.events.push("fence".into());
        if trace.fail_insert {
            0
        } else {
            trace.next_name += 1;
            trace.next_name
        }
    }
    fn clientWaitSync(&mut self, _sync: GLsync, flags: GLbitfield, timeout: u64) -> GLenum {
        assert_eq!(flags, 0);
        assert_eq!(timeout, 0);
        if self.trace.borrow().fence_pending {
            GL_TIMEOUT_EXPIRED
        } else {
            GL_ALREADY_SIGNALED
        }
    }
    fn deleteSync(&mut self, sync: GLsync) {
        self.trace
            .borrow_mut()
            .events
            .push(format!("delete_sync:{sync}"));
    }
    fn getInteger(&mut self, parameter: GLenum) -> GLint {
        self.canvas.getInteger(parameter)
    }
    fn getFloat(&mut self, parameter: GLenum) -> GLfloat {
        self.canvas.getFloat(parameter)
    }
    fn getString(&mut self, parameter: GLenum) -> Option<Vec<u8>> {
        self.canvas.getString(parameter)
    }
    fn getExtension(&mut self, index: GLuint) -> Option<Vec<u8>> {
        self.canvas.getExtension(index)
    }
    fn enableWebGLExtension(&mut self, name: &str) -> bool {
        self.canvas.enableWebGLExtension(name)
    }
    fn isObject(&mut self, kind: GLObjectKind, name: GLuint) -> bool {
        self.canvas.isObject(kind, name)
    }
    fn checkFramebufferStatus(&mut self, target: GLenum) -> GLenum {
        self.canvas.checkFramebufferStatus(target)
    }
    fn shaderParameter(&mut self, _shader: GLuint, _parameter: GLenum) -> GLint {
        GL_TRUE as GLint
    }
    fn shaderInfoLog(&mut self, shader: GLuint, length: usize) -> Vec<u8> {
        self.canvas.shaderInfoLog(shader, length)
    }
    fn programParameter(&mut self, program: GLuint, parameter: GLenum) -> GLint {
        self.trace.borrow_mut().queries.push((program, parameter));
        if parameter == GL_COMPLETION_STATUS_KHR {
            0
        } else {
            GL_TRUE as GLint
        }
    }
    fn programInfoLog(&mut self, program: GLuint, length: usize) -> Vec<u8> {
        self.canvas.programInfoLog(program, length)
    }
    fn uniformBlockIndex(&mut self, program: GLuint, name: &[u8]) -> GLuint {
        self.canvas.uniformBlockIndex(program, name)
    }
    fn uniformLocation(&mut self, program: GLuint, name: &[u8]) -> GLint {
        self.canvas.uniformLocation(program, name)
    }
    fn readPixelsRGBA8(&mut self, x: i32, y: i32, width: u32, height: u32) -> Vec<u8> {
        self.canvas.readPixelsRGBA8(x, y, width, height)
    }
    fn contextLost(&mut self, next: u64) {
        self.canvas.contextLost(next);
    }
}

fn ring_test_context(
    rotates: bool,
    ssbo: bool,
) -> (
    GLExecutionDomain,
    Box<RenderContextGLImpl>,
    Rc<RefCell<RingTrace>>,
) {
    let trace = Rc::new(RefCell::new(RingTrace::default()));
    let domain = GLExecutionDomain::new(Box::new(RingTestProvider {
        canvas: CanvasTestProvider {
            commands: Rc::new(RefCell::new(Vec::new())),
            lifecycleIngress: None,
            finalReleaseIngress: Rc::new(RefCell::new(None)),
            finalReleaseWake: std::sync::Arc::new(TestFinalReleaseWake::default()),
        },
        trace: trace.clone(),
    }));
    let mut caps = GLCapabilities::default();
    caps.avoidTexSubImage2DOnInFlightTextures = rotates;
    caps.setARB_shader_storage_buffer_object(ssbo);
    caps.setKHR_parallel_shader_compile(true);
    caps.maxSupportedInstancesPerFlush = u32::MAX;
    let context = domain.withCurrent(|| newComponent097TestContextOwner(caps, domain.clone()));
    (domain, context, trace)
}

fn prepare_ring_buffers(context: &mut RenderContextGLImpl) {
    *context.base.m_flushUniformBuffer = makeUniformBufferRing(context, 256);
    *context.base.m_pathBuffer =
        makeStorageBufferRing(context, 65536, gpu::StorageBufferStructure::uint32x4);
    *context.base.m_paintAuxBuffer =
        makeStorageBufferRing(context, 65536, gpu::StorageBufferStructure::float32x4);
    *context.base.m_contourBuffer =
        makeStorageBufferRing(context, 65536, gpu::StorageBufferStructure::uint32x4);
}

fn map_ring_frame(context: &mut RenderContextGLImpl) {
    let base: &mut RenderContextHelperImpl = &mut context.base;
    for ring in [
        &mut *base.m_pathBuffer,
        &mut *base.m_paintAuxBuffer,
        &mut *base.m_contourBuffer,
    ] {
        let ring = ring.as_mut().unwrap();
        let bytes = ring.mapBuffer(65536);
        assert!(!bytes.is_null());
        // The recording provider never executes shader reads. Zeroed storage
        // supplies the same record counts without claiming drawn rectangles.
        unsafe { std::ptr::write_bytes(bytes.cast::<u8>(), 0, 65536) };
        ring.unmapAndSubmitBuffer();
    }
}

fn ring_flush(
    context: &mut RenderContextGLImpl,
    target: std::ptr::NonNull<RenderTarget>,
    path_count: u32,
) {
    let mut list = gpu::BlockAllocatedLinkedList::default();
    let mut desc = dynamic_msaa_descriptor(target, &mut list);
    desc.pathCount = path_count;
    desc.contourCount = path_count;
    // SAFETY: target, list and all mapped storage owners outlive this call.
    unsafe { flush(context, &desc) };
}

fn bound_ring_texture(trace: &Rc<RefCell<RingTrace>>, binding: GLuint) -> GLuint {
    trace.borrow().textures[&(GL_TEXTURE0 + binding)]
}

#[test]
fn texel_ring_constructor_generates_all_names_then_clears_them_in_forward_order() {
    let (domain, context, trace) = ring_test_context(true, false);
    trace.borrow_mut().events.clear();
    let first_name = trace.borrow().next_name + 1;
    let ring = makeStorageBufferRing(&context, 65536, gpu::StorageBufferStructure::uint32x4);
    let mut expected = Vec::new();
    for index in 0..gpu::kBufferRingSize as GLuint {
        expected.push(format!("generate:{}", first_name + index));
    }
    for index in 0..gpu::kBufferRingSize as GLuint {
        expected.push(format!("delete:{}", first_name + index));
    }
    // Exact order also excludes premature storage allocation or binding.
    assert_eq!(trace.borrow().events, expected);
    drop(ring);
    assert_eq!(trace.borrow().events, expected);
    drop(context);
    domain.shutdown();
}

#[test]
fn populated_texel_ring_drop_routes_reverse_deletion_to_its_owner() {
    // Exercise no current context, a foreign current context, and a stale
    // generation while a replacement/foreign context is current.
    for case in 0..3 {
        let (owner, context, owner_trace) = ring_test_context(true, false);
        let (foreign, foreign_context, foreign_trace) = ring_test_context(false, false);
        let mut ring = TexelBufferRingWebGL::new(
            65536,
            gpu::StorageBufferStructure::uint32x4,
            (&*context.m_state).clone(),
            (&*context.rust_execution).clone(),
        );
        let bytes = ring.mapBuffer(65536);
        assert!(!bytes.is_null());
        // SAFETY: mapBuffer exposes the ring's complete writable allocation.
        unsafe { std::ptr::write_bytes(bytes.cast::<u8>(), 0, 65536) };
        ring.unmapAndSubmitBuffer();
        let mut textures = Vec::new();
        for slot in 0..gpu::kBufferRingSize as usize {
            ring.bindToRenderContext(PATH_BUFFER_IDX, 256, 0, slot, false);
            textures.push(bound_ring_texture(&owner_trace, PATH_BUFFER_IDX));
        }
        assert!(textures.iter().all(|texture| *texture != 0));
        assert!(textures.windows(2).all(|pair| pair[0] != pair[1]));
        if case == 2 {
            let ingress = owner_trace.borrow().lifecycle.clone().unwrap();
            assert!(ingress.contextLost());
        }
        owner_trace.borrow_mut().events.clear();
        foreign_trace.borrow_mut().events.clear();
        if case == 0 {
            assert_eq!(currentGLExecutionIdentity(), None);
            drop(ring);
            assert_eq!(currentGLExecutionIdentity(), None);
        } else {
            foreign.withCurrent(|| {
                let identity = currentGLExecutionIdentity();
                drop(ring);
                assert_eq!(currentGLExecutionIdentity(), identity);
            });
        }
        let expected: Vec<_> = if case == 2 {
            Vec::new()
        } else {
            textures
                .iter()
                .rev()
                .map(|texture| format!("delete:{texture}"))
                .collect()
        };
        assert_eq!(owner_trace.borrow().events, expected);
        assert!(foreign_trace.borrow().events.is_empty());
        drop(context);
        drop(foreign_context);
        owner.shutdown();
        foreign.shutdown();
    }
}

#[test]
fn texel_buffer_ring_rotates_source_case_recording_provider() {
    for rotates in [false, true] {
        let (domain, mut context, trace) = ring_test_context(rotates, false);
        prepare_ring_buffers(&mut context);
        let mut target =
            domain.withCurrent(|| FramebufferRenderTargetGL::new(64, 64, 9, 4, domain.stamp()));
        let target_handle = std::ptr::NonNull::from(&mut *target.base.base);
        // Initial target frame plus the two upstream allocation warmups.
        ring_flush(&mut context, target_handle, 0);
        for _ in 0..2 {
            map_ring_frame(&mut context);
            ring_flush(&mut context, target_handle, 16);
        }
        let frames = gpu::kBufferRingSize as usize * 2;
        let mut paths = Vec::with_capacity(frames);
        let mut contours = Vec::with_capacity(frames);
        for _ in 0..frames {
            map_ring_frame(&mut context);
            ring_flush(&mut context, target_handle, 16);
            paths.push(bound_ring_texture(&trace, PATH_BUFFER_IDX));
            contours.push(bound_ring_texture(&trace, CONTOUR_BUFFER_IDX));
        }
        for textures in [&paths, &contours] {
            assert!(textures.iter().all(|id| *id != 0));
            for i in 0..gpu::kBufferRingSize as usize {
                for j in i + 1..gpu::kBufferRingSize as usize {
                    assert_eq!(textures[i] != textures[j], rotates);
                }
            }
            for i in gpu::kBufferRingSize as usize..frames {
                assert_eq!(textures[i], textures[i - gpu::kBufferRingSize as usize]);
            }
        }
        let last = *paths.last().unwrap();
        map_ring_frame(&mut context);
        // Two logical flush descriptors share one frame's storage mapping.
        ring_flush(&mut context, target_handle, 1);
        ring_flush(&mut context, target_handle, 1);
        let index = paths.iter().position(|id| *id == last).unwrap();
        assert_eq!(
            bound_ring_texture(&trace, PATH_BUFFER_IDX),
            paths[(index + 2) % gpu::kBufferRingSize as usize]
        );
        assert_eq!(
            trace.borrow().fence_calls,
            if rotates { frames + 5 } else { 0 }
        );
        drop(target);
        drop(context);
        domain.shutdown();
    }
}

#[test]
fn texel_ring_replaces_pending_or_failed_fence_slots_before_upload() {
    for fail_insert in [false, true] {
        let (domain, mut context, trace) = ring_test_context(true, false);
        prepare_ring_buffers(&mut context);
        trace.borrow_mut().fence_pending = true;
        trace.borrow_mut().fail_insert = fail_insert;
        let mut target =
            domain.withCurrent(|| FramebufferRenderTargetGL::new(64, 64, 9, 4, domain.stamp()));
        let handle = std::ptr::NonNull::from(&mut *target.base.base);
        map_ring_frame(&mut context);
        ring_flush(&mut context, handle, 16);
        let old = bound_ring_texture(&trace, PATH_BUFFER_IDX);
        for _ in 1..gpu::kBufferRingSize {
            map_ring_frame(&mut context);
            ring_flush(&mut context, handle, 16);
        }
        trace.borrow_mut().events.clear();
        map_ring_frame(&mut context);
        ring_flush(&mut context, handle, 16);
        let new = bound_ring_texture(&trace, PATH_BUFFER_IDX);
        assert_ne!(new, old);
        let events = trace.borrow().events.clone();
        let deleted = events
            .iter()
            .position(|event| event == &format!("delete:{old}"))
            .unwrap();
        let rebound = events
            .iter()
            .position(|event| event == &format!("bind:{new}"))
            .unwrap();
        let uploaded = events.iter().position(|event| event == "upload").unwrap();
        assert!(deleted < rebound && rebound < uploaded);
        let fenced = events.iter().position(|event| event == "fence").unwrap();
        let flushed = events.iter().rposition(|event| event == "flush").unwrap();
        assert!(fenced < flushed);
        drop(target);
        drop(context);
        domain.shutdown();
    }
}

#[test]
fn real_ssbo_path_does_not_insert_polyfill_fences() {
    let (domain, mut context, trace) = ring_test_context(true, true);
    prepare_ring_buffers(&mut context);
    let mut target =
        domain.withCurrent(|| FramebufferRenderTargetGL::new(64, 64, 9, 4, domain.stamp()));
    map_ring_frame(&mut context);
    ring_flush(
        &mut context,
        std::ptr::NonNull::from(&mut *target.base.base),
        16,
    );
    assert_eq!(trace.borrow().fence_calls, 0);
    drop(target);
    drop(context);
    domain.shutdown();
}

#[test]
fn pending_program_advancement_is_synchronous_only_for_ubershaders() {
    for mode in [
        ShaderCompilationMode::allowAsynchronous,
        ShaderCompilationMode::onlyUbershaders,
        ShaderCompilationMode::alwaysSynchronous,
    ] {
        let (domain, mut context, trace) = ring_test_context(false, false);
        context.m_pipelineManager.base.m_mode = mode;
        let props = StandardPipelineProps {
            drawType: gpu::DrawType::midpointFanPatches,
            shaderFeatures: gpu::ShaderFeatures::NONE,
            interlockMode: gpu::InterlockMode::rasterOrdering,
            shaderMiscFlags: gpu::ShaderMiscFlags::none,
            #[cfg(feature = "with-rive-tools")]
            synthesizedFailureType: gpu::SynthesizedFailureType::none,
        };
        let mut uber = props;
        uber.shaderFeatures = gpu::UbershaderFeaturesMaskFor(
            props.shaderFeatures,
            props.drawType,
            props.interlockMode,
            props.shaderMiscFlags,
            context.platformFeatures(),
        );
        assert_ne!(props.shaderFeatures, uber.shaderFeatures);
        let (specialized_id, uber_id) = domain.withCurrent(|| {
            assert!(queuePipelineIfNotFound(&mut context, &props));
            assert!(queuePipelineIfNotFound(&mut context, &uber));
            let specialized =
                pipelinePointer(&context, props.createKey(context.platformFeatures())).unwrap();
            let fallback =
                pipelinePointer(&context, uber.createKey(context.platformFeatures())).unwrap();
            // Cache owners remain alive while these identities are read.
            unsafe {
                assert_eq!((*specialized).m_pipelineStatus, PipelineStatus::notReady);
                assert_eq!((*fallback).m_pipelineStatus, PipelineStatus::notReady);
                ((*specialized).id(), (*fallback).id())
            }
        });
        trace.borrow_mut().queries.clear();
        let result = domain
            .withCurrent(|| tryGetPipeline(&mut context, &props))
            .unwrap();
        assert_eq!(unsafe { (*result).id() }, uber_id);
        let queries = trace.borrow().queries.clone();
        assert!(!queries.contains(&(uber_id, GL_COMPLETION_STATUS_KHR)));
        assert!(queries.contains(&(uber_id, GL_LINK_STATUS)));
        if mode == ShaderCompilationMode::onlyUbershaders {
            assert!(!queries.iter().any(|(id, _)| *id == specialized_id));
        } else {
            assert!(queries.contains(&(specialized_id, GL_COMPLETION_STATUS_KHR)));
            assert!(!queries.contains(&(specialized_id, GL_LINK_STATUS)));
        }
        drop(context);
        domain.shutdown();
    }
}

#[test]
fn fence_drop_uses_its_owner_and_restores_a_foreign_current_domain() {
    let (owner, owner_context, owner_trace) = ring_test_context(false, false);
    let (foreign, foreign_context, foreign_trace) = ring_test_context(false, false);
    let mut fence = glutils::Fence::default();
    owner.withCurrent(|| fence.insert());
    owner_trace.borrow_mut().events.clear();
    foreign_trace.borrow_mut().events.clear();
    foreign.withCurrent(|| {
        let identity = currentGLExecutionIdentity();
        drop(fence);
        assert_eq!(currentGLExecutionIdentity(), identity);
    });
    assert_eq!(
        owner_trace
            .borrow()
            .events
            .iter()
            .filter(|event| event.starts_with("delete_sync:"))
            .count(),
        1
    );
    assert!(foreign_trace.borrow().events.is_empty());
    drop(owner_context);
    drop(foreign_context);
    owner.shutdown();
    foreign.shutdown();
}

#[test]
fn stale_fences_cannot_delete_restored_context_syncs() {
    let (old, old_context, old_trace) = ring_test_context(false, false);
    let mut reset_fence = glutils::Fence::default();
    let mut drop_fence = glutils::Fence::default();
    old.withCurrent(|| {
        reset_fence.insert();
        drop_fence.insert();
    });
    let ingress = old_trace.borrow().lifecycle.clone().unwrap();
    assert!(ingress.contextLost());
    assert_eq!(
        ingress.contextRestored(),
        Some(GLContextRecovery::RecreateRenderer)
    );
    assert!(!reset_fence.poll());
    // Restoration creates a new renderer/domain, as the browser host requires.
    // The deterministic provider restarts its name space for that new context.
    let (new, new_context, new_trace) = ring_test_context(false, false);
    let mut live = glutils::Fence::default();
    new.withCurrent(|| live.insert());
    old_trace.borrow_mut().events.clear();
    new_trace.borrow_mut().events.clear();
    new.withCurrent(|| {
        reset_fence.reset();
        drop(drop_fence);
        assert!(live.poll());
    });
    assert!(old_trace.borrow().events.is_empty());
    assert!(new_trace.borrow().events.is_empty());
    drop(live);
    assert_eq!(
        new_trace
            .borrow()
            .events
            .iter()
            .filter(|event| event.starts_with("delete_sync:"))
            .count(),
        1
    );
    drop(old_context);
    drop(new_context);
    old.shutdown();
    new.shutdown();
}
