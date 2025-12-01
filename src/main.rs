#![no_std]
#![no_main]

use core::ffi::{
    CStr,
    c_void,
};

use crate::prelude::MysteryStr;
extern crate libc;

mod day1a;
mod day1b;
mod prelude;

type DayFn = fn(path: &CStr) -> Result<(), MysteryStr<'static>>;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(argc: isize, argv: *const *const i8) -> isize {
    if argc != 3 {
        prelude::println_str("error: Expected exactly 2 arguments");
        return 1;
    }
    let day = unsafe { core::ffi::CStr::from_ptr(*argv.add(1)).to_bytes() };
    let path = unsafe { core::ffi::CStr::from_ptr(*argv.add(2)) };

    let day_fn: Option<DayFn> = match day {
        b"day1a" => Some(day1a::run),
        b"day1b" => Some(day1b::run),
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

static mut IS_ABORTING: bool = false;

#[panic_handler]
fn my_panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        if IS_ABORTING {
            let buf = b"Panicked while panicking. Aborting.\n";
            libc::write(libc::STDERR_FILENO, buf.as_ptr() as *const c_void, buf.len());
            libc::abort();
        } else {
            IS_ABORTING = true;
            prelude::eprintf!("{info}\n");
        }
        libc::exit(1);
    }
}
