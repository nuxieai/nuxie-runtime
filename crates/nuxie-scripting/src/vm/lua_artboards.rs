// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/lua_artboards.cpp
use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{
    AnyUserData, Error, Lua, Result, Table, UserData, UserDataFields, UserDataMethods, Value,
    Vector as LuaVector,
};
use nuxie_runtime::mechanical_port::source::{
    core::CoreHandle,
    custom_property::{CustomProperty, CustomPropertyKind},
    drawable::Drawable,
    generated::core_registry::CoreRegistry,
};
use nuxie_runtime::{
    ScriptAnimation, ScriptAnimationTime, ScriptArtboard, ScriptMethod, ScriptNode,
};

use super::lua_mat2d::ScriptedMat2D;
use super::lua_paint::ScriptedPaintData;
use super::lua_path::{ScriptedPath, create_scripted_path};
use super::lua_renderer::ScriptedRenderer;
use super::lua_renderer_library::RendererBindings;
use super::view_model::{create_scripted_view_model, model_from_table};

impl RendererBindings {
    pub(crate) fn create_scripted_artboard(
        &self,
        lua: &Lua,
        artboard: Box<dyn ScriptArtboard>,
    ) -> Result<AnyUserData> {
        lua.create_userdata(ScriptedArtboard::new(artboard, self.clone()))
    }
}

struct ScriptedArtboardOwner {
    artboard: Box<dyn ScriptArtboard>,
}

struct ScriptedArtboard {
    owner: Rc<ScriptedArtboardOwner>,
    bindings: RendererBindings,
    data: RefCell<Option<Value>>,
}

struct ScriptedAnimation {
    owner: Rc<ScriptedArtboardOwner>,
    animation: ScriptAnimation,
}

