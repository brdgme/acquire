use combine::{Parser, many1, parser, try};
use combine::char::{digit, spaces, string_cmp};
use combine::combinator::satisfy_map;
use combine::primitives::{Stream, ParseResult};

use brdgme_game::parser::cmp_ignore_case;
use brdgme_game::GameError;

use board::Loc;
use corp::Corp;

use std::usize;
use std::ascii::AsciiExt;

pub enum Command {
    Play(Loc),
    Buy(usize, Corp),
    Merge(Corp, Corp),
    Sell(usize),
    Trade(usize),
    Done,
}

pub fn play<I>(input: I) -> ParseResult<Command, I>
    where I: Stream<Item = char>
{
    (try(string_cmp("play", cmp_ignore_case)), spaces(), parser(loc))
        .map(|(_, _, l)| Command::Play(l))
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
        assert_eq!(Ok(Loc(0, 0)), parser(loc).parse("a1").map(|x| x.0));
        assert_eq!(Ok(Loc(8, 11)), parser(loc).parse("I12").map(|x| x.0));
    }
}
