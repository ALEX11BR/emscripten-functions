use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    os::raw::c_int,
};

use emscripten_sys::emscripten;

// The function to run in `set_main_loop_with_arg` sits in this thread-local object so that it will remain permanent throughout the main loop's run.
// It needs to stay in a global place so that the `wrapper_func` that is passed as argument to `emscripten_set_main_loop`, which must be an `extern "C"` function, can access it (it couldn't have been a closure).
// As the `thread_local` thing only gives us an immutable reference, we use a `RefCell` to be able to change the data when the function gets called.
thread_local! {
    static MAIN_LOOP_FUNCTION: RefCell<Option<Box<dyn FnMut()>>> = RefCell::new(None);
}

/// Sets the given function as the main loop of the calling thread, using the emscripten-defined `emscripten_set_main_loop`.
/// The given function accepts a mutable reference to the variable `arg` that may contain the loop state and whatever is needed for it to run.
/// 
/// The main loop can be cancelled using the `cancel_main_loop` function.
///
/// # Arguments
/// * `func` - The function to be set as main event loop for the calling thread.
/// * `arg` - The variable that represents the state that the main event loop ought to interact with.
///   It will be consumed so that it can be kept alive during the loop.
///   It must be `'static`, that means `arg`'s type should only contain owned data and `'static` references, if any.
/// * `fps` - The number of calls of the function per second.
///   If set to a value <= 0, the browser's `requestAnimationFrame` function will be used (recommended when using the main function for rendering) instead of a fixed rate.
/// * `simulate_infinite_loop` - If `true`, no code after the function call will be executed, otherwise the code after the function call will be executed.
pub fn set_main_loop_with_arg<F, T>(
    mut func: F,
    mut arg: T,
    fps: c_int,
    simulate_infinite_loop: bool,
) where
    F: 'static + FnMut(&mut T),
    T: 'static,
{
    // In `MAIN_LOOP_FUNCTION` we store a closure with no arguments, so that its type would be independent of `T`.
    // That closure calls the `func` parameter with `arg` as parameter, and owns them both.
    MAIN_LOOP_FUNCTION.with(|func_ref| {
        *func_ref.borrow_mut() = Some(Box::new(move || {
            func(&mut arg);
        }));
    });

    unsafe extern "C" fn wrapper_func() {
        MAIN_LOOP_FUNCTION.with(|func_ref| {
            if let Some(function) = &mut *func_ref.borrow_mut() {
                (*function)();
            }
        });
    }

    unsafe {
        emscripten::emscripten_set_main_loop(Some(wrapper_func), fps, simulate_infinite_loop as i32)
    };
}

/// Cancels the main loop of the calling thread that was set using `set_main_loop_with_arg`.
pub fn cancel_main_loop() {
    unsafe {
        emscripten::emscripten_cancel_main_loop();
    }

    // Also let's not forget to free up the main loop function and its state arg.
    MAIN_LOOP_FUNCTION.with(|func_ref| {
        *func_ref.borrow_mut() = None;
    });
}

/// Runs the given JavaScript script string using the `eval()` function,
/// using the emscripten-defined `emscripten_run_script`.
///
/// # Arguments
/// * `script` - The script to execute.
pub fn run_script<T>(script: T)
where
    T: AsRef<str>,
{
    let script_cstring = CString::new(script.as_ref()).unwrap();
    unsafe { emscripten::emscripten_run_script(script_cstring.as_ptr()) }
}

/// Runs the given JavaScript script string using the `eval()` function,
/// using the emscripten-defined `emscripten_run_script_int`.
/// It returns the return result of the script, interpreted as a C int.
/// Most probably the return result is passed to `parseInt()`, with NaN represented as 0.
///
/// # Arguments
/// * `script` - The script to execute.
pub fn run_script_int<T>(script: T) -> c_int
where
    T: AsRef<str>,
{
    let script_cstring = CString::new(script.as_ref()).unwrap();
    unsafe { emscripten::emscripten_run_script_int(script_cstring.as_ptr()) }
}

/// Runs the given JavaScript script string using the `eval()` function,
/// using the emscripten-defined `emscripten_run_script_string`.
/// It returns the return result of the script, interpreted as a string if possible.
/// Otherwise, it returns None.
///
/// # Arguments
/// * `script` - The script to execute.
pub fn run_script_string<T>(script: T) -> Option<String>
where
    T: AsRef<str>,
{
    let script_cstring = CString::new(script.as_ref()).unwrap();
    let result = unsafe { emscripten::emscripten_run_script_string(script_cstring.as_ptr()) };

    if result.is_null() {
        return None;
    }

    let result_cstr = unsafe { CStr::from_ptr(result) };
    Some(result_cstr.to_str().unwrap().to_string())
}
