use std::{io::Read, path::Path};

use super::{buffer::{StagedSSBO, StagedUBO}, LogicalDevice};
use ash::vk::{ComputePipelineCreateInfo, DescriptorBufferInfo, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo, DescriptorType, PipelineCache, PipelineLayout, PipelineLayoutCreateInfo, PipelineShaderStageCreateInfo, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags, StructureType, WriteDescriptorSet};

fn create_shader_module(code: &[u8], device: &LogicalDevice) -> ShaderModule {
    let info = ShaderModuleCreateInfo {
        s_type: StructureType::SHADER_MODULE_CREATE_INFO,
        code_size: code.len(),
        p_code: code.as_ptr().cast(),
        ..Default::default()
    };
    unsafe { device.device.create_shader_module(&info, None).unwrap() }
}
fn create_pipeline<A: Copy, B: Copy>(path: &Path, ubo_buffer: &StagedUBO<'_, '_, A>, ssbo_buffer: &StagedSSBO<'_, '_, B>, device: &LogicalDevice) -> (ash::vk::Pipeline, PipelineLayout, DescriptorSetLayout, DescriptorPool, DescriptorSet) {
    let mut buf = vec![];
    std::fs::File::open(path).unwrap().read_to_end(&mut buf).unwrap();
    let bindings = vec![DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: ShaderStageFlags::COMPUTE,
        ..Default::default()
    },
    DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
        stage_flags: ShaderStageFlags::COMPUTE,
        ..Default::default()
    }];
    let info = DescriptorSetLayoutCreateInfo {
        s_type: StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
        ..Default::default()
    };
    let descriptors = unsafe { device.device.create_descriptor_set_layout(&info, None) }.unwrap();
    let module = create_shader_module(&buf, device);
    let info = PipelineLayoutCreateInfo {
        s_type: StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        set_layout_count: 1,
        p_set_layouts: &descriptors,
        // push_constant_range_count: todo!(),
        // p_push_constant_ranges: todo!(),
        ..Default::default()
    };
    let layout = unsafe { device.device.create_pipeline_layout(&info, None) }.unwrap();
    let pool_sizes = vec![
        DescriptorPoolSize {
            ty: DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1
        },
        DescriptorPoolSize {
            ty: DescriptorType::STORAGE_BUFFER,
            descriptor_count: 1
        }
    ];
    let info = DescriptorPoolCreateInfo {
        s_type: StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        max_sets: 2,
        pool_size_count: pool_sizes.len() as u32,
        p_pool_sizes: pool_sizes.as_ptr(),
        ..Default::default()
    };
    let descriptor_pool = unsafe { device.device.create_descriptor_pool(&info, None) }.unwrap();
    let info = DescriptorSetAllocateInfo {
        s_type: StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        descriptor_pool,
        descriptor_set_count: 1,
        p_set_layouts: &descriptors,
        ..Default::default()
    };
    let descriptor_set = unsafe { device.device.allocate_descriptor_sets(&info) }.unwrap()[0];
    unsafe { device.device.update_descriptor_sets(&[
        WriteDescriptorSet {
            s_type: StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: descriptor_set,
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: DescriptorType::UNIFORM_BUFFER,
            p_buffer_info: &DescriptorBufferInfo {
                buffer: ubo_buffer.get_ubo(),
                offset: 0,
                range: ubo_buffer.get_size() as u64,
            },
            ..Default::default()
        },
        WriteDescriptorSet {
            s_type: StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: descriptor_set,
            dst_binding: 1,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: DescriptorType::STORAGE_BUFFER,
            p_buffer_info: &DescriptorBufferInfo {
                buffer: ssbo_buffer.get_ssbo(),
                offset: 0,
                range: ssbo_buffer.get_size() as u64,
            },
            ..Default::default()
        }
    ], &[]) };
    let name = "main\x00";
    let info = [ComputePipelineCreateInfo {
        s_type: StructureType::COMPUTE_PIPELINE_CREATE_INFO,
        stage: PipelineShaderStageCreateInfo {
            s_type: StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            stage: ShaderStageFlags::COMPUTE,
            module,
            p_name: name.as_ptr().cast(),
            ..Default::default()
        },
        layout,
        ..Default::default()
    }];
    let pipeline = unsafe { device.device.create_compute_pipelines(PipelineCache::null(), &info, None) }.unwrap()[0];
    unsafe { device.device.destroy_shader_module(module, None) };
    (pipeline, layout, descriptors, descriptor_pool, descriptor_set)
}

pub struct Pipeline<'a> {
    pub pipeline: ash::vk::Pipeline,
    pub layout: PipelineLayout,
    pub descriptors: DescriptorSetLayout,
    pub descriptor_pool: DescriptorPool,
    pub descriptor_set: DescriptorSet,
    owner: &'a LogicalDevice<'a>
}
impl<'a> Pipeline<'a> {
    pub fn new<T: AsRef<Path>, A: Copy, B: Copy>(path: T, ubo_buffer: &StagedUBO<'_, '_, A>, ssbo_buffer: &StagedSSBO<'_, '_, B>, device: &'a LogicalDevice) -> Self {
        let (pipeline, layout, descriptors, descriptor_pool, descriptor_set) = create_pipeline(path.as_ref(), ubo_buffer, ssbo_buffer, device);
        Self { pipeline, layout, descriptors, owner: device, descriptor_pool, descriptor_set }
    }
}
impl Drop for Pipeline<'_> {
    fn drop(&mut self) {
        unsafe {
            self.owner.device.free_descriptor_sets(self.descriptor_pool, &[self.descriptor_set]).unwrap();
            self.owner.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.owner.device.destroy_pipeline_layout(self.layout, None);
            self.owner.device.destroy_descriptor_set_layout(self.descriptors, None);
            self.owner.device.destroy_pipeline(self.pipeline, None);
        };
    }
}