impl UserData for ScriptedAnimation {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("duration", |_, this| Ok(this.animation.duration()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("advance", |_, this, seconds: f32| {
            let mut animation = this.animation.clone();
            this.owner
                .artboard
                .retained_handle()
                .advance_animation(&mut animation, seconds)
                .map_err(|error| Error::runtime(error.to_string()))
        });
        for (name, mode) in [
            ("setTime", ScriptAnimationTime::Seconds),
            ("setTimeFrames", ScriptAnimationTime::Frames),
            ("setTimePercentage", ScriptAnimationTime::Percentage),
        ] {
            methods.add_method(name, move |_, this, value: f32| {
                let mut animation = this.animation.clone();
                this.owner
                    .artboard
                    .retained_handle()
                    .set_animation_time(&mut animation, value, mode)
                    .map_err(|error| Error::runtime(error.to_string()))
            });
        }
    }
}

impl ScriptedArtboard {
    fn new(artboard: Box<dyn ScriptArtboard>, bindings: RendererBindings) -> Self {
        Self {
            owner: Rc::new(ScriptedArtboardOwner { artboard }),
            bindings,
            data: RefCell::new(None),
        }
    }
}

impl UserData for ScriptedArtboard {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.owner.artboard.width()));
        fields.add_field_method_set("width", |_, this, value: f32| {
            this.owner.artboard.retained_handle().set_width(value);
            Ok(())
        });
        fields.add_field_method_get("height", |_, this| Ok(this.owner.artboard.height()));
        fields.add_field_method_set("height", |_, this, value: f32| {
            this.owner.artboard.retained_handle().set_height(value);
            Ok(())
        });
        fields.add_field_method_get("frameOrigin", |_, this| {
            Ok(this.owner.artboard.frame_origin())
        });
        fields.add_field_method_set("frameOrigin", |_, this, value: bool| {
            this.owner
                .artboard
                .retained_handle()
                .set_frame_origin(value);
            Ok(())
        });
        fields.add_field_method_get("data", |lua, this| {
            if let Some(data) = this.data.borrow().as_ref() {
                return Ok(data.clone());
            }
            let data = match this.owner.artboard.data() {
                Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                None => Value::Nil,
            };
            *this.data.borrow_mut() = Some(data.clone());
            Ok(data)
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("bounds", |_, this, ()| {
            let bounds = this.owner.artboard.bounds();
            Ok((
                LuaVector::new(bounds.min_x, bounds.min_y, 0.0),
                LuaVector::new(bounds.max_x, bounds.max_y, 0.0),
            ))
        });
        methods.add_method(
            "addToPath",
            |_, this, (path, transform): (AnyUserData, Option<AnyUserData>)| {
                let transform = transform
                    .as_ref()
                    .map(|transform| transform.borrow::<ScriptedMat2D>().map(|value| value.0))
                    .transpose()?;
                let mut path = path.borrow_mut::<ScriptedPath>()?;
                path.with_render_raw_path_mut(|raw_path| {
                    this.owner
                        .artboard
                        .retained_handle()
                        .add_to_path(raw_path, transform)
                })
                .map_err(|error| Error::runtime(error.to_string()))?;
                path.mark_dirty();
                Ok(())
            },
        );
        for method in [
            ScriptMethod::PointerDown,
            ScriptMethod::PointerMove,
            ScriptMethod::PointerUp,
            ScriptMethod::PointerExit,
            ScriptMethod::GamepadConnected,
            ScriptMethod::GamepadEvent,
            ScriptMethod::GamepadDisconnected,
        ] {
            methods.add_method(method.as_str(), move |_, this, event: AnyUserData| {
                let invocation =
                    super::listener_invocation::artboard_input_invocation(method, &event)?;
                this.owner
                    .artboard
                    .retained_handle()
                    .dispatch_input(method, &invocation)
                    .map_err(|error| Error::runtime(error.to_string()))
            });
        }
        methods.add_method("pointerScroll", |_, this, event: AnyUserData| {
            let event = event.borrow::<super::lua_input::ScriptedScrollEvent>()?;
            this.owner
                .artboard
                .retained_handle()
                .dispatch_scroll(
                    event.id,
                    event.position.x,
                    event.position.y,
                    event.event,
                    event.timestamp,
                )
                .map_err(|error| Error::runtime(error.to_string()))
        });
        methods.add_method("instance", |lua, this, view_model: Option<Table>| {
            let view_model = view_model.as_ref().map(model_from_table).transpose()?;
            let instance = this.bindings.with_factory(|factory| {
                this.owner
                    .artboard
                    .instance_with_factory(view_model, factory)
                    .map_err(|error| Error::runtime(error.to_string()))
            })?;
            lua.create_userdata(ScriptedArtboard::new(instance, this.bindings.clone()))
        });
        methods.add_method("advance", |_, this, seconds: f32| {
            this.owner
                .artboard
                .retained_handle()
                .advance(seconds)
                .map_err(|error| Error::runtime(error.to_string()))
        });
        methods.add_method("animation", |lua, this, name: String| {
            let animation = this
                .owner
                .artboard
                .animation(&name)
                .map_err(|error| Error::runtime(error.to_string()))?;
            Ok(match animation {
                Some(animation) => Value::UserData(lua.create_userdata(ScriptedAnimation {
                    owner: Rc::clone(&this.owner),
                    animation,
                })?),
                None => Value::Nil,
            })
        });
        methods.add_method("node", |lua, this, name: String| {
            let node = this
                .owner
                .artboard
                .node(&name)
                .map_err(|error| Error::runtime(error.to_string()))?;
            Ok(match node {
                Some(node) => Value::UserData(lua.create_userdata(ScriptedNode {
                    node,
                    owner: Some(this.owner.clone()),
                })?),
                None => Value::Nil,
            })
        });
        methods.add_method("propertyKey", |_, this, name: luaur_rt::LuaString| {
            Ok(this.owner.artboard.property_key(&name.as_bytes()))
        });
        methods.add_method(
            "drawModulated",
            |_, this, (renderer, key): (AnyUserData, u32)| {
                let scripted_renderer = renderer.borrow::<ScriptedRenderer>()?;
                scripted_renderer.bindings.with_factory(|factory| {
                    scripted_renderer.with_renderer_mut(|renderer| {
                        this.owner
                            .artboard
                            .retained_handle()
                            .draw_modulated(factory, renderer, key)
                            .map_err(|error| Error::runtime(error.to_string()))
                    })
                })
            },
        );
        methods.add_method("draw", |lua, this, (renderer, visitor, context): (AnyUserData, Option<luaur_rt::Function>, Value)| {
            let scripted_renderer = renderer.borrow::<ScriptedRenderer>()?;
            if let Some(visitor) = visitor {
                let current = Rc::new(RefCell::new(None));
                let visited = lua.create_userdata(VisitedDrawable { current: current.clone() })?;
                let failure = Rc::new(RefCell::new(None));
                let draw_visitor: nuxie_runtime::mechanical_port::source::artboard::RuntimeDrawVisitor = {
                    let current = current.clone();
                    let failure = failure.clone();
                    let renderer_userdata = renderer.clone();
                    Rc::new(move |drawable, renderer| {
                        if failure.borrow().is_some() {
                            Drawable::draw_handle(drawable, renderer);
                            return;
                        }
                        let previous = current.replace(Some(drawable.clone()));
                        let result = (|| {
                            let scripted_renderer = renderer_userdata.borrow::<ScriptedRenderer>()?;
                            scripted_renderer.with_reborrowed_renderer(renderer, || {
                                let saves = scripted_renderer.save_count();
                                let result = visitor.call::<()>((context.clone(), visited.clone(), renderer_userdata.clone()));
                                if result.is_err() {
                                    scripted_renderer.restore_to(saves)?;
                                }
                                result
                            })
                        })();
                        current.replace(previous);
                        if let Err(error) = result {
                            *failure.borrow_mut() = Some(error);
                        }
                    })
                };
                let result = scripted_renderer.bindings.with_factory(|factory| {
                    scripted_renderer.with_renderer_mut(|renderer| {
                        this.owner.artboard.retained_handle().draw_with_visitor(factory, renderer, draw_visitor)
                            .map_err(|error| Error::runtime(error.to_string()))
                    })
                });
                current.replace(None);
                if let Some(error) = failure.take() {
                    return Err(error);
                }
                return result;
            }
            scripted_renderer.bindings.with_factory(|factory| {
                scripted_renderer.with_renderer_mut(|renderer| {
                    this.owner
                        .artboard
                        .retained_handle()
                        .draw(factory, renderer)
                        .map_err(|error| Error::runtime(error.to_string()))
                })
            })
        });
    }
}

