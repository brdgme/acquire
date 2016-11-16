use combine::{Parser, many1, parser, try, choice};
use combine::char::{digit, spaces, string_cmp};
use combine::combinator::{satisfy_map, FnParser};
use combine::primitives::{Stream, ParseResult};

use brdgme_game::parser::{cmp_ignore_case, arg, match_first};
use brdgme_game::GameError;

use board::Loc;
use corp::Corp;

use std::usize;
use std::ascii::AsciiExt;

pub enum Command {
    Play(Loc),
    Buy(usize, Corp),
    Done,
    Merge(Corp, Corp),
    Sell(usize),
    Trade(usize),
    Keep,
}

type FnP<T, I> = FnParser<I, fn(I) -> ParseResult<T, I>>;

pub fn command<I>() -> FnP<Command, I>
    where I: Stream<Item = char>
{
    fn command_<I>(input: I) -> ParseResult<Command, I>
        where I: Stream<Item = char>
    {
        choice([play, buy, done]).parse_stream(input)
    }
    parser(command_)
}

pub fn play<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string_cmp("play", cmp_ignore_case)), spaces(), parser(loc))
        .map(|(_, _, l)| Command::Play(l))
        .parse_stream(input)
}

pub fn done<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    try(string_cmp("done", cmp_ignore_case)).map(|_| Command::Done).parse_stream(input)
}

pub fn buy<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string_cmp("buy", cmp_ignore_case)),
     spaces(),
     many1(digit())
            .and_then(|s: String| s.parse::<usize>())
            .and_then(|d: usize| if d > 0 {
                Ok(d)
            } else {
                Err(GameError::InvalidInput("amount must be 1 or higher".to_string()))
            }),
     spaces(),
     parser(corp))
        .map(|(_, _, n, _, c)| Command::Buy(n, c))
        .parse_stream(input)
}

pub fn corp<I>(input: I) -> ParseResult<Corp, I>
    where I: Stream<Item = char>
{
    arg()
        .and_then(|a| {
            match_first(a,
                        Corp::iter()
                            .map(|c| (c.name(), c))
                            .collect::<Vec<(String, &Corp)>>()
                            .iter())
                .map(|c| **c)
        })
        .parse_stream(input)
}

pub fn loc<I>(input: I) -> ParseResult<Loc, I>
    where I: Stream<Item = char>
{
    (parser(loc_row), parser(loc_col)).map(|(r, c)| Loc { row: r, col: c }).parse_stream(input)
}

pub fn loc_row<I>(input: I) -> ParseResult<usize, I>
    where I: Stream<Item = char>
{
    let a_index = 'a' as usize;
    satisfy_map(|c: char| {
            if c.is_ascii() {
                let lc = c.to_ascii_lowercase() as usize;
                if lc >= a_index {
                    return Some(lc - a_index);
                }
            }
            None
        })
        .parse_stream(input)
}

pub fn loc_col<I>(input: I) -> ParseResult<usize, I>
    where I: Stream<Item = char>
{
    many1(digit())
        .and_then(|s: String| s.parse::<usize>())
        .and_then(|d: usize| if d > 0 {
            Ok(d - 1)
        } else {
            Err(GameError::InvalidInput("column must be 1 or higher".to_string()))
        })
        .parse_stream(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use board::Loc;
    use corp::Corp;
    use combine::{parser, Parser};

    #[test]
    fn loc_row_works() {
        assert_eq!(Ok(0), parser(loc_row).parse("a").map(|x| x.0));
        assert_eq!(Ok(0), parser(loc_row).parse("A").map(|x| x.0));
        assert_eq!(Ok(8), parser(loc_row).parse("i").map(|x| x.0));
    }

    #[test]
    fn loc_col_works() {
        assert_eq!(Ok(0), parser(loc_col).parse("1").map(|x| x.0));
        assert_eq!(Ok(11), parser(loc_col).parse("12").map(|x| x.0));
    }

    #[test]
    fn loc_works() {
        assert_eq!(Ok(Loc { row: 0, col: 0 }),
                   parser(loc).parse("a1").map(|x| x.0));
        assert_eq!(Ok(Loc { row: 8, col: 11 }),
                   parser(loc).parse("I12").map(|x| x.0));
    }

    #[test]
    fn corp_works() {
        assert_eq!(Ok((Corp::American, "")), parser(corp).parse("amer"));
        assert_eq!(Ok((Corp::Sackson, " blash")),
                   parser(corp).parse("sacks blash"));
    }
}
