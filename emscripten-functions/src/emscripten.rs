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

/// Sets the given function as the main loop of the calling thread, using the emscripten-defined [`emscripten_set_main_loop`].
/// The given function accepts a mutable reference (argument `arg`) to the variable that will contain the loop state and whatever else is needed for it to run.
///
/// If you don't need that state argument, check out [`set_main_loop`].
/// 
/// The main loop can be cancelled using the [`cancel_main_loop`] function.
/// 
/// [`emscripten_set_main_loop`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_set_main_loop
///
/// # Arguments
/// * `func` - The function to be set as main event loop for the calling thread.
/// * `arg` - The variable that represents the state that the main event loop ought to interact with.
///   It will be consumed so that it can be kept alive during the loop.
///   It must be `'static`, that means `arg`'s type should only contain owned data and `'static` references, if any.
/// * `fps` - The number of calls of the function per second.
///   If set to a value <= 0, the browser's [`requestAnimationFrame()`] function will be used (recommended when using the main function for rendering) instead of a fixed rate.
/// * `simulate_infinite_loop` - If `true`, no code after the function call will be executed, otherwise the code after the function call will be executed.
/// 
/// [`requestAnimationFrame()`]: https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame
/// 
/// # Examples
/// ```rust
/// struct GameData {
///     level: u32,
///     score: u32
/// }
/// let mut game_data = GameData {
///     level: 1,
///     score: 0
/// }
/// 
/// set_main_loop_with_arg(|data| {
///     if score < level {
///         score += 1;
///     } else {
///         score = 0;
///         level += 1;
///     }
/// }, game_data, 0, true);
/// ```
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

/// Sets the given function as the main loop of the calling thread, using the emscripten-defined [`emscripten_set_main_loop`].
/// The given function has no parameters.
///
/// The main loop can be cancelled using the [`cancel_main_loop`] function.
///
/// [`emscripten_set_main_loop`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_set_main_loop
/// 
/// # Arguments
/// * `func` - The function to be set as main event loop for the calling thread.
/// * `fps` - The number of calls of the function per second.
///   If set to a value <= 0, the browser's [`requestAnimationFrame()`] function will be used (recommended when using the main function for rendering) instead of a fixed rate.
/// * `simulate_infinite_loop` - If `true`, no code after the function call will be executed, otherwise the code after the function call will be executed.
/// 
/// [`requestAnimationFrame()`]: https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame
/// 
/// # Examples
/// ```rust
/// set_main_loop(|| {
///     println!("Hello world every half second!");
/// }, 2, true);
/// ```
pub fn set_main_loop<F>(mut func: F, fps: c_int, simulate_infinite_loop: bool)
where
    F: 'static + FnMut(),
{
    set_main_loop_with_arg(move |_| func(), (), fps, simulate_infinite_loop);
}

/// Cancels the main loop of the calling thread that was set using [`set_main_loop_with_arg`] or [`set_main_loop`].
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
/// If you're only interested in seeing whether the main loop function is set or not, check out the [`is_main_loop_set`] function.
///
/// # Arguments
/// * `timing` - the timing parameters to apply to the main loop.
/// 
/// # Examples
/// ```rust
/// set_main_loop_timing(MainLoopTiming::SetTimeout(33));
/// // the main function will now run at ~30fps
/// ```
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
/// 
/// # Examples
/// ```rust
/// match get_main_loop_timing() {
///     Ok(MainLoopTiming::SetTimeout(ms)) => {
///         println!("It runs every {} ms", ms);
///     }
///     Ok(MainLoopTiming::RequestAnimationFrame(_)) => {
///         println!("You render stuff as you should");
///     }
///     Ok(MainLoopTiming::SetImmediate) => {
///         println!("Why are you doing this???");
///     }
///     Err(err) => {
///         println!("What??? {}", err);
///     }
/// };
/// ```
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
/// 
/// # Examples
/// ```rust
/// if is_main_loop_set() {
///     println!("It's set. It may be paused, but that should be pretty rare.");
/// }
/// ```
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

