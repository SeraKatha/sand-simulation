use crate::{grid, Simulation};
use macroquad::prelude::*;

// Views to map the 1D slice of all cells in an chunk to 2D coordinate system
pub struct ChunkViewMut<'a, T: Copy> {
    cells: &'a mut [T],
    chunk_index: usize,
    world_size: IVec2,
}

impl<'a, T: Copy> ChunkViewMut<'a, T> {
    pub fn new(chunk_index: usize, cells: &'a mut [T], world_size: IVec2) -> Self {
        return Self {
            chunk_index,
            cells,
            world_size,
        };
    }

    pub fn get_cell(&self, local_coord: IVec2) -> T {
        let local_index = grid::map_2d_to_1d(local_coord, Simulation::CHUNK_SIZE_XY);
        return self.cells[local_index];
    }

    pub fn set_cell(&mut self, cell: T, local_coord: IVec2) {
        let local_index = grid::map_2d_to_1d(local_coord, Simulation::CHUNK_SIZE_XY);
        self.cells[local_index] = cell;
    }

    pub fn get_chunk_index(&self) -> usize {
        self.chunk_index
    }

    pub fn get_chunk_coord(&self) -> IVec2 {
        return grid::map_1d_to_2d(
            self.chunk_index,
            self.world_size / Simulation::CHUNK_SIZE_XY,
        );
    }
}

pub struct ChunkView<'a, T: Copy> {
    cells: &'a [T],
    chunk_index: usize,
    world_size: IVec2,
}

impl<'a, T: Copy> ChunkView<'a, T> {
    pub fn new(chunk_index: usize, cells: &'a [T], world_size: IVec2) -> Self {
        return Self {
            chunk_index,
            cells,
            world_size,
        };
    }

    pub fn get_cell(&self, local_coord: IVec2) -> T {
        let local_index = grid::map_2d_to_1d(local_coord, Simulation::CHUNK_SIZE_XY);
        return self.cells[local_index];
    }

    pub fn get_cells(&'a self) -> &'a [T] {
        self.cells
    }

    pub fn get_chunk_index(&self) -> usize {
        self.chunk_index
    }

    pub fn get_chunk_coord(&self) -> IVec2 {
        return grid::map_1d_to_2d(
            self.chunk_index,
            self.world_size / Simulation::CHUNK_SIZE_XY,
        );
    }
}
