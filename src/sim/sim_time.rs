/// The SimTime component
pub struct SimTime {
    pub sim_time: f64,
    pub sim_time_delta: f64
}

impl SimTime {
    pub fn new(sim_time: f64, sim_time_delta: f64) -> Self {
        Self {
            sim_time,
            sim_time_delta
        }
    }
}
