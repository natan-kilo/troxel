#![allow(dead_code, unused_imports)]

use wgpu::ShaderModule;

pub fn create_shader_module(
    shader_source: &str,
    shader_type: glsl_to_spirv::ShaderType,
    device: &wgpu::Device,
) -> ShaderModule {
    let spirv = glsl_to_spirv::compile(shader_source, shader_type).unwrap();
    let shader_data = wgpu::read_spirv(spirv).unwrap();
    let shader_module = device.create_shader_module(&shader_data);

    shader_module
}
