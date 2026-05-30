pub struct Pulse {
    base_freq : f32,
    speed_scale : f32,
    time_since_last_tick : f32,
}

impl Pulse {
    pub fn new(base_freq : f32) -> Self {
        return Self { base_freq, speed_scale : 1.0, time_since_last_tick: 0.0 };
    }

    pub fn tick(&mut self, delta_time : f32) -> bool {
        // Round delta_time to a integer frame rate to prevent stuttering from slight FPS variation e.g. 59.9
        self.time_since_last_tick += 1.0 / (1.0 / (delta_time * self.speed_scale)).round();
        if self.time_since_last_tick >= 1.0 / self.base_freq {
            self.time_since_last_tick -= 1.0 / self.base_freq;
            return true;
        }
        else {
            return false;
        }
    }

    pub fn get_speed_scale_mut(&mut self) -> &mut f32 {
        &mut self.speed_scale
    }
}
