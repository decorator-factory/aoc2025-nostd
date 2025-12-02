#![expect(unused)]

use core::{
    ffi::{
        CStr,
        c_char,
        c_int,
        c_void,
    },
    mem::MaybeUninit,
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

pub fn errnah(header: &str) -> Result<(), &'static str> {
    if let Some(errno) = get_errno() {
        let explanation = strerror_leak(errno);
        let full_error = sprintf_leak!([16384] "{header}: {explanation:?}");
        Err(full_error)
    } else {
        Ok(())
    }
}

pub fn strerror_leak(errno: NonZero<c_int>) -> &'static CStr {
    const BUFFER_LEN: usize = 4096;
    let ptr = unsafe { libc::malloc(BUFFER_LEN) as *mut c_char };
    if ptr.is_null() {
        panic!("download more RAM")
    }
    if unsafe { libc::strerror_r(errno.into(), ptr, BUFFER_LEN) != 0 } {
        panic!("errorception")
    }

    // SAFETY: strerror return 0 means it succeeded, and now `ptr` points to a valid C string
    unsafe { CStr::from_ptr(ptr) }
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
    ([ $size:expr ] $($arg:tt)*) => {
        $crate::prelude::sprintf_leak_impl(::core::format_args!($($arg)*), $size).unwrap()
    };

    ($($arg:tt)*) => {
        $crate::prelude::sprintf_leak_impl(::core::format_args!($($arg)*), 4096).unwrap()
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

            unsafe {
                libc::memcpy(self.ptr as *mut c_void, s.as_ptr() as *const c_void, s.len())
            };

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

pub fn fdprintf_impl<'a>(
    fd: c_int,
    fmt: core::fmt::Arguments<'a>,
) -> Result<(), &'static CStr> {
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
pub fn open_mmap(path: &CStr) -> Result<OwnedMmap, &'static CStr> {
    let fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY) };
    if fd == -1 {
        return Err(strerror_leak(get_errno().unwrap()));
    }

    // SAFETY: path is a valid C string because we got it from a &CStr
    let stat = unsafe {
        let mut maybe_stat = MaybeUninit::uninit();
        if libc::stat(path.as_ptr(), maybe_stat.as_mut_ptr()) != 0 {
            return Err(strerror_leak(get_errno().unwrap()));
        }
        maybe_stat.assume_init()
    };
    let file_length = stat.st_size as usize;

    // SAFETY: passing null pointer is fine as per mmap documentation
    // Note: if someone edits the file and e.g. truncates it while we're reading
    //       it, we are going to get a SIBGUS from the operating system. That will
    //       crash the program, but as far as I can tell it's not undefined behaviour
    let start_ptr = unsafe {
        libc::mmap(
            core::ptr::null_mut(),
            file_length,
            libc::PROT_READ,
            libc::MAP_PRIVATE,
            fd,
            0i64,
        ) as *mut u8
    };

    Ok(unsafe { OwnedMmap::new(start_ptr, file_length) })
}

pub struct OwnedMmap {
    ptr: *mut u8,
    length: usize,
}

impl OwnedMmap {
    /// # Safety
    /// `ptr` must point to a valid memory mapped region that is `length`
    /// bytes long. When created, `FileLines` takes ownership of that region.
    unsafe fn new(ptr: *mut u8, length: usize) -> Self {
        Self { ptr, length }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.length) }
    }
}

impl Drop for OwnedMmap {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.ptr as *mut c_void, self.length) };
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

macro_rules! advent_of_code_impl {
    ($e:expr) => {
        const _: () = {
            #[unsafe(export_name = concat!("advent_of_code_solution__", file!()))]
            static DUMMY: $crate::prelude::DayFn = $e;
        };
    };
}
pub(crate) use advent_of_code_impl;

pub type DayFn = fn(path: &CStr) -> Result<(), MysteryStr<'static>>;

/// Find function solving a particular Advent of Code day.
/// The day must be a C string like "day1a" or "day12b".
pub(crate) fn fetch_day_solution(day: &CStr) -> Option<DayFn> {
    let handle = unsafe { libc::dlopen(core::ptr::null(), libc::RTLD_NOW) };
    assert!(!handle.is_null(), "We cannot open our own executable... strange");

    let mut symbol_name = [c_char::default(); 1024];
    let length = unsafe {
        libc::snprintf(
            symbol_name.as_mut_ptr(),
            1024,
            c"advent_of_code_solution__src/%s.rs".as_ptr(),
            day,
        )
    };
    if length <= 0 {
        return None;
    }

    // Exported symbols are actually static variables of type DayFn
    // So the symbol is a pointer to a function pointer.
    let day_fn_ptr: *const DayFn = unsafe { libc::dlsym(handle, symbol_name.as_ptr()).cast() };
    if day_fn_ptr.is_null() { None } else { Some(unsafe { *day_fn_ptr }) }
}
