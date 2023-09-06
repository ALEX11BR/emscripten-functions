fn main() {
    cc::Build::new()
        .file("asm_in_main_thread.c")
        .compile("asm_in_main_thread");
}
