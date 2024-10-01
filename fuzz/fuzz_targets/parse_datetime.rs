#![no_main]
#![allow(dead_code)]

use std::fmt::{Debug, Display};

use libfuzzer_sys::arbitrary::{self, Arbitrary};

#[macro_use]
extern crate libfuzzer_sys;

#[derive(Debug)]
struct Format(&'static str);

// These are formats to test the compatibility with GNU
const FORMATS: &[&str] = &["%G-%m-%d %H:%M:%S", "%b %d %Y %H:%M:%S"];

impl<'a> Arbitrary<'a> for Format {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Format(u.choose(FORMATS)?))
    }
}

struct Input {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    format: Format,
}

impl<'a> Arbitrary<'a> for Input {
    fn arbitrary(
        u: &mut libfuzzer_sys::arbitrary::Unstructured<'a>,
    ) -> libfuzzer_sys::arbitrary::Result<Self> {
        let year = u.arbitrary::<u32>()?;
        let month = u.arbitrary::<u32>()?;
        let day = u.arbitrary::<u32>()?;
        let hour = u.arbitrary::<u32>()?;
        let minute = u.arbitrary::<u32>()?;
        let second = u.arbitrary::<u32>()?;
        let format = u.arbitrary::<Format>()?;

        // GNU max    2147485547
        // chrono max 262143
        // chrono outputs + before the year if it is >9999
        if !(1..=9999).contains(&year)
            || !(1..=12).contains(&month)
            || !(1..=31).contains(&day)
            || !(0..24).contains(&hour)
            || !(0..60).contains(&minute)
            || !(0..60).contains(&second)
        {
            return Err(crate::arbitrary::Error::IncorrectFormat);
        }

        Ok(Input {
            year,
            month,
            day,
            hour,
            minute,
            second,
            format,
        })
    }
}

impl Input {
    fn format(&self) -> String {
        let Input {
            year,
            month,
            day,
            hour,
            minute,
            second,
            format,
        } = self;
        let as_string = format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}");
        std::process::Command::new("date")
            .arg("-d")
            .arg(as_string)
            .arg(format!("+{}", &format.0))
            .output()
            .map(|mut output| {
                output.stdout.pop(); // remove trailing \n
                String::from_utf8(output.stdout).expect("from_utf8")
            })
            .expect("gnu date")
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

fuzz_target!(|input: Input| {
    let fmt = input.format.0;
    let gnu = std::process::Command::new("date")
        .arg("-d")
        .arg(input.format())
        .arg(format!("+{fmt}"))
        .output()
        .map(|mut output| {
            output.stdout.pop(); // remove trailing \n
            String::from_utf8(output.stdout).expect("from_utf8")
        });
    let us = parse_datetime::parse_datetime(&input.format()).map(|d| d.format(fmt).to_string());

    match (us, gnu) {
        (Ok(us), Ok(gnu)) => assert_eq!(
            us, gnu,
            "\n\nGNU Incompatibility found for the input: {input}\nExpected: {gnu}\nFound:    {us}\n\n"
        ),
        (Err(_), Err(_)) => (),
        (Ok(us), Err(e)) => {
            panic!("Expecting to fail, but succeeded for input `{input}`, gnu error: {e}, parsed date: {us}")
        }
        (Err(_), Ok(gnu)) => {
            panic!("Expecting to succeed, but failed for input `{input}`, gnu output: {gnu}")
        }
    };
});
