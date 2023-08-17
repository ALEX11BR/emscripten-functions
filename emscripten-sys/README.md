# emscripten-sys

This crate contains bindgen-generated bindings for the system emscripten functions, as presented in [their API reference](https://emscripten.org/docs/api_reference/index.html).
The following headers have bindings available:
- `emscripten`
- `html5`
- `console`

## A little description of the files in this project

The bindings are based on the emscripten headers from a compiled emscripten release, like the ones at [https://storage.googleapis.com/webassembly/](https://storage.googleapis.com/webassembly/)emscripten-releases-builds/, that are downloaded by [emsdk](https://github.com/emscripten-core/emsdk).

The `emscripten` folder contains the headers taken from the emscripten release (currently at version 3.1.44).

The `build_bindings.rs` file that can be run with e.g. [rust-script](https://rust-script.org/) creates declarations for the emscripten functions using bindgen.

The `src` folder already contains the generated bindings.
