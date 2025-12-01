use core::{
    ffi::{
        CStr,
        c_char,
        c_int,
        c_void,
    },
    num::NonZero,
    slice,
    str,
};

pub fn println_str(s: &str) -> c_int {
    // SAFETY:
    //  - passing the correct format string, `.*` means "length (as usize), then pointer"
    //  - the length and the pointer are definitely coming from the same string
    unsafe { libc::printf(c"%.*s\n".as_ptr(), s.len(), s.as_ptr()) }
}

pub fn get_errno() -> Option<NonZero<c_int>> {
    // SAFETY: errno is threadsafe (https://linux.die.net/man/3/errno)
    NonZero::new(unsafe { core::ptr::read_volatile(libc::__errno_location()) })
}

pub fn reset_errno() {
    // SAFETY: errno is threadsafe (https://linux.die.net/man/3/errno)
    unsafe { core::ptr::write_volatile(libc::__errno_location(), 0) }
}

pub fn strerror_leak(errno: NonZero<c_int>) -> &'static CStr {
    unsafe {
        const BUFFER_LEN: usize = 4096;
        let ptr = libc::malloc(BUFFER_LEN) as *mut c_char;
        if ptr.is_null() {
            panic!("download more RAM")
        }
        if libc::strerror_r(errno.into(), ptr, BUFFER_LEN) != 0 {
            panic!("errorception")
        }
        CStr::from_ptr(ptr)
    }
}

// printf and friends

macro_rules! printf {
    ($($arg:tt)*) => {
        $crate::prelude::printf_impl(::core::format_args!($($arg)*)).unwrap();
    };
}
pub(crate) use printf;

macro_rules! eprintf {
    ($($arg:tt)*) => {
        $crate::prelude::eprintf_impl(::core::format_args!($($arg)*));
    };
}
pub(crate) use eprintf;

macro_rules! fdprintf {
    ($dst:expr, $($arg:tt)*) => {
        $crate::prelude::fdprintf_impl($expr, ::core::format_args!($($arg)*)).unwrap();
    };
}
pub(crate) use fdprintf;

macro_rules! sprintf_leak {
    ($($arg:tt)*) => {
        $crate::prelude::sprintf_leak_impl(::core::format_args!($($arg)*), 4096).unwrap()
    };

    ([ $size:expr ] $($arg:tt)*) => {
        $crate::prelude::sprintf_leak_impl(::core::format_args!($($arg)*), $size).unwrap()
    };
}
pub(crate) use sprintf_leak;

pub fn sprintf_leak_impl<'a>(
    fmt: core::fmt::Arguments<'a>,
    bufsize: usize,
) -> Result<&'static str, ()> {
    assert!(bufsize > 1, "bufsize is too small: {bufsize}");

    let ptr = unsafe { libc::malloc(bufsize) as *mut u8 };
    if ptr.is_null() {
        panic!("download more RAM")
    };

    let mut writer = unsafe { RawPtrWriter::new(ptr, bufsize) };
    if core::fmt::write(&mut writer, fmt).is_err() {
        Err(())
    } else {
        let length = bufsize - writer.capacity_left();
        let slc = unsafe { slice::from_raw_parts(ptr as _, length) };
        Ok(str::from_utf8(slc).unwrap())
    }
}

mod raw_ptr_writer {
    use core::ffi::c_void;

    pub(super) struct RawPtrWriter {
        ptr: *mut u8,
        capacity: usize,
    }

    impl RawPtrWriter {
        /// # Safety
        /// - ptr must be pointing to at least `capacity` bytes of memory
        /// - to use the Write trait, no references must be aliasing
        pub(super) unsafe fn new(ptr: *mut u8, capacity: usize) -> Self {
            Self { ptr, capacity }
        }

        pub(super) fn capacity_left(&self) -> usize {
            self.capacity
        }
    }

    impl core::fmt::Write for RawPtrWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            if s.len() > self.capacity {
                return Err(core::fmt::Error);
            }

