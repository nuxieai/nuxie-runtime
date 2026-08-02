use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{NoopScriptHost, ScriptInterpolatorMethod, ScriptOptionalNumberResult};
use nuxie_scripting::vm::ScriptVm;

fn compile_protocol(source: &str) -> Vec<u8> {
    use luaur_compiler::functions::luau_compile::luau_compile;

    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null(), "pinned Luau compiler returned null");
    // SAFETY: luau_compile returns a malloc allocation of output_size bytes.
    let bytecode: Vec<u8> =
        unsafe { std::slice::from_raw_parts(output.cast::<u8>(), output_size) }.to_vec();
    unsafe extern "C" {
        fn free(pointer: *mut std::ffi::c_void);
    }
    // SAFETY: output is the allocation returned by luau_compile above.
    unsafe { free(output.cast()) };
    let mut payload = vec![0];
    payload.extend(bytecode);
    payload
}

#[test]
fn registered_lua_protocol_drives_both_interpolator_callbacks() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let program = vm
        .register_protocol_script_with_factory(
            "scripted-interpolator",
            &compile_protocol(
                r#"
                return function(_context)
                    return {
                        transform = function(_self, factor)
                            return " 0.25 "
                        end,
                        transformValue = function(_self, from, to, factor)
                            return from + (to - from) * factor * factor
                        end,
                    }
                end
            "#,
            ),
            &mut factory,
        )
        .expect("protocol registers");
    let mut instance = vm
        .instantiate_registered_script_with_context(&program, None, Vec::new())
        .expect("protocol instantiates");
    let mut host = NoopScriptHost;

    assert_eq!(
        instance
            .call_interpolator(ScriptInterpolatorMethod::Transform, &[0.5], &mut host,)
            .expect("transform callback succeeds"),
        ScriptOptionalNumberResult::Returned(0.25)
    );
    assert_eq!(
        instance
            .call_interpolator(
                ScriptInterpolatorMethod::TransformValue,
                &[10.0, 30.0, 0.5],
                &mut host,
            )
            .expect("transformValue callback succeeds"),
        ScriptOptionalNumberResult::Returned(15.0)
    );
}
