use std::ffi::CString;

use emscripten_sys::console;

/// Prints the given string using `console.log()`
/// 
/// # Arguments
/// * `string` - The string to print.
pub fn log<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_console_log(cstring.as_ptr());
    }
}

/// Prints the given string using `console.warn()`
/// 
/// # Arguments
/// * `string` - The string to print.
pub fn warn<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_console_warn(cstring.as_ptr());
    }
}

/// Prints the given string using `console.error()`
/// 
/// # Arguments
/// * `string` - The string to print.
pub fn error<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_console_error(cstring.as_ptr());
    }
}

/// Prints the given string using `out()`
/// 
/// # Arguments
/// * `string` - The string to print.
pub fn out<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_out(cstring.as_ptr());
    }
}

/// Prints the given string using `err()`
/// 
/// # Arguments
/// * `string` - The string to print.
pub fn err<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_err(cstring.as_ptr());
    }
}

/// Prints the given string using `dbg()`
/// 
/// # Arguments
/// * `string` - The string to print.
pub fn dbg<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_dbg(cstring.as_ptr());
    }
}
