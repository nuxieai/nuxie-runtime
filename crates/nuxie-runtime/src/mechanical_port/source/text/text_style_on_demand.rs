use super::{
    text_style_instance::TextStyleInstance,
    text_style_reference::TextStyleReferenceWeak,
    text_style_share_cache::{SharedTextStyleLease, TextStyleShareCache, TextStyleShareInfo},
};
use crate::source::{
    advance_flags::AdvanceFlags, advancing_component::QuietState,
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle, artboard::Artboard,
    core::CoreHandle, data_bind::data_context::RuntimeDataContextHandle,
    factory::RuntimeFactoryHandle, file::RuntimeFileHandle,
};
use std::{cell::RefCell, rc::Rc};
pub struct TextStyleOnDemand {
    source: CoreHandle,
    info: TextStyleShareInfo,
    _cross_file: Option<RuntimeFileHandle>,
    bound_view_model_instance: Option<CoreHandle>,
    item: Option<CoreHandle>,
    owner: Option<(usize, usize, u64)>,
    instance: Option<TextStyleInstance>,
    shared: SharedTextStyleLease,
    paint: Option<CoreHandle>,
}
impl TextStyleOnDemand {
    pub fn new(
        source: CoreHandle,
        bound: Option<CoreHandle>,
        file: Option<RuntimeFileHandle>,
        cross_file: Option<RuntimeFileHandle>,
        factory: RuntimeFactoryHandle,
        parent: Option<RuntimeDataContextHandle>,
        text: Option<CoreHandle>,
        item: Option<CoreHandle>,
        values_source: Option<CoreHandle>,
        owner: Option<(usize, usize, u64)>,
    ) -> Rc<RefCell<Self>> {
        Rc::new_cyclic(|this| {
            let mut s = Self {
                source: source.clone(),
                info: TextStyleShareInfo::default(),
                _cross_file: cross_file.clone(),
                bound_view_model_instance: bound.clone(),
                item,
                owner,
                instance: None,
                shared: SharedTextStyleLease::default(),
                paint: None,
            };
            let vm_file = cross_file.as_ref().or(file.as_ref());
            let info = if cross_file.is_none() && file.is_some() {
                let cache = file.as_ref().unwrap().with_file(|f| f.text_style_shares());
                let info = cache.borrow_mut().info(&source);
                if bound.is_none() && info.shareable {
                    s.shared = cache
                        .borrow_mut()
                        .acquire(&source, factory.clone(), text.clone());
                    if let Some(shared) = s.shared.get() {
                        s.paint = shared.paint();
                        return RefCell::new(s);
                    }
                }
                info
            } else {
                TextStyleShareCache::analyze(&source, vm_file)
            };
            s.info = info;
            let mut vm = bound;
            if vm.is_none() {
                if let Some(file) = vm_file {
                    let id = source
                        .with_downcast::<Artboard, _>(|s| s.base.view_model_id())
                        .unwrap();
                    vm = file.view_model(id as usize).and_then(|model| {
                        file.with_file(|f| f.create_default_view_model_instance(model))
                    });
                    if cross_file.is_none() {
                        if let Some(vm) = &vm {
                            file.complete_view_model_properties(vm);
                        }
                    }
                }
            }
            let served = info.listens_to_runs.then(|| text.clone()).flatten();
            let mut instance = TextStyleInstance::new(
                &source,
                factory,
                vm,
                info.reads_outside_own_view_model,
                text,
                served,
                Some(TextStyleReferenceWeak::OnDemand(this.clone())),
            );
            s.paint = instance.paint();
            instance.copy_values_from(values_source);
            instance.bind(parent, false);
            let artboard = instance.artboard_instance();
            artboard.update_data_binds(false);
            Artboard::update_pass_handle(&artboard.core_handle(), false);
            s.instance = Some(instance);
            RefCell::new(s)
        })
    }
    pub fn source(&self) -> CoreHandle {
        self.source.clone()
    }
    pub fn bound_view_model_instance(&self) -> Option<CoreHandle> {
        self.bound_view_model_instance.clone()
    }
    pub fn item(&self) -> Option<CoreHandle> {
        self.item.clone()
    }
    pub fn owner(&self) -> Option<(usize, usize, u64)> {
        self.owner
    }
    pub fn is_shared(&self) -> bool {
        self.shared.get().is_some()
    }
    pub fn instance(&self) -> Option<&TextStyleInstance> {
        self.instance.as_ref()
    }
    pub fn resolved_paint(&self) -> Option<CoreHandle> {
        self.paint.clone()
    }
    pub fn listens_to_runs(&self) -> bool {
        self.instance.is_some() && self.info.listens_to_runs
    }
    pub fn run_state_machine(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        self.listens_to_runs()
            .then(|| self.instance.as_ref().unwrap().state_machine())
            .flatten()
    }
    pub fn list_item(&mut self, value: Option<CoreHandle>) {
        if let Some(i) = &mut self.instance {
            i.list_item(value);
        }
    }
    pub fn host_data_context(&mut self, parent: Option<RuntimeDataContextHandle>) {
        if let Some(i) = &mut self.instance {
            i.bind(parent, false);
            i.flush();
        }
    }
    pub fn relink_host_data_context(&mut self) {
        if let Some(i) = &mut self.instance {
            i.relink();
        }
    }
    pub fn update_hosted_data_binds(&mut self) -> bool {
        if let Some(i) = &mut self.instance {
            i.sync_inputs();
            i.flush()
        } else {
            false
        }
    }
    pub fn advance(&mut self, seconds: f32, flags: AdvanceFlags, changed: &mut bool) -> bool {
        let Some(i) = &mut self.instance else {
            return false;
        };
        let dirty = i
            .artboard_instance()
            .with_artboard(|a| a.has_component_dirt());
        let keep = i.advance(seconds, flags);
        if dirty {
            *changed = true;
        }
        keep
    }
    pub fn quiet_state(&self) -> QuietState {
        if self.instance.is_none() {
            QuietState::Quiet
        } else if self.bound_view_model_instance.is_some() {
            QuietState::Never
        } else {
            self.instance.as_ref().unwrap().quiet_state()
        }
    }
}
impl Drop for TextStyleOnDemand {
    fn drop(&mut self) {
        self.paint = None;
        self.instance = None;
        self.shared.release();
        self.bound_view_model_instance = None;
        self.item = None;
    }
}
