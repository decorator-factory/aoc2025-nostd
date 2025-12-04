// Naive solution for day 3a

use core::simd::prelude::*;

// ferris_elf compatible
pub fn run(bytes: &[u8]) -> u64 {
    let mut total = 0u64;
    let bytes = bytes.trim_ascii_end();

    let mut last = 0usize;
    for uwu in memchr::memchr_iter(b'\n', bytes) {
        let line = unsafe { bytes.get_unchecked(last..uwu) };
        total += get_line_joltage12(line);
        last = uwu + 1;
    }
    if last < bytes.len() {
        let line = unsafe { bytes.get_unchecked(last..) };
        if !line.is_empty() {
            total += get_line_joltage12(line);
        }
    }
    total
}

const TOP_TENS: u64x4 =
    u64x4::from_array([10u64.pow(11), 10u64.pow(10), 10u64.pow(9), 10u64.pow(8)]);

const BOT_TENS: u32x8 = u32x8::from_array([
    10u32.pow(7),
    10u32.pow(6),
    10u32.pow(5),
    10u32.pow(4),
    10u32.pow(3),
    10u32.pow(2),
    10u32.pow(1),
    10u32.pow(0),
]);

#[allow(clippy::identity_op, clippy::needless_range_loop)]
fn get_line_joltage12(line: &[u8]) -> u64 {
    unsafe {
        core::hint::assert_unchecked(line.len() > 12);

        let mut tops = [0u64; 4];
        let mut bottoms = [0u32; 8];
        let mut index = 0usize;
        for i in 0..4 {
            let (b, digit_pos) =
                largest_byte(line.get_unchecked(index..line.len() - (11 - i)));
            tops[i] = (b - b'0') as u64;
            index += digit_pos + 1;
        }
        for i in 4..12 {
            let (b, digit_pos) =
                largest_byte(line.get_unchecked(index..line.len() - (11 - i)));
            bottoms[i - 4] = (b - b'0') as u32;
            index += digit_pos + 1;
        }

        let tops = u64x4::from_array(tops);
        let bottoms = u32x8::from_array(bottoms);

        (tops * TOP_TENS).reduce_sum() + ((bottoms * BOT_TENS).reduce_sum() as u64)
    }
}

fn largest_byte(s: &[u8]) -> (u8, usize) {
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

////////////////////

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
    use super::{
        get_line_joltage12,
        largest_byte,
    };

    #[test]
    fn example() {
        assert_eq!(get_line_joltage12(b"987654321111111"), 987654321111);
        assert_eq!(get_line_joltage12(b"811111111111119"), 811111111119);
        assert_eq!(get_line_joltage12(b"234234234234278"), 434234234278);
        assert_eq!(get_line_joltage12(b"818181911112111"), 888911112111);
    }

    #[test]
    fn largest_byte_test() {
        assert_eq!(largest_byte(b"019239558431753"), (b'9', 2));
        assert_eq!(largest_byte(b"0000000300000006"), (b'6', 15));
        assert_eq!(largest_byte(b"0000000600000003"), (b'6', 7));
        assert_eq!(largest_byte(b"0000000600000073"), (b'7', 14));
        assert_eq!(largest_byte(b"00000006000000738"), (b'8', 16));

        assert_eq!(largest_byte(b"0"), (b'0', 0));
        assert_eq!(largest_byte(b"00"), (b'0', 0));
        assert_eq!(largest_byte(b"000"), (b'0', 0));
        assert_eq!(largest_byte(b"0000"), (b'0', 0));
        assert_eq!(largest_byte(b"00000"), (b'0', 0));
        assert_eq!(largest_byte(b"000000"), (b'0', 0));
        assert_eq!(largest_byte(b"0000000"), (b'0', 0));
        assert_eq!(largest_byte(b"00000000"), (b'0', 0));
        assert_eq!(largest_byte(b"000000000"), (b'0', 0));
        assert_eq!(largest_byte(b"0000000000"), (b'0', 0));
        assert_eq!(largest_byte(b"00000000000000000000000000000000"), (b'0', 0));
        assert_eq!(largest_byte(b"1"), (b'1', 0));
        assert_eq!(largest_byte(b"00001"), (b'1', 4));
    }
}