struct VisitedDrawable {
    current: Rc<RefCell<Option<CoreHandle>>>,
}

impl VisitedDrawable {
    fn drawable(&self) -> Result<CoreHandle> {
        self.current
            .borrow()
            .clone()
            .ok_or_else(|| Error::runtime("Drawable is only valid inside the draw visitor"))
    }
}

impl UserData for VisitedDrawable {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("number", |_, this, key: u32| {
            Ok(Drawable::custom_property_handle(&this.drawable()?, key)
                .filter(|property| {
                    CustomProperty::kind_handle(property) == CustomPropertyKind::Number
                })
                .and_then(|property| CoreRegistry::get_double_handle(&property, 243)))
        });
        methods.add_method("boolean", |_, this, key: u32| {
            Ok(Drawable::custom_property_handle(&this.drawable()?, key)
                .filter(|property| {
                    CustomProperty::kind_handle(property) == CustomPropertyKind::Boolean
                })
                .and_then(|property| CoreRegistry::get_bool_handle(&property, 245)))
        });
        methods.add_method("color", |_, this, key: u32| {
            Ok(Drawable::custom_property_handle(&this.drawable()?, key)
                .filter(|property| {
                    CustomProperty::kind_handle(property) == CustomPropertyKind::Color
                })
                .and_then(|property| CoreRegistry::get_color_handle(&property, 836))
                .map(|color| color as u32))
        });
        methods.add_method("string", |_, this, key: u32| {
            Ok(Drawable::custom_property_handle(&this.drawable()?, key)
                .filter(|property| {
                    CustomProperty::kind_handle(property) == CustomPropertyKind::String
                })
                .and_then(|property| CoreRegistry::get_string_handle(&property, 246)))
        });
        methods.add_method("draw", |_, this, renderer: AnyUserData| {
            let drawable = this.drawable()?;
            renderer
                .borrow::<ScriptedRenderer>()?
                .with_renderer_mut(|renderer| {
                    Drawable::draw_handle(&drawable, renderer);
                    Ok(())
                })
        });
        #[cfg(feature = "tools")]
        methods.add_method("properties", |lua, this, ()| {
            use nuxie_runtime::mechanical_port::source::{
                artboard::Artboard,
                assets::manifest_asset::ManifestAsset,
                custom_property::{CustomProperty, CustomPropertyKind},
                generated::custom_property_base::CustomPropertyBase,
            };
            let drawable = this.drawable()?;
            let result = lua.create_table();
            let file = drawable
                .with(|object| {
                    object
                        .as_component()
                        .and_then(|component| component.artboard_handle())
                })
                .flatten()
                .and_then(|artboard| Artboard::draw_visitor_file_handle(&artboard));
            let manifest = file.and_then(|file| file.with_file(|file| file.manifest()));
            if let Some(manifest) = manifest {
                for property in Drawable::tagging_properties_handle(&drawable) {
                    let id = CoreRegistry::get_uint_handle(
                        &property,
                        CustomPropertyBase::NAME_ID_PROPERTY_KEY.into(),
                    )
                    .unwrap();
                    let name = manifest
                        .with_downcast::<ManifestAsset, _>(|manifest| {
                            manifest.resolve_name(id as i32).to_owned()
                        })
                        .unwrap();
                    let kind = match CustomProperty::kind_handle(&property) {
                        CustomPropertyKind::Number => "number",
                        CustomPropertyKind::Boolean => "boolean",
                        CustomPropertyKind::String => "string",
                        CustomPropertyKind::Color => "color",
                        CustomPropertyKind::Enumeration => "enum",
                        CustomPropertyKind::Trigger => "trigger",
                    };
                    let entry = lua.create_table();
                    entry.set("name", name)?;
                    entry.set("kind", kind)?;
                    result.raw_push(entry)?;
                }
            }
            Ok(result)
        });
    }
}

