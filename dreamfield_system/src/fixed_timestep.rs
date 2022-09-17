/// Fixed timestep - https://gafferongames.com/post/fix_your_timestep/
pub struct FixedTimestep {
    fixed_timestep: f64,
    actual_time: f64,
    sim_time: f64,
    accumulator: f64
}

impl FixedTimestep {
    pub fn new(fixed_timestep: f64, actual_time: f64) -> Self {
        FixedTimestep {
            fixed_timestep,
            actual_time,
            sim_time: 0.0,
            // Set to fixed_timestep because we want it to run at once initially instead of having
            // to wait for one timestep.
            accumulator: fixed_timestep,
        }
    }

    pub fn update_actual_time(&mut self, actual_time: f64) {
        let frame_time = actual_time - self.actual_time;
        self.actual_time = actual_time;
        self.accumulator += frame_time;
    }

    pub fn should_update(&mut self) -> bool {
        if self.accumulator >= self.fixed_timestep {
            self.accumulator -= self.fixed_timestep;
            self.sim_time += self.fixed_timestep;
            true
        }
        else {
            false
        }
    }

    pub fn sim_time(&self) -> f64 {
        self.sim_time
    }
}
