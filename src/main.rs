use image::{ImageBuffer, Rgba};
use image_examples::create_simple_image::{generate_simple_image, ImageSizeInfo};
use vulkano::sync::GpuFuture;
use vulkano::{VulkanLibrary, sync};
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, ClearColorImageInfo, CopyImageToBufferInfo};
use vulkano::device::{DeviceCreateInfo, QueueCreateInfo, Device};
use vulkano::format::Format;
use vulkano::image::{StorageImage, ImageDimensions};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{StandardMemoryAllocator, GenericMemoryAllocator};
use vulkano::pipeline::graphics;

mod image_examples;

use bytemuck::{Pod, Zeroable};

/*
    https://vulkano.rs/guide
*/

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable,  Pod)]
struct MyStruct {
    a: u32,
    b: u32
}

fn main() {
    let library = VulkanLibrary::new().expect("Local Vulkan lib not found");
    let instance = Instance::new(library, InstanceCreateInfo::default()).expect("Failed to create instance");

    let physical = instance
        .enumerate_physical_devices()
        .expect("Could not enumerate devices")
        .next()
        .expect("No devices available");

    let queue_family_index = physical
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_, q)| q.queue_flags.graphics)
        .expect("Could not find a graphical queue family") as u32;

    let (device, mut queues) = Device::new(
        physical,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    ).expect("Failed to create device");

    let queue = queues.next().unwrap(); // Now ready to ask the GPU to perform operations
    let memory_allocator = StandardMemoryAllocator::new_default(device.clone());
    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let image_size_info = ImageSizeInfo {
        width: 1024,
        height: 720,
    };

    generate_simple_image(device, &memory_allocator, &command_buffer_allocator, queue, image_size_info);

    println!("Done!");
}
