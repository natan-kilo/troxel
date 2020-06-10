use crate::state::traits::Stateful;
use std::any::Any;
use winit::event::WindowEvent;
use crate::render::pipeline;
use crate::render;
use glsl_to_spirv::ShaderType;
use winit_input_helper::WinitInputHelper;

pub struct NoneState {
    render_pipeline: wgpu::RenderPipeline,
}

impl NoneState {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        let vs_module = render::shader::create_shader_module(
            include_str!("../../../assets/shaders/none.vert"),
            ShaderType::Vertex,
            &device,
        );

        let fs_module = render::shader::create_shader_module(
            include_str!("../../../assets/shaders/none.frag"),
            ShaderType::Fragment,
            &device,
        );

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[]
            }),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: sc_desc.format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
            ],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        Self {
            render_pipeline,
        }
    }
}

impl Stateful for NoneState {
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
                    a: 0.0,
                },
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1)
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {}

    fn input(&mut self, input: &WinitInputHelper) -> bool {
        false
    }

    fn resize(
        &mut self,
        device: &mut wgpu::Device,
        sc_desc: &mut wgpu::SwapChainDescriptor,
        size: &winit::dpi::PhysicalSize<u32>,
    ) {}

    fn id(&self) -> usize {
        super::state_ids::NONE
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        unimplemented!()
    }
}
