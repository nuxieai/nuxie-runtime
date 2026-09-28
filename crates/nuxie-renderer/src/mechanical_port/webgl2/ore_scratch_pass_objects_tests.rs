// Included inside ore_context_gl_impl::tests to reuse its command provider.
// Command/lifetime coverage, NOT a replacement for the browser pixel scenarios.

#[test]
fn scratch_resolve_is_reused_detached_and_vao_arrays_are_scrubbed() {
    let (domain, trace) = execution([401, 402, 403]);
    let mut ctx = context(&domain);
    for iteration in 0..2 {
        let mut pass = beginRenderPass(&mut ctx, &RenderPassDesc::default(), None).unwrap();
        {
            // Install the source finish metadata directly: this unit exercises
            // teardown, while the browser scenarios exercise real attachments.
            let gl = pass.asAny().downcast_ref::<RenderPassGL>().unwrap();
            let mut state = gl.inner.borrowState();
            state.m_usedAttribs = true;
            state.m_maxAttribSlot = 2;
            state.m_glResolveCount = 1;
            state.m_glResolves[0] = GLResolveEntry {
                colorIndex: 0,
                resolveTarget: GL_TEXTURE_2D,
                resolveTex: 77,
                width: 64,
                height: 64,
            };
        }
        clearTrace(&trace);
        pass.finish();
        let commands = trace.borrow().commands.clone();
        for slot in 0..=2 {
            assert!(commands.contains(&GLCommand::DisableVertexAttribArray(slot)));
        }
        assert!(commands.contains(&GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0)));
        assert!(commands.contains(&GLCommand::FramebufferTexture2D {
            target: GL_DRAW_FRAMEBUFFER,
            attachment: GL_COLOR_ATTACHMENT0,
            texture_target: GL_TEXTURE_2D,
            texture: 0,
            level: 0
        }));
        assert!(!commands.contains(&GLCommand::DeleteFramebuffer(403)));
        assert_eq!(
            trace.borrow().generated,
            if iteration == 0 {
                vec![(GLObjectKind::Framebuffer, 403)]
            } else {
                vec![]
            }
        );
    }
    clearTrace(&trace);
    let mut empty = RenderPassDesc::default();
    empty.colorCount = 0;
    beginRenderPass(&mut ctx, &empty, None).unwrap().finish();
    assert!(trace
        .borrow()
        .commands
        .contains(&GLCommand::DrawBuffers(vec![GL_COLOR_ATTACHMENT0])));
    assert!(trace
        .borrow()
        .commands
        .contains(&GLCommand::ReadBuffer(GL_COLOR_ATTACHMENT0)));
    drop(ctx);
    domain.withCurrent(|| {});
    assert_eq!(
        trace
            .borrow()
            .commands
            .iter()
            .filter(|c| **c == GLCommand::DeleteFramebuffer(403))
            .count(),
        1
    );
    domain.shutdown();
}

#[test]
fn lost_context_abandons_lent_scratch_names_without_gl_deletion() {
    let (domain, trace) = execution([501, 502]);
    let mut ctx = context(&domain);
    let pass = beginRenderPass(&mut ctx, &RenderPassDesc::default(), None).unwrap();
    let lifecycle = trace.borrow().lifecycleIngress.clone().unwrap();
    assert!(lifecycle.contextLost());
    clearTrace(&trace);
    drop(ctx);
    drop(pass);
    assert!(trace.borrow().commands.is_empty());
    domain.shutdown();
}

