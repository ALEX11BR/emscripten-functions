fn main() {
    // To allow docs.rs to build documentation for this emscripten-exclusive project,
    // we won't compile the C file in the docs.rs environment, as they don't have the emsdk,
    // and the C file isn't relevant to the documentation.
    if std::env::var_os("DOCS_RS").is_none() {
        cc::Build::new()
            .file("asm_in_main_thread.c")
            .compile("asm_in_main_thread");
    }
}
