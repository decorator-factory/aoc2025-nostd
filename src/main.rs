#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(slice_split_once)]
#![feature(int_from_ascii)]

use core::ffi::CStr;

extern crate libc;

mod day1a;
mod day1b;
mod day2a;
mod day2b;
mod prelude;

#[inline(always)]
#[cfg_attr(test, allow(unused))]
unsafe fn main_impl(argc: isize, argv: *const *const i8) -> isize {
    type DayFn = fn(path: &CStr) -> Result<(), prelude::MysteryStr<'static>>;

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

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(argc: isize, argv: *const *const i8) -> isize {
    unsafe { main_impl(argc, argv) }
}

#[cfg(not(test))]
#[panic_handler]
fn my_panic(info: &core::panic::PanicInfo) -> ! {
    use core::ffi::c_void;
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
