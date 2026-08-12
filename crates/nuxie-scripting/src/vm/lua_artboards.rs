// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/lua_artboards.cpp
use std::cell::RefCell;
use std::rc::Rc;

use luaur_rt::{
    AnyUserData, Error, Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value,
};
use nuxie_render_api::RawPath;
use nuxie_runtime::{
    ScriptAnimation, ScriptAnimationTime, ScriptArtboard, ScriptNode,
    ScriptPaint as RuntimeScriptPaint,
};

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
    artboard: RefCell<Box<dyn ScriptArtboard>>,
    _registration: Option<ScriptViewModelRegistration>,
}

struct ScriptedArtboard {
    owner: Rc<ScriptedArtboardOwner>,
    bindings: RendererBindings,
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
        methods.add_method_mut("advance", |_, this, seconds: f32| {
            this.owner
                .artboard
                .borrow_mut()
                .advance_animation(&mut this.animation, seconds)
                .map_err(|error| Error::runtime(error.to_string()))
        });
        for (name, mode) in [
            ("setTime", ScriptAnimationTime::Seconds),
            ("setTimeFrames", ScriptAnimationTime::Frames),
            ("setTimePercentage", ScriptAnimationTime::Percentage),
        ] {
            methods.add_method_mut(name, move |_, this, value: f32| {
                this.owner
                    .artboard
                    .borrow_mut()
                    .set_animation_time(&mut this.animation, value, mode)
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
                artboard: RefCell::new(artboard),
                _registration: registration,
            }),
            bindings,
        }
    }
}

impl UserData for ScriptedArtboard {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.owner.artboard.borrow().width()));
        fields.add_field_method_set("width", |_, this, value: f32| {
            this.owner.artboard.borrow_mut().set_width(value);
            Ok(())
        });
        fields.add_field_method_get("height", |_, this| {
            Ok(this.owner.artboard.borrow().height())
        });
        fields.add_field_method_set("height", |_, this, value: f32| {
            this.owner.artboard.borrow_mut().set_height(value);
            Ok(())
        });
        fields.add_field_method_get("frameOrigin", |_, this| {
            Ok(this.owner.artboard.borrow().frame_origin())
        });
        fields.add_field_method_set("frameOrigin", |_, this, value: bool| {
            this.owner.artboard.borrow_mut().set_frame_origin(value);
            Ok(())
        });
        fields.add_field_method_get("data", |lua, this| {
            Ok(match this.owner.artboard.borrow().data() {
                Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                None => Value::Nil,
            })
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("instance", |lua, this, view_model: Option<Table>| {
            let view_model = view_model.as_ref().map(model_from_table).transpose()?;
            let instance = this
                .owner
                .artboard
                .borrow()
                .instance(view_model)
                .map_err(|error| Error::runtime(error.to_string()))?;
            lua.create_userdata(ScriptedArtboard::new(instance, this.bindings.clone()))
        });
        methods.add_method_mut("advance", |_, this, seconds: f32| {
            this.owner
                .artboard
                .borrow_mut()
                .advance(seconds)
                .map_err(|error| Error::runtime(error.to_string()))
        });
        methods.add_method("animation", |lua, this, name: String| {
            let animation = this
                .owner
                .artboard
                .borrow()
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
                .borrow()
                .node(&name)
                .map_err(|error| Error::runtime(error.to_string()))?;
            Ok(match node {
                Some(node) => Value::UserData(lua.create_userdata(ScriptedNode::new(node))?),
                None => Value::Nil,
            })
        });
        methods.add_method_mut("draw", |_, this, args: MultiValue| {
            let arg_types = args
                .iter()
                .map(|value| match value {
                    Value::UserData(userdata) if userdata.borrow::<ScriptedRenderer>().is_ok() => {
                        "Renderer"
                    }
                    Value::UserData(userdata) if userdata.borrow::<ScriptedArtboard>().is_ok() => {
                        "ScriptedArtboard"
                    }
                    other => other.type_name(),
                })
                .collect::<Vec<_>>()
                .join(",");
            let renderer = args
                .into_iter()
                .filter_map(|value| match value {
                    Value::UserData(userdata) if userdata.borrow::<ScriptedRenderer>().is_ok() => {
                        Some(userdata)
                    }
                    _ => None,
                })
                .next()
                .ok_or_else(|| {
                    Error::runtime(format!(
                        "ScriptedArtboard.draw expected Renderer userdata, got [{arg_types}]"
                    ))
                })?;
            let scripted_renderer = renderer.borrow::<ScriptedRenderer>()?;
            scripted_renderer.bindings.with_factory(|factory| {
                let mut renderer_ref = scripted_renderer.renderer_mut()?;
                this.owner
                    .artboard
                    .borrow_mut()
                    .draw(factory, unsafe { renderer_ref.as_mut() })
                    .map_err(|error| Error::runtime(error.to_string()))
            })
        });
    }
}