            unsafe { libc::memcpy(self.ptr as *mut c_void, s.as_ptr() as *const c_void, s.len()) };

            self.ptr = unsafe { self.ptr.add(s.len()) };
            self.capacity -= s.len();
            Ok(())
        }
    }
}
use raw_ptr_writer::RawPtrWriter;

pub fn printf_impl<'a>(fmt: core::fmt::Arguments<'a>) -> Result<(), &'static CStr> {
    fdprintf_impl(libc::STDOUT_FILENO, fmt)
}

pub fn eprintf_impl<'a>(fmt: core::fmt::Arguments<'a>) {
    if let Err(e) = fdprintf_impl(libc::STDERR_FILENO, fmt) {
        unsafe { libc::printf(c"Houston, we have a problem: %s\n".as_ptr(), e.as_ptr()) };
    }
}

pub fn fdprintf_impl<'a>(fd: c_int, fmt: core::fmt::Arguments<'a>) -> Result<(), &'static CStr> {
    let mut fw = FileWriter { fd, error: None };
    reset_errno();
    _ = core::fmt::write(&mut fw, fmt);
    if let Some(errno) = fw.error { Err(strerror_leak(errno)) } else { Ok(()) }
}

struct FileWriter {
    fd: c_int,
    error: Option<NonZero<c_int>>,
}

impl core::fmt::Write for FileWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            libc::write(self.fd, s.as_ptr() as *const c_void, s.len());
        }
        if let Some(errno) = get_errno() {
            self.error = Some(errno);
            return Err(core::fmt::Error);
        }
        Ok(())
    }
}

// -------------------------
// open_lines implementation
//
pub fn open_lines(path: &CStr) -> Result<FileLines, &'static CStr> {
    reset_errno();
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY) };
    if let Some(errno) = get_errno() {
        Err(strerror_leak(errno))
    } else {
        Ok(FileLines { buf: [0; _], fd, exhausted: false })
    }
}

const BUF_SIZE: usize = 4096;

pub struct FileLines {
    buf: [u8; BUF_SIZE],
    fd: c_int,
    exhausted: bool,
}

impl Drop for FileLines {
    fn drop(&mut self) {
        // SAFETY: if FileLines is constructed (in [`open_lines`] only), the fd is valid
        unsafe { libc::close(self.fd) };
    }
}

impl FileLines {
    // This is a lending iterator, the standard Iterator trait doesn't support that :^(

    pub fn next(&mut self) -> Option<&[u8]> {
        if self.exhausted {
            return None;
        }

        for i in 0..BUF_SIZE {
            let mut byte: u8 = 0;
            let byte_ptr: *mut u8 = &mut byte;

            // SAFETY: preconditions of libc::read obviously met
            let count = unsafe { libc::read(self.fd, byte_ptr as _, 1) }; // TODO: this is dumb

            if byte == b'\n' {
                return Some(&self.buf[0..i]);
            } else if count == 0 {
                self.exhausted = true;
                if i == 0 {
                    return None;
                } else {
                    return Some(&self.buf[0..i]);
                }
            } else {
                self.buf[i] = byte;
            }
        }
        panic!("exceeded buffer size when reading fd {}", self.fd);
    }
}

//--------------------------
// MysteryStr implementation
//

/// String that is Either a Rustful UTF-8 encoded string with possibly null bytes,
/// or a C string that can contain whatever bytes (but most probably ASCII), and
/// can only have one null byte, the terminating one.
#[derive(Debug)]
#[allow(dead_code)]
pub enum MysteryStr<'a> {
    Rusty(&'a str),
    Crusty(&'a CStr),
}

impl<'a> From<&'a str> for MysteryStr<'a> {
    fn from(s: &'a str) -> Self {
        Self::Rusty(s)
    }
}

impl<'a> From<&'a CStr> for MysteryStr<'a> {
    fn from(s: &'a CStr) -> Self {
        Self::Crusty(s)
    }
}
