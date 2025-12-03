// Naive solution for day 3a

use core::ffi::CStr;

use crate::prelude::*;

advent_of_code_impl!(aoc);

fn aoc(path: &CStr) -> Result<(), MysteryStr<'static>> {
    let mmap = open_mmap(path)?;
    let bytes = mmap.as_slice();

    let start = get_nanos();
    let answer = run(bytes);
    let end = get_nanos();
    printf!("Elapsed time: {:.2}us\n", (end as f64 - start as f64) / 1000.0);

    printf!("Total joltage is: {answer}\n");
    Ok(())
}

// ferris_elf compatible
pub fn run(bytes: &[u8]) -> u64 {
    let mut total = 0u64;
    for line in bytes.split(|c| *c == b'\n') {
        if line.is_empty() {
            break;
        }

        total += get_line_joltage(line) as u64;
    }
    total
}

fn get_line_joltage(line: &[u8]) -> u8 {
    unsafe { core::hint::assert_unchecked(!line.is_empty()) }

    let first_digit_pos = largest_byte_pos(&line[..line.len() - 1]);
    let second_digit_pos =
        first_digit_pos + 1 + largest_byte_pos(&line[first_digit_pos + 1..]);
    10 * (line[first_digit_pos] - b'0') + (line[second_digit_pos] - b'0')
}

fn largest_byte_pos(s: &[u8]) -> usize {
    let mut max_pos = 0;
    let mut max_byte = 0;
    for (i, &byte) in s.iter().enumerate() {
        if byte == 9 {
            return i;
        };

        if byte > max_byte {
            max_byte = byte;
            max_pos = i;
        }
    }
    max_pos
}

#[cfg(test)]
mod test {
    use super::get_line_joltage;

    #[test]
    fn trivial() {
        assert_eq!(get_line_joltage(b"17"), 17);
        assert_eq!(get_line_joltage(b"52"), 52);
        assert_eq!(get_line_joltage(b"69"), 69);
    }

    #[test]
    fn example() {
        assert_eq!(get_line_joltage(b"987654321111111"), 98);
        assert_eq!(get_line_joltage(b"811111111111119"), 89);
        assert_eq!(get_line_joltage(b"234234234234278"), 78);
        assert_eq!(get_line_joltage(b"818181911112111"), 92);
    }
}
