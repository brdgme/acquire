use brdgme_game::Renderer;
use brdgme_markup::{Node as N, Align as A};
use brdgme_color::*;

use super::PubState;
use board::{self, Board, Loc, Tile};
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
    r: 190,
    g: 190,
    b: 190,
};

static UNINCORPORATED_COLOR: Color = Color {
    r: 100,
    g: 100,
    b: 100,
};

static UNAVAILABLE_LOC_TEXT_COLOR: Color = Color {
    r: 80,
    g: 80,
    b: 80,
};

static AVAILABLE_LOC_BG: Color = Color {
    r: 248,
    g: 187,
    b: 208,
};

impl Renderer for PubState {
    fn render(&self) -> Vec<N> {
        let tiles = self.clone().priv_state.map(|ps| ps.tiles).unwrap_or_else(|| vec![]);
        vec![N::Table(vec![vec![(A::Center, vec![self.board.render(&tiles)])]])]
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

fn corp_main_text_thin(c: Corp, size: usize) -> Vec<N> {
    vec![N::Fg(c.color().inv().mono(),
               vec![N::Align(A::Center,
                             TILE_WIDTH,
                             vec![N::text(format!("{}\n${}", c.abbrev(), c.value(size)))])])]
}

fn corp_main_text_wide(c: Corp, size: usize) -> Vec<N> {
    let mut c_name = c.name();
    c_name.truncate(TILE_WIDTH * 2 - 2);
    vec![N::Fg(c.color().inv().mono(),
               vec![N::Align(A::Center,
                             TILE_WIDTH * 2,
                             vec![N::text(format!("{}\n${}", c_name, c.value(size)))])])]
}

impl Board {
    pub fn render(&self, player_tiles: &[Loc]) -> N {
        let mut layers = vec![];
        // Tile backgrounds and location text.
        for l in Loc::all() {
            let render_x = l.col * TILE_WIDTH;
            let render_y = l.row * TILE_HEIGHT;
            match self.get_tile(l.into()) {
                Tile::Empty => {
                    layers.push((render_x, render_y, vec![tile_background(empty_color(l))]));
                    layers.push((render_x,
                                 render_y,
                                 vec![N::Align(A::Center,
                                               TILE_WIDTH,
                                               vec![N::Fg(UNAVAILABLE_LOC_TEXT_COLOR,
                                                          vec![N::text(l.name())])])]));
                }
                Tile::Unincorporated => {
                    layers.push((render_x, render_y, vec![tile_background(UNINCORPORATED_COLOR)]));
                }
                Tile::Corp(ref c) => {
                    layers.push((render_x, render_y, vec![tile_background(c.color())]));
                }
                Tile::Discarded => {}
            }
        }
        // Player tiles.
        for t in player_tiles {
            let l = Loc::from(*t);
            let render_x = l.col * TILE_WIDTH;
            let render_y = l.row * TILE_HEIGHT;
            layers.push((render_x, render_y, vec![tile_background(AVAILABLE_LOC_BG)]));
            layers.push((render_x,
                         render_y,
                         vec![N::Align(A::Center,
                                       TILE_WIDTH,
                                       vec![N::Bold(vec![N::Fg(AVAILABLE_LOC_BG.inv()
                                                                   .mono(),
                                                               vec![N::text(l.name())])])])]));
        }
        // Corp text.
        layers.extend(Corp::iter()
            .flat_map(|c| {
                let mut c_text = vec![];
                // Find the widest lines.
                // `widths` is a tuple of x, y, width.
                let widths: Vec<(usize, usize, usize)> = board::rows()
                    .flat_map(|row| {
                        let mut start: Option<usize> = None;
                        board::cols()
                            .filter_map(|col| {
                                let l = Loc {
                                    row: row,
                                    col: col,
                                };
                                match self.get_tile(l.into()) {
                                    Tile::Corp(tc) if tc == *c => {
                                        if start.is_none() {
                                            start = Some(col);
                                        }
                                        if col == board::WIDTH - 1 {
                                            Some((start.unwrap(), row, col - start.unwrap() + 1))
                                        } else {
                                            None
                                        }
                                    }
                                    _ => {
                                        if let Some(s) = start {
                                            start = None;
                                            Some((s, row, col - s))
                                        } else {
                                            None
                                        }
                                    }
                                }
                            })
                            .collect::<Vec<(usize, usize, usize)>>()
                    })
                    .collect();
                if !widths.is_empty() {
                    let (x, y, w) = widths[(widths.len() - 1) / 2];
                    c_text.push(((x + (w - 1) / 2) * TILE_WIDTH,
                                 y * TILE_HEIGHT,
                                 if w > 1 {
                                     corp_main_text_wide(*c, self.corp_size(*c))
                                 } else {
                                     corp_main_text_thin(*c, self.corp_size(*c))
                                 }));
                }
                c_text
            })
            .collect::<Vec<(usize, usize, Vec<N>)>>());
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
