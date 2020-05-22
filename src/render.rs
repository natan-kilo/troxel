use std::{borrow::Borrow, iter, mem::ManuallyDrop, ptr};

use gfx_hal as hal;
use hal::{
    adapter::Adapter,
    adapter::PhysicalDevice,
    buffer::Usage,
    command::Level,
    device::Device,
    format::{ChannelType, Format},
    image::{self, Layout},
    memory::{Properties, Segment},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso::Viewport,
    queue::{family::QueueGroup, QueueFamily},
    window::{Extent2D, PresentationSurface, Surface},
    Instance,
};

use crate::buffer::new_buffer;
use crate::pipeline::new_pipeline;
use crate::test_values::{MESH, TRIANGLE};
use crate::types::{Rectangle, Triangle, Vertex};
use crate::utils::{drop, undrop};

pub struct Renderer<B: hal::Backend> {
    instance: B::Instance,
    surface: ManuallyDrop<B::Surface>,
    adapter: Adapter<B>,
    device: B::Device,
    render_passes: ManuallyDrop<Vec<B::RenderPass>>,
    pipeline_layouts: ManuallyDrop<Vec<B::PipelineLayout>>,
    pipelines: ManuallyDrop<Vec<B::GraphicsPipeline>>,
    buffer: ManuallyDrop<Vec<B::Buffer>>,
    buffer_memory: ManuallyDrop<Vec<B::Memory>>,
    command_pools: Vec<B::CommandPool>,
    command_buffers: Vec<B::CommandBuffer>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    submission_complete_fences: Vec<B::Fence>,
    pub dimensions: Extent2D,
    viewport: Viewport,
    format: Format,
    queue_group: QueueGroup<B>,
}

impl<B: hal::Backend> Renderer<B> {
    pub fn new(
        instance: B::Instance,
        mut surface: B::Surface,
        adapter: Adapter<B>,
        dimensions: Extent2D,
    ) -> Renderer<B> {
        let (device, queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|family| {
                    surface.supports_queue_family(family) && family.queue_type().supports_graphics()
                })
                .expect("No compatible queue family found");

            let mut gpu = unsafe {
                adapter
                    .physical_device
                    .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
                    .expect("Failed to open device")
            };

            (gpu.device, gpu.queue_groups.pop().unwrap())
        };

        let (command_pool, command_buffer) = unsafe {
            let mut command_pool = device
                .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
                .expect("Out of memory");

            let command_buffer = command_pool.allocate_one(Level::Primary);

            (command_pool, command_buffer)
        };

        let format = {
            let supported_formats = surface
                .supported_formats(&adapter.physical_device)
                .unwrap_or(vec![]);

            let default_format = *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);

