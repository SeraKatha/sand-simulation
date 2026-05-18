use macroquad::prelude::*;
use crate::Cell;
use crate::Simulation;
use crate::Grid;

pub struct WorldView<'a> {
    cells : &'a[Cell],
    world_size : IVec2,
}


impl<'a> WorldView<'a> {
    pub fn new(cells : &'a[Cell], world_size : IVec2) -> Self {
        Self { cells, world_size }
    }


    pub fn get_cell(&self, global_coord : IVec2) -> Cell {
        if global_coord.x < 0 {
            return Cell::STONE;
        };
        if global_coord.y < 0 {
            return Cell::STONE;
        };
        if global_coord.x >= self.world_size.x as i32 {
            return Cell::STONE;
        }
        if global_coord.y >= self.world_size.y as i32 {
            return Cell::STONE;
        }
        let local_coord = global_coord % (Simulation::CHUNK_SIZE as i32); 
        let local_index = Grid::map2Dto1D(local_coord, IVec2::ONE * (Simulation::CHUNK_SIZE as i32));

        let chunk_coord = global_coord / (Simulation::CHUNK_SIZE as i32); 
        let chunk_index = Grid::map2Dto1D(chunk_coord, self.world_size / (Simulation::CHUNK_SIZE as i32));
        let global_index = chunk_index * Simulation::CELLS_PER_CHUNK + local_index;
        // println!("A: {global_index}");
        // println!("{chunk_index}");
        return self.cells[global_index];
    }


    pub fn size(&self) -> IVec2 {
        self.world_size
    }
}