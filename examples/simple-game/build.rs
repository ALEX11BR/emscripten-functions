fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "emscripten" {
        println!("cargo:rustc-link-arg=--use-port=sdl2");
        println!("cargo:rustc-link-arg=--use-port=sdl2_image:formats=png");
        println!("cargo:rustc-link-arg=--embed-file=assets");
    }
}