#[test]
fn scratch_pair_is_reused_and_two_color_depth_attachments_are_scrubbed() {
    let (domain, trace) = execution([11, 12, 13, 14, 101, 102]);
    let mut ctx = context(&domain);
    let mut view = |size, format| {
        let texture = makeTexture(
            &mut ctx,
            &TextureDesc {
                width: size,
                height: size,
                format,
                renderTarget: true,
                ..TextureDesc::default()
            },
        )
        .unwrap();
        makeTextureView(
            &mut ctx,
            &TextureViewDesc {
                texture: Some(&texture),
                ..TextureViewDesc::default()
            },
        )
        .unwrap()
    };
    let a0 = view(32, TextureFormat::rgba8unorm);
    let a1 = view(32, TextureFormat::rgba8unorm);
    let depth = view(32, TextureFormat::depth32float);
    let b = view(64, TextureFormat::rgba8unorm);
    let mut desc_a = RenderPassDesc::default();
    desc_a.colorCount = 2;
    desc_a.colorAttachments[0].view = Some(&a0);
    desc_a.colorAttachments[1].view = Some(&a1);
    desc_a.depthStencil.view = Some(&depth);
    beginRenderPass(&mut ctx, &desc_a, None).unwrap().finish();
    assert!(trace
        .borrow()
        .generated
        .contains(&(GLObjectKind::Framebuffer, 101)));
    assert!(trace
        .borrow()
        .generated
        .contains(&(GLObjectKind::VertexArray, 102)));
    clearTrace(&trace);
    let mut desc_b = RenderPassDesc::default();
    desc_b.colorAttachments[0].view = Some(&b);
    let mut pass = beginRenderPass(&mut ctx, &desc_b, None).unwrap();
    assert!(
        trace.borrow().generated.is_empty(),
        "borrowed pair must not consume new names"
    );
    let commands = trace.borrow().commands.clone();
    let detach = |attachment| GLCommand::FramebufferTexture2D {
        target: GL_FRAMEBUFFER,
        attachment,
        texture_target: GL_TEXTURE_2D,
        texture: 0,
        level: 0,
    };
    let indices: Vec<_> = [
        GL_COLOR_ATTACHMENT0,
        GL_COLOR_ATTACHMENT0 + 1,
        GL_DEPTH_ATTACHMENT,
    ]
    .into_iter()
    .map(|attachment| {
        commands
            .iter()
            .position(|c| *c == detach(attachment))
            .expect("old attachment detached")
    })
    .collect();
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
    let attach_b = commands
        .iter()
        .position(|c| matches!(c, GLCommand::FramebufferTexture2D { texture: 14, .. }))
        .unwrap();
    assert!(indices[2] < attach_b);
    assert!(commands.contains(&GLCommand::ReadBuffer(GL_COLOR_ATTACHMENT0)));
    clearTrace(&trace);
    pass.finish();
    pass.finish();
    drop(pass);
    assert!(!trace.borrow().commands.iter().any(|c| matches!(
        c,
        GLCommand::DeleteFramebuffer(_) | GLCommand::DeleteVertexArray(_)
    )));
    assert!(trace
        .borrow()
        .commands
        .contains(&GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0)));
    drop((a0, a1, depth, b));
    drop(ctx);
    domain.withCurrent(|| {});
    assert_eq!(
        trace
            .borrow()
            .commands
            .iter()
            .filter(|c| **c == GLCommand::DeleteFramebuffer(101))
            .count(),
        1
    );
    assert_eq!(
        trace
            .borrow()
            .commands
            .iter()
            .filter(|c| **c == GLCommand::DeleteVertexArray(102))
            .count(),
        1
    );
    domain.shutdown();
}

#[test]
fn overlapping_pass_owns_distinct_names_and_restores_outer_pair() {
    let (domain, trace) = execution([201, 202, 301, 302]);
    let mut ctx = context(&domain);
    let mut outer = beginRenderPass(&mut ctx, &RenderPassDesc::default(), None).unwrap();
    {
        let mut state = trace.borrow_mut();
        state.integers.insert(GL_FRAMEBUFFER_BINDING, 201);
        state.integers.insert(GL_VERTEX_ARRAY_BINDING, 202);
    }
    let mut inner = beginRenderPass(&mut ctx, &RenderPassDesc::default(), None).unwrap();
    assert_eq!(
        trace.borrow().generated,
        vec![
            (GLObjectKind::Framebuffer, 201),
            (GLObjectKind::VertexArray, 202),
            (GLObjectKind::Framebuffer, 301),
            (GLObjectKind::VertexArray, 302)
        ]
    );
    clearTrace(&trace);
    inner.finish();
    assert!(trace
        .borrow()
        .commands
        .contains(&GLCommand::DeleteFramebuffer(301)));
    assert!(trace
        .borrow()
        .commands
        .contains(&GLCommand::DeleteVertexArray(302)));
    assert!(trace
        .borrow()
        .commands
        .contains(&GLCommand::BindFramebuffer(GL_FRAMEBUFFER, 201)));
    assert!(trace
        .borrow()
        .commands
        .contains(&GLCommand::BindVertexArray(202)));
    clearTrace(&trace);
    outer.finish();
    assert!(!trace.borrow().commands.iter().any(|c| matches!(
        c,
        GLCommand::DeleteFramebuffer(_) | GLCommand::DeleteVertexArray(_)
    )));
    drop((inner, outer));
    drop(ctx);
    domain.withCurrent(|| {});
    assert_eq!(
        trace
            .borrow()
            .commands
            .iter()
            .filter(|c| **c == GLCommand::DeleteFramebuffer(201))
            .count(),
        1
    );
    assert_eq!(
        trace
            .borrow()
            .commands
            .iter()
            .filter(|c| **c == GLCommand::DeleteVertexArray(202))
            .count(),
        1
    );
    domain.shutdown();
}
