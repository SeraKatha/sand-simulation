use macroquad::math::{IVec2, UVec2};
use macroquad::prelude::*;

mod double_buffer;
use double_buffer::DoubleBuffer;

mod world_view;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::{ParallelSlice, ParallelSliceMut};
use world_view::WorldView;

mod chunk_view;
use chunk_view::ChunkViewMut;


pub enum Error {
    InvalidWorldSize,
    CellOutOfBounds,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    AIR, SAND, STONE,
}


pub struct Grid {}

impl Grid {
    pub fn map2Dto1D(coord : IVec2, grid_size : IVec2) -> usize {
        return (coord.y * grid_size.x + coord.x) as usize;
    }

    pub fn map1Dto2D(index : usize, grid_size : IVec2) -> IVec2 {
        let x = (index as i32) % grid_size.x;
        let y = (index as i32) / grid_size.x;
        return ivec2(x, y)
    }
}




pub struct Simulation {
    cells : DoubleBuffer<Vec<Cell>>,
    push_buffer : Vec<IVec2>,
    pull_buffer : Vec<[bool; Self::NEIGHBOR_COUNT]>,
    world_size : IVec2,
}

impl Simulation {
    pub const CHUNK_SIZE : usize = 32;
    pub const CELLS_PER_CHUNK : usize = Self::CHUNK_SIZE * Self::CHUNK_SIZE;
    const NEIGHBOR_COUNT : usize = 8;
    const NEIGHBOR_IDX2OFFSET : [IVec2; Self::NEIGHBOR_COUNT] = [
        ivec2(-1, -1), ivec2( 0, -1), ivec2( 1, -1),
        ivec2(-1,  0),                ivec2( 1,  0),
        ivec2(-1, -1), ivec2( 0, -1), ivec2( 1, -1),
    ];

    pub fn new(world_size : IVec2) -> Result<Simulation, Error> {
        if (world_size.x as usize % Self::CHUNK_SIZE) != 0 {
            return Err(Error::InvalidWorldSize);
        } 
        if (world_size.y as usize % Self::CHUNK_SIZE) != 0 {
            return Err(Error::InvalidWorldSize);
        }
        let num_of_cells : usize = (world_size.x * world_size.y) as usize;
        let cells = Vec::from_iter(
            std::iter::repeat_with(|| match ::rand::random::<i32>() % 3 {
                0 => Cell::AIR,
                1 => Cell::SAND,
                2 => Cell::STONE,
                _ => Cell::AIR,
            })
            .take(num_of_cells));

        let push_buffer = Vec::from_iter(std::iter::repeat_with(||IVec2::ZERO).take(num_of_cells));
        let pull_buffer = Vec::from_iter(std::iter::repeat_with(||[false; 8]).take(num_of_cells));

        return Ok(Simulation {
            cells : DoubleBuffer::new(cells),
            world_size,
            push_buffer,
            pull_buffer,
        });
    }


    pub fn num_of_chunks_xy(&self) -> IVec2 {
        return self.world_size / (Self::CHUNK_SIZE as i32);
    }


    pub fn num_of_chunks(&self) -> usize {
        return self.num_of_chunks_xy().dot(IVec2::new(1, 1)) as usize;
    }


    fn read_cell(read_world : &[Cell], world_size : IVec2, global_coord : IVec2, offset : IVec2) -> Cell {
        let global_target_coord = global_coord + offset;
        if global_target_coord.x < 0 {
            return Cell::STONE;
        };
        if global_target_coord.y < 0 {
            return Cell::STONE;
        };
        if global_target_coord.x >= world_size.x as i32 {
            return Cell::STONE;
        }
        if global_target_coord.y >= world_size.y as i32 {
            return Cell::STONE;
        }
        let local_coord = global_target_coord % (Self::CHUNK_SIZE as i32); 
        let local_index = Grid::map2Dto1D(local_coord, IVec2::ONE * (Self::CHUNK_SIZE as i32));

        let chunk_coord = global_target_coord / (Self::CHUNK_SIZE as i32); 
        let chunk_index = Grid::map2Dto1D(chunk_coord, world_size / (Self::CHUNK_SIZE as i32));
        let global_index = chunk_index * Self::CELLS_PER_CHUNK + local_index;
        // println!("A: {global_index}");
        // println!("{chunk_index}");
        return read_world[global_index];
    }


