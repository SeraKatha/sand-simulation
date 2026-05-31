use macroquad::prelude::*;
use crate::Simulation;

pub fn map_2d_to_1d(coord: IVec2, grid_size: IVec2) -> usize {
    return (coord.y * grid_size.x + coord.x) as usize;
}

pub fn map_1d_to_2d(index: usize, grid_size: IVec2) -> IVec2 {
    let x = (index as i32) % grid_size.x;
    let y = (index as i32) / grid_size.x;
    return ivec2(x, y);
}

pub fn global_coords_to_index(global_coord: IVec2, world_size: IVec2) -> usize {
    let chunk_coord = global_coord / Simulation::CHUNK_SIZE_XY;
    let chunk_index = map_2d_to_1d(chunk_coord, world_size / Simulation::CHUNK_SIZE_XY);
    let local_coord = global_coord % Simulation::CHUNK_SIZE_XY;
    let local_index = map_2d_to_1d(local_coord, Simulation::CHUNK_SIZE_XY);
    return chunk_index * Simulation::CELLS_PER_CHUNK + local_index;
}