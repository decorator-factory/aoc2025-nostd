#![no_std]
#![no_main]
#![feature(slice_split_once)]
#![feature(int_from_ascii)]

use core::ffi::{
    CStr,
    c_void,
};

extern crate libc;

mod day1a;
mod day1b;
mod day2a;
mod day2b;
mod prelude;

type DayFn = fn(path: &CStr) -> Result<(), prelude::MysteryStr<'static>>;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(argc: isize, argv: *const *const i8) -> isize {
    if argc == 1 {
        // HACK
        day2b::fake_test();
        return 0;
    }

    if argc != 3 {
        prelude::println_str("error: Expected exactly 2 arguments");
        return 1;
    }
    let day = unsafe { core::ffi::CStr::from_ptr(*argv.add(1)).to_bytes() };
    let path = unsafe { core::ffi::CStr::from_ptr(*argv.add(2)) };

    let day_fn: Option<DayFn> = match day {
        b"day1a" => Some(day1a::run),
        b"day1b" => Some(day1b::run),
        b"day2a" => Some(day2a::run),
        b"day2b" => Some(day2b::run),
        _ => None,
    };
    let Some(day_fn) = day_fn else {
        prelude::eprintf!("error: invalid day specified\n");
        return 1;
    };

    let owo = day_fn(path);
    if let Err(e) = owo {
        prelude::eprintf!("error: {e:?}\n");
        return 1;
    }
    0
}

#[panic_handler]
fn my_panic(info: &core::panic::PanicInfo) -> ! {
    static mut IS_PANICKING: bool = false;

    unsafe {
        if IS_PANICKING {
            let buf = b"Panicked while panicking. Aborting.\n";
            libc::write(libc::STDERR_FILENO, buf.as_ptr() as *const c_void, buf.len());
            libc::abort();
        } else {
            IS_PANICKING = true;
            prelude::eprintf!("{info}\n");
        }
        libc::exit(1);
    }
}
