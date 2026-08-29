use super::*;
#[derive(Clone)]
pub struct RuntimeOwnedViewModelContext {
    context: RuntimeDataContextHandle,
    file: Option<RuntimeFileHandle>,
}
impl std::fmt::Debug for RuntimeOwnedViewModelContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeOwnedViewModelContext")
            .field(
                "instances",
                &self
                    .context
                    .with_context(|context| context.view_model_instances().to_vec()),
            )
            .finish()
    }
}
impl Default for RuntimeOwnedViewModelContext {
    fn default() -> Self {
        Self::new()
    }
}
impl RuntimeOwnedViewModelContext {
    pub fn new() -> Self {
        Self {
            context: RuntimeDataContextHandle::new(DataContext::new(None)),
            file: None,
        }
    }
    pub fn from_native(file: RuntimeFileHandle, context: RuntimeDataContextHandle) -> Self {
        Self {
            context,
            file: Some(file),
        }
    }
    pub fn from_main(main: RuntimeOwnedViewModelInstance) -> Self {
        Self::from_main_handle(RuntimeOwnedViewModelHandle::new(main))
    }
    pub fn from_main_handle(main: RuntimeOwnedViewModelHandle) -> Self {
        Self {
            context: RuntimeDataContextHandle::new(DataContext::new(Some(main.native_handle()))),
            file: Some(main.native_file()),
        }
    }
    pub fn native_handle(&self) -> RuntimeDataContextHandle {
        self.context.clone()
    }
    pub fn is_empty(&self) -> bool {
        self.context
            .with_context(|context| context.view_model_instances().is_empty())
    }
    pub fn main_handle(&self) -> Option<RuntimeOwnedViewModelHandle> {
        RuntimeOwnedViewModelHandle::from_native(
            self.file.clone()?,
            self.context
                .with_context(DataContext::main_view_model_instance)?,
        )
    }
    pub fn main(&self) -> Option<RuntimeOwnedViewModelInstance> {
        RuntimeOwnedViewModelInstance::from_native(
            self.file.clone()?,
            self.context
                .with_context(DataContext::main_view_model_instance)?,
        )
    }
    pub fn main_mut(&self) -> Option<RuntimeOwnedViewModelInstance> {
        self.main()
    }
    pub fn set_main(&mut self, main: RuntimeOwnedViewModelInstance) {
        self.set_main_handle(RuntimeOwnedViewModelHandle::new(main));
    }
    pub fn set_main_handle(&mut self, main: RuntimeOwnedViewModelHandle) {
        self.file = Some(main.native_file());
        self.context.with_context_mut(|context| {
            context.set_main_view_model_instance(Some(main.native_handle()))
        });
    }
    pub fn take_main(&mut self) -> Option<RuntimeOwnedViewModelHandle> {
        let main = self.main_handle()?;
        self.context
            .with_context_mut(DataContext::remove_main_view_model_instance);
        Some(main)
    }
    pub fn handles(&self) -> std::vec::IntoIter<RuntimeOwnedViewModelHandle> {
        let instances = self
            .context
            .with_context(|context| context.view_model_instances().to_vec());
        instances
            .into_iter()
            .flatten()
            .map(|instance| {
                RuntimeOwnedViewModelHandle::from_native(
                    self.file
                        .clone()
                        .expect("populated native context retains File"),
                    instance,
                )
                .expect("native context instance")
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
    pub fn instances(&self) -> std::vec::IntoIter<RuntimeOwnedViewModelInstance> {
        let instances = self
            .context
            .with_context(|context| context.view_model_instances().to_vec());
        instances
            .into_iter()
            .flatten()
            .map(|instance| {
                RuntimeOwnedViewModelInstance::from_native(
                    self.file
                        .clone()
                        .expect("populated native context retains File"),
                    instance,
                )
                .expect("native context instance")
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
    pub fn global_slot_handle(&self, slot: usize) -> Option<RuntimeOwnedViewModelHandle> {
        RuntimeOwnedViewModelHandle::from_native(
            self.file.clone()?,
            self.context
                .with_context(|context| context.instance_for_slot(slot as u32))?,
        )
    }
    pub fn global_slot(&self, slot: usize) -> Option<RuntimeOwnedViewModelInstance> {
        RuntimeOwnedViewModelInstance::from_native(
            self.file.clone()?,
            self.context
                .with_context(|context| context.instance_for_slot(slot as u32))?,
        )
    }
    pub fn global_slot_mut(&self, slot: usize) -> Option<RuntimeOwnedViewModelInstance> {
        self.global_slot(slot)
    }
    pub fn set_global_slot_handle(
        &mut self,
        slot: usize,
        instance: RuntimeOwnedViewModelHandle,
    ) -> bool {
        let Ok(slot) = u32::try_from(slot) else {
            return false;
        };
        self.file = Some(instance.native_file());
        self.context.with_context_mut(|context| {
            context.set_view_model_instance_for_slot(slot, Some(instance.native_handle()))
        });
        true
    }
    pub fn set_global_slot(
        &mut self,
        slot: usize,
        instance: RuntimeOwnedViewModelInstance,
    ) -> bool {
        self.set_global_slot_handle(slot, RuntimeOwnedViewModelHandle::new(instance))
    }
    pub fn set_parent(&self, parent: Option<&Self>) {
        self.context
            .with_context_mut(|context| context.set_parent(parent.map(Self::native_handle)));
    }
    pub fn parent(&self) -> Option<Self> {
        Some(Self {
            context: self.context.with_context(DataContext::parent)?,
            file: self.file.clone(),
        })
    }
}
#[derive(Clone, Debug)]
pub struct RuntimeOwnedViewModelContextHandle(Rc<RefCell<RuntimeOwnedViewModelContext>>);
impl RuntimeOwnedViewModelContextHandle {
    pub fn new(context: RuntimeOwnedViewModelContext) -> Self {
        Self(Rc::new(RefCell::new(context)))
    }
    pub fn borrow(&self) -> Ref<'_, RuntimeOwnedViewModelContext> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<'_, RuntimeOwnedViewModelContext> {
        self.0.borrow_mut()
    }
    pub fn native_handle(&self) -> RuntimeDataContextHandle {
        self.0.borrow().native_handle()
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.native_handle().ptr_eq(&other.native_handle())
    }
}
pub type RuntimeDataContext = RuntimeOwnedViewModelContext;
