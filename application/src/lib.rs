use std::time::{Duration, Instant};

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use simulation::{Cell, Grid, Simulation};

mod view;
use view::View;
mod pulse;
mod tool;
use pulse::Pulse;

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

pub struct Application {
    simulation: Simulation,
    view: View,
    textures: Vec<Texture2D>,
    dropper: tool::Dropper,
    eraser: tool::Dropper,
    new_world_size: IVec2,
    last_tick_duration: Duration,
    last_render_duration: Duration,
    pulse: Pulse,
}

impl Application {
    const WORLD_SIZE_DEFAULT: IVec2 = ivec2(128, 128);
    pub fn new() -> Application {
        if let Ok(simulation) = Simulation::new(ivec2(0, 0)) {
            let view = View::new(vec2(0.0, 0.0));
            let textures: Vec<Texture2D> = Vec::new();
            let dropper = tool::Dropper::new(Cell::SAND, 3);
            let eraser = tool::Dropper::new(Cell::AIR, 3);
            let new_world_size = Self::WORLD_SIZE_DEFAULT;
            let mut application = Application {
                simulation,
                view,
                textures,
                dropper,
                eraser,
                new_world_size,
                last_tick_duration: Duration::ZERO,
                last_render_duration: Duration::ZERO,
                pulse: Pulse::new(60.0),
            };
            application.generate_simulation(Self::WORLD_SIZE_DEFAULT);
            return application;
        } else {
            panic!("AAAHHH!!!")
        }
    }

    pub fn generate_simulation(&mut self, world_size: IVec2) {
        let simulation = Simulation::new(world_size);
        if let Ok(simulation) = simulation {
            self.simulation = simulation;
            let num_of_chunks_total = self.simulation.num_of_chunks();
            self.textures = std::iter::repeat_with(init_chunk_texture)
                .take(num_of_chunks_total)
                .collect();
            self.view = View::new(vec2(world_size.x as f32, world_size.y as f32));
        } else {
            panic!("AAAHHH!!!")
        }
    }

    pub fn update(&mut self) {
        let camera = self.view.into_camera_2d();
        set_camera(&camera);
        if self.pulse.tick(get_frame_time()) {
            let time_tick_pre = Instant::now();
            self.simulation.tick();
            let time_tick_post = Instant::now();
            self.last_tick_duration = time_tick_post.duration_since(time_tick_pre);
        } else {
            self.simulation.pass();
        }
        let mouse_position = macroquad::input::mouse_position();
        let global_coord = camera.screen_to_world(vec2(mouse_position.0, mouse_position.1));
        if !root_ui().is_mouse_over(mouse_position.into()) {
            if macroquad::input::is_mouse_button_down(MouseButton::Left) {
                self.dropper.apply(
                    &mut self.simulation,
                    ivec2(global_coord.x as i32, global_coord.y as i32),
                );
            }
            if macroquad::input::is_mouse_button_down(MouseButton::Right) {
                self.eraser.apply(
                    &mut self.simulation,
                    ivec2(global_coord.x as i32, global_coord.y as i32),
                );
            }
        }
        self.simulation.swap_buffers();
        self.view.update();
        clear_background(DARKGRAY);
        set_camera(&camera);
    }

