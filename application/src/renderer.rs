use macroquad::prelude::*;
use simulation::{Simulation, Cell, Grid, ChunkView};

fn init_chunk_texture() -> Texture2D {
    let bytes = [128u8; Simulation::CELLS_PER_CHUNK * 4];
    Texture2D::from_rgba8(
        Simulation::CHUNK_SIZE as u16,
        Simulation::CHUNK_SIZE as u16,
        &bytes,
    )
}

fn cell_to_color(cell: Cell) -> [u8; 4] {
    match cell {
        Cell::AIR => [0, 0, 0, 255],
        Cell::SAND => [242, 203, 151, 255],
        Cell::STONE => [93, 93, 93, 255],
        Cell::WATER => [0, 96, 255, 255],
    }
}

pub struct Renderer {
    textures: Vec<Texture2D>,
}

impl Renderer {
    pub fn new() -> Self {
        return Self {
            textures : Vec::new()
        }
    }

    pub fn resize(&mut self, simulation : &Simulation) {
        let num_of_chunks_total = simulation.num_of_chunks();
        self.textures = std::iter::repeat_with(init_chunk_texture)
            .take(num_of_chunks_total)
            .collect();
    }


    fn render_chunk<'a>(&mut self, chunk : ChunkView<'a, Cell>) {
        let chunk_index = chunk.get_chunk_index();
        let chunk_coord = chunk.get_chunk_coord();
        let cells = chunk.get_cells();
        let texture = &mut self.textures[chunk_index];

        let mut bytes = [0u8; 4 * Simulation::CELLS_PER_CHUNK];
        for local_index in 0..Simulation::CELLS_PER_CHUNK {
            let [r, g, b, a] = cell_to_color(cells[local_index]);
            bytes[4 * local_index + 0] = r;
            bytes[4 * local_index + 1] = g;
            bytes[4 * local_index + 2] = b;
            bytes[4 * local_index + 3] = a;
        }

        texture.update_from_bytes(
            Simulation::CHUNK_SIZE as u32,
            Simulation::CHUNK_SIZE as u32,
            &bytes,
        );

        draw_texture(
            texture,
            (chunk_coord.x as f32) * (Simulation::CHUNK_SIZE as f32),
            (chunk_coord.y as f32) * (Simulation::CHUNK_SIZE as f32),
            WHITE,
        );
    }


    pub fn render(&mut self, simulation : &Simulation) {
        let num_of_chunks_total = simulation.num_of_chunks();
        for chunk_index in 0..num_of_chunks_total {
            let chunk = simulation.get_chunk_by_index(chunk_index);
            self.render_chunk(chunk);
        } 
    }
}