use std::mem::ManuallyDrop;

use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use gfx_hal::{
    device::Device,
    window::{Extent2D, PresentationSurface, Surface},
    Instance,
};

const APP_NAME: &'static str = "Troxel";
const WINDOW_SIZE: [u32; 2] = [512, 512];

mod main2;

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

    // access to command queues to give commands to gpu
    // queue must be compatible with surface and support the graphics card
    let (device, mut queue_group) = {
        use gfx_hal::queue::QueueFamily;

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

unsafe fn make_buffer<B: gfx_hal::Backend>(
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
