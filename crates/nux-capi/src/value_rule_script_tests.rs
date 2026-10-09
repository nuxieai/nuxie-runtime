#![allow(clippy::unwrap_used, reason = "handwritten checked script assertions")]
use super::*;
use nuxie::ScriptHostExtension;
use nuxie_runtime::{
    RuntimeRuleGroup, RuntimeRuleGroupMember, RuntimeValueRule, RuntimeValueRuleKind,
    RuntimeValueRuleMode,
};

#[path = "../tests/support/value_markers.rs"]
mod fixture;

/// Name a Luau value type from a witness of the same type.
fn same_type<T>(_: &T, value: T) -> T {
    value
}

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
    let command = HostCommandImportConfig::new(
        "bridge",
        ScriptExecutionLimits::new(),
        HostCommandLimits::new(),
    )
    .unwrap();
    let (extension, slot) = Extension::wrap(command.extension(), Some("bridge".into()));
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
    let source =
        b"return function(root, path, value) return require('bridge').set(root, path, value) end";
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

#[test]
fn no_command_module_leaves_file_module_names_available() {
    let vm = nuxie::ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let (extension, _) = Extension::wrap(Arc::new(nuxie::NoopScriptHostExtension), None);
    let _installed = extension.install(&vm).unwrap();
    assert!(vm.registered_module("value_rules").unwrap().is_nil());
}

/// One active step over the item list fixture, held as the player step holds
/// it around script work: a graph checkpoint, a change capture and the rule
/// operation (all absent without an operation), plus the installed module.
struct Step {
    _guard: Guard,
    _extension: Box<dyn nuxie::ScriptHostExtensionInstance>,
    vm: nuxie::ScriptVm,
    context: Rc<Context>,
    capture: Option<RuntimeViewModelChangeCapture>,
    checkpoint: Option<RuntimeOwnedViewModelGraphTransaction>,
    root: RuntimeOwnedViewModelHandle,
    file: *mut NuxFile,
}

impl Drop for Step {
    fn drop(&mut self) {
        self.capture.take();
        if let Some(checkpoint) = self.checkpoint.take() {
            checkpoint.commit();
        }
        unsafe { nux_file_free(self.file) };
    }
}

