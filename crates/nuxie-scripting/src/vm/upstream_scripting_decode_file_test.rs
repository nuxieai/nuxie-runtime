//! Live cases from scripting_decode_file_test.cpp at 6cd5d108.
use super::*;
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::RuntimeArtboardInstanceHandle, nested_artboard::NestedArtboard,
    viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle, RuntimeFileHandle, ScriptViewModel};

fn bytes(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    std::fs::read(root.join("tests/unit_tests/assets").join(name)).unwrap()
}
fn import(name: &str, factory: &mut PersistentFactory<SerializingFactory>) -> RuntimeFileHandle {
    File::import(
        &bytes(name),
        RuntimeFactoryHandle::from_factory(factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}
const SCRIPT: &str = r#"
file = nil
function decode(context: Context, bytes: buffer)
    local status = "unsettled"
    context:decodeFile(bytes):andThen(function(decoded)
        file = decoded
        status = "ok"
    end, function(reason)
        status = reason
    end)
    return status
end
function names() return table.concat(file.artboardNames, ",") end
function assign(host, name)
    local bindable = if name == "" then file:bindableArtboard()
        else file:bindableArtboard(name)
    host:getArtboard("someArtboard").value = bindable
    return if bindable == nil then "nil" else bindable.name
end
function clear(host) host:getArtboard("someArtboard").value = nil return nil end
function forget() file = nil return nil end
"#;
struct Script {
    vm: ScriptVm,
}
impl Script {
    fn new(factory: &mut PersistentFactory<SerializingFactory>) -> Self {
        let vm = ScriptVm::new();
        vm.install_render_factory(factory).unwrap();
        vm.install_rive_globals().unwrap();
        vm.lua
            .globals()
            .set(
                "context",
                vm.lua
                    .create_userdata(view_model::ScriptedContext::new(
                        &vm.lua,
                        Rc::new(RefCell::new(None)),
                        vec![],
                        Rc::new(Cell::new(false)),
                        None,
                    ))
                    .unwrap(),
            )
            .unwrap();
        vm.lua.load(SCRIPT).exec().unwrap();
        Self { vm }
    }
    fn decode(&self, bytes: &[u8]) -> String {
        self.vm
            .lua
            .globals()
            .get::<Function>("decode")
            .unwrap()
            .call((
                self.vm.lua.globals().get::<AnyUserData>("context").unwrap(),
                self.vm.lua.create_buffer(bytes).unwrap(),
            ))
            .unwrap()
    }
    fn host(&self, host: &Host) -> Table {
        view_model::create_scripted_view_model(&self.vm.lua, host.model()).unwrap()
    }
    fn assign(&self, host: &Host, name: &str) -> String {
        self.vm
            .lua
            .globals()
            .get::<Function>("assign")
            .unwrap()
            .call((self.host(host), name))
            .unwrap()
    }
    fn clear(&self, host: &Host) {
        self.vm
            .lua
            .globals()
            .get::<Function>("clear")
            .unwrap()
            .call::<()>(self.host(host))
            .unwrap();
    }
    fn forget(&self) {
        self.vm
            .lua
            .globals()
            .get::<Function>("forget")
            .unwrap()
            .call::<()>(())
            .unwrap();
        self.vm.lua.gc_collect().unwrap();
    }
}
struct Host {
    view_model: CoreHandle,
    machine: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    file: RuntimeFileHandle,
}
impl Host {
    fn new(factory: &mut PersistentFactory<SerializingFactory>) -> Self {
        let file = import("bindable_artboard_nesty.riv", factory);
        let artboard = file.with_file(File::artboard_default).unwrap();
        let machine = artboard.state_machine_at(0).unwrap();
        let view_model = file
            .with_file(|f| f.create_view_model_instance_for_artboard(f.artboard().unwrap()))
            .unwrap();
        Self {
            view_model,
            machine,
            artboard,
            file,
        }
    }
    fn model(&self) -> ScriptViewModel {
        ScriptViewModel::from_native(self.view_model.clone(), self.file.clone()).unwrap()
    }
    fn property(&self) -> CoreHandle {
        self.view_model
            .with(|v| {
                v.as_view_model_instance()
                    .unwrap()
                    .property_value_named("someArtboard")
            })
            .unwrap()
            .unwrap()
    }
    fn bound(&self) -> Option<CoreHandle> {
        self.property()
            .with_downcast::<ViewModelInstanceArtboard, _>(|p| p.bound_view_model_instance())
            .unwrap()
    }
    fn bind(&self) {
        self.machine
            .with_instance_mut(|m| m.bind_view_model_instance(self.view_model.clone()));
    }
    fn mounted(&self) -> Option<RuntimeArtboardInstanceHandle> {
        let source = self
            .property()
            .with_downcast::<ViewModelInstanceArtboard, _>(|p| p.asset())
            .flatten()?
            .source_artboard_handle();
        self.artboard.with_artboard(|a| {
            a.objects().iter().flatten().find_map(|object| {
                object
                    .with_downcast::<NestedArtboard, _>(|n| n.artboard_instance_default())
                    .flatten()
                    .filter(|a| a.with_artboard(|a| a.artboard_source_handle()) == source)
            })
        })
    }
    fn shows(&self, name: &str) -> bool {
        self.artboard.with_artboard(|a| {
            a.objects().iter().flatten().any(|o| {
                o.with_downcast::<NestedArtboard, _>(|n| n.artboard_instance_default())
                    .flatten()
                    .is_some_and(|a| a.with_artboard(|a| a.base.base.name() == name))
            })
        })
    }
}
#[test]
fn decode_resolves_names_rejects_malformed_and_raises_non_bytes() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let script = Script::new(&mut factory);
    let child = bytes("bindable_artboard_child.riv");
    assert_eq!(script.decode(&child), "ok");
    let expected = import("bindable_artboard_child.riv", &mut factory).with_file(|f| {
        (0..f.artboard_count())
            .map(|i| f.artboard_name_at(i))
            .collect::<Vec<_>>()
            .join(",")
    });
    assert_eq!(
        script
            .vm
            .lua
            .globals()
            .get::<Function>("names")
            .unwrap()
            .call::<String>(())
            .unwrap(),
        expected
    );
    assert_eq!(
        script.decode(&[1, 2, 3, 4, 5, 6, 7, 8]),
        "malformed: not a Rive file"
    );
    assert_eq!(
        script.decode(&child[..child.len() / 2]),
        "malformed: not a Rive file"
    );
    for (expression, message) in [
        ("buffer.create(0)", "decodeFile: empty buffer"),
        ("'RIVE'", "buffer"),
    ] {
        let error = script
            .vm
            .lua
            .load(format!("return context:decodeFile({expression})"))
            .eval::<Value>()
            .unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    }
}
#[test]
fn decoded_caches_survive_their_first_access_coroutine() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let host = Host::new(&mut factory);
    let script = Script::new(&mut factory);
    assert_eq!(script.decode(&bytes("bindable_artboard_child.riv")), "ok");
    let lua = &script.vm.lua;
    lua.globals().set("host", script.host(&host)).unwrap();
    lua.load(
        r#"
        property = host:getArtboard("someArtboard")
        bindable = file:bindableArtboard("Artboard")
        property.value = bindable
        local thread = coroutine.create(function()
            firstData = bindable.data
            firstAsset = property.value
            assert(firstData ~= nil)
            assert(firstAsset ~= nil)
        end)
        local ok, reason = coroutine.resume(thread)
        assert(ok, reason)
    "#,
    )
    .exec()
    .unwrap();
    lua.gc_collect().unwrap();
    lua.gc_collect().unwrap();
    lua.load(
        r#"
        assert(bindable.data == firstData)
        assert(property.value == firstAsset)
        assert(property.value.name == "Artboard")
        property.value = nil
        bindable = nil
        firstData = nil
        firstAsset = nil
        property = nil
        host = nil
    "#,
    )
    .exec()
    .unwrap();
    lua.gc_collect().unwrap();
    lua.gc_collect().unwrap();
}

