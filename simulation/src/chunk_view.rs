use macroquad::prelude::*;
use crate::{Cell, Grid, Simulation};

pub struct ChunkViewMut<'a> {
    cells : &'a mut [Cell],
    chunk_index : usize,
}

impl<'a> ChunkViewMut<'a> {
    pub fn new(chunk_index : usize, cells : &'a mut [Cell]) -> Self {
        return Self { chunk_index, cells }
    }

    pub fn write_cell(&mut self, cell : Cell, local_coord : IVec2) {
        let local_index = Grid::map2Dto1D(local_coord, IVec2::ONE * (Simulation::CHUNK_SIZE as i32));
        self.cells[local_index] = cell;
    }

    pub fn index(&self) -> usize {
        self.chunk_index
    } 
}