/// Exits the program immediately while keeping the runtime alive, using [`emscripten_exit_with_live_runtime`].
/// 
/// [`emscripten_exit_with_live_runtime`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_exit_with_live_runtime
pub fn exit_with_live_runtime() {
    unsafe {
        emscripten::emscripten_exit_with_live_runtime();
    }
}

/// Exits the program and kills the runtime, using [`emscripten_force_exit`].
/// Like libc's [`exit`], but works even if [`exit_with_live_runtime`] was run.
///
/// Only works if the project is built with `EXIT_RUNTIME` set - this is not the default.
/// Build with `-sEXIT_RUNTIME` if you want to use this function.
///
/// [`emscripten_force_exit`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_force_exit
/// [`exit`]: https://linux.die.net/man/3/exit
/// 
/// # Arguments
/// * `status` - the exit status, the same as for libc's `exit`.
/// 
/// # Examples
/// ```rust
/// force_exit(0); // Exits with status 0.
/// ```
/// ```rust
/// force_exit(1); // Exits with status 1.
/// ```
/// ```rust
/// force_exit(101); // Exits with status 101.
/// ```
pub fn force_exit(status: c_int) {
    unsafe {
        emscripten::emscripten_force_exit(status);
    }
}

/// Returns the value of [`window.devicePixelRatio`], using the emscripten-defined [`emscripten_get_device_pixel_ratio`].
/// 
/// [`window.devicePixelRatio`]: https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio
/// [`emscripten_get_device_pixel_ratio`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_get_device_pixel_ratio
/// 
/// # Examples
/// ```rust
/// println!("Your device pixel ratio is {}", get_device_pixel_ratio());
/// ```
pub fn get_device_pixel_ratio() -> f64 {
    unsafe { emscripten::emscripten_get_device_pixel_ratio() }
}

/// Returns the window title, using the emscripten-defined [`emscripten_get_window_title`].
/// 
/// [`emscripten_get_window_title`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_get_window_title
/// 
/// # Examples
/// ```rust
/// println!("Your tab title should be '{}'", get_window_title());
/// ```
pub fn get_window_title() -> String {
    let title = unsafe { emscripten::emscripten_get_window_title() };

    let title_cstr = unsafe { CStr::from_ptr(title) };
    title_cstr.to_str().unwrap().to_string()
}

/// Sets the window title, using the emscripten-defined [`emscripten_set_window_title`].
///
/// [`emscripten_set_window_title`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_set_window_title
/// 
/// # Arguments
/// * `title` - The new title
/// 
/// # Examples
/// ```rust
/// set_window_title("My Web App");
/// set_window_title(format!("My app {} u", 3 + 1));
/// ```
pub fn set_window_title<T>(title: T)
where
    T: AsRef<str>,
{
    let title = CString::new(title.as_ref()).unwrap();

    unsafe {
        emscripten::emscripten_set_window_title(title.as_ptr());
    }
}

/// The result of the [`get_screen_size`] function.
///
/// Implements [`Display`] as `{width}x{height}`.
/// Useful for writing the screen size easily, e.g. using [`.to_string()`] in your app.
/// 
/// [`Display`]: https://doc.rust-lang.org/std/fmt/trait.Display.html
/// [`.to_string()`]: https://doc.rust-lang.org/std/string/trait.ToString.html#tymethod.to_string
/// 
/// # Examples
/// ```rust
/// let screen_size = get_screen_size();
/// 
/// // This should display something like: "Your screen size is 800x600"
/// println!("Your screen size is {}", screen_size);
/// 
/// let an_old_size = ScreenSize {
///     width: 800,
///     height: 600
/// };
/// if an_old_size == screen_size {
///     println!("Wow, you have the 'old' size of 800x600");
/// }
/// 
/// println!("You have {} pixels", screen_size.width * screen_size.height);
/// ```
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

/// Returns the width and height of the screen in a nice [`ScreenSize`] structure.
/// 
/// # Examples
/// ```rust
/// let screen_size = get_screen_size();
/// 
/// // This should display something like: "Your screen size is 800x600"
/// println!("Your screen size is {}", screen_size);
/// 
/// println!("You have {} pixels", screen_size.width * screen_size.height);
/// ```
pub fn get_screen_size() -> ScreenSize {
    let mut width = 0;
    let mut height = 0;

    unsafe {
        emscripten::emscripten_get_screen_size(&mut width, &mut height);
    }

    return ScreenSize { width, height };
}

