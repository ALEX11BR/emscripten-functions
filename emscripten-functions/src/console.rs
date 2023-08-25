//! Select functions (with rust-native parameter types) from the emscripten [`console.h`] [header file].
//! 
//! [`console.h`]: https://github.com/emscripten-core/emscripten/blob/main/site/source/docs/api_reference/console.h.rst
//! [header file]: https://github.com/emscripten-core/emscripten/blob/main/system/include/emscripten/console.h

use std::ffi::CString;

use emscripten_sys::console;

/// Prints the given string using the [`console.log()`] JS function.
///
/// [`console.log()`]: https://developer.mozilla.org/en-US/docs/Web/API/console/log
///
/// # Arguments
/// * `string` - The string to print.
///
/// # Examples
/// ```rust
/// log("Hello, world!");
/// log(format!("0.1 + 0.2 = {}", 0.1 + 0.2));
/// ```
pub fn log<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_console_log(cstring.as_ptr());
    }
}

/// Prints the given string using the [`console.warn()`] JS function.
///
/// [`console.warn()`]: https://developer.mozilla.org/en-US/docs/Web/API/console/warn
///
/// # Arguments
/// * `string` - The string to print.
///
/// # Examples
/// ```rust
/// warn("Hello, world!");
/// warn(format!("0.1 + 0.2 = {}", 0.1 + 0.2));
/// ```
pub fn warn<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_console_warn(cstring.as_ptr());
    }
}

/// Prints the given string using the [`console.error()`] function.
///
/// [`console.error()`]: https://developer.mozilla.org/en-US/docs/Web/API/console/error
///
/// # Arguments
/// * `string` - The string to print.
///
/// # Examples
/// ```rust
/// error("Hello, world!");
/// error(format!("0.1 + 0.2 = {}", 0.1 + 0.2));
/// ```
pub fn error<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_console_error(cstring.as_ptr());
    }
}

/// Prints the given string using the emscripten-defined `out()` JS function.
///
/// # Arguments
/// * `string` - The string to print.
///
/// # Examples
/// ```rust
/// out("Hello, world!");
/// out(format!("0.1 + 0.2 = {}", 0.1 + 0.2));
/// ```
pub fn out<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_out(cstring.as_ptr());
    }
}

/// Prints the given string using the emscripten-defined `err()` JS function.
///
/// # Arguments
/// * `string` - The string to print.
///
/// # Examples
/// ```rust
/// err("Hello, world!");
/// err(format!("0.1 + 0.2 = {}", 0.1 + 0.2));
/// ```
pub fn err<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_err(cstring.as_ptr());
    }
}

/// Prints the given string using the emscripten-defined `dbg()` JS function.
///
/// # Arguments
/// * `string` - The string to print.
///
/// # Examples
/// ```rust
/// dbg("Hello, world!");
/// dbg(format!("0.1 + 0.2 = {}", 0.1 + 0.2));
/// ```
pub fn dbg<T>(string: T)
where
    T: AsRef<str>,
{
    let cstring = CString::new(string.as_ref()).unwrap();
    unsafe {
        console::emscripten_dbg(cstring.as_ptr());
    }
}
