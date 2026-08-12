#![cfg(feature = "async")]

use std::future::Future;
use std::task::{Context, Poll, Waker};

use nuxie_scripting::vm::ScriptVm;

#[test]
fn default_build_preserves_luaur_async_api() {
    let vm = ScriptVm::new();
    let function = vm
        .lua()
        .create_async_function(|_, value: i64| async move { Ok(value + 1) })
        .expect("default scripting builds expose luaur's async bindings");

    let mut future = Box::pin(function.call_async::<i64>(41));
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(result) = future.as_mut().poll(&mut context) else {
        panic!("immediately ready async callback stayed pending");
    };
    assert_eq!(result.expect("async callback completes"), 42);
}
