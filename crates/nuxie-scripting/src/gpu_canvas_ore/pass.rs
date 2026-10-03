//! ScriptedGPURenderPass command dispatch from lua_gpu.cpp.
use super::pipeline::{BindGroup, Pipeline};
use super::*;
use luaur_rt::FromLua;
use nuxie_ore_metal::render_pass::RenderPassApi;
use nuxie_ore_metal::script_guards::*;

pub(super) struct Pass {
    pub pass: Option<Box<dyn RenderPassApi>>,
    pub finished: bool,
    pub sample_count: u32,
    pub pipeline_set: bool,
    pub draw_call_count: u32,
    pub label: String,
}
impl Pass {
    fn validate(&self) -> Result<bool> {
        if self.finished
            || self.pass.as_ref().is_some_and(|pass| {
                pass.activeToken()
                    .upgrade()
                    .is_none_or(|token| token.isFinished())
            })
        {
            return Err(Error::runtime(
                "render pass expired: it was already finished",
            ));
        }
        Ok(self.pass.is_some())
    }
    fn require_pipeline(&self) -> Result<()> {
        if !self.pipeline_set {
            return Err(Error::runtime(kGuardSetPipelineBeforeDraw));
        }
        Ok(())
    }
    fn pass(&mut self) -> &mut dyn RenderPassApi {
        &mut **self.pass.as_mut().expect("validated pass")
    }
}
impl UserData for Pass {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("setPipeline",|lua,this,data:Value| {
            if !this.validate()? { return Ok(()); }let data=AnyUserData::from_lua(data,lua)?;let pipeline=data.borrow::<Pipeline>()?;
            if pipeline.sample_count!=this.sample_count {return Err(Error::runtime(format!("pipeline sampleCount ({}) does not match render pass sampleCount ({}) — recreate the pipeline with matching sampleCount",pipeline.sample_count,this.sample_count)));}
            let context=context(lua)?;context.borrow().clearLastError();
            this.pass().setPipeline(Some(&pipeline.resource));
            let error=context.borrow().lastError();if !error.is_empty(){return Err(Error::runtime(format!("setPipeline: {error}")));}
            this.pipeline_set=true;Ok(())
        });
        methods.add_method_mut(
            "setVertexBuffer",
            |lua, this, (slot, data): (Value, Value)| {
                if !this.validate()? {
                    return Ok(());
                }
                let slot = u32::from_lua(slot, lua)?;
                if slot >= kMaxVertexBufferSlots {
                    return Err(Error::runtime(vertex_slot_range_message(
                        kMaxVertexBufferSlots - 1,
                        slot,
                    )));
                }
                let data = AnyUserData::from_lua(data, lua)?;
                let buffer = data.borrow::<Buffer>()?;
                this.pass().setVertexBuffer(slot, Some(&buffer.resource), 0);
                Ok(())
            },
        );
        methods.add_method_mut(
            "setIndexBuffer",
            |lua, this, (data, format): (Value, Value)| {
                if !this.validate()? {
                    return Ok(());
                }
                let data = AnyUserData::from_lua(data, lua)?;
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
        methods.add_method_mut("setBindGroup",|lua,this,(group,data,offsets):(Value,Value,Value)| {
            if !this.validate()? { return Ok(()); }let group=u32::from_lua(group,lua)?;if group>=kMaxBindGroups {return Err(Error::runtime(format!("setBindGroup: groupIndex must be in [0, {kMaxBindGroups}) (got {group})")));}
            let data=AnyUserData::from_lua(data,lua)?;let bg=data.borrow::<BindGroup>()?;let mut values=Vec::new();
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
            |lua, this, (x, y, w, h): (Value, Value, Value, Value)| {
                if !this.validate()? {
                    return Ok(());
                }
                let x = f32::from_lua(x, lua)?;
                let y = f32::from_lua(y, lua)?;
                let w = f32::from_lua(w, lua)?;
                let h = f32::from_lua(h, lua)?;
                this.pass().setViewport(x, y, w, h, 0.0, 1.0);
                Ok(())
            },
        );
        methods.add_method_mut(
            "setScissorRect",
            |lua, this, (x, y, w, h): (Value, Value, Value, Value)| {
                if !this.validate()? {
                    return Ok(());
                }
                let x = u32::from_lua(x, lua)?;
                let y = u32::from_lua(y, lua)?;
                let w = u32::from_lua(w, lua)?;
                let h = u32::from_lua(h, lua)?;
                this.pass().setScissorRect(x, y, w, h);
                Ok(())
            },
        );
        methods.add_method_mut("setStencilReference", |lua, this, value: Value| {
            if !this.validate()? {
                return Ok(());
            }
            let value = u32::from_lua(value, lua)?;
            this.pass().setStencilReference(value);
            Ok(())
        });
        methods.add_method_mut(
            "setBlendColor",
            |lua, this, (r, g, b, a): (Value, Value, Value, Value)| {
                if !this.validate()? {
                    return Ok(());
                }
                let r = f32::from_lua(r, lua)?;
                let g = f32::from_lua(g, lua)?;
                let b = f32::from_lua(b, lua)?;
                let a = f32::from_lua(a, lua)?;
                this.pass().setBlendColor(r, g, b, a);
                Ok(())
            },
        );
        methods.add_method_mut(
            "draw",
            |lua, this, (count, instances, first, first_instance): (Value, Value, Value, Value)| {
                if !this.validate()? {
                    return Ok(());
                }
                this.require_pipeline()?;
                let count = u32::from_lua(count, lua)?;
                let instances = number_value(lua, instances, 1.0)? as u32;
                let first = number_value(lua, first, 0.0)? as u32;
                let first_instance = number_value(lua, first_instance, 0.0)? as u32;
                let context = context(lua)?;
                let ctx = context.borrow();
                if first_instance > 0 && ctx.featuresKnown() && !ctx.features().drawBaseInstance {
                    return Err(Error::runtime(first_instance_message(
                        "draw",
                        first_instance,
                    )));
                }
                drop(ctx);
                this.pass().draw(count, instances, first, first_instance);
                this.draw_call_count = this.draw_call_count.wrapping_add(1);
                Ok(())
            },
        );
        methods.add_method_mut(
            "drawIndexed",
            |lua,
             this,
             (count, instances, first, base, first_instance): (
                Value,
                Value,
                Value,
                Value,
                Value,
            )| {
                if !this.validate()? {
                    return Ok(());
                }
                this.require_pipeline()?;
                let count = u32::from_lua(count, lua)?;
                let instances = number_value(lua, instances, 1.0)? as u32;
                let first = number_value(lua, first, 0.0)? as u32;
                let base = number_value(lua, base, 0.0)? as i32;
                let first_instance = number_value(lua, first_instance, 0.0)? as u32;
                let context = context(lua)?;
                let ctx = context.borrow();
                if ctx.featuresKnown() && !ctx.features().drawBaseInstance {
                    if base != 0 {
                        return Err(Error::runtime(base_vertex_message("drawIndexed", base)));
                    }
                    if first_instance > 0 {
                        return Err(Error::runtime(first_instance_message(
                            "drawIndexed",
                            first_instance,
                        )));
                    }
                }
                drop(ctx);
                this.pass()
                    .drawIndexed(count, instances, first, base, first_instance);
                this.draw_call_count = this.draw_call_count.wrapping_add(1);
                Ok(())
            },
        );
        methods.add_method_mut("finish", |_, this, ()| {
            if this.validate()? {
                this.pass().finish();
            }
            this.finished = true;
            Ok(())
        });
    }
}
