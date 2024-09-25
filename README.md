# Crates for emscripten targets in rust

This project contains 2 crates with functions (and types too) to help rust development for emscripten targets.

## The crates

- `emscripten-functions-sys` - Raw bindgen-generated rust bindings to emscripten’s system functions.
- `emscripten-functions` - Various emscripten system functions that make programming in rust for emscripten targets easier.

## Why emscripten for rust

If you want to write web apps in rust, the `wasm32-unknown-unknown` target is the top choice, with a quite mature ecosystem of functions that interact with the web ecosystem.
That being said, if your project has parts or libraries written in C or C++, then the `wasm32-unknown-unknown` target doesn't work anymore.
Also, you might be interested in using [asm.js](https://en.wikipedia.org/wiki/Asm.js) instead of [WASM](https://en.wikipedia.org/wiki/WebAssembly).

Thankfully, there is an alternative: `wasm32-unknown-emscripten` (and `asmjs-unknown-emscripten` too).
Emscripten provides a ready-to-use libc for web apps, and a few other popular C libraries, like SDL.
Using the 2 crates, interacting with the web environment of emscripten becomes easier.

## Some tips

### Functions that can be triggered dynamically
- Make a function with `#[no_mangle] pub extern "C" fn function_name ...` (they can accept and return integers, floats,  C-style strings, booleans, pointers or arrays of byte-sized numbers),
- add `-sEXPORTED_RUNTIME_METHODS=ccall,cwrap` and `-sEXPORTED_FUNCTIONS=_function_name,...` (add a preceding underscore to your functions' names and separate them with commas) as link arguments,

... and you can call your functions using the [ccall/cwrap](https://emscripten.org/docs/porting/connecting_cpp_and_javascript/Interacting-with-code.html#interacting-with-code-ccall-cwrap) emscripten javascript functions.

#### Example

The rust part:
```rust
use std::ffi::{CStr, CString};
use std::mem::ManuallyDrop;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn concat_str_int_float(s: *const c_char, i: i32, f: f64) -> *const c_char {
    // s (char pointer) => string (rust &str)
    let string = if s.is_null() {
        ""
    } else {
        (unsafe { CStr::from_ptr(s) }).to_str().unwrap()
    };

    let result = format!("{}_{}_{:.2}", string, i, f);

    // result (rust String) => return value (char pointer)
    // The return value will be `free`d by the caller
    let result_cstring = ManuallyDrop::new(CString::new(result.as_bytes()).unwrap());
    return result_cstring.as_ptr();
}
```

The javascript part (to be put in a `<script>` after `<script src="your_project_name.js"></script>`):
```js
// As we need to `free` the pointer returned by our function, we need its raw address, so we'll consider the return type to be `number`.
let concat_str_int_float = Module.cwrap("concat_str_int_float", "number", ["string", "number", "number"])
// The function's signature decides if a `number` is an integer or a float.

let button = document.querySelector("button");
button.onclick = () => {
  let result_ptr = concat_str_int_float(document.title, performance.now(), performance.now())
  // `UTF8ToString` is an emscripten JS function which returns a proper JS garbage collected string.
  document.title = UTF8ToString(result_ptr)
  // We need to `free` our pointer so we don't cause memory leaks
  Module._free(result_ptr)
}
```

### Run javascript from rust

Using the [`emscripten_functions::emscripten::run_script`](emscripten-functions/src/emscripten.rs) family of functions you can run the javascript you need in your web app.

#### Example
```rust
// The `.escape_unicode()` method makes it safe to pass untrusted user input.
run_script(
    format!(
        r##"
            document.querySelector("#this-is-secure").innerHTML = "{}"
        "##,
        "untrusted user input".escape_unicode()
    )
);
```

### Main loop control

If you need to run a loop function over and over, emscripten has its own main loop managing system.
Using the [`emscripten_functions::emscripten::set_main_loop`](emscripten-functions/src/emscripten.rs) and [`emscripten_functions::emscripten::set_main_loop_with_arg`](emscripten-functions/src/emscripten.rs) functions you can run your rust functions as main loops, with full control over the main loop running parameters.

#### Example
```rust
struct GameData {
    level: u32,
    score: u32
}
let mut game_data = GameData {
    level: 1,
    score: 0
}

set_main_loop_with_arg(|data| {
    if data.score < data.level {
        data.score += 1;
    } else {
        data.score = 0;
        data.level += 1;
    }

    // Here you call your display to screen functions.
    // For demonstration purposes I chose `println!`.
    println!("Score {}, level {}", data.score, data.level);
}, game_data, 0, true);
```

### An SDL game example

An SDL game example that has image handling can be found in [`examples/simple-game`](examples/simple-game).
