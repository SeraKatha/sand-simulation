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

    pub fn is_gaseous(&self) -> bool {
        match self {
            Self::Air => true,
            Self::Steam => true,
            _ => false,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self {
            Self::Barrier => true,
            Self::Sand => true,
            Self::Stone => true,
            _ => false,
        }
    }

    pub fn is_non_solid(&self) -> bool {
        !self.is_solid()
    }

    pub fn is_hot(&self) -> bool {
        match self {
            Self::Lava => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Air => true,
            _ => false,
        }
    }
}
