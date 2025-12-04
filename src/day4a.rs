// Naive solution for day 4a

pub fn run(raw: &[u8]) -> u32 {
    let raw = raw.trim_ascii_end();

    let mut total = 0u32;

    let input = Input::new(raw);

    // handle corners
    total += input.is_paper_at(0, 0) as u32
        + input.is_paper_at(input.width - 1, 0) as u32
        + input.is_paper_at(0, input.height - 1) as u32
        + input.is_paper_at(input.width - 1, input.height - 1) as u32;

    // handle the first row
    for x in 1..input.width - 1 {
        if !input.is_paper_at(x, 0) {
            continue;
        };

        let around = input.is_paper_at(x - 1, 0) as u32
            + input.is_paper_at(x + 1, 0) as u32
            + input.is_paper_at(x - 1, 1) as u32
            + input.is_paper_at(x, 1) as u32
            + input.is_paper_at(x + 1, 1) as u32;
        if around < 4 {
            total += 1;
        }
    }

    // handle the left column
    for y in 1..input.height - 1 {
        if !input.is_paper_at(0, y) {
            continue;
        };

        let around = input.is_paper_at(0, y - 1) as u32
            + input.is_paper_at(0, y + 1) as u32
            + input.is_paper_at(1, y - 1) as u32
            + input.is_paper_at(1, y) as u32
            + input.is_paper_at(1, y + 1) as u32;
        if around < 4 {
            total += 1;
        }
    }

    // handle the right column
    let x = input.width - 1;
    for y in 1..input.height - 1 {
        if !input.is_paper_at(x, y) {
            continue;
        };

        let around = input.is_paper_at(x, y - 1) as u32
            + input.is_paper_at(x, y + 1) as u32
            + input.is_paper_at(x - 1, y - 1) as u32
            + input.is_paper_at(x - 1, y) as u32
            + input.is_paper_at(x - 1, y + 1) as u32;

        if around < 4 {
            total += 1;
        }
    }

    // handle the last row
    let y = input.width - 1;
    for x in 1..input.width - 1 {
        if !input.is_paper_at(x, y) {
            continue;
        };

        let around = input.is_paper_at(x - 1, y) as u32
            + input.is_paper_at(x + 1, y) as u32
            + input.is_paper_at(x - 1, y - 1) as u32
            + input.is_paper_at(x, y - 1) as u32
            + input.is_paper_at(x + 1, y - 1) as u32;

        if around < 4 {
            total += 1;
        }
    }

    // handle the main part

    for x in 1..input.width - 1 {
        for y in 1..input.height - 1 {
            if !input.is_paper_at(x, y) {
                continue;
            }

            let mut around = 0u8;
            for dx in [-1i16, 0, 1] {
                for dy in [-1i16, 0, 1] {
                    if !(dx == 0 && dy == 0) {
                        around += input
                            .is_paper_at((x as i16 + dx) as usize, (y as i16 + dy) as usize)
                            as u8;
                    }
                }
            }

            if around < 4 {
                total += 1;
            }
        }
    }

    total
}

struct Input<'a> {
    data: &'a [u8],
    width: usize,
    height: usize,
}

impl<'a> Input<'a> {
    fn new(data: &'a [u8]) -> Self {
        let width = memchr::memchr(b'\n', data).unwrap();
        let height = (data.len() + 1) / (width + 1);
        Self { data, width, height }
    }

    #[inline(always)]
    fn is_paper_at(&self, x: usize, y: usize) -> bool {
        debug_assert!(
            (x < self.width && y < self.width),
            "x, y = {}, {}, wh = {}x{}",
            x,
            y,
            self.width,
            self.height
        );

        unsafe {
            let index = y * (self.width + 1) + x;
            *self.data.get_unchecked(index) == b'@'
        }
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
