#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(slice_split_once)]
#![feature(int_from_ascii)]
#![feature(portable_simd)]
#![feature(stmt_expr_attributes)]

mod day1a;
mod day1b;
mod day2a;
mod day2b;
mod day3a;
mod day3b;
mod prelude;

#[inline(always)]
#[cfg_attr(test, allow(unused))]
unsafe fn main_impl(argc: isize, argv: *const *const i8) -> isize {
    if argc != 3 {
        prelude::println_str("error: Expected exactly 2 arguments");
        return 1;
    }
    let day = unsafe { core::ffi::CStr::from_ptr(*argv.add(1)) };
    let path = unsafe { core::ffi::CStr::from_ptr(*argv.add(2)) };

    let Some(day_fn) = prelude::fetch_day_solution(day) else {
        prelude::eprintf!("error: unknown AoC day\n");
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
