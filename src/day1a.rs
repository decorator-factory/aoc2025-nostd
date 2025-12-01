use core::ffi::CStr;

use crate::prelude::*;

pub fn run(path: &CStr) -> Result<(), MysteryStr<'static>> {
    let mut lines = open_lines(path)?;

    let mut lineno = 1;
    let mut acc: i64 = 50;
    let mut times_zero: u64 = 0;

    while let Some(line) = lines.next() {
        if line.iter().all(|c| c.is_ascii_whitespace()) {
            continue;
        }

        printf!("Got line: {line:?}\n");
        let Some(action) = parse_line(line) else {
            return Err(sprintf_leak!("Error in line {lineno} (file {path:?})").into());
        };

        acc += action;
        acc %= 100;

        if acc == 0 {
            times_zero += 1;
        }
        lineno += 1;
    }
    printf!("The password is: {times_zero}\n");
    Ok(())
}

fn parse_line(src: &[u8]) -> Option<i64> {
    let sign = match src.first()? {
        b'L' => -1,
        b'R' => 1,
        _ => {
            return None;
        }
    };
    let Ok(digits) = str::from_utf8(&src[1..]) else { return None };
    let value: i64 = digits.parse().ok()?;
    Some(sign * value)
}
