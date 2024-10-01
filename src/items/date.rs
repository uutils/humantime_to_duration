// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

//! Parse a date item (without time component)
//!
//! The GNU docs say:
//!
//! > A calendar date item specifies a day of the year. It is specified
//! > differently, depending on whether the month is specified numerically
//! > or literally.
//! >
//! > ...
//! >
//! > For numeric months, the ISO 8601 format ‘year-month-day’ is allowed,
//! > where year is any positive number, month is a number between 01 and
//! > 12, and day is a number between 01 and 31. A leading zero must be
//! > present if a number is less than ten. If year is 68 or smaller, then
//! > 2000 is added to it; otherwise, if year is less than 100, then 1900
//! > is added to it. The construct ‘month/day/year’, popular in the United
//! > States, is accepted. Also ‘month/day’, omitting the year.
//! >
//! > Literal months may be spelled out in full: ‘January’, ‘February’,
//! > ‘March’, ‘April’, ‘May’, ‘June’, ‘July’, ‘August’, ‘September’,
//! > ‘October’, ‘November’ or ‘December’. Literal months may be
//! > abbreviated to their first three letters, possibly followed by an
//! > abbreviating dot. It is also permitted to write ‘Sept’ instead of
//! > ‘September’.

use winnow::{
    ascii::{alpha1, dec_uint},
    combinator::{alt, opt, preceded},
    seq,
    stream::AsChar,
    token::take_while,
    trace::trace,
    PResult, Parser,
};

use super::s;
use crate::ParseDateTimeError;

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct Date {
    pub day: u32,
    pub month: u32,
    pub year: Option<u32>,
}

pub fn parse(input: &mut &str) -> PResult<Date> {
    alt((iso, us, literal1, literal2)).parse_next(input)
}

/// Parse `YYYY-MM-DD` or `YY-MM-DD`
///
/// This is also used by [`combined`](super::combined).
pub fn iso(input: &mut &str) -> PResult<Date> {
    seq!(Date {
        year: year.map(Some),
        _: s('-'),
        month: month,
        _: s('-'),
        day: day,
    })
    .parse_next(input)
}

/// Parse `MM/DD/YYYY`, `MM/DD/YY` or `MM/DD`
fn us(input: &mut &str) -> PResult<Date> {
    seq!(Date {
        month: month,
        _: s('/'),
        day: day,
        year: opt(preceded(s('/'), year)),
    })
    .parse_next(input)
}

/// Parse `14 November 2022`, `14 Nov 2022`, "14nov2022", "14-nov-2022", "14-nov2022", "14nov-2022"
fn literal1(input: &mut &str) -> PResult<Date> {
    seq!(Date {
        day: day,
        _: opt(s('-')),
        month: literal_month,
        year: opt(preceded(opt(s('-')), year)),
    })
    .parse_next(input)
}

/// Parse `November 14, 2022` and `Nov 14, 2022`
fn literal2(input: &mut &str) -> PResult<Date> {
    seq!(Date {
        month: literal_month,
        day: day,
        // FIXME: GNU requires _some_ space between the day and the year,
        // probably to distinguish with floats.
        year: opt(preceded(s(","), year)),
    })
    .parse_next(input)
}

pub fn year(input: &mut &str) -> PResult<u32> {
    // 2147485547 is the maximum value accepted
    // by GNU, but chrono only behave like GNU
    // for years in the range: [0, 9999], so we
    // keep in the range [0, 9999]
    trace(
        "year",
        s(
            take_while(1..=4, AsChar::is_dec_digit).map(|number_str: &str| {
                let year = number_str.parse::<u32>().unwrap();
                if number_str.len() == 2 {
                    if year <= 68 {
                        year + 2000
                    } else {
                        year + 1900
                    }
                } else {
                    year
                }
            }),
        ),
    )
    .parse_next(input)
}

fn month(input: &mut &str) -> PResult<u32> {
    s(dec_uint)
        .try_map(|x| {
            (1..=12)
                .contains(&x)
                .then_some(x)
                .ok_or(ParseDateTimeError::InvalidInput)
        })
        .parse_next(input)
}

