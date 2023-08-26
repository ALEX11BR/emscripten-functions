//! Raw bindgen-generated rust bindings to emscripten's system functions.
//! They are grouped by their header file.

#![cfg(target_os = "emscripten")]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod console;
pub mod emscripten;
pub mod html5;