pub(super) struct ScriptedNode {
    node: ScriptNode,
    owner: Option<Rc<ScriptedArtboardOwner>>,
}

impl ScriptedNode {
    pub(super) fn new(node: ScriptNode) -> Self {
        Self { node, owner: None }
    }
}

impl UserData for ScriptedNode {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.node.x()));
        fields.add_field_method_set("x", |_, this, value: f32| {
            this.node.set_x(value);
            Ok(())
        });
        fields.add_field_method_get("y", |_, this| Ok(this.node.y()));
        fields.add_field_method_set("y", |_, this, value: f32| {
            this.node.set_y(value);
            Ok(())
        });
        fields.add_field_method_get("position", |_, this| {
            Ok(LuaVector::new(this.node.x(), this.node.y(), 0.0))
        });
        fields.add_field_method_set("position", |_, this, value: LuaVector| {
            this.node.set_x(value.x());
            this.node.set_y(value.y());
            Ok(())
        });
        fields.add_field_method_get("rotation", |_, this| Ok(this.node.rotation()));
        fields.add_field_method_set("rotation", |_, this, value: f32| {
            this.node.set_rotation(value);
            Ok(())
        });
        fields.add_field_method_get("scale", |_, this| {
            Ok(LuaVector::new(
                this.node.scale_x(),
                this.node.scale_y(),
                0.0,
            ))
        });
        fields.add_field_method_set("scale", |_, this, value: LuaVector| {
            this.node.set_scale_x(value.x());
            this.node.set_scale_y(value.y());
            Ok(())
        });
        fields.add_field_method_get("scaleX", |_, this| Ok(this.node.scale_x()));
        fields.add_field_method_set("scaleX", |_, this, value: f32| {
            this.node.set_scale_x(value);
            Ok(())
        });
        fields.add_field_method_get("scaleY", |_, this| Ok(this.node.scale_y()));
        fields.add_field_method_set("scaleY", |_, this, value: f32| {
            this.node.set_scale_y(value);
            Ok(())
        });
        fields.add_field_method_get("worldTransform", |lua, this| {
            lua.create_userdata(ScriptedMat2D(this.node.world_transform()))
        });
        fields.add_field_method_set("worldTransform", |_, this, value: AnyUserData| {
            this.node
                .set_world_transform(value.borrow::<ScriptedMat2D>()?.0);
            Ok(())
        });
        fields.add_field_method_get("children", |lua, this| {
            let children = this.node.children();
            let table = lua.create_table();
            for (index, child) in children.into_iter().enumerate() {
                table.raw_set(
                    index + 1,
                    lua.create_userdata(Self {
                        node: child,
                        owner: this.owner.clone(),
                    })?,
                )?;
            }
            Ok(table)
        });
        fields.add_field_method_get("parent", |lua, this| {
            Ok(match this.node.parent() {
                Some(parent) => Value::UserData(lua.create_userdata(Self {
                    node: parent,
                    owner: this.owner.clone(),
                })?),
                None => Value::Nil,
            })
        });
        fields.add_field_method_get("paint", |lua, this| {
            Ok(match this.node.paint() {
                Some(paint) => Value::UserData(lua.create_userdata(ScriptedPaintData(paint))?),
                None => Value::Nil,
            })
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("decompose", |_, this, transform: AnyUserData| {
            this.node.decompose(transform.borrow::<ScriptedMat2D>()?.0);
            Ok(())
        });
        methods.add_method("asPath", |lua, this, ()| {
            Ok(match this.node.path() {
                Some(path) => Value::UserData(create_scripted_path(
                    lua,
                    ScriptedPath::from_render_raw_path(path),
                )?),
                None => Value::Nil,
            })
        });
        methods.add_method("asPaint", |lua, this, ()| {
            Ok(match this.node.paint() {
                Some(paint) => Value::UserData(lua.create_userdata(ScriptedPaintData(paint))?),
                None => Value::Nil,
            })
        });
    }
}

