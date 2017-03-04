use brdgme_game::Renderer;
use brdgme_markup::ast::{Node as N, Align as A};
use brdgme_color::*;

use super::PubState;
use board::{Board, Loc, Tile};
use corp::Corp;

use std::iter::repeat;

const TILE_WIDTH: usize = 5;
const TILE_HEIGHT: usize = 2;

static EMPTY_COLOR_EVEN: Color = Color {
    r: 220,
    g: 220,
    b: 220,
};

static EMPTY_COLOR_ODD: Color = Color {
    r: 180,
    g: 180,
    b: 180,
};

static UNINCORPORATED_COLOR: Color = Color {
    r: 40,
    g: 40,
    b: 40,
};

impl Renderer for PubState {
    fn render(&self) -> Vec<N> {
        vec![N::Table(vec![vec![(A::Center, vec![self.board.render()])]])]
    }
}

fn tile_background(c: Color) -> N {
    N::Bg(c,
          vec![N::text(repeat(repeat(" ").take(TILE_WIDTH).collect::<String>())
                   .take(TILE_HEIGHT)
                   .collect::<Vec<String>>()
                   .join("\n"))])
}

fn empty_color(l: Loc) -> Color {
    if (l.row + l.col) % 2 == 0 {
        EMPTY_COLOR_EVEN
    } else {
        EMPTY_COLOR_ODD
    }
}

impl Board {
    pub fn render(&self) -> N {
        let mut layers = vec![];
        for l in Loc::all() {
            let render_x = l.col * TILE_WIDTH;
            let render_y = l.row * TILE_HEIGHT;
            match self.get_tile(l.into()) {
                Tile::Empty => {
                    layers.push((render_x, render_y, vec![tile_background(empty_color(l))]));
                    layers.push((render_x,
                                 render_y,
                                 vec![N::Align(A::Center, TILE_WIDTH, vec![N::text(l.name())])]));
                }
                Tile::Unincorporated => {
                    layers.push((render_x, render_y, vec![tile_background(UNINCORPORATED_COLOR)]));
                }
                _ => {}
            }
        }
        N::Canvas(layers)
    }
}

impl Corp {
    pub fn render_in_color(self, content: Vec<N>) -> N {
        N::Bg(self.color(),
              vec![N::Fg(self.color().mono().inv(), vec![N::Bold(content)])])
    }

    pub fn render_name(self) -> N {
        self.render_in_color(vec![N::text(self.name())])
    }

    pub fn render_abbrev(self) -> N {
        self.render_in_color(vec![N::text(self.abbrev())])
    }
}