#[test]
fn decoded_artboard_assignment_mounts_and_clears() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let host = Host::new(&mut factory);
    let script = Script::new(&mut factory);
    assert_eq!(script.decode(&bytes("bindable_artboard_child.riv")), "ok");
    host.bind();
    assert_eq!(script.assign(&host, "Artboard"), "Artboard");
    host.machine.advance_and_apply(0.016);
    let nested = host.mounted().unwrap();
    let bound = host.bound().unwrap();
    let mounted_bound = nested
        .with_artboard(|a| a.data_context())
        .unwrap()
        .with_context(|c| c.main_view_model_instance());
    assert_eq!(mounted_bound, Some(bound.clone()));
    let child = import("bindable_artboard_child.riv", &mut factory);
    let model_name = |model: CoreHandle| {
        model
            .with(|m| m.as_view_model().unwrap().base.name().to_owned())
            .unwrap()
    };
    let actual_model = bound
        .with(|v| v.as_view_model_instance().unwrap().get_view_model())
        .flatten()
        .unwrap();
    let expected_model = child.with_file(|f| {
        let artboard = f.artboard_named("Artboard").unwrap();
        f.view_model(artboard.with_artboard(|a| a.base.base.view_model_id()) as usize)
            .unwrap()
    });
    assert_eq!(model_name(actual_model), model_name(expected_model));
    let default_name = child
        .with_file(File::artboard_default)
        .unwrap()
        .with_artboard(|a| a.base.base.name().to_owned());
    assert_eq!(script.assign(&host, ""), default_name);
    assert_eq!(script.assign(&host, "Nope"), "nil");
    assert!(
        host.property()
            .with_downcast::<ViewModelInstanceArtboard, _>(|p| p.asset())
            .flatten()
            .is_none()
    );
    assert!(host.bound().is_none());
}
#[test]
fn mounted_bindable_keeps_decoded_file_alive() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let host = Host::new(&mut factory);
    let script = Script::new(&mut factory);
    host.bind();
    let mut renderer = factory.borrow().make_renderer();
    for _ in 0..2 {
        assert_eq!(script.decode(&bytes("bindable_artboard_child.riv")), "ok");
        assert_eq!(script.assign(&host, "Artboard"), "Artboard");
        script.forget();
        host.artboard.draw(&mut renderer);
        for _ in 0..3 {
            host.machine.advance_and_apply(0.1);
            host.artboard.draw(&mut renderer);
        }
        assert!(host.mounted().is_some());
    }
    script.clear(&host);
    script.vm.lua.gc_collect().unwrap();
    host.artboard.draw(&mut renderer);
    for _ in 0..3 {
        host.machine.advance_and_apply(0.1);
        host.artboard.draw(&mut renderer);
    }
    assert!(!host.shows("Artboard"));
}
#[test]
fn replacing_bindable_notifies_and_same_bindable_does_not_remount() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let host = Host::new(&mut factory);
    let script = Script::new(&mut factory);
    script.decode(&bytes("bindable_artboard_child.riv"));
    script
        .vm
        .lua
        .globals()
        .set("host", script.host(&host))
        .unwrap();
    script
        .vm
        .lua
        .load(
            r#"
changes = 0
host:getArtboard("someArtboard"):addListener(function() changes += 1 end)
function fresh()
    bindable = file:bindableArtboard("Artboard")
    host:getArtboard("someArtboard").value = bindable
end
function same() host:getArtboard("someArtboard").value = bindable end
"#,
        )
        .exec()
        .unwrap();
    host.bind();
    let call = |name| {
        script
            .vm
            .lua
            .globals()
            .get::<Function>(name)
            .unwrap()
            .call::<()>(())
            .unwrap();
        host.machine.advance_and_apply(0.016);
        host.mounted().unwrap()
    };
    let first = call("fresh");
    assert_eq!(script.vm.lua.globals().get::<u32>("changes").unwrap(), 1);
    assert!(first.downgrade().ptr_eq(&call("same").downgrade()));
    assert!(first.downgrade().ptr_eq(&call("same").downgrade()));
    assert!(!first.downgrade().ptr_eq(&call("fresh").downgrade()));
    assert_eq!(script.vm.lua.globals().get::<u32>("changes").unwrap(), 2);
}