impl Step {
    fn new(with_operation: bool) -> Self {
        let bytes = fixture::item_list_fixture();
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
            RuntimeOwnedViewModelInstance::from_instance(native.clone(), 2, 0).unwrap(),
        );
        let mut policy = nuxie::RuntimeValuePolicy::new(native.clone());
        policy
            .set_rules(&[RuntimeValueRule {
                model: "Root".into(),
                property: "items".into(),
                kind: RuntimeValueRuleKind::PickedCount {
                    property: "on".into(),
                    minimum: 0,
                    maximum: 1,
                },
                mode: RuntimeValueRuleMode::Refuse,
                code: "most".into(),
                message: "Too many".into(),
            }])
            .unwrap();
        policy
            .set_groups(&[RuntimeRuleGroup {
                model: "Root".into(),
                valid: "valid".into(),
                members: vec![RuntimeRuleGroupMember {
                    property: "items".into(),
                    errors_path: "items_errors".into(),
                    item_model: "ErrorEntry".into(),
                    code_property: "code".into(),
                    message_property: "message".into(),
                }],
            }])
            .unwrap();
        let roots = std::slice::from_ref(&root);
        let checkpoint = with_operation
            .then(|| RuntimeOwnedViewModelGraphTransaction::begin(roots, 4096).unwrap());
        let operation =
            with_operation.then(|| Rc::new(RefCell::new(policy.begin_rules(roots).unwrap())));
        let capture = with_operation.then(|| {
            let capture = RuntimeViewModelChangeCapture::begin().unwrap();
            policy.prepare_capture(&capture);
            capture
        });
        let vm = nuxie::ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let command = HostCommandImportConfig::new(
            "bridge",
            ScriptExecutionLimits::new(),
            HostCommandLimits::new(),
        )
        .unwrap();
        let (extension, slot) = Extension::wrap(command.extension(), Some("bridge".into()));
        *slot.borrow_mut() = Some(native.downgrade());
        let installed = extension.install(&vm).unwrap();
        let context = Rc::new(Context {
            file: native,
            roots: [(String::new(), root.clone())].into(),
            policy: Rc::new(RefCell::new(Some(policy))),
            operation,
        });
        let guard = Guard(ACTIVE.with(|slot| slot.replace(Some(Rc::clone(&context)))));
        Self {
            _guard: guard,
            _extension: installed,
            vm,
            context,
            capture,
            checkpoint,
            root,
            file,
        }
    }

    fn compile(&self, body: &str) -> Vec<u8> {
        let source = format!("local bridge = require('bridge')\nreturn function()\n{body}\nend");
        let _flags = luaur_common::ScopedAllFlags::enter(true);
        let mut size = 0;
        let code = luaur_compiler::functions::luau_compile::luau_compile(
            source.as_ptr().cast(),
            source.len(),
            std::ptr::null_mut(),
            &mut size,
        );
        assert!(!code.is_null());
        unsafe { std::slice::from_raw_parts(code.cast(), size) }.to_vec()
    }

    /// Run one module call that returns `(applied, code)`.
    fn call(&self, call: &str) -> Result<(bool, Option<String>), String> {
        let witness = self.vm.lua().create_function(|_, ()| Ok(())).unwrap();
        let code = self.compile(&format!("return {call}"));
        let function = same_type(
            &witness,
            self.vm.run_bytecode("checked_batch", &code).unwrap(),
        );
        function.call(()).map_err(|error| error.to_string())
    }

    /// Run `listValues`; each value comes back as "<type>:<value>".
    fn list_values(&self, arguments: &str) -> Result<Option<Vec<String>>, String> {
        let witness = self.vm.lua().create_function(|_, ()| Ok(())).unwrap();
        let code = self.compile(&format!(
            "local values = bridge.listValues({arguments})
            if values == nil then return nil end
            local described = {{}}
            for index = 1, #values do
                described[index] = type(values[index]) .. ':' .. tostring(values[index])
            end
            return described"
        ));
        let function = same_type(
            &witness,
            self.vm.run_bytecode("checked_list", &code).unwrap(),
        );
        function.call(()).map_err(|error| error.to_string())
    }

    /// The `on` flags of the three records, read through the list.
    fn flags(&self) -> [bool; 3] {
        [0, 1, 2].map(|index| {
            self.root
                .list_item_by_property_name_path("items", index)
                .unwrap()
                .borrow()
                .boolean_value_by_property_name("on")
                .unwrap()
        })
    }

    fn operation(&self) -> std::cell::RefMut<'_, nuxie::RuntimeValuePolicyOperation> {
        self.context.operation.as_ref().unwrap().borrow_mut()
    }

    fn write_count(&self) -> usize {
        self.capture.as_ref().unwrap().write_count().unwrap()
    }

    /// Settle the operation and its group, as the step does before commit.
    fn apply(&self) {
        let policy = self.context.policy.borrow();
        let policy = policy.as_ref().unwrap();
        let roots = std::slice::from_ref(&self.root);
        let mut operation = self.operation();
        operation
            .apply(policy, self.capture.as_ref().unwrap(), roots)
            .unwrap();
        operation
            .apply_groups(policy, roots, |_, _| Ok(()))
            .unwrap();
    }

    fn errors(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.root
            .list_items_by_property_name_path("items_errors")
            .unwrap()
            .iter()
            .map(|entry| {
                let entry = entry.borrow();
                let text = |name| entry.string_value_by_property_name(name).unwrap().to_vec();
                (text("code"), text("message"))
            })
            .collect()
    }
}

