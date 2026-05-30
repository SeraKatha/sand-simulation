use macroquad::math::IVec2;
use macroquad::prelude::*;

mod double_buffer;
use double_buffer::DoubleBuffer;

use macroquad::rand::ChooseRandom;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

mod chunk_view;
pub use chunk_view::ChunkView;
pub use chunk_view::ChunkViewMut;
mod world_view;
pub use world_view::WorldView;
pub use world_view::WorldViewMut;

pub enum Error {
    InvalidWorldSize,
    CellOutOfBounds,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Cell {
    AIR,
    SAND,
    STONE,
    WATER,
}

pub struct Grid {}

impl Grid {
    pub fn map_2d_to_1d(coord: IVec2, grid_size: IVec2) -> usize {
        return (coord.y * grid_size.x + coord.x) as usize;
    }

    pub fn map_1d_to_2d(index: usize, grid_size: IVec2) -> IVec2 {
        let x = (index as i32) % grid_size.x;
        let y = (index as i32) / grid_size.x;
        return ivec2(x, y);
    }

    pub fn global_coords_to_index(global_coord: IVec2, world_size: IVec2) -> usize {
        let local_coord = global_coord % (Simulation::CHUNK_SIZE as i32);
        let local_index = Grid::map_2d_to_1d(local_coord, IVec2::ONE * (Simulation::CHUNK_SIZE as i32));
        let chunk_coord = global_coord / (Simulation::CHUNK_SIZE as i32);
        let chunk_index = Grid::map_2d_to_1d(chunk_coord, world_size / (Simulation::CHUNK_SIZE as i32));
        return chunk_index * Simulation::CELLS_PER_CHUNK + local_index;
    }
}

pub struct Simulation {
    cells: DoubleBuffer<Vec<Cell>>,
    push_buffer: Vec<IVec2>,
    pull_buffer: Vec<[bool; Self::NEIGHBOR_COUNT]>,
    world_size: IVec2,
}

impl Simulation {
    pub const CHUNK_SIZE: usize = 32;
    pub const CHUNK_SIZE_XY: IVec2 = ivec2(Self::CHUNK_SIZE as i32, Self::CHUNK_SIZE as i32);
    pub const CELLS_PER_CHUNK: usize = Self::CHUNK_SIZE * Self::CHUNK_SIZE;
    const NEIGHBOR_COUNT: usize = 8;
    const NEIGHBOR_IDX2OFFSET: [IVec2; Self::NEIGHBOR_COUNT] = [
        ivec2(-1, -1),
        ivec2(0, -1),
        ivec2(1, -1),
        ivec2(-1, 0),
        ivec2(1, 0),
        ivec2(-1, -1),
        ivec2(0, -1),
        ivec2(1, -1),
    ];

    pub fn new(world_size: IVec2) -> Result<Simulation, Error> {
        if (world_size.x as usize % Self::CHUNK_SIZE) != 0 {
            return Err(Error::InvalidWorldSize);
        }
        if (world_size.y as usize % Self::CHUNK_SIZE) != 0 {
            return Err(Error::InvalidWorldSize);
        }
        let num_of_cells: usize = (world_size.x * world_size.y) as usize;
        let cells = Vec::from_iter(std::iter::repeat_with(|| Cell::AIR).take(num_of_cells));

        let push_buffer = Vec::from_iter(std::iter::repeat_with(|| IVec2::ZERO).take(num_of_cells));
        let pull_buffer = Vec::from_iter(std::iter::repeat_with(|| [false; 8]).take(num_of_cells));

        return Ok(Simulation {
            cells: DoubleBuffer::new(cells),
            world_size,
            push_buffer,
            pull_buffer,
        });
    }

    pub fn num_of_chunks_xy(&self) -> IVec2 {
        return self.world_size / (Self::CHUNK_SIZE as i32);
    }

    pub fn num_of_chunks(&self) -> usize {
        let num_of_chunks = self.num_of_chunks_xy();
        return (num_of_chunks.x * num_of_chunks.y) as usize;
    }