#[test]
fn script_assigned_bindable_renders_like_embedder_api() {
    use nuxie_runtime::source::{math::vec2d::Vec2D, pointer_button::PointerButton};
    fn render(host: &Host, factory: &PersistentFactory<SerializingFactory>) {
        let (width, height) = host
            .artboard
            .with_artboard(|a| (a.base.base.width(), a.base.base.height()));
        factory.borrow_mut().frame_size(width as u32, height as u32);
        let mut renderer = factory.borrow().make_renderer();
        host.bind();
        host.machine.advance_and_apply(0.016);
        host.artboard.draw(&mut renderer);
        factory.borrow_mut().add_frame();
        host.machine.with_instance_mut(|m| {
            m.pointer_down(Vec2D::new(250.0, 250.0), 0, PointerButton::Primary);
            m.pointer_up(Vec2D::new(250.0, 250.0), 0, PointerButton::Primary);
        });
        host.machine.advance_and_apply(0.016);
        host.artboard.draw(&mut renderer);
        for _ in 0..5 {
            factory.borrow_mut().add_frame();
            host.machine.advance_and_apply(0.1);
            host.artboard.draw(&mut renderer);
        }
        factory.borrow_mut().add_frame();
    }
    let mut embedder_factory = PersistentFactory::new(SerializingFactory::new());
    let embedder_host = Host::new(&mut embedder_factory);
    let child = import("bindable_artboard_child.riv", &mut embedder_factory);
    let bindable = child
        .with_file(|f| f.bindable_artboard_named("Artboard"))
        .unwrap();
    let bound = child.with_file(|f| {
        f.create_default_view_model_instance_for_artboard(
            bindable.source_artboard_handle().unwrap(),
        )
    });
    embedder_host
        .property()
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|p| {
            p.set_bound_view_model_instance(bound);
            p.set_asset(Some(bindable));
        });
    render(&embedder_host, &embedder_factory);
    let mut script_factory = PersistentFactory::new(SerializingFactory::new());
    let script_host = Host::new(&mut script_factory);
    let script = Script::new(&mut script_factory);
    assert_eq!(script.decode(&bytes("bindable_artboard_child.riv")), "ok");
    assert_eq!(script.assign(&script_host, "Artboard"), "Artboard");
    render(&script_host, &script_factory);
    let expected = embedder_factory.borrow().bytes().to_vec();
    assert!(!expected.is_empty());
    assert_eq!(script_factory.borrow().bytes().to_vec(), expected);
}

