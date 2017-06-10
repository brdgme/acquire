extern crate brdgme_cmd;
extern crate acquire;

use acquire::Game;
use brdgme_cmd::repl;

fn main() {
    repl::<Game>();
}
