use crate::state::traits::Stateful;
use crate::state::states::none_sate::NoneState;

pub struct StateHandler {
    pub states: Vec<Box<dyn Stateful>>,
    pub current_state_id: usize,
}

impl StateHandler {
    pub fn new() -> Self {
        let mut states: Vec<Box<dyn Stateful>> = Vec::new();
        let none_state = Box::new(NoneState::new());
        let none_state_id = none_state.id();
        states.push(none_state);
        Self {
            states,
            current_state_id: none_state_id
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

        &self.states.remove(
            self.states.iter().position(|s| s.id() == state_id).unwrap()
        );
    }

    pub fn set_state(&mut self, state_id: usize) {
        assert!(state_id <= self.states.len());
        self.current_state_id = state_id
    }
}