            supported_formats
                .into_iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .unwrap_or(default_format)
        };
        let caps = surface.capabilities(&adapter.physical_device);
        let swap_config = hal::window::SwapchainConfig::from_caps(&caps, format, dimensions);
        unsafe {
            surface
                .configure_swapchain(&device, swap_config)
                .expect("Can't configure swapchain");
        };

        let render_pass = {
            let color_attachment = Attachment {
                format: Some(format),
                samples: 1,
                ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts: Layout::Undefined..Layout::Present,
            };

            let subpass = SubpassDesc {
                colors: &[(0, Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            unsafe {
                device
                    .create_render_pass(&[color_attachment], &[subpass], &[])
                    .expect("Out of memory")
            }
        };

        let pipeline_layout: B::PipelineLayout = unsafe {
            device
                .create_pipeline_layout(&[], &[])
                .expect("Out of memory")
        };

        let mesh1: [Vertex; 3] = crate::utils::clone_into_array(&MESH[0..3]);
        let mesh2: [Vertex; 3] = crate::utils::clone_into_array(&MESH[3..6]);

        let mut triangle_1: Triangle = Triangle::from_slice(&TRIANGLE);

        let triangle_2: Triangle = Triangle::from_slice(&mesh1);
        let triangle_3: Triangle = Triangle::from_slice(&mesh2);

        let verty = Vertex {
            pos: [-0.2, -0.2, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
        };

        triangle_1.change_vertex(0, &verty);

        let data: Vec<Triangle> = vec![triangle_2, triangle_3, triangle_1];

        let triangle_1: Triangle = Triangle::from_slice(&TRIANGLE);
        println!("data: {:?}", data);
        println!("triangle: {}", triangle_1.size());

        let vertex_buffer_len = data.len() * std::mem::size_of::<Triangle>();

        println!("vertex buffer: {}", vertex_buffer_len);

        let (vertex_buffer_memory, vertex_buffer) = unsafe {
            new_buffer::<B>(
                &device,
                &adapter.physical_device,
                vertex_buffer_len,
                Usage::VERTEX,
                Properties::CPU_VISIBLE,
            )
        };

        unsafe {
            let mapped_memory = device
                .map_memory(&vertex_buffer_memory, Segment::ALL)
                .expect("TODO");

            ptr::copy(data.as_ptr() as *const u8, mapped_memory, vertex_buffer_len);
            device
                .flush_mapped_memory_ranges(vec![(&vertex_buffer_memory, Segment::ALL)])
                .expect("TODO");

            device.unmap_memory(&vertex_buffer_memory);
        }

        let vertex_shader = include_str!("../assets/shaders/shader.vs");
        let fragment_shader = include_str!("../assets/shaders/shader.fs");

        let pipeline: B::GraphicsPipeline = unsafe {
            new_pipeline::<B>(
                &device,
                &render_pass,
                &pipeline_layout,
                vertex_shader,
                fragment_shader,
            )
        };

        let viewport = hal::pso::Viewport {
            rect: hal::pso::Rect {
                x: 0,
                y: 0,
                w: dimensions.width as i16,
                h: dimensions.height as i16,
            },
            depth: 0.0..1.0,
        };

        let submission_complete_fence: B::Fence = device.create_fence(true).expect("Out of memory");
        let rendering_complete_semaphore: B::Semaphore =
            device.create_semaphore().expect("Out of memory");

        Renderer {
            instance,
            surface: drop(surface),
            adapter,
            device,
            render_passes: drop(vec![render_pass]),
            pipeline_layouts: drop(vec![pipeline_layout]),
            pipelines: drop(vec![pipeline]),
            buffer: drop(vec![vertex_buffer]),
            buffer_memory: drop(vec![vertex_buffer_memory]),
            command_pools: vec![command_pool],
            command_buffers: vec![command_buffer],
            submission_complete_semaphores: vec![rendering_complete_semaphore],
            submission_complete_fences: vec![submission_complete_fence],
            dimensions,
            viewport,
            format,
            queue_group,
        }
    }

    pub fn renew_swapchain(&mut self) {
        let caps = self.surface.capabilities(&self.adapter.physical_device);
        let swap_config =
            hal::window::SwapchainConfig::from_caps(&caps, self.format, self.dimensions);
        let extent = swap_config.extent.to_extent();

        unsafe {
            self.surface
                .configure_swapchain(&self.device, swap_config)
                .expect("Can't create swapchain");
        }
        self.viewport.rect.w = extent.width as i16;
        self.viewport.rect.h = extent.height as i16;
    }

    pub fn render(&mut self) {
        let render_timeout_ns = 1_000_000_000;
        unsafe {
            self.device
                .wait_for_fence(&self.submission_complete_fences[0], render_timeout_ns)
                .expect("Out of memory");

            self.device
                .reset_fence(&self.submission_complete_fences[0])
                .expect("Out of memory");

            self.command_pools[0].reset(false);
        };

        let surface_image = unsafe {
            match self.surface.acquire_image(!0) {
                Ok((image, _)) => image,
                Err(_) => {
                    self.renew_swapchain();
                    return;
                }
            }
        };

        let framebuffer = unsafe {
            self.device
                .create_framebuffer(
                    &self.render_passes[0],
                    iter::once(surface_image.borrow()),
                    image::Extent {
                        width: self.dimensions.width,
                        height: self.dimensions.height,
                        depth: 1,
                    },
                )
                .unwrap()
        };

        unsafe {
            use hal::command::{
                ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, SubpassContents,
            };

            self.command_buffers[0].begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);

            self.command_buffers[0].set_viewports(0, &[self.viewport.clone()]);
            self.command_buffers[0].set_scissors(0, &[self.viewport.rect]);

            self.command_buffers[0]
                .bind_vertex_buffers(0, vec![(&self.buffer[0], hal::buffer::SubRange::WHOLE)]);

            self.command_buffers[0].begin_render_pass(
                &self.render_passes[0],
                &framebuffer,
                self.viewport.rect,
                &[ClearValue {
                    color: ClearColor {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                }],
                SubpassContents::Inline,
            );

            self.command_buffers[0].bind_graphics_pipeline(&self.pipelines[0]);

            let num_vertecies = 9 as u32;
            self.command_buffers[0].draw(0..num_vertecies, 0..1);

            self.command_buffers[0].end_render_pass();
            self.command_buffers[0].finish();
        }

        unsafe {
            use hal::queue::{CommandQueue, Submission};

            let submission = Submission {
                command_buffers: &self.command_buffers,
                wait_semaphores: None,
                signal_semaphores: &self.submission_complete_semaphores,
            };

            self.queue_group.queues[0]
                .submit(submission, Some(&self.submission_complete_fences[0]));

            let result = self.queue_group.queues[0].present_surface(
                &mut self.surface,
                surface_image,
                Some(&self.submission_complete_semaphores[0]),
            );

            if result.is_err() {
                self.renew_swapchain();
            }

            self.device.destroy_framebuffer(framebuffer);
        }
    }
}

impl<B: gfx_hal::Backend> Drop for Renderer<B> {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap();
        unsafe {
            for mem in self.buffer_memory.drain(..) {
                self.device.free_memory(mem);
            }

            for buff in self.buffer.drain(..) {
                self.device.destroy_buffer(buff);
            }

            for semaphore in self.submission_complete_semaphores.drain(..) {
                self.device.destroy_semaphore(semaphore);
            }

            for fence in self.submission_complete_fences.drain(..) {
                self.device.destroy_fence(fence);
            }

            for pipeline in self.pipelines.drain(..) {
                self.device.destroy_graphics_pipeline(pipeline);
            }

            for layout in self.pipeline_layouts.drain(..) {
                self.device.destroy_pipeline_layout(layout);
            }

            for render_pass in self.render_passes.drain(..) {
                self.device.destroy_render_pass(render_pass);
            }

            for command_pool in self.command_pools.drain(..) {
                self.device.destroy_command_pool(command_pool);
            }

            self.surface.unconfigure_swapchain(&self.device);
            self.instance.destroy_surface(undrop(&self.surface));
        }
    }
}
