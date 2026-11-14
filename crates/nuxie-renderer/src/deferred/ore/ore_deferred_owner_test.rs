//! A recording keeps the owners handed to it until the frame is reset.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{context::ContextApi, types::*};
use std::{
    any::Any,
    rc::{Rc, Weak},
};

fn owner() -> (Rc<dyn Any>, Weak<dyn Any>) {
    let owner: Rc<dyn Any> = Rc::new(7_u32);
    let weak = Rc::downgrade(&owner);
    (owner, weak)
}

#[test]
fn the_context_keeps_an_owner_until_reset_frame() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let (owner, weak) = owner();
    ctx.keepWithRecording(owner);
    assert!(weak.upgrade().is_some(), "the recording holds the owner");
    ctx.resetFrame();
    assert!(weak.upgrade().is_none(), "reset drops the owner");
}

#[test]
fn a_pass_hands_its_owner_to_the_stream_it_records_into() {
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut pass = ctx
        .beginRenderPass(
            &RenderPassDesc {
                colorCount: 0,
                ..Default::default()
            },
            None,
        )
        .unwrap();
    let (owner, weak) = owner();
    pass.keepWithRecording(owner);
    pass.finish();
    drop(pass);
    assert!(weak.upgrade().is_some(), "the stream outlives the pass");
    ctx.resetFrame();
    assert!(weak.upgrade().is_none(), "reset drops the owner");
}

#[test]
fn keeping_the_same_owner_twice_keeps_it_once() {
    let ctx = DeferredOreContext::fromReal(None);
    let (owner, _weak) = owner();
    ctx.keepWithRecording(Rc::clone(&owner));
    ctx.keepWithRecording(owner);
    assert_eq!(ctx.stream().borrow().keptOwners(), 1);
}