#[test]
fn async_body_fetches_decodes_and_binds_artboard() {
    use nuxie_runtime::source::scriptnet::{
        http::{HttpRequest, HttpResponse},
        net,
        net_policy::NetLimits,
    };
    use std::sync::{Arc, Mutex};
    let _guard = super::upstream_scripting_fetch_test::NET_TEST_LOCK
        .lock()
        .unwrap();
    #[derive(Default)]
    struct BytesProvider(Mutex<Vec<u32>>);
    impl net::Provider for BytesProvider {
        fn start(&self, id: u32, _: &HttpRequest) {
            self.0.lock().unwrap().push(id);
        }
    }
    let provider = Arc::new(BytesProvider::default());
    net::set_provider(Some(provider.clone()));
    net::set_limits(NetLimits::default());
    {
        let mut factory = PersistentFactory::new(SerializingFactory::new());
        let host = Host::new(&mut factory);
        let script = Script::new(&mut factory);
        script
            .vm
            .lua
            .load(
                r#"
state = { step = "started" }
function load(context: Context, host)
    local character = host:getArtboard("someArtboard")
    async(function()
        local ok, response = await(fetch("https://example.com/child.riv"))
        if not ok then
            state.step = `fetch failed: {response}`
            return nil
        end
        local _, bytes = await(response:arrayBuffer())
        local loaded, file = await(context:decodeFile(bytes))
        if not loaded then
            state.step = `decode failed: {file}`
            return nil
        end
        host:getArtboard("someArtboard").value = file:bindableArtboard("Artboard")
        state.step = if character.value ~= nil then character.value.name else "unbound"
        return nil
    end)
    return nil
end
"#,
            )
            .exec()
            .unwrap();
        script
            .vm
            .lua
            .globals()
            .get::<Function>("load")
            .unwrap()
            .call::<()>((
                script
                    .vm
                    .lua
                    .globals()
                    .get::<AnyUserData>("context")
                    .unwrap(),
                script.host(&host),
            ))
            .unwrap();
        let starts = provider.0.lock().unwrap().clone();
        assert_eq!(starts.len(), 1);
        net::complete(
            starts[0],
            HttpResponse {
                status: 200,
                url: b"https://example.com/child.riv".to_vec(),
                body: bytes("bindable_artboard_child.riv"),
                ..Default::default()
            },
        );
        nuxie_runtime::rive_poll_async_work(32);
        assert_eq!(
            script
                .vm
                .lua
                .globals()
                .get::<Table>("state")
                .unwrap()
                .get::<String>("step")
                .unwrap(),
            "Artboard"
        );
        host.bind();
        host.machine.advance_and_apply(0.016);
        assert!(host.mounted().is_some());
    }
    net::set_provider(None);
}

