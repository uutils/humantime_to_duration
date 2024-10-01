// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// spell-checker:ignore datetime

use std::error::Error;
use std::fmt::{self, Display};

use chrono::{DateTime, FixedOffset, Local};

mod items;

#[derive(Debug, PartialEq)]
pub enum ParseDateTimeError {
    InvalidInput,
}

impl Display for ParseDateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseDateTimeError::InvalidInput => {
                write!(
                    f,
                    "Invalid input string: cannot be parsed as a relative time"
                )
            }
        }
    }
}

impl Error for ParseDateTimeError {}

/// Parses a time string and returns a `DateTime` representing the
/// absolute time of the string.
///
/// # Arguments
///
/// * `s` - A string slice representing the time.
///
/// # Examples
///
/// ```
/// use chrono::{DateTime, Utc, TimeZone};
/// let time = parse_datetime::parse_datetime("2023-06-03 12:00:01Z");
/// assert_eq!(time.unwrap(), Utc.with_ymd_and_hms(2023, 06, 03, 12, 00, 01).unwrap());
/// ```
///
///
/// # Returns
///
/// * `Ok(DateTime<FixedOffset>)` - If the input string can be parsed as a time
/// * `Err(ParseDateTimeError)` - If the input string cannot be parsed as a relative time
///
/// # Errors
///
/// This function will return `Err(ParseDateTimeError::InvalidInput)` if the input string
/// cannot be parsed as a relative time.
pub fn parse_datetime<S: AsRef<str> + Clone>(
    input: S,
) -> Result<DateTime<FixedOffset>, ParseDateTimeError> {
    let input = input.as_ref().to_ascii_lowercase();
    match items::parse(&mut input.as_str()) {
        Ok(x) => items::at_local(x),
        Err(_) => Err(ParseDateTimeError::InvalidInput),
    }
}
/// Parses a time string at a specific date and returns a `DateTime` representing the
/// absolute time of the string.
///
/// # Arguments
///
/// * date - The date represented in local time
/// * `s` - A string slice representing the time.
///
/// # Examples
///
/// ```
/// use chrono::{Duration, Local};
/// use parse_datetime::parse_datetime_at_date;
///
///  let now = Local::now();
///  let after = parse_datetime_at_date(now, "2024-09-13 +3 days");
///
///  assert_eq!(
///    "2024-09-16",
///    after.unwrap().naive_utc().format("%F").to_string()
///  );
/// ```
///
/// # Returns
///
/// * `Ok(DateTime<FixedOffset>)` - If the input string can be parsed as a time
/// * `Err(ParseDateTimeError)` - If the input string cannot be parsed as a relative time
///
/// # Errors
///
/// This function will return `Err(ParseDateTimeError::InvalidInput)` if the input string
/// cannot be parsed as a relative time.
pub fn parse_datetime_at_date<S: AsRef<str> + Clone>(
    date: DateTime<Local>,
    input: S,
) -> Result<DateTime<FixedOffset>, ParseDateTimeError> {
    let input = input.as_ref().to_ascii_lowercase();
    match items::parse(&mut input.as_str()) {
        Ok(x) => items::at_date(x, date.into()),
        Err(_) => Err(ParseDateTimeError::InvalidInput),
    }
}

#[cfg(test)]
mod tests {
    static TEST_TIME: i64 = 1613371067;

    #[cfg(test)]
    mod iso_8601 {
        use std::env;

        use chrono::{TimeZone, Utc};

        use crate::ParseDateTimeError;
        use crate::{parse_datetime, tests::TEST_TIME};

        #[test]
        fn test_t_sep() {
            env::set_var("TZ", "UTC");
            let dt = "2021-02-15T06:37:47";
            let actual = parse_datetime(dt).unwrap();
            assert_eq!(
                actual,
                Utc.timestamp_opt(TEST_TIME, 0).unwrap().fixed_offset()
            );
        }

        #[test]
        fn test_space_sep() {
            env::set_var("TZ", "UTC");
            let dt = "2021-02-15 06:37:47";
            let actual = parse_datetime(dt);
            assert_eq!(actual.unwrap().timestamp(), TEST_TIME);
        }

        #[test]
        fn test_space_sep_offset() {
            env::set_var("TZ", "UTC");
            //       "2021-02-14T22:37:47+00:00"
            let dt = "2021-02-14 22:37:47 -0800";
            let actual = parse_datetime(dt).unwrap();
            let t = Utc.timestamp_opt(TEST_TIME, 0).unwrap().fixed_offset();
            assert_eq!(actual, t);
        }

        #[test]
        fn test_t_sep_offset() {
            env::set_var("TZ", "UTC");
            let dt = "2021-02-14T22:37:47 -0800";
            let actual = parse_datetime(dt);
            assert_eq!(actual.unwrap().timestamp(), TEST_TIME);
        }

        #[test]
        fn invalid_formats() {
            let invalid_dts = vec![
                "NotADate",
                // @TODO
                //"202104",
                //"202104-12T22:37:47"
            ];
            for dt in invalid_dts {
                assert_eq!(parse_datetime(dt), Err(ParseDateTimeError::InvalidInput));
            }
        }

