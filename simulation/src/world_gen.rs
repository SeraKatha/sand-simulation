use macroquad::prelude::*;
use noise::NoiseFn;
use crate::Cell;

pub trait WorldGenerator {
    fn gen_cell(&self, position : IVec2, size : IVec2) -> Cell;
}

pub struct EmptyWorldGenerator {}

impl EmptyWorldGenerator {
    pub fn new() -> Self {
        return Self {};
    }
}

impl WorldGenerator for EmptyWorldGenerator {
    fn gen_cell(&self, _position : IVec2, _size : IVec2) -> Cell {
        return Cell::Air;
    }
}

pub struct RandomWorldGenerator {}

impl RandomWorldGenerator {
    pub fn new() -> Self {
        return Self {};
    }
}

impl WorldGenerator for RandomWorldGenerator {
    fn gen_cell(&self, _position : IVec2, _size : IVec2) -> Cell {
        match ::rand::random_range(0..6) {
            0 => Cell::Air,
            1 => Cell::Sand,
            2 => Cell::Stone,
            3 => Cell::Water,
            4 => Cell::Lava,
            5 => Cell::Steam,
            _ => Cell::Air
        }
    }
}

pub struct IslandWorldGenerator {
    noise : noise::Perlin,
}

impl IslandWorldGenerator {
    pub fn new() -> Self {
        return Self {
            noise : noise::Perlin::new(::rand::random()),
        };
    }
}

impl WorldGenerator for IslandWorldGenerator {
    fn gen_cell(&self, position : IVec2, size : IVec2) -> Cell {
        let coord_x = position.x as f64 / size.x as f64;
        let coord_y = position.y as f64 / size.y as f64;
        println!("{} / {}", coord_x, coord_y);
        let ground_height : f64 = 0.5 + 0.1 * self.noise.get([5.0 * coord_x, 1337.0]);
        if ground_height < coord_y {
            return Cell::Sand;
        }
        else {
            if coord_y < 0.5 {
                return Cell::Air;
            }
            else {
                return Cell::Water;
            }
        }
    }
}