#[test]
#[cfg(feature = "tools")]
fn decoded_files_run_only_signed_scripts_in_their_own_vm() {
    use nuxie_runtime::{
        RuntimeScriptingVmHandle,
        source::{assets::script_asset::ScriptAsset, scripted::decoded_file::import_decoded_file},
    };
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let factory_handle = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let scripts = |file: &RuntimeFileHandle| {
        file.with_file(|f| {
            f.assets()
                .iter()
                .filter(|a| a.with_downcast::<ScriptAsset, _>(|_| ()).is_some())
                .cloned()
                .collect::<Vec<_>>()
        })
    };
    let fresh_vm = || {
        let vm = ScriptVm::new();
        factory_handle
            .with_factory_mut(|f| vm.install_render_factory(f))
            .unwrap();
        RuntimeScriptingVmHandle::new(Box::new(vm))
    };
    let unsigned = bytes("library_scope_edge_test.riv");
    let embedded = File::import(
        &unsigned,
        factory_handle.clone(),
        None,
        None,
        Some(fresh_vm()),
    )
    .unwrap();
    let assets = scripts(&embedded);
    assert!(!assets.is_empty());
    for asset in assets {
        assert!(
            !asset
                .with_downcast::<ScriptAsset, _>(|s| s.verified())
                .unwrap()
        );
    }
    assert!(embedded.with_file(|f| f.scripting_vm()).is_some());
    let mut lazy_vm = |_: RuntimeFactoryHandle| fresh_vm();
    let decoded =
        import_decoded_file(&unsigned, factory_handle.clone(), None, Some(&mut lazy_vm)).unwrap();
    assert!(decoded.with_file(|f| f.scripting_vm()).is_none());
    for asset in scripts(&decoded) {
        assert!(
            !asset
                .with_downcast::<ScriptAsset, _>(|s| s.has_generator())
                .unwrap()
        );
    }
    let signed = import_decoded_file(
        &bytes("joel_signed.riv"),
        factory_handle.clone(),
        None,
        Some(&mut lazy_vm),
    )
    .unwrap();
    assert!(signed.with_file(|f| f.scripting_vm()).is_some());
    let mut registered = false;
    for asset in scripts(&signed) {
        asset.with_downcast::<ScriptAsset, _>(|s| {
            assert!(s.verified());
            registered |= s.has_generator();
        });
    }
    assert!(registered);
}
