//! Android window ownership. Swapchain work retires before the surface, and
//! the surface retires before its parent instance/device in the product root.
use super::surface_config::SurfaceConfig;
use super::surface_swapchain::SurfaceSwapchain;
use crate::RendererError;
use ash::vk;
use std::ffi::c_void;
use std::ptr::NonNull;

pub(super) struct AndroidSurface {
    loader: ash::khr::surface::Instance,
    handle: vk::SurfaceKHR,
    pub config: Option<SurfaceConfig>,
    pub swapchain: Option<SurfaceSwapchain>,
}

impl AndroidSurface {
    /// # Safety
    /// The window must be a valid ANativeWindow, with no other graphics producer
    /// attached. All presentation extensions and maintenance1 feature must be
    /// enabled. Instance/device must outlive this object; calls are lane-confined.
    /// Vulkan retains the window on successful creation until surface destruction.
    pub(super) unsafe fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
        window: NonNull<c_void>,
        requested: vk::Extent2D,
        native_premultiplied_alpha: bool,
    ) -> Result<Self, RendererError> {
        let android = ash::khr::android_surface::Instance::new(entry, instance);
        let handle = unsafe {
            android.create_android_surface(
                &vk::AndroidSurfaceCreateInfoKHR::default().window(window.as_ptr().cast()),
                None,
            )
        }
        .map_err(|error| surface_error("create", error))?;
        // Install ownership before any fallible query so every rejection also
        // releases Vulkan's native-window reference.
        let mut owned = Self {
            loader: ash::khr::surface::Instance::new(entry, instance),
            handle,
            config: None,
            swapchain: None,
        };
        let supports_queue = unsafe {
            owned.loader.get_physical_device_surface_support(
                physical_device,
                queue_family_index,
                handle,
            )
        }
        .map_err(|error| surface_error("query queue support", error))?;
        if !supports_queue {
            return Err(RendererError::Unsupported(
                "Vulkan graphics queue presentation",
            ));
        }
        unsafe {
            owned.reconfigure(
                instance,
                device,
                physical_device,
                requested,
                native_premultiplied_alpha,
            )?;
        }
        Ok(owned)
    }

    /// # Safety
    /// Same parent handles and lane as creation. No outstanding frame may use
    /// the old swapchain. A failure leaves this surface attached but suspended.
    pub(super) unsafe fn reconfigure(
        &mut self,
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        requested: vk::Extent2D,
        native_premultiplied_alpha: bool,
    ) -> Result<(), RendererError> {
        self.swapchain.take();
        self.config = None;
        let capabilities = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(physical_device, self.handle)
        }
        .map_err(|error| surface_error("query capabilities", error))?;
        let formats = unsafe {
            self.loader
                .get_physical_device_surface_formats(physical_device, self.handle)
        }
        .map_err(|error| surface_error("query formats", error))?;
        let config = SurfaceConfig::choose(
            &capabilities,
            &formats,
            requested,
            native_premultiplied_alpha,
        )?;
        if let Some(config) = config {
            let swapchain = unsafe { SurfaceSwapchain::new(instance, device, self.handle, config) }
                .map_err(|error| surface_error("create swapchain", error))?;
            self.swapchain = Some(swapchain);
            self.config = Some(config);
        }
        Ok(())
    }
}

impl Drop for AndroidSurface {
    fn drop(&mut self) {
        self.swapchain.take();
        unsafe { self.loader.destroy_surface(self.handle, None) };
    }
}

fn surface_error(operation: &str, error: vk::Result) -> RendererError {
    RendererError::Device(format!("Android Vulkan surface {operation}: {error:?}"))
}
