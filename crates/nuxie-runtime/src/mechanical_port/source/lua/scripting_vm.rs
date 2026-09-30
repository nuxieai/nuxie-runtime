use std::{fmt, rc::Rc};

/// Cloneable, runtime-neutral ownership for the concrete scripting backend.
///
/// `nuxie-runtime` owns only the VM trait. The host installs the
/// `nuxie-scripting` implementation behind this handle, avoiding a reverse
/// crate dependency while keeping the single-threaded VM and every mutable
/// call on one shared occurrence.
#[derive(Clone)]
pub struct RuntimeScriptingVmHandle {
    inner: Rc<Box<dyn crate::scripting::ScriptingVm>>,
}

impl RuntimeScriptingVmHandle {
    #[cfg(feature = "tools")]
    pub fn dispose_orphan_scripted_properties(&self, all_tags: bool) {
        self.inner.dispose_orphan_scripted_properties(all_tags);
    }
    pub fn display_scale(&self) -> f32 {
        self.inner.script_backend().display_scale()
    }

    pub fn set_display_scale(&self, scale: f32) {
        self.inner.script_backend().set_display_scale(scale);
    }

    pub fn register_scripted_object(
        &self,
        object: crate::mechanical_port::source::core::CoreHandle,
    ) {
        self.inner.script_backend().register_scripted_object(object);
    }

    pub fn unregister_scripted_object(
        &self,
        object: &crate::mechanical_port::source::core::CoreHandle,
    ) {
        self.inner
            .script_backend()
            .unregister_scripted_object(object);
    }

    pub fn call_layout_resize(
        &self,
        instance: &mut dyn crate::scripting::ScriptInstance,
        size: crate::mechanical_port::source::math::vec2d::Vec2D,
        host: &mut dyn crate::scripting::ScriptHost,
    ) -> Result<crate::scripting::ScriptOptionalMethodResult, crate::scripting::ScriptError> {
        self.inner.call_layout_resize(instance, size, host)
    }
    pub fn new(vm: Box<dyn crate::scripting::ScriptingVm>) -> Self {
        Self { inner: Rc::new(vm) }
    }

    pub fn with_vm_mut<R>(
        &self,
        callback: impl FnOnce(&dyn crate::scripting::ScriptingVm) -> R,
    ) -> R {
        callback(self.inner.as_ref().as_ref())
    }

    pub fn install_render_factory(
        &self,
        factory: &crate::mechanical_port::source::factory::RuntimeFactoryHandle,
    ) -> Result<(), crate::scripting::ScriptError> {
        self.with_vm_mut(|vm| {
            factory.with_factory_mut(|factory| vm.install_render_factory(factory))
        })
    }

    pub fn poll_async_work(&self) -> Result<bool, crate::scripting::ScriptError> {
        self.with_vm_mut(|vm| vm.poll_async_work())
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
