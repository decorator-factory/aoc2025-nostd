// Naive solution for day 5a
#![allow(unused_attributes)]
// need these for ferris_elf
#![feature(slice_split_once)]
#![feature(portable_simd)]
#![feature(bstr)]

#[cfg(not(feature = "banana"))]
use arrayvec::ArrayVec;

pub fn run(raw: &[u8]) -> u16 {
    let bounds_t0 = get_nanos_unchecked();
    let last_dash_pos = memchr::memrchr(b'-', raw).unwrap();
    let ranges_end =
        memchr::memchr(b'\n', &raw[last_dash_pos + 1..]).unwrap() + last_dash_pos + 1;

        let (branges, bfridge) = (&raw[..ranges_end + 1], &raw[ranges_end + 2..]);
    let bounds_t1 = get_nanos_unchecked();

    let mut range_mins = ArrayVec::<u64, 256>::new();
    let mut range_maxs = ArrayVec::<u64, 256>::new();
    let mut fridge = ArrayVec::<u64, 1024>::new();

    let ranges_t0 = get_nanos_unchecked();
    {
        let mut start = 0usize;
        for pos in memchr::memchr_iter(b'\n', branges) {
            unsafe {
                let line = branges.get_unchecked(start..pos);
                let dash_pos = memchr::memchr(b'-', line).unwrap_unchecked();
                let left = u64::from_ascii(&line[..dash_pos]).unwrap_unchecked();
                let right = u64::from_ascii(&line[dash_pos + 1..]).unwrap_unchecked();
                range_mins.push_unchecked(left);
                range_maxs.push_unchecked(right);
            }
            start = pos + 1;
        }
    }
    let range_mins = range_mins.as_slice();
    let range_maxs = range_maxs.as_slice();
    let ranges_t1 = get_nanos_unchecked();

    let fridge_t0 = get_nanos_unchecked();
    {
        let mut start = 0usize;
        for pos in memchr::memchr_iter(b'\n', bfridge) {
            unsafe {
                let line = bfridge.get_unchecked(start..pos);
                fridge.push_unchecked(u64::from_ascii(line).unwrap_unchecked());
            }
            start = pos + 1;
        }
    }
    let fridge_t1 = get_nanos_unchecked();

    let tada_t0 = get_nanos_unchecked();
    let mut fresh_count: u16 = 0;
    for &ingredient in fridge.as_slice() {
        for (&min, &max) in range_mins.iter().zip(range_maxs.iter()) {
            if min <= ingredient && ingredient <= max {
                fresh_count += 1;
                break;
            }
        }
    }
    let tada_t1 = get_nanos_unchecked();

    eprintf!("finding bounds: {}\n", bounds_t1 - bounds_t0);
    eprintf!("parsing ranges: {}\n", ranges_t1 - ranges_t0);
    eprintf!("parsing fridge: {}\n", fridge_t1 - fridge_t0);
    eprintf!("big expensive loop: {}\n", tada_t1 - tada_t0);

    fresh_count
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
