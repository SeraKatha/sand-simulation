use std::path::PathBuf;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    SAVE,
    LOAD,
    CLOSED,
}

pub struct FileExplorer {
    path : PathBuf,
    state : State,
    input_string : String,
}

impl FileExplorer {
    pub fn new() -> Self {
        Self { path: PathBuf::from(".").canonicalize().unwrap(), state: State::CLOSED, input_string : "".to_string() }
    }

    fn go_into_dir(&mut self, name : &str) {
        self.path = self.path.join(name);
        println!("{:?} - {}", self.path, name);
        // self.path = self.path.canonicalize().unwrap();
    }

    fn go_to_parent(&mut self) {
        if let Some(p) = self.path.parent() {
            self.path = p.to_path_buf()
        }
    }

    pub fn save(&mut self) {
        self.path = PathBuf::from(".").canonicalize().unwrap();
        self.state = State::SAVE;
    }

    pub fn load(&mut self) {
        self.path = PathBuf::from(".").canonicalize().unwrap();
        self.state = State::LOAD;
    }

    fn ui_impl(&mut self, title : &str, callback : impl FnOnce(PathBuf) -> (), allow_new_file : bool) {
        widgets::Window::new(hash!(), vec2(200.0, 500.0), vec2(400.0, 400.0))
            .label(title)
            .movable(true)
            .ui(&mut root_ui(), |ui| {
                if ui.button(None, "..") {
                    self.go_to_parent();
                }
                let entries = std::fs::read_dir(self.path.clone()).unwrap();
                for entry in entries.into_iter() {
                    let path = entry.unwrap().path();
                    if path.is_dir() {
                        let name = path.file_name().unwrap().to_str().unwrap();
                        if ui.button(None, format!("| {}", name)) {
                            self.go_into_dir(&name);
                        }
                    }
                }
                let entries = std::fs::read_dir(self.path.clone()).unwrap();
                let mut selected = None;
                for entry in entries.into_iter() {
                    let path = entry.unwrap().path();
                    if path.is_file() && path.extension().map(|x| x == "sand") == Some(true) {
                        let name = path.file_name().unwrap().to_str().unwrap();
                        if ui.button(None, format!("> {}", name)) {
                            selected = Some(self.path.join(name));
                        }
                    }
                }
                if allow_new_file {
                    ui.input_text(hash!(), ".sand", &mut self.input_string);
                    if ui.button(None, "Save as new file!") {
                        selected = Some(self.path.join(format!("{}.sand", &self.input_string)));
                    }
                }
                if let Some(path) = selected {
                    callback(path);
                    self.state = State::CLOSED;
                }
            }
        );
    }

    pub fn ui(&mut self, on_save : impl FnOnce(PathBuf) -> (), on_load : impl FnOnce(PathBuf) -> ()) {
        match self.state {
            State::SAVE => {
                self.ui_impl("Save Simulation", on_save, true);
            },
            State::LOAD => {
                self.ui_impl("Load Simulation", on_load, false);
            },
            State::CLOSED => {},
        }
    }

    pub fn state(&self) -> State {
        self.state
    }
}