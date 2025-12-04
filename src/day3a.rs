// Slightly optimized solution for day 3a

use core::simd::prelude::*;

// ferris_elf compatible
pub fn run(bytes: &[u8]) -> u32 {
    // For this problem, single threaded is faster
    let mut total = 0u32;
    let bytes = bytes.trim_ascii_end();

    let mut last = 0usize;
    for uwu in memchr::memchr_iter(b'\n', bytes) {
        let line = unsafe { bytes.get_unchecked(last..uwu) };
        total += get_line_joltage(line) as u32;
        last = uwu + 1;
    }
    if last < bytes.len() {
        let line = unsafe { bytes.get_unchecked(last..) };
        if !line.is_empty() {
            total += get_line_joltage(line) as u32;
        }
    }
    total
}

fn get_line_joltage(line: &[u8]) -> u8 {
    unsafe {
        core::hint::assert_unchecked(!line.is_empty());
        let (high, index) = largest_byte_and_index(line.get_unchecked(..line.len() - 1));
        let low = largest_byte_just(line.get_unchecked(index + 1..));
        10 * (high - b'0') + (low - b'0')
    }
}

fn largest_byte_and_index(s: &[u8]) -> (u8, usize) {
    #[cfg(target_feature = "avx512f")]
    const LANES: usize = 32;
    #[cfg(not(target_feature = "avx512f"))]
    const LANES: usize = 16;

    unsafe {
        core::hint::assert_unchecked(!s.is_empty());

        let mut max_byte = 0;

        let tail = s.len() % LANES;
        let chunks = s.len() / LANES;

        let mut apple = 0;

        for j in 0..chunks {
            let lane =
                Simd::<u8, LANES>::from_slice(s.get_unchecked(j * LANES..(j + 1) * LANES));
            let max = lane.reduce_max();
            if max > max_byte {
                max_byte = max;
                apple = j;
            }
        }

        let mut tail_max_pos = 0;
        let mut tail_max_byte = 0;
        for i in 0..tail {
            let b = *s.get_unchecked(chunks * LANES + i);

            if b > tail_max_byte {
                tail_max_byte = b;
                tail_max_pos = chunks * LANES + i;
            }
        }

        if tail_max_byte > max_byte {
            (tail_max_byte, tail_max_pos)
        } else {
            // here, max_byte must be >0, and therefore apple must have been assigned to

            let banana = Simd::<u8, LANES>::from_slice(
                s.get_unchecked(apple * LANES..(apple + 1) * LANES),
            );
            let i = banana.simd_eq(Simd::splat(max_byte)).first_set().unwrap_unchecked();
            (max_byte, apple * LANES + i)
        }
    }
}

fn largest_byte_just(s: &[u8]) -> u8 {
    #[cfg(target_feature = "avx512f")]
    const LANES: usize = 32;
    #[cfg(not(target_feature = "avx512f"))]
    const LANES: usize = 16;

    unsafe {
        core::hint::assert_unchecked(!s.is_empty());

        let mut max_byte = 0;

        let tail = s.len() % LANES;
        let chunks = s.len() / LANES;

        for j in 0..chunks {
            let lane =
                Simd::<u8, LANES>::from_slice(s.get_unchecked(j * LANES..(j + 1) * LANES));
            max_byte = lane.reduce_max().max(max_byte);
        }

        for i in 0..tail {
            let b = *s.get_unchecked(chunks * LANES + i);
            max_byte = b.max(max_byte);
        }

        max_byte
    }
}

////////////
use crate::prelude::*;

advent_of_code_impl!(aoc);
fn aoc(path: &core::ffi::CStr) -> Result<(), MysteryStr<'static>> {
    let mem = Malloc::clone(open_mmap(path)?.as_slice());

    let start = get_nanos();
    let answer = run(mem.as_bytes());
    let end = get_nanos();
    printf!("Elapsed time: {:.2}us\n", (end as f64 - start as f64) / 1000.0);

    printf!("Total joltage is: {answer}\n");
    Ok(())
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
