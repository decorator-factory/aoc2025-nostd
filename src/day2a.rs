// Naive solution for day 2a

use core::ffi::CStr;

use crate::prelude::*;

advent_of_code_impl!(run);

fn run(path: &CStr) -> Result<(), MysteryStr<'static>> {
    let mmap = open_mmap(path)?;
    let bytes = mmap.as_slice();

    let mut total = 0u64;

    for range in bytes.split(|c| *c == b',') {
        let range = range.trim_ascii();
        let (left, right) = range.split_once(|c| *c == b'-').unwrap();
        let left = u64::from_ascii(left).unwrap();
        let right = u64::from_ascii(right).unwrap();
        assert!(right > left);

        const NUMBUF_SIZE: usize = 20; // 64-bit numbers are at most 20 characters
        let mut buf = [0u8; NUMBUF_SIZE];
        for n in left..right + 1 {
            let as_str: &[u8] = unsafe {
                let count =
                    libc::snprintf(buf.as_mut_ptr().cast(), NUMBUF_SIZE, c"%llu".as_ptr(), n);
                debug_assert!(count > 0);
                buf.get_unchecked(0..count as usize)
            };
            if is_silly(as_str) {
                total += n;
            }
        }
    }

    printf!("Total silly IDs are: {total}\n");
    Ok(())
}

fn is_silly(s: &[u8]) -> bool {
    s.len() % 2 == 0 && s[0..s.len() / 2] == s[s.len() / 2..]
}
