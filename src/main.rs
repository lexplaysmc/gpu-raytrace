use std::{mem::size_of, ptr::slice_from_raw_parts, sync, thread};

use ash::vk::{BufferCopy, CommandBufferBeginInfo, CommandBufferResetFlags, FenceCreateInfo, PipelineBindPoint, StructureType, SubmitInfo};
use image::Rgba;
use vulkan::device::buffer::{StagedSSBO, StagedUBO};

pub mod vulkan;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1200;

#[repr(C)]
struct UBOData {
    size: [u32; 8],
    pos: [f64; 2],
    zoom: f64,
    gamma: f64
}
impl<'a> UBOData {
    fn new(size: [usize; 2], pos: [f64; 2], zoom: f64, gamma: f64) -> Self {
        Self { size: [size[0] as u32, size[1] as u32, 0, 0, 0, 0, 0, 0], pos: [pos[0], pos[1]], zoom, gamma }
    }
    fn bytes(&'a self) -> &'a [u8] {
        unsafe { &*slice_from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }
}

fn main() {
    let vk = vulkan::VulkanHandle::new().unwrap();
    let physical = vulkan::device::PhysicalDevice::find_device(&vk).unwrap().unwrap();
    let logical = vulkan::device::LogicalDevice::create_logical_device(physical);
    let queue = logical.create_queue();
    //-1.768800555, -0.001768198
    //-0.7454607800914831, 0.09635969753181201
    //0.1360592161152, 0.6257839882281
    let mut ubodata = UBOData::new([WIDTH, HEIGHT], [-0.63981440099510262, -0.40987916004273033], 0.0000001, 2.0);
    let mut ubo = StagedUBO::new(&logical, ubodata.bytes().to_vec());
    let mut ssbo = StagedSSBO::<u8>::new(&logical, WIDTH * HEIGHT * 4);
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
    
    let (send, recv) = sync::mpsc::sync_channel(5);
    let recv = recv;
    let thread = thread::spawn(move || {
        for (x, image) in recv {
            image::ImageBuffer::<Rgba<u8>, _>::from_vec(WIDTH as u32, HEIGHT as u32, image).unwrap().save(format!("frames\\img{x}.png")).map_err(|x| x.to_string()).unwrap();
        }
    });

    let frames = 500;
    for x in 0usize..frames {
        ubodata.gamma = lerp(0.5, 2.0, x as f64/(frames-1) as f64);
        ubodata.zoom = lerp(3.0f64.ln(), 0.00000000001f64.ln(), x as f64/(frames-1) as f64).exp();
        ubo.get_slice().copy_from_slice(&ubodata.bytes());
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
        let mut image = Vec::with_capacity(WIDTH*HEIGHT*4);
        unsafe { image.set_len(ssbo.get_slice().len()) };
        image.copy_from_slice(ssbo.get_slice());
        send.send((x, image)).unwrap();
    }

    unsafe { logical.device.destroy_fence(fence, None) };
    drop(send);
    thread.join().unwrap();
}

fn lerp(start: f64, end: f64, t: f64) -> f64 {
    (end-start)*t + start
}