// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use super::s;
use winnow::{
    ascii::{alpha1, dec_uint},
    combinator::{alt, opt},
    PResult, Parser,
};

pub fn ordinal(input: &mut &str) -> PResult<i32> {
    alt((text_ordinal, number_ordinal)).parse_next(input)
}

fn number_ordinal(input: &mut &str) -> PResult<i32> {
    let sign = opt(alt(('+'.value(1), '-'.value(-1)))).map(|s| s.unwrap_or(1));
    (s(sign), s(dec_uint))
        .verify_map(|(s, u): (i32, u32)| {
            let i: i32 = u.try_into().ok()?;
            Some(s * i)
        })
        .parse_next(input)
}

fn text_ordinal(input: &mut &str) -> PResult<i32> {
    s(alpha1)
        .verify_map(|s: &str| {
            Some(match s {
                "last" => -1,
                "this" => 0,
                "next" | "first" => 1,
                "third" => 3,
                "fourth" => 4,
                "fifth" => 5,
                "sixth" => 6,
                "seventh" => 7,
                "eight" => 8,
                "ninth" => 9,
                "tenth" => 10,
                "eleventh" => 11,
                "twelfth" => 12,
                _ => return None,
            })
        })
        .parse_next(input)
}
