use brdgme_color::*;

use std::slice::Iter;
use std::fmt;

pub const SAFE_SIZE: usize = 11;
pub const GAME_END_SIZE: usize = 41;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Corp {
    Worldwide,
    Sackson,
    Festival,
    Imperial,
    American,
    Continental,
    Tower,
}

pub static CORPS: [Corp; 7] = [Corp::Worldwide,
                               Corp::Sackson,
                               Corp::Festival,
                               Corp::Imperial,
                               Corp::American,
                               Corp::Continental,
                               Corp::Tower];

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
    pub fn iter() -> Iter<'static, Corp> {
        CORPS.into_iter()
    }

    pub fn base_value(self) -> usize {
        match self {
            Corp::Worldwide | Corp::Sackson => 200,
            Corp::Festival | Corp::Imperial | Corp::American => 300,
            Corp::Continental | Corp::Tower => 400,
        }
    }

    pub fn value(self, size: usize) -> usize {
        self.base_value() + additional_value(size)
    }

    pub fn color(self) -> Color {
        match self {
            Corp::Worldwide => PURPLE,
            Corp::Sackson => DEEP_ORANGE,
            Corp::Festival => GREEN,
            Corp::Imperial => YELLOW,
            Corp::American => BLUE,
            Corp::Continental => RED,
            Corp::Tower => BLACK,
        }
    }

    pub fn name(self) -> String {
        format!("{}", self)
    }

    pub fn abbrev(self) -> String {
        self.name()[..2].to_uppercase()
    }
}

impl fmt::Display for Corp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
