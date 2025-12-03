// Naive solution for day 3a

use core::ffi::CStr;

use crate::prelude::*;

// ferris_elf compatible
pub fn run(bytes: &[u8]) -> u64 {
    let mut total = 0u64;
    for line in bytes.split(|c| *c == b'\n') {
        if line.is_empty() {
            break;
        }

        total += get_line_joltage12(line);
    }
    total
}

#[rustfmt::skip]
#[allow(clippy::identity_op)]
fn get_line_joltage12(line: &[u8]) -> u64  {
    unsafe { core::hint::assert_unchecked(line.len() > 12) }

    // loops are for cowards

    let dp11 @ last =              largest_byte_pos(&line[          ..line.len() - 11]);
    let dp10 @ last = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 10]);
    let dp9 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 9]);
    let dp8 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 8]);
    let dp7 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 7]);
    let dp6 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 6]);
    let dp5 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 5]);
    let dp4 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 4]);
    let dp3 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 3]);
    let dp2 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 2]);
    let dp1 @ last  = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len() - 1]);
    let dp0         = (last + 1) + largest_byte_pos(&line[(last + 1)..line.len()]);

    0
    + (line[dp11] - b'0') as u64 * (10u64.pow(11))
    + (line[dp10] - b'0') as u64 * (10u64.pow(10))
    + (line[dp9] - b'0') as u64 * (10u64.pow(9))
    + (line[dp8] - b'0') as u64 * (10u64.pow(8))
    + (line[dp7] - b'0') as u64 * (10u64.pow(7))
    + (line[dp6] - b'0') as u64 * (10u64.pow(6))
    + (line[dp5] - b'0') as u64 * (10u64.pow(5))
    + (line[dp4] - b'0') as u64 * (10u64.pow(4))
    + (line[dp3] - b'0') as u64 * (10u64.pow(3))
    + (line[dp2] - b'0') as u64 * (10u64.pow(2))
    + (line[dp1] - b'0') as u64 * (10u64.pow(1))
    + (line[dp0] - b'0') as u64 * (10u64.pow(0))
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

#[cfg(test)]
mod test {
    use super::get_line_joltage12;

    #[test]
    fn example() {
        assert_eq!(get_line_joltage12(b"987654321111111"), 987654321111);
        assert_eq!(get_line_joltage12(b"811111111111119"), 811111111119);
        assert_eq!(get_line_joltage12(b"234234234234278"), 434234234278);
        assert_eq!(get_line_joltage12(b"818181911112111"), 888911112111);
    }
}
