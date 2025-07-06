use std::sync::Arc;
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::swapchain::Surface;
use vulkano::{VulkanLibrary, Version};
use crate::graphics::window::window_instance::create_window_bundle;



pub struct VulkanoBundle{
    pub instance: Arc<Instance>,
}

pub fn create_vulkano_bundle() -> VulkanoBundle{
    let window_bundle = create_window_bundle();
    let library = VulkanLibrary::new().expect("no local Vulkan library");
    let required_extensions = Surface::required_extensions(&window_bundle.event_loop);
     let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create instance");

    VulkanoBundle {
        instance,
    }
}


