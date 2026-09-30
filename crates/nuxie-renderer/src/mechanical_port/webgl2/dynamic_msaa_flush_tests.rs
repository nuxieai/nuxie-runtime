// Source-derived regressions for ff8e6c4847c1686bbacbe4da5cf67b420769ff5f.
// Included in render_context_gl_impl::tests. Exercise the actual flush loop;
// the recording provider checks GL commands, not rasterized pixels.

fn dynamic_msaa_descriptor(
    target: std::ptr::NonNull<RenderTarget>,
    list: &mut gpu::BlockAllocatedLinkedList<gpu::DrawBatch>,
) -> gpu::FlushDescriptor {
    gpu::FlushDescriptor {
        renderTarget: Some(target),
        combinedShaderFeatures: gpu::ShaderFeatures::NONE,
        interlockMode: gpu::InterlockMode::depthStencil,
        msaaSampleCount: 4,
        colorLoadAction: gpu::LoadAction::clear,
        colorClearValue: 0,
        coverageClearValue: 0,
        depthClearValue: 0.0,
        stencilClearValue: 0,
        renderTargetUpdateBounds: gpu::IAABB {
            left: 0,
            top: 0,
            right: 64,
            bottom: 64,
        },
        virtualTileWidth: 0,
        virtualTileHeight: 0,
        manuallyResolved: false,
        fixedFunctionColorOutput: true,
        featherAtlasTextureWidth: 0,
        featherAtlasTextureHeight: 0,
        featherAtlasContentWidth: 0,
        featherAtlasContentHeight: 0,
        coverageBufferPrefix: 0,
        needsCoverageBufferClear: false,
        flushUniformDataOffsetInBytes: 0,
        pathCount: 0,
        firstPath: 0,
        firstPaint: 0,
        firstPaintAux: 0,
        contourCount: 0,
        firstContour: 0,
        gradSpanCount: 0,
        firstGradSpan: 0,
        tessVertexSpanCount: 0,
        firstTessVertexSpan: 0,
        gradDataHeight: 0,
        gradTextureHeight: 0,
        tessDataHeight: 0,
        clockwiseFillOverride: false,
        hasTriangleVertices: false,
        wireframe: false,
        ditherMode: gpu::DitherMode::none,
        #[cfg(feature = "with-rive-tools")]
        synthesizedFailureType: gpu::SynthesizedFailureType::none,
        externalCommandBuffer: None,
        featherAtlasFillBatches: None,
        featherAtlasFillBatchCount: 0,
        featherAtlasStrokeBatches: None,
        featherAtlasStrokeBatchCount: 0,
        drawList: Some(std::ptr::NonNull::from(list)),
        firstDstBlendBarrier: None,
        unresolvedBarriers: gpu::BarrierFlags::none,
    }
}

fn dynamic_msaa_flush_commands(draw_type: gpu::DrawType) -> Vec<GLCommand> {
    let commands = Rc::new(RefCell::new(Vec::new()));
    let domain = GLExecutionDomain::new(Box::new(CanvasTestProvider {
        commands: commands.clone(),
        lifecycleIngress: None,
        finalReleaseIngress: Rc::new(RefCell::new(None)),
        finalReleaseWake: std::sync::Arc::new(TestFinalReleaseWake::default()),
    }));
    let mut capabilities = GLCapabilities::default();
    capabilities.maxSupportedInstancesPerFlush = u32::MAX;
    let mut context =
        domain.withCurrent(|| newComponent097TestContextOwner(capabilities, domain.clone()));
    assert!(context.platformFeatures().supportsPipelineDynamicState);
    *context.base.m_flushUniformBuffer = makeUniformBufferRing(&context, 256);
    let mut target =
        domain.withCurrent(|| FramebufferRenderTargetGL::new(64, 64, 9, 4, domain.stamp()));
    let props = StandardPipelineProps {
        drawType: draw_type,
        shaderFeatures: gpu::ShaderFeatures::NONE,
        interlockMode: gpu::InterlockMode::depthStencil,
        shaderMiscFlags: gpu::ShaderMiscFlags::none,
        #[cfg(feature = "with-rive-tools")]
        synthesizedFailureType: gpu::SynthesizedFailureType::none,
    };
    // Seed a ready driver program, as this is flush dispatch coverage, not a
    // claim that the recording provider compiles shaders or produces pixels.
    let key = props.createKey(context.platformFeatures());
    let program = Box::new(DrawProgram {
        m_fragmentShader: std::ptr::null(),
        m_vertexShader: std::ptr::null(),
        m_pipelineStatus: PipelineStatus::ready,
        m_id: 77,
        m_baseVertexUniformLocation: 24,
        m_baseInstanceUniformLocation: 23,
        m_state: ManuallyDrop::new((&*context.m_state).clone()),
        #[cfg(feature = "with-rive-tools")]
        m_synthesizedFailureType: gpu::SynthesizedFailureType::none,
    });
    context
        .m_pipelineManager
        .base
        .m_pipelines
        .insert(key, Some(program));
    let mut list = gpu::BlockAllocatedLinkedList::default();
    let mut batch = gpu::DrawBatch::new(
        draw_type,
        gpu::ShaderMiscFlags::none,
        gpu::DrawContents::clockwiseFill | gpu::DrawContents::opaquePaint,
        5,
        7,
        nuxie_render_api::BlendMode::SrcOver,
        gpu::ImageSampler::default(),
        gpu::BarrierFlags::none,
    );
    batch.indexCountPerInstance = 12;
    batch.baseIndex = 3;
    batch.scissorRect = Some(gpu::AABBu16 {
        left: 2,
        top: 3,
        right: 31,
        bottom: 41,
    });
    list.push_back(batch);
    let descriptor =
        dynamic_msaa_descriptor(std::ptr::NonNull::from(&mut *target.base.base), &mut list);
    commands.borrow_mut().clear();
    // SAFETY: target, list, batch and their owned GL execution state remain
    // alive and unaliased for the synchronous flush call.
    unsafe { flush(&mut context, &descriptor) };
    let result = commands.borrow().clone();
    drop(target);
    drop(context);
    domain.shutdown();
    result
}

