pub mod device;

use ash::vk;

#[allow(dead_code)]
pub struct VulkanHandle {
    entry: ash::Entry,
    instance: ash::Instance,
}
impl VulkanHandle {
    pub fn new() -> Result<Self, vk::Result> {
        let entry = unsafe {
            ash::Entry::load().map_err(|_| vk::Result::ERROR_UNKNOWN)?
        };
        
        let info = ash::vk::InstanceCreateInfo {
            s_type: ash::vk::StructureType::INSTANCE_CREATE_INFO,
            p_application_info: &ash::vk::ApplicationInfo {
                s_type: ash::vk::StructureType::APPLICATION_INFO,
                p_application_name: "\x00".as_ptr() as *const i8,
                application_version: ash::vk::make_api_version(0, 0, 0, 0),
                p_engine_name: "\x00".as_ptr() as *const i8,
                engine_version: ash::vk::make_api_version(0, 0, 0, 0),
                api_version: ash::vk::API_VERSION_1_0,
                ..Default::default()
            },
            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&info, None)? };
        Ok(Self {
            entry,
            instance
        })
    }
}
impl Drop for VulkanHandle {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}