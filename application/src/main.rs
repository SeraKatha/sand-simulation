use macroquad::prelude::*;
use application::Application;

#[macroquad::main("SandFalls")]
async fn main() {
    set_default_filter_mode(FilterMode::Nearest);
    let mut application = Application::new();
    
    loop {
        application.update();
        application.render();
        application.ui();
            
        next_frame().await
    }
}