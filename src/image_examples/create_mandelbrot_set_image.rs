use std::sync::Arc;
use image::{ImageBuffer, Rgba};
use vulkano::{device::{Device, Queue}, memory::allocator::StandardMemoryAllocator, command_buffer::{allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo}, image::{StorageImage, ImageDimensions, view::ImageView}, format::Format, pipeline::{ComputePipeline, Pipeline, PipelineBindPoint}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet, self, allocator::{DescriptorSetAllocator, StandardDescriptorSetAllocator}}, buffer::{CpuAccessibleBuffer, BufferUsage}, sync::GpuFuture};
use super::utils::ImageSizeInfo;

pub fn generate_mandelbrot_set_image(device: Arc<Device>, memory_allocator: &StandardMemoryAllocator, command_buffer_allocator: &StandardCommandBufferAllocator, queue: Arc<Queue>, image_size_info: ImageSizeInfo) {
    let gpu_image = StorageImage::new(
        memory_allocator,
        ImageDimensions::Dim2d {
            width: image_size_info.width,
            height: image_size_info.height,
            array_layers: 1,
        },
        Format::R8G8B8A8_UNORM,
        Some(queue.queue_family_index()),
    )
    .unwrap();

    let gpu_image_view = ImageView::new_default(gpu_image.clone()).unwrap();

    mod cs {
        vulkano_shaders::shader!{
            ty: "compute",
            src: "#version 450
    
            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
            layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

            void main() {
                vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
                vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

                vec2 z = vec2(0.0, 0.0);
                float i;
                for (i = 0.0; i < 1.0; i += 0.005) {
                    z = vec2(
                        z.x * z.x - z.y * z.y + c.x,
                        z.y * z.x + z.x * z.y + c.y
                    );

                    if (length(z) > 4.0) {
                        break;
                    }
                }

                vec4 to_write = vec4(vec3(i), 1.0);
                imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
            }"
        }
    }

    let shader = cs::load(device.clone())
        .expect("failed to create shader module");

    let compute_pipeline = ComputePipeline::new(
        device.clone(),
        shader.entry_point("main").unwrap(),
        &(),
        None,
        |_| {},
    ).expect("Failed to create compute pipeline");

    let layout = compute_pipeline.layout().set_layouts().get(0).unwrap();
    let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    let descriptor_set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout.clone(),
        [WriteDescriptorSet::image_view(0, gpu_image_view.clone())]
    ).unwrap();

    let image_output_buffer = CpuAccessibleBuffer::from_iter(
        memory_allocator,
        BufferUsage {
            transfer_dst: true,
            ..Default::default()
        },
        false,
        (0..image_size_info.width * image_size_info.height * 4).map(|_| 0u8)
    ).expect("Failed to create image output buffer");

    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit
    ).unwrap();

    builder
        .bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(PipelineBindPoint::Compute, compute_pipeline.layout().clone(), 0, descriptor_set)
        .dispatch([image_size_info.width / 8, image_size_info.height / 8, 1])
        .unwrap()
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(gpu_image.clone(), image_output_buffer.clone()))
        .unwrap();
    
    let command_buffer = builder.build().unwrap();

    let future = vulkano::sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let output_buffer_content = image_output_buffer.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(image_size_info.width, image_size_info.height, &output_buffer_content[..]).unwrap();

    image.save("mandelbrot_set.png").unwrap();

}
