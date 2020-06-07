use std::any::Any;
use std::ops::Index;

pub trait Stateful: Any {
    fn render(&mut self, frame: &wgpu::SwapChainOutput, encoder: &mut wgpu::CommandEncoder);
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
    fn input(&mut self, event: &winit::event::WindowEvent) -> bool;
    fn resize(
        &mut self,
        device: &mut wgpu::Device,
        sc_desc: &mut wgpu::SwapChainDescriptor,
        size: &winit::dpi::PhysicalSize<u32>,
    );
    fn id(&self) -> usize;
    fn box_eq(&self, other: &dyn Any) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl IntoIterator for Box<dyn Stateful> {
    type Item = Box<dyn Stateful>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[allow(unconditional_recursion)]
    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}
