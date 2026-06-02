#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Cell {
    Air,
    Sand,
    Stone,
    Water,
}

impl Cell {
    pub const NUM_OF_TYPES: usize = 4; //
}

impl Cell {
    pub fn is_liquid(&self) -> bool {
        match self {
            Self::Water => true,
            _ => false,
        }
    }

    pub fn is_gaseous(&self) -> bool {
        match self {
            Self::Air => true,
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
}
