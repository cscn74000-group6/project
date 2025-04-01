#[derive(Debug)]
pub enum State {
    OPEN = 1,
    CLOSED = 0,
}

#[derive(Debug)]
pub struct StateMachine {
    state: State,
}

impl StateMachine {
    /// Initialize state machine.
    /// Default state is State::OPEN.
    pub fn new() -> StateMachine {
        StateMachine{
            state: State::OPEN
        }
    }

    /// Set state to State::OPEN.
    pub fn open(&mut self) {
        self.state = State::OPEN;
    }

    /// Set state to State::CLOSED.
    pub fn close(&mut self) {
        self.state = State::CLOSED;
    }

    /// Get state from state machine
    /// Does not copy or borrow state.
    pub fn get_state(&self) -> State {
        match self.state {
            State::OPEN => State::OPEN,
            State::CLOSED => State::CLOSED
        }
    }
}
