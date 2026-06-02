use macroquad::{prelude::*};
use simulation::{Cell, ChunkView, Simulation};
use super::Canvas;

fn rgba_f32_to_u8(color : Color) -> [u8; 4] {
    [
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8,
        (color.a * 255.0) as u8,
    ]
}



pub struct TexturedRenderer {
    canvas: Canvas,
    cell_textures : [Image; Cell::NUM_OF_TYPES],
    frame_number : usize,
}

impl super::Renderer for TexturedRenderer {
    fn fit_simulation(&mut self, simulation: &Simulation) {
        self.canvas.fit_simulation(simulation);
    }

    fn render(&mut self, simulation: &Simulation) {
        self.frame_number += 1;
        let num_of_chunks_total = simulation.num_of_chunks();
        for chunk_index in 0..num_of_chunks_total {
            let chunk = simulation.get_chunk_by_index(chunk_index);
            self.render_chunk(chunk);
        }
    }
}

impl TexturedRenderer {
    pub fn new() -> Self {
        let cell_textures : [Image; Cell::NUM_OF_TYPES] = [
            Image::from_file_with_format(include_bytes!("../../../assets/textures/air.png"), Some(ImageFormat::Png)).unwrap(),
            Image::from_file_with_format(include_bytes!("../../../assets/textures/sand.png"), Some(ImageFormat::Png)).unwrap(),
            Image::from_file_with_format(include_bytes!("../../../assets/textures/stone.png"), Some(ImageFormat::Png)).unwrap(),
            Image::from_file_with_format(include_bytes!("../../../assets/textures/water.png"), Some(ImageFormat::Png)).unwrap(),
        ];
        return Self {
            canvas: Canvas::new(),
            cell_textures,
            frame_number : 0,
        };
    }

    fn render_chunk<'a>(&mut self, chunk: ChunkView<'a, Cell>) {
        let chunk_index = chunk.get_chunk_index();
        let chunk_coord = chunk.get_chunk_coord();
        let cells = chunk.get_cells();
        let texture = &mut self.canvas.get_textures()[chunk_index];

        // Write color channels into a byte buffer.
        let mut bytes = [0u8; 4 * Simulation::CELLS_PER_CHUNK];
        for local_index in 0..Simulation::CELLS_PER_CHUNK {
            let local_coord = simulation::grid::map_1d_to_2d(local_index, Simulation::CHUNK_SIZE_XY);
            let mut texture_coord = uvec2(local_coord.x as u32  % 16, local_coord.y as u32 % 16);
            let cell = cells[local_index];
            let cell_type_index = cell as usize;
            if cell.is_liquid() {
                let scroll_offset_x = (self.frame_number as u32 / 4) % 16;
                if texture_coord.y % 2 == 0{
                    texture_coord.x = (texture_coord.x + scroll_offset_x) % 16;
                }
                else {
                    texture_coord.x = (texture_coord.x - scroll_offset_x) % 16;
                }
            }
            let color = self.cell_textures[cell_type_index].get_pixel(texture_coord.x, texture_coord.y);
            let [r, g, b, a] = rgba_f32_to_u8(color);
            bytes[4 * local_index + 0] = r;
            bytes[4 * local_index + 1] = g;
            bytes[4 * local_index + 2] = b;
            bytes[4 * local_index + 3] = a;
        }

        // Sends color data to the GPU
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
}