#[test]
fn dynamic_msaa_batches_use_three_exact_states_without_changing_draw_or_scissor() {
    for draw_type in [
        gpu::DrawType::stencilDynamicMidpointFans,
        gpu::DrawType::stencilDynamicOuterCubics,
    ] {
        let commands = dynamic_msaa_flush_commands(draw_type);
        let draws: Vec<_> = commands
            .iter()
            .enumerate()
            .filter_map(|(i, command)| {
                matches!(command, GLCommand::DrawElements { .. }).then_some(i)
            })
            .collect();
        assert_eq!(draws.len(), 3);
        let last_before = |end: usize, predicate: fn(&GLCommand) -> bool| {
            commands[..end]
                .iter()
                .rev()
                .find(|command| predicate(command))
                .unwrap()
                .clone()
        };
        for (pass, &index) in draws.iter().enumerate() {
            assert_eq!(
                commands[index],
                GLCommand::DrawElements {
                    mode: GL_TRIANGLES,
                    count: 5 * gpu::dsFillPatchIndexCount(gpu::drawTypeSubmitsOuterCubicPatches(draw_type)),
                    type_: GL_UNSIGNED_SHORT,
                    offset: gpu::dsFillIndexOffset(gpu::drawTypeSubmitsOuterCubicPatches(draw_type)) as u32,
                }
            );
            assert_eq!(
                commands[index - 1],
                GLCommand::Uniform1iLocation {
                    location: 24,
                    value: if gpu::drawTypeSubmitsOuterCubicPatches(draw_type) {
                        (7 << gpu::DSOuterCubicFillPatchStrideLog2) | gpu::DSFillVertexFlagOuterCubic
                    } else {
                        7 << gpu::DSMidpointFanFillPatchStrideLog2
                    }
                }
            );
            assert_eq!(
                last_before(index, |c| matches!(c, GLCommand::UseProgram(_))),
                GLCommand::UseProgram(77)
            );
            assert_eq!(
                last_before(index, |c| matches!(c, GLCommand::ColorMask(..))),
                GLCommand::ColorMask(pass == 1, pass == 1, pass == 1, pass == 1)
            );
            assert_eq!(
                last_before(index, |c| matches!(c, GLCommand::DepthMask(_))),
                GLCommand::DepthMask(pass == 1)
            );
            assert_eq!(
                last_before(index, |c| matches!(c, GLCommand::CullFace(_))),
                GLCommand::CullFace(if pass == 1 { GL_BACK } else { GL_FRONT })
            );
            assert_eq!(
                last_before(index, |c| matches!(c, GLCommand::StencilMask(_))),
                GLCommand::StencilMask(0x7f)
            );
            for capability in [
                GL_DEPTH_TEST,
                GL_STENCIL_TEST,
                GL_CULL_FACE,
                GL_SCISSOR_TEST,
            ] {
                let state = commands[..index].iter().rev().find(|c| matches!(c, GLCommand::Enable(v) | GLCommand::Disable(v) if *v == capability));
                assert_eq!(state, Some(&GLCommand::Enable(capability)));
            }
            let blend = commands[..index].iter().rev().find(|c| {
                matches!(
                    c,
                    GLCommand::Enable(GL_BLEND) | GLCommand::Disable(GL_BLEND)
                )
            });
            assert_eq!(blend, Some(&GLCommand::Disable(GL_BLEND)));
            let stencil: Vec<_> = commands[..index]
                .iter()
                .filter(|c| {
                    matches!(
                        c,
                        GLCommand::StencilFunc(..)
                            | GLCommand::StencilOp(..)
                            | GLCommand::StencilFuncSeparate(..)
                            | GLCommand::StencilOpSeparate(..)
                    )
                })
                .cloned()
                .collect();
            if pass == 0 {
                assert!(stencil.ends_with(&[
                    GLCommand::StencilFunc(GL_ALWAYS, 0x80, 0xff),
                    GLCommand::StencilOp(GL_KEEP, GL_KEEP, GL_INCR_WRAP),
                ]));
            } else {
                assert!(stencil.ends_with(&[
                    GLCommand::StencilFuncSeparate(GL_FRONT, GL_EQUAL, 0x80, 0x7f),
                    GLCommand::StencilOpSeparate(GL_FRONT, GL_DECR, GL_KEEP, GL_KEEP),
                    GLCommand::StencilFuncSeparate(GL_BACK, GL_LESS, 0x80, 0x7f),
                    GLCommand::StencilOpSeparate(GL_BACK, GL_KEEP, GL_KEEP, GL_ZERO),
                ]));
            }
        }
        assert_eq!(
            commands
                .iter()
                .filter(|c| matches!(c, GLCommand::UseProgram(77)))
                .count(),
            1
        );
        assert!(commands.contains(&GLCommand::Scissor(2, 23, 29, 38)));
        assert!(!commands[draws[0] + 1..draws[2]].iter().any(|c| matches!(
            c,
            GLCommand::Scissor(..)
                | GLCommand::Enable(GL_SCISSOR_TEST)
                | GLCommand::Disable(GL_SCISSOR_TEST)
        )));
    }
}

#[test]
fn ordinary_msaa_batch_still_issues_one_draw() {
    let commands = dynamic_msaa_flush_commands(gpu::DrawType::stencilMidpointFans);
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, GLCommand::DrawElements { .. }))
            .count(),
        1
    );
}
