use std::{marker::PhantomData, mem::size_of, ptr::slice_from_raw_parts_mut};

use ash::vk::{self, Buffer, BufferCreateFlags, BufferUsageFlags, DeviceMemory, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements, SharingMode, StructureType};

use super::{LogicalDevice, PhysicalDevice};

pub struct StagedUBO<'a, 'b, T> {
    stage: vk::Buffer,
    ubo: vk::Buffer,
    stage_mem: vk::DeviceMemory,
    ubo_mem: vk::DeviceMemory,
    buffer: PhantomData<[T]>,
    device: &'a LogicalDevice<'b>,
    size: usize,
    mem_map: *mut T
}
impl<'a, 'b, T: Sized + Copy + 'a> StagedUBO<'a, 'b, T> {
    pub fn new(logical: &'a LogicalDevice<'b>, data: Vec<T>) -> Self {
        let size = size_of::<T>() * data.len();
        let stage_info = vk::BufferCreateInfo {
            s_type: StructureType::BUFFER_CREATE_INFO,
            size: size as u64,
            usage: BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let uniform_info = vk::BufferCreateInfo {
            s_type: StructureType::BUFFER_CREATE_INFO,
            size: size as u64,
            usage: BufferUsageFlags::UNIFORM_BUFFER | BufferUsageFlags::TRANSFER_DST,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let stage = unsafe { logical.device.create_buffer(&stage_info, None).unwrap() };
        let ubo = unsafe { logical.device.create_buffer(&uniform_info, None).unwrap() };
        
        let stage_mem = alloc_vram(&logical.physical, logical, &unsafe { logical.device.get_buffer_memory_requirements(stage) }, &(MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT));
        let ubo_mem = alloc_vram(&logical.physical, logical, &unsafe { logical.device.get_buffer_memory_requirements(ubo) }, &MemoryPropertyFlags::DEVICE_LOCAL);

        unsafe { logical.device.bind_buffer_memory(stage, stage_mem, 0) }.unwrap();
        unsafe { logical.device.bind_buffer_memory(ubo, ubo_mem, 0) }.unwrap();

        let map = unsafe { logical.device.map_memory(stage_mem, 0, size as u64, MemoryMapFlags::empty()) }.unwrap().cast();
        let mut buff = Self { stage, stage_mem, ubo, ubo_mem, buffer: PhantomData, device: &logical, mem_map: map, size };
        buff.get_slice().copy_from_slice(&data);
        buff

    }
    pub fn get_slice<'c>(&'c mut self) -> &'c mut [T] {
        unsafe {
            &mut *slice_from_raw_parts_mut(self.mem_map, self.size/size_of::<T>())
        }
    }
    pub fn get_ubo(&self) -> Buffer {
        self.ubo
    }
    pub fn get_stage(&self) -> Buffer {
        self.stage
    }
    /// size IN BYTES
    pub fn get_size(&self) -> usize {
        self.size
    }
}
impl<T> Drop for StagedUBO<'_, '_, T> {
    fn drop(&mut self) {
        unsafe { self.device.device.unmap_memory(self.stage_mem) }

        unsafe { self.device.device.free_memory(self.ubo_mem, None) }
        unsafe { self.device.device.free_memory(self.stage_mem, None) }

        unsafe { self.device.device.destroy_buffer(self.ubo, None) }
        unsafe { self.device.device.destroy_buffer(self.stage, None) }
    }
}

pub struct StagedSSBO<'a, 'b, T> {
    stage: vk::Buffer,
    ssbo: vk::Buffer,
    stage_mem: vk::DeviceMemory,
    ssbo_mem: vk::DeviceMemory,
    buffer: PhantomData<[T]>,
    device: &'a LogicalDevice<'b>,
    size: usize,
    mem_map: *mut T
}
impl<'a, 'b, T: Sized + Copy + 'a> StagedSSBO<'a, 'b, T> {
    pub fn new(logical: &'a LogicalDevice<'b>, len: usize) -> Self {
        let size = size_of::<T>() * len;
        let stage_info = vk::BufferCreateInfo {
            s_type: StructureType::BUFFER_CREATE_INFO,
            size: size as u64,
            usage: BufferUsageFlags::TRANSFER_DST,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let ssbo_info = vk::BufferCreateInfo {
            s_type: StructureType::BUFFER_CREATE_INFO,
            size: size as u64,
            usage: BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_SRC,
            sharing_mode: SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let stage = unsafe { logical.device.create_buffer(&stage_info, None).unwrap() };
        let ssbo = unsafe { logical.device.create_buffer(&ssbo_info, None).unwrap() };
        
        let stage_mem = alloc_vram(&logical.physical, logical, &unsafe { logical.device.get_buffer_memory_requirements(stage) }, &(MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT));
        let ssbo_mem = alloc_vram(&logical.physical, logical, &unsafe { logical.device.get_buffer_memory_requirements(ssbo) }, &MemoryPropertyFlags::DEVICE_LOCAL);

        unsafe { logical.device.bind_buffer_memory(stage, stage_mem, 0) }.unwrap();
        unsafe { logical.device.bind_buffer_memory(ssbo, ssbo_mem, 0) }.unwrap();

        let map = unsafe { logical.device.map_memory(stage_mem, 0, size as u64, MemoryMapFlags::empty()) }.unwrap().cast();
        Self { stage, stage_mem, ssbo, ssbo_mem, buffer: PhantomData, device: &logical, mem_map: map, size }
    }
    pub fn get_slice<'c>(&'c mut self) -> &'c mut [T] {
        unsafe {
            &mut *slice_from_raw_parts_mut(self.mem_map, self.size/size_of::<T>())
        }
    }
    pub fn get_ssbo(&self) -> Buffer {
        self.ssbo
    }
    pub fn get_stage(&self) -> Buffer {
        self.stage
    }
    /// size IN BYTES
    pub fn get_size(&self) -> usize {
        self.size
    }
}
impl<T> Drop for StagedSSBO<'_, '_, T> {
    fn drop(&mut self) {
        unsafe { self.device.device.unmap_memory(self.stage_mem) }

        unsafe { self.device.device.free_memory(self.ssbo_mem, None) }
        unsafe { self.device.device.free_memory(self.stage_mem, None) }

        unsafe { self.device.device.destroy_buffer(self.ssbo, None) }
        unsafe { self.device.device.destroy_buffer(self.stage, None) }
    }
}


fn find_vram(device: &PhysicalDevice, type_filter: u32, props: &MemoryPropertyFlags) -> usize {
    let mem_props = unsafe { device.handle.instance.get_physical_device_memory_properties(device.device) };
    for memory in 0..mem_props.memory_type_count as usize {
        if (type_filter & (1<<memory)) != 0 && mem_props.memory_types[memory].property_flags == props.to_owned() {
            return memory;
        }
    }
    -1_isize as usize
}
fn alloc_vram(device: &PhysicalDevice, logical: &LogicalDevice, mem_reqs: &MemoryRequirements, props: &MemoryPropertyFlags) -> DeviceMemory {
    let info = MemoryAllocateInfo {
        s_type: StructureType::MEMORY_ALLOCATE_INFO,
        allocation_size: mem_reqs.size,
        memory_type_index: find_vram(device, mem_reqs.memory_type_bits, props) as u32,
        ..Default::default()
    };
    unsafe { logical.device.allocate_memory(&info, None).unwrap() }
}