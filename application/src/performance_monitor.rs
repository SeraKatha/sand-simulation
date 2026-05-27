use std::{iter::Sum, ops::{Add, Div}, time::{Duration, Instant}};

use macroquad::time::get_frame_time;

struct SmoothingBuffer<T : Default + Copy + Add<T, Output = T> + Div<u32, Output = T>, const N : usize> {
    data : [T; N],
    value : T,
    first : bool,
    index : usize
}

impl <T : Default + Copy + Add<T, Output = T> + Div<u32, Output = T>, const N : usize> SmoothingBuffer<T, N>  {
    pub fn new() -> Self {
        Self { data: [T::default(); N], first: true, index : 0, value : T::default() }
    }

    pub fn set(&mut self, value : T) {
        if self.first {
            self.data.fill_with(|| value);
        }
        else {
            self.data[self.index] = value
        }
        if self.index == 0 {
            self.value = self.data.iter().fold(T::default(), |a, b| a + *b) / (N as u32);
        }
        self.index = (self.index + 1) % N;
    }

    pub fn get(&self) -> T {
        return self.value;
    }
}

pub struct PerformanceMonitor {
    simulation_duration : SmoothingBuffer<Duration, 60>,
    rendering_duration : SmoothingBuffer<Duration, 60>,
    frame_duration : SmoothingBuffer<Duration, 60>,
}

fn measure(mut f : impl FnMut()->()) -> Duration {
    let time_tick_pre = Instant::now();
    f();
    let time_tick_post = Instant::now();
    return time_tick_post.duration_since(time_tick_pre);
} 

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            simulation_duration: SmoothingBuffer::new(),
            rendering_duration: SmoothingBuffer::new(),
            frame_duration: SmoothingBuffer::new(),
        }
    }

    pub fn meassure_simulation(&mut self, f : impl FnMut()->()) {
        self.simulation_duration.set(measure(f));
    }
    
    pub fn meassure_rendering(&mut self, f : impl FnMut()->()) {
        self.rendering_duration.set(measure(f));
    }

    pub fn meassure_frame(&mut self) {
        self.frame_duration.set(Duration::from_secs_f32(get_frame_time()));
    }

    pub fn get_simulation_duration(&self) -> Duration {
        return self.simulation_duration.get();
    }
    
    pub fn get_rendering_duration(&self) -> Duration {
        return self.rendering_duration.get();
    }

    pub fn get_frame_duration(&self) -> Duration {
        return self.frame_duration.get();
    }

    pub fn get_fps(&self) -> f32 {
        return 1.0 / self.get_frame_duration().as_secs_f32()
    }
}