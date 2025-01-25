# emscripten-functions
[![crates.io badge](https://img.shields.io/crates/v/emscripten-functions.svg)](https://crates.io/crates/emscripten-functions)

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

### Calling JavaScript functions using the val API

Using the [`emscripten-val`](https://github.com/MoAlyousef/emscripten-val) crate, you can make use of [emscripten's val API](https://emscripten.org/docs/api_reference/val.h.html) to call JavaScript from the Rust side.

#### Example
Example taken from the [`emscripten-val` README](https://github.com/MoAlyousef/emscripten-val/blob/main/README.md):
```rust
use emscripten_functions::emscripten::{run_script, run_script_int};
use emscripten_val::*;

fn main() {
    let a = Val::from_array(&[1, 2]);
    run_script(format!(
        r#"
        console.log(Emval.toValue({}));
    "#,
        a.as_handle() as i32
    ));

    a.call("push", argv![3]);
    run_script(format!(
        r#"
        console.log(Emval.toValue({}));
    "#,
        a.as_handle() as i32
    ));

    let handle = run_script_int("let n = new Number('123'); Emval.toHandle(n)");
    let number = Val::take_ownership(handle as EM_VAL);
    println!("{}", number.call("valueOf", &[]).as_i32());

    #[no_mangle]
    pub extern "C" fn event_handler(ev: EM_VAL) {
        let val = Val::take_ownership(ev);
        let target = val.get(&"target");
        target.set(&"textContent", &"Clicked");
    }

    let button = Val::take_ownership(run_script_int(
        r#"
        let button = document.createElement('BUTTON');
        button.addEventListener('click', (ev) => {
            _event_handler(Emval.toHandle(ev));
        });
        let body = document.getElementsByTagName('body')[0];
        body.appendChild(button);
        Emval.toHandle(button)
    "#,
    ) as EM_VAL);
    button.set(&"textContent", &"click");
}
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

### An SDL game example

An SDL game example that has image handling can be found [here](../examples/simple-game).
