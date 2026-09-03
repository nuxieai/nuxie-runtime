//! Upstream tests/unit_tests/renderer/ore_deferred_alias_test.cpp at e949498e.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{
    context::ContextApi,
    ore_cmd::{ore_deferred_resource::DeferredShaderModule, ore_handle::REAL_RESOURCE_FLAG},
    types::*,
};
use std::collections::HashSet;

#[test]
fn recycled_address_resolves_through_object_holding_it_now() {
    let mut a = DeferredOreContext::fromReal(None);
    let desc = ShaderModuleDesc::default();
    let mut dead = HashSet::new();
    {
        let mut modules = vec![];
        for _ in 0..32 {
            let module = a.makeShaderModule(&desc).unwrap();
            dead.insert(module.allocation_identity());
            modules.push(module);
        }
    }
    let mut b = DeferredOreContext::fromReal(None);
    let mut recycled = None;
    let mut modules = vec![];
    for _ in 0..32 {
        if recycled.is_some() {
            break;
        }
        let module = b.makeShaderModule(&desc).unwrap();
        if dead.contains(&module.allocation_identity()) {
            recycled = Some(module.clone());
        }
        modules.push(module);
    }
    // The upstream premise is deliberately required, never a vacuous pass.
    // Like the C++ original, this allocator-reuse test must be excluded in an
    // AddressSanitizer run whose quarantine disables address reuse.
    let recycled = recycled.expect("allocator must recycle a dead module address");
    let own = recycled
        .downcast_ref::<DeferredShaderModule>()
        .unwrap()
        .clientHandle();
    assert_eq!(own & REAL_RESOURCE_FLAG, 0);
    assert_eq!(b.handleFor(Some(&recycled)), own);
    let foreign = a.handleFor(Some(&recycled));
    assert_ne!(foreign, own);
    assert_ne!(foreign & REAL_RESOURCE_FLAG, 0);
    assert!(a
        .makePipeline(
            &PipelineDesc {
                vertexModule: Some(&recycled),
                ..Default::default()
            },
            None
        )
        .is_some());
    assert_eq!(a.handleFor(Some(&recycled)), foreign);
}

#[test]
fn session_teardown_off_recording_thread_stays_quiet() {
    let mut d = Box::new(DeferredOreContext::fromReal(None));
    let desc = BufferDesc {
        usage: BufferUsage::vertex,
        size: 16,
        data: None,
        immutable: false,
        label: None,
    };
    let live = d.makeBuffer(&desc).unwrap();
    let doomed = d.makeBuffer(&desc).unwrap();
    std::thread::spawn(move || drop(doomed)).join().unwrap();
    drop(live);
    // This exact source fixture transfers only a terminal destructor, not a
    // usable Context. It created no passes, canvas callbacks, bound real device,
    // exported stream aliases, or retained real-resource references. All GPU
    // handles are gone before transfer; only Arc/Mutex destroy queues remain.
    // The private capsule never exposes the !Send Rc owners on another thread.
    struct Teardown(Box<DeferredOreContext>);
    impl Teardown {
        fn destroy(self) {
            drop(self);
        }
    }
    // SAFETY: Construction is confined to this fixture after the above joins
    // and the ownership assertions below. The two realResources Rc owners are
    // both inside this capsule (context + its stream callback), with no Weak
    // aliases. No Rc allocation can be accessed concurrently across transfer.
    unsafe impl Send for Teardown {}
    d.assertExclusiveTeardownFixture();
    let teardown = Teardown(d);
    std::thread::spawn(move || teardown.destroy())
        .join()
        .unwrap();
}
