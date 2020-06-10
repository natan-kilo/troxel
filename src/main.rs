extern crate ultraviolet as uv;

use std::mem;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use futures::executor::block_on;
use glsl_to_spirv::ShaderType;
use wgpu::{RenderPassDescriptor, ShaderModule};
use winit_input_helper::WinitInputHelper;

mod camera;
mod config;
mod render;
mod state;
mod types;
mod utils;
mod tools;

use state::traits::Stateful;


use crate::state::state_handler::StateHandler;


fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(config::APP_NAME)
        .build(&event_loop)
        .unwrap();

    window.set_cursor_grab(true).unwrap();

    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        state.input.update(&event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event, control_flow) {
                    match event {
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                state.render();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    })
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    input: WinitInputHelper,

    state_handler: state::state_handler::StateHandler,

    size: winit::dpi::PhysicalSize<u32>,
}

impl State {
    async fn new(window: &Window) -> Self {
        let mut input = WinitInputHelper::new();

        let size = window.inner_size();

        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let mut state_handler = StateHandler::new(&device, &sc_desc);

        state_handler.add_state(Box::new(state::states::chaotic_state::ChaoticState::new(
            &device, &queue, &sc_desc, &size,
        )));

        state_handler.set_state(state::states::state_ids::CHAOTIC);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            input,

            state_handler,

            size,
        }
    }

    fn update(&mut self) {
        self.state_handler.states[self.state_handler.current_state_in_vec].update(&mut self.device, &mut self.queue);
    }

    fn render(&mut self) {
        let frame = self
            .swap_chain
            .get_next_texture()
            .expect("Timeout getting texture");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            self.state_handler.states[self.state_handler.current_state_in_vec].render(&frame, &mut encoder);
        }

        self.queue.submit(&[encoder.finish()]);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.state_handler.states[self.state_handler.current_state_in_vec].resize(&mut self.device, &mut self.sc_desc, &self.size);
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    // RESIZE STATE AFTER STATE CHANGE TO FULFILL ASSERTION
    fn input(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) -> bool {
        use crate::state::states::state_ids;

        if self.input.key_released(VirtualKeyCode::Escape) || self.input.quit() {
            *control_flow = ControlFlow::Exit;
            return false
        }
        if self.input.key_pressed(VirtualKeyCode::F1) {
            self.state_handler.set_state(state_ids::NONE);
            self.state_handler.states[self.state_handler.current_state_in_vec].resize(&mut self.device, &mut self.sc_desc, &self.size);

        }
        if self.input.key_pressed(VirtualKeyCode::F3) {
            self.state_handler.set_state(state_ids::MENU);
            self.state_handler.states[self.state_handler.current_state_in_vec].resize(&mut self.device, &mut self.sc_desc, &self.size);
        }
        if self.input.key_pressed(VirtualKeyCode::F4) {
            self.state_handler.set_state(state_ids::CHAOTIC);
            self.state_handler.states[self.state_handler.current_state_in_vec].resize(&mut self.device, &mut self.sc_desc, &self.size);
        }
        self.state_handler.states[self.state_handler.current_state_in_vec].input(&self.input)
    }
}