/// Hides the OS mouse cursor over the canvas, unlike SDL's [`SDL_ShowCursor`], which works with the SDL cursor.
///
/// Useful if you draw your own cursor.
/// 
/// [`SDL_ShowCursor`]: https://wiki.libsdl.org/SDL2/SDL_ShowCursor
pub fn hide_mouse() {
    unsafe {
        emscripten::emscripten_hide_mouse();
    }
}

/// Returns the representation of the current app running time with the highest precision using the emscripten-defined [`emscripten_get_now`].
/// It is most likely implemented using [`performance.now()`], and is relevant only in comparison with other calls to this function.
/// 
/// [`emscripten_get_now`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_get_now
/// [`performance.now()`]: https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
/// 
/// # Examples
/// ```rust
/// let start_time = get_now();
/// 
/// let x = 0;
/// for i in 1..100 {
///     x += i;
/// }
/// 
/// println!("It took {} seconds", get_now() - start_time);
/// ```
pub fn get_now() -> f64 {
    unsafe { emscripten::emscripten_get_now() }
}

/// Returns a random number in range [0,1), with [`Math.random()`], using the emscripten-defined [`emscripten_random`].
/// 
/// [`Math.random()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/random
/// [`emscripten_random`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_random
/// 
/// # Examples
/// ```rust
/// assert!(random() >= 0);
/// assert!(random() < 1);
/// ```
pub fn random() -> f32 {
    unsafe { emscripten::emscripten_random() }
}

/// Runs the given JavaScript script string with the [`eval()`] JS function,
/// using the emscripten-defined [`emscripten_run_script`].
/// 
/// [`eval()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval
/// [`emscripten_run_script`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_run_script
///
/// # Arguments
/// * `script` - The script to execute.
/// 
/// # Examples
/// ```rust
/// run_script("alert('Hello world')");
/// 
/// // The `.escape_unicode()` method makes it safe to pass untrusted user input.
/// run_script(
///     format!(
///         r##"
///             document.querySelector("#this-is-secure").innerHTML = "{}"
///         "##, 
///         "untrusted user input".escape_unicode()
///     )
/// );
/// ```
pub fn run_script<T>(script: T)
where
    T: AsRef<str>,
{
    let script_cstring = CString::new(script.as_ref()).unwrap();
    unsafe { emscripten::emscripten_run_script(script_cstring.as_ptr()) }
}

/// Runs the given JavaScript script string with the [`eval()`] JS function,
/// using the emscripten-defined [`emscripten_run_script_int`].
/// It returns the return result of the script, interpreted as a C int.
/// Most probably the return result is passed to [`parseInt()`], with NaN represented as 0.
/// 
/// [`eval()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval
/// [`emscripten_run_script_int`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_run_script_int
/// [`parseInt()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseInt
///
/// # Arguments
/// * `script` - The script to execute.
/// 
/// # Examples
/// ```rust
/// assert_eq!(run_script_int("1 + 2"), 3);
/// ```
pub fn run_script_int<T>(script: T) -> c_int
where
    T: AsRef<str>,
{
    let script_cstring = CString::new(script.as_ref()).unwrap();
    unsafe { emscripten::emscripten_run_script_int(script_cstring.as_ptr()) }
}

/// Runs the given JavaScript script string with the [`eval()`] JS function,
/// using the emscripten-defined [`emscripten_run_script_string`].
/// It returns the return result of the script, interpreted as a string if possible.
/// Otherwise, it returns None.
///
/// [`eval()`]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/eval
/// [`emscripten_run_script_string`]: https://emscripten.org/docs/api_reference/emscripten.h.html#c.emscripten_run_script_string
/// 
/// # Arguments
/// * `script` - The script to execute.
/// 
/// # Examples
/// ```rust
/// assert_eq!(run_script_string("alert('hi')"), None);
/// assert_eq!(run_script_string("2"), Some("2"));
/// assert_eq!(run_script_string("'hi'"), Some("hi"));
/// ```
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
