use rdsys;

use libc::c_char;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Return a tuple representing the version of `librdkafka` in
/// hexadecimal and string format.
pub fn get_rdkafka_version() -> (u16, String) {
    let version_number = unsafe { rdsys::rd_kafka_version() } as u16;
    let c_str = unsafe { CStr::from_ptr(rdsys::rd_kafka_version_str()) };
    (version_number, c_str.to_string_lossy().into_owned())
}

/// Converts a Duration into milliseconds
pub fn duration_to_millis(duration: Duration) -> u64 {
    let nanos = duration.subsec_nanos() as u64;
    duration.as_secs() * 1000 + nanos/1_000_000
}

/// Converts the given time to milliseconds since unix epoch.
pub fn millis_to_epoch(time: SystemTime) -> i64 {
    let duration_since_epoch = time.duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    duration_to_millis(duration_since_epoch) as i64
}

/// Returns the current time in millis since unix epoch.
pub fn current_time_millis() -> i64 {
    millis_to_epoch(SystemTime::now())
}


/// A trait for the conversion of Rust data to raw pointers. This conversion is used
/// to pass opaque objects to the C library and vice versa.
pub trait IntoOpaque: Send + Sync {
    /// Converts the object into a raw pointer.
    fn into_ptr(self) -> *mut c_void;
    /// Converts the raw pointer back to the original Rust object.
    unsafe fn from_ptr(*mut c_void) -> Self;
}

impl IntoOpaque for () {
    fn into_ptr(self) -> *mut c_void {
        ptr::null_mut()
    }

    unsafe fn from_ptr(_: *mut c_void) -> Self {
        ()
    }
}

impl IntoOpaque for usize {
    fn into_ptr(self) -> *mut c_void {
        self as *mut c_void
    }

    unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        ptr as usize
    }
}

impl<T: Send + Sync> IntoOpaque for Box<T> {
    fn into_ptr(self) -> *mut c_void {
        Box::into_raw(self) as *mut c_void
    }

    unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Box::from_raw(ptr as *mut T)
    }
}

impl<T: IntoOpaque> IntoOpaque for Option<T> {
    fn into_ptr(self) -> *mut c_void {
        match self {
            Some(x) => x.into_ptr(),
            None => ptr::null_mut(),
        }
    }

    unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        if ptr.is_null() {
            None
        } else {
            Some(T::from_ptr(ptr))
        }
    }
}


// TODO: check if the implementation returns a copy of the data and update the documentation
/// Converts a byte array representing a C string into a String.
pub unsafe fn bytes_cstr_to_owned(bytes_cstr: &[c_char]) -> String {
    CStr::from_ptr(bytes_cstr.as_ptr()).to_string_lossy().into_owned()
}

/// Converts a C string into a String.
pub unsafe fn cstr_to_owned(cstr: *const c_char) -> String {
    CStr::from_ptr(cstr).to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_duration_to_millis() {
        assert_eq!(duration_to_millis(Duration::from_secs(1)), 1000);
        assert_eq!(duration_to_millis(Duration::from_millis(1500)), 1500);
        assert_eq!(duration_to_millis(Duration::new(5, 123_000_000)), 5123);
    }
}

