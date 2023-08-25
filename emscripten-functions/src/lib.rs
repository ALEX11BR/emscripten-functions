//! Various emscripten system functions that make programming in rust for emscripten targets easier.
//! They are grouped by the original function's header file.

#![cfg(target_os = "emscripten")]

pub mod emscripten;
pub mod console;
