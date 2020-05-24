use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::{Window, WindowBuilder},
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Troxel - WGPU REMAKE")
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input,
                    ..
                } => {
                    match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
                _ => {}
            }
            _ => {}
        }
    });
}

struct State {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>
}

impl State {
    async fn new(window: &Window) {
        unimplemented!()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        unimplemented!()
    }

    fn update(&mut self) {
        unimplemented!()
    }

    fn render(&mut self) {
        unimplemented!()
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        unimplemented!()
    }
}