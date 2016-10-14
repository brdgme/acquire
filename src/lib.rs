#![feature(proc_macro)]

extern crate rand;
extern crate combine;
#[macro_use]
extern crate serde_derive;

extern crate brdgme_game;
extern crate brdgme_color;
extern crate brdgme_markup;

use brdgme_game::{Gamer, GameError, Log};

pub mod corp;
pub mod board;
mod render;
mod parser;

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {}

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Game {}

impl Gamer for Game {
    type PlayerState = PlayerState;

    fn start(&mut self, players: usize) -> Result<Vec<Log>, GameError> {
        Err(GameError::Internal("Not implemented".to_string()))
    }

    fn is_finished(&self) -> bool {
        false
    }

    fn winners(&self) -> Vec<usize> {
        vec![]
    }

    fn whose_turn(&self) -> Vec<usize> {
        vec![]
    }

    fn player_state(&self, player: Option<usize>) -> Self::PlayerState {
        PlayerState::default()
    }

    fn command(&mut self,
               player: usize,
               input: &str,
               players: &[String])
               -> Result<(Vec<Log>, String), GameError> {
        Err(GameError::Internal("Not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
