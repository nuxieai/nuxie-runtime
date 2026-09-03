use crate::mechanical_port::source::{
    assets::{blob_asset::BlobAsset, image_asset::ImageAsset},
    lua::{lua_image_decode::context_decode_image_impl, rive_lua_libs::*},
    scripted::scripted_object::ScriptedObject,
};
pub fn push_gpu_features(state: &mut LuaState) -> i32 {
    if let Some(context) = state.thread_data::<dyn ScriptingContext>().ore_context() {
        let f = context.features();
        state.create_table(0, 19);
        for (name, value) in [
            ("bc", f.bc),
            ("etc2", f.etc2),
            ("astc", f.astc),
            ("anisotropicFiltering", f.anisotropic_filtering),
            ("texture3D", f.texture_3d),
            ("textureArrays", f.texture_arrays),
            ("colorBufferFloat", f.color_buffer_float),
            ("colorBufferHalfFloat", f.color_buffer_half_float),
            ("perTargetBlend", f.per_target_blend),
            ("perTargetWriteMask", f.per_target_write_mask),
            ("drawBaseInstance", f.draw_base_instance),
            ("depthBiasClamp", f.depth_bias_clamp),
        ] {
            state.push_boolean(value);
            state.set_field(-2, name);
        }
        for (name, value) in [
            ("maxTextureSize2D", f.max_texture_size_2d),
            ("maxTextureSizeCube", f.max_texture_size_cube),
            ("maxTextureSize3D", f.max_texture_size_3d),
            ("maxColorAttachments", f.max_color_attachments),
            ("maxUniformBufferSize", f.max_uniform_buffer_size),
            ("maxSamplers", f.max_samplers),
            ("maxSamples", f.max_samples),
        ] {
            state.push_number(value as f64);
            state.set_field(-2, name);
        }
        state.set_readonly(-1, true);
        return 1;
    }
    state.create_table(0, 19);
    for name in [
        "bc",
        "etc2",
        "astc",
        "anisotropicFiltering",
        "texture3D",
        "textureArrays",
        "colorBufferFloat",
        "colorBufferHalfFloat",
        "perTargetBlend",
        "perTargetWriteMask",
        "drawBaseInstance",
        "depthBiasClamp",
    ] {
        state.push_boolean(false);
        state.set_field(-2, name);
    }
    for (name, value) in [
        ("maxTextureSize2D", 4096),
        ("maxTextureSizeCube", 4096),
        ("maxTextureSize3D", 256),
        ("maxColorAttachments", 4),
        ("maxUniformBufferSize", 16384),
        ("maxSamplers", 16),
        ("maxSamples", 4),
    ] {
        state.push_number(value as f64);
        state.set_field(-2, name);
    }
    state.set_readonly(-1, true);
    1
}
impl ScriptedContext {
    pub fn new(object: CoreHandle) -> Self {
        Self {
            scripted_object: Some(object),
            missing_requested_data: false,
        }
    }

    fn current_data_context(&self) -> Option<RuntimeDataContextHandle> {
        self.scripted_object
            .as_ref()
            .and_then(ScriptedObject::effective_data_context)
    }

    fn current_file(&self) -> Option<RuntimeFileWeakHandle> {
        self.scripted_object.as_ref().and_then(|object| {
            object
                .with(|object| object.scripted_object_file())
                .flatten()
        })
    }