#[test]
fn script_list_values_reads_one_property_per_item_in_list_order() {
    let step = Step::new(true);
    assert_eq!(
        step.list_values("'', 'items', 'text'").unwrap(),
        Some(vec![
            "string:x".to_owned(),
            "string:y".to_owned(),
            "string:z".to_owned()
        ])
    );
    assert_eq!(
        step.list_values("'', 'items', 'on'").unwrap(),
        Some(vec![
            "boolean:true".to_owned(),
            "boolean:false".to_owned(),
            "boolean:false".to_owned()
        ])
    );
    // A path that names a scalar has no list.
    assert_eq!(step.list_values("'', 'x/text', 'text'").unwrap(), None);
    let null = step.list_values("'', 'holes', 'text'").unwrap_err();
    assert!(null.contains("checked list has a null item"), "{null}");
    assert_eq!(step.write_count(), 0);
}

#[test]
fn script_set_all_refuses_the_whole_batch_with_the_first_rule_code() {
    let step = Step::new(true);
    assert_eq!(step.flags(), [true, false, false]);
    assert_eq!(
        step.call(
            "bridge.setAll({
                {root = '', path = 'x/on', value = false},
                {root = '', path = 'y/on', value = true},
                {root = '', path = 'z/on', value = true},
            })"
        )
        .unwrap(),
        (false, Some("most".to_owned()))
    );
    assert_eq!(step.flags(), [true, false, false]);
    step.apply();
    assert_eq!(
        step.operation()
            .reports()
            .iter()
            .map(|report| (report.rule_index, report.refused))
            .collect::<Vec<_>>(),
        vec![(0, true)]
    );
    assert_eq!(step.flags(), [true, false, false]);
    assert_eq!(
        step.errors(),
        vec![(b"most".to_vec(), b"Too many".to_vec())]
    );
}

#[test]
fn script_set_all_applies_an_accepted_batch_through_item_references() {
    let step = Step::new(true);
    assert_eq!(
        step.call(
            "bridge.setAll({
                {root = '', path = 'x/on', value = false},
                {root = '', path = 'y/on', value = true},
                {root = '', path = 'z/on', value = false},
            })"
        )
        .unwrap(),
        (true, None)
    );
    assert_eq!(step.flags(), [false, true, false]);
    step.apply();
    assert_eq!(step.flags(), [false, true, false]);
    assert!(step.operation().reports().is_empty());
    assert!(step.errors().is_empty());
}

#[test]
fn script_set_all_clears_every_flag_and_keeps_the_items() {
    let step = Step::new(true);
    assert_eq!(
        step.call(
            "bridge.setAll({
                {root = '', path = 'x/on', value = false},
                {root = '', path = 'y/on', value = false},
                {root = '', path = 'z/on', value = false},
            })"
        )
        .unwrap(),
        (true, None)
    );
    assert_eq!(step.flags(), [false, false, false]);
    assert_eq!(
        step.root.list_item_count_by_property_name_path("items"),
        Some(3)
    );
}

#[test]
fn script_set_all_rejects_malformed_writes_before_any_write() {
    let step = Step::new(true);
    for (call, expected) in [
        (
            "bridge.setAll(true)",
            "checked writes must be a list of tables",
        ),
        (
            "bridge.setAll({[2] = {root = '', path = 'y/on', value = true}})",
            "checked writes must have the keys 1..n",
        ),
        (
            "bridge.setAll({first = {root = '', path = 'y/on', value = true}})",
            "checked writes must have the keys 1..n",
        ),
        (
            "bridge.setAll({{path = 'y/on', value = true}})",
            "checked write needs a string root and path",
        ),
        (
            "bridge.setAll({{root = '', value = true}})",
            "checked write needs a string root and path",
        ),
        (
            "bridge.setAll({{root = '', path = 'y/on', valeu = true}})",
            "checked write fields are root, path and value",
        ),
        (
            "bridge.setAll({
                {root = '', path = 'x/on', value = false},
                {root = '', path = 'y/on', value = {}},
            })",
            "checked value must be a scalar or nil",
        ),
        (
            "bridge.setAll((function()
                local writes = {}
                for index = 1, 4097 do
                    writes[index] = {root = '', path = 'y/on', value = true}
                end
                return writes
            end)())",
            "checked writes exceed 4096 entries",
        ),
        // The single-value set stays scalar or nil.
        (
            "bridge.set('', 'y/on', {})",
            "checked value must be a scalar or nil",
        ),
    ] {
        let error = step.call(call).unwrap_err();
        assert!(error.contains(expected), "{call}: {error}");
        assert_eq!(step.flags(), [true, false, false], "{call}");
        assert_eq!(step.write_count(), 0, "{call}");
        assert!(step.operation().reports().is_empty(), "{call}");
    }
}

