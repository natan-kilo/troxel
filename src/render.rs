use std::{borrow::Borrow, iter, mem::ManuallyDrop, ptr};

use gfx_hal as hal;
use hal::{
    adapter::Adapter,
    adapter::PhysicalDevice,
    buffer::Usage,
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
use crate::types::{Triangle, Vertex};

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
    frames_in_flight: usize,
    frame: u64,
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

        let command_pool = unsafe {
            let command_pool = device
                .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
                .expect("Out of memory");

            command_pool
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
                w: dimensions.width as _,
                h: dimensions.height as _,
            },
            depth: 0.0..1.0,
        };

        // presetting a value for a vertex buffer with the max value of 1024 triangles
        let vertex_buffer_len = 1024 * std::mem::size_of::<Triangle>();

        let (vertex_buffer_memory, vertex_buffer) = unsafe {
            new_buffer::<B>(
                &device,
                &adapter.physical_device,
                vertex_buffer_len,
                Usage::VERTEX,
                Properties::CPU_VISIBLE,
            )
        };

        let frames_in_flight = 3;

        //device.create_fence(true).expect("Out of memory");
        let mut submission_complete_semaphores: Vec<B::Semaphore> =
            Vec::with_capacity(frames_in_flight);
        let mut submission_complete_fences: Vec<B::Fence> = Vec::with_capacity(frames_in_flight);

        let mut command_pools: Vec<B::CommandPool> = Vec::with_capacity(frames_in_flight);
        let mut command_buffers: Vec<B::CommandBuffer> = Vec::with_capacity(frames_in_flight);

        command_pools.push(command_pool);
        for _ in 1..frames_in_flight {
            unsafe {
                command_pools.push(
                    device
                        .create_command_pool(
                            queue_group.family,
                            hal::pool::CommandPoolCreateFlags::empty(),
                        )
                        .expect("Can't create command pool"),
                );
            }
        }

        for i in 0..frames_in_flight {
            submission_complete_semaphores.push(
                device
                    .create_semaphore()
                    .expect("Could not create semaphore"),
            );
            submission_complete_fences
                .push(device.create_fence(true).expect("Could not create fence"));
            command_buffers
                .push(unsafe { command_pools[i].allocate_one(hal::command::Level::Primary) });
        }

        Renderer {
            instance,
            surface: ManuallyDrop::new(surface),
            adapter,
            device,
            render_passes: ManuallyDrop::new(vec![render_pass]),
            pipeline_layouts: ManuallyDrop::new(vec![pipeline_layout]),
            pipelines: ManuallyDrop::new(vec![pipeline]),
            buffer: ManuallyDrop::new(vec![vertex_buffer]),
            buffer_memory: ManuallyDrop::new(vec![vertex_buffer_memory]),
            command_pools: command_pools,
            command_buffers: command_buffers,
            submission_complete_semaphores: submission_complete_semaphores,
            submission_complete_fences: submission_complete_fences,
            dimensions,
            viewport,
            format,
            queue_group,
            frames_in_flight,
            frame: 0,
        }
    }

    pub fn renew_swapchain(&mut self) {
        let caps = self.surface.capabilities(&self.adapter.physical_device);
        let swap_config =
            hal::window::SwapchainConfig::from_caps(&caps, self.format, self.dimensions);
        let dimensions = swap_config.extent.to_extent();

        unsafe {
            self.surface
                .configure_swapchain(&self.device, swap_config)
                .expect("Can't create swapchain");
        }
        self.viewport.rect.w = dimensions.width as _;
        self.viewport.rect.h = dimensions.height as _;
    }

    pub fn render(&mut self) {
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

        let frame_idx = self.frame as usize % self.frames_in_flight;

        unsafe {
            let fence = &self.submission_complete_fences[frame_idx];
            self.device
                .wait_for_fence(fence, !0)
                .expect("Failed to wait for fence");
            self.device
                .reset_fence(fence)
                .expect("Failed to reset fence");
            self.command_pools[frame_idx].reset(false);
        }

        let command_buffer = &mut self.command_buffers[frame_idx];

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
        let num_vertecies = (data.len() * 3) as u32;

        let data_mem_len = data.len() * std::mem::size_of::<Triangle>();

        unsafe {
            let mapped_memory = self
                .device
                .map_memory(&self.buffer_memory[0], Segment::ALL)
                .expect("TODO");

            ptr::copy(data.as_ptr() as *const u8, mapped_memory, data_mem_len);
            self.device
                .flush_mapped_memory_ranges(vec![(&self.buffer_memory[0], Segment::ALL)])
                .expect("TODO");

            self.device.unmap_memory(&self.buffer_memory[0]);
        }

        unsafe {
            use hal::command::{
                ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, SubpassContents,
            };

            command_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);

            command_buffer.set_viewports(0, &[self.viewport.clone()]);
            command_buffer.set_scissors(0, &[self.viewport.rect]);

            command_buffer.bind_graphics_pipeline(&self.pipelines[0]);

            command_buffer.bind_vertex_buffers(
                0,
                iter::once((&self.buffer[0], hal::buffer::SubRange::WHOLE)),
            );

            command_buffer.begin_render_pass(
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

            command_buffer.draw(0..num_vertecies, 0..1);

            command_buffer.end_render_pass();
            command_buffer.finish();
        }

        unsafe {
            use hal::queue::{CommandQueue, Submission};

            let submission = Submission {
                command_buffers: iter::once(&*command_buffer),
                wait_semaphores: None,
                signal_semaphores: iter::once(&self.submission_complete_semaphores[frame_idx]),
            };

            self.queue_group.queues[0].submit(
                submission,
                Some(&self.submission_complete_fences[frame_idx]),
            );

            let result = self.queue_group.queues[0].present_surface(
                &mut self.surface,
                surface_image,
                Some(&self.submission_complete_semaphores[frame_idx]),
            );

            self.device.destroy_framebuffer(framebuffer);

            if result.is_err() {
                self.renew_swapchain();
            }
        }
        self.frame += 1;
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
            self.instance
                .destroy_surface(ManuallyDrop::into_inner(ptr::read(&self.surface)));
        }
    }
}
