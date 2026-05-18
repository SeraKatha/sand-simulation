use simulation::{Cell, Grid, Simulation};
use macroquad::prelude::*;


fn init_chunk_texture() -> Texture2D {
    let bytes =  [128u8; Simulation::CELLS_PER_CHUNK * 4];
    Texture2D::from_rgba8(
        Simulation::CHUNK_SIZE as u16,
        Simulation::CHUNK_SIZE as u16,
        &bytes,
    )
}

fn cell_to_color(cell : Cell) -> [u8; 4] {
    match cell {
        Cell::AIR   => [  0,   0,   0, 255],
        Cell::SAND  => [242, 203, 151, 255],
        Cell::STONE => [ 93,  93,  93, 255],
    }
}

#[macroquad::main("SandFalls")]
async fn main() {
    let simulation = Simulation::new(ivec2(512, 512));
    
    if let Ok(mut simulation) = simulation {
        let num_of_chunks_xy = simulation.num_of_chunks_xy();
        let num_of_chunks_total = simulation.num_of_chunks();
        let mut textures : Vec<Texture2D> = std::iter::repeat_with(init_chunk_texture).take(num_of_chunks_total).collect();
        loop {
            simulation.tick();
            println!("Tick");
            clear_background(BLACK);
            for chunk_index in 0..num_of_chunks_total {
                let chunk_coord = Grid::map1Dto2D(chunk_index, num_of_chunks_xy);
                let chunk = simulation.get_chunk(chunk_coord);
                let texture = &mut textures[chunk_index]; 
                
                let mut bytes = [0u8; 4 * Simulation::CELLS_PER_CHUNK];
                for local_index in 0..Simulation::CELLS_PER_CHUNK {
                    let [r, g, b, a] = cell_to_color(chunk[local_index]);
                    bytes[4 * local_index + 0] = r;
                    bytes[4 * local_index + 1] = g;
                    bytes[4 * local_index + 2] = b;
                    bytes[4 * local_index + 3] = a;
                }


                texture.update_from_bytes(
                    Simulation::CHUNK_SIZE as u32,
                    Simulation::CHUNK_SIZE as u32,
                    &bytes
                );

                draw_texture(
                    texture,
                    (chunk_coord.x as f32) * (Simulation::CHUNK_SIZE as f32),
                    (chunk_coord.y as f32) * (Simulation::CHUNK_SIZE as f32),
                    WHITE
                );

            }
            // sleep(Duration::from_secs(1));
            
            next_frame().await
        }
    }
}