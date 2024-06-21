pub mod buffer;
pub mod shaders;

use std::marker::PhantomData;

use ash::vk::{self, CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandPool, CommandPoolCreateInfo, DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDeviceFeatures, PhysicalDeviceType, QueueFlags, StructureType};
use super::VulkanHandle;

pub struct PhysicalDevice<'a> {
    device: vk::PhysicalDevice,
    handle: &'a VulkanHandle,
}
impl<'a> PhysicalDevice<'a> {
    pub fn find_device(handle: &'a VulkanHandle) -> Result<Option<Self>, vk::Result> {
        let device = unsafe {
            handle.instance.enumerate_physical_devices()
        }?.into_iter().map(|x| (x, score_device(x, &handle.instance))).filter(|x| x.1 >= 0).max_by_key(|x| x.1);
        match device {
            None => Ok(None),
            Some((device, _)) => Ok(Some(Self { device, handle }))
        }
    }
}

fn score_device(device: vk::PhysicalDevice, instance: &ash::Instance) -> i32 {
    let properties = unsafe {
        instance.get_physical_device_properties(device)
    };
    println!("{:?}", properties.limits.max_compute_work_group_size);
    println!("{:?}", properties.limits.max_compute_work_group_count);
    println!("{:?}", properties.limits.max_compute_work_group_invocations);
    if find_queue_family(device, instance).is_none() {
        return -1
    }
    let mut score = 0;
    score += match properties.device_type {
        PhysicalDeviceType::CPU | PhysicalDeviceType::OTHER => {0},
        PhysicalDeviceType::VIRTUAL_GPU => {1},
        PhysicalDeviceType::INTEGRATED_GPU => {2},
        PhysicalDeviceType::DISCRETE_GPU => {3},
        _ => {return -1;}
    };
    score
}
fn find_queue_family(device: vk::PhysicalDevice, instance: &ash::Instance) -> Option<usize> {
    let families = unsafe {
        instance.get_physical_device_queue_family_properties(device)
    };
    for x in families.into_iter().enumerate() {
        if x.1.queue_flags.contains(QueueFlags::COMPUTE) && x.1.queue_flags.contains(QueueFlags::TRANSFER) {
            return Some(x.0)
        }
    }
    None
}

pub struct LogicalDevice<'a> {
    pub device: ash::Device,
    queue: u32,
    physical: PhysicalDevice<'a>,
    pool: CommandPool,
}
impl Drop for LogicalDevice<'_> {
    fn drop(&mut self) {
        unsafe { self.device.device_wait_idle() }.unwrap();
        unsafe { self.device.destroy_command_pool(self.pool, None) };
        unsafe { self.device.destroy_device(None) }
    }
}
impl<'a> LogicalDevice<'a> {
    pub fn create_queue(&self) -> ComputeTransferQueue<'a> {
        ComputeTransferQueue {
            queue: unsafe {
                self.device.get_device_queue(self.queue, 0)
            },
            owner: PhantomData, 
        }
    }
    pub fn create_logical_device(physical: PhysicalDevice<'a>) -> LogicalDevice<'a> {
        let family = find_queue_family(physical.device, &physical.handle.instance).unwrap() as u32;
        let queue_info = DeviceQueueCreateInfo {
            s_type: StructureType::DEVICE_QUEUE_CREATE_INFO,
            queue_family_index: family,
            queue_count: 1,
            p_queue_priorities: [1.0f32].as_ptr(),
            ..Default::default()
        };
        let create_info = DeviceCreateInfo {
            s_type: StructureType::DEVICE_CREATE_INFO,
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_info,
            p_enabled_features: &PhysicalDeviceFeatures {
                ..Default::default()
            },
            ..Default::default()
        };
        let device = unsafe {
            physical.handle.instance.create_device(physical.device, &create_info, None)
        }.unwrap();
        let info = CommandPoolCreateInfo {
            s_type: StructureType::COMMAND_POOL_CREATE_INFO,
            queue_family_index: family,
            ..Default::default()
        };
        let pool = unsafe { device.create_command_pool(&info, None) }.unwrap();
        LogicalDevice { device, pool, queue: family, physical }
    }
    pub fn create_command_buffer(&self) -> CommandBuffer {
        let info = CommandBufferAllocateInfo {
            s_type: StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: self.pool,
            level: CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };
        unsafe { self.device.allocate_command_buffers(&info) }.unwrap()[0]
    }
}

pub struct ComputeTransferQueue<'a> {
    pub queue: ash::vk::Queue,
    owner: PhantomData<&'a LogicalDevice<'a>>
}