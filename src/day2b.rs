// Naive solution for day 2b

use core::ffi::CStr;

use crate::prelude::*;

pub fn run(path: &CStr) -> Result<(), MysteryStr<'static>> {
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
            if is_very_silly(as_str) {
                total += n;
            }
        }
    }

    printf!("Total silly IDs are: {total}\n");
    Ok(())
}

fn is_very_silly(s: &[u8]) -> bool {
    let len = s.len();
    'banana: for n in 1..=10.min(len - 1) {
        if len % n == 0 {
            let pat = &s[..n];
            for i in 1..(len / n) {
                if pat != &s[i * pat.len()..(i + 1) * pat.len()] {
                    continue 'banana;
                }
            }
            return true;
        }
    }

    false
}

#[cfg(test)]
mod test {
    use super::is_very_silly;

    #[test]
    fn len1() {
        assert!(!is_very_silly(b"0"));
        assert!(!is_very_silly(b"1"));
        assert!(!is_very_silly(b"a"));
    }

    #[test]
    fn len2() {
        assert!(is_very_silly(b"00"));
        assert!(is_very_silly(b"11"));
        assert!(!is_very_silly(b"69"));
    }

    #[test]
    fn len3() {
        assert!(is_very_silly(b"777"));
        assert!(!is_very_silly(b"420"));
        assert!(!is_very_silly(b"001"));
        assert!(!is_very_silly(b"010"));
        assert!(!is_very_silly(b"100"));
        assert!(!is_very_silly(b"110"));
    }

    #[test]
    fn len4() {
        assert!(is_very_silly(b"0101"));
        assert!(is_very_silly(b"5555"));
        assert!(!is_very_silly(b"0011"));
        assert!(!is_very_silly(b"0001"));
    }

    #[test]
    fn len8() {
        assert!(is_very_silly(b"12341234"));
        assert!(is_very_silly(b"12121212"));
        assert!(is_very_silly(b"22222222"));
        assert!(!is_very_silly(b"12345678"));
    }

    #[test]
    fn len14() {
        assert!(is_very_silly(b"12345671234567"));
        assert!(!is_very_silly(b"12345671234568"));
    }

    #[test]
    fn len16() {
        assert!(is_very_silly(b"1234567812345678"));
        assert!(is_very_silly(b"1234123412341234"));
        assert!(is_very_silly(b"1212121212121212"));
        assert!(is_very_silly(b"9999999999999999"));
        assert!(!is_very_silly(b"1234123412341235"));
        assert!(!is_very_silly(b"1234567812345679"));
    }
}
