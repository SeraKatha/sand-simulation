mod performance_monitor;
mod pulse;
mod renderer;
mod tool;
mod view;

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use simulation::{Cell, Simulation};

use performance_monitor::PerformanceMonitor;
use pulse::Pulse;
use renderer::Renderer;
use view::View;

pub struct Application {
    simulation: Simulation,
    view: View,
    renderer: Renderer,
    dropper: tool::Dropper,
    eraser: tool::Dropper,
    new_world_size: IVec2,
    pulse: Pulse,
    performance_monitor: PerformanceMonitor,
}

impl Application {
    const WORLD_SIZE_DEFAULT: IVec2 = ivec2(128, 128);
    pub fn new() -> Application {
        if let Ok(simulation) = Simulation::new(ivec2(0, 0)) {
            let view = View::new(vec2(0.0, 0.0));
            let renderer = Renderer::new();
            let dropper = tool::Dropper::new(Cell::SAND, 3);
            let eraser = tool::Dropper::new(Cell::AIR, 3);
            let new_world_size = Self::WORLD_SIZE_DEFAULT;
            let mut application = Application {
                simulation,
                view,
                renderer,
                dropper,
                eraser,
                new_world_size,
                performance_monitor: PerformanceMonitor::new(),
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
            self.renderer.resize(&self.simulation);
            self.view = View::new(vec2(world_size.x as f32, world_size.y as f32));
        } else {
            panic!("AAAHHH!!!")
        }
    }

    pub fn update(&mut self) {
        let camera = self.view.into_camera_2d();
        set_camera(&camera);
        if self.pulse.tick(get_frame_time()) {
            self.performance_monitor.meassure_simulation(|| {
                self.simulation.tick();
            });
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
        self.performance_monitor.meassure_frame();
    }

    pub fn render(&mut self) {
        self.performance_monitor.meassure_rendering(|| {
            self.renderer.render(&self.simulation);
        });
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
                    self.new_world_size = ivec2(1, 1) * WORLD_SIZE_MIN as i32;
                }
                if ui.button(None, "World Size: Default") {
                    self.new_world_size = Self::WORLD_SIZE_DEFAULT;
                }
                if ui.button(None, "World Size: Large") {
                    self.new_world_size = ivec2(1, 1) * WORLD_SIZE_MAX as i32;
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
                ui.label(
                    None,
                    &format!("FPS:        {:.2}", self.performance_monitor.get_fps()),
                );
                ui.label(
                    None,
                    &format!(
                        "Simumation: {} ms",
                        (self
                            .performance_monitor
                            .get_simulation_duration()
                            .as_micros() as f32)
                            / 1000.0
                    ),
                );
                ui.label(
                    None,
                    &format!(
                        "Render:     {} ms",
                        (self
                            .performance_monitor
                            .get_rendering_duration()
                            .as_micros() as f32)
                            / 1000.0
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
