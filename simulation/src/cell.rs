#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Cell {
    Air,
    Sand,
    Stone,
    Water,
    Lava,
    Steam,
}

impl Cell {
    pub const NUM_OF_TYPES: usize = 6; //
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
