// Slightly less naive solution for day 5a

#[cfg(not(feature = "banana"))]
use arrayvec::ArrayVec;

pub fn run(raw: &[u8]) -> u16 {
    unsafe /* :^) */ {
        // TODO: find faster way to parse integers
        let last_dash_pos = memchr::memrchr(b'-', raw).unwrap();
        let ranges_end =
            memchr::memchr(b'\n', &raw[last_dash_pos + 1..]).unwrap() + last_dash_pos + 1;

        let (branges, bfridge) = (&raw[..ranges_end + 1], &raw[ranges_end + 2..]);

        let mut ranges = ArrayVec::<(u64, u64), 256>::new();
        let mut fridge = ArrayVec::<u64, 1024>::new();

        let mut buckets = {
            // crimes against humanity
            let mut owo: ArrayVec<ArrayVec<u8, 16>, 2048> =
                core::mem::MaybeUninit::zeroed().assume_init();
            owo.set_len(1024);
            owo
        };

        {
            let mut start = 0usize;
            for pos in memchr::memchr_iter(b'\n', branges) {
                let line = branges.get_unchecked(start..pos);
                let dash_pos = memchr::memchr(b'-', line).unwrap_unchecked();

                let left = u64::from_ascii(line.get_unchecked(..dash_pos)).unwrap_unchecked();
                let right =
                    u64::from_ascii(line.get_unchecked(dash_pos + 1..)).unwrap_unchecked();
                let i = ranges.len() as u8;
                ranges.push_unchecked((left, right));

                let low_mask = (left >> (50 - 11)) as u16;
                let high_mask = (right >> (50 - 11)) as u16;
                for b in low_mask..=high_mask {
                    buckets.as_mut_slice().get_unchecked_mut(b as usize).push_unchecked(i)
                }

                start = pos + 1;
            }
        }

        {
            let mut start = 0usize;
            for pos in memchr::memchr_iter(b'\n', bfridge) {
                let line = bfridge.get_unchecked(start..pos);
                fridge.push_unchecked(u64::from_ascii(line).unwrap_unchecked());
                start = pos + 1;
            }
        }

        let mut fresh_count: u16 = 0;
        for &ingredient in fridge.as_slice() {
            let mask = (ingredient >> (50 - 11)) as u16;
            let indexes = buckets.as_slice().get_unchecked(mask as usize);

            for &i in indexes.as_slice() {
                // let min = *range_mins.get_unchecked(i as usize);
                // let max = *range_maxs.get_unchecked(i as usize);
                let (min, max) = *ranges.as_slice().get_unchecked(i as usize);
                if min <= ingredient && ingredient <= max {
                    fresh_count += 1;
                    break;
                }
            }
        }

        fresh_count
    }
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
