#![feature(conservative_impl_trait)]
extern crate rand;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate error_chain;

extern crate brdgme_game;
extern crate brdgme_color;
extern crate brdgme_markup;

pub mod corp;
pub mod board;
mod render;
mod command;

use rand::{thread_rng, Rng};
use brdgme_game::{Gamer, Log, Status, CommandResponse};
use brdgme_game::errors::*;
use brdgme_game::command::Spec as CommandSpec;
use brdgme_markup::Node as N;

use std::collections::HashMap;
use std::iter::FromIterator;

use corp::Corp;
use board::{Board, Loc, Tile};
use command::Command;

pub const MIN_PLAYERS: usize = 2;
pub const MAX_PLAYERS: usize = 6;
pub const STARTING_MONEY: usize = 6000;
pub const STARTING_SHARES: usize = 25;
pub const TILE_HAND_SIZE: usize = 6;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Phase {
    Play(usize),
    Found { player: usize, at: Loc },
    Buy { player: usize, remaining: usize },
    ChooseMerger(usize),
    SellOrTrade {
        player: usize,
        corp: Corp,
        next_phase: Box<Phase>,
    },
}

impl Phase {
    pub fn whose_turn(&self) -> usize {
        match *self {
            Phase::Play(player) |
            Phase::Found { player, .. } |
            Phase::Buy { player, .. } |
            Phase::ChooseMerger(player) |
            Phase::SellOrTrade { player, .. } => player,
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
    pub priv_state: Option<PrivState>,
    pub players: HashMap<usize, PubPlayer>,
    pub board: Board,
    pub shares: HashMap<Corp, usize>,
    pub remaining_tiles: usize,
    pub finished: bool,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PrivState {
    pub id: usize,
    pub tiles: Vec<Loc>,
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

    fn new(players: usize) -> Result<(Self, Vec<Log>)> {
        let mut g = Game::default();
        if players < MIN_PLAYERS || players > MAX_PLAYERS {
            return Err(ErrorKind::PlayerCount(MIN_PLAYERS, MAX_PLAYERS, players).into());
        }

        // Shuffle up the draw tiles.
        let mut tiles = Loc::all();
        thread_rng().shuffle(tiles.as_mut_slice());
        g.draw_tiles = tiles;

        // Place initial tiles onto the board.
        for l in g.draw_tiles.drain(0..players) {
            g.board.set_tile(&l, Tile::Unincorporated);
        }

        // Set starting shares.
        for c in Corp::iter() {
            g.shares.insert(*c, STARTING_SHARES);
        }

        // Setup for each player.
        for p in 0..players {
            let mut player = Player::default();
            player.tiles = g.draw_tiles.drain(0..TILE_HAND_SIZE).collect();
            g.players.insert(p, player);
        }

        // Set the start player.
        let start_player = (thread_rng().next_u32() as usize) % players;
        g.phase = Phase::Play(start_player);

        Ok((g, vec![Log::public(vec![N::Player(start_player), N::text(" will start the game")])]))
    }

    fn status(&self) -> Status {
        if self.finished {
            Status::Finished {
                winners: vec![],
                stats: vec![],
            }
        } else {
            Status::Active {
                whose_turn: vec![self.phase.whose_turn()],
                eliminated: vec![],
            }
        }
    }

    fn pub_state(&self, player: Option<usize>) -> Self::PubState {
        PubState {
            priv_state: player.map(|ref p| {
                PrivState {
                    id: *p,
                    tiles: self.players
                        .get(p)
                        .map(|ps| ps.tiles.to_owned())
                        .unwrap_or_else(|| vec![]),
                }
            }),
            ..self.to_owned().into()
        }
    }

    fn command(&mut self,
               player: usize,
               input: &str,
               players: &[String])
               -> Result<CommandResponse> {
        let parser =
            self.command_parser(player)
                .ok_or_else::<Error, _>(|| {
                                            ErrorKind::InvalidInput("not your turn".to_string())
                                                .into()
                                        })?;
        let output = parser.parse(input, players)?;
        match output.value {
                Command::Play(loc) => self.play(player, &loc),
                Command::Found(corp) => self.found(player, &corp),
                Command::Buy(n, corp) => self.buy(player, n, corp).map(|l| (l, false)),
                Command::Done => self.done(player).map(|l| (l, false)),
                Command::Merge(corp, into) => self.merge(player, corp, into).map(|l| (l, false)),
                Command::Sell(n) => self.sell(player, n).map(|l| (l, false)),
                Command::Trade(n) => self.trade(player, n).map(|l| (l, false)),
                Command::Keep => self.keep(player).map(|l| (l, false)),
                Command::End => self.end(player).map(|l| (l, false)),
            }
            .map(|(logs, can_undo)| {
                     CommandResponse {
                         logs,
                         can_undo,
                         remaining_input: output.remaining.to_string(),
                     }
                 })
    }

    fn player_count(&self) -> usize {
        self.players.len()
    }

    fn player_counts() -> Vec<usize> {
        (2..6).collect()
    }

    fn command_spec(&self, player: usize) -> Option<CommandSpec> {
        self.command_parser(player).map(|p| p.to_spec())
    }
}

impl Game {
    fn can_end(&self) -> bool {
        false
    }
}

impl Game {
    pub fn can_play(&self, player: usize) -> bool {
        match self.phase {
            Phase::Play(p) if p == player => true,
            _ => false,
        }
    }

    pub fn assert_loc_playable(&self, loc: &Loc) -> Result<()> {
        if self.board.get_tile(loc) != Tile::Empty {
            bail!(ErrorKind::InvalidInput("location not empty".into()));
        }
        // Disallow joining multiple neighboring safe corps.
        let mut neighbouring: Option<Corp> = None;
        for n_loc in &loc.neighbours() {
            if let Tile::Corp(c) = self.board.get_tile(n_loc) {
                if self.board.corp_is_safe(c) {
                    if let Some(nc) = neighbouring {
                        if c != nc {
                            bail!(ErrorKind::InvalidInput(format!("can't merge {} and {} as they are both safe",
                                                                  c,
                                                                  nc)));
                        }
                    }
                    neighbouring = Some(c);
                }
            }
        }
        // Disallow founding a corp if there are no new ones available.
        Ok(())
    }

    pub fn play(&mut self, player: usize, loc: &Loc) -> Result<(Vec<Log>, bool)> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        if !self.can_play(player) {
            return Err(ErrorKind::InvalidInput("You can't play a tile right now".to_string())
                           .into());
        }
        let pos = match self.players
                  .get(&player)
                  .unwrap()
                  .tiles
                  .iter()
                  .position(|l| l == loc) {
            Some(p) => p,
            None => bail!(ErrorKind::InvalidInput("You don't have that tile".to_string())),
        };
        let neighbouring_corps = self.board.neighbouring_corps(loc);
        match neighbouring_corps.len() {
            1 => {
                self.board
                    .extend_corp(loc, neighbouring_corps.iter().next().unwrap());
                self.buy_phase();
            }
            0 => {
                self.board.set_tile(loc, Tile::Unincorporated);
                if loc.neighbours()
                       .iter()
                       .any(|n_loc| self.board.get_tile(n_loc) == Tile::Unincorporated) {
                    self.found_phase(loc.to_owned());
                } else {
                    self.buy_phase();
                }
            }
            _ => {}
        }
        self.board.set_tile(loc, Tile::Unincorporated);
        self.players
            .get_mut(&player)
            .unwrap()
            .tiles
            .swap_remove(pos);
        Ok((vec![], true))
    }

    fn buy_phase(&mut self) {
        self.phase = Phase::Buy {
            player: self.phase.whose_turn(),
            remaining: 3,
        };
    }

    fn found_phase(&mut self, loc: Loc) {
        self.phase = Phase::Found {
            player: self.phase.whose_turn(),
            at: loc,
        }
    }

    pub fn found(&mut self, player: usize, corp: &Corp) -> Result<(Vec<Log>, bool)> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        let at = match self.phase {
            Phase::Found { at, .. } => at,
            _ => {
                bail!(ErrorKind::InvalidInput("not able to found a corporation at the moment"
                                                  .to_string()))
            }
        };
        if !self.board.available_corps().contains(corp) {
            bail!(ErrorKind::InvalidInput(format!("{} is already on the board", corp)));
        }
        self.board.extend_corp(&at, corp);
        self.buy_phase();
        Ok((vec![], true))
    }

    pub fn buy(&mut self, player: usize, _n: usize, _corp: Corp) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        panic!("Not implemented");
    }

    pub fn done(&mut self, player: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        match self.phase {
            Phase::Buy { .. } => Ok(self.end_turn()),
            _ => bail!(ErrorKind::InvalidInput("can't end your turn at the moment".to_string())),
        }
    }

    fn end_turn(&mut self) -> Vec<Log> {
        self.phase = Phase::Play(self.next_player(self.phase.whose_turn()));
        vec![]
    }

    fn next_player(&self, player: usize) -> usize {
        (player + 1) % self.players.len()
    }

    pub fn merge(&mut self, player: usize, _corp: Corp, _into: Corp) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        panic!("Not implemented");
    }

    pub fn sell(&mut self, player: usize, _n: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        panic!("Not implemented");
    }

    pub fn trade(&mut self, player: usize, _n: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        panic!("Not implemented");
    }

    pub fn keep(&mut self, player: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        panic!("Not implemented");
    }

    pub fn end(&mut self, player: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
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
            priv_state: None,
            players: HashMap::from_iter(self.players.iter().map(|(k, v)| {
                                                                    (*k, v.to_owned().into())
                                                                })),
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
