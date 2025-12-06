// Very naive solution for day 5b

pub fn run(raw: &[u8]) -> u64 {
    let last_dash_pos = memchr::memrchr(b'-', raw).unwrap();
    let ranges_end =
        memchr::memchr(b'\n', &raw[last_dash_pos + 1..]).unwrap() + last_dash_pos + 1;

    let input = &raw[..ranges_end + 1];

    let mut ranges = ArrayVec::<(u64, u64), 256>::new();

    {
        let mut start = 0usize;
        for pos in memchr::memchr_iter(b'\n', input) {
            let line = &input[start..pos];
            let dash_pos = memchr::memchr(b'-', line).unwrap();

            let left = u64::from_ascii(&line[..dash_pos]).unwrap();
            let right = u64::from_ascii(&line[dash_pos + 1..]).unwrap();
            ranges.push((left, right));

            start = pos + 1;
        }
    }

    // Remove range overlaps using stupid quadratic algorithm
    let rs = ranges.as_mut_slice();
    rs.sort_unstable_by_key(|&(start, _)| start);

    for i in 0..rs.len() {
        let (_, max1) = rs[i];
        for pair in &mut rs[i + 1..] {
            pair.0 = pair.0.max(max1 + 1);
        }
    }

    rs.iter().map(|&(min, max)| (max + 1).saturating_sub(min)).sum()
}

use crate::prelude::*;
advent_of_code_impl!(aoc);
fn aoc(path: &core::ffi::CStr) -> Result<(), MysteryStr<'static>> {
    let mem = Malloc::clone(open_mmap(path)?.as_slice());

    let start = get_nanos();
    let answer = run(mem.as_bytes());
    let end = get_nanos();
    printf!("Elapsed time: {:.2}us\n", (end as f64 - start as f64) / 1000.0);

    printf!("Answer is: {answer}\n");
    Ok(())
}
