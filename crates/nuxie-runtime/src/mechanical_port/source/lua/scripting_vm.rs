use std::{
    cell::{BorrowMutError, RefCell},
    fmt,
    rc::Rc,
};

/// Cloneable, runtime-neutral ownership for the concrete scripting backend.
///
/// `nuxie-runtime` owns only the VM trait. The host installs the
/// `nuxie-scripting` implementation behind this handle, avoiding a reverse
/// crate dependency while keeping the single-threaded VM and every mutable
/// call on one shared occurrence.
#[derive(Clone)]
pub struct RuntimeScriptingVmHandle {
    inner: Rc<RefCell<Box<dyn crate::scripting::ScriptingVm>>>,
}

impl RuntimeScriptingVmHandle {
    pub fn new(vm: Box<dyn crate::scripting::ScriptingVm>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(vm)),
        }
    }

    pub fn with_vm_mut<R>(
        &self,
        callback: impl FnOnce(&mut dyn crate::scripting::ScriptingVm) -> R,
    ) -> R {
        let mut vm = self.inner.borrow_mut();
        callback(vm.as_mut())
    }

    pub fn try_with_vm_mut<R>(
        &self,
        callback: impl FnOnce(&mut dyn crate::scripting::ScriptingVm) -> R,
    ) -> Result<R, BorrowMutError> {
        let mut vm = self.inner.try_borrow_mut()?;
        Ok(callback(vm.as_mut()))
    }

    pub fn install_render_factory(
        &self,
        factory: &crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    ) -> Result<(), crate::scripting::ScriptError> {
        self.with_vm_mut(|vm| {
            factory.with_factory_mut(|factory| vm.install_render_factory(factory))
        })
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl PartialEq for RuntimeScriptingVmHandle {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}

impl Eq for RuntimeScriptingVmHandle {}

impl fmt::Debug for RuntimeScriptingVmHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeScriptingVmHandle")
            .field("shared", &true)
            .finish()
    }
}

/// Transitional spelling used by fresh owners while their root call sites
/// move from borrowed concrete Luau state to the runtime-neutral handle.
pub type ScriptingVM = RuntimeScriptingVmHandle;
