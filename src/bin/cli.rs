extern crate brdgme_cmd;
extern crate acquire;

use acquire::Game;
use brdgme_cmd::cli::cli;

use std::io;

fn main() {
    cli::<Game, _, _>(io::stdin(), &mut io::stdout());
}
