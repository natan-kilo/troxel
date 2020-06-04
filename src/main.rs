use std::mem;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use futures::executor::block_on;
use glsl_to_spirv::ShaderType;
use wgpu::{ShaderModule, RenderPassDescriptor};


mod config;
mod types;
mod utils;
mod texture;
mod camera;

use types::{Vertex, VertexC, Rectangle};
use crate::camera::{Camera, CameraController, Uniforms};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(config::APP_NAME)
        .build(&event_loop)
        .unwrap();

    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
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
    });
}

struct State {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,

    camera: Camera,
    camera_controller: CameraController,

    uniforms: camera::Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    size: winit::dpi::PhysicalSize<u32>,
    depth_texture: texture::Texture,
    clear_color: wgpu::Color,
}

impl State {
    async fn new(window: &Window) -> Self {
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

        let diffuse_bytes = include_bytes!("../assets/images/cat.png");

        let (diffuse_texture, cmd_buffer) = texture::Texture::from_bytes(
            &device,
            diffuse_bytes,
            "cat.png"
        ).unwrap();


        queue.submit(&[cmd_buffer]);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                    },
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera = camera::Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fov_y: 45.0,
            z_near: 0.1,
            z_far: 100.0
        };

        let camera_controller = CameraController::new(0.2);

        let mut uniforms = camera::Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false
                    },
                },
            ],
            label: Some("uniform_bind_group_layout"),
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..std::mem::size_of_val(&uniforms) as wgpu::BufferAddress,
                    }
                }
            ],
            label: Some("uniform_bind_group")
        });


        let vs_src = include_str!("../assets/shaders/shader_tex.vert");
        let fs_src = include_str!("../assets/shaders/shader_tex.frag");

        let vs_module = utils::create_shader_module(vs_src, ShaderType::Vertex, &device);
        let fs_module = utils::create_shader_module(fs_src, ShaderType::Fragment, &device);

        let depth_texture = texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::all(),
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: true,
        });

        let vertex_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(VERTICES), wgpu::BufferUsage::VERTEX);

        let index_buffer =
            device.create_buffer_with_data(bytemuck::cast_slice(INDICES), wgpu::BufferUsage::INDEX);

        let num_indices = INDICES.len() as u32;

        let clear_color = wgpu::Color::BLACK;


        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            camera_controller,
            uniforms,
            uniform_buffer,
            uniform_bind_group,
            diffuse_texture,
            diffuse_bind_group,
            size,
            depth_texture,
            clear_color,
        }
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("update encoder")
        });

        let staging_buffer = self.device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.uniform_buffer, 0, std::mem::size_of::<Uniforms>() as wgpu::BufferAddress);

        self.queue.submit(&[encoder.finish()])
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: self.clear_color,
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture.view,
                    depth_load_op: wgpu::LoadOp::Clear,
                    depth_store_op: wgpu::StoreOp::Store,
                    clear_depth: 1.0,
                    stencil_load_op: wgpu::LoadOp::Clear,
                    stencil_store_op: wgpu::StoreOp::Store,
                    clear_stencil: 0,
                })
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

            render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
            render_pass.set_index_buffer(&self.index_buffer, 0, 0);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }
        self.queue.submit(&[encoder.finish()]);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event);
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            _ => false,
        }
    }
}

// const RECT: Rectangle = Rectangle {
//     vertices: [
//         Vertex {
//             position: [0.5, -0.5, 0.0],
//             color: [1.0, 0.0, 1.0, 1.0],
//         },
//         Vertex {
//             position: [0.5, -1.0, 0.0],
//             color: [1.0, 0.0, 0.3, 1.0],
//         },
//         Vertex {
//             position: [1.0, -0.5, 0.0],
//             color: [1.0, 0.0, 1.0, 0.0],
//         },
//         Vertex {
//             position: [1.0, -1.0, 0.0],
//             color: [1.0, 0.0, 1.0, 1.0],
//         }],
//     indices: [2, 3]
// };

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.4131759, 0.00759614], }, // 0
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.0048659444, 0.43041354], }, // 1
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.28081453, 0.949397057], }, // 2
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.85967, 0.84732911], }, // 3
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.9414737, 0.2652641], }, // 4

    Vertex { position: [-0.0868241, 0.49240386, 0.5], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.4131759, 0.00759614], }, // 5
    Vertex { position: [-0.49513406, 0.06958647, 0.5], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.0048659444, 0.43041354], }, // 6
    Vertex { position: [-0.21918549, -0.44939706, -0.2], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.28081453, 0.949397057], }, // 7
    Vertex { position: [0.35966998, -0.3473291, -0.2], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.85967, 0.84732911], }, // 8
    Vertex { position: [0.44147372, 0.2347359, -0.2], color: [1.0, 1.0, 1.0, 1.0], tex_coords: [0.9414737, 0.2652641], }, // 9
];

const INDICES: &[u16] = &[0, 1, 4,  1, 2, 4,  2, 3, 4,  5, 6, 9,  6, 7, 9,  7, 8, 9];
