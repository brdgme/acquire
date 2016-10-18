use brdgme_game::Renderer;
use brdgme_markup::ast::{Node as N, Align as A};

use std::fmt::Display;

use super::PubState;
use board::{Board, Loc, Tile, rows, cols};
use corp::Corp;

impl Renderer for PubState {
    fn render(&self) -> Vec<N> {
        vec![self.board.render()]
    }
}

impl Board {
    pub fn render(&self) -> N {
        N::Table(rows()
            .map(|r| {
                cols()
                    .map(|c| {
                        let l = Loc { row: r, col: c };
                        (A::Left, vec![N::text(l.name())])
                    })
                    .collect()
            })
            .collect())
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
