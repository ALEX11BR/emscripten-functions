# A simple web-compatible game with Rust and SDL2

[**Web version demo**](https://alex11br.github.io/emscripten-functions/simple-game/)

This is a little game where you move an image across the screen with the arrow keys and change the background to black with Enter and to white with Space.

It is meant to serve as a starting point in building SDL2 games in Rust with web support via emscripten using the `emscripten-functions` crate.

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

### Images

The game images are in the `assets` folder.
For other formats than PNG you need to add support for the ones you need in:
- `build.rs` - change the `--use-port=sdl2_image:formats=png` linker argument to add support for your desired formats
- `src/main.rs` - add the desired formats' flags to the parameter of `sdl2::image::init`

For native (non-web) builds you'll need to run the executable in the folder where the `assets` are located.
This usually means copying the `assets` folder with the executable and the eventual dynamic libraries.