pub(super) struct ScriptedNode {
    path: Option<RawPath>,
    paint: Option<RuntimeScriptPaint>,
}

impl ScriptedNode {
    pub(super) fn new(node: ScriptNode) -> Self {
        Self {
            path: node.path,
            paint: node.paint,
        }
    }
}

impl UserData for ScriptedNode {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("asPath", |lua, this, ()| {
            Ok(match this.path.clone() {
                Some(path) => Value::UserData(create_scripted_path(
                    lua,
                    ScriptedPath::from_raw_path(path),
                )?),
                None => Value::Nil,
            })
        });
        methods.add_method("asPaint", |lua, this, ()| {
            Ok(match this.paint {
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
    use nuxie_render_api::{Factory as RenderFactory, Renderer};
    use nuxie_runtime::{ScriptError, ScriptViewModel, ScriptViewModelProperty};

    struct TestScriptArtboard {
        model: ScriptViewModel,
    }

    impl ScriptArtboard for TestScriptArtboard {
        fn width(&self) -> f32 {
            0.0
        }

        fn height(&self) -> f32 {
            0.0
        }

        fn frame_origin(&self) -> bool {
            false
        }

        fn set_width(&mut self, _width: f32) {}

        fn set_height(&mut self, _height: f32) {}

        fn set_frame_origin(&mut self, _frame_origin: bool) {}

        fn data(&self) -> Option<ScriptViewModel> {
            Some(self.model.clone())
        }

        fn instance(
            &self,
            _view_model: Option<ScriptViewModel>,
        ) -> std::result::Result<Box<dyn ScriptArtboard>, ScriptError> {
            Err(ScriptError::new("not used by owner-lifetime test"))
        }

        fn draw(
            &mut self,
            _factory: &mut dyn RenderFactory,
            _renderer: &mut dyn Renderer,
        ) -> std::result::Result<(), ScriptError> {
            Ok(())
        }
    }

    fn trigger_model() -> (ScriptViewModel, String) {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets/script_create_viewmodel_instance.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let file = nuxie_binary::read_runtime_file(&bytes).expect("fixture parses");
        nuxie_runtime::script_view_models(&file)
            .into_values()
            .find_map(|model| {
                let trigger = model.properties().iter().find_map(|(name, kind)| {
                    (*kind == ScriptViewModelProperty::Trigger).then(|| name.clone())
                })?;
                Some((model.named_instance(None)?, trigger))
            })
            .expect("fixture has a trigger model")
    }

    #[test]
    fn scripted_artboard_keeps_its_bound_instance_registered_for_its_lifetime() {
        let (model, trigger) = trigger_model();
        let context = ScriptViewModelFrameContext::default();
        let artboard = ScriptedArtboard::new(
            Box::new(TestScriptArtboard {
                model: model.clone(),
            }),
            RendererBindings::new(context.clone()),
        );

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
        let (model, trigger) = trigger_model();
        let context = ScriptViewModelFrameContext::default();
        let artboard = ScriptedArtboard::new(
            Box::new(TestScriptArtboard {
                model: model.clone(),
            }),
            RendererBindings::new(context.clone()),
        );
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
}
