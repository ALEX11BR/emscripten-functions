use std::{ffi::{CStr, CString}, os::raw::c_int};

use emscripten_sys::emscripten;

/// Runs the given JavaScript script string using the `eval()` function,
/// using the emscripten-defined `emscripten_run_script`.
/// 
/// # Arguments
/// * `script` - the script to execute
pub fn run_script<T>(script: T)
where
    T: AsRef<str>,
{
    let script_cstring = CString::new(script.as_ref()).unwrap();
    unsafe {
        emscripten::emscripten_run_script(script_cstring.as_ptr())
    }
}

/// Runs the given JavaScript script string using the `eval()` function,
/// using the emscripten-defined `emscripten_run_script_int`.
/// It returns the return result of the script, interpreted as a C int.
/// Most probably the return result is passed to `parseInt()`, with NaN represented as 0.
/// 
/// # Arguments
/// * `script` - the script to execute
pub fn run_script_int<T>(script: T) -> c_int
where
    T: AsRef<str>,
{
    let script_cstring = CString::new(script.as_ref()).unwrap();
    unsafe {
        emscripten::emscripten_run_script_int(script_cstring.as_ptr())
    }
}

/// Runs the given JavaScript script string using the `eval()` function,
/// using the emscripten-defined `emscripten_run_script_string`.
/// It returns the return result of the script, interpreted as a string if possible.
/// Otherwise, it returns None.
/// 
/// # Arguments
/// * `script` - the script to execute
pub fn run_script_string<T>(script: T) -> Option<String>
where
    T: AsRef<str>,
{
    let script_cstring = CString::new(script.as_ref()).unwrap();
    let result = unsafe {
        emscripten::emscripten_run_script_string(script_cstring.as_ptr())
    };

    if result.is_null() {
        return None;
    }

    let result_cstr = unsafe {
        CStr::from_ptr(result)
    };
    Some(result_cstr.to_str().unwrap().to_string())
}