#[cfg(all(test, feature = "compiler"))]
mod artboard_owner_tests {
    use super::*;
    use crate::vm::ScriptViewModelFrameContext;
    use crate::vm::ScriptVm;
    use nuxie_render_api::{
        Factory as RenderFactory, PersistentFactory, RecordingFactory, SerializingFactory,
    };
    use nuxie_runtime::{
        File, RuntimeFactoryHandle, RuntimeFileHandle, RuntimeScriptingVmHandle, ScriptViewModel,
        ScriptViewModelProperty, native_script_artboard,
    };

    use nuxie_sriv as sriv;

    fn fixture_bytes(name: &str) -> Vec<u8> {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets")
            .join(name);
        std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()))
    }

    /// Complete port of scripting_draw_visitor_test.cpp's source case.
    #[cfg(feature = "tools")]
    #[test]
    fn artboard_draw_visitor_reads_custom_properties() {
        use nuxie_render_api::*;
        let lua = Lua::new();
        let mut factory = PersistentFactory::new(SerializingFactory::new());
        let file = import_fixture("drawable_custom_properties.riv", &mut factory);
        let source = file.with_file(|file| file.artboard_handle(0)).unwrap();
        let bindings = RendererBindings::new(ScriptViewModelFrameContext::default());
        bindings.bootstrap_render_context(&mut factory).unwrap();
        bindings.install(&lua).unwrap();
        lua.load(
            r#"
visits = 0
emissive = 0
glows = 0
tint = 0
label = ""
missing = 0
kinds = ""
passed = 0
stored = nil
function render(artboard: Artboard, renderer: Renderer)
    artboard:advance(0)
    local EMISSIVE = artboard:propertyKey("emissive")
    local GLOW = artboard:propertyKey("glow")
    local TINT = artboard:propertyKey("tint")
    local LABEL = artboard:propertyKey("label")
    local NOPE = artboard:propertyKey("nope")
    renderer:save()
    renderer:modulateColor(Color.rgba(0, 0, 0, 255))
    artboard:draw(renderer, function(context, drawable, r)
        visits += 1
        passed = context
        stored = drawable
        emissive += drawable:number(EMISSIVE) or 0
        if drawable:boolean(GLOW) then
            glows += 1
            tint = drawable:color(TINT) or 0
            label = drawable:string(LABEL) or ""
            for _, property in drawable:properties() do
                kinds ..= property.name .. ":" .. property.kind .. " "
            end
        end
        if drawable:number(LABEL) == nil and drawable:number(NOPE) == nil then
            missing += 1
        end
        r:setColorModulation(Color.rgba(255, 255, 255, 255))
        drawable:draw(r)
    end, 7)
    renderer:restore()
end
function afterwards() stored:number(0) end
function modulated(artboard: Artboard, renderer: Renderer)
    artboard:advance(0)
    renderer:save()
    renderer:setColorModulation(Color.rgba(0, 0, 0, 255))
    artboard:drawModulated(renderer, artboard:propertyKey("emissive"))
    renderer:restore()
end
function passing(artboard: Artboard, renderer: Renderer)
    artboard:advance(0)
    artboard:draw(renderer, function(context, drawable, r) drawable:draw(r) end)
end
function failingOpen(artboard: Artboard, renderer: Renderer)
    artboard:advance(0)
    artboard:draw(renderer, function(context, drawable, r)
        r:save()
        error("visitor failed with a save open")
    end)
end
function failing(artboard: Artboard, renderer: Renderer)
    artboard:advance(0)
    artboard:draw(renderer, function() error("visitor failed") end)
end
"#,
        )
        .exec()
        .unwrap();
        let call = |renderer: &mut dyn Renderer, name: &str| -> Result<()> {
            let artboard = bindings.create_scripted_artboard(
                &lua,
                native_script_artboard(
                    file.clone(),
                    nuxie_runtime::Artboard::instance_from_handle(&source).unwrap(),
                    None,
                    None,
                )
                .unwrap(),
            )?;
            let (userdata, _scope) =
                ScriptedRenderer::create_call_scoped_userdata(&lua, renderer, bindings.clone())?;
            let result = lua
                .globals()
                .get::<luaur_rt::Function>(name)?
                .call::<()>((artboard, userdata.clone()));
            userdata.borrow::<ScriptedRenderer>()?.end();
            result
        };
        let mut renderer = factory.borrow().make_renderer();
        call(&mut renderer, "render").unwrap();
        for (name, expected) in [
            ("visits", 4.0),
            ("glows", 1.0),
            ("tint", 0xFF8040FFu32 as f64),
            ("missing", 4.0),
            ("passed", 7.0),
        ] {
            assert_eq!(lua.globals().get::<f64>(name).unwrap(), expected);
        }
        assert!((lua.globals().get::<f64>("emissive").unwrap() - 2.3).abs() < 0.00001);
        assert_eq!(lua.globals().get::<String>("label").unwrap(), "ghost");
        assert_eq!(
            lua.globals().get::<String>("kinds").unwrap(),
            "emissive:number glow:boolean tint:color label:string "
        );
        assert!(
            lua.globals()
                .get::<luaur_rt::Function>("afterwards")
                .unwrap()
                .call::<()>(())
                .is_err()
        );
        call(&mut renderer, "modulated").unwrap();
        #[derive(Default)]
        struct DepthRecorder {
            depth: i32,
            deepest_draw: i32,
            draws: i32,
        }
        impl Renderer for DepthRecorder {
            fn save(&mut self) {
                self.depth += 1;
            }
            fn restore(&mut self) {
                self.depth -= 1;
            }
            fn transform(&mut self, _: Mat2D) {}
            fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {
                self.draws += 1;
                self.deepest_draw = self.deepest_draw.max(self.depth);
            }
            fn clip_path(&mut self, _: &dyn RenderPath) {}
            fn draw_image(
                &mut self,
                _: Option<&dyn RenderImage>,
                _: ImageSampler,
                _: BlendMode,
                _: f32,
            ) {
            }
            fn draw_image_mesh(
                &mut self,
                _: Option<&dyn RenderImage>,
                _: ImageSampler,
                _: Option<&dyn RenderBuffer>,
                _: Option<&dyn RenderBuffer>,
                _: Option<&dyn RenderBuffer>,
                _: u32,
                _: u32,
                _: BlendMode,
                _: f32,
            ) {
            }
            fn modulate_opacity(&mut self, _: f32) {}
        }
        let mut passing = DepthRecorder::default();
        call(&mut passing, "passing").unwrap();
        assert!(passing.deepest_draw > 0);
        let mut failed = DepthRecorder::default();
        assert!(call(&mut failed, "failingOpen").is_err());
        assert_eq!(failed.deepest_draw, passing.deepest_draw);
        assert_eq!(failed.draws, passing.draws - 1);
        assert!(
            call(&mut renderer, "failing")
                .unwrap_err()
                .to_string()
                .contains("visitor failed")
        );
    }

    fn import_fixture(name: &str, factory: &mut dyn RenderFactory) -> RuntimeFileHandle {
        File::import(
            &fixture_bytes(name),
            RuntimeFactoryHandle::from_factory(factory).expect("retained factory"),
            None,
            None,
            Some(RuntimeScriptingVmHandle::new(Box::new(ScriptVm::new()))),
        )
        .expect("pinned native fixture import")
    }

    fn trigger_artboard() -> (ScriptViewModel, String, Box<dyn ScriptArtboard>) {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = import_fixture("script_create_viewmodel_instance.riv", &mut factory);
        let (model, trigger) = nuxie_runtime::script_view_models(&file)
            .into_values()
            .find_map(|model| {
                let trigger = model.properties().iter().find_map(|(name, kind)| {
                    (*kind == ScriptViewModelProperty::Trigger).then(|| name.clone())
                })?;
                Some((model.named_instance(None)?, trigger))
            })
            .expect("fixture has a trigger model");
        let source = file.with_file(|file| file.artboard_handle(0)).unwrap();
        let instance = nuxie_runtime::Artboard::instance_from_handle(&source).unwrap();
        let artboard = native_script_artboard(file, instance, model.native_instance(), None)
            .expect("native owner binds the actual trigger instance");
        (model, trigger, artboard)
    }

    #[test]
    fn scripted_child_artboard_advance_does_not_consume_detached_view_models() {
        let (model, trigger, native) = trigger_artboard();
        let context = ScriptViewModelFrameContext::default();
        let artboard = ScriptedArtboard::new(native, RendererBindings::new(context.clone()));
        let lua = Lua::new();
        let userdata = lua
            .create_userdata(artboard)
            .expect("scripted artboard userdata");
        lua.globals()
            .set("child", userdata)
            .expect("publish scripted child");

        assert!(model.fire_trigger(&trigger));
        lua.load("child:advance(0)")
            .exec()
            .expect("child advance succeeds");
        assert_eq!(model.trigger(&trigger), Some(1));

        assert!(model.fire_trigger(&trigger));
        assert_eq!(model.trigger(&trigger), Some(2));
    }

    fn fixture_userdata(lua: &Lua, fixture: &str, artboard_name: Option<&str>) -> AnyUserData {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = import_fixture(fixture, &mut factory);
        let source = file
            .with_file(|file| match artboard_name {
                Some(name) => file.artboard_named_source(name),
                None => file.artboard_handle(0),
            })
            .expect("pinned source artboard");
        let bindings = RendererBindings::new(ScriptViewModelFrameContext::default());
        bindings.bootstrap_render_context(&mut factory).unwrap();
        bindings.install(lua).unwrap();
        lua.create_userdata(ScriptedArtboard::new(
            native_script_artboard(
                file,
                nuxie_runtime::Artboard::instance_from_handle(&source).unwrap(),
                None,
                None,
            )
            .expect("native scripted artboard"),
            bindings,
        ))
        .expect("scripted artboard")
    }

    /// Direct ports of the artboard owner cases in pinned
    /// `scripting_artboard_test.cpp`. The PointerEvent case is retained in
    /// `listener_invocation::tests::pointer_hit_propagates_the_cpp_tristate_out_of_the_lua_callback`.
    #[test]
    fn upstream_can_access_artboard_width_and_height() {
        let lua = Lua::new();
        lua.globals()
            .set("artboard", fixture_userdata(&lua, "coin.riv", None))
            .unwrap();
        let values: Table = lua
            .load(
                r#"
                function accessWidth(artboard) return artboard.width end
                function accessHeight(artboard) return artboard.height end
                function changeWidth(artboard)
                  artboard.width = 24
                  return artboard.width
                end
                function changeHeight(artboard)
                  artboard.height = 22
                  return artboard.height
                end
                return {
                  accessWidth(artboard), accessHeight(artboard),
                  changeWidth(artboard), changeHeight(artboard)
                }
                "#,
            )
            .eval()
            .unwrap();
        assert_eq!(values.get::<f32>(1).unwrap(), 92.0);
        assert_eq!(values.get::<f32>(2).unwrap(), 92.0);
        assert_eq!(values.get::<f32>(3).unwrap(), 24.0);
        assert_eq!(values.get::<f32>(4).unwrap(), 22.0);
    }

    #[test]
    fn upstream_can_access_artboard_bounds() {
        let lua = Lua::new();
        lua.globals()
            .set("artboard", fixture_userdata(&lua, "coin.riv", None))
            .unwrap();
        let values: Table = lua
            .load(
                r#"
                local min, max = artboard:bounds()
                return { min.x, min.y, max.x, max.y }
                "#,
            )
            .eval()
            .expect("pinned bounds method");
        assert_eq!(values.get::<f32>(1).unwrap(), 0.0);
        assert_eq!(values.get::<f32>(2).unwrap(), 0.0);
        assert_eq!(values.get::<f32>(3).unwrap(), 92.0);
        assert_eq!(values.get::<f32>(4).unwrap(), 92.0);
    }

    #[test]
    fn upstream_can_render_an_artboard_via_the_scripting_engine() {
        let lua = Lua::new();
        let mut factory = PersistentFactory::new(SerializingFactory::new());
        let file = import_fixture("coin.riv", &mut factory);
        let source = file.with_file(|file| file.artboard_handle(0)).unwrap();
        let artboard = native_script_artboard(
            file,
            nuxie_runtime::Artboard::instance_from_handle(&source).unwrap(),
            None,
            None,
        )
        .unwrap();
        let (width, height) = (artboard.width(), artboard.height());
        let bindings = RendererBindings::new(ScriptViewModelFrameContext::default());
        bindings.bootstrap_render_context(&mut factory).unwrap();
        bindings.install(&lua).unwrap();
        let userdata = lua
            .create_userdata(ScriptedArtboard::new(artboard, bindings.clone()))
            .unwrap();
        lua.globals().set("artboard", userdata.clone()).unwrap();
        lua.load(
            r#"
            function render(artboard, renderer)
              artboard:advance(0.1)
              artboard:draw(renderer)
              artboard.data.Vertical.value += 5
            end
            "#,
        )
        .exec()
        .unwrap();
        for frame in 0..10 {
            if frame != 0 {
                factory.borrow_mut().add_frame();
            }
            factory.borrow_mut().frame_size(width as u32, height as u32);
            let mut renderer = factory.borrow().make_renderer();
            let (scripted_renderer, _scope) = ScriptedRenderer::create_call_scoped_userdata(
                &lua,
                &mut renderer,
                bindings.clone(),
            )
            .unwrap();
            lua.globals()
                .get::<luaur_rt::Function>("render")
                .unwrap()
                .call::<()>((userdata.clone(), scripted_renderer.clone()))
                .expect("pinned renderer userdata and Vertical model");
            assert!(
                scripted_renderer
                    .borrow::<ScriptedRenderer>()
                    .unwrap()
                    .end()
            );
        }
        let expected = fixture_bytes("../silvers/scripted_artboard_render.sriv");
        sriv::compare_sriv(
            &sriv::parse_sriv(&expected).unwrap(),
            &sriv::parse_sriv(&factory.borrow().bytes()).unwrap(),
        )
        .expect("pinned scripted_artboard_render silver");
    }

    #[test]
    fn upstream_can_access_nodes_from_artboards() {
        let lua = Lua::new();
        lua.globals()
            .set(
                "artboard",
                fixture_userdata(&lua, "joel_v3.riv", Some("Character")),
            )
            .unwrap();
        let values: Table = lua
            .load(
                r#"
                local muzzle = artboard:node('muzzle')
                local before = { muzzle.x, muzzle.y, muzzle.scaleX, muzzle.scaleY }
                muzzle:decompose(Mat2D.identity())
                return {
                  muzzle ~= nil,
                  before[1], before[2], before[3], before[4],
                  muzzle.x, muzzle.y, muzzle.scaleX, muzzle.scaleY,
                  #muzzle.children, muzzle.parent ~= nil,
                  #artboard:node('Weapon').children
                }
                "#,
            )
            .eval()
            .expect("pinned ScriptedNode surface");
        assert!(values.get::<bool>(1).unwrap());
        assert_eq!(values.get::<f32>(2).unwrap(), 203.0);
        assert_eq!(values.get::<f32>(3).unwrap(), 0.0);
        assert!((values.get::<f32>(4).unwrap() - 1.250_002_980_2).abs() < 1e-6);
        assert!((values.get::<f32>(5).unwrap() - 1.250_002_980_2).abs() < 1e-6);
        assert_eq!(values.get::<f32>(6).unwrap(), 0.0);
        assert_eq!(values.get::<f32>(7).unwrap(), 0.0);
        assert_eq!(values.get::<f32>(8).unwrap(), 1.0);
        assert_eq!(values.get::<f32>(9).unwrap(), 1.0);
        assert_eq!(values.get::<usize>(10).unwrap(), 0);
        assert!(values.get::<bool>(11).unwrap());
        assert_eq!(values.get::<usize>(12).unwrap(), 9);
    }

    #[test]
    fn upstream_can_add_artboard_to_path() {
        let lua = Lua::new();
        lua.globals()
            .set(
                "artboard",
                fixture_userdata(&lua, "joel_v3.riv", Some("Character")),
            )
            .unwrap();
        lua.load(
            r#"
            local path = Path.new()
            artboard:addToPath(path)
            local transformed = Path.new()
            artboard:addToPath(transformed, Mat2D.identity())
            "#,
        )
        .exec()
        .expect("pinned addToPath overloads");
    }
}
