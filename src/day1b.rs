/// Very naive solution for day 1b
use core::ffi::CStr;

use crate::prelude::*;

advent_of_code_impl!(run);

fn run(path: &CStr) -> Result<(), MysteryStr<'static>> {
    let mmap = open_mmap(path)?;
    let bytes = mmap.as_slice();

    let mut lineno = 1;
    let mut acc: i64 = 50;
    let mut times_zero: u64 = 0;

    for line in bytes.split(|c| *c == b'\n') {
        if line.iter().all(|c| c.is_ascii_whitespace()) {
            continue;
        }
        let Some(mut action) = parse_line(line) else {
            return Err(sprintf_leak!("Error in line {lineno} (file {path:?})").into());
        };

        while action != 0 {
            acc += action.signum();
            if acc == 0 || acc == 100 || acc == -100 {
                times_zero += 1;
                acc = 0;
            }
            action -= action.signum();
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
        _ => return None,
    };
    let Ok(digits) = str::from_utf8(&src[1..]) else { return None };
    let value: i64 = digits.parse().ok()?;
    Some(sign * value)
}
