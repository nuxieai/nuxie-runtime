#![cfg(feature = "luau")]

use nuxie_scripting::vm::ScriptVm;

mod support;
use support::ScriptVmSourceTestExt as _;

#[test]
fn native_argument_errors_leave_the_same_vm_usable() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    for _ in 0..3 {
        for source in [
            "return Mat4.fromScale({})",
            "return Mat4.identity().m55",
            "Mat4.identity():writeToBuffer(buffer.create(63), 0)",
        ] {
            let error = vm.eval::<()>(source).unwrap_err();
            assert!(!error.to_string().is_empty());
            let value: f64 = vm.eval("return Mat4.fromScale(3).m11").unwrap();
            assert_eq!(value, 3.0);
        }
    }
}

#[test]
fn failed_module_registration_does_not_poison_later_registration() {
    let vm = ScriptVm::new();

    for attempt in 0..3 {
        let name = format!("recovery_{attempt}");
        let error = vm
            .register_source_module(&name, "return require('missing_dependency')")
            .unwrap_err();
        assert!(!error.to_string().is_empty());

        vm.register_source_module(&name, "return { value = 42 }")
            .unwrap();
        let value: i64 = vm.eval(&format!("return require('{name}').value")).unwrap();
        assert_eq!(value, 42);
    }
}
