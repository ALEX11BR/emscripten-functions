fn main() {
    if !std::env::var("DOCS_RS").is_ok() {
        cc::Build::new()
            .file("asm_in_main_thread.c")
            .compile("asm_in_main_thread");
    }
}
