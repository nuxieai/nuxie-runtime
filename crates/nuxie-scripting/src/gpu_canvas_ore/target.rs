//! Stateless GPUTarget userdata, read live from the script's ORE context.
use super::*;
use luaur_rt::FromLua;

pub(super) struct Target;

pub(crate) fn target_userdata(lua: &Lua) -> Result<Option<AnyUserData>> {
    let Some(context) = RendererBindings::for_lua(lua).and_then(|b| b.ore_context()) else {
        return Ok(None);
    };
    if !context.borrow().isRecording() {
        return Ok(None);
    }
    const KEY: &str = "rive.gpuTarget";
    if let Some(target) = lua.named_registry_value::<Option<AnyUserData>>(KEY)? {
        return Ok(Some(target));
    }
    let target = lua.create_userdata(Target)?;
    lua.set_named_registry_value(KEY, target.clone())?;
    Ok(Some(target))
}

fn view(lua: &Lua) -> Option<AnyResourceHandle> {
    RendererBindings::for_lua(lua)
        .and_then(|b| b.ore_context())
        .and_then(|context| context.borrow_mut().targetView())
}

impl UserData for Target {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |lua, _| {
            Ok(view(lua)
                .and_then(|v| v.textureViewBase().unwrap().texture().width())
                .unwrap_or(0))
        });
        fields.add_field_method_get("height", |lua, _| {
            Ok(view(lua)
                .and_then(|v| v.textureViewBase().unwrap().texture().height())
                .unwrap_or(0))
        });
        fields.add_field_method_get("format", |lua, _| {
            Ok(format_string(
                view(lua)
                    .and_then(|v| v.textureViewBase().unwrap().texture().format())
                    .unwrap_or(TextureFormat::rgba8unorm),
            ))
        });
        fields.add_field_method_get("sampleCount", |lua, _| {
            Ok(view(lua)
                .and_then(|v| v.textureViewBase().unwrap().texture().sampleCount())
                .unwrap_or(1))
        });
        fields.add_field_method_get("view", |lua, _| {
            view(lua)
                .map(|resource| {
                    lua.create_userdata(TextureView {
                        resource,
                        retained_image: None,
                    })
                })
                .transpose()
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("beginRenderPass", |lua, _, descriptor: Value| {
            let context = super::canvas::check_begin_pass(lua, "GPUTarget")?;
            let table = Table::from_lua(descriptor, lua)?;
            let target = context.borrow_mut().targetView();
            // luaur propagates errors as Result: the owned view drops on every
            // error path, including descriptor callbacks and stale-view errors.
            super::canvas::begin_pass(lua, "GPUTarget", target.as_ref(), true, &table)
        });
    }
}
