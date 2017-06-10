use brdgme_game::command::parser::*;
use brdgme_game::Gamer;

use board::Loc;
use corp::{Corp, CORPS};
use Game;
use Phase;

use std::usize;

pub enum Command {
    Play(Loc),
    Buy(usize, Corp),
    Done,
    Merge(Corp, Corp),
    Sell(usize),
    Trade(usize),
    Keep,
    End,
}

impl Game {
    pub fn command_parser(&self, player: usize) -> Option<Box<Parser<Command>>> {
        if self.is_finished() {
            return None;
        }
        let mut parsers: Vec<Box<Parser<Command>>> = vec![];
        if self.phase.whose_turn() == player {
            match self.phase {
                Phase::Play(_) => {
                    parsers.push(Box::new(self.play_parser(player)));
                }
                Phase::Buy { remaining, .. } => {
                    parsers.push(Box::new(self.buy_parser(player, remaining)));
                    parsers.push(Box::new(buy_done_parser()));
                }
                Phase::ChooseMerger(..) => {
                    parsers.push(Box::new(self.merge_parser(&CORPS)));
                }
                Phase::SellOrTrade { player, corp, .. } => {
                    parsers.push(Box::new(self.sell_parser(player, corp)));
                    parsers.push(Box::new(self.trade_parser(player, corp)));
                    parsers.push(Box::new(sell_done_parser()));
                }
            }
            if self.can_end() {
                parsers.push(Box::new(end_parser()));
            }
        }
        if parsers.is_empty() {
            None
        } else {
            Some(Box::new(OneOf::new(parsers)))
        }
    }

    fn play_parser(&self, player: usize) -> impl Parser<Command> {
        Map::new(Chain2::new(Doc::name_desc("play",
                                            "play a tile to the board",
                                            Token::new("play")),
                             AfterSpace::new(Enum::exact(self.players
                                                             .get(&player)
                                                             .map(|p| p.tiles.clone())
                                                             .unwrap_or_else(|| vec![])))),
                 |(_, loc)| Command::Play(loc))
    }

    fn buy_parser(&self, _player: usize, remaining: usize) -> impl Parser<Command> {
        Map::new(Chain3::new(Doc::name_desc("buy", "buy shares", Token::new("buy")),
                             AfterSpace::new(Doc::name_desc("#",
                                                            "number of shares to buy",
                                                            Int::bounded(1, remaining as i32))),
                             AfterSpace::new(Doc::name_desc("(corp)",
                                                            "the corporation to buy shares in",
                                                            Enum::partial(CORPS.to_vec())))),
                 |(_, n, corp)| Command::Buy(n as usize, corp))
    }

    fn sell_parser(&self, player: usize, corp: Corp) -> impl Parser<Command> {
        Map::new(Chain2::new(Doc::name_desc("sell", "sell shares", Token::new("sell")),
                             AfterSpace::new(Doc::name_desc("#",
                                                            "number of shares to sell",
                                                            self.player_shares_parser(player,
                                                                                      corp)))),
                 |(_, n)| Command::Sell(n as usize))
    }

    fn trade_parser(&self, player: usize, corp: Corp) -> impl Parser<Command> {
        Map::new(Chain2::new(Doc::name_desc("trade",
                                            "trade shares, two-for-one",
                                            Token::new("trade")),
                             AfterSpace::new(Doc::name_desc("#",
                                                            "number of shares to trade, two-for-one",
                                                            self.player_shares_parser(player,
                                                                                      corp)))),
                 |(_, n)| Command::Sell(n as usize))
    }

    fn player_shares_parser(&self, player: usize, corp: Corp) -> impl Parser<i32> {
        Int::bounded(1,
                     self.players
                         .get(&player)
                         .and_then(|p| p.shares.get(&corp).cloned())
                         .unwrap_or(1) as i32)
    }

    fn merge_parser(&self, corps: &[Corp]) -> impl Parser<Command> {
        Map::new(Chain4::new(Doc::name_desc("merge",
                                            "choose which corporation to merge into another",
                                            Token::new("merge")),
                             AfterSpace::new(Doc::name_desc("(corp)",
                                                            "the corporation to merge into another",
                                                            Enum::partial(corps.to_owned()))),
                             Token::new("into"),
                             AfterSpace::new(Doc::name_desc("(corp)",
                                                            "the corporation to be merged into",
                                                            Enum::partial(corps.to_owned())))),
                 |(_, from, _, into)| Command::Merge(from, into))
    }
}

fn end_parser() -> impl Parser<Command> {
    Doc::name_desc("end",
                   "trigger the end of the game at the end of your turn",
                   Map::new(Token::new("end"), |_| Command::End))
}

fn buy_done_parser() -> impl Parser<Command> {
    Doc::name_desc("done",
                   "finish buying shares and end your turn",
                   Map::new(Token::new("done"), |_| Command::Done))
}

fn sell_done_parser() -> impl Parser<Command> {
    Doc::name_desc("done",
                   "finish selling and trading shares",
                   Map::new(Token::new("done"), |_| Command::Done))
}