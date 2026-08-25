use std::ffi::c_void;

type size_t = usize;
type ssize_t = isize;
pub type off_t = i64; // may vary by platform; see note below

// NOTE: real signature can differ by OS; adjust if your system headers differ.
unsafe extern "C" {
    pub unsafe fn getdirentries(fd: i32, buf: *mut c_void, nbytes: size_t, basep: *mut off_t) -> ssize_t;
}
