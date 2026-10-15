// Supplemental coverage of 6fa2cccf's GL precreation API. Included in the
// implementation test module so this exercises the actual queue and driver
// command path, not a duplicate implementation.
fn precreation_test_context(
    parallel: bool,
) -> (
    GLExecutionDomain,
    Box<RenderContextGLImpl>,
    Rc<RefCell<Vec<GLCommand>>>,
) {
    let commands = Rc::new(RefCell::new(Vec::new()));
    let domain = GLExecutionDomain::new(Box::new(CanvasTestProvider {
        commands: commands.clone(),
        lifecycleIngress: None,
        finalReleaseIngress: Rc::new(RefCell::new(None)),
        finalReleaseWake: std::sync::Arc::new(TestFinalReleaseWake::default()),
    }));
    let mut caps = GLCapabilities::default();
    caps.setKHR_parallel_shader_compile(parallel);
    let context = domain.withCurrent(|| newComponent097TestContextOwner(caps, domain.clone()));
    commands.borrow_mut().clear();
    (domain, context, commands)
}

#[test]
fn async_ubershader_precreation_without_extension_has_no_effect() {
    let (domain, mut context, commands) = precreation_test_context(false);
    assert_eq!(currentGLExecutionIdentity(), None);
    assert!(!context.tryStartAsyncUbershaderCreation(gpu::InterlockMode::rasterOrdering));
    assert_eq!(currentGLExecutionIdentity(), None);
    assert!(context.m_pipelineManager.base.m_pipelines.is_empty());
    assert!(commands.borrow().is_empty());
    drop(context);
    domain.shutdown();
}

#[test]
fn async_ubershader_precreation_queues_supported_permutations_once() {
    for mode in [
        gpu::InterlockMode::rasterOrdering,
        gpu::InterlockMode::clockwise,
        gpu::InterlockMode::atomics,
        gpu::InterlockMode::depthStencil,
    ] {
        let (domain, mut context, commands) = precreation_test_context(true);
        let mut expected = std::collections::BTreeSet::new();
        let mut resolve_keys = std::collections::BTreeSet::new();
        crate::mechanical_port::source::renderer::src::gpu_cpp::ForEachUbershaderPermutation(
            mode,
            context.platformFeatures(),
            |drawType, shaderFeatures, shaderMiscFlags| {
                let props = StandardPipelineProps {
                    drawType,
                    shaderFeatures,
                    interlockMode: mode,
                    shaderMiscFlags,
                    #[cfg(feature = "with-rive-tools")]
                    synthesizedFailureType: gpu::SynthesizedFailureType::none,
                };
                let key = props.createKey(context.platformFeatures());
                if drawType == gpu::DrawType::renderPassResolve {
                    resolve_keys.insert(key);
                    if mode != gpu::InterlockMode::atomics {
                        return true;
                    }
                }
                expected.insert(key);
                true
            },
        );
        assert!(!expected.is_empty());
        assert_eq!(currentGLExecutionIdentity(), None);
        assert!(context.tryStartAsyncUbershaderCreation(mode));
        assert_eq!(currentGLExecutionIdentity(), None);
        let actual: std::collections::BTreeSet<_> = context
            .m_pipelineManager
            .base
            .m_pipelines
            .keys()
            .copied()
            .collect();
        assert_eq!(actual, expected);
        for key in resolve_keys {
            assert_eq!(actual.contains(&key), mode == gpu::InterlockMode::atomics);
        }
        // The provider reports compilation pending. Precreation must not turn
        // an async request into a synchronous link-status/ready path.
        assert!(context
            .m_pipelineManager
            .base
            .m_pipelines
            .values()
            .all(|program| {
                program.as_ref().unwrap().m_pipelineStatus == PipelineStatus::notReady
            }));
        assert!(!commands.borrow().is_empty());
        commands.borrow_mut().clear();
        assert!(context.tryStartAsyncUbershaderCreation(mode));
        assert!(
            commands.borrow().is_empty(),
            "already queued programs are retained"
        );
        assert_eq!(
            context.m_pipelineManager.base.m_pipelines.len(),
            expected.len()
        );
        drop(context);
        domain.shutdown();
    }
}

#[test]
fn async_ubershader_precreation_isolates_and_restores_foreign_current_domain() {
    let (owner, mut context, owner_commands) = precreation_test_context(true);
    let (foreign, foreign_context, foreign_commands) = precreation_test_context(true);
    foreign.withCurrent(|| {
        let before = currentGLExecutionIdentity().expect("foreign domain is current");
        assert!(context.tryStartAsyncUbershaderCreation(gpu::InterlockMode::rasterOrdering));
        assert_eq!(currentGLExecutionIdentity(), Some(before));
        assert!(!owner_commands.borrow().is_empty());
        assert!(foreign_commands.borrow().is_empty());
    });
    assert_eq!(currentGLExecutionIdentity(), None);
    assert!(foreign_context
        .m_pipelineManager
        .base
        .m_pipelines
        .is_empty());
    drop(context);
    drop(foreign_context);
    owner.shutdown();
    foreign.shutdown();
}