        #[test]
        fn test_epoch_seconds() {
            env::set_var("TZ", "UTC");
            let dt = "@1613371067";
            let actual = parse_datetime(dt);
            assert_eq!(actual.unwrap().timestamp(), TEST_TIME);
        }

        #[test]
        fn test_epoch_seconds_non_utc() {
            env::set_var("TZ", "EST");
            let dt = "@1613371067";
            let actual = parse_datetime(dt);
            assert_eq!(actual.unwrap().timestamp(), TEST_TIME);
        }
    }

    #[cfg(test)]
    mod formats {
        use crate::parse_datetime;
        use chrono::{DateTime, Local, TimeZone};

        #[test]
        fn single_digit_month_day() {
            let x = Local.with_ymd_and_hms(1987, 5, 7, 0, 0, 0).unwrap();
            let expected = DateTime::fixed_offset(&x);

            assert_eq!(Ok(expected), parse_datetime("1987-05-07"));
            assert_eq!(Ok(expected), parse_datetime("1987-5-07"));
            assert_eq!(Ok(expected), parse_datetime("1987-05-7"));
            assert_eq!(Ok(expected), parse_datetime("1987-5-7"));
        }
    }

    #[cfg(test)]
    mod offsets {
        use chrono::Local;

        use crate::parse_datetime;
        use crate::ParseDateTimeError;

        #[test]
        fn test_positive_offsets() {
            let offsets: Vec<&str> = vec![
                "+07:00",
                "UTC+07:00",
                "UTC+0700",
                "UTC+07",
                "Z+07:00",
                "Z+0700",
                "Z+07",
            ];

            let expected = format!("{}{}", Local::now().format("%Y%m%d"), "0000+0700");
            for offset in offsets {
                let actual =
                    parse_datetime(offset).unwrap_or_else(|_| panic!("parse_datetime {offset}"));
                assert_eq!(expected, format!("{}", actual.format("%Y%m%d%H%M%z")));
            }
        }

        #[test]
        fn test_partial_offset() {
            let offsets = vec!["UTC+00:15", "UTC+0015", "Z+00:15", "Z+0015"];
            let expected = format!("{}{}", Local::now().format("%Y%m%d"), "0000+0015");
            for offset in offsets {
                let actual = parse_datetime(offset).unwrap();
                assert_eq!(expected, format!("{}", actual.format("%Y%m%d%H%M%z")));
            }
        }

        #[test]
        fn invalid_offset_format() {
            let invalid_offsets = vec!["UTC+01005"];
            for offset in invalid_offsets {
                assert_eq!(
                    parse_datetime(offset),
                    Err(ParseDateTimeError::InvalidInput)
                );
            }
        }
    }

    #[cfg(test)]
    mod relative_time {
        use crate::parse_datetime;
        #[test]
        fn test_positive_offsets() {
            let relative_times = vec![
                "today",
                "yesterday",
                "1 minute",
                "3 hours",
                "1 year 3 months",
            ];

            for relative_time in relative_times {
                assert!(parse_datetime(relative_time).is_ok());
            }
        }
    }

    #[cfg(test)]
    mod weekday {
        use chrono::{DateTime, Local, TimeZone};

        use crate::parse_datetime_at_date;

        fn get_formatted_date(date: DateTime<Local>, weekday: &str) -> String {
            let result = parse_datetime_at_date(date, weekday).unwrap();

            return result.format("%F %T %f").to_string();
        }

        #[test]
        fn test_weekday() {
            // add some constant hours and minutes and seconds to check its reset
            let date = Local.with_ymd_and_hms(2023, 2, 28, 10, 12, 3).unwrap();

            // 2023-2-28 is tuesday
            assert_eq!(
                get_formatted_date(date, "tuesday"),
                "2023-02-28 00:00:00 000000000"
            );

            // 2023-3-01 is wednesday
            assert_eq!(
                get_formatted_date(date, "wed"),
                "2023-03-01 00:00:00 000000000"
            );

            assert_eq!(
                get_formatted_date(date, "thu"),
                "2023-03-02 00:00:00 000000000"
            );

            assert_eq!(
                get_formatted_date(date, "fri"),
                "2023-03-03 00:00:00 000000000"
            );

            assert_eq!(
                get_formatted_date(date, "sat"),
                "2023-03-04 00:00:00 000000000"
            );

            assert_eq!(
                get_formatted_date(date, "sun"),
                "2023-03-05 00:00:00 000000000"
            );
        }
    }

    #[cfg(test)]
    mod timestamp {
        use crate::parse_datetime;
        use chrono::{TimeZone, Utc};

        #[test]
        fn test_positive_and_negative_offsets() {
            let offsets: Vec<i64> = vec![
                0, 1, 2, 10, 100, 150, 2000, 1234400000, 1334400000, 1692582913, 2092582910,
            ];

            for offset in offsets {
                // positive offset
                let time = Utc.timestamp_opt(offset, 0).unwrap();
                let dt = parse_datetime(format!("@{}", offset));
                assert_eq!(dt.unwrap(), time);

                // negative offset
                let time = Utc.timestamp_opt(-offset, 0).unwrap();
                let dt = parse_datetime(format!("@-{}", offset));
                assert_eq!(dt.unwrap(), time);
            }
        }
    }