    pub fn render(&mut self) {
        let time_render_pre = Instant::now();
        let num_of_chunks_xy = self.simulation.num_of_chunks_xy();
        let num_of_chunks_total = self.simulation.num_of_chunks();
        for chunk_index in 0..num_of_chunks_total {
            let chunk_coord = Grid::map_1d_to_2d(chunk_index, num_of_chunks_xy);
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
                &bytes,
            );

            draw_texture(
                texture,
                (chunk_coord.x as f32) * (Simulation::CHUNK_SIZE as f32),
                (chunk_coord.y as f32) * (Simulation::CHUNK_SIZE as f32),
                WHITE,
            );
        }
        let time_render_post = Instant::now();
        self.last_render_duration = time_render_post.duration_since(time_render_pre);
    }

    fn ui_tool(&mut self) {
        widgets::Window::new(hash!(), vec2(0.0, 0.0), vec2(200.0, 150.0))
            .label("Tool")
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                if ui.button(None, "Air") {
                    self.dropper.set_material(Cell::AIR);
                }
                if ui.button(None, "Sand") {
                    self.dropper.set_material(Cell::SAND);
                }
                if ui.button(None, "Water") {
                    self.dropper.set_material(Cell::WATER);
                }
                if ui.button(None, "Stone") {
                    self.dropper.set_material(Cell::STONE);
                }
                let mut tool_size = self.dropper.get_size() as f32;
                ui.slider(0, "Tool Size", 1.0..10.0, &mut tool_size);
                self.dropper.set_size(tool_size.round() as i32);
                self.eraser.set_size(tool_size.round() as i32);
            });
    }

    fn ui_world(&mut self) {
        widgets::Window::new(hash!(), vec2(0.0, 150.0), vec2(200.0, 150.0))
            .label("World Creator")
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                let mut world_size_x: f32 = self.new_world_size.x as f32;
                let mut world_size_y: f32 = self.new_world_size.y as f32;

                const CHUNK_SIZE_F: f32 = Simulation::CHUNK_SIZE as f32;
                const WORLD_SIZE_MIN: f32 = Simulation::CHUNK_SIZE as f32;
                const WORLD_SIZE_MAX: f32 = 32.0 * Simulation::CHUNK_SIZE as f32;

                ui.slider(
                    1,
                    "World Size X",
                    WORLD_SIZE_MIN..WORLD_SIZE_MAX,
                    &mut world_size_x,
                );
                ui.slider(
                    2,
                    "World Size Y",
                    WORLD_SIZE_MIN..WORLD_SIZE_MAX,
                    &mut world_size_y,
                );
                world_size_x = (world_size_x / CHUNK_SIZE_F).round() * CHUNK_SIZE_F;
                world_size_y = (world_size_y / CHUNK_SIZE_F).round() * CHUNK_SIZE_F;
                self.new_world_size = ivec2(world_size_x as i32, world_size_y as i32);
                if ui.button(None, "World Size: Small") {
                    self.new_world_size = ivec2(1,1) * WORLD_SIZE_MIN as i32;
                }
                if ui.button(None, "World Size: Default") {
                    self.new_world_size = Self::WORLD_SIZE_DEFAULT;
                }
                if ui.button(None, "World Size: Large") {
                    self.new_world_size = ivec2(1,1) * WORLD_SIZE_MAX as i32;
                }
                if ui.button(None, "New World") {
                    self.generate_simulation(self.new_world_size);
                }
            });
    }

    fn ui_performance(&mut self) {
        widgets::Window::new(hash!(), vec2(0.0, 300.0), vec2(200.0, 150.0))
            .label("Performance and Speed")
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                ui.label(
                    None,
                    &format!(
                        "World Size: {}x{}",
                        self.simulation.size().x,
                        self.simulation.size().y
                    ),
                );
                ui.label(None, &format!("FPS:        {:.2}", 1.0 / get_frame_time()));
                ui.label(
                    None,
                    &format!(
                        "Sim tick:   {} ms",
                        (self.last_tick_duration.as_micros() as f32) / 1000.0
                    ),
                );
                ui.label(
                    None,
                    &format!(
                        "Render:     {} ms",
                        (self.last_render_duration.as_micros() as f32) / 1000.0
                    ),
                );
                ui.slider(
                    hash!(),
                    "Speed",
                    0.0..1.0,
                    &mut self.pulse.get_speed_scale_mut(),
                );
            });
    }

    pub fn ui(&mut self) {
        self.ui_tool();
        self.ui_world();
        self.ui_performance();
    }
}
