use crate::state::traits::Stateful;
use std::any::Any;
use wgpu::{Device, SwapChain, Queue, SwapChainDescriptor};
use winit::event::WindowEvent;

pub struct ChaoticState { }

impl ChaoticState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Stateful for ChaoticState {
    fn render(&mut self, frame: &wgpu::SwapChainOutput, encoder: &mut wgpu::CommandEncoder) {
        unimplemented!()
    }

    fn update(&mut self, device: &mut Device, queue: &mut Queue) {
        unimplemented!()
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        unimplemented!()
    }

    fn resize(&mut self, device: &mut Device, sc_desc: &mut SwapChainDescriptor, size: &winit::dpi::PhysicalSize<u32>) {
        unimplemented!()
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