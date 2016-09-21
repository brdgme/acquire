extern crate brdgme_cmd;
extern crate acquire;

use brdgme_cmd::repl;
use acquire::Game;

fn main() {
    repl(&Game::default());
}
