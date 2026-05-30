use macroquad::prelude::*;
use crate::{Cell, Grid, Simulation};

pub struct ChunkViewMut<'a, T : Copy> {
    cells : &'a mut [T],
    chunk_index : usize,
}

impl<'a, T : Copy> ChunkViewMut<'a, T> {
    pub fn new(chunk_index : usize, cells : &'a mut [T]) -> Self {
        return Self { chunk_index, cells }
    }

    pub fn get_cell(&self, local_coord : IVec2) -> T {
        let local_index = Grid::map_2d_to_1d(local_coord, IVec2::ONE * (Simulation::CHUNK_SIZE as i32));
        return self.cells[local_index];
    }

    pub fn set_cell(&mut self, cell : T, local_coord : IVec2) {
        let local_index = Grid::map_2d_to_1d(local_coord, IVec2::ONE * (Simulation::CHUNK_SIZE as i32));
        self.cells[local_index] = cell;
    }

    pub fn index(&self) -> usize {
        self.chunk_index
    } 
}


pub struct ChunkView<'a, T : Copy> {
    cells : &'a [T],
    chunk_index : usize,
    chunk_coord : IVec2
}

impl<'a, T : Copy> ChunkView<'a, T> {
    pub fn new(chunk_index : usize, cells : &'a [T], chunk_coord : IVec2) -> Self {
        return Self { chunk_index, cells, chunk_coord }
    }

    pub fn get_cell(&self, local_coord : IVec2) -> T {
        let local_index = Grid::map_2d_to_1d(local_coord, IVec2::ONE * (Simulation::CHUNK_SIZE as i32));
        return self.cells[local_index];
    }

    pub fn get_cells(&'a self) -> &'a[T] {
        self.cells
    }

    pub fn get_chunk_index(&self) -> usize {
        self.chunk_index
    }

    pub fn get_chunk_coord(&self) -> IVec2 {
        self.chunk_coord
    }
}