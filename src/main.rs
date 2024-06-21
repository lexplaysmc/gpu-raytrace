use std::mem::size_of;

use ash::vk::{self, AccessFlags, BufferCopy, BufferMemoryBarrier, CommandBufferBeginInfo, CommandBufferResetFlags, DependencyFlags, Fence, FenceCreateInfo, PipelineBindPoint, PipelineStageFlags, StructureType, SubmitInfo, WHOLE_SIZE};
use image::Rgba;
use vulkan::device::buffer::{StagedSSBO, StagedUBO};

pub mod vulkan;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1200;

fn main() {
    let vk = vulkan::VulkanHandle::new().unwrap();
    let physical = vulkan::device::PhysicalDevice::find_device(&vk).unwrap().unwrap();
    let logical = vulkan::device::LogicalDevice::create_logical_device(physical);
    let queue = logical.create_queue();
    let ubo = StagedUBO::new(&logical, vec![WIDTH as u32, HEIGHT as u32, 0, 0]);
    let mut ssbo = StagedSSBO::<f32>::new(&logical, WIDTH * HEIGHT * 4);
    let pipe = vulkan::device::shaders::Pipeline::new("shader.spv", &ubo, &ssbo, &logical);
    let cmd = logical.create_command_buffer();
    let info = CommandBufferBeginInfo {
        s_type: StructureType::COMMAND_BUFFER_BEGIN_INFO,
        ..Default::default()
    };
    let fence = FenceCreateInfo {
        s_type: StructureType::FENCE_CREATE_INFO,
        ..Default::default()
    };
    let fence = unsafe { logical.device.create_fence(&fence, None) }.unwrap();
    
    unsafe { logical.device.begin_command_buffer(cmd, &info) }.unwrap();
    unsafe { logical.device.cmd_copy_buffer(cmd, ubo.get_stage(), ubo.get_ubo(), &[BufferCopy { size: ubo.get_size() as u64, ..Default::default() }]) };
    unsafe { logical.device.end_command_buffer(cmd) }.unwrap();
    unsafe { logical.device.queue_submit(queue.queue, &[SubmitInfo{
        s_type: StructureType::SUBMIT_INFO,
        command_buffer_count: 1,
        p_command_buffers: &cmd,
        ..Default::default()
    }], fence) }.unwrap();
    unsafe { logical.device.wait_for_fences(&[fence], true, u64::MAX) }.unwrap();
    unsafe { logical.device.reset_fences(&[fence]) }.unwrap();
    unsafe { logical.device.queue_wait_idle(queue.queue) }.unwrap();
    unsafe { logical.device.reset_command_buffer(cmd, CommandBufferResetFlags::default()) }.unwrap();

    unsafe { logical.device.begin_command_buffer(cmd, &info) }.unwrap();
    unsafe { logical.device.cmd_bind_pipeline(cmd, PipelineBindPoint::COMPUTE, pipe.pipeline) };
    unsafe { logical.device.cmd_bind_descriptor_sets(cmd, PipelineBindPoint::COMPUTE, pipe.layout, 0, &[pipe.descriptor_set], &[]) };
    unsafe { logical.device.cmd_dispatch(cmd, (WIDTH as f32/32.0).ceil() as u32, (HEIGHT as f32/32.0).ceil() as u32, 1) };
    unsafe { logical.device.end_command_buffer(cmd) }.unwrap();
    unsafe { logical.device.queue_submit(queue.queue, &[SubmitInfo{
        s_type: StructureType::SUBMIT_INFO,
        command_buffer_count: 1,
        p_command_buffers: &cmd,

        ..Default::default()
    }], fence) }.unwrap();
    unsafe { logical.device.wait_for_fences(&[fence], true, u64::MAX) }.unwrap();
    unsafe { logical.device.reset_fences(&[fence]) }.unwrap();
    unsafe { logical.device.queue_wait_idle(queue.queue) }.unwrap();
    unsafe { logical.device.reset_command_buffer(cmd, CommandBufferResetFlags::default()) }.unwrap();

    // unsafe { logical.device.begin_command_buffer(cmd, &info) }.unwrap();
    // unsafe { logical.device.cmd_pipeline_barrier(cmd, PipelineStageFlags::COMPUTE_SHADER, PipelineStageFlags::TRANSFER, DependencyFlags::empty(), &[], &[BufferMemoryBarrier {
    //     s_type: StructureType::BUFFER_MEMORY_BARRIER,
    //     src_access_mask: AccessFlags::SHADER_WRITE,
    //     dst_access_mask: AccessFlags::TRANSFER_READ,
    //     src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    //     dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    //     buffer: ssbo.get_ssbo(),
    //     offset: 0,
    //     size: WHOLE_SIZE,
    //     ..Default::default()
    // }], &[]) }
    // unsafe { logical.device.end_command_buffer(cmd) }.unwrap();
    // unsafe { logical.device.queue_submit(queue.queue, &[SubmitInfo{
    //     s_type: StructureType::SUBMIT_INFO,
    //     command_buffer_count: 1,
    //     p_command_buffers: &cmd,
    //     ..Default::default()
    // }], fence) }.unwrap();
    // unsafe { logical.device.wait_for_fences(&[fence], true, u64::MAX) }.unwrap();
    // unsafe { logical.device.reset_fences(&[fence]) }.unwrap();
    // unsafe { logical.device.queue_wait_idle(queue.queue) }.unwrap();
    // unsafe { logical.device.reset_command_buffer(cmd, CommandBufferResetFlags::default()) }.unwrap();

    unsafe { logical.device.begin_command_buffer(cmd, &info) }.unwrap();
    unsafe { logical.device.cmd_copy_buffer(cmd, ssbo.get_ssbo(), ssbo.get_stage(), &[BufferCopy { src_offset: 0, dst_offset: 0, size: ssbo.get_size() as u64 }]) }
    unsafe { logical.device.end_command_buffer(cmd) }.unwrap();
    unsafe { logical.device.queue_submit(queue.queue, &[SubmitInfo{
        s_type: StructureType::SUBMIT_INFO,
        command_buffer_count: 1,
        p_command_buffers: &cmd,
        ..Default::default()
    }], fence) }.unwrap();
    unsafe { logical.device.wait_for_fences(&[fence], true, u64::MAX) }.unwrap();
    unsafe { logical.device.reset_fences(&[fence]) }.unwrap();
    unsafe { logical.device.queue_wait_idle(queue.queue) }.unwrap();
    let image = Vec::from_iter(ssbo.get_slice().iter().map(|x| (x*255.0) as u8));
    image::ImageBuffer::<Rgba<u8>, _>::from_vec(WIDTH as u32, HEIGHT as u32, image).unwrap().save("img.png").map_err(|x| x.to_string()).unwrap();

    unsafe { logical.device.destroy_fence(fence, None) };
}