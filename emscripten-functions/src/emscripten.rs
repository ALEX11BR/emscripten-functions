use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    fmt::Display,
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

/// Sets the given function as the main loop of the calling thread, using the emscripten-defined `emscripten_set_main_loop`.
/// The given function has no parameters.
///
/// The main loop can be cancelled using the `cancel_main_loop` function.
///
/// # Arguments
/// * `func` - The function to be set as main event loop for the calling thread.
/// * `fps` - The number of calls of the function per second.
///   If set to a value <= 0, the browser's `requestAnimationFrame` function will be used (recommended when using the main function for rendering) instead of a fixed rate.
/// * `simulate_infinite_loop` - If `true`, no code after the function call will be executed, otherwise the code after the function call will be executed.
pub fn set_main_loop<F>(mut func: F, fps: c_int, simulate_infinite_loop: bool)
where
    F: 'static + FnMut(),
{
    set_main_loop_with_arg(move |_| func(), (), fps, simulate_infinite_loop);
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

/// Pauses the main loop of the calling thread.
pub fn pause_main_loop() {
    unsafe {
        emscripten::emscripten_pause_main_loop();
    }
}

/// Resumes the main loop of the calling thread.
pub fn resume_main_loop() {
    unsafe {
        emscripten::emscripten_resume_main_loop();
    }
}

/// Parameters of the main loop's scheduling mode.
///
/// While emscripten implements this using 2 `int` variables: `mode` and `value`; we put here only the valid modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainLoopTiming {
    /// The main loop function gets called periodically using `setTimeout()`, with the payload being the interval between the calls of `setTimeout()`.
    SetTimeout(c_int),
    /// The main loop function gets called using `requestAnimationFrame()`, with the payload being the "swap interval" rate for the main loop:
    /// * if the payload is `1`, the loop function gets called at every vsync (60fps for the common 60Hz display, 120fps for a 120Hz display, etc.),
    /// * if the payload is `2`, the loop function gets called every second vsync (that usually means 30fps, depends on the display),
    /// * in general, the rate is `{display frequency}/{the payload}` fps.
    RequestAnimationFrame(c_int),
    /// The main loop function gets called using `setImmediate()`, a function only available in Legacy Edge and partially in node.js.
    /// While the said function can be emulated using `postMessage()`, this mode of running the main loop is discouraged by the Emscripten devs.
    SetImmediate,
}

/// Applies the given main loop timing parameters to the main loop.
///
/// It returns:
/// * `true` if the main loop function is set.
/// * `false` if the main loop function isn't set.
///
/// If you're only interested in seeing whether the main loop function is set or not, check out the `is_main_loop_set` function.
///
/// # Arguments
/// * `timing` - the timing parameters to apply to the main loop.
pub fn set_main_loop_timing(timing: &MainLoopTiming) -> bool {
    let (mode, value) = match timing {
        MainLoopTiming::SetTimeout(ms) => (emscripten::EM_TIMING_SETTIMEOUT, *ms),
        MainLoopTiming::RequestAnimationFrame(n) => (emscripten::EM_TIMING_RAF, *n),
        MainLoopTiming::SetImmediate => (emscripten::EM_TIMING_SETIMMEDIATE, 0),
    };

    unsafe { emscripten::emscripten_set_main_loop_timing(mode as c_int, value) == 0 }
}

/// The main loop timing parameters can be set to other values than the ones with valid modes.
/// If this is the case and we want to retrieve them, we use this error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainLoopInvalidTiming {
    mode: c_int,
    value: c_int,
}
impl Display for MainLoopInvalidTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid main loop timing parameters: mode = {}, value = {}",
            self.mode, self.value
        )
    }
}

/// Returns the main loop timing parameters of the main loop.
///
/// If the parameters have an invalid mode, an error with the found parameters is returned instead.
pub fn get_main_loop_timing() -> Result<MainLoopTiming, MainLoopInvalidTiming> {
    let mut mode: c_int = 0;
    let mut value: c_int = 0;

    unsafe {
        emscripten::emscripten_get_main_loop_timing(&mut mode, &mut value);
    }

    match mode as u32 {
        emscripten::EM_TIMING_SETTIMEOUT => Ok(MainLoopTiming::SetTimeout(value)),
        emscripten::EM_TIMING_RAF => Ok(MainLoopTiming::RequestAnimationFrame(value)),
        emscripten::EM_TIMING_SETIMMEDIATE => Ok(MainLoopTiming::SetImmediate),
        _ => Err(MainLoopInvalidTiming { mode, value }),
    }
}

/// It returns:
/// * `true` if the main loop function is set.
/// * `false` if the main loop function isn't set.
pub fn is_main_loop_set() -> bool {
    // This is done by setting the main loop timing to the values that it already is,
    // and using the `emscripten_set_main_loop_timing` return value to see
    // if the main loop function is set.
    let mut mode = 0;
    let mut value = 0;

    unsafe {
        emscripten::emscripten_get_main_loop_timing(&mut mode, &mut value);
        emscripten::emscripten_set_main_loop_timing(mode, value) == 0
    }
}

/// Exits the program immediately while keeping the runtime alive, using `emscripten_exit_with_live_runtime`.
pub fn exit_with_live_runtime() {
    unsafe {
        emscripten::emscripten_exit_with_live_runtime();
    }
}

/// Exits the program and kills the runtime, using `emscripten_force_exit`.
/// Like libc's `exit`, but works even if `exit_with_live_runtime` was run.
///
/// Only works if the project is built with `EXIT_RUNTIME` set - this is not the default.
/// Build with `-sEXIT_RUNTIME` if you want to use this function.
///
/// # Arguments
/// * `status` - the exit status, the same as for libc's `exit`.
pub fn force_exit(status: c_int) {
    unsafe {
        emscripten::emscripten_force_exit(status);
    }
}

/// Returns the value of `window.devicePixelRatio`, using the emscripten-defined `emscripten_get_device_pixel_ratio`.
pub fn get_device_pixel_ratio() -> f64 {
    unsafe { emscripten::emscripten_get_device_pixel_ratio() }
}

/// Returns the window title, using the emscripten-defined `emscripten_get_window_title`.
pub fn get_window_title() -> String {
    let title = unsafe { emscripten::emscripten_get_window_title() };

    let title_cstr = unsafe { CStr::from_ptr(title) };
    title_cstr.to_str().unwrap().to_string()
}

/// Sets the window title, using the emscripten-defined `emscripten_set_window_title`.
///
/// # Arguments
/// * `title` - The new title
pub fn set_window_title<T>(title: T)
where
    T: AsRef<str>,
{
    let title = CString::new(title.as_ref()).unwrap();

    unsafe {
        emscripten::emscripten_set_window_title(title.as_ptr());
    }
}

/// The result of the `get_screen_size` function.
///
/// Implements `Display` as `{width}x{height}`.
/// Useful for writing the screen size easily, e.g. using `.to_string()` in your app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenSize {
    pub width: c_int,
    pub height: c_int,
}
impl Display for ScreenSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

/// Returns the width and height of the screen in a nice `ScreenSize` structure.
pub fn get_screen_size() -> ScreenSize {
    let mut width = 0;
    let mut height = 0;

    unsafe {
        emscripten::emscripten_get_screen_size(&mut width, &mut height);
    }

    return ScreenSize { width, height };
}

/// Hides the OS mouse cursor over the canvas, unlike SDL's `SDL_ShowCursor`, which works with the SDL cursor.
///
/// Useful if you draw your own cursor.
pub fn hide_mouse() {
    unsafe {
        emscripten::emscripten_hide_mouse();
    }
}

/// Returns the representation of the current app running time with the highest precision using the emscripten-defined `emscripten_get_now`.
/// It is most likely implemented using `performance.now()`, and is relevant only in comparison with other calls to this function.
pub fn get_now() -> f64 {
    unsafe { emscripten::emscripten_get_now() }
}

/// Returns a random number in range [0,1), with `Math.random()`, using the emscripten-defined `emscripten_random`.
pub fn random() -> f32 {
    unsafe { emscripten::emscripten_random() }
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
