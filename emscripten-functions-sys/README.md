# emscripten-functions-sys
[![crates.io badge](https://img.shields.io/crates/v/emscripten-functions-sys.svg)](https://crates.io/crates/emscripten-functions-sys)

This crate contains bindgen-generated bindings for the system emscripten functions, as presented in [their API reference](https://emscripten.org/docs/api_reference/index.html).
The following headers have bindings available:
- `emscripten`
- `html5`
- `console`

## A little description of the files in this project

The bindings are based on the emscripten headers from a compiled emscripten release, like the ones at [https://storage.googleapis.com/webassembly/](https://storage.googleapis.com/webassembly/)emscripten-releases-builds/, that are downloaded by [emsdk](https://github.com/emscripten-core/emsdk).

The `emscripten` folder contains the headers taken from the emscripten release (currently at version 4.0.1).

The `build_bindings.rs` file that can be run with e.g. [rust-script](https://rust-script.org/) creates declarations for the emscripten functions using bindgen.

The `build_bindings.sh` script does the same thing, and requires the bindgen CLI, obtainable e.g. using `cargo install bindgen-cli`.

The `update_emscripten.sh` script updates the headers in the `emscripten` folder to the latest version from the official docker image.
Set the `DOCKER` variable when running this script to specify a path to your docker.
Otherwise `docker`, then `podman` will be tried.

The `src` folder already contains the generated bindings.