#[test]
fn script_set_all_raises_batch_errors_after_a_valid_first_write() {
    let step = Step::new(true);
    for (call, expected) in [
        (
            "bridge.setAll({
                {root = '', path = 'x/on', value = false},
                {root = '', path = 'nope/on', value = true},
            })",
            "checked value batch: NotFound",
        ),
        (
            "bridge.setAll({
                {root = '', path = 'x/on', value = false},
                {root = '', path = 'x/on', value = true},
            })",
            "checked value batch: InvalidArgument",
        ),
        // 4096 writes pass the list bound and reach the batch, which refuses
        // the repeated property.
        (
            "bridge.setAll((function()
                local writes = {}
                for index = 1, 4096 do
                    writes[index] = {root = '', path = 'y/on', value = true}
                end
                return writes
            end)())",
            "checked value batch: InvalidArgument",
        ),
    ] {
        let error = step.call(call).unwrap_err();
        assert!(error.contains(expected), "{call}: {error}");
        assert!(!error.contains("exceed"), "{call}: {error}");
        assert_eq!(step.flags(), [true, false, false], "{call}");
        assert_eq!(step.write_count(), 0, "{call}");
        assert!(step.operation().reports().is_empty(), "{call}");
    }
}

#[test]
fn script_set_all_raises_while_the_rule_operation_is_borrowed() {
    let step = Step::new(true);
    let busy = step.operation();
    let error = step
        .call("bridge.setAll({{root = '', path = 'y/on', value = true}})")
        .unwrap_err();
    assert!(error.contains("value rule operation is active"), "{error}");
    assert!(busy.reports().is_empty());
    drop(busy);
    assert_eq!(step.flags(), [true, false, false]);
    assert_eq!(step.write_count(), 0);
}

#[test]
fn script_calls_outside_a_step_say_whether_they_read_or_write() {
    let step = Step::new(true);
    let active = ACTIVE.with(|slot| slot.replace(None));
    let read = step.list_values("'', 'items', 'text'").unwrap_err();
    let batch = step
        .call("bridge.setAll({{root = '', path = 'y/on', value = true}})")
        .unwrap_err();
    let single = step.call("bridge.set('', 'y/on', true)").unwrap_err();
    ACTIVE.with(|slot| *slot.borrow_mut() = active);
    assert!(
        read.contains("checked reads require an active step"),
        "{read}"
    );
    assert!(
        batch.contains("checked writes require an active step"),
        "{batch}"
    );
    assert!(
        single.contains("checked writes require an active step"),
        "{single}"
    );
    assert_eq!(step.flags(), [true, false, false]);
    assert_eq!(step.write_count(), 0);
}

#[test]
fn script_set_all_requires_the_step_operation() {
    let step = Step::new(false);
    let error = step
        .call("bridge.setAll({{root = '', path = 'y/on', value = true}})")
        .unwrap_err();
    assert!(
        error.contains("checked batches require a value rule operation"),
        "{error}"
    );
    assert_eq!(step.flags(), [true, false, false]);
}
