//! Public vulkan_bootstrap_enumeration_test.cpp at upstream 269ebcab.
//! Calls the real bootstrap through a scoped stand-in Vulkan loader.
use super::*;
use std::cell::RefCell;
use std::ffi::c_char;

#[derive(Clone, Debug)]
struct EnumerationCall {
    command: &'static str,
    queried_count: bool,
}
struct Loader {
    failing_call: Option<usize>,
    failure: vk::Result,
    calls: Vec<EnumerationCall>,
    created_instance: bool,
    created_device: bool,
    enabled_device_extensions: Vec<String>,
}
impl Default for Loader {
    fn default() -> Self {
        Self {
            failing_call: None,
            failure: vk::Result::ERROR_INITIALIZATION_FAILED,
            calls: Vec::new(),
            created_instance: false,
            created_device: false,
            enabled_device_extensions: Vec::new(),
        }
    }
}
thread_local! { static LOADER: RefCell<Loader> = RefCell::new(Loader::default()); }

unsafe fn enumerate<T: Copy>(
    command: &'static str,
    available: &[T],
    count: *mut u32,
    properties: *mut T,
) -> vk::Result {
    let (fail, failure) = LOADER.with(|loader| {
        let mut loader = loader.borrow_mut();
        let fail = loader.failing_call == Some(loader.calls.len());
        loader.calls.push(EnumerationCall {
            command,
            queried_count: properties.is_null(),
        });
        (fail, loader.failure)
    });
    if fail && failure != vk::Result::INCOMPLETE {
        return failure;
    }
    if properties.is_null() {
        unsafe {
            *count = available.len() as u32;
        }
        return vk::Result::SUCCESS;
    }
    let copied = unsafe { (*count as usize).min(available.len()) };
    unsafe {
        core::ptr::copy_nonoverlapping(available.as_ptr(), properties, copied);
        *count = copied as u32;
    }
    if fail || copied < available.len() {
        vk::Result::INCOMPLETE
    } else {
        vk::Result::SUCCESS
    }
}
fn extension(name: &CStr) -> vk::ExtensionProperties {
    let mut p = vk::ExtensionProperties::default();
    for (dst, &src) in p.extension_name.iter_mut().zip(name.to_bytes_with_nul()) {
        *dst = src as c_char;
    }
    p
}
unsafe extern "system" fn instance_extensions(
    _: *const c_char,
    count: *mut u32,
    properties: *mut vk::ExtensionProperties,
) -> vk::Result {
    unsafe {
        enumerate(
            "vkEnumerateInstanceExtensionProperties",
            &[extension(ash::khr::surface::NAME)],
            count,
            properties,
        )
    }
}
unsafe extern "system" fn instance_layers(
    count: *mut u32,
    properties: *mut vk::LayerProperties,
) -> vk::Result {
    let mut layer = vk::LayerProperties::default();
    for (dst, &src) in layer.layer_name.iter_mut().zip(b"VK_LAYER_rive_fake\0") {
        *dst = src as c_char;
    }
    unsafe {
        enumerate(
            "vkEnumerateInstanceLayerProperties",
            &[layer],
            count,
            properties,
        )
    }
}
unsafe extern "system" fn device_extensions(
    _: vk::PhysicalDevice,
    _: *const c_char,
    count: *mut u32,
    properties: *mut vk::ExtensionProperties,
) -> vk::Result {
    unsafe {
        enumerate(
            "vkEnumerateDeviceExtensionProperties",
            &[extension(ash::khr::swapchain::NAME)],
            count,
            properties,
        )
    }
}
unsafe extern "system" fn instance_version(version: *mut u32) -> vk::Result {
    unsafe {
        *version = vk::API_VERSION_1_1;
    }
    vk::Result::SUCCESS
}
unsafe extern "system" fn make_instance(
    _: *const vk::InstanceCreateInfo<'_>,
    _: *const vk::AllocationCallbacks<'_>,
    instance: *mut vk::Instance,
) -> vk::Result {
    LOADER.with(|loader| loader.borrow_mut().created_instance = true);
    unsafe {
        *instance = vk::Instance::from_raw(0x1000);
    }
    vk::Result::SUCCESS
}
unsafe extern "system" fn physical_devices(
    _: vk::Instance,
    count: *mut u32,
    devices: *mut vk::PhysicalDevice,
) -> vk::Result {
    unsafe {
        if !devices.is_null() {
            *devices = vk::PhysicalDevice::from_raw(0x2000);
        }
        *count = 1;
    }
    vk::Result::SUCCESS
}
unsafe extern "system" fn physical_properties(
    _: vk::PhysicalDevice,
    properties: *mut vk::PhysicalDeviceProperties,
) {
    let mut p = vk::PhysicalDeviceProperties::default();
    p.api_version = vk::API_VERSION_1_1;
    p.device_type = vk::PhysicalDeviceType::INTEGRATED_GPU;
    for (dst, &src) in p.device_name.iter_mut().zip(b"Rive fake driver\0") {
        *dst = src as c_char;
    }
    unsafe {
        *properties = p;
    }
}
unsafe extern "system" fn physical_features(
    _: vk::PhysicalDevice,
    features: *mut vk::PhysicalDeviceFeatures,
) {
    unsafe {
        *features = vk::PhysicalDeviceFeatures::default();
    }
}
// The Rust owner queries core 1.1 feature structures; preserve its zeroed
// extension chain and provide the same empty feature set as the source fake.
unsafe extern "system" fn physical_features2(
    _: vk::PhysicalDevice,
    features: *mut vk::PhysicalDeviceFeatures2<'_>,
) {
    unsafe {
        (*features).features = vk::PhysicalDeviceFeatures::default();
    }
}
unsafe extern "system" fn queue_families(
    _: vk::PhysicalDevice,
    count: *mut u32,
    properties: *mut vk::QueueFamilyProperties,
) {
    unsafe {
        if !properties.is_null() {
            *properties = vk::QueueFamilyProperties {
                queue_flags: vk::QueueFlags::GRAPHICS,
                queue_count: 1,
                ..Default::default()
            };
        }
        *count = 1;
    }
}
unsafe extern "system" fn make_device(
    _: vk::PhysicalDevice,
    info: *const vk::DeviceCreateInfo<'_>,
    _: *const vk::AllocationCallbacks<'_>,
    _: *mut vk::Device,
) -> vk::Result {
    let info = unsafe { &*info };
    let extensions = (0..info.enabled_extension_count as usize)
        .map(|i| unsafe {
            CStr::from_ptr(*info.pp_enabled_extension_names.add(i))
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    LOADER.with(|loader| {
        let mut loader = loader.borrow_mut();
        loader.created_device = true;
        loader.enabled_device_extensions = extensions;
    });
    vk::Result::ERROR_INITIALIZATION_FAILED
}
unsafe extern "system" fn destroy_instance(_: vk::Instance, _: *const vk::AllocationCallbacks<'_>) {
}

unsafe extern "system" fn get_instance_proc_addr(
    _: vk::Instance,
    name: *const c_char,
) -> vk::PFN_vkVoidFunction {
    let name = unsafe { CStr::from_ptr(name) }.to_bytes();
    macro_rules! entry {
        ($name:literal, $function:ident, $ty:ty) => {
            if name == $name {
                return Some(unsafe {
                    core::mem::transmute::<$ty, unsafe extern "system" fn()>($function as $ty)
                });
            }
        };
    }
    entry!(
        b"vkEnumerateInstanceExtensionProperties",
        instance_extensions,
        vk::PFN_vkEnumerateInstanceExtensionProperties
    );
    entry!(
        b"vkEnumerateInstanceLayerProperties",
        instance_layers,
        vk::PFN_vkEnumerateInstanceLayerProperties
    );
    entry!(
        b"vkEnumerateDeviceExtensionProperties",
        device_extensions,
        vk::PFN_vkEnumerateDeviceExtensionProperties
    );
    entry!(
        b"vkEnumerateInstanceVersion",
        instance_version,
        vk::PFN_vkEnumerateInstanceVersion
    );
    entry!(b"vkCreateInstance", make_instance, vk::PFN_vkCreateInstance);
    entry!(
        b"vkEnumeratePhysicalDevices",
        physical_devices,
        vk::PFN_vkEnumeratePhysicalDevices
    );
    entry!(
        b"vkGetPhysicalDeviceProperties",
        physical_properties,
        vk::PFN_vkGetPhysicalDeviceProperties
    );
    entry!(
        b"vkGetPhysicalDeviceFeatures",
        physical_features,
        vk::PFN_vkGetPhysicalDeviceFeatures
    );
    entry!(
        b"vkGetPhysicalDeviceFeatures2",
        physical_features2,
        vk::PFN_vkGetPhysicalDeviceFeatures2
    );
    entry!(
        b"vkGetPhysicalDeviceQueueFamilyProperties",
        queue_families,
        vk::PFN_vkGetPhysicalDeviceQueueFamilyProperties
    );
    entry!(b"vkCreateDevice", make_device, vk::PFN_vkCreateDevice);
    entry!(
        b"vkDestroyInstance",
        destroy_instance,
        vk::PFN_vkDestroyInstance
    );
    None
}
struct FakeLoader(Option<vk::PFN_vkGetInstanceProcAddr>);
impl FakeLoader {
    fn new() -> Self {
        LOADER.with(|loader| *loader.borrow_mut() = Loader::default());
        Self(
            TESTING_VK_GET_INSTANCE_PROC_ADDR
                .with(|slot| slot.replace(Some(get_instance_proc_addr))),
        )
    }
}
impl Drop for FakeLoader {
    fn drop(&mut self) {
        TESTING_VK_GET_INSTANCE_PROC_ADDR.with(|slot| slot.set(self.0));
    }
}
fn instance() -> Result<ash::Instance, RendererError> {
    create_instance(&load_vulkan_entry()?).map(|created| created.0)
}
fn device(
    instance: &ash::Instance,
    headless: bool,
) -> Result<(ash::Device, VulkanFeatures, bool, bool), RendererError> {
    let (physical, queue) = select_physical_device(instance)?;
    create_device(
        instance,
        physical,
        queue,
        vk::API_VERSION_1_1,
        false,
        headless,
    )
}

#[test]
fn vulkan_instance_is_created_when_enumeration_succeeds() {
    let _loader = FakeLoader::new();
    let instance = instance().unwrap();
    LOADER.with(|loader| assert!(loader.borrow().created_instance));
    unsafe {
        instance.destroy_instance(None);
    }
}

#[test]
fn vulkan_instance_creation_stops_at_a_failed_enumeration() {
    use vk::Result as R;
    for (failing_call, failure) in [
        (0, R::ERROR_INITIALIZATION_FAILED),
        (0, R::ERROR_OUT_OF_HOST_MEMORY),
        (1, R::ERROR_INITIALIZATION_FAILED),
        (1, R::ERROR_OUT_OF_HOST_MEMORY),
        (1, R::INCOMPLETE),
        (2, R::ERROR_INITIALIZATION_FAILED),
        (2, R::ERROR_OUT_OF_HOST_MEMORY),
        (3, R::ERROR_INITIALIZATION_FAILED),
        (3, R::ERROR_OUT_OF_HOST_MEMORY),
        (3, R::INCOMPLETE),
    ] {
        let _loader = FakeLoader::new();
        LOADER.with(|loader| {
            let mut loader = loader.borrow_mut();
            loader.failing_call = Some(failing_call);
            loader.failure = failure;
        });
        assert!(instance().is_err());
        LOADER.with(|loader| {
            let loader = loader.borrow();
            assert!(!loader.created_instance);
            assert_eq!(loader.calls.len(), failing_call + 1);
        });
    }
}

#[test]
fn vulkan_device_enables_the_extensions_it_needs() {
    let _loader = FakeLoader::new();
    let instance = instance().unwrap();
    let _ = device(&instance, false);
    LOADER.with(|loader| {
        let loader = loader.borrow();
        assert!(loader.created_device);
        assert_eq!(loader.enabled_device_extensions, ["VK_KHR_swapchain"]);
    });
    unsafe {
        instance.destroy_instance(None);
    }
}

#[test]
fn vulkan_device_carries_on_without_extensions_when_enumeration_fails() {
    use vk::Result as R;
    for (offset, failure) in [
        (0, R::ERROR_INITIALIZATION_FAILED),
        (0, R::ERROR_OUT_OF_HOST_MEMORY),
        (1, R::ERROR_INITIALIZATION_FAILED),
        (1, R::ERROR_OUT_OF_HOST_MEMORY),
        (1, R::INCOMPLETE),
    ] {
        for headless in [true, false] {
            let _loader = FakeLoader::new();
            let instance = instance().unwrap();
            let failing_call = LOADER.with(|loader| {
                let mut loader = loader.borrow_mut();
                let index = loader.calls.len() + offset;
                loader.failing_call = Some(index);
                loader.failure = failure;
                index
            });
            assert!(device(&instance, headless).is_err()); // fake vkCreateDevice intentionally fails
            LOADER.with(|loader| {
                let loader = loader.borrow();
                assert_eq!(loader.created_device, headless);
                if headless {
                    assert!(loader.enabled_device_extensions.is_empty());
                }
                assert_eq!(loader.calls.len(), failing_call + 1);
                let last = loader.calls.last().unwrap();
                assert_eq!(last.command, "vkEnumerateDeviceExtensionProperties");
                assert_eq!(last.queried_count, offset == 0);
            });
            unsafe {
                instance.destroy_instance(None);
            }
        }
    }
}
