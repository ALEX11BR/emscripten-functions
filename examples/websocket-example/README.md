# A simple websocket example

This example sends during the initialization a `Hello` to wss://echo.websocket.org/ and prints to the browser console the response.

It is meant as a sample of using the `websocket` module of the `emscripten-functions` crate.

## Notes for developers

### Building

For web builds we have a `Makefile`.
Just run `make build-web` or just `make` and you'll have the page with the game and everything needed in the `out` folder.
Make sure you have the Emscripten SDK in your `PATH` when compiling.

### Customizing the project name

You'll need to change it in the following places:
- `Makefile`: change the `PROJECT` variable
- `Cargo.toml`: change the `package.name`
- `src/main.rs`: change the title parameter given to the `window` function

### Customizing the HTML game shell

The default HTML shell shows the game on the entire page, with a little loading animation.

You can change the `shell.html` file to your liking.
Keep in mind that you'll need to keep the final `{{{ SCRIPT }}}` thing as this is where the emscripten js import is placed by the `Makefile` rule.

