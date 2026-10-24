//! Occurrence values belong to a flight, never to a player or a retained row.
use crate::{
    CoreHandle, RuntimeArtboardInstanceHandle,
    source::{
        data_bind::{
            data_bind::{BINDINGS, DataBind},
            data_bind_context::DataBindContext,
        },
        generated::{
            core_registry::CoreRegistry,
            text::text_input_base::TextInputBase,
            viewmodel::{
                viewmodel_instance_color_base::ViewModelInstanceColorBase,
                viewmodel_instance_number_base::ViewModelInstanceNumberBase,
                viewmodel_instance_string_base::ViewModelInstanceStringBase,
            },
        },
        shapes::paint::color::color_lerp,
        text::text_input::TextInput,
        viewmodel::viewmodel_instance::ViewModelInstance,
    },
};
use std::{cell::Cell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy)]
enum Start {
    Number(f32),
    Color(i32),
}

/// The binding has only a weak link here. Ordinary bind updates are held;
/// only the follower samples and projects its converted source-to-target value.
pub(crate) struct ValueBlend {
    start: Start,
    progress: Cell<f32>,
    sampling: Cell<bool>,
}
impl ValueBlend {
    pub(crate) fn sampling(&self) -> bool {
        self.sampling.get()
    }
    pub(crate) fn number(&self, end: f32) -> f32 {
        let Start::Number(start) = self.start else {
            return end;
        };
        let t = self.progress.get();
        if t == 0.0 {
            start
        } else if t == 1.0 {
            end
        } else {
            start + (end - start) * t
        }
    }
    pub(crate) fn color(&self, end: i32) -> i32 {
        let Start::Color(start) = self.start else {
            return end;
        };
        color_lerp(start as u32, end as u32, self.progress.get()) as i32
    }
}

#[derive(Default)]
pub(super) struct FlightValues(Vec<(CoreHandle, Rc<ValueBlend>)>);
impl Drop for FlightValues {
    fn drop(&mut self) {
        // Cancellation/reconciliation can retain the mounted binding instead
        // of replacing it. Queue its source-only handoff at the next ordinary
        // settlement; never write values from destruction or visit other rows.
        for (bind, blend) in std::mem::take(&mut self.0) {
            drop(blend);
            if bind.is_alive() {
                DataBind::add_dirt_handle(&bind, BINDINGS, false);
            }
        }
    }
}
impl Clone for FlightValues {
    // Flight clones are public pose/control snapshots. They must not keep the
    // live flight's binding overrides alive after landing or reconciliation.
    fn clone(&self) -> Self {
        Self::default()
    }
}
impl FlightValues {
    pub(super) fn attach(&mut self, bind: &CoreHandle, target: &CoreHandle) {
        let Some(key) = bind
            .with(|o| {
                o.as_data_bind()
                    .filter(|b| b.to_target())
                    .map(|b| b.property_key())
            })
            .flatten()
        else {
            return;
        };
        let start = match key as u16 {
            ViewModelInstanceNumberBase::PROPERTY_VALUE_PROPERTY_KEY => {
                CoreRegistry::get_double_handle(target, key as i32).map(Start::Number)
            }
            ViewModelInstanceColorBase::PROPERTY_VALUE_PROPERTY_KEY => {
                CoreRegistry::get_color_handle(target, key as i32).map(Start::Color)
            }
            _ => None,
        };
        let Some(start) = start else {
            return;
        };
        let blend = Rc::new(ValueBlend {
            start,
            progress: Cell::new(0.0),
            sampling: Cell::new(false),
        });
        bind.with_mut(|o| {
            let bind = o.as_data_bind_mut().unwrap();
            bind.require_source_adoption();
            bind.live_move_blend = Rc::downgrade(&blend);
        });
        self.0.push((bind.clone(), blend));
    }
    pub(super) fn follow(&self, progress: f32) {
        for (bind, blend) in &self.0 {
            if !bind.is_alive() {
                continue;
            }
            blend.progress.set(progress);
            blend.sampling.set(true);
            // Sample even a clean/once binding: its destination is live for
            // the entire flight. Converters still use the ordinary bind path.
            DataBind::update_handle(bind, BINDINGS);
            blend.sampling.set(false);
        }
    }
}

