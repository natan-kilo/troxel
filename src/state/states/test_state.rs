use crate::state::traits::Stateful;
use glsl_to_spirv::ShaderType;
use std::any::Any;

use crate::camera;
use crate::render::texture;
use crate::types::Vertex;
use wgpu::{Device, SwapChainDescriptor};

pub struct TestState {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    render_pipeline: wgpu::RenderPipeline,

    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,

    camera: camera::Camera,
    camera_controller: camera::CameraController,

    uniforms: camera::Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
}

impl TestState {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let (diffuse_texture, cmd_buffer) =
            texture::Texture::new(&device, "assets/images/cat.png", "cat.png");

        queue.submit(&[cmd_buffer]);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                        ty: wgpu::BindingType::Sampler { comparison: false },
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
                },
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
            z_far: 100.0,
        };

        let camera_controller = camera::CameraController::new(0.2);

        let mut uniforms = camera::Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..std::mem::size_of_val(&uniforms) as wgpu::BufferAddress,
                },
            }],
            label: Some("uniform_bind_group"),
        });

        let vs_src = include_str!("../../../assets/shaders/shader_tex.vert");
        let fs_src = include_str!("../../../assets/shaders/shader_tex.frag");

        let vs_module = crate::utils::create_shader_module(vs_src, ShaderType::Vertex, &device);
        let fs_module = crate::utils::create_shader_module(fs_src, ShaderType::Fragment, &device);

        let depth_texture =
            texture::Texture::new_depth(&device, &sc_desc, "depth_texture");

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
            vertex_buffer,
            index_buffer,
            num_indices,

            render_pipeline,

            diffuse_texture,
            diffuse_bind_group,
            depth_texture,

            camera,
            camera_controller,

            uniforms,
            uniform_buffer,
            uniform_bind_group,

            size: size.clone(),
            clear_color,
        }
    }
}

impl Stateful for TestState {
    fn render(&mut self, frame: &wgpu::SwapChainOutput, encoder: &mut wgpu::CommandEncoder) {
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
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.camera_controller.update_camera(&mut self.camera);
        self.uniforms.update_view_proj(&self.camera);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Update Encoder"),
        });

        let staging_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<camera::Uniforms>() as wgpu::BufferAddress,
        );

        queue.submit(&[encoder.finish()]);
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        use winit::event::*;
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

    fn resize(
        &mut self,
        device: &mut Device,
        sc_desc: &mut SwapChainDescriptor,
        size: &winit::dpi::PhysicalSize<u32>,
    ) {
        self.size = size.clone();
        self.depth_texture =
            texture::Texture::new_depth(&device, &sc_desc, "depth_texture");
    }

    fn id(&self) -> usize {
        super::state_ids::TEST
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        unimplemented!()
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // 0
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // 1
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.28081453, 0.949397057],
    }, // 2
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.85967, 0.84732911],
    }, // 3
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // 4
    Vertex {
        position: [-0.0868241, 0.49240386, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // 5
    Vertex {
        position: [-0.49513406, 0.06958647, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // 6
    Vertex {
        position: [-0.21918549, -0.44939706, -0.2],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.28081453, 0.949397057],
    }, // 7
    Vertex {
        position: [0.35966998, -0.3473291, -0.2],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.85967, 0.84732911],
    }, // 8
    Vertex {
        position: [0.44147372, 0.2347359, -0.2],
        color: [1.0, 1.0, 1.0, 1.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // 9
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, 5, 6, 9, 6, 7, 9, 7, 8, 9];
