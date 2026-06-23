use macroquad::math::IVec2;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

mod double_buffer;
use double_buffer::DoubleBuffer;

use macroquad::rand::ChooseRandom;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::{ParallelSliceMut};

mod chunk_view;
pub use chunk_view::ChunkView;
pub use chunk_view::ChunkViewMut;
mod world_view;
pub use world_view::WorldView;
pub use world_view::WorldViewMut;
mod cell;
pub mod grid;
pub use cell::Cell;

mod world_gen;
pub use world_gen::*;

pub enum Error {
    InvalidWorldSize,
    CellOutOfBounds,
}

pub struct Simulation {
    cells: DoubleBuffer<Vec<Cell>>,
    transmutation_buffer: Vec<Cell>,
    push_buffer: Vec<IVec2>,
    pull_buffer: Vec<[bool; Self::NEIGHBOR_COUNT]>,
    world_size: IVec2,
}

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    cells: Vec<u8>,
    world_size_x: i32,
    world_size_y: i32,
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

    pub fn new(world_size: IVec2, generator : impl world_gen::WorldGenerator) -> Result<Simulation, Error> {
        if (world_size.x as usize % Self::CHUNK_SIZE) != 0 {
            return Err(Error::InvalidWorldSize);
        }
        if (world_size.y as usize % Self::CHUNK_SIZE) != 0 {
            return Err(Error::InvalidWorldSize);
        }
        let num_of_cells: usize = (world_size.x * world_size.y) as usize;
        let cells = Vec::from_iter(std::iter::repeat_with(|| Cell::Air).take(num_of_cells));
        let transmutation_buffer = cells.clone();

        let push_buffer = Vec::from_iter(std::iter::repeat_with(|| IVec2::ZERO).take(num_of_cells));
        let pull_buffer = Vec::from_iter(
            std::iter::repeat_with(|| [false; Self::NEIGHBOR_COUNT]).take(num_of_cells),
        );

        let mut simulation = Ok(Simulation {
            cells: DoubleBuffer::new(cells),
            transmutation_buffer,
            world_size,
            push_buffer,
            pull_buffer,
        })?;

        for x in 0..world_size.x {
            for y in 0..world_size.y {
                let pos = ivec2(x, y);
                simulation.write_cell(pos, generator.gen_cell(pos, world_size));
            }
        }
        simulation.swap_buffers();

        return Ok(simulation);
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

        let viscosity_slow_down = ::rand::random_bool(1.0 - cell_center.viscosity());

        let is_falling = cell_center.is_falling()
            && cell_below.is_falling()
            && cell_center.density() > cell_below.density();
        let is_piling_a = cell_center.is_piling()
            && cell_below_a.is_piling()
            && cell_center.density() > cell_below_a.density()
            && viscosity_slow_down;
        let is_piling_b = cell_center.is_piling()
            && cell_below_b.is_piling()
            && cell_center.density() > cell_below_b.density()
            && viscosity_slow_down;
        let is_spreading_a = cell_center.is_spreading()
            && cell_side_a.is_spreading()
            && cell_center.density() > cell_side_a.density()
            && viscosity_slow_down;
        let is_spreading_b = cell_center.is_spreading()
            && cell_side_b.is_spreading()
            && cell_center.density() > cell_side_b.density()
            && viscosity_slow_down;
        let is_rising = cell_center.is_falling()
            && cell_above.is_falling()
            && cell_center.density() < cell_above.density()
            && viscosity_slow_down;
        let is_rising_a = cell_center.is_piling()
            && cell_above_a.is_piling()
            && cell_center.density() < cell_above_a.density()
            && viscosity_slow_down;
        let is_rising_b = cell_center.is_piling()
            && cell_above_b.is_piling()
            && cell_center.density() < cell_above_b.density();

        return None
            .or_else(|| is_falling.then_some(ivec2(0, 1)))
            .or_else(|| is_piling_a.then_some(ivec2(0, 1) + offset_a))
            .or_else(|| is_piling_b.then_some(ivec2(0, 1) + offset_b))
            .or_else(|| is_spreading_a.then_some(offset_a))
            .or_else(|| is_spreading_b.then_some(offset_b))
            .or_else(|| is_rising.then_some(ivec2(0, -1)))
            .or_else(|| is_rising_a.then_some(ivec2(0, -1) + offset_a))
            .or_else(|| is_rising_b.then_some(ivec2(0, -1) + offset_b))
            .unwrap_or(IVec2::ZERO);
    }

    fn update_push_vectors(
        chunk_index: usize,
        write_chunk: &mut [IVec2],
        read_world: &WorldView<Cell>,
    ) {
        let chunk_coord =
            grid::map_1d_to_2d(chunk_index, read_world.size() / (Self::CHUNK_SIZE as i32));
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord =
                grid::map_1d_to_2d(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let global_coord = chunk_coord * (Self::CHUNK_SIZE as i32) + local_coord;
            write_chunk[local_index] = Self::calc_push_vector(read_world, global_coord);
        }
    }

    fn pick_pulls(
        read_world_push: &WorldView<IVec2>,
        write_chunk_pull: &mut ChunkViewMut<[bool; Self::NEIGHBOR_COUNT]>,
    ) {
        let chunk_coord = grid::map_1d_to_2d(
            write_chunk_pull.get_chunk_index(),
            read_world_push.size() / (Self::CHUNK_SIZE as i32),
        );
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord =
                grid::map_1d_to_2d(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
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
            let mut new_pulls = [false; Self::NEIGHBOR_COUNT];
            if let Some(picked_pull) = pulls[0..size].choose() {
                new_pulls[*picked_pull] = true;
            }
            write_chunk_pull.set_cell(new_pulls, local_coord);
        }
    }

    fn resolve_movement(
        write_chunk: &mut ChunkViewMut<Cell>,
        read_world: &WorldView<Cell>,
        read_world_push: &WorldView<IVec2>,
        read_world_pull: &WorldView<[bool; Self::NEIGHBOR_COUNT]>,
    ) {
        let chunk_coord = grid::map_1d_to_2d(
            write_chunk.get_chunk_index(),
            read_world.size() / (Self::CHUNK_SIZE as i32),
        );
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord =
                grid::map_1d_to_2d(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
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
                let neighbor_push = read_world_push.get_cell(global_coord + offset);
                if pull_field[n] && neighbor_push == -offset {
                    cell = read_world.get_cell(global_coord + offset);
                }
            }

            write_chunk.set_cell(cell, local_coord);
        }
    }

    fn resolve_transmutation(write_chunk: &mut ChunkViewMut<Cell>, read_world: &WorldView<Cell>) {
        let chunk_coord = grid::map_1d_to_2d(
            write_chunk.get_chunk_index(),
            read_world.size() / (Self::CHUNK_SIZE as i32),
        );
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord =
                grid::map_1d_to_2d(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let global_coord = chunk_coord * (Self::CHUNK_SIZE as i32) + local_coord;
            let mut cell = read_world.get_cell(global_coord);

            if cell == Cell::Water {
                for offset in Self::NEIGHBOR_IDX2OFFSET {
                    if read_world.get_cell(global_coord + offset).is_hot() {
                        cell = Cell::Steam;
                    }
                }
            }

            if cell == Cell::Lava {
                for offset in Self::NEIGHBOR_IDX2OFFSET {
                    if read_world.get_cell(global_coord + offset) == Cell::Water {
                        cell = Cell::Stone;
                    }
                }
            }

            if cell == Cell::Steam {
                const CONDENSATION_RATE: f64 = 0.0001;
                if ::rand::random_bool(CONDENSATION_RATE) {
                    cell = Cell::Water;
                    for offset in Self::NEIGHBOR_IDX2OFFSET {
                        if read_world.get_cell(global_coord + offset) == Cell::Air {}
                    }
                }
            }
            write_chunk.set_cell(cell, local_coord);
        }
    }

    pub fn tick(&mut self) {
        let world_size = self.size();
        let (read_buffer, write_buffer) = self.cells.pick_read_and_write_buffer();
        // println!("Water: {}", read_buffer.iter().filter(|c| **c==Cell::Water).count());
        // println!("Sand:  {}", read_buffer.iter().filter(|c| **c==Cell::Sand).count());
        let read_world = WorldView::new(read_buffer, world_size, Cell::Barrier);
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

        self.transmutation_buffer
            .par_chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .map(|(chunk_index, chunk)| ChunkViewMut::new(chunk_index, chunk, world_size))
            .for_each(|mut write_chunk| {
                Self::resolve_movement(
                    &mut write_chunk,
                    &read_world,
                    &read_world_push,
                    &read_world_pull,
                )
            });

        let read_world_transformation =
            &WorldView::new(&self.transmutation_buffer, world_size, Cell::Barrier);

        write_buffer
            .par_chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .map(|(chunk_index, chunk)| ChunkViewMut::new(chunk_index, chunk, world_size))
            .for_each(|mut write_chunk| {
                Self::resolve_transmutation(&mut write_chunk, &read_world_transformation)
            });
    }

    // Just copies the read buffer to write buffer. useful for skipping simulation ticks but still use write_cell.
    pub fn pass(&mut self) {
        let (read_buffer, write_buffer) = self.cells.pick_read_and_write_buffer();
        write_buffer.copy_from_slice(&read_buffer);
    }

    pub fn write_cell(&mut self, global_coord: IVec2, cell: Cell) {
        let (_read_buffer, write_buffer) = self.cells.pick_read_and_write_buffer();
        let mut world_mut = WorldViewMut::new(write_buffer, self.world_size, Cell::Barrier);
        world_mut.set_cell(global_coord, cell);
    }

    pub fn get_chunk_by_coord<'a>(&'a self, chunk_coord: IVec2) -> ChunkView<'a, Cell> {
        let chunk_index = grid::map_2d_to_1d(chunk_coord, self.num_of_chunks_xy());
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

    pub fn from_save_data(save_data: SaveData) -> Result<Simulation, Error> {
        let world_size = ivec2(save_data.world_size_x, save_data.world_size_y);
        let mut simulation = Self::new(world_size, world_gen::EmptyWorldGenerator::new())?;
        simulation.cells = DoubleBuffer::new(
            save_data
                .cells
                .iter()
                .map(|x| Cell::try_from(*x).unwrap())
                .collect(),
        );
        return Ok(simulation);
    }

    pub fn to_save_data(&self) -> SaveData {
        return SaveData {
            cells: self
                .cells
                .get_read_buffer()
                .iter()
                .map(|x| *x as u8)
                .collect(),
            world_size_x: self.world_size.x,
            world_size_y: self.world_size.y,
        };
    }
}
