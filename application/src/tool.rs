use simulation::{Cell, Simulation};
use macroquad::prelude::*;


pub trait Tool {
    fn apply(&self, simulation : &mut Simulation, global_coord : IVec2);
}


pub struct Dropper {
    cell : Cell,
    size : i32,
}


impl Dropper {
    pub fn new(cell : Cell, size : i32) -> Dropper {
        Dropper {
            cell, size
        }
    }

    pub fn apply(&self, simulation : &mut Simulation, global_coord : IVec2) {
        for dx in 0..self.size {
            for dy in 0..self.size {
                simulation.write_cell(global_coord + ivec2(dx,dy), self.cell);
            }
        }
    }

    pub fn set_material(&mut self, cell : Cell) {
        self.cell = cell
    }

    pub fn get_size(&mut self) -> i32 {
        return self.size;
    }

    pub fn set_size(&mut self, size : i32) {
        self.size = size;
    }
}