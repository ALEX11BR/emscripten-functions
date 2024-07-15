#!/usr/bin/env sh

build_binding() {
    emscripten_headers="emscripten/cache/sysroot/include"

    bindgen "$emscripten_headers/emscripten/$1.h" --allowlist-file "$emscripten_headers/emscripten/"'.*' -o "src/$1.rs" -- -I"$emscripten_headers"
}

build_binding "emscripten"
build_binding "html5"
build_binding "console"
