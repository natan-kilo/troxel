#![allow(dead_code, unused_imports)]

use crate::config::WINDOW_SIZE;
use gfx_hal::{
    adapter::Adapter,
    window::{Extent2D, Surface},
    Instance,
};
use glsl_to_spirv::ShaderType;
use std::{mem::ManuallyDrop, ptr};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub fn compile_shader(glsl: &str, shader_type: ShaderType) -> Vec<u32> {
    use std::io::{Cursor, Read};

    let mut compiled_file =
        glsl_to_spirv::compile(glsl, shader_type).expect("Failed to compile shader");

    let mut spirv_bytes = vec![];
    compiled_file.read_to_end(&mut spirv_bytes).unwrap();

    let spirv = gfx_hal::pso::read_spirv(Cursor::new(&spirv_bytes)).expect("Invalid SPIR-V");

    spirv
}

pub fn drop<T>(object: T) -> ManuallyDrop<T> {
    ManuallyDrop::new(object)
}

pub fn window_sizes(event_loop: &EventLoop<()>) -> (LogicalSize<u32>, PhysicalSize<u32>) {
    let dpi = event_loop.primary_monitor().scale_factor();
    let logical_size: LogicalSize<u32> = WINDOW_SIZE.into();
    let physical_size: PhysicalSize<u32> = logical_size.to_physical(dpi);
    (logical_size, physical_size)
}

pub fn new_window(title: &str, size: LogicalSize<u32>, event_loop: &EventLoop<()>) -> Window {
    WindowBuilder::new()
        .with_title(title)
        .with_inner_size(size)
        .build(&event_loop)
        .expect("Failed to create window")
}

pub fn new_surface_extent(physical_size: &PhysicalSize<u32>) -> Extent2D {
    Extent2D {
        width: physical_size.width,
        height: physical_size.height,
    }
}

pub fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Sized + Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}