/// Only edited fields whose local string property had no outgoing occurrence
/// binding can override an incoming prop. An internal field <-> unit-property
/// bind is local state, not a Screen occurrence binding.
#[derive(Default)]
pub(super) struct TypedText(HashMap<u32, String>);
impl TypedText {
    pub(super) fn capture(instance: &RuntimeArtboardInstanceHandle, bound: &[u32]) -> Self {
        let Some(vm) = instance
            .data_context()
            .and_then(|c| c.with_context(|c| c.main_view_model_instance()))
        else {
            return Self::default();
        };
        let values = vm
            .with_downcast::<ViewModelInstance, _>(|v| v.property_values().to_vec())
            .unwrap_or_default();
        let binds = instance.with_artboard(|b| b.data_bind_handles());
        let mut bound = bound.to_vec();
        // After take-off the mounted props are the landing props, whereas the
        // flight slot's inline model still describes its placeholder.
        for bind in &binds {
            if let Some(target) = bind
                .with(|o| {
                    let b = o.as_data_bind()?;
                    b.is_instance_value_bind().then(|| b.target()).flatten()
                })
                .flatten()
                .filter(|t| values.contains(t))
            {
                if let Some(id) = target
                    .with(|o| {
                        o.as_view_model_instance_value()
                            .map(|v| v.view_model_property_id())
                    })
                    .flatten()
                {
                    bound.push(id);
                }
            }
        }
        let mut text = HashMap::new();
        for field in instance.with_artboard(|b| b.find_all_handles::<TextInput>()) {
            let Some(typed) = field
                .with_downcast::<TextInput, _>(|f| f.live_move_typed_text())
                .flatten()
            else {
                continue;
            };
            for bind in &binds {
                let source = bind
                    .with(|o| {
                        let b = o.as_data_bind()?;
                        (b.target().as_ref() == Some(&field)
                            && b.property_key() == u32::from(TextInputBase::TEXT_PROPERTY_KEY))
                        .then(|| b.source())
                        .flatten()
                    })
                    .flatten()
                    .filter(|s| values.contains(s));
                if let Some(source) = source {
                    let id = source
                        .with(|o| {
                            o.as_view_model_instance_value()
                                .map(|v| v.view_model_property_id())
                        })
                        .flatten();
                    if let Some(id) = id.filter(|id| !bound.contains(id)) {
                        text.insert(id, typed.clone());
                    }
                }
            }
        }
        Self(text)
    }
    pub(super) fn write_back(
        &self,
        id: u32,
        bind: &CoreHandle,
        target: &CoreHandle,
        instance: &RuntimeArtboardInstanceHandle,
    ) {
        let Some(text) = self.0.get(&id) else {
            return;
        };
        let key = i32::from(ViewModelInstanceStringBase::PROPERTY_VALUE_PROPERTY_KEY);
        // Occurrence props in this runtime target unit VM properties. The
        // TextInput itself retains its internal binding and edited buffer.
        if CoreRegistry::get_string_handle(target, key).is_none() {
            return;
        }
        CoreRegistry::set_string_handle(target, key, text.clone());
        DataBindContext::bind_from_context_handle(bind, instance.data_context());
        let Some((source, to_source)) = bind
            .with(|o| {
                let b = o.as_data_bind()?;
                Some((b.source()?, b.to_source()))
            })
            .flatten()
        else {
            return;
        };
        if to_source {
            // Use the binding's reverse converter and normal source setter.
            DataBind::update_source_binding_handle(bind, true);
        } else if CoreRegistry::get_string_handle(&source, key).is_some() {
            // A one-way string prop has no reverse conversion contract.
            CoreRegistry::set_string_handle(&source, key, text.clone());
        }
    }
}
