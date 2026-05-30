// Simple double buffer implementation
pub struct DoubleBuffer<T: Clone> {
    buffer_a: T,
    buffer_b: T,
    swapped: bool,
}

impl<T: Clone> DoubleBuffer<T> {
    pub fn new(data: T) -> DoubleBuffer<T> {
        let buffer_a = data;
        let buffer_b = buffer_a.clone();
        return Self {
            buffer_a,
            buffer_b,
            swapped: false,
        };
    }

    pub fn pick_read_and_write_buffer<'a>(&'a mut self) -> (&'a T, &'a mut T) {
        if self.swapped {
            (&self.buffer_b, &mut self.buffer_a)
        } else {
            (&self.buffer_a, &mut self.buffer_b)
        }
    }

    pub fn get_read_buffer(&self) -> &T {
        if self.swapped {
            &self.buffer_b
        } else {
            &self.buffer_a
        }
    }

    pub fn swap(&mut self) {
        self.swapped = !self.swapped;
    }
}
