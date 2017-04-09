extern crate brdgme_cmd;

extern crate acquire;

use brdgme_cmd::cli::cli;
use acquire::Game;
use std::io::{stdin, stdout};

fn main() {
    cli::<Game, _, _>(stdin(), &mut stdout());
}
