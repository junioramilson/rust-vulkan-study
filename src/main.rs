use image_examples::create_simple_image::{generate_simple_image, ImageSizeInfo};
use vulkano::{VulkanLibrary};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::device::{DeviceCreateInfo, QueueCreateInfo, Device};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{StandardMemoryAllocator};

mod image_examples;

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

    let queue = queues.next().unwrap();

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
