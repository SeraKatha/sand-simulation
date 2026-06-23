mod performance_monitor;
mod pulse;
mod renderer;
mod tool;
mod view;

use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use simulation::{Cell, SaveData, Simulation};

use performance_monitor::PerformanceMonitor;
use pulse::Pulse;
use renderer::TexturedRenderer;
use renderer::Renderer;
use tool::Tool;
use view::View;

use crate::file_explorer::FileExplorer;
use crate::renderer::SingleColorRenderer;
mod file_explorer;
pub struct Application {
    simulation: Simulation,
    view: View,
    renderer: Box::<dyn Renderer>,
    dropper: tool::Dropper,
    eraser: tool::Dropper,
    new_world_size: IVec2,
    pulse: Pulse,
    performance_monitor: PerformanceMonitor,
    file_explorer: FileExplorer,
}

impl Application {
    const WORLD_SIZE_DEFAULT: IVec2 = ivec2(128, 128);
    pub fn new() -> Application {
        if let Ok(simulation) = Simulation::new(ivec2(0, 0)) {
            let view = View::new(vec2(0.0, 0.0));
            let renderer = TexturedRenderer::new();
            let dropper = tool::Dropper::new(Cell::Sand, 3);
            let eraser = tool::Dropper::new(Cell::Air, 3);
            let new_world_size = Self::WORLD_SIZE_DEFAULT;
            let mut application = Application {
                simulation,
                view,
                renderer : Box::from(renderer),
                dropper,
                eraser,
                new_world_size,
                performance_monitor: PerformanceMonitor::new(),
                file_explorer: file_explorer::FileExplorer::new(),
                pulse: Pulse::new(60.0),
            };
            application.generate_simulation(Self::WORLD_SIZE_DEFAULT);
            return application;
        } else {
            panic!("AAAHHH!!!")
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.update();
            self.render();
            self.ui();
            next_frame().await
        }
    }

    fn generate_simulation(&mut self, world_size: IVec2) {
        if let Ok(simulation) = Simulation::new(world_size) {
            self.simulation = simulation;
            self.renderer.fit_simulation(&self.simulation);
            self.view = View::new(vec2(
                self.simulation.size().x as f32,
                self.simulation.size().y as f32,
            ));
        }
    }

    fn update_simulation(&mut self) {
        if self.pulse.tick(get_frame_time()) {
            // Update simulation when ready.
            self.performance_monitor.meassure_simulation(|| {
                self.simulation.tick();
            });
        } else {
            // Skip simulation steps when the simulation speed is slowed down
            self.simulation.pass();
        }
    }

    fn update_tool(&mut self) {
        // Apply tool if user presses the corresponding mouse buttons
        let camera = self.view.get_camera_2d();
        let mouse_position = macroquad::input::mouse_position();
        let global_coord_vec2 = camera.screen_to_world(vec2(mouse_position.0, mouse_position.1));
        let global_coord_ivec2 = ivec2(global_coord_vec2.x as i32, global_coord_vec2.y as i32);
        // Checking whether the mouse is over an UI element prevent accidentally drawing cells behind the UI.
        if !root_ui().is_mouse_over(mouse_position.into()) {
            if is_mouse_button_down(MouseButton::Left) {
                self.dropper.apply(&mut self.simulation, global_coord_ivec2);
            }
            if is_mouse_button_down(MouseButton::Right) {
                self.eraser.apply(&mut self.simulation, global_coord_ivec2);
            }
        }
    }

    fn update(&mut self) {
        if self.file_explorer.state() == file_explorer::State::CLOSED {
            self.update_simulation();
            self.update_tool();
            self.simulation.swap_buffers();
            self.view.update();
            self.performance_monitor.meassure_frame();
        }
    }

    fn render(&mut self) {
        self.performance_monitor.meassure_rendering(|| {
            let camera = self.view.get_camera_2d();
            set_camera(camera);
            clear_background(DARKGRAY);
            self.renderer.render(&self.simulation);
        });
    }

    fn ui_tool(&mut self) {
        widgets::Window::new(hash!(), vec2(0.0, 0.0), vec2(200.0, 150.0))
            .label("Tool")
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                let mut tool_size = self.dropper.get_size() as f32;
                ui.slider(0, "Tool Size", 1.0..10.0, &mut tool_size);
                self.dropper.set_size(tool_size.round() as i32);
                self.eraser.set_size(tool_size.round() as i32);

                let materials = [
                    ("Air", Cell::Air),
                    ("Sand", Cell::Sand),
                    ("Water", Cell::Water),
                    ("Stone", Cell::Stone),
                    ("Lava", Cell::Lava),
                    ("Steam", Cell::Steam),
                ];

                for (i, (label, cell)) in materials.iter().enumerate() {
                    let spacing_h = 50.0;
                    let spacing_v = 20.0;
                    let columns = 2;
                    let offset = vec2(
                        spacing_h * (i % columns) as f32,
                        spacing_v * (i / columns) as f32,
                    );
                    if ui.button(vec2(2.0, 24.0) + offset, *label) {
                        self.dropper.set_material(*cell);
                    }
                }
            }
        );
    }

    fn ui_world(&mut self) {
        widgets::Window::new(hash!(), vec2(0.0, 150.0), vec2(200.0, 200.0))
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
                if ui.button(None, "Load") {
                    self.file_explorer.load();
                }
                if ui.button(None, "Save") {
                    self.file_explorer.save();
                }
            });
    }

    fn ui_performance(&mut self) {
        widgets::Window::new(hash!(), vec2(0.0, 350.0), vec2(200.0, 150.0))
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

    fn ui_rendering(&mut self) {
        widgets::Window::new(hash!(), vec2(0.0, 500.0), vec2(200.0, 150.0))
            .label("Rendering")
            .movable(false)
            .ui(&mut root_ui(), |ui| {
                if ui.button(None, "Single Colored") {
                    self.renderer = Box::from(SingleColorRenderer::new());
                    self.renderer.fit_simulation(&self.simulation);
                }
                if ui.button(None, "Textures") {
                    self.renderer = Box::from(TexturedRenderer::new());
                    self.renderer.fit_simulation(&self.simulation);
                }
            }
        );
    }

    fn handle_file_explorer(&mut self) {
        // Ugly work-around to make the borrow-checker-happy
        match self.file_explorer.state() {
            file_explorer::State::SAVE => {
                let on_save = |path| {
                    let serialized =
                        serde_json::to_string(&self.simulation.to_save_data()).unwrap();
                    std::fs::create_dir_all("./saves").unwrap();
                    std::fs::write(path, serialized).unwrap();
                };
                self.file_explorer.ui(on_save, |_| {});
            }
            file_explorer::State::LOAD => {
                let on_load = |path| {
                    let serialized = std::fs::read_to_string(path).unwrap();
                    let save_data: SaveData = serde_json::from_str(&serialized).unwrap();
                    if let Ok(simulation) = Simulation::from_save_data(save_data) {
                        self.simulation = simulation;
                        self.renderer.fit_simulation(&self.simulation);
                        self.view = View::new(vec2(
                            self.simulation.size().x as f32,
                            self.simulation.size().y as f32,
                        ));
                    }
                };
                self.file_explorer.ui(|_| {}, on_load);
            }
            file_explorer::State::CLOSED => {}
        }
    }

    fn ui(&mut self) {
        self.ui_tool();
        self.ui_world();
        self.ui_performance();
        self.ui_rendering();
        self.handle_file_explorer();
    }
}