    fn tick_chunk(write_chunk : &mut ChunkViewMut, read_world : &WorldView<Cell>) {
        for local_index in 0..Self::CELLS_PER_CHUNK {
            // let (local_x, local_y) = Grid::index_to_coord(local_index, (Self::CHUNK_SIZE, Self::CHUNK_SIZE));
            let local_coord = Grid::map1Dto2D(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let chunk_coord = Grid::map1Dto2D(write_chunk.index(), read_world.size() / (Self::CHUNK_SIZE as i32));
            let global_coord = local_coord + chunk_coord * (Self::CHUNK_SIZE as i32);
            // println!("{global_coord:?}");

            let read_cell       = read_world.get_cell(global_coord);
            let read_cell_above = read_world.get_cell(global_coord + ivec2(0, -1));
            let read_cell_below = read_world.get_cell(global_coord + ivec2(0, 1));
            
            let new_cell = match read_cell {
                Cell::AIR => {
                    match read_cell_above {
                        Cell::AIR => Cell::AIR,
                        Cell::SAND => Cell::SAND,
                        Cell::STONE => Cell::AIR,
                    }
                }
                Cell::SAND  => {
                    match read_cell_below {
                        Cell::AIR => Cell::AIR,
                        Cell::SAND => Cell::SAND,
                        Cell::STONE => Cell::SAND,
                    }
                }
                Cell::STONE => Cell::STONE,
            };
            write_chunk.write_cell(new_cell, local_coord);
        }
        // println!("{chunk_index:?} {:?}", write_chunk[0])
    }


    fn calc_push_vector(read_world : &WorldView<Cell>, global_coord : IVec2) -> IVec2 {
        let cell_center = read_world.get_cell(global_coord);
        let cell_below = read_world.get_cell(global_coord + ivec2(0, 1));
        if cell_below == Cell::AIR && cell_center == Cell::SAND {
            return ivec2(0, 1);
        }
        else {
            return IVec2::ZERO;
        }
    }


    fn update_push_vectors(chunk_index : usize, write_chunk : &mut [IVec2], read_world : &WorldView<Cell>) {
        let chunk_coord = Grid::map1Dto2D(chunk_index, read_world.size() / (Self::CHUNK_SIZE as i32));
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord = Grid::map1Dto2D(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let global_coord = chunk_coord * (Self::CHUNK_SIZE as i32) + local_coord;
            write_chunk[local_index] = Self::calc_push_vector(read_world, global_coord);
        }
    }


    fn calc_pull_field(read_world : &WorldView<Cell>, global_coord : IVec2) -> [bool; 8] {
        let mut pull_vectors = [false; Self::NEIGHBOR_COUNT];
        let cell_center = read_world.get_cell(global_coord);
        let cell_above  = read_world.get_cell(global_coord + ivec2(0, -1));

        pull_vectors[1] = match cell_center {
            Cell::AIR => {
                if cell_above == Cell::SAND {
                    true
                }
                else {
                    false
                }
            }
            Cell::SAND => false,
            Cell::STONE => false,
        };

        return pull_vectors;
    }


    fn update_pull_fields(chunk_index : usize, write_chunk : &mut [[bool; 8]], read_world : &WorldView<Cell>) {
        let chunk_coord = Grid::map1Dto2D(chunk_index, read_world.size() / (Self::CHUNK_SIZE as i32));
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord = Grid::map1Dto2D(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let global_coord = chunk_coord * (Self::CHUNK_SIZE as i32) + local_coord;
            write_chunk[local_index] = Self::calc_pull_field(read_world, global_coord);
        }
    }


    fn resolve_movements(
        write_chunk : &mut ChunkViewMut,
        read_world : &WorldView<Cell>,
        read_world_push : &WorldView<IVec2>,
        read_world_pull : &WorldView<[bool; 8]>) {
        
        let chunk_coord = Grid::map1Dto2D(write_chunk.index(), read_world.size() / (Self::CHUNK_SIZE as i32));
        for local_index in 0..Self::CELLS_PER_CHUNK {
            let local_coord = Grid::map1Dto2D(local_index, IVec2::ONE * (Self::CHUNK_SIZE as i32));
            let global_coord = chunk_coord * (Self::CHUNK_SIZE as i32) + local_coord;
            let pull_field = read_world_pull.get_cell(global_coord);
            let mut cell = read_world.get_cell(global_coord);
            let center_push = read_world_push.get_cell(global_coord);
            let target_pull = read_world_pull.get_cell(global_coord + center_push);
            for n in 0..Self::NEIGHBOR_COUNT {
                let offset = Self::NEIGHBOR_IDX2OFFSET[n];
                let neighbor_push = read_world_push.get_cell(global_coord + offset);
                if target_pull[n] && center_push == -offset {
                    cell = Cell::AIR;
                }
            }
            for n in 0..Self::NEIGHBOR_COUNT {
                let offset = Self::NEIGHBOR_IDX2OFFSET[n];
                let neighbor_push = read_world_push.get_cell(global_coord + offset);
                if pull_field[n] && neighbor_push == -offset {
                    cell = read_world.get_cell(global_coord + offset);
                }
            }
   
            write_chunk.write_cell(cell, local_coord);
        }
    }


    pub fn tick(&mut self) {
        let (read_buffer, write_buffer) = self.cells.pick_read_and_write_buffer();
        let read_world = WorldView::new(read_buffer, self.world_size, Cell::STONE);

        self.push_buffer
            .chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .for_each(|(chunk_index, chunk)| Self::update_push_vectors(chunk_index, chunk, &read_world));
        
        self.pull_buffer
            .chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .for_each(|(chunk_index, chunk)| Self::update_pull_fields(chunk_index, chunk, &read_world));
        
        let read_world_push = WorldView::new(&self.push_buffer, self.world_size, IVec2::ZERO);
        let read_world_pull = WorldView::new(&self.pull_buffer, self.world_size, [false; Self::NEIGHBOR_COUNT]);

        write_buffer
            .chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .map(|(chunk_index, chunk)| ChunkViewMut::new(chunk_index, chunk))
            .for_each(|mut write_chunk| Self::resolve_movements(&mut write_chunk, &read_world, &read_world_push, &read_world_pull));
        
        self.swap_buffers()
    }


    pub fn get_chunk(&self, coord : IVec2) -> &[Cell] {
        let num_chunks_xy = self.num_of_chunks_xy();
        let chunk_index = Grid::map2Dto1D(coord, num_chunks_xy);
        let global_index = Self::CELLS_PER_CHUNK * chunk_index;
        let read_buffer = self.cells.get_read_buffer();
        let begin = global_index;
        let end = global_index + Self::CELLS_PER_CHUNK;
        return &read_buffer[begin..end];
    }


    fn swap_buffers(&mut self) {
        self.cells.swap();
    } 
}