use simulation::{Cell, Grid, Simulation};
use macroquad::prelude::*;
mod view;
use view::View;
mod tool;
use tool::{Tool, Dropper};

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
        Cell::WATER => [  0,  96, 255, 255],
    }
}


pub struct Application {
    simulation : Simulation,
    view : View,
    textures : Vec<Texture2D>,
    current_tool : tool::Dropper,
    eraser : tool::Dropper,
}


impl Application {
    const DEFAULT_WORLD_SIZE : IVec2 = ivec2(128, 128);
    pub fn new() -> Application {
        if let Ok(mut simulation) = Simulation::new(ivec2(0, 0)) {
            let view = View::new(vec2(0.0, 0.0));
            let textures : Vec<Texture2D> = Vec::new();
            let current_tool = tool::Dropper::new(Cell::SAND, 3);
            let eraser = tool::Dropper::new(Cell::AIR, 3);
            let mut application = Application {
                simulation, view, textures, current_tool, eraser,
            };
            application.generate_simulation(Self::DEFAULT_WORLD_SIZE);
            return application;
        }
        else {
            panic!("AAAHHH!!!")
        }
    }
    
    pub fn generate_simulation(&mut self, world_size : IVec2) {
        let simulation = Simulation::new(world_size);
        if let Ok(mut simulation) = simulation {
            self.simulation = simulation;
            let num_of_chunks_xy = self.simulation.num_of_chunks_xy();
            let num_of_chunks_total = self.simulation.num_of_chunks();
            self.textures = std::iter::repeat_with(init_chunk_texture).take(num_of_chunks_total).collect();
            self.view = View::new(vec2(world_size.x as f32, world_size.y as f32));
        }
        else {
            panic!("AAAHHH!!!")
        }
    }


    pub fn update(&mut self) {
        let camera = self.view.into_camera_2d(); 
        set_camera(&camera);
        self.simulation.tick();
        let mouse_position = macroquad::input::mouse_position(); 
        let global_coord = camera.screen_to_world(vec2(mouse_position.0, mouse_position.1));
        if macroquad::input::is_mouse_button_down(MouseButton::Left) {
            self.current_tool.apply(&mut self.simulation, ivec2(global_coord.x as i32, global_coord.y as i32));
        }
        if macroquad::input::is_mouse_button_down(MouseButton::Right) {
            self.eraser.apply(&mut self.simulation, ivec2(global_coord.x as i32, global_coord.y as i32));
        }
        self.simulation.swap_buffers();
        self.view.update();
        clear_background(BLACK);
        set_camera(&camera);
    }


    pub fn render(&mut self) {
        let num_of_chunks_xy = self.simulation.num_of_chunks_xy();
        let num_of_chunks_total = self.simulation.num_of_chunks();
        for chunk_index in 0..num_of_chunks_total {
            let chunk_coord = Grid::map1Dto2D(chunk_index, num_of_chunks_xy);
            let chunk = self.simulation.get_chunk(chunk_coord);
            let texture = &mut self.textures[chunk_index]; 
            
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
    }
}