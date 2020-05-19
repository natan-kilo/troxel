use std::{
    mem::{self, ManuallyDrop},
    ptr,
};

use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use gfx_hal::{
    buffer, command,
    device::Device,
    memory,
    queue::{QueueFamily, QueueGroup, Submission},
    window::{Extent2D, PresentationSurface, Surface},
    Instance,
};

use glsl_to_spirv::ShaderType;

const APP_NAME: &'static str = "Troxel";
const WINDOW_SIZE: [u32; 2] = [512, 512];

fn main() {
    let event_loop: EventLoop<()> = EventLoop::new();

    let (logical_size, physical_size) = window_sizes(&event_loop);

    let window = WindowBuilder::new()
        .with_title(APP_NAME)
        .with_inner_size(logical_size)
        .build(&event_loop)
        .expect("Failed to create window");

    // Surface to integrate vulkan into window
    let mut surface_extent = create_surface_extent(&physical_size);

    let (instance, surface, adapter) = {
        let instance = backend::Instance::create(APP_NAME, 1).expect("Backend not supported");

        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("Failed to create surface for window")
        };

        let adapter = instance.enumerate_adapters().remove(0);

        (instance, surface, adapter)
    };

    let mut renderer = Renderer::new(instance, surface, adapter);

    // access to command queues to give commands to gpu
    // queue must be compatible with surface and support the graphics card

    let mut should_configure_swapchain = true;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(dims) => {
                surface_extent = Extent2D {
                    width: dims.width,
                    height: dims.height,
                };
                should_configure_swapchain = true;
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                surface_extent = Extent2D {
                    width: new_inner_size.width,
                    height: new_inner_size.height,
                };
                should_configure_swapchain = true;
            }
            _ => (),
        },
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(_) => {}
        _ => (),
    });
}

struct Renderer<B: gfx_hal::Backend> {
    instance: B::Instance,
    surface: ManuallyDrop<B::Surface>,
    adapter: gfx_hal::adapter::Adapter<B>,
    device: ManuallyDrop<B::Device>,
    render_passes: ManuallyDrop<Vec<B::RenderPass>>,
    pipeline_layouts: ManuallyDrop<Vec<B::PipelineLayout>>,
    pipelines: ManuallyDrop<Vec<B::GraphicsPipeline>>,
    command_pools: Vec<B::CommandPool>,
    submission_complete_semaphores: Vec<B::Semaphore>,
    submission_complete_fences: Vec<B::Fence>,
}

impl<B: gfx_hal::Backend> Renderer<B> {
    fn new(
        instance: B::Instance,
        mut surface: B::Surface,
        adapter: gfx_hal::adapter::Adapter<B>,
    ) -> Renderer<B> {
        let (device, mut queue_group) = {
            let queue_family = adapter
                .queue_families
                .iter()
                .find(|family| {
                    surface.supports_queue_family(family) && family.queue_type().supports_graphics()
                })
                .expect("No compatible queue family found");

            let mut gpu = unsafe {
                use gfx_hal::adapter::PhysicalDevice;

                adapter
                    .physical_device
                    .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
                    .expect("Failed to open device")
            };

            (gpu.device, gpu.queue_groups.pop().unwrap())
        };

        let (command_pool, mut command_buffer) = unsafe {
            use gfx_hal::command::Level;
            use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};

            let mut command_pool = device
                .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
                .expect("Out of memory");

            let command_buffer = command_pool.allocate_one(Level::Primary);

            (command_pool, command_buffer)
        };

        let surface_color_format = {
            use gfx_hal::format::{ChannelType, Format};

            let supported_formats = surface
                .supported_formats(&adapter.physical_device)
                .unwrap_or(vec![]);

            let default_format = *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);

            supported_formats
                .into_iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .unwrap_or(default_format)
        };

        let render_pass = {
            use gfx_hal::image::Layout;
            use gfx_hal::pass::{
                Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc,
            };

            let color_attachment = Attachment {
                format: Some(surface_color_format),
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
            use gfx_hal::pso::ShaderStageFlags;

            let push_constant_bytes = std::mem::size_of::<Vertex>() as u32;

            device
                .create_pipeline_layout(&[], &[(ShaderStageFlags::VERTEX, 0..push_constant_bytes)])
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

        let submission_complete_fence: B::Fence = device.create_fence(true).expect("Out of memory");
        let rendering_complete_semaphore: B::Semaphore =
            device.create_semaphore().expect("Out of memory");

        Renderer {
            instance,
            surface: drop(surface),
            adapter,
            device: drop(device),
            render_passes: drop(vec![render_pass]),
            pipeline_layouts: drop(vec![pipeline_layout]),
            pipelines: drop(vec![pipeline]),
            command_pools: vec![command_pool],
            submission_complete_semaphores: vec![rendering_complete_semaphore],
            submission_complete_fences: vec![submission_complete_fence],
        }
    }
}

impl<B: gfx_hal::Backend> Drop for Renderer<B> {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap();
        unsafe {
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
                .destroy_surface(undrop(&self.surface));
        }
    }
}
struct Vertex {
    position: [f32; 3],
    scale: [f32; 3],
    color: [f32; 4],
}

fn window_sizes(event_loop: &EventLoop<()>) -> (LogicalSize<u32>, PhysicalSize<u32>) {
    let dpi = event_loop.primary_monitor().scale_factor();
    let logical_size: LogicalSize<u32> = WINDOW_SIZE.into();
    let physical_size: PhysicalSize<u32> = logical_size.to_physical(dpi);
    (logical_size, physical_size)
}

fn create_surface_extent(physical_size: &PhysicalSize<u32>) -> Extent2D {
    Extent2D {
        width: physical_size.width,
        height: physical_size.height,
    }
}

unsafe fn new_buffer<B: gfx_hal::Backend>(
    device: &B::Device,
    physical_device: &B::PhysicalDevice,
    buffer_len: usize,
    usage: gfx_hal::buffer::Usage,
    properties: gfx_hal::memory::Properties,
) -> (B::Memory, B::Buffer) {
    use gfx_hal::{adapter::PhysicalDevice, MemoryTypeId};

    // This creates a handle to a buffer. The `buffer_len` is in bytes,
    // and the usage states what kind of buffer it is.
    let mut buffer = device
        .create_buffer(buffer_len as u64, usage)
        .expect("Failed to create buffer");

    // determine memory type and requirements
    let req = device.get_buffer_requirements(&buffer);

    let memory_types = physical_device.memory_properties().memory_types;

    let memory_type = memory_types
        .iter()
        .enumerate()
        .find(|(id, mem_type)| {
            let type_supported = req.type_mask & (1_u64 << id) != 0;
            type_supported && mem_type.properties.contains(properties)
        })
        .map(|(id, _ty)| MemoryTypeId(id))
        .expect("No compatible memory type available");

    // allocate memory for the buffer
    let buffer_memory = device
        .allocate_memory(memory_type, req.size)
        .expect("Failed to allocate buffer memory");

    // bind the memory to the buffer
    device
        .bind_buffer_memory(&buffer_memory, 0, &mut buffer)
        .expect("Failed to bind buffer memory");

    (buffer_memory, buffer)
}

unsafe fn new_pipeline<B: gfx_hal::Backend>(
    device: &B::Device,
    render_pass: &B::RenderPass,
    pipeline_layout: &B::PipelineLayout,
    vertex_shader: &str,
    fragment_shader: &str,
) -> B::GraphicsPipeline {
    use gfx_hal::pass::Subpass;
    use gfx_hal::pso::{
        BlendState, ColorBlendDesc, ColorMask, EntryPoint, Face, GraphicsPipelineDesc,
        GraphicsShaderSet, Primitive, Rasterizer, Specialization,
    };
    let vertex_shader_module = device
        .create_shader_module(&compile_shader(vertex_shader, ShaderType::Vertex))
        .expect("Failed to create vertex shader module");

    let fragment_shader_module = device
        .create_shader_module(&compile_shader(fragment_shader, ShaderType::Fragment))
        .expect("Failed to create fragment shader module");

    let (vs_entry, fs_entry) = (
        EntryPoint {
            entry: "main",
            module: &vertex_shader_module,
            specialization: Specialization::default(),
        },
        EntryPoint {
            entry: "main",
            module: &fragment_shader_module,
            specialization: Specialization::default(),
        },
    );

    let shader_entries = GraphicsShaderSet {
        vertex: vs_entry,
        hull: None,
        domain: None,
        geometry: None,
        fragment: Some(fs_entry),
    };

    let mut pipeline_desc = GraphicsPipelineDesc::new(
        shader_entries,
        Primitive::TriangleList,
        Rasterizer {
            cull_face: Face::BACK,
            ..Rasterizer::FILL
        },
        pipeline_layout,
        Subpass {
            index: 0,
            main_pass: render_pass,
        },
    );

    pipeline_desc.blender.targets.push(ColorBlendDesc {
        mask: ColorMask::ALL,
        blend: Some(BlendState::ALPHA),
    });
    let pipeline = device
        .create_graphics_pipeline(&pipeline_desc, None)
        .expect("Failed to create graphics pipeline");

    device.destroy_shader_module(vertex_shader_module);
    device.destroy_shader_module(fragment_shader_module);

    pipeline
}

fn compile_shader(glsl: &str, shader_type: ShaderType) -> Vec<u32> {
    use std::io::{Cursor, Read};

    let mut compiled_file =
        glsl_to_spirv::compile(glsl, shader_type).expect("Failed to compile shader");

    let mut spirv_bytes = vec![];
    compiled_file.read_to_end(&mut spirv_bytes).unwrap();

    let spirv = gfx_hal::pso::read_spirv(Cursor::new(&spirv_bytes)).expect("Invalid SPIR-V");

    spirv
}

fn drop<T>(object: T) -> ManuallyDrop<T> {
    ManuallyDrop::new(object)
}

unsafe fn undrop<T>(object: &ManuallyDrop<T>) -> T {
    ManuallyDrop::into_inner(ptr::read(object))
}
