//! ScriptedGPURenderPass command dispatch from lua_gpu.cpp.
use super::pipeline::{BindGroup, Pipeline};
use super::*;
use nuxie_ore_metal::render_pass::RenderPassApi;

pub(super) struct Pass {
    pub pass: Option<Box<dyn RenderPassApi>>,
    pub finished: bool,
    pub sample_count: u32,
    pub pipeline_set: bool,
    pub draw_call_count: u32,
}
impl Pass {
    fn validate(&self) -> Result<()> {
        if self.finished
            || self.pass.as_ref().is_none_or(|pass| {
                pass.activeToken()
                    .upgrade()
                    .is_none_or(|token| token.isFinished())
            })
        {
            return Err(Error::runtime(
                "render pass expired — already finished, or auto-finished by a subsequent beginRenderPass",
            ));
        }
        Ok(())
    }
    fn require_pipeline(&self) -> Result<()> {
        if !self.pipeline_set {
            return Err(Error::runtime(
                "setPipeline must be called before draw/setVertexBuffer/setBindGroup",
            ));
        }
        Ok(())
    }
    fn pass(&mut self) -> &mut dyn RenderPassApi {
        &mut **self.pass.as_mut().expect("validated pass")
    }
}
impl UserData for Pass {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("setPipeline",|lua,this,data:AnyUserData| {
            this.validate()?;let pipeline=data.borrow::<Pipeline>()?;
            if pipeline.sample_count!=this.sample_count {return Err(Error::runtime(format!("pipeline sampleCount ({}) does not match render pass sampleCount ({}) — recreate the pipeline with matching sampleCount",pipeline.sample_count,this.sample_count)));}
            let context=context(lua)?;context.borrow().clearLastError();
            this.pass().setPipeline(Some(&pipeline.resource));
            let error=context.borrow().lastError();if !error.is_empty(){return Err(Error::runtime(format!("setPipeline: {error}")));}
            this.pipeline_set=true;Ok(())
        });
        methods.add_method_mut(
            "setVertexBuffer",
            |_, this, (slot, data): (u32, AnyUserData)| {
                this.validate()?;
                if slot > 7 {
                    return Err(Error::runtime(format!(
                        "setVertexBuffer: slot must be 0-7 (got {slot})"
                    )));
                }
                let buffer = data.borrow::<Buffer>()?;
                this.pass().setVertexBuffer(slot, Some(&buffer.resource), 0);
                Ok(())
            },
        );
        methods.add_method_mut(
            "setIndexBuffer",
            |lua, this, (data, format): (AnyUserData, Value)| {
                this.validate()?;
                let buffer = data.borrow::<Buffer>()?;
                let format = string_value(lua, format)?;
                this.pass().setIndexBuffer(
                    Some(&buffer.resource),
                    if format.as_deref() == Some("uint32") {
                        IndexFormat::uint32
                    } else {
                        IndexFormat::uint16
                    },
                    0,
                );
                Ok(())
            },
        );
        methods.add_method_mut("setBindGroup",|lua,this,(group,data,offsets):(u32,AnyUserData,Value)| {
            this.validate()?;if group>=kMaxBindGroups {return Err(Error::runtime(format!("setBindGroup: groupIndex must be in [0, {kMaxBindGroups}) (got {group})")));}
            let bg=data.borrow::<BindGroup>()?;let mut values=Vec::new();
            if let Value::Table(offsets)=offsets {
                if offsets.raw_len()>8 {return Err(Error::runtime(format!("setBindGroup: dynamicOffsets count {} exceeds maximum of 8",offsets.raw_len())));}
                for index in 0..offsets.raw_len() {let offset=number_value(lua,offsets.raw_get::<Value>(index+1)?,0.0)? as u32;if offset%256!=0{return Err(Error::runtime(format!("setBindGroup: dynamicOffsets[{index}] = {offset} is not a multiple of 256 (alignment requirement)")));}values.push(offset);}
            }
            let expected=bg.resource.bindGroupBase().expect("bind group").dynamicOffsetCount();
            if values.len() as u32!=expected {return Err(Error::runtime(format!("setBindGroup: dynamicOffsets count {} does not match the BindGroup's declared dynamic UBO count {expected}",values.len())));}
            this.pass().setBindGroup(group,Some(&bg.resource),if values.is_empty(){None}else{Some(&values)},values.len() as u32);Ok(())
        });
        methods.add_method_mut(
            "setViewport",
            |_, this, (x, y, w, h): (f32, f32, f32, f32)| {
                this.validate()?;
                this.pass().setViewport(x, y, w, h, 0.0, 1.0);
                Ok(())
            },
        );
        methods.add_method_mut(
            "setScissorRect",
            |_, this, (x, y, w, h): (u32, u32, u32, u32)| {
                this.validate()?;
                this.pass().setScissorRect(x, y, w, h);
                Ok(())
            },
        );
        methods.add_method_mut("setStencilReference", |_, this, value: u32| {
            this.validate()?;
            this.pass().setStencilReference(value);
            Ok(())
        });
        methods.add_method_mut(
            "setBlendColor",
            |_, this, (r, g, b, a): (f32, f32, f32, f32)| {
                this.validate()?;
                this.pass().setBlendColor(r, g, b, a);
                Ok(())
            },
        );
        methods.add_method_mut("draw",|lua,this,(count,instances,first,first_instance):(u32,Value,Value,Value)| {
            this.validate()?;this.require_pipeline()?;let first_instance=number_value(lua,first_instance,0.0)? as u32;
            let context=context(lua)?;let ctx=context.borrow();
            if first_instance>0 && ctx.featuresKnown() && !ctx.features().drawBaseInstance {return Err(Error::runtime(format!("draw: firstInstance={first_instance} requires the drawBaseInstance feature, which the active backend does not support")));}
            drop(ctx);this.pass().draw(count,number_value(lua,instances,1.0)? as u32,number_value(lua,first,0.0)? as u32,first_instance);this.draw_call_count=this.draw_call_count.wrapping_add(1);Ok(())
        });
        methods.add_method_mut("drawIndexed",|lua,this,(count,instances,first,base,first_instance):(u32,Value,Value,Value,Value)| {
            this.validate()?;this.require_pipeline()?;let base=number_value(lua,base,0.0)? as i32;let first_instance=number_value(lua,first_instance,0.0)? as u32;
            let context=context(lua)?;let ctx=context.borrow();
            if ctx.featuresKnown() && !ctx.features().drawBaseInstance {
                if base!=0 {return Err(Error::runtime(format!("drawIndexed: baseVertex={base} requires the drawBaseInstance feature, which the active backend does not support")));}
                if first_instance>0 {return Err(Error::runtime(format!("drawIndexed: firstInstance={first_instance} requires the drawBaseInstance feature, which the active backend does not support")));}
            }
            drop(ctx);this.pass().drawIndexed(count,number_value(lua,instances,1.0)? as u32,number_value(lua,first,0.0)? as u32,base,first_instance);this.draw_call_count=this.draw_call_count.wrapping_add(1);Ok(())
        });
        methods.add_method_mut("finish", |lua, this, ()| {
            this.validate()?;
            this.pass().finish();
            this.finished = true;
            let context = context(lua)?;
            let ctx = context.borrow();
            if let Some(active) = ctx.activeRenderPass() {
                if std::rc::Weak::ptr_eq(&active, &this.pass.as_ref().expect("pass").activeToken())
                {
                    ctx.setActiveRenderPass(None);
                }
            }
            Ok(())
        });
    }
}
