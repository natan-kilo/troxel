use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use gfx_hal::{window::Extent2D, Instance};

mod buffer;
mod config;
mod pipeline;
mod render;
mod test_values;
mod types;
mod utils;

use config::APP_NAME;
use render::Renderer;
use utils::{new_surface_extent, new_window, window_sizes};

fn main() {
    let event_loop: EventLoop<()> = EventLoop::new();

    let (logical_size, physical_size) = window_sizes(&event_loop);

    let window = new_window(APP_NAME, logical_size, &event_loop);
    // Surface to integrate vulkan into window
    let surface_extent = new_surface_extent(&physical_size);

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

    let mut renderer = Renderer::new(instance, surface, adapter, surface_extent);

    // access to command queues to give commands to gpu
    // queue must be compatible with surface and support the graphics card

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(dims) => {
                renderer.dimensions = Extent2D {
                    width: dims.width,
                    height: dims.height,
                };
                renderer.renew_swapchain();
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.dimensions = Extent2D {
                    width: new_inner_size.width,
                    height: new_inner_size.height,
                };
                renderer.renew_swapchain();
            }
            _ => (),
        },
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(_) => {
            renderer.render();
        }
        _ => (),
    });
}
