#![allow(clippy::unwrap_used, reason = "handwritten checked script assertions")]
use super::*;
use nuxie::ScriptHostExtension;
use nuxie_runtime::{RuntimeValueRule, RuntimeValueRuleKind, RuntimeValueRuleMode};

#[path = "../tests/support/value_markers.rs"]
mod fixture;

#[test]
fn script_set_answers_synchronously_and_preserves_ordered_reports() {
    let bytes = fixture::fixture(None, &[], false);
    let mut file = std::ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_file_import(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file,
            )
        },
        NuxStatus::Ok
    );
    let native = unsafe { &*file }.file.clone();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::new(native.clone(), 0).unwrap(),
    );
    let mut policy = nuxie::RuntimeValuePolicy::new(native.clone());
    policy
        .set_rules(&[
            RuntimeValueRule {
                model: "Values".into(),
                property: "n".into(),
                kind: RuntimeValueRuleKind::NumberMinimum(10.0),
                mode: RuntimeValueRuleMode::Mark,
                code: "low".into(),
                message: "At least ten".into(),
            },
            RuntimeValueRule {
                model: "Values".into(),
                property: "n".into(),
                kind: RuntimeValueRuleKind::NumberMaximum(365.0),
                mode: RuntimeValueRuleMode::Refuse,
                code: "high".into(),
                message: "At most 365".into(),
            },
        ])
        .unwrap();
    let roots = std::slice::from_ref(&root);
    let checkpoint = RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap();
    let operation = Rc::new(RefCell::new(policy.begin_rules(roots).unwrap()));
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    policy.prepare_capture(&capture);
    let vm = nuxie::ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let (extension, slot) = Extension::wrap(Arc::new(nuxie::NoopScriptHostExtension), false);
    *slot.borrow_mut() = Some(native.downgrade());
    let _extension = extension.install(&vm).unwrap();
    let context = Rc::new(Context {
        file: native,
        roots: [(String::new(), root.clone())].into(),
        policy: Rc::new(RefCell::new(Some(policy))),
        operation: Some(Rc::clone(&operation)),
    });
    let _guard = Guard(ACTIVE.with(|slot| slot.replace(Some(Rc::clone(&context)))));
    let _flags = luaur_common::ScopedAllFlags::enter(true);
    let source = b"return function(root, path, value) return require('value_rules').set(root, path, value) end";
    let mut size = 0;
    let code = luaur_compiler::functions::luau_compile::luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut size,
    );
    assert!(!code.is_null());
    let code = unsafe { std::slice::from_raw_parts(code.cast(), size) };
    let witness = vm.lua().create_function(|_, ()| Ok(())).unwrap();
    let set = same_type(&witness, vm.run_bytecode("checked_write", code).unwrap());
    for (value, expected) in [
        (5.0, (true, None)),
        (400.0, (false, Some("high".to_owned()))),
        (20.0, (true, None)),
    ] {
        let result: (bool, Option<String>) = set.call(("", "n", value)).unwrap();
        assert_eq!(result, expected);
        assert_eq!(
            root.borrow().number_value_by_property_name("n"),
            Some(if value == 400.0 { 5.0 } else { value })
        );
    }
    let policy = context.policy.borrow();
    operation
        .borrow_mut()
        .apply(policy.as_ref().unwrap(), &capture, roots)
        .unwrap();
    assert_eq!(
        operation
            .borrow()
            .reports()
            .iter()
            .map(|report| (report.rule_index, report.refused))
            .collect::<Vec<_>>(),
        vec![(0, false), (1, true)]
    );
    let changes = root.resolve_change_capture(capture).unwrap();
    assert_eq!(changes.len(), 2);
    checkpoint.commit();
    unsafe {
        nux_file_free(file);
    }
}