    fn calc_push_vector(read_world: &WorldView<Cell>, global_coord: IVec2) -> IVec2 {
        let cell_center = read_world.get_cell(global_coord);
        let cell_above = read_world.get_cell(global_coord + ivec2(0, -1));
        let cell_below = read_world.get_cell(global_coord + ivec2(0, 1));
        let (offset_a, offset_b) = if ::rand::random_bool(0.5) {
            (ivec2(-1, 0), ivec2(1, 0))
        } else {
            (ivec2(1, 0), ivec2(-1, 0))
        };
        let cell_below_a = read_world.get_cell(global_coord + ivec2(0, 1) + offset_a);
        let cell_below_b = read_world.get_cell(global_coord + ivec2(0, 1) + offset_b);
        let cell_above_a = read_world.get_cell(global_coord - ivec2(0, 1) + offset_a);
        let cell_above_b = read_world.get_cell(global_coord - ivec2(0, 1) + offset_b);
        let cell_side_a = read_world.get_cell(global_coord + offset_a);
        let cell_side_b = read_world.get_cell(global_coord + offset_b);
        return match cell_center {
            Cell::AIR => IVec2::ZERO,
            Cell::SAND => {
                if cell_below == Cell::AIR {
                    ivec2(0, 1)
                } else if cell_below_a == Cell::AIR {
                    ivec2(0, 1) + offset_a
                } else if cell_below_b == Cell::AIR {
                    ivec2(0, 1) + offset_b
                } else if cell_below == Cell::WATER {
                    ivec2(0, 1)
                } else if cell_below_a == Cell::WATER {
                    ivec2(0, 1) + offset_a
                } else if cell_below_b == Cell::WATER {
                    ivec2(0, 1) + offset_b
                } else {
                    IVec2::ZERO
                }
            }
            Cell::STONE => IVec2::ZERO,
            Cell::WATER => {
                if cell_below == Cell::AIR {
                    ivec2(0, 1)
                } else if cell_below_a == Cell::AIR {
                    ivec2(0, 1) + offset_a
                } else if cell_below_b == Cell::AIR {
                    ivec2(0, 1) + offset_b
                } else if cell_side_a == Cell::AIR {
                    offset_a
                } else if cell_side_b == Cell::AIR {
                    offset_b
                } else if cell_above == Cell::SAND {
                    ivec2(0, -1)
                } else if cell_above_a == Cell::SAND {
                    ivec2(0, -1) + offset_a
                } else if cell_above_b == Cell::SAND {
                    ivec2(0, -1) + offset_b
                } else {
                    IVec2::ZERO
                }
            }
        };
    }

    fn update_push_vectors(
        chunk_index: usize,
        write_chunk: &mut [IVec2],
        read_world: &WorldView<Cell>,
    ) {
        let chunk_coord =
            Grid::map_1d_to_2d(chunk_index, read_world.size() / (Self::CHUNK_SIZE as i32));
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord =
                Grid::map_1d_to_2d(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let global_coord = chunk_coord * (Self::CHUNK_SIZE as i32) + local_coord;
            write_chunk[local_index] = Self::calc_push_vector(read_world, global_coord);
        }
    }

    fn pick_pulls(
        read_world_push: &WorldView<IVec2>,
        write_chunk_pull: &mut ChunkViewMut<[bool; Self::NEIGHBOR_COUNT]>,
    ) {
        let chunk_coord = Grid::map_1d_to_2d(
            write_chunk_pull.get_chunk_index(),
            read_world_push.size() / (Self::CHUNK_SIZE as i32),
        );
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord =
                Grid::map_1d_to_2d(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let global_coord = chunk_coord * (Self::CHUNK_SIZE as i32) + local_coord;
            let mut size = 0;
            let mut pulls = [0; Self::NEIGHBOR_COUNT];
            for i in 0..Self::NEIGHBOR_COUNT {
                let offset = Self::NEIGHBOR_IDX2OFFSET[i];
                let push = read_world_push.get_cell(global_coord + offset);
                if push == -offset {
                    pulls[size] = i;
                    size += 1;
                }
            }
            let mut new_pulls = [false; 8];
            if let Some(picked_pull) = pulls[0..size].choose() {
                new_pulls[*picked_pull] = true;
            }
            write_chunk_pull.set_cell(new_pulls, local_coord);
        }
    }

