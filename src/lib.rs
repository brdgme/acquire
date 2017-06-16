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
    ChooseMerger { player: usize, at: Loc },
    SellOrTrade {
        player: usize,
        corp: Corp,
        at: Loc,
        turn_player: usize,
    },
}

impl Phase {
    pub fn whose_turn(&self) -> usize {
        match *self {
            Phase::Play(player) |
            Phase::Found { player, .. } |
            Phase::Buy { player, .. } |
            Phase::ChooseMerger { player, .. } |
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
    pub players: Vec<PubPlayer>,
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
    pub players: Vec<Player>,
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
            return Err(
                ErrorKind::PlayerCount(MIN_PLAYERS, MAX_PLAYERS, players).into(),
            );
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

        Ok((
            g,
            vec![
                Log::public(vec![
                    N::Player(start_player),
                    N::text(" will start the game"),
                ]),
            ],
        ))
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
            priv_state: player.map(|p| {
                PrivState {
                    id: p,
                    tiles: self.players[p].tiles.to_owned(),
                }
            }),
            ..self.to_owned().into()
        }
    }

    fn command(
        &mut self,
        player: usize,
        input: &str,
        players: &[String],
    ) -> Result<CommandResponse> {
        let parser = self.command_parser(player).ok_or_else::<Error, _>(|| {
            ErrorKind::InvalidInput("not your turn".to_string()).into()
        })?;
        let output = parser.parse(input, players)?;
        match output.value {
            Command::Play(loc) => self.play(player, &loc),
            Command::Found(corp) => self.found(player, &corp),
            Command::Buy(n, corp) => self.buy(player, n, corp),
            Command::Done => self.done(player).map(|l| (l, false)),
            Command::Merge(corp, into) => self.merge(player, &corp, &into).map(|l| (l, false)),
            Command::Sell(n) => self.sell(player, n).map(|l| (l, false)),
            Command::Trade(n) => self.trade(player, n).map(|l| (l, false)),
            Command::Keep => self.keep(player).map(|l| (l, false)),
            Command::End => self.end(player).map(|l| (l, false)),
        }.map(|(logs, can_undo)| {
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

    fn draw_replacement_tiles(&mut self, player: usize) -> (Vec<Log>, bool) {
        let remaining = TILE_HAND_SIZE - self.players[player].tiles.len();
        if self.draw_tiles.len() < remaining {
            // End of game
            return (vec![], false);
        }
        self.players[player].tiles.extend(self.draw_tiles.drain(
            0..remaining,
        ));
        (vec![], true)
    }

    pub fn play(&mut self, player: usize, loc: &Loc) -> Result<(Vec<Log>, bool)> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        if !self.can_play(player) {
            return Err(
                ErrorKind::InvalidInput("You can't play a tile right now".to_string()).into(),
            );
        }
        let pos = match self.players[player].tiles.iter().position(|l| l == loc) {
            Some(p) => p,
            None => {
                bail!(ErrorKind::InvalidInput(
                    "You don't have that tile".to_string(),
                ))
            }
        };
        let mut logs: Vec<Log> = vec![
            Log::public(vec![
                N::Player(player),
                N::text(" played "),
                N::Bold(vec![N::text(format!("{}", loc))]),
            ]),
        ];
        let neighbouring_corps = self.board.neighbouring_corps(loc);
        match neighbouring_corps.len() {
            1 => {
                let n_corp = neighbouring_corps.iter().next().unwrap();
                self.board.extend_corp(loc, n_corp);
                logs.push(Log::public(vec![
                    n_corp.render(),
                    N::text(" increased in size to "),
                    N::Bold(vec![
                        N::text(format!("{}", self.board.corp_size(n_corp))),
                    ]),
                ]));
                self.buy_phase(player);
            }
            0 => {
                let has_unincorporated_neighbour = loc.neighbours().iter().any(|n_loc| {
                    self.board.get_tile(n_loc) == Tile::Unincorporated
                });
                if has_unincorporated_neighbour {
                    if self.board.available_corps().is_empty() {
                        bail!(ErrorKind::InvalidInput(
                            "there aren't any corporations available to found"
                                .to_string(),
                        ));
                    }
                    self.found_phase(player, loc.to_owned());
                } else {
                    self.buy_phase(player);
                }
                // Set the tile last as errors can be thrown above.
                self.board.set_tile(loc, Tile::Unincorporated);
            }
            _ => {
                let safe_corp_count = neighbouring_corps.iter().fold(0, |acc, corp| {
                    if self.board.corp_is_safe(corp) {
                        acc + 1
                    } else {
                        acc
                    }
                });
                if safe_corp_count > 1 {
                    bail!(ErrorKind::InvalidInput(
                        "can't merge safe corporations together".to_string(),
                    ));
                }
                self.board.set_tile(loc, Tile::Unincorporated);
                logs.extend(self.choose_merger_phase(player, *loc)?);
            }
        }
        self.players[player].tiles.swap_remove(pos);
        Ok((logs, true))
    }

    fn buy_phase(&mut self, player: usize) {
        self.phase = Phase::Buy {
            player: player,
            remaining: 3,
        };
    }

    fn found_phase(&mut self, player: usize, loc: Loc) {
        self.phase = Phase::Found {
            player: player,
            at: loc,
        }
    }

    fn choose_merger_phase(&mut self, player: usize, loc: Loc) -> Result<Vec<Log>> {
        let (from, into) = self.board.merge_candidates(&loc);
        if into.is_empty() {
            // No mergers, go to buy phase.
            self.buy_phase(player);
            return Ok(vec![]);
        }
        // We set the phase as the merge function validates this.
        self.phase = Phase::ChooseMerger {
            player: player,
            at: loc,
        };
        if from.len() == 1 && into.len() == 1 && from[0] != into[0] {
            // There's no ambiguity, automatically make the merge.
            self.merge(player, &from[0], &into[0])
        } else {
            // Stay in this phase so the player can choose.
            Ok(vec![])
        }
    }

    pub fn found(&mut self, player: usize, corp: &Corp) -> Result<(Vec<Log>, bool)> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        let at = match self.phase {
            Phase::Found { at, .. } => at,
            _ => {
                bail!(ErrorKind::InvalidInput(
                    "not able to found a corporation at the moment".to_string(),
                ))
            }
        };
        if !self.board.available_corps().contains(corp) {
            bail!(ErrorKind::InvalidInput(
                format!("{} is already on the board", corp),
            ));
        }
        self.board.extend_corp(&at, corp);
        {
            let corp_shares = self.shares.entry(*corp).or_insert(STARTING_SHARES);
            if *corp_shares > 0 {
                let player_shares = self.players[player].shares.entry(*corp).or_insert(0);
                *player_shares += 1;
                *corp_shares -= 1;
            }
        }
        self.buy_phase(player);
        Ok((
            vec![
                Log::public(vec![
                    N::Player(player),
                    N::text(" founded "),
                    corp.render(),
                ]),
            ],
            match self.phase {
                Phase::Buy { .. } => true,
                _ => false,
            },
        ))
    }

    pub fn buy(&mut self, player: usize, n: usize, corp: Corp) -> Result<(Vec<Log>, bool)> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        if n == 0 {
            bail!(ErrorKind::InvalidInput("can't buy 0 shares".to_string()));
        }
        match self.phase {
            Phase::Buy { remaining, .. } => {
                if n > remaining {
                    bail!(ErrorKind::InvalidInput(
                        format!("can only buy {} more", remaining),
                    ));
                }
                let corp_size = self.board.corp_size(&corp);
                if corp_size == 0 {
                    bail!(ErrorKind::InvalidInput(
                        format!("{} is not on the board", corp),
                    ));
                }
                let corp_shares = self.shares.get(&corp).cloned().unwrap_or(0);
                if n > corp_shares {
                    bail!(ErrorKind::InvalidInput(
                        format!("{} has {} left", corp, corp_shares),
                    ));
                }
                let price = corp.value(corp_size) * n;
                let player_money = self.players[player].money;
                if price > player_money {
                    bail!(ErrorKind::InvalidInput(
                        format!("costs ${}, you only have ${}", price, player_money),
                    ));
                }
                self.players[player].money -= price;
                {
                    let player_shares = self.players[player].shares.entry(corp).or_insert(0);
                    *player_shares += n;
                    let corp_shares = self.shares.entry(corp).or_insert(STARTING_SHARES);
                    *corp_shares -= n;
                }
                let new_remaining = remaining - n;
                let mut logs: Vec<Log> = vec![
                    Log::public(vec![
                        N::Player(player),
                        N::text(" bought "),
                        N::Bold(vec![N::text(format!("{} ", n))]),
                        corp.render(),
                        N::text(" for "),
                        N::Bold(vec![N::text(format!("${}", price))]),
                    ]),
                ];

                if new_remaining == 0 {
                    logs.extend(self.end_turn());
                    return Ok((logs, false));
                }
                self.phase = Phase::Buy {
                    player,
                    remaining: new_remaining,
                };
                Ok((logs, true))
            }
            _ => {
                bail!(ErrorKind::InvalidInput(
                    "can't buy shares at the moment".to_string(),
                ))
            }
        }
    }

