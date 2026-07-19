use crate::state::RuntimeState;

pub struct RuntimeManager {
    state: RuntimeState,
}

impl RuntimeManager {

    pub fn new() -> Self {
        Self {
            state: RuntimeState::Stopped,
        }
    }

    pub fn state(&self) -> RuntimeState {
        self.state
    }

    pub fn start(&mut self) {
        self.state = RuntimeState::Running;
    }

    pub fn stop(&mut self) {
        self.state = RuntimeState::Stopped;
    }
}