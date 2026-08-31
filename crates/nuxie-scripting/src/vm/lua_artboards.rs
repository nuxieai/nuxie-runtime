// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/lua_artboards.cpp
use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{
    AnyUserData, Error, Lua, Result, Table, UserData, UserDataFields, UserDataMethods, Value,
    Vector as LuaVector,
};
use nuxie_runtime::{
    ScriptAnimation, ScriptAnimationTime, ScriptArtboard, ScriptMethod, ScriptNode,
};

use super::lua_mat2d::ScriptedMat2D;
use super::lua_paint::ScriptedPaintData;
use super::lua_path::{ScriptedPath, create_scripted_path};
use super::lua_renderer::ScriptedRenderer;
use super::lua_renderer_library::RendererBindings;
use super::view_model::{
    ScriptViewModelRegistration, create_scripted_view_model, model_from_table,
};

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
    _registration: Option<ScriptViewModelRegistration>,
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
        let registration = artboard
            .data()
            .as_ref()
            .map(|model| bindings.view_model_frame_context.register(model));
        Self {
            owner: Rc::new(ScriptedArtboardOwner {
                artboard,
                _registration: registration,
            }),
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
        methods.add_method("draw", |_, this, renderer: AnyUserData| {
            let scripted_renderer = renderer.borrow::<ScriptedRenderer>()?;
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
    fn scripted_artboard_keeps_its_bound_instance_registered_for_its_lifetime() {
        let (model, trigger, native) = trigger_artboard();
        let context = ScriptViewModelFrameContext::default();
        let artboard = ScriptedArtboard::new(native, RendererBindings::new(context.clone()));

        assert!(model.fire_trigger(&trigger));
        assert!(context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(0));

        drop(artboard);
        assert!(model.fire_trigger(&trigger));
        assert!(!context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(1));
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

        assert!(context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(0));
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
