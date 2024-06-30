use std::ptr::slice_from_raw_parts;

use ash::vk::{BufferCopy, CommandBufferBeginInfo, CommandBufferResetFlags, FenceCreateInfo, PipelineBindPoint, StructureType, SubmitInfo};
use image::Rgba;
use rand::RngCore;
use vulkan::device::buffer::{StagedSSBO, StagedUBO};

pub mod vulkan;
mod obj;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1200;

#[repr(C)]
struct UBOData {
    size: [u32; 4],
    cam: [f32; 8],
    spheres: Vec<f32>,
}
impl<'a> UBOData {
    fn new(size: [usize; 2], spheres: Vec<f32>, count: usize, seed: u32, vfov: f32, lookfrom: [f32; 3], lookat: [f32; 3]) -> Self {
        Self { size: [size[0] as u32, size[1] as u32, count as u32, seed], spheres, cam: [lookfrom[0], lookfrom[1], lookfrom[2], vfov.to_radians(), lookat[0], lookat[1], lookat[2], 0.0]}
    }
    fn vec(&self) -> Vec<u8> {
        let mut vec = unsafe { &*slice_from_raw_parts(self as *const Self as *const u8, 16+32) }.to_vec();
        vec.extend_from_slice(unsafe { &*slice_from_raw_parts(self.spheres.as_ptr() as *const u8, 4*self.spheres.len()) });
        vec
    }
}

fn main() {
    let vk = vulkan::VulkanHandle::new().unwrap();
    let physical = vulkan::device::PhysicalDevice::find_device(&vk).unwrap().unwrap();
    let logical = vulkan::device::LogicalDevice::create_logical_device(physical);
    let queue = logical.create_queue();

    let mut dat = vec![0.0, 0.0, -1.0, 0.5, 0.0, -1000.5, -1.0, 1000.0, -1.0, 0.0, -1.0, 0.5, 1.0, 0.0, -1.0, 0.5];
    dat.resize(64*4, 0.0);
    dat.extend_from_slice(&[0.1, 0.2, 0.5, f32::INFINITY, 0.8, 0.8, 0.0, f32::INFINITY, 1.5, f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0, 1.0, 1.0, 0.0]);
    let mut ubodata = UBOData::new([WIDTH, HEIGHT], dat, 4, rand::thread_rng().next_u32(), 30.0, [-3.0, 2.0, 1.0], [0.0, 0.0, -1.0]);

    let mut ubo = StagedUBO::new(&logical, ubodata.vec());
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
    
    let samples = 100;
    for x in 0usize..samples {
        ubodata.size[3] = rand::thread_rng().next_u32();
        ubo.get_slice().copy_from_slice(&ubodata.vec());

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
        if (x+1)%10 == 0 {
            println!("{}", x+1);
        }
    }
    println!("{:?}", &ssbo.get_slice()[0..20]);
    let image = Vec::from_iter(ssbo.get_slice().iter().map(|x| (*x*255.0/samples as f32) as u8));
    image::ImageBuffer::<Rgba<u8>, _>::from_vec(WIDTH as u32, HEIGHT as u32, image).unwrap().save(format!("img.png")).map_err(|x| x.to_string()).unwrap();

    unsafe { logical.device.destroy_fence(fence, None) };
}