extern crate brdgme_game;
extern crate brdgme_rand_bot;
extern crate acquire;

use acquire::Game;
use brdgme_rand_bot::fuzz;

use std::io::stdout;

fn main() {
    fuzz::<Game, _>(&mut stdout());
}
