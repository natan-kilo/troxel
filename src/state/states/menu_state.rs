use crate::state::traits::Stateful;
use std::any::Any;
use wgpu::{Device, Queue, SwapChain, SwapChainDescriptor};
use winit::event::WindowEvent;
use winit_input_helper::WinitInputHelper;

pub struct MenuState {}

impl MenuState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Stateful for MenuState {
    fn render(&mut self, frame: &wgpu::SwapChainOutput, encoder: &mut wgpu::CommandEncoder) {
        unimplemented!()
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        unimplemented!()
    }

    fn input(&mut self, input: &WinitInputHelper) -> bool {
        false
    }

    fn resize(
        &mut self,
        device: &mut Device,
        sc_desc: &mut SwapChainDescriptor,
        size: &winit::dpi::PhysicalSize<u32>,
    ) {
        unimplemented!()
    }

    fn id(&self) -> usize {
        super::state_ids::MENU
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        unimplemented!()
    }
}
