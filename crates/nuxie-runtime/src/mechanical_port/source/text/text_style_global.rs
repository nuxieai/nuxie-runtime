use super::{
    text_interface::TextInterface,
    text_style_instance::TextStyleInstance,
    text_style_reference::TextStyleReferenceWeak,
    text_style_share_cache::{SharedTextStyleLease, TextStyleShareCache, TextStyleShareInfo},
};
use crate::source::{
    advance_flags::AdvanceFlags,
    advancing_component::QuietState,
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    artboard_referencer::{ArtboardReferencer, ArtboardReferencerBehavior},
    backboard::Backboard,
    component_dirt::ComponentDirt,
    core::{CoreHandle, CoreType},
    core_context::CoreContext,
    data_bind::data_context::RuntimeDataContextHandle,
    file::{RuntimeFileHandle, RuntimeFileWeakHandle},
    generated::text::text_style_global_base::{TextStyleGlobalBase, TextStyleGlobalBaseCallbacks},
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    status_code::StatusCode,
    viewmodel::viewmodel_instance::ViewModelInstance,
};
use std::{cell::RefCell, rc::Rc};
#[derive(Default)]
pub struct TextStyleGlobal {
    pub base: TextStyleGlobalBase,
    pub(crate) referencer: ArtboardReferencer,
    instance: Option<Rc<RefCell<TextStyleInstance>>>,
    shared: SharedTextStyleLease,
    paint: Option<CoreHandle>,
    info: TextStyleShareInfo,
    file: RuntimeFileWeakHandle,
    source_artboard: Option<CoreHandle>,
    instance_source: Option<CoreHandle>,
    child_view_model_instance: Option<CoreHandle>,
    bound_view_model_instance: Option<CoreHandle>,
    cross_file: Option<RuntimeFileHandle>,
    host_data_context: Option<RuntimeDataContextHandle>,
    list_item: Option<CoreHandle>,
}
impl std::ops::Deref for TextStyleGlobal {
    type Target = TextStyleGlobalBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextStyleGlobal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl TextStyleGlobal {
    pub const TYPE_KEY: u16 = TextStyleGlobalBase::TYPE_KEY;
    pub fn set_artboard_id(&mut self, value: u32) {
        if self.base.set_artboard_id_value(value) {
            self.base
                .base
                .notify_property_changed(TextStyleGlobalBase::ARTBOARD_ID_PROPERTY_KEY);
        }
    }
    pub fn set_file(&mut self, file: RuntimeFileWeakHandle) {
        self.file = file;
    }
    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.file.clone()
    }
    pub fn set_referenced_artboard(&mut self, source: Option<CoreHandle>) {
        self.referencer.set_referenced_artboard(source.clone());
        self.source_artboard = source;
    }
    pub fn first_text_style_paint(source: Option<&CoreHandle>) -> Option<CoreHandle> {
        source?.with_downcast::<Artboard,_>(|a|a.objects().iter().flatten().find(|o|o.is_type_of(crate::source::generated::text::text_style_paint_base::TextStylePaintBase::TYPE_KEY)).cloned()).flatten()
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<BackboardImporter>(Backboard::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_artboard_referencer(this);
        self.base.import(stack)
    }
    pub fn clone_value(&self) -> Box<Self> {
        let mut clone = Box::new(Self::default());
        let mut base = std::mem::take(&mut clone.base);
        base.copy(&self.base, clone.as_mut());
        clone.base = base;
        clone.file = self.file.clone();
        clone.set_referenced_artboard(self.source_artboard.clone());
        clone
    }
    fn release_style(&mut self) {
        self.paint = None;
        self.instance = None;
        self.shared.release();
        self.instance_source = None;
    }
    fn acquire_style(
        &mut self,
        source: Option<CoreHandle>,
        bound: Option<CoreHandle>,
        source_file: Option<RuntimeFileHandle>,
    ) {
        let host = self.base.artboard_handle();
        let text = <dyn TextInterface>::from_core(self.base.parent_handle());
        if self.instance_source.is_some() {
            if let Some(text) = &text {
                text.with_mut(|t| t.text_interface_style_reference_will_change());
            }
        }
        self.release_style();
        self.info = TextStyleShareInfo::default();
        self.bound_view_model_instance = bound.clone();
        let file = self.file.upgrade();
        let owned = source_file
            .as_ref()
            .is_none_or(|source| file.as_ref().is_some_and(|file| source.ptr_eq(file)));
        self.cross_file = if owned { None } else { source_file };
        let (Some(source), Some(host)) = (source, host) else {
            return;
        };
        let factory = host
            .with_downcast::<Artboard, _>(|h| h.factory())
            .flatten()
            .expect("style host factory");
        self.instance_source = Some(source.clone());
        let vm_file = if owned {
            file.clone()
        } else {
            self.cross_file.clone()
        };
        let info = if owned && file.is_some() {
            file.as_ref()
                .unwrap()
                .with_file(|f| f.text_style_shares())
                .borrow_mut()
                .info(&source)
        } else {
            TextStyleShareCache::analyze(&source, vm_file.as_ref())
        };
        self.info = info;
        if bound.is_none() && info.shareable && text.is_some() {
            if let Some(file) = &file {
                self.shared = file
                    .with_file(|f| f.text_style_shares())
                    .borrow_mut()
                    .acquire(&source, factory.clone(), text.clone());
                if let Some(shared) = self.shared.get() {
                    self.paint = shared.paint();
                    return;
                }
            }
        }
        let mut vm = bound;
        let id = source
            .with_downcast::<Artboard, _>(|a| a.base.view_model_id())
            .unwrap();
        if vm.is_none() {
            if let Some(child) = &self.child_view_model_instance {
                if child
                    .with_downcast::<ViewModelInstance, _>(|v| v.base.view_model_id() == id)
                    .unwrap_or(false)
                {
                    vm = Some(child.clone());
                }
            }
        }
        if vm.is_none() {
            if let Some(file) = &vm_file {
                vm = file.view_model(id as usize).and_then(|model| {
                    file.with_file(|f| f.create_default_view_model_instance(model))
                });
                if owned {
                    if let Some(vm) = &vm {
                        file.complete_view_model_properties(vm);
                    }
                }
            }
        }
        let served = text
            .as_ref()
            .filter(|t| {
                info.listens_to_runs
                    && t.is_type_of(crate::source::generated::text::text_base::TextBase::TYPE_KEY)
            })
            .cloned();
        let mut instance = TextStyleInstance::new(
            &source,
            factory,
            vm,
            info.reads_outside_own_view_model,
            text,
            served,
            self.base.handle().map(TextStyleReferenceWeak::Authored),
        );
        self.paint = instance.paint();
        instance.list_item(self.list_item.clone());
        instance.bind(self.host_data_context.clone(), false);
        let artboard = instance.artboard_instance();
        artboard.update_data_binds(false);
        Artboard::update_pass_handle(&artboard.core_handle(), false);
        self.instance = Some(Rc::new(RefCell::new(instance)));
    }
    pub fn update_artboard_occurrence(owner: &CoreHandle, property: Option<CoreHandle>) {
        let changed = owner
            .with_mut(|o| {
                o.as_text_style_global_mut()
                    .unwrap()
                    .update_artboard(property)
            })
            .unwrap_or(false);
        if changed {
            let text = owner.with(|o| o.component_parent_handle()).flatten();
            if let Some(text) = <dyn TextInterface>::from_core(text) {
                if text.is_type_of(super::text::Text::TYPE_KEY) {
                    super::text::Text::style_reference_did_change_occurrence(&text);
                } else {
                    text.with_mut(|text| text.text_interface_mark_shape_dirty());
                }
            }
        }
    }
    fn update_artboard(&mut self, property: Option<CoreHandle>) -> bool {
        let (Some(property), Some(host)) = (property, self.base.artboard_handle()) else {
            return false;
        };
        if !host
            .with_downcast::<Artboard, _>(Artboard::is_instance)
            .unwrap_or(false)
        {
            return false;
        }
        let Some((asset, index, bound)) = property
            .with(|p| {
                p.as_view_model_instance_artboard().map(|p| {
                    (
                        p.asset(),
                        p.base.property_value(),
                        p.bound_view_model_instance(),
                    )
                })
            })
            .flatten()
        else {
            return false;
        };
        let null = asset.is_none() && index == u32::MAX;
        let target = if null {
            self.source_artboard.clone()
        } else {
            ArtboardReferencer::find_artboard(Some(property), Some(host), Some(self.file.clone()))
        };
        let Some(target) = target else {
            return false;
        };
        if Self::first_text_style_paint(Some(&target)).is_none() {
            return false;
        }
        let source = target
            .with_downcast::<Artboard, _>(Artboard::artboard_source_handle)
            .flatten();
        let bound = if null { None } else { bound };
        let source_file = if null {
            None
        } else {
            asset.and_then(|a| a.file()).and_then(|f| f.upgrade())
        };
        if self.paint.is_some()
            && self.instance_source == source
            && self.bound_view_model_instance == bound
        {
            return false;
        }
        self.acquire_style(source, bound, source_file);
        self.base.add_dirt(ComponentDirt::COMPONENTS, false);
        true
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.child_view_model_instance=self.base.children().iter().find(|c|c.is_type_of(crate::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::TYPE_KEY)).cloned();
        let code = self.base.on_added_clean(context);
        if self.base.artboard_handle().is_some_and(|h| {
            h.with_downcast::<Artboard, _>(Artboard::is_instance)
                .unwrap_or(false)
        }) && self.source_artboard.is_some()
        {
            if let Some(child) = &self.child_view_model_instance {
                self.file.complete_view_model_properties(child);
            }
            self.acquire_style(self.source_artboard.clone(), None, None);
        }
        code
    }
    pub fn instance_source(&self) -> Option<CoreHandle> {
        self.instance_source.clone()
    }
    pub fn is_shared(&self) -> bool {
        self.shared.get().is_some()
    }
    pub fn style_view_model_instance(&self) -> Option<CoreHandle> {
        self.instance
            .as_ref()
            .and_then(|i| i.borrow().view_model_instance())
    }
    pub fn artboard_instance(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.instance
            .as_ref()
            .map(|i| i.borrow().artboard_instance())
            .or_else(|| self.shared.get().and_then(|s| s.artboard_instance()))
    }
    pub fn resolved_paint(&self) -> Option<CoreHandle> {
        if self.paint.is_some() {
            return self.paint.clone();
        }
        let mut source = self.referencer.referenced_artboard();
        if cfg!(feature = "editor") && source.is_none() {
            source = self
                .base
                .artboard_handle()
                .and_then(|h| {
                    h.with_downcast::<Artboard, _>(|a| a.resolve(self.base.artboard_id()))
                        .flatten()
                })
                .filter(|s| {
                    s.is_type_of(crate::source::generated::artboard_base::ArtboardBase::TYPE_KEY)
                });
        }
        Self::first_text_style_paint(source.as_ref())
    }
    pub fn listens_to_runs(&self) -> bool {
        self.instance.is_some() && self.info.listens_to_runs
    }
    pub fn run_state_machine(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        self.listens_to_runs()
            .then(|| self.instance.as_ref().unwrap().borrow().state_machine())
            .flatten()
    }
    pub fn quiet_state(&self) -> QuietState {
        if self.instance.is_none() {
            QuietState::Quiet
        } else if self.bound_view_model_instance.is_some() {
            QuietState::Never
        } else {
            self.instance.as_ref().unwrap().borrow().quiet_state()
        }
    }
    fn instance_handle(owner: &CoreHandle) -> Option<Rc<RefCell<TextStyleInstance>>> {
        owner
            .with(|o| o.as_text_style_global().and_then(|g| g.instance.clone()))
            .flatten()
    }
    pub fn host_data_context_occurrence(
        owner: &CoreHandle,
        value: Option<RuntimeDataContextHandle>,
    ) {
        let i = owner
            .with_mut(|o| {
                let g = o.as_text_style_global_mut().unwrap();
                g.host_data_context = value.clone();
                g.instance.clone()
            })
            .flatten();
        if let Some(i) = i {
            let mut i = i.borrow_mut();
            i.bind(value, false);
            i.flush();
        }
    }
    pub fn relink_host_data_context_occurrence(owner: &CoreHandle) {
        if let Some(i) = Self::instance_handle(owner) {
            i.borrow_mut().relink();
        }
    }
    pub fn update_hosted_data_binds_occurrence(owner: &CoreHandle) {
        if let Some(i) = Self::instance_handle(owner) {
            i.borrow_mut().flush();
        }
    }
    pub fn advance_component_occurrence(
        owner: &CoreHandle,
        seconds: f32,
        flags: AdvanceFlags,
    ) -> bool {
        Self::instance_handle(owner).is_some_and(|i| i.borrow_mut().advance(seconds, flags))
    }
    pub fn list_item_occurrence(owner: &CoreHandle, value: Option<CoreHandle>) {
        let i = owner
            .with_mut(|o| {
                let Some(g) = o.as_text_style_global_mut() else {
                    return None;
                };
                g.list_item = value.clone();
                g.instance.clone()
            })
            .flatten();
        if let Some(i) = i {
            i.borrow_mut().list_item(value);
        }
    }
}
impl Drop for TextStyleGlobal {
    fn drop(&mut self) {
        self.release_style();
        self.bound_view_model_instance = None;
        self.host_data_context = None;
    }
}
impl TextStyleGlobalBaseCallbacks for TextStyleGlobal {
    fn notify_property_changed(&mut self, key: u16) {
        self.base.base.notify_property_changed(key);
    }
}
impl ArtboardReferencerBehavior for TextStyleGlobal {
    fn artboard_referencer(&self) -> &ArtboardReferencer {
        &self.referencer
    }
    fn artboard_referencer_mut(&mut self) -> &mut ArtboardReferencer {
        &mut self.referencer
    }
    fn referenced_artboard_id(&self) -> i32 {
        self.base.artboard_id() as i32
    }
    fn set_referenced_artboard(&mut self, a: Option<CoreHandle>) {
        TextStyleGlobal::set_referenced_artboard(self, a)
    }
}
