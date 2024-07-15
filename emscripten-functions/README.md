# emscripten-functions

This crate contains various emscripten system functions (made with rust-native parameter and return value types) that make programming in rust for emscripten targets easier.
Functions based on ones from the following emscripten headers are available:
- `emscripten`
- `console`

## Examples
For more examples and tips for emscripten in rust refer to my [main project's README](https://github.com/ALEX11BR/emscripten-functions/blob/main/README.md).

### Run javascript from rust

Using the [`emscripten_functions::emscripten::run_script`](src/emscripten.rs) family of functions you can run the javascript you need in your web app.

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
Using the [`emscripten_functions::emscripten::set_main_loop`](src/emscripten.rs) and [`emscripten_functions::emscripten::set_main_loop_with_arg`](src/emscripten.rs) functions you can run your rust functions as main loops, with full control over the main loop running parameters.

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
