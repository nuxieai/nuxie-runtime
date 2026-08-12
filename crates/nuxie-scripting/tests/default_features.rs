use nuxie_scripting::vm::ScriptVm;

#[test]
fn default_build_preserves_luaur_async_api() {
    let vm = ScriptVm::new();
    let function = vm
        .lua()
        .create_async_function(|_, value: i64| async move { Ok(value + 1) })
        .expect("default scripting builds expose luaur's async bindings");

    drop(function);
}