    fn resolve_movements(
        write_chunk: &mut ChunkViewMut<Cell>,
        read_world: &WorldView<Cell>,
        read_world_push: &WorldView<IVec2>,
        read_world_pull: &WorldView<[bool; Self::NEIGHBOR_COUNT]>,
    ) {
        let chunk_coord = Grid::map_1d_to_2d(
            write_chunk.get_chunk_index(),
            read_world.size() / (Self::CHUNK_SIZE as i32),
        );
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord =
                Grid::map_1d_to_2d(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let global_coord = chunk_coord * (Self::CHUNK_SIZE as i32) + local_coord;
            let pull_field = read_world_pull.get_cell(global_coord);
            let mut cell = read_world.get_cell(global_coord);
            let center_push = read_world_push.get_cell(global_coord);
            let target_pull = read_world_pull.get_cell(global_coord + center_push);
            for n in 0..Self::NEIGHBOR_COUNT {
                let offset = Self::NEIGHBOR_IDX2OFFSET[n];
                if target_pull[n] && center_push == -offset {
                    cell = read_world.get_cell(global_coord + center_push);
                }
            }
            for n in 0..Self::NEIGHBOR_COUNT {
                let offset = Self::NEIGHBOR_IDX2OFFSET[n];
                let neighbor_push = read_world_push.get_cell(global_coord + offset);
                if pull_field[n] && neighbor_push == -offset {
                    cell = read_world.get_cell(global_coord + offset);
                }
            }

            write_chunk.set_cell(cell, local_coord);
        }
    }

    pub fn tick(&mut self) {
        let mut array = [0i32; 4];
        let world_size = self.size();

        let (read_buffer, write_buffer) = self.cells.pick_read_and_write_buffer();
        let read_world = WorldView::new(read_buffer, world_size, Cell::STONE);
        for x in 0..world_size.x {
            for y in 0..world_size.y {
                array[read_world.get_cell(ivec2(x, y)) as usize] += 1;
            }
        }
        self.push_buffer
            .par_chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .for_each(|(chunk_index, chunk)| {
                Self::update_push_vectors(chunk_index, chunk, &read_world)
            });

        let read_world_push = WorldView::new(&self.push_buffer, world_size, IVec2::ZERO);

        self.pull_buffer
            .par_chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .map(|(chunk_index, chunk)| ChunkViewMut::new(chunk_index, chunk, world_size))
            .for_each(|mut write_chunk_pull| {
                Self::pick_pulls(&read_world_push, &mut write_chunk_pull)
            });

        let read_world_pull =
            WorldView::new(&self.pull_buffer, world_size, [false; Self::NEIGHBOR_COUNT]);

        write_buffer
            .par_chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .map(|(chunk_index, chunk)| ChunkViewMut::new(chunk_index, chunk, world_size))
            .for_each(|mut write_chunk| {
                Self::resolve_movements(
                    &mut write_chunk,
                    &read_world,
                    &read_world_push,
                    &read_world_pull,
                )
            });
    }

    // Just copies the read buffer to write buffer. useful for skipping simulation ticks but still use write_cell.
    pub fn pass(&mut self) {
        let (read_buffer, write_buffer) = self.cells.pick_read_and_write_buffer();
        write_buffer.copy_from_slice(&read_buffer);
    }

    pub fn write_cell(&mut self, global_coord: IVec2, cell: Cell) {
        let (_read_buffer, write_buffer) = self.cells.pick_read_and_write_buffer();
        let mut world_mut = WorldViewMut::new(write_buffer, self.world_size, Cell::STONE);
        world_mut.set_cell(global_coord, cell);
    }

    pub fn get_chunk_by_coord<'a>(&'a self, chunk_coord: IVec2) -> ChunkView<'a, Cell> {
        let chunk_index = Grid::map_2d_to_1d(chunk_coord, self.num_of_chunks_xy());
        return self.get_chunk_by_index(chunk_index);
    }

    pub fn get_chunk_by_index<'a>(&'a self, chunk_index: usize) -> ChunkView<'a, Cell> {
        let global_index = Self::CELLS_PER_CHUNK * chunk_index;
        let read_buffer = self.cells.get_read_buffer();
        let begin = global_index;
        let end = global_index + Self::CELLS_PER_CHUNK;
        return ChunkView::new(chunk_index, &read_buffer[begin..end], self.size());
    }

    pub fn swap_buffers(&mut self) {
        self.cells.swap();
    }

    pub fn size(&self) -> IVec2 {
        return self.world_size;
    }
}
