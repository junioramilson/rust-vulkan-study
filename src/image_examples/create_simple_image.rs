use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use vulkano::{memory::allocator::StandardMemoryAllocator, device::{Queue, Device}, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, ClearColorImageInfo, CopyImageToBufferInfo}, image::{StorageImage, ImageDimensions}, format::{Format, ClearColorValue}, buffer::{CpuAccessibleBuffer, BufferUsage}, sync::{self, GpuFuture}};

pub struct ImageSizeInfo {
    pub width: u32,
    pub height: u32,
}

pub fn generate_simple_image(device: Arc<Device>, memory_allocator: &StandardMemoryAllocator, command_buffer_allocator: &StandardCommandBufferAllocator, queue: Arc<Queue>, image_size_info: ImageSizeInfo) {
    println!("Creating GPU Image");
    let gpu_image = StorageImage::new(
        memory_allocator,
        ImageDimensions::Dim2d { width: image_size_info.width, height: image_size_info.height, array_layers: 1 },
        Format::R8G8B8A8_UNORM,
        Some(queue.queue_family_index())
    ).unwrap();

    println!("Creating CPU buffer to receive GPU image content");
    let buffer_cpu = CpuAccessibleBuffer::from_iter(
        memory_allocator,
        BufferUsage {
            transfer_dst: true,
            ..Default::default()
        },
        false,
        (0..image_size_info.width * image_size_info.height * 4).map(|_| 0u8)
    ).expect("Failed to create buffer");

    let mut image_builder = AutoCommandBufferBuilder::primary(command_buffer_allocator, queue.queue_family_index(), CommandBufferUsage::OneTimeSubmit).unwrap();

    println!("Creating GPU Image builder");
    image_builder.clear_color_image(ClearColorImageInfo {
        clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 1.0]),
        ..ClearColorImageInfo::image(gpu_image.clone())
    }).unwrap()
    .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(gpu_image.clone(), buffer_cpu.clone())).unwrap();

    let command_buffer = image_builder.build().unwrap();

    println!("Building image on GPU");
    let gpu_execution_future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    gpu_execution_future.wait(None).unwrap();

    println!("Image built");

    let image_buffer_content = buffer_cpu.read().unwrap();
    let final_image = ImageBuffer::<Rgba<u8>, _>::from_raw(image_size_info.width, image_size_info.height, &image_buffer_content[..]).unwrap();

    println!("Saving final image to a file: result.png");

    final_image.save("result.png").unwrap();
}

