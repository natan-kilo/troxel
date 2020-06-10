use std::any::Any;
use wgpu::{Device, Queue, SwapChain, SwapChainDescriptor};
use winit::event::WindowEvent;

use crate::state::traits::Stateful;
use crate::tools::camera;
use crate::tools::uniforms;
use crate::render::{buffer, texture, pipeline, shader};

pub struct ChaoticState {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    render_pipeline: wgpu::RenderPipeline,
    depth_texture: texture::Texture,

    camera: camera::Camera,
    camera_controller: camera::CameraController,

    uniforms: uniforms::Uniforms,
    uniform_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,

    size: winit::dpi::PhysicalSize<u32>,
}

impl ChaoticState {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &wgpu::SwapChainDescriptor,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> Self {
        let camera = camera::Camera::new(sc_desc);

        let camera_controller = camera::CameraController::default();

        let mut uniforms = uniforms::Uniforms::new();
        uniforms.update_view_proj(camera.to_matrix());


        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let (uniform_bind_group, uniform_bind_group_layout) = buffer::create_uniform_bind_group(
            device,
            wgpu::ShaderStage::VERTEX,
            0,
            "uniform_bind_group_layout",
            0,
            "uniform_bind_group",
            &uniform_buffer,
            std::mem::size_of_val(&uniforms)
        );

        let vs_module = shader::create_shader_module(
            include_str!("../../../assets/shaders/default_vertex.glsl"),
            ShaderType::Vertex,
            &device,
        );

        let fs_module = shader::create_shader_module(
            include_str!("../../../assets/shaders/default_fragment.glsl"),
            ShaderType::Fragment,
            &device,
        );

        let depth_texture = texture::Texture::new_depth(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
           bind_group_layouts: &[&uniform_bind_group_layout],
        });
        
        let render_pipeline = pipeline::create_render_pipeline(
            &device,
            &render_pipeline_layout,
            wgpu::PrimitiveTopology::TriangleList,
            &vs_module,
            &fs_module,
            sc_desc.format.clone(),
            texture::DEPTH_FORMAT,
            &[VertexC::desc()],
            true,
            "main",
        );

        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(VERTICES),
            wgpu::BufferUsage::VERTEX,
        );

        let index_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(INDICES),
            wgpu::BufferUsage::INDEX,
        );
        
        let num_indices = INDICES.len() as u32;
        
        Self {
            vertex_buffer,
            index_buffer,
            num_indices,
            render_pipeline,
            depth_texture,
            camera,
            camera_controller,
            uniforms,
            uniform_bind_group,
            uniform_buffer,
            size: size.clone(),
        }
    }
}

impl Stateful for ChaoticState {
    fn render(&mut self, frame: &wgpu::SwapChainOutput, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
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

        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);

        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);

        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.camera_controller.update(&mut self.camera);
        self.uniforms.update_view_proj(self.camera.to_matrix());

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
            std::mem::size_of::<uniforms::Uniforms>() as wgpu::BufferAddress,
        );

        queue.submit(&[encoder.finish()]);
    }

    fn input(&mut self, input: &WinitInputHelper) -> bool {
        self.camera_controller.input(input);
        false
    }

    fn resize(
        &mut self,
        device: &mut Device,
        sc_desc: &mut SwapChainDescriptor,
        size: &winit::dpi::PhysicalSize<u32>,
    ) {
        self.size = size.clone();
        self.depth_texture = texture::Texture::new_depth(&device, &sc_desc, "depth_texture");
    }

    fn id(&self) -> usize {
        super::state_ids::CHAOTIC
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        unimplemented!()
    }
}

use crate::types::VertexC;
use glsl_to_spirv::ShaderType;
use winit_input_helper::WinitInputHelper;

const VERTICES: &[VertexC] = &[
    VertexC { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5, 1.0] }, // 0
    VertexC { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5, 1.0] }, // 1
    VertexC { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5, 1.0] }, // 2
    VertexC { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5, 1.0] }, // 3
    VertexC { position: [0.44147372, 0.2347359, 0.0],color: [0.5, 0.0, 0.5, 1.0] }, // 4

    VertexC { position: [-10.0, -5.0, -10.0], color: [1.0, 0.0, 0.0, 1.0] }, // 5 R
    VertexC { position: [-10.0, -5.0, 10.0],color: [0.0, 1.0, 0.0, 1.0] }, // 6  G
    VertexC { position: [10.0, -5.0, -10.0], color: [0.0, 0.0, 1.0, 1.0] }, // 7 B
    VertexC { position: [10.0, -5.0, 10.0], color: [0.5, 0.5, 0.5, 1.0] }, // 8

    VertexC { position: [-10.0, -5.0, -10.0], color: [1.0, 0.0, 0.0, 1.0] }, // 9 R
    VertexC { position: [-10.0, -5.0, 10.0],color: [0.0, 1.0, 0.0, 1.0] }, // 10  G
    VertexC { position: [10.0, -5.0, -10.0], color: [0.0, 0.0, 1.0, 1.0] }, // 11 B
    VertexC { position: [10.0, -5.0, 10.0], color: [0.5, 0.5, 0.5, 1.0] }, // 12
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,

    7, 5, 6,
    6, 8, 7
];