use macroquad::prelude::*;
use crate::Grid;


fn is_in_bounds(global_coord: IVec2, world_size: IVec2) -> bool {
    return (global_coord.x >= 0)
        && (global_coord.y >= 0)
        && (global_coord.x < world_size.x)
        && (global_coord.y < world_size.y);
}

// Views to map the 1D slice of all cells in a world into CHUNKED 2D coordinate system
pub struct WorldView<'a, T: Copy> {
    cells: &'a [T],
    world_size: IVec2,
    border: T,
}

impl<'a, T: Copy> WorldView<'a, T> {
    pub fn new(cells: &'a [T], world_size: IVec2, border: T) -> Self {
        Self {
            cells,
            world_size,
            border,
        }
    }

    pub fn get_cell(&self, global_coord: IVec2) -> T {
        if is_in_bounds(global_coord, self.size()) {
            let global_index = Grid::global_coords_to_index(global_coord, self.size());
            return self.cells[global_index];
        } else {
            return self.border;
        }
    }

    pub fn size(&self) -> IVec2 {
        self.world_size
    }
}

pub struct WorldViewMut<'a, T: Copy> {
    cells: &'a mut [T],
    world_size: IVec2,
    border: T,
}

impl<'a, T: Copy> WorldViewMut<'a, T> {
    pub fn new(cells: &'a mut [T], world_size: IVec2, border: T) -> Self {
        Self {
            cells,
            world_size,
            border,
        }
    }

    pub fn get_cell(&self, global_coord: IVec2) -> T {
        if is_in_bounds(global_coord, self.size()) {
            let global_index = Grid::global_coords_to_index(global_coord, self.size());
            return self.cells[global_index];
        } else {
            return self.border;
        }
    }

    pub fn size(&self) -> IVec2 {
        self.world_size
    }

    pub fn set_cell(&mut self, global_coord: IVec2, cell: T) {
        if is_in_bounds(global_coord, self.world_size) {
            let global_index = Grid::global_coords_to_index(global_coord, self.size());
            self.cells[global_index] = cell;
        }
    }
}
