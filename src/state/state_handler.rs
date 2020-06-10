use crate::state::states::none_sate::NoneState;
use crate::state::traits::Stateful;

pub struct StateHandler {
    pub states: Vec<Box<dyn Stateful>>,
    pub current_state_in_vec: usize,
    pub current_state_id: usize,
}

impl StateHandler {
    pub fn new(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        let mut states: Vec<Box<dyn Stateful>> = Vec::new();
        let none_state = Box::new(
            NoneState::new(device, sc_desc)
        );
        let none_state_id = none_state.id();
        states.push(none_state);

        let current_state_in_vec = states.iter()
            .position(|s| s.id() == none_state_id)
            .unwrap();

        Self {
            states,
            current_state_in_vec,
            current_state_id: none_state_id,
        }
    }

    pub fn add_state(&mut self, state: Box<dyn Stateful>) {
        self.states.push(state)
    }

    pub fn add_states(&mut self, states: Vec<Box<dyn Stateful>>) {
        self.states.extend(states);
    }

    pub fn remove_state(&mut self, state_id: usize) {
        assert_ne!(state_id, 0);
        assert!(state_id <= self.states.len());

        if self.current_state_id == state_id {
            self.current_state_id = 0;
        }

        &self
            .states
            .remove(self.states.iter().position(|s| s.id() == state_id).unwrap());
    }

    pub fn set_state(&mut self, state_id: usize) {
        println!("{}", state_id);
        let current_state_in_vec = self.states.iter()
            .position(|s| s.id() == state_id).unwrap();
        self.current_state_in_vec = current_state_in_vec;
        self.current_state_id = state_id
    }
}
