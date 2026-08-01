// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_renderer.cpp
use std::cell::{Cell, RefCell};
use std::mem;
use std::ptr::NonNull;
use std::rc::Rc;

use luaur_rt::{AnyUserData, Error, Result, Table, UserData, UserDataMethods, Value};
use nuxie_render_api::{Factory as RenderFactory, Renderer};

use super::lua_mat2d::ScriptedMat2D;
use super::lua_paint::{ScriptedPaint, parse_blend_mode_name};
use super::lua_path::ScriptedPath;
use super::lua_renderer_library::RendererBindings;
use crate::gpu_canvas::{GpuCanvasImage, ScriptedImageSampler, with_gpu_canvas_image};

impl RendererBindings {
    pub(crate) fn call_draw(
        &self,
        table: &Table,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
    ) -> Result<()> {
        let value: Value = table.get("draw")?;
        let Value::Function(function) = value else {
            return Ok(());
        };

        let lua = table.lua();
        self.verify_render_context(factory)?;
        let save_count = Rc::new(Cell::new(0usize));
        let valid = Rc::new(Cell::new(true));
        let renderer_ref = Rc::new(RefCell::new(erase_renderer_lifetime(renderer)));

        let scripted_renderer = lua.create_userdata(ScriptedRenderer {
            renderer: Rc::clone(&renderer_ref),
            bindings: self.clone(),
            save_count: Rc::clone(&save_count),
            valid: Rc::clone(&valid),
        })?;
        let result = function.call::<()>((table.clone(), scripted_renderer));

        while save_count.get() > 0 {
            let mut renderer = renderer_ref.borrow_mut();
            // The renderer userdata is still valid while this cleanup runs;
            // the pointer is invalidated immediately after the save stack is
            // balanced.
            unsafe { renderer.as_mut().restore() };
            save_count.set(save_count.get() - 1);
        }
        valid.set(false);
        result
    }
}

fn erase_renderer_lifetime(renderer: &mut dyn Renderer) -> NonNull<dyn Renderer> {
    let ptr: NonNull<dyn Renderer + '_> = NonNull::from(renderer);
    // The pointer is held only by userdata created for one draw call; `valid`
    // is cleared before `call_draw` returns.
    unsafe { mem::transmute::<NonNull<dyn Renderer + '_>, NonNull<dyn Renderer>>(ptr) }
}

pub(super) struct ScriptedRenderer {
    renderer: Rc<RefCell<NonNull<dyn Renderer>>>,
    pub(super) bindings: RendererBindings,
    save_count: Rc<Cell<usize>>,
    valid: Rc<Cell<bool>>,
}

impl ScriptedRenderer {
    fn validate(&self) -> Result<()> {
        if self.valid.get() {
            Ok(())
        } else {
            Err(Error::runtime("Renderer is no longer valid"))
        }
    }

    pub(super) fn renderer_mut(&self) -> Result<std::cell::RefMut<'_, NonNull<dyn Renderer>>> {
        self.validate()?;
        Ok(self.renderer.borrow_mut())
    }

    fn save(&self) -> Result<()> {
        let mut renderer = self.renderer_mut()?;
        unsafe { renderer.as_mut().save() };
        self.save_count.set(self.save_count.get() + 1);
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        if self.save_count.get() == 0 {
            return Err(Error::runtime("Renderer save/restore stack was unbalanced"));
        }
        let mut renderer = self.renderer_mut()?;
        unsafe { renderer.as_mut().restore() };
        self.save_count.set(self.save_count.get() - 1);
        Ok(())
    }
}

impl UserData for ScriptedRenderer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("save", |_, this, ()| this.save());
        methods.add_method("restore", |_, this, ()| this.restore());
        methods.add_method("transform", |_, this, matrix: AnyUserData| {
            this.validate()?;
            let matrix = matrix.borrow::<ScriptedMat2D>()?;
            let mut renderer = this.renderer_mut()?;
            unsafe { renderer.as_mut().transform(matrix.0) };
            Ok(())
        });
        methods.add_method("clipPath", |_, this, path: AnyUserData| {
            this.validate()?;
            let mut path = path.borrow_mut::<ScriptedPath>()?;
            this.bindings.with_factory(|factory| {
                let render_path = path.render_path(factory);
                let mut renderer = this.renderer_mut()?;
                unsafe { renderer.as_mut().clip_path(render_path) };
                Ok(())
            })
        });
        methods.add_method(
            "drawPath",
            |_, this, (path, paint): (AnyUserData, AnyUserData)| {
                this.validate()?;
                let mut path = path.borrow_mut::<ScriptedPath>()?;
                let paint = paint.borrow::<ScriptedPaint>()?;
                this.bindings.with_factory(|factory| {
                    let render_path = path.render_path(factory);
                    let mut renderer = this.renderer_mut()?;
                    unsafe {
                        renderer
                            .as_mut()
                            .draw_path(render_path, paint.render_paint.as_ref())
                    };
                    Ok(())
                })
            },
        );
        methods.add_method(
            "drawImage",
            |_,
             this,
             (image, sampler, blend_mode, opacity): (
                AnyUserData,
                AnyUserData,
                String,
                Option<f32>,
            )| {
                this.validate()?;
                let image = image.borrow::<GpuCanvasImage>()?;
                let sampler = sampler.borrow::<ScriptedImageSampler>()?;
                let blend_mode = parse_blend_mode_name(&blend_mode)?;
                with_gpu_canvas_image(&image, |image| {
                    let mut renderer = this.renderer_mut()?;
                    unsafe {
                        renderer.as_mut().draw_image(
                            Some(image),
                            sampler.0,
                            blend_mode,
                            opacity.unwrap_or(1.0),
                        )
                    };
                    Ok(())
                })?
            },
        );
    }
}
