use macroquad::math::UVec2;
use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::ParallelSliceMut};

mod double_buffer;
use double_buffer::DoubleBuffer;

pub enum Error {
    InvalidWorldSize,
    CellOutOfBounds,
}

#[derive(Clone, Copy, Debug)]
pub enum Cell {
    AIR, SAND, STONE,
}


pub struct Grid {}

impl Grid {
    pub fn coord_to_index(coord : (usize, usize), grid_size : (usize, usize)) -> usize {
        return (coord.1 * grid_size.0 + coord.0) as usize;
    }

    pub fn index_to_coord(index : usize, grid_size : (usize, usize)) -> (usize, usize) {
        let (grid_size_x, _grid_size_y) = grid_size;
        let x = index % grid_size_x;
        let y = index / grid_size_x;
        return (x, y)
    }   
}



pub struct World {
    cells : DoubleBuffer<Vec<Cell>>,
    world_size : (usize, usize),
}

impl World {
    pub const CHUNK_SIZE : usize = 256;
    pub const CELLS_PER_CHUNK : usize = Self::CHUNK_SIZE * Self::CHUNK_SIZE;


    pub fn new(world_size : (usize, usize)) -> Result<World, Error> {
        if (world_size.0 as usize % Self::CHUNK_SIZE) != 0 {
            return Err(Error::InvalidWorldSize);
        } 
        if (world_size.1 as usize % Self::CHUNK_SIZE) != 0 {
            return Err(Error::InvalidWorldSize);
        }
        let num_of_cells : usize = (world_size.0 * world_size.1) as usize;
        let cells = Vec::from_iter(
            std::iter::repeat_with(|| match rand::random::<i32>() % 3 {
                0 => Cell::AIR,
                1 => Cell::SAND,
                2 => Cell::AIR,
                _ => Cell::AIR, 
            })
            .take(num_of_cells));
        return Ok(World {
            cells : DoubleBuffer::new(cells),
            world_size,
        });
    }


    pub fn num_of_chunks_xy(&self) -> (usize, usize) {
        let num_chunks_x = self.world_size.0 / Self::CHUNK_SIZE;
        let num_chunks_y = self.world_size.1 / Self::CHUNK_SIZE;
        return (num_chunks_x, num_chunks_y);
    }


    pub fn num_of_chunks(&self) -> usize {
        let (num_chunks_x, num_chunks_y) = self.num_of_chunks_xy();
        return num_chunks_x * num_chunks_y;
    }


    fn read_cell(read_world : &[Cell], world_size : (usize, usize), global_coord : (usize, usize), offset : (i32, i32)) -> Cell {
        let (global_coord_x, global_coord_y) = global_coord;
        let (offset_x, offset_y) = offset;
        let global_target_coord = (
            (global_coord_x as i32) + offset_x,
            (global_coord_y as i32) + offset_y,
        );
        let (global_target_coord_x, global_target_coord_y) = global_target_coord;
        let (world_size_x, world_size_y) = world_size;
        if global_target_coord_x < 0 {
            return Cell::STONE;
        };
        if global_target_coord_y < 0 {
            return Cell::STONE;
        };
        if global_target_coord_x >= world_size_x as i32 {
            return Cell::STONE;
        }
        if global_target_coord_y >= world_size_y as i32 {
            return Cell::STONE;
        }
        let local_coord_x = (global_target_coord_x as usize) % Self::CHUNK_SIZE;
        let local_coord_y = (global_target_coord_y as usize) % Self::CHUNK_SIZE;
        let local_index   = Grid::coord_to_index((local_coord_x, local_coord_y), (Self::CHUNK_SIZE, Self::CHUNK_SIZE));

        let chunk_coord_x = (global_target_coord_x as usize) / Self::CHUNK_SIZE;
        let chunk_coord_y = (global_target_coord_y as usize) / Self::CHUNK_SIZE;
        let chunk_index   = Grid::coord_to_index((chunk_coord_x, chunk_coord_y), (world_size_x / Self::CHUNK_SIZE, world_size_y / Self::CHUNK_SIZE));
        let global_index = chunk_index * Self::CELLS_PER_CHUNK + local_index;
        // println!("A: {global_index}");
        // println!("{chunk_index}");
        return read_world[global_index];
    }


    fn tick_chunk(chunk_index : usize, write_chunk : &mut [Cell], read_world : &[Cell], world_size : (usize, usize)) {
        for local_index in 0..Self::CELLS_PER_CHUNK {
            // let (local_x, local_y) = Grid::index_to_coord(local_index, (Self::CHUNK_SIZE, Self::CHUNK_SIZE));
            let local_coord = Grid::index_to_coord(local_index, (Self::CHUNK_SIZE, Self::CHUNK_SIZE));
            let chunk_coord = Grid::index_to_coord(chunk_index, (world_size.0 / Self::CHUNK_SIZE, world_size.1 / Self::CHUNK_SIZE));
            let global_coord = (
                local_coord.0 + chunk_coord.0 * Self::CHUNK_SIZE,
                local_coord.1 + chunk_coord.1 * Self::CHUNK_SIZE,
            );
            // println!("{global_coord:?}");

            let read_cell       = Self::read_cell(read_world, world_size, global_coord, (0,  0));
            let read_cell_above = Self::read_cell(read_world, world_size, global_coord, (0, -1));
            let read_cell_below = Self::read_cell(read_world, world_size, global_coord, (0,  1));
            
            // write_chunk[local_index] = read_cell;

            write_chunk[local_index] = match read_cell {
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
        }
        // println!("{chunk_index:?} {:?}", write_chunk[0])
    }


    pub fn tick(&mut self) {
        let (read_buffer, write_buffer) = self.cells.pick_read_and_write_buffer();

        write_buffer
            .chunks_mut(Self::CELLS_PER_CHUNK)
            .enumerate()
            .for_each(|(chunk_index, chunk)| Self::tick_chunk(chunk_index, chunk, read_buffer, self.world_size));
        
        self.swap_buffers()
    }


    pub fn get_chunk(&self, coord : (usize, usize)) -> &[Cell] {
        let num_chunks_xy = self.num_of_chunks_xy();
        let chunk_index = Grid::coord_to_index(coord, num_chunks_xy);
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