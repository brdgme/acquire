extern crate acquire;
extern crate brdgme_cmd;
extern crate brdgme_fuzz;
extern crate brdgme_game;
extern crate brdgme_rand_bot;

use acquire::Game;

fn main() {
    brdgme_fuzz::fuzz_gamer::<Game>();
}
