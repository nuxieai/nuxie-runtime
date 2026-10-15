// scripting_artboard_test.cpp at 8f06a91c: host-created thread with null
// thread data, artboard instance, context replacement, collection, VM close
// while the importing File remains alive.
#[test]
fn script_artboard_instances_work_from_host_threads_and_survive_context_replacement() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let mut vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm.renderer_bindings
        .bootstrap_render_context(&mut factory)
        .unwrap();
    vm.renderer_bindings.install(vm.lua()).unwrap();
    let file = import_fixture("coin.riv", &mut factory);
    let source = file.with_file(|file| file.artboard_handle(0)).unwrap();
    let main_state = vm.lua().current_thread().state();
    let constructed_on_host_thread = Rc::new(std::cell::Cell::new(false));
    let constructed = constructed_on_host_thread.clone();
    let callback_file = file.clone();
    let bindings = vm.renderer_bindings.clone();
    let make_source = vm
        .lua()
        .create_function(move |lua, ()| {
            let state = lua.current_thread().state();
            assert_ne!(state, main_state);
            // SAFETY: the callback's Lua handle retains this current host thread.
            assert!(
                unsafe { luaur_vm::functions::lua_getthreaddata::lua_getthreaddata(state) }
                    .is_null()
            );
            constructed.set(true);
            let native = native_script_artboard(
                callback_file.clone(),
                nuxie_runtime::Artboard::instance_from_handle(&source).unwrap(),
                None,
                None,
            )
            .unwrap();
            lua.create_userdata(ScriptedArtboard::new(native, bindings.clone()))
        })
        .unwrap();
    vm.lua()
        .globals()
        .set("makeHostSource", make_source)
        .unwrap();
    let body: luaur_rt::Function = vm.lua().load(
        "function cloneOnThread(source) return source:instance() end\nreturn function() return cloneOnThread(makeHostSource()) end",
    ).eval().unwrap();
    // This host API calls lua_newthread directly. It does not require the
    // coroutine library omitted by upstream's public sandbox.
    let thread = vm.lua().create_thread(body).unwrap();
    unsafe {
        luaur_vm::functions::lua_setthreaddata::lua_setthreaddata(
            thread.state(),
            std::ptr::null_mut(),
        );
    }
    let cloned: AnyUserData = thread.resume(()).expect("host-thread artboard clone");
    assert!(constructed_on_host_thread.get());
    assert!(cloned.is::<ScriptedArtboard>());
    vm.lua()
        .globals()
        .set("makeHostSource", Value::Nil)
        .unwrap();

    // Rust has no native-REV ScriptingContext virtual host. Its live context
    // ownership is split across the VM's frame context and RendererBindings,
    // also published in Lua app data. Replace those exact roots, including the
    // bindings retained by the old artboard userdata, rather than unrelated
    // app data. Old userdata retains its scoped Rust owner; no retired native
    // host release callback is introduced by this approved boundary.
    let replacement = ScriptViewModelFrameContext::default();
    let replacement_bindings = RendererBindings::new(replacement.clone());
    replacement_bindings
        .bootstrap_render_context(&mut factory)
        .unwrap();
    replacement_bindings.install(vm.lua()).unwrap();
    vm.lua().set_app_data(replacement.clone());
    vm.renderer_bindings = replacement_bindings;
    vm.view_model_frame_context = replacement;

    drop(cloned);
    // Explicitly clear the host thread before releasing its registry handle,
    // matching the upstream lua_settop/thread-pop/GC sequence.
    unsafe { luaur_vm::functions::lua_settop::lua_settop(thread.state(), 0).unwrap() };
    drop(thread);
    vm.lua().gc_collect().unwrap();
    let weak_vm = vm.lua().weak();
    drop(vm);
    assert!(
        weak_vm.try_upgrade().is_none(),
        "VM closes while File is alive"
    );
    assert!(file.with_file(|file| file.artboard_handle(0)).is_some());
}

#[test]
fn failed_host_artboard_hydration_does_not_publish_an_input() {
    use nuxie_runtime::{PreparedScriptArtboard, ScriptCoreString, ScriptError, ScriptInstance};

    #[derive(Debug)]
    struct FailedInstantiation;
    impl PreparedScriptArtboard for FailedInstantiation {
        fn construct(self: Box<Self>) -> std::result::Result<Box<dyn ScriptArtboard>, ScriptError> {
            Err(ScriptError::new("Could not instantiate artboard"))
        }
    }

    let lua = Lua::new();
    let table = lua.create_table();
    let mut instance = crate::vm::LuaScriptInstance::new(table.clone());
    instance.generator = Some(lua.load("return function() return {} end").eval().unwrap());
    // Host hydration reports a typed error instead of long-jumping through an
    // unprotected Lua call; the source's leave-input-unset contract is exact.
    let error = instance
        .set_prepared_artboard_input_core(
            &ScriptCoreString::from("panel"),
            Box::new(FailedInstantiation),
        )
        .unwrap_err();
    assert_eq!(error.to_string(), "Could not instantiate artboard");
    assert!(matches!(table.get::<Value>("panel").unwrap(), Value::Nil));
}