fn day(input: &mut &str) -> PResult<u32> {
    s(dec_uint)
        .try_map(|x| {
            (1..=31)
                .contains(&x)
                .then_some(x)
                .ok_or(ParseDateTimeError::InvalidInput)
        })
        .parse_next(input)
}

/// Parse the name of a month (case-insensitive)
fn literal_month(input: &mut &str) -> PResult<u32> {
    s(alpha1)
        .verify_map(|s: &str| {
            Some(match s {
                "january" | "jan" => 1,
                "february" | "feb" => 2,
                "march" | "mar" => 3,
                "april" | "apr" => 4,
                "may" => 5,
                "june" | "jun" => 6,
                "july" | "jul" => 7,
                "august" | "aug" => 8,
                "september" | "sep" | "sept" => 9,
                "october" | "oct" => 10,
                "november" | "nov" => 11,
                "december" | "dec" => 12,
                _ => return None,
            })
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::{parse, Date};

    // Test cases from the GNU docs:
    //
    // ```
    // 2022-11-14     # ISO 8601.
    // 22-11-14       # Assume 19xx for 69 through 99,
    //                # 20xx for 00 through 68 (not recommended).
    // 11/14/2022     # Common U.S. writing.
    // 14 November 2022
    // 14 Nov 2022    # Three-letter abbreviations always allowed.
    // November 14, 2022
    // 14-nov-2022
    // 14nov2022
    // ```

    #[test]
    fn with_year() {
        let reference = Date {
            year: Some(2022),
            month: 11,
            day: 14,
        };

        for mut s in [
            "2022-11-14",
            "2022    -  11  -   14",
            "22-11-14",
            "2022---11----14",
            "22(comment 1)-11(comment 2)-14",
            "11/14/2022",
            "11--/14--/2022",
            "11(comment 1)/(comment 2)14(comment 3)/(comment 4)2022",
            "11   /  14   /      2022",
            "11/14/22",
            "14 november 2022",
            "14 nov 2022",
            "november 14, 2022",
            "november 14     ,     2022",
            "nov 14, 2022",
            "14-nov-2022",
            "14nov2022",
            "14nov      2022",
        ] {
            let old_s = s.to_owned();
            assert_eq!(parse(&mut s).unwrap(), reference, "Format string: {old_s}");
        }
    }

    #[test]
    fn no_year() {
        let reference = Date {
            year: None,
            month: 11,
            day: 14,
        };
        for mut s in [
            "11/14",
            "14 november",
            "14 nov",
            "14(comment!)nov",
            "november 14",
            "november(comment!)14",
            "nov 14",
            "14-nov",
            "14nov",
            "14(comment????)nov",
        ] {
            assert_eq!(parse(&mut s).unwrap(), reference);
        }
    }

    #[test]
    fn test_year() {
        use super::year;

        // the minimun input length is 2
        // assert!(year(&mut "0").is_err());
        // -> GNU accepts year 0
        // test $(date -d '1-1-1' '+%Y') -eq '0001'

        // test $(date -d '68-1-1' '+%Y') -eq '2068'
        // 2-characters are converted to 19XX/20XX
        assert_eq!(year(&mut "10").unwrap(), 2010u32);
        assert_eq!(year(&mut "68").unwrap(), 2068u32);
        assert_eq!(year(&mut "69").unwrap(), 1969u32);
        assert_eq!(year(&mut "99").unwrap(), 1999u32);
        // 3,4-characters are converted verbatim
        assert_eq!(year(&mut "468").unwrap(), 468u32);
        assert_eq!(year(&mut "469").unwrap(), 469u32);
        assert_eq!(year(&mut "1568").unwrap(), 1568u32);
        assert_eq!(year(&mut "1569").unwrap(), 1569u32);
        // consumes at most 4 characters from the input
        //assert_eq!(year(&mut "1234567").unwrap(), 1234u32);
    }
}
