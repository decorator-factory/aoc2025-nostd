#![allow(dead_code)]

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

#[allow(unused_macros)]
macro_rules! printf {
    ($($arg:tt)*) => {
        $crate::prelude::printf_impl(::core::format_args!($($arg)*)).unwrap();
    };
}
#[cfg(not(test))]
pub(crate) use printf;
#[cfg(test)]
pub(crate) use std::print as printf;

#[allow(unused_macros)]
macro_rules! debugf {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            printf!($($arg)*);
        }
    };
}
#[allow(unused_imports)]
pub(crate) use debugf;

macro_rules! eprintf {
    ($($arg:tt)*) => {
        $crate::prelude::eprintf_impl(::core::format_args!($($arg)*));
    };
}
pub(crate) use eprintf;

#[allow(unused_macros)]
macro_rules! fdprintf {
    ($dst:expr, $($arg:tt)*) => {
        $crate::prelude::fdprintf_impl($expr, ::core::format_args!($($arg)*)).unwrap();
    };
}
#[allow(unused_imports)]
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

pub fn get_nanos() -> i64 {
    let mut timespec = MaybeUninit::<libc::timespec>::uninit();
    let rv = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, timespec.as_mut_ptr()) };
    assert!(rv != -1, "failed to get time: {:?}", unsafe {
        strerror_leak(get_errno().unwrap_unchecked())
    });
    let timespec = unsafe { timespec.assume_init() };
    timespec.tv_sec * 1_000_000_000 + timespec.tv_nsec
}

// it's supposed to be unsafe but whatever
// assumes seconds is zero
pub fn get_nanos_unchecked() -> i64 {
    let mut timespec = MaybeUninit::<libc::timespec>::uninit();
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, timespec.as_mut_ptr()) };
    let timespec = unsafe { timespec.assume_init() };
    timespec.tv_nsec
}

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
        )
    };
    unsafe {
        libc::madvise(start_ptr, file_length, libc::MADV_WILLNEED | libc::MADV_SEQUENTIAL);
    }

    Ok(unsafe { OwnedMmap::new(start_ptr.cast(), file_length) })
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

pub struct Malloc {
    ptr: *mut u8,
    length: usize,
}

impl Malloc {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.length) }
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.length) }
    }

    /// # Safety
    ///
    pub unsafe fn uninit(length: usize) -> Self {
        unsafe {
            let ptr: *mut u8 = libc::malloc(length).cast();
            assert!(!ptr.is_null(), "download more RAM");
            Self { ptr, length }
        }
    }

    pub fn copy_from(&mut self, src: &[u8]) {
        assert!(
            src.len() >= self.length,
            "src is of length: {}, but our memory is of length: {}",
            src.len(),
            self.length
        );

        unsafe {
            libc::memcpy(self.ptr.cast(), src.as_ptr().cast(), self.length);
        }
    }

    pub fn zeroed(length: usize) -> Self {
        unsafe {
            let ptr: *mut u8 = libc::calloc(length, 1).cast();
            assert!(!ptr.is_null(), "download more RAM");
            Self { ptr, length }
        }
    }

    pub fn clone(bytes: &[u8]) -> Self {
        let mut mem = unsafe { Self::uninit(bytes.len()) };
        mem.copy_from(bytes);
        mem
    }
}

pub struct Boks<T> {
    ptr: *mut T,
}

impl<T> Boks<T> {
    pub fn new(value: T) -> Boks<T> {
        const { assert!(core::mem::size_of::<T>() != 0, "we don't support ZSTs for now") };
        let ptr = unsafe { libc::malloc(core::mem::size_of::<T>()) } as *mut T;
        unsafe { core::ptr::write(ptr.cast(), value) };
        Self { ptr }
    }

    pub fn deref(self) -> T {
        unsafe { core::ptr::read(self.ptr.cast_const()) }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref_unchecked() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut_unchecked() }
    }
}

#[cfg(test)]
mod test_box {
    use super::Boks;

    #[test]
    fn simple() {
        let b = Boks::new(69);
        assert!(!b.ptr.is_null());
        assert_eq!(b.as_ref(), &69);
        assert_eq!(b.deref(), 69);
    }
}

pub struct ArrayVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    length: usize,
}

impl<T, const N: usize> core::fmt::Debug for ArrayVec<T, N>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ArrayVec[")?;
        let mut iter = self.as_slice().iter().peekable();
        while let Some(x) = iter.next() {
            f.write_fmt(format_args!("{:?}", x))?;

            if iter.peek().is_some() {
                f.write_str(", ")?;
            }
        }
        f.write_str("]")
    }
}

impl<T, const N: usize> ArrayVec<T, N> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self { data: [const { MaybeUninit::uninit() }; N], length: 0 }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline(always)]
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.length >= N {
            Err(value)
        } else {
            self.data[self.length] = MaybeUninit::new(value);
            self.length += 1;
            Ok(())
        }
    }

    #[inline(always)]
    pub const unsafe fn zeroed() -> Self {
        Self { data: [const { MaybeUninit::zeroed() }; N], length: N }
    }

    #[inline(always)]
    pub const unsafe fn set_len(&mut self, length: usize) {
        self.length = length;
    }

    #[inline(always)]
    pub fn push(&mut self, value: T) {
        self.try_push(value).unwrap_or_else(|_| panic!("ArrayVec<_, {}> overflow", N));
    }

    #[inline(always)]
    pub unsafe fn push_unchecked(&mut self, value: T) {
        self.data[self.length] = MaybeUninit::new(value);
        self.length += 1;
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        let start = self.data.as_ptr();
        unsafe { slice::from_raw_parts(start.cast(), self.length) }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let start = self.data.as_mut_ptr();
        unsafe { slice::from_raw_parts_mut(start.cast(), self.length) }
    }
}

impl<T, const N: usize> ArrayVec<T, N>
where
    T: Copy,
{
    pub fn clear(&mut self) {
        self.length = 0;
    }
}

#[cfg(test)]
mod array_vec_tests {
    use super::ArrayVec;

    #[test]
    fn iterate() {
        let mut foo = ArrayVec::<u8, 10>::new();
        foo.push(100);
        foo.push(20);
        foo.push(0);
        foo.push(69);
        let mut things = [1u8; 4];
        for (i, &x) in foo.as_slice().iter().enumerate() {
            things[i] = x;
        }
        assert_eq!(things, [100, 20, 0, 69]);
    }

    #[test]
    fn thanos_snap() {
        let mut foo = ArrayVec::<u8, 10>::new();
        foo.push(100);
        foo.push(20);
        foo.push(0);
        foo.push(69);
        assert_eq!(foo.as_slice(), &[100, 20, 0, 69]);
        foo.clear();
        assert_eq!(foo.as_slice(), &[]);
    }
}

impl<T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        for uninit in self.data.iter_mut().take(self.length) {
            unsafe { uninit.assume_init_drop() };
        }
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
