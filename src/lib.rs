#![feature(proc_macro, plugin)]
#![plugin(clippy)]

extern crate rand;
extern crate combine;
#[macro_use]
extern crate serde_derive;

extern crate brdgme_game;
extern crate brdgme_color;
extern crate brdgme_markup;

pub mod corp;
pub mod board;
mod render;
mod parser;

use rand::{thread_rng, Rng};
use combine::Parser;
use brdgme_game::{Gamer, GameError, Log};
use brdgme_markup::ast::{Node as N, Align as A};

use std::collections::HashMap;
use std::iter::FromIterator;

use corp::Corp;
use board::{Board, Loc, Tile};
use parser::Command;

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 6;
pub const STARTING_MONEY: usize = 6000;
pub const STARTING_SHARES: usize = 25;
pub const TILE_HAND_SIZE: usize = 6;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    Play(usize),
    Buy(usize),
    ChooseMerger(usize),
    SellOrTrade(usize, Box<Phase>),
}

impl Phase {
    pub fn whose_turn(&self) -> usize {
        match *self {
            Phase::Play(p) |
            Phase::Buy(p) |
            Phase::ChooseMerger(p) |
            Phase::SellOrTrade(p, _) => p,
        }
    }
}

impl Default for Phase {
    fn default() -> Self {
        Phase::Play(0)
    }
}

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PubState {
    pub phase: Phase,
    pub player: Option<usize>,
    pub players: HashMap<usize, PubPlayer>,
    pub board: Board,
    pub shares: HashMap<Corp, usize>,
    pub remaining_tiles: usize,
    pub finished: bool,
}

#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub phase: Phase,
    pub players: HashMap<usize, Player>,
    pub board: Board,
    pub draw_tiles: Vec<Loc>,
    pub shares: HashMap<Corp, usize>,
    pub finished: bool,
}

impl Gamer for Game {
    type PubState = PubState;

    fn start(&mut self, players: usize) -> Result<Vec<Log>, GameError> {
        if players < MIN_PLAYERS || players > MAX_PLAYERS {
            return Err(GameError::PlayerCount(MIN_PLAYERS, MAX_PLAYERS, players));
        }

        // Shuffle up the draw tiles.
        let mut tiles = Loc::all();
        thread_rng().shuffle(tiles.as_mut_slice());
        self.draw_tiles = tiles;

        // Place initial tiles onto the board.
        for l in self.draw_tiles.drain(0..players) {
            self.board.set_tile(l.into(), Tile::Unincorporated);
        }

        // Set starting shares.
        for c in Corp::iter() {
            self.shares.insert(*c, STARTING_SHARES);
        }

        // Setup for each player.
        for p in 0..players {
            let mut player = Player::default();
            player.tiles = self.draw_tiles.drain(0..TILE_HAND_SIZE).collect();
            self.players.insert(p, player);
        }

        // Set the start player.
        let start_player = (thread_rng().next_u32() as usize) % players;
        self.phase = Phase::Play(start_player);

        Ok(vec![
           Log::public(vec![
                N::Player(start_player),
                N::text(" will start the game"),
           ]),
        ])
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn winners(&self) -> Vec<usize> {
        vec![]
    }

    fn whose_turn(&self) -> Vec<usize> {
        match self.is_finished() {
            false => vec![self.phase.whose_turn()],
            true => vec![],
        }
    }

    fn pub_state(&self, player: Option<usize>) -> Self::PubState {
        PubState { player: player, ..self.to_owned().into() }
    }

    fn command(&mut self,
               player: usize,
               input: &str,
               players: &[String])
               -> Result<(Vec<Log>, String), GameError> {
        match parser::command().parse(input) {
            Ok((Command::Play(loc), remaining)) => {
                self.play(player, loc).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Buy(n, corp), remaining)) => {
                self.buy(player, n, corp).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Done, remaining)) => self.done(player).map(|l| (l, remaining.to_string())),
            Ok((Command::Merge(corp, into), remaining)) => {
                self.merge(player, corp, into).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Sell(n), remaining)) => {
                self.sell(player, n).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Trade(n), remaining)) => {
                self.trade(player, n).map(|l| (l, remaining.to_string()))
            }
            Ok((Command::Keep, remaining)) => self.keep(player).map(|l| (l, remaining.to_string())),
            Err(e) => Err(brdgme_game::parser::to_game_error(e)),
        }
    }
}

impl Game {
    pub fn can_play(&self, player: usize) -> bool {
        match self.phase {
            Phase::Play(p) if p == player => true,
            _ => false,
        }
    }
    pub fn play(&mut self, player: usize, loc: Loc) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        if !self.can_play(player) {
            return Err(GameError::InvalidInput("You can't play a tile right now".to_string()));
        }
        panic!("Not implemented");
    }

    pub fn buy(&mut self, player: usize, n: usize, corp: Corp) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        panic!("Not implemented");
    }

    pub fn done(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        panic!("Not implemented");
    }

    pub fn merge(&mut self, player: usize, corp: Corp, into: Corp) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        panic!("Not implemented");
    }

    pub fn sell(&mut self, player: usize, n: usize) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        panic!("Not implemented");
    }

    pub fn trade(&mut self, player: usize, n: usize) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        panic!("Not implemented");
    }

    pub fn keep(&mut self, player: usize) -> Result<Vec<Log>, GameError> {
        try!(self.assert_not_finished());
        try!(self.assert_player_turn(player));
        panic!("Not implemented");
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub money: usize,
    pub shares: HashMap<Corp, usize>,
    pub tiles: Vec<Loc>,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            money: STARTING_MONEY,
            shares: HashMap::new(),
            tiles: vec![],
        }
    }
}

impl Into<PubState> for Game {
    fn into(self) -> PubState {
        PubState {
            phase: self.phase,
            player: None,
            players: HashMap::from_iter(self.players
                .iter()
                .map(|(k, v)| (*k, v.to_owned().into()))),
            board: self.board,
            shares: self.shares,
            remaining_tiles: self.draw_tiles.len(),
            finished: self.finished,
        }
    }
}

impl Into<PubPlayer> for Player {
    fn into(self) -> PubPlayer {
        PubPlayer {
            money: self.money,
            shares: self.shares,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PubPlayer {
    pub money: usize,
    pub shares: HashMap<Corp, usize>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
