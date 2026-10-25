use super::{text_style::TextStyleOwner, text_style_global::TextStyleGlobal};
use crate::source::{
    animation::{
        state_machine::StateMachine, state_machine_layer::StateMachineLayer,
        state_machine_listener::StateMachineListener,
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    core::CoreHandle,
    core_context::CoreContext,
    factory::RuntimeFactoryHandle,
    file::{RuntimeFileHandle, RuntimeFileWeakHandle},
    viewmodel::viewmodel::ViewModel,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};
#[derive(Clone, Copy, Debug, Default)]
pub struct TextStyleShareInfo {
    pub shareable: bool,
    pub reads_outside_own_view_model: bool,
    pub listens_to_runs: bool,
    pub has_state_machine: bool,
}
struct SharedNotifier {
    instance: CoreHandle,
    consumers: Rc<RefCell<Vec<CoreHandle>>>,
}
impl TextStyleOwner for SharedNotifier {
    fn style_shape_changed(&self) {
        Artboard::update_pass_handle(&self.instance, false);
        for text in self.consumers.borrow().clone() {
            text.with_mut(|t| t.text_interface_mark_shape_dirty());
        }
    }
}
pub struct SharedTextStyle {
    instance: Option<RuntimeArtboardInstanceHandle>,
    view_model_instance: Option<CoreHandle>,
    paint: Option<CoreHandle>,
    consumers: Rc<RefCell<Vec<CoreHandle>>>,
    _owner: Rc<dyn TextStyleOwner>,
}
impl SharedTextStyle {
    fn new(
        source: &CoreHandle,
        file: Option<&RuntimeFileHandle>,
        factory: RuntimeFactoryHandle,
    ) -> Self {
        let instance = Artboard::instance_from_handle_with_factory(source, Some(factory))
            .expect("style source");
        let paint = TextStyleGlobal::first_text_style_paint(Some(&instance.core_handle()));
        let id = source
            .with_downcast::<Artboard, _>(|s| s.base.view_model_id() as usize)
            .unwrap();
        let vm = file.and_then(|f| {
            f.view_model(id)
                .and_then(|vm| f.with_file(|f| f.create_default_view_model_instance(vm)))
        });
        if let (Some(vm), Some(file)) = (&vm, file) {
            file.complete_view_model_properties(vm);
            instance.bind_view_model_instance(Some(vm.clone()));
        }
        instance.update_data_binds(false);
        Artboard::update_pass_handle(&instance.core_handle(), false);
        let consumers = Rc::new(RefCell::new(Vec::new()));
        let owner: Rc<dyn TextStyleOwner> = Rc::new(SharedNotifier {
            instance: instance.core_handle(),
            consumers: consumers.clone(),
        });
        if let Some(p) = &paint {
            p.with_mut(|p| {
                let p = p.as_text_style_paint_mut().unwrap();
                p.set_owner(Some(Rc::downgrade(&owner)));
                p.set_shared(true);
            });
        }
        Self {
            instance: Some(instance),
            view_model_instance: vm,
            paint,
            consumers,
            _owner: owner,
        }
    }
    pub fn paint(&self) -> Option<CoreHandle> {
        self.paint.clone()
    }
    pub fn artboard_instance(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.instance.clone()
    }
}
impl Drop for SharedTextStyle {
    fn drop(&mut self) {
        self.instance = None;
        self.view_model_instance = None;
    }
}
#[derive(Default)]
pub struct SharedTextStyleLease {
    cache: Weak<RefCell<TextStyleShareCache>>,
    style: Weak<SharedTextStyle>,
    consumer: Option<CoreHandle>,
}
impl SharedTextStyleLease {
    pub fn get(&self) -> Option<Rc<SharedTextStyle>> {
        let c = self.cache.upgrade()?;
        if !c.borrow().connected {
            return None;
        }
        self.style.upgrade()
    }
    pub fn release(&mut self) {
        if let (Some(cache), Some(style), Some(consumer)) = (
            self.cache.upgrade(),
            self.style.upgrade(),
            self.consumer.take(),
        ) {
            let dead = {
                let mut cache = cache.borrow_mut();
                if cache.connected {
                    cache.release(&style, &consumer)
                } else {
                    None
                }
            };
            drop(dead);
        }
        self.cache = Weak::new();
        self.style = Weak::new();
    }
}
impl Drop for SharedTextStyleLease {
    fn drop(&mut self) {
        self.release();
    }
}
pub struct TextStyleShareCache {
    file: RuntimeFileWeakHandle,
    this: Weak<RefCell<Self>>,
    connected: bool,
    info: HashMap<CoreHandle, TextStyleShareInfo>,
    any: Option<bool>,
    shared: Vec<(CoreHandle, RuntimeFactoryHandle, Rc<SharedTextStyle>)>,
}
impl TextStyleShareCache {
    pub fn new(file: RuntimeFileWeakHandle) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|this| {
            RefCell::new(Self {
                file,
                this: this.clone(),
                connected: true,
                info: HashMap::new(),
                any: None,
                shared: Vec::new(),
            })
        })
    }
    pub fn set_file(&mut self, file: RuntimeFileWeakHandle) {
        self.file = file;
    }
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.shared.clear();
    }
    pub fn live_count(&self) -> usize {
        self.shared.len()
    }
    pub fn info(&mut self, source: &CoreHandle) -> TextStyleShareInfo {
        if let Some(info) = self.info.get(source) {
            return *info;
        }
        let info = Self::analyze(source, self.file.upgrade().as_ref());
        self.info.insert(source.clone(), info);
        info
    }
    pub fn any_listens_to_runs(&mut self) -> bool {
        if let Some(any) = self.any {
            return any;
        }
        let sources = self.file.with_file(|f| f.artboards()).unwrap_or_default();
        let any = sources.iter().any(|s| {
            TextStyleGlobal::first_text_style_paint(Some(s)).is_some()
                && self.info(s).listens_to_runs
        });
        self.any = Some(any);
        any
    }
    pub fn forget(&mut self, source: &CoreHandle) {
        self.info.remove(source);
        self.any = None;
    }
    pub fn acquire(
        &mut self,
        source: &CoreHandle,
        factory: RuntimeFactoryHandle,
        consumer: Option<CoreHandle>,
    ) -> SharedTextStyleLease {
        let Some(consumer) = consumer else {
            return SharedTextStyleLease::default();
        };
        if !self.info(source).shareable {
            return SharedTextStyleLease::default();
        }
        let style = self
            .shared
            .iter()
            .find(|(s, f, _)| {
                s == source
                    && f.persistent_context().identity() == factory.persistent_context().identity()
            })
            .map(|(_, _, s)| s.clone())
            .unwrap_or_else(|| {
                let s = Rc::new(SharedTextStyle::new(
                    source,
                    self.file.upgrade().as_ref(),
                    factory.clone(),
                ));
                if s.paint().is_some() {
                    self.shared.push((source.clone(), factory, s.clone()));
                }
                s
            });
        if style.paint().is_none() {
            return SharedTextStyleLease::default();
        }
        style.consumers.borrow_mut().push(consumer.clone());
        SharedTextStyleLease {
            cache: self.this.clone(),
            style: Rc::downgrade(&style),
            consumer: Some(consumer),
        }
    }
    fn release(
        &mut self,
        style: &Rc<SharedTextStyle>,
        consumer: &CoreHandle,
    ) -> Option<Rc<SharedTextStyle>> {
        let mut consumers = style.consumers.borrow_mut();
        if let Some(i) = consumers.iter().position(|c| c == consumer) {
            consumers.remove(i);
        }
        if !consumers.is_empty() {
            return None;
        }
        drop(consumers);
        let i = self
            .shared
            .iter()
            .position(|(_, _, s)| Rc::ptr_eq(s, style))?;
        Some(self.shared.remove(i).2)
    }
    pub fn analyze(source: &CoreHandle, file: Option<&RuntimeFileHandle>) -> TextStyleShareInfo {
        let mut info = TextStyleShareInfo::default();
        let Some((own, binds, machine, objects)) = source.with_downcast::<Artboard, _>(|s| {
            let index = s.default_state_machine_index();
            (
                s.base.view_model_id(),
                s.data_bind_handles(),
                (index >= 0)
                    .then(|| s.state_machine_handle_at(index as usize))
                    .flatten(),
                s.objects().to_vec(),
            )
        }) else {
            return info;
        };
        let reads_own = |bind: &CoreHandle| {
            bind.with_downcast::<crate::source::data_bind::data_bind_context::DataBindContext, _>(
                |context| {
                    !context.base.is_name_based() && context.source_path_ids().first() == Some(&own)
                },
            )
            .unwrap_or(false)
        };
        let mut only_own = true;
        let mut static_binds = true;
        for bind in binds {
            if !reads_own(&bind) {
                only_own = false;
            }
            let converter = bind
                .with_downcast::<crate::source::data_bind::data_bind_context::DataBindContext, _>(
                    |b| (!b.base.is_name_based()).then(|| b.base.converter()),
                )
                .flatten();
            if let Some(converter) = converter {
                if !static_converter(converter) {
                    static_binds = false;
                }
            }
        }
        if let Some(machine) = machine {
            machine.with_downcast::<StateMachine,_>(|m|{
   for i in 0..m.data_bind_count(){if let Some(b)=m.data_bind(i){if !reads_own(&b){only_own=false;}}}
   info.has_state_machine=m.listener_count()!=0||m.data_bind_count()!=0;
   for i in 0..m.layer_count(){if let Some(layer)=m.layer(i){layer.with_downcast::<StateMachineLayer,_>(|l|{for state in l.states(){if !state.is_type_of(crate::source::generated::animation::entry_state_base::EntryStateBase::TYPE_KEY)&&!state.is_type_of(crate::source::generated::animation::any_state_base::AnyStateBase::TYPE_KEY)&&!state.is_type_of(crate::source::generated::animation::exit_state_base::ExitStateBase::TYPE_KEY){info.has_state_machine=true;}}});}}
   info.listens_to_runs=(0..m.listener_count()).any(|i|m.listener(i).is_some_and(|l|l.with_downcast::<StateMachineListener,_>(|l|l.has_pointer_listeners()&&source.with_downcast::<Artboard,_>(|a|a.resolve(l.base.target_id())).flatten().is_none()).unwrap_or(false)));
  });
        }
        info.reads_outside_own_view_model = !only_own;
        if cfg!(feature = "editor") || !only_own || !static_binds || info.has_state_machine {
            return info;
        }
        if objects.into_iter().flatten().any(|o| !plain_object(&o)) {
            return info;
        }
        if file
            .and_then(|f| f.view_model(own as usize))
            .is_some_and(|vm| {
                vm.with_downcast::<ViewModel, _>(|vm| {
                    vm.properties().iter().any(|p| {
                        p.with(|p| {
                            p.as_view_model_property()
                                .is_some_and(|p| p.is_input() || p.is_output())
                        })
                        .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
            })
        {
            return info;
        }
        info.shareable = true;
        info
    }
}
impl Drop for TextStyleShareCache {
    fn drop(&mut self) {
        self.disconnect();
    }
}
fn plain_object(o: &CoreHandle) -> bool {
    use crate::source::generated::{
        artboard_base::ArtboardBase,
        data_bind::{data_bind_base::DataBindBase, data_bind_context_base::DataBindContextBase},
        layout::layout_component_style_base::LayoutComponentStyleBase,
        shapes::paint::{
            feather_base::FeatherBase, fill_base::FillBase, gradient_stop_base::GradientStopBase,
            linear_gradient_base::LinearGradientBase, radial_gradient_base::RadialGradientBase,
            solid_color_base::SolidColorBase, stroke_base::StrokeBase,
        },
        text::{
            text_style_axis_base::TextStyleAxisBase, text_style_feature_base::TextStyleFeatureBase,
            text_style_paint_base::TextStylePaintBase,
        },
    };
    o.with(|o| match o.core_type() {
        ArtboardBase::TYPE_KEY
        | LayoutComponentStyleBase::TYPE_KEY
        | TextStylePaintBase::TYPE_KEY
        | TextStyleAxisBase::TYPE_KEY
        | TextStyleFeatureBase::TYPE_KEY
        | FillBase::TYPE_KEY
        | StrokeBase::TYPE_KEY
        | SolidColorBase::TYPE_KEY
        | LinearGradientBase::TYPE_KEY
        | RadialGradientBase::TYPE_KEY
        | GradientStopBase::TYPE_KEY
        | DataBindBase::TYPE_KEY
        | DataBindContextBase::TYPE_KEY => true,
        FeatherBase::TYPE_KEY => !o
            .as_any()
            .downcast_ref::<crate::source::shapes::paint::feather::Feather>()
            .unwrap()
            .inner(),
        _ => false,
    })
    .unwrap_or(false)
}
fn static_converter(converter: Option<CoreHandle>) -> bool {
    let Some(c) = converter else {
        return true;
    };
    use crate::source::generated::{
        data_bind::converters::{
            data_converter_formula_base::DataConverterFormulaBase,
            data_converter_operation_viewmodel_base::DataConverterOperationViewModelBase,
        },
        scripted::scripted_data_converter_base::ScriptedDataConverterBase,
    };
    if c.with(|c| {
        matches!(
            c.core_type(),
            DataConverterFormulaBase::TYPE_KEY
                | DataConverterOperationViewModelBase::TYPE_KEY
                | ScriptedDataConverterBase::TYPE_KEY
        )
    })
    .unwrap_or(false)
    {
        return false;
    }
    let Some((advances, binds)) = c
        .with(|c| {
            c.as_data_converter().map(|base| {
                (
                    c.as_data_converter_capability()
                        .is_some_and(|c| c.may_advance()),
                    !base.data_binds().is_empty(),
                )
            })
        })
        .flatten()
    else {
        return false;
    };
    if advances || binds {
        return false;
    }
    use crate::source::data_bind::converters::{
        data_converter_group::DataConverterGroup, data_converter_group_item::DataConverterGroupItem,
    };
    let items = c.with_downcast::<DataConverterGroup, _>(|g| g.items().to_vec());
    items.is_none_or(|items| {
        items.iter().all(|i| {
            static_converter(
                i.with_downcast::<DataConverterGroupItem, _>(|i| i.converter())
                    .flatten(),
            )
        })
    })
}
