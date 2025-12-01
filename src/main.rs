#![no_std]
#![no_main]
extern crate libc;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(argc: isize, argv: *const *const i8) -> isize {
    unsafe {
        libc::printf(c"Hello, world!\n".as_ptr() as *const _);
    }

    for i in 0..argc {
        if unsafe { libc::strcmp(c"banana".as_ptr(), *argv.offset(i)) } == 0 {
            panic!("banana mentioned");
        }
    }

    0
}

#[panic_handler]
fn my_panic(info: &core::panic::PanicInfo) -> ! {
    let message = info.message().as_str().unwrap_or("<no message>");
    let location = info.location().unwrap();
    let file = location.file();

    unsafe {
        libc::printf(
            c"panic at %u:%u in %.*s: %.*s\n".as_ptr() as *const _,
            location.line() as core::ffi::c_uint,
            location.column() as core::ffi::c_uint,
            file.len(),
            file.as_ptr(),
            message.len(),
            message.as_ptr(),
        );

        libc::exit(1);
    }
}
