//! Complete context-wrapper cache regressions from scripting_context_test.cpp
//! at a637bc5e. Lua rawequal deliberately tests wrapper identity, not __eq.
use super::*;
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{Artboard, File, NoopScriptHost, RuntimeFactoryHandle, ScriptViewModel};

fn models() -> (ScriptViewModel, ScriptViewModel) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(root.join("tests/unit_tests/assets/scripted_color.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let source = file
        .with_file(|file| file.artboard_named_source("ColorArtboard"))
        .unwrap();
    let artboard = Artboard::instance_from_handle(&source).unwrap();
    let create = || {
        let instance = file
            .with_file(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .unwrap();
        ScriptViewModel::from_native(instance, file.clone()).unwrap()
    };
    let first = create();
    let second = create();
    assert!(first.native_instance() != second.native_instance());
    (first, second)
}

struct Harness {
    _vm: ScriptVm,
    instance: Box<dyn ScriptInstance>,
    console: Rc<RefCell<Vec<String>>>,
    _factory: PersistentFactory<RecordingFactory>,
}
impl Harness {
    fn new(source: &str, model: ScriptViewModel) -> Self {
        let console = Rc::new(RefCell::new(Vec::new()));
        let output = console.clone();
        let vm = ScriptVm::new_with_log_sink(move |_, bytes| {
            output
                .borrow_mut()
                .push(String::from_utf8(bytes.to_vec()).unwrap());
        });
        vm.install_rive_globals().unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        vm.renderer_bindings
            .bootstrap_render_context(&mut factory)
            .unwrap();
        let generator: Function = vm
            .lua
            .load(source)
            .eval()
            .expect("top result is a function");
        let program = ScriptProgram { generator };
        let mut instance = vm
            .instantiate_registered_script_with_context(&program, Some(model), Vec::new())
            .unwrap();
        assert!(
            instance
                .call_init_with_factory(&mut NoopScriptHost, &mut factory)
                .unwrap()
        );
        Self {
            _vm: vm,
            instance,
            console,
            _factory: factory,
        }
    }
    fn trigger(&mut self, name: &str) {
        self.instance
            .call_input_trigger(name, &mut NoopScriptHost)
            .unwrap();
    }
    fn dispose(self) {
        // LuaScriptInstance::drop performs scriptDispose while the VM and
        // renderer factory are still retained by this harness.
        drop(self.instance);
    }
}

#[test]
fn context_view_model_wrappers_are_reused_across_calls() {
    let (model, _) = models();
    let harness = Harness::new(
        r#"
function init(self, context)
  self.context = context
  print(tostring(rawequal(context:viewModel(), context:viewModel())))
  print(tostring(rawequal(context:rootViewModel(), context:rootViewModel())))
  print(tostring(rawequal(context:dataContext(), context:dataContext())))
  local dc = context:dataContext()
  print(tostring(rawequal(dc:viewModel(), dc:viewModel())))
  return true
end
return function()
  return { init = init, context = late() }
end
"#,
        model,
    );
    assert_eq!(
        &*harness.console.borrow(),
        &["true", "true", "true", "true"]
    );
    harness.dispose();
}

#[test]
fn repeated_context_view_model_access_allocates_no_new_wrappers() {
    let (model, _) = models();
    let mut harness = Harness::new(
        r#"
function init(self, context)
  self.context = context
  return true
end
function churn(self)
  local first = self.context:viewModel()
  local firstProp = first:getColor("colorProp")
  local sameViewModel = true
  local sameProperty = true
  for i = 1, 2000 do
    local vm = self.context:viewModel()
    if not rawequal(vm, first) then sameViewModel = false end
    if not rawequal(vm:getColor("colorProp"), firstProp) then sameProperty = false end
  end
  print(tostring(sameViewModel))
  print(tostring(sameProperty))
end
return function()
  return { init = init, churn = churn, context = late() }
end
"#,
        model,
    );
    harness.trigger("churn");
    assert_eq!(&*harness.console.borrow(), &["true", "true"]);
    harness.dispose();
}

#[test]
fn context_view_model_wrapper_is_rebuilt_when_the_instance_changes() {
    let (first, second) = models();
    assert!(first.set_color("colorProp", 0xff112233));
    assert!(second.set_color("colorProp", 0xff445566));
    let mut harness = Harness::new(
        r#"
function init(self, context)
  self.context = context
  return true
end
function probe(self)
  local vm = self.context:viewModel()
  print(tostring(vm:getColor("colorProp").value))
end
return function()
  return { init = init, probe = probe, context = late() }
end
"#,
        first,
    );
    harness.trigger("probe");
    assert_eq!(harness.console.borrow().len(), 1);
    let first_value = harness.console.borrow()[0].clone();
    harness
        .instance
        .set_context_view_model(Some(second))
        .unwrap();
    harness.trigger("probe");
    assert_eq!(harness.console.borrow().len(), 2);
    assert_ne!(harness.console.borrow()[1], first_value);
    harness.dispose();
}

// Rust-boundary regression: C++ resolves the method before entering
// rive_lua_pcall_with_context. A property resolved by __index belongs to the
// previous scope, not to the object whose method is about to run.
#[test]
#[cfg(feature = "tools")]
fn advance_method_lookup_does_not_claim_properties_for_the_callee() {
    let (model, _) = models();
    assert!(model.set_color("colorProp", 0xff112233));
    let mut harness = Harness::new(
        r#"
return function()
  return { init = function(self, context)
    setmetatable(self, { __index = function(_, key)
      if key == "advance" then
        lookupProperty = context:viewModel():getColor("colorProp")
        return function() return {} end
      end
    end })
    return true
  end }
end
"#,
        model,
    );
    assert!(
        harness
            .instance
            .call_advance_truthy(0.0, &mut NoopScriptHost)
            .unwrap()
    );
    let read = || {
        harness
            ._vm
            .lua
            .load("return lookupProperty.value")
            .eval::<u32>()
            .unwrap()
    };
    assert_eq!(read(), 0xff112233);
    harness.instance.invalidate_for_init_retry();
    assert_eq!(
        read(),
        0xff112233,
        "callee teardown must not dispose the lookup's orphan"
    );
    harness._vm.dispose_orphan_scripted_properties(false);
    assert_eq!(
        read(),
        0,
        "the lookup property must remain an untagged orphan"
    );
    harness.dispose();
}
