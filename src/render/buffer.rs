pub fn create_uniform_bind_group_layout(
    device: &wgpu::Device,
    visibility: wgpu::ShaderStage,
    binding: u32,
    label: &str,
) -> wgpu::BindGroupLayout {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
        }],
        label: Some(label)
    });

    bind_group_layout
}

pub fn create_uniform_bind_group(
    device: &wgpu::Device,
    visibility: wgpu::ShaderStage,
    binding_desc: u32,
    label_desc: &str,
    binding_group: u32,
    label_group: &str,
    uniform_buffer: &wgpu::Buffer,
    mem_size: usize,
) -> (wgpu::BindGroup, wgpu::BindGroupLayout) {
    let layout = create_uniform_bind_group_layout(
        device,
        visibility,
        binding_desc,
        label_desc,
    );

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        bindings: &[wgpu::Binding {
            binding: binding_group,
            resource: wgpu::BindingResource::Buffer {
                buffer: &uniform_buffer,
                range: 0..mem_size as wgpu::BufferAddress,
            },
        }],
        label: Some(label_group),
    });

    (bind_group, layout)
}