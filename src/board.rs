use super::corp::{self, Corp};

use std::iter;
use std::ops::Range;

pub const WIDTH: usize = 12;
pub const HEIGHT: usize = 9;
pub const SIZE: usize = WIDTH * HEIGHT;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Discarded,
    Unincorporated,
    Corp(Corp),
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Board(pub Vec<Tile>);

impl Board {
    pub fn get_tile(&self, at: usize) -> Tile {
        self.0.get(at).cloned().unwrap_or(Tile::default())
    }

    pub fn set_tile(&mut self, at: usize, t: Tile) {
        let len = self.0.len();
        if len <= at {
            self.0.extend(iter::repeat(Tile::default()).take(at - len + 1))
        }
        self.0[at] = t;
    }

    pub fn corp_size(&self, c: Corp) -> usize {
        self.0
            .iter()
            .filter(|t| match t {
                &&Tile::Corp(tc) if tc == c => true,
                _ => false,
            })
            .count()
    }

    pub fn corp_is_safe(&self, c: Corp) -> bool {
        self.corp_size(c) >= corp::SAFE_SIZE
    }
}

impl Default for Board {
    fn default() -> Self {
        Board(iter::repeat(Tile::default()).take(SIZE).collect())
    }
}

pub fn rows() -> Range<usize> {
    0..HEIGHT
}

pub fn cols() -> Range<usize> {
    0..WIDTH
}

#[derive(Copy, Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Loc {
    pub row: usize,
    pub col: usize,
}

impl Loc {
    pub fn all() -> Vec<Loc> {
        rows().flat_map(move |r| cols().map(move |c| Loc { row: r, col: c })).collect()
    }

    pub fn neighbours(&self) -> Vec<Loc> {
        let mut n = vec![];
        if self.col > 0 {
            n.push(Loc { col: self.col - 1, ..*self });
        }
        if self.col < WIDTH - 1 {
            n.push(Loc { col: self.col + 1, ..*self });
        }
        if self.row > 0 {
            n.push(Loc { row: self.row - 1, ..*self });
        }
        if self.row < HEIGHT - 1 {
            n.push(Loc { row: self.row + 1, ..*self });
        }
        n
    }

    pub fn name(&self) -> String {
        format!("{}{}", ('A' as u8 + self.row as u8) as char, self.col + 1)
    }
}

impl From<usize> for Loc {
    fn from(u: usize) -> Self {
        Loc {
            row: u / WIDTH,
            col: u % WIDTH,
        }
    }
}

impl From<Loc> for usize {
    fn from(l: Loc) -> Self {
        l.row * WIDTH + l.col
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::corp::Corp;

    #[test]
    fn usize_into_loc_works() {
        assert_eq!(Loc::default(), 0.into());
        assert_eq!(Loc(8, 0), 8.into());
        assert_eq!(Loc(3, 2), 27.into());
        assert_eq!(Loc(11, 1), 23.into());
    }

    #[test]
    fn loc_into_usize_works() {
        assert_eq!(0 as usize, Loc::default().into());
        assert_eq!(8 as usize, Loc(8, 0).into());
        assert_eq!(27 as usize, Loc(3, 2).into());
        assert_eq!(23 as usize, Loc(11, 1).into());
    }

    #[test]
    fn board_get_tile_works() {
        let mut b = Board::default();
        b.set_tile(5, Tile::Discarded);
        assert_eq!(Tile::Discarded, b.get_tile(5));
        assert_eq!(Tile::Empty, b.get_tile(99999));
    }

    #[test]
    fn board_indexing_by_loc_works() {
        let b = Board::default();
        assert_eq!(Tile::Empty, b.get_tile(Loc(4, 5).into()));
    }

    #[test]
    fn board_set_tile_works() {
        let mut b = Board::default();
        b.set_tile(99999, Tile::Unincorporated);
    }

    #[test]
    fn board_corp_size_works() {
        let mut b = Board::default();
        b.set_tile(2, Tile::Corp(Corp::American));
        b.set_tile(3, Tile::Corp(Corp::American));
        b.set_tile(4, Tile::Corp(Corp::Sackson));
        assert_eq!(0, b.corp_size(Corp::Continental));
        assert_eq!(1, b.corp_size(Corp::Sackson));
        assert_eq!(2, b.corp_size(Corp::American));
    }
}
