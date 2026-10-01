//! All cases from tests/unit_tests/runtime/scripting/scripting_gpu_target_test.cpp.
use super::*;
use nuxie_ore_metal::context::TargetDesc;
use nuxie_ore_metal::types::TextureFormat;
use nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext;

fn vm_with_target(ore: Option<Rc<RefCell<DeferredOreContext>>>) -> ScriptVm {
    let vm = ScriptVm::new();
    let mut factory = nuxie_render_api::PersistentFactory::new(RoutedTestFactory {
        inner: nuxie_render_api::RecordingFactory::new(),
        ore: ore.map(|ore| ore as nuxie_render_api::OreContextHandle),
        canvas_host: None,
    });
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    let context = view_model::ScriptedContext::new(
        Rc::new(RefCell::new(None)),
        Vec::new(),
        Rc::new(Cell::new(false)),
        None,
    );
    vm.lua()
        .globals()
        .set("context", vm.lua().create_userdata(context).unwrap())
        .unwrap();
    vm
}

fn recording() -> Rc<RefCell<DeferredOreContext>> {
    Rc::new(RefCell::new(DeferredOreContext::fromReal(None)))
}

fn run_init(vm: &ScriptVm, source: &str) -> bool {
    vm.lua().load(source).exec().unwrap();
    vm.lua().load("return init({}, context)").eval().unwrap()
}

#[test]
fn gpu_target_is_nil_without_a_recording_context() {
    let vm = vm_with_target(None);
    assert!(run_init(
        &vm,
        "function init(self, context) return context:gpuTarget() == nil end"
    ));
}

#[test]
fn gpu_target_is_there_in_init_before_a_target_is_declared() {
    let vm = vm_with_target(Some(recording()));
    assert!(run_init(
        &vm,
        r#"
function init(self, context)
  local target = context:gpuTarget()
  return target ~= nil and target.view == nil and target.width == 0
end
"#
    ));
}

#[test]
fn hidden_target_still_validates_the_pass_it_drops() {
    let ore = recording();
    let vm = vm_with_target(Some(ore.clone()));
    assert!(run_init(
        &vm,
        r#"
function init(self, context)
  local target = context:gpuTarget()
  target:beginRenderPass({ color = {{ storeOp = 'store' }} }):finish()
  local ok = pcall(function()
    target:beginRenderPass({ color = {{ loadOp = 'clear' }} })
  end)
  return not ok
end
"#
    ));
    assert!(!ore.borrow().targetDrawn());
}

#[test]
fn script_reads_target_and_draws_into_it() {
    let ore = recording();
    ore.borrow_mut().setTarget(TargetDesc {
        width: 64,
        height: 32,
        format: TextureFormat::bgra8unorm,
        sampleCount: 4,
    });
    let vm = vm_with_target(Some(ore.clone()));
    assert!(run_init(
        &vm,
        r#"
function init(self, context)
  local target = context:gpuTarget()
  assert(target.width == 64 and target.height == 32)
  assert(target.format == 'bgra8unorm' and target.sampleCount == 4)
  assert(target.view ~= nil)
  local pass = target:beginRenderPass({
    color = {{ loadOp = 'clear', storeOp = 'store' }},
  })
  pass:finish()
  return true
end
"#
    ));
    assert!(ore.borrow().targetDrawn());
}

#[test]
fn pass_on_frame_without_target_drops_commands() {
    let ore = recording();
    ore.borrow_mut().setTarget(TargetDesc {
        width: 64,
        height: 64,
        ..TargetDesc::default()
    });
    let vm = vm_with_target(Some(ore.clone()));
    assert!(run_init(
        &vm,
        r#"
local target
function init(self, context)
  target = context:gpuTarget()
  return target ~= nil
end
function drawLater()
  local pass = target:beginRenderPass({ color = {{ storeOp = 'store' }} })
  pass:setViewport(0, 0, 1, 1)
  pass:finish()
  return target.width == 0 and target.view == nil
end
"#
    ));
    ore.borrow_mut().resetFrame();
    ore.borrow_mut().setTarget(TargetDesc::default());
    assert!(vm.lua().load("return drawLater()").eval::<bool>().unwrap());
    assert!(!ore.borrow().targetDrawn());
}

#[test]
fn cached_target_view_drops_hidden_and_errors_once_replaced() {
    let ore = recording();
    ore.borrow_mut().setTarget(TargetDesc {
        width: 64,
        height: 64,
        ..TargetDesc::default()
    });
    let vm = vm_with_target(Some(ore.clone()));
    assert!(run_init(
        &vm,
        r#"
local target, view
function init(self, context)
  target = context:gpuTarget()
  view = target.view
  return view ~= nil
end
function drawCached()
  target:beginRenderPass({ color = {{ view = view, storeOp = 'store' }} }):finish()
end
"#
    ));
    ore.borrow_mut().resetFrame();
    ore.borrow_mut().setTarget(TargetDesc::default());
    vm.lua().load("drawCached()").exec().unwrap();
    assert!(!ore.borrow().targetDrawn());
    ore.borrow_mut().resetFrame();
    ore.borrow_mut().setTarget(TargetDesc {
        width: 64,
        height: 64,
        ..TargetDesc::default()
    });
    vm.lua().load("drawCached()").exec().unwrap();
    assert!(ore.borrow().targetDrawn());
    ore.borrow_mut().resetFrame();
    ore.borrow_mut().setTarget(TargetDesc {
        width: 128,
        height: 64,
        ..TargetDesc::default()
    });
    assert!(vm.lua().load("drawCached()").exec().is_err());
}