    pub fn push_viewmodel(&mut self, state: &mut LuaState) -> i32 {
        if let Some(instance) = self
            .current_data_context()
            .and_then(|context| context.with_context(DataContext::main_view_model_instance))
        {
            let model = instance
                .with(|instance| instance.as_view_model_instance()?.get_view_model())
                .flatten();
            state.new_rive(ScriptedViewModel::new(state, model, Some(instance)));
            return 1;
        }
        self.missing_requested_data = true;
        0
    }
    pub fn push_root_viewmodel(&mut self, state: &mut LuaState) -> i32 {
        if let Some(instance) = self
            .current_data_context()
            .and_then(|context| context.with_context(DataContext::root_view_model_instance))
        {
            let model = instance
                .with(|instance| instance.as_view_model_instance()?.get_view_model())
                .flatten();
            state.new_rive(ScriptedViewModel::new(state, model, Some(instance)));
            return 1;
        }
        self.missing_requested_data = true;
        0
    }
    pub fn push_global_viewmodel(&mut self, state: &mut LuaState, name: &[u8]) -> i32 {
        let context = self.current_data_context();
        let file = self.current_file().and_then(|file| file.upgrade());
        if let Some(instance) = context.zip(file).and_then(|(context, file)| {
            crate::scripting::resolve_global_view_model_instance(&context, &file, name)
        }) {
            let model = instance
                .with(|instance| instance.as_view_model_instance()?.get_view_model())
                .flatten();
            state.new_rive(ScriptedViewModel::new(state, model, Some(instance)));
            return 1;
        }
        0
    }
    pub fn push_global_viewmodel_names(&mut self, state: &mut LuaState) -> i32 {
        let names = self
            .current_file()
            .and_then(|file| file.with_file(|file| file.global_view_model_names()))
            .unwrap_or_default();
        state.create_table(names.len() as i32, 0);
        for (index, name) in names.iter().enumerate() {
            let name = name.split('\0').next().unwrap_or_default();
            state.push_string(name);
            state.raw_set_i(-2, index as i32 + 1);
        }
        1
    }
    pub fn push_data_context(&mut self, state: &mut LuaState) -> i32 {
        if let Some(context) = self.current_data_context() {
            state.new_rive(ScriptedDataContext::new(
                ScriptedDataContextHandle::Runtime(context),
            ));
            return 1;
        }
        self.missing_requested_data = true;
        0
    }
}
fn descriptor_size(s: &mut LuaState) -> (u32, u32) {
    if s.is_none_or_nil(2) {
        return (0, 0);
    }
    s.check_type(2, LuaType::Table);
    s.get_field(2, "width");
    let w = if s.is_nil(-1) {
        0
    } else {
        s.check_number(-1) as u32
    };
    s.pop(1);
    s.get_field(2, "height");
    let h = if s.is_nil(-1) {
        0
    } else {
        s.check_number(-1) as u32
    };
    s.pop(1);
    (w, h)
}
fn context_namecall(s: &mut LuaState) -> i32 {
    let (name, atom) = s.namecall_atom();
    let name = name.unwrap_or_default();
    let context = s.to_rive_mut::<ScriptedContext>(1);
    let Some(object) = context.scripted_object.clone() else {
        return s.error(format!("context:{name}() called on a disposed context — the context passed to init() must not be used after init() returns"));
    };
    let scripted_object_file = || {
        object
            .with(|object| object.scripted_object_file())
            .flatten()
    };
    match atom {
        LuaAtoms::MarkNeedsUpdate => {
            object.with_mut(|object| {
                if let Some(object) = object.as_scripted_object_mut() {
                    object.mark_needs_update();
                }
            });
            0
        }
        LuaAtoms::ViewModel => context.push_viewmodel(s),
        LuaAtoms::RootViewModel => context.push_root_viewmodel(s),
        LuaAtoms::GlobalViewModel => {
            let wanted = s.check_string(2);
            context.push_global_viewmodel(s, wanted.as_bytes())
        }
        LuaAtoms::GlobalViewModelNames => context.push_global_viewmodel_names(s),
        LuaAtoms::DataContext => context.push_data_context(s),
        LuaAtoms::Image => {
            let wanted = s.check_string(2);
            let file = scripted_object_file().and_then(|file| file.upgrade());
            if let Some(image) = file.as_ref().and_then(|file| {
                file.with_file(|file| {
                    file.assets().iter().find_map(|asset| {
                        asset
                            .with(|asset| {
                                let image = asset.as_image_asset()?;
                                (image.base.name() == wanted)
                                    .then(|| image.render_image().cloned())
                                    .flatten()
                            })
                            .flatten()
                    })
                })
            }) {
                s.new_rive(ScriptedImage { image: Some(image) });
                return 1;
            }
            0
        }
        LuaAtoms::Blob => {
            let wanted = s.check_string(2);
            let reference = ScopedAssetReference::new(s, &wanted);
            let file = scripted_object_file().and_then(|file| file.upgrade());
            if let Some(blob) = file.as_ref().and_then(|file| {
                file.with_file(|file| {
                    file.assets()
                        .iter()
                        .filter_map(|asset_handle| {
                            asset_handle
                                .with(|asset| {
                                    let blob = asset.as_blob_asset()?;
                                    (!blob.bytes().is_empty()).then(|| {
                                        (
                                            reference
                                                .match_name(blob.base.name(), blob.base.name()),
                                            asset_handle.clone(),
                                        )
                                    })
                                })
                                .flatten()
                        })
                        .max_by_key(|(rank, _)| *rank)
                        .and_then(|(_, asset)| asset)
                })
            }) {
                s.new_rive(ScriptedBlob { asset: Some(blob) });
                return 1;
            }
            0
        }
        LuaAtoms::Audio => {
            let wanted = s.check_string(2);
            let file = scripted_object_file().and_then(|file| file.upgrade());
            if let Some(source) = file.as_ref().and_then(|file| {
                file.with_file(|file| {
                    file.assets().iter().find_map(|asset| {
                        asset
                            .with(|asset| {
                                let audio = asset.as_audio_asset()?;
                                (audio.base.name() == wanted)
                                    .then(|| audio.audio_source())
                                    .flatten()
                            })
                            .flatten()
                    })
                })
            }) {
                let mut scripted = ScriptedAudioSource::default();
                scripted.set_source(source);
                s.new_rive(scripted);
                return 1;
            }
            0
        }
        LuaAtoms::Canvas => {
            let (w, h) = descriptor_size(s);
            {
                let Some(render_context) = s.thread_data::<dyn ScriptingContext>().render_context()
                else {
                    return s.error(
                        "context:canvas() requires a RenderContext — call setRenderContext() first",
                    );
                };
                let mut handle = ScriptedCanvas::new(s, render_context);
                if w == 0 || h == 0 {
                    s.new_rive(handle);
                    return 1;
                }
                let Some(canvas) = render_context.make_render_canvas(w, h) else {
                    return s.error("context:canvas() failed to create RenderCanvas");
                };
                let image = ScriptedImage {
                    image: Some(canvas.render_image().clone()),
                    source_canvas: Some(canvas.clone()),
                    ..Default::default()
                };
                let image_ref = s.new_rive_ref(image);
                handle.canvas = Some(canvas);
                handle.image_ref = image_ref;
                s.new_rive(handle);
                1
            }
        }
        LuaAtoms::GpuCanvas => {
            let (w, h) = descriptor_size(s);
            {
                let scripting = s.thread_data::<dyn ScriptingContext>();
                if scripting.gpu_canvas_defer_only() {
                    s.new_rive(ScriptedGPUCanvas::deferred(s));
                    return 1;
                }
                let Some(render) = scripting.render_context() else {
                    return s.error("context:gpuCanvas() requires a RenderContext — call setRenderContext() first");
                };
                let Some(ore) = scripting.ore_context() else {
                    return s.error("context:gpuCanvas() requires a GPU context — call scriptingWorkspaceSetOreContext() before requestVM()");
                };
                let mut handle = ScriptedGPUCanvas::new(s, render);
                if w == 0 || h == 0 {
                    s.new_rive(handle);
                    return 1;
                }
                let Some(canvas) = render.make_render_canvas(w, h) else {
                    return s.error("context:gpuCanvas() failed to create RenderCanvas");
                };
                let Some(view) = ore.wrap_canvas_texture(&canvas) else {
                    return s.error("context:gpuCanvas() failed to wrap canvas texture");
                };
                handle.image_ref = s.new_rive_ref(ScriptedImage {
                    image: Some(canvas.render_image().clone()),
                    source_canvas: Some(canvas.clone()),
                    ..Default::default()
                });
                handle.canvas = Some(canvas);
                handle.ore_color_view = Some(view);
                s.new_rive(handle);
                1
            }
        }
        LuaAtoms::Features => push_gpu_features(s),
        LuaAtoms::Shader => {
            let wanted = s.check_string(2);
            let reference = ScopedAssetReference::new(s, &wanted);
            let file_asset = lua_gpu_find_shader_asset(scripted_object_file(), &reference);
            let mut scripted = ScriptedShader::default();
            if lua_gpu_load_shader_by_name(
                &mut scripted,
                s.thread_data::<dyn ScriptingContext>(),
                &reference,
                file_asset,
            ) {
                s.new_rive(scripted);
                return 1;
            }
            return 0;
        }
        LuaAtoms::DecodeImage => context_decode_image_impl(s),
        _ => s.error(format!(
            "{name} is not a valid method of {}",
            ScriptedContext::LUA_NAME
        )),
    }
}
pub fn luaopen_rive_contex(s: &mut LuaState) -> i32 {
    s.register_rive::<ScriptedContext>();
    s.push_function(context_namecall);
    s.set_field(-2, "__namecall");
    s.set_readonly(-1, true);
    s.pop(1);
    0
}
