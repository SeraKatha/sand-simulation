use macroquad::prelude::*;
use simulation::Simulation;

fn init_chunk_texture() -> Texture2D {
    let bytes = [128u8; Simulation::CELLS_PER_CHUNK * 4];
    Texture2D::from_rgba8(
        Simulation::CHUNK_SIZE as u16,
        Simulation::CHUNK_SIZE as u16,
        &bytes,
    )
}

pub struct Canvas {
    textures: Vec<Texture2D>,
}

impl Canvas {
    pub fn new() -> Self {
        return Self {
            textures: Vec::new(),
        };
    }

    pub fn fit_simulation(&mut self, simulation: &Simulation) {
        let num_of_chunks_total = simulation.num_of_chunks();
        self.textures = std::iter::repeat_with(init_chunk_texture)
            .take(num_of_chunks_total)
            .collect();
    }

    pub fn get_textures(&mut self) -> &mut [Texture2D] {
        return &mut self.textures;
    }
}
