// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Parse an ISO 8601 date and time item
//!
//! The GNU docs state:
//!
//! > The ISO 8601 date and time of day extended format consists of an ISO 8601
//! > date, a ‘T’ character separator, and an ISO 8601 time of day. This format
//! > is also recognized if the ‘T’ is replaced by a space.
//! >
//! > In this format, the time of day should use 24-hour notation. Fractional
//! > seconds are allowed, with either comma or period preceding the fraction.
//! > ISO 8601 fractional minutes and hours are not supported. Typically, hosts
//! > support nanosecond timestamp resolution; excess precision is silently discarded.
#![allow(deprecated)]

use winnow::ascii::dec_uint;
use winnow::token::take;
use winnow::{combinator::alt, seq, trace::trace, PResult, Parser};

use crate::items::combined;
use crate::items::space;

use super::{
    date::{self, Date},
    s,
    time::{self, Time},
};

#[derive(PartialEq, Debug, Clone, Default)]
pub struct DateTime {
    pub(crate) date: Date,
    pub(crate) time: Time,
}

pub fn parse(input: &mut &str) -> PResult<DateTime> {
    alt((parse_basic, parse_8digits)).parse_next(input)
}

fn parse_basic(input: &mut &str) -> PResult<DateTime> {
    seq!(DateTime {
        date: trace("date iso", date::iso),
        // Note: the `T` is lowercased by the main parse function
        _: alt((s('t').void(), (' ', space).void())),
        time: trace("time iso", time::iso),
    })
    .parse_next(input)
}

fn parse_8digits(input: &mut &str) -> PResult<DateTime> {
    s((
        take(2usize).and_then(dec_uint),
        take(2usize).and_then(dec_uint),
        take(2usize).and_then(dec_uint),
        take(2usize).and_then(dec_uint),
    ))
    .map(
        |(hour, minute, day, month): (u32, u32, u32, u32)| combined::DateTime {
            date: date::Date {
                day,
                month,
                year: None,
            },
            time: time::Time {
                hour,
                minute,
                second: 0.0,
                offset: None,
            },
        },
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::{parse, DateTime};
    use crate::items::{date::Date, time::Time};

    #[test]
    fn some_date() {
        let reference = Some(DateTime {
            date: Date {
                day: 10,
                month: 10,
                year: Some(2022),
            },
            time: Time {
                hour: 10,
                minute: 10,
                second: 55.0,
                offset: None,
            },
        });

        for mut s in [
            "2022-10-10t10:10:55",
            "2022-10-10 10:10:55",
            "2022-10-10    t   10:10:55",
            "2022-10-10       10:10:55",
            "2022-10-10  (A comment!) t   10:10:55",
            "2022-10-10  (A comment!)   10:10:55",
        ] {
            let old_s = s.to_owned();
            assert_eq!(parse(&mut s).ok(), reference, "Failed string: {old_s}")
        }
    }
}
