// Naive solution for day 4b

const QUEUE_SIZE: usize = 32768;

pub fn run(raw: &[u8]) -> u32 {
    let mut buf = Malloc::clone(raw.trim_ascii_end());
    let mut input = Input::new(buf.as_mut_bytes());

    let mut total = 0u32;

    let mut queue = ArrayVec::new();
    for (x, y) in [
        (0, 0),
        (input.width - 1, 0),
        (0, input.height - 1),
        (input.width - 1, input.height - 1),
    ] {
        if input.is_paper_at(x, y) {
            queue.push((x as u16, y as u16));
        }
    }

    loop {
        single_step(&input, &mut queue);
        if queue.len() == 0 {
            break;
        }
        total += queue.len() as u32;
        for &(x, y) in queue.as_slice() {
            input.clear_paper_at(x as usize, y as usize);
        }
        queue.clear();
    }
    total
}


fn single_step(input: &Input, out: &mut ArrayVec<(u16, u16), QUEUE_SIZE>) {
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
            out.push((x as u16, 0));
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
            out.push((0, y as u16));
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
            out.push((x as u16, y as u16));
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
            out.push((x as u16, y as u16));
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
                out.push((x as u16, y as u16));
            }
        }
    }
}

struct Input<'a> {
    data: &'a mut [u8],
    width: usize,
    height: usize,
}

impl<'a> Input<'a> {
    fn new(data: &'a mut [u8]) -> Self {
        let width = memchr::memchr(b'\n', data).unwrap();
        let height = (data.len() + 1) / (width + 1);
        Self { data, width, height }
    }

    #[inline(always)]
    pub fn is_paper_at(&self, x: usize, y: usize) -> bool {
        (unsafe { *self.mut_at(x, y) }) == b'@'
    }

    #[inline(always)]
    pub fn clear_paper_at(&mut self, x: usize, y: usize) {
        unsafe { *self.mut_at(x, y) = b'x' };
    }

    #[inline(always)]
    unsafe fn mut_at(&self, x: usize, y: usize) -> *mut u8 {
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
            self.data.as_ptr().add(index) as *mut u8
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