    #[cfg(test)]
    mod timeonly {
        use crate::parse_datetime_at_date;
        use chrono::{Local, TimeZone};
        use std::env;

        #[test]
        fn test_time_only() {
            env::set_var("TZ", "UTC");
            let test_date = Local.with_ymd_and_hms(2024, 3, 3, 0, 0, 0).unwrap();
            let parsed_time = parse_datetime_at_date(test_date, "9:04:30 PM +0530").unwrap();
            // convert the timezone to an offset
            let offset = 5 * 3600 + 30 * 60;
            let tz = chrono::FixedOffset::east_opt(offset).unwrap();
            let t = chrono::Utc
                .timestamp_opt(1709480070, 0)
                .unwrap()
                .with_timezone(&tz)
                .fixed_offset();
            assert_eq!(parsed_time, t)
        }
    }
    /// Used to test example code presented in the README.
    mod readme_test {
        use crate::parse_datetime;
        use chrono::{Local, TimeZone};

        #[test]
        fn test_readme_code() {
            let dt = parse_datetime("2021-02-14 06:37:47");
            assert_eq!(
                dt.unwrap(),
                Local.with_ymd_and_hms(2021, 2, 14, 6, 37, 47).unwrap()
            );
        }
    }

    mod invalid_test {
        use crate::parse_datetime;
        use crate::ParseDateTimeError;

        #[test]
        fn test_invalid_input() {
            let result = parse_datetime("foobar");
            assert_eq!(result, Err(ParseDateTimeError::InvalidInput));

            let result = parse_datetime("invalid 1");
            assert_eq!(result, Err(ParseDateTimeError::InvalidInput));
        }
    }

    mod test_relative {

        use crate::parse_datetime;
        use std::env;

        #[test]
        fn test_month() {
            env::set_var("TZ", "UTC");

            assert_eq!(
                parse_datetime("28 feb + 1 month")
                    .expect("parse_datetime")
                    .format("%+")
                    .to_string(),
                "2024-03-28T00:00:00+00:00"
            );

            // 29 feb 2025 is invalid
            assert!(parse_datetime("29 feb + 1 year").is_err());

            // 29 feb 2025 is an invalid date
            assert!(parse_datetime("29 feb 2025").is_err());

            // because 29 feb 2025 is invalid, 29 feb 2025 + 1 day is invalid
            // arithmetic does not operate on invalid dates
            assert!(parse_datetime("29 feb 2025 + 1 day").is_err());

            // 28 feb 2023 + 1 day = 1 mar
            assert_eq!(
                parse_datetime("28 feb 2023 + 1 day")
                    .unwrap()
                    .format("%+")
                    .to_string(),
                "2023-03-01T00:00:00+00:00"
            );
        }

        #[test]
        fn month_overflow() {
            env::set_var("TZ", "UTC");
            assert_eq!(
                parse_datetime("2024-01-31 + 1 month")
                    .unwrap()
                    .format("%+")
                    .to_string(),
                "2024-03-02T00:00:00+00:00",
            );

            assert_eq!(
                parse_datetime("2024-02-29 + 1 month")
                    .unwrap()
                    .format("%+")
                    .to_string(),
                "2024-03-29T00:00:00+00:00",
            );
        }
    }

    mod test_gnu {
        use crate::parse_datetime;

        fn make_gnu_date(input: &str, fmt: &str) -> String {
            std::process::Command::new("date")
                .arg("-d")
                .arg(input)
                .arg(format!("+{fmt}"))
                .output()
                .map(|mut output| {
                    //io::stdout().write_all(&output.stdout).unwrap();
                    output.stdout.pop(); // remove trailing \n
                    String::from_utf8(output.stdout).expect("from_utf8")
                })
                .unwrap()
        }

        fn has_gnu_date() -> bool {
            std::process::Command::new("date")
                .arg("--version")
                .output()
                .map(|output| String::from_utf8(output.stdout).unwrap())
                .map(|output| output.starts_with("date (GNU coreutils)"))
                .unwrap_or(false)
        }

        #[test]
        fn gnu_compat() {
            // skip if GNU date is not present
            if !has_gnu_date() {
                eprintln!("GNU date not found, skipping gnu_compat tests");
                return;
            }

            const FMT: &str = "%Y-%m-%d %H:%M:%S";
            let input = "0000-03-02 00:00:00";
            assert_eq!(
                make_gnu_date(input, FMT),
                parse_datetime(input).unwrap().format(FMT).to_string()
            );

            let input = "2621-03-10 00:00:00";
            assert_eq!(
                make_gnu_date(input, FMT),
                parse_datetime(input)
                    .expect("parse_datetime")
                    .format(FMT)
                    .to_string()
            );

            let input = "1038-03-10 00:00:00";
            assert_eq!(
                make_gnu_date(input, FMT),
                parse_datetime(input)
                    .expect("parse_datetime")
                    .format(FMT)
                    .to_string()
            );
        }
    }
}
