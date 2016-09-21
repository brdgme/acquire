pub const SAFE_SIZE: usize = 11;
pub const GAME_END_SIZE: usize = 41;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Corp {
    Worldwide,
    Sackson,
    Festival,
    Imperial,
    American,
    Continental,
    Tower,
}

fn additional_value(size: usize) -> usize {
    match size {
        _ if size >= 41 => 800,
        _ if size >= 31 => 700,
        _ if size >= 21 => 600,
        _ if size >= 11 => 500,
        _ if size >= 6 => 400,
        _ if size == 5 => 300,
        _ if size == 4 => 200,
        _ if size == 3 => 100,
        _ => 0,
    }
}

impl Corp {
    pub fn base_value(self) -> usize {
        use self::Corp::*;
        match self {
            Worldwide | Sackson => 200,
            Festival | Imperial | American => 300,
            Continental | Tower => 400,
        }
    }

    pub fn value(self, size: usize) -> usize {
        self.base_value() + additional_value(size)
    }
}
