use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use gfx_hal::window::Extent2D;

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

    let mut surface_extent = Extent2D {
        width: physical_size.width,
        height: physical_size.height,
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
        _ => (),
    });
}

fn window_sizes(event_loop: &EventLoop<()>) -> (LogicalSize<u32>, PhysicalSize<u32>) {
    let dpi = event_loop.primary_monitor().scale_factor();
    let logical_size: LogicalSize<u32> = WINDOW_SIZE.into();
    let physical_size: PhysicalSize<u32> = logical_size.to_physical(dpi);
    (logical_size, physical_size)
}