    pub fn done(&mut self, player: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        match self.phase {
            Phase::Buy { .. } => Ok(self.end_turn()),
            _ => {
                bail!(ErrorKind::InvalidInput(
                    "can't end your turn at the moment".to_string(),
                ))
            }
        }
    }

    fn end_turn(&mut self) -> Vec<Log> {
        let current_player = self.phase.whose_turn();
        self.draw_replacement_tiles(current_player);
        self.phase = Phase::Play(self.next_player(current_player));
        vec![]
    }

    fn next_player(&self, player: usize) -> usize {
        (player + 1) % self.players.len()
    }

    pub fn merge(&mut self, player: usize, from: &Corp, into: &Corp) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        let at = match self.phase {
            Phase::ChooseMerger { at, .. } => at,
            _ => {
                bail!(ErrorKind::InvalidInput(
                    "can't choose a merger at the moment".to_string(),
                ))
            }
        };
        if from == into {
            bail!(ErrorKind::InvalidInput(
                "can't merge the same corp into itself".to_string(),
            ));
        }
        let (from_candidates, into_candidates) = self.board.merge_candidates(&at);
        if from_candidates.is_empty() || into_candidates.is_empty() {
            bail!(ErrorKind::Internal(
                "merge was called with an empty from or into candidates"
                    .to_string(),
            ));
        }
        if !from_candidates.contains(from) {
            bail!(ErrorKind::InvalidInput(
                format!("{} is not a valid corporation to be merged", from),
            ));
        }
        if !into_candidates.contains(into) {
            bail!(ErrorKind::InvalidInput(
                format!("{} is not a valid corporation to merge into", into),
            ));
        }
        if self.board.get_tile(at) == Tile::Unincorporated {
            self.board.set_tile(at, Tile::Corp(*into));
        }
        let mut logs = vec![
            Log::public(vec![
                from.render(),
                N::text(" is merging into "),
                into.render(),
            ]),
        ];
        logs.extend(self.pay_bonuses(from));
        self.phase = Phase::SellOrTrade {
            player,
            corp: *from,
            at,
            turn_player: player,
        };
        if self.players[player].shares.get(from).cloned().unwrap_or(0) == 0 {
            // The player has none of the shares anyway, just skip them.
            self.next_player_sell_trade();
        }
        unimplemented!();
    }

    fn pay_bonuses(&mut self, corp: &Corp) -> Vec<Log> {
        unimplemented!();
    }

    fn next_player_sell_trade(&mut self) {
        unimplemented!();
    }

    pub fn sell(&mut self, player: usize, _n: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        unimplemented!();
    }

    pub fn trade(&mut self, player: usize, _n: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        unimplemented!();
    }

    pub fn keep(&mut self, player: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        unimplemented!();
    }

    pub fn end(&mut self, player: usize) -> Result<Vec<Log>> {
        self.assert_not_finished()?;
        self.assert_player_turn(player)?;
        unimplemented!();
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
            players: self.players.iter().map(|v| v.to_owned().into()).collect(),
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
