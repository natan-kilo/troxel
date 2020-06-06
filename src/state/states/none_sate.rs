use crate::state::traits::Stateful;
use std::any::Any;

pub struct NoneState { }

impl NoneState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Stateful for NoneState {
    fn render(&mut self) {
        unimplemented!()
    }

    fn update(&mut self) {
        unimplemented!()
    }

    fn input(&mut self) {
        unimplemented!()
    }

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