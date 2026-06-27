use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum Cell {
    Barrier = 0,
    Air = 1,
    Sand = 2,
    Stone = 3,
    Water = 4,
    Lava = 5,
    Steam = 6,
}

impl TryFrom<u8> for Cell {
    type Error = ();
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Barrier),
            1 => Ok(Self::Air),
            2 => Ok(Self::Sand),
            3 => Ok(Self::Stone),
            4 => Ok(Self::Water),
            5 => Ok(Self::Lava),
            6 => Ok(Self::Steam),
            _ => Err(()),
        }
    }
}

impl Cell {
    pub const NUM_OF_TYPES: usize = 7; //
}

impl Cell {
    pub fn is_liquid(&self) -> bool {
        match self {
            Self::Water => true,
            Self::Lava => true,
            _ => false,
        }
    }

    pub fn viscosity(&self) -> f64 {
        match self {
            Self::Lava => 0.9,
            _ => 0.0,
        }
    }

    pub fn is_hot(&self) -> bool {
        match self {
            Self::Lava => true,
            _ => false,
        }
    }

    pub fn density(&self) -> i32 {
        match self {
            Cell::Barrier => 0,
            Cell::Air     => 1,
            Cell::Sand    => 3,
            Cell::Stone   => 0,
            Cell::Water   => 2,
            Cell::Lava    => 2,
            Cell::Steam   => 0,
        }
    }

    pub fn is_falling(&self) -> bool {
        match self {
            Cell::Barrier => false,
            Cell::Air     => true,
            Cell::Sand    => true,
            Cell::Stone   => false,
            Cell::Water   => true,
            Cell::Lava    => true,
            Cell::Steam   => true,
        }
    }

    pub fn is_piling(&self) -> bool {
        match self {
            Cell::Barrier => false,
            Cell::Air     => true,
            Cell::Sand    => true,
            Cell::Stone   => false,
            Cell::Water   => true,
            Cell::Lava    => true,
            Cell::Steam   => true,
        }
    }

    pub fn is_spreading(&self) -> bool {
        match self {
            Cell::Barrier => false,
            Cell::Air     => true,
            Cell::Sand    => false,
            Cell::Stone   => false,
            Cell::Water   => true,
            Cell::Lava    => true,
            Cell::Steam   => true,
        }
    }
}
