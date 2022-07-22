use std::collections::VecDeque;

use crate::sim::input::{InputName, InputState};

const MAX_SNAPSHOTS: usize = 600;

pub trait SnapshotState<T> {
    fn new() -> T;
    fn snapshot(&self) -> T;
    fn simulate(&mut self, sim_time: f64, input_state: &InputState);
}

pub struct RewindableGameState<T: SnapshotState<T>> {
    cur_state: T,
    state_history: VecDeque<T>
}

impl<T: SnapshotState<T>> RewindableGameState<T> {
    pub fn new() -> Self {
        RewindableGameState {
            cur_state: T::new(),
            state_history: VecDeque::new()
        }
    }

    pub fn cur_state(&self) -> &T {
        &self.cur_state
    }

    pub fn simulate(&mut self, sim_time: f64, input_state: &InputState) {
        if input_state.inputs[InputName::Rewind as usize] {
            if let Some(last_state) = self.state_history.pop_back() {
                self.cur_state = last_state
            }
        }
        else {
            // Add snapshot to history
            self.state_history.push_back(self.cur_state.snapshot());

            // Limit number of snapshots
            while self.state_history.len() > MAX_SNAPSHOTS {
                self.state_history.pop_front();
            }

            // Simulate
            self.cur_state.simulate(sim_time, input_state);
        }
    }
}
