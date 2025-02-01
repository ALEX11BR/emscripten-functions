//! ```cargo
//! [dependencies]
//! bindgen = "0.66.1"
//! regex = "1.9.3"
//! ```

use regex::escape;
use std::path::PathBuf;

fn build_binding(header_name: &str) {
    let emscripten_headers_path = PathBuf::from("emscripten/cache/sysroot/include");
    let out_path = PathBuf::from("src");

    bindgen::Builder::default()
        .header(
            emscripten_headers_path
                .join(format!("emscripten/{}.h", header_name))
                .to_string_lossy(),
        )
        .clang_arg(format!("-I{}", emscripten_headers_path.to_string_lossy()))
        // We're interested only in the functions & types defined in `emscripten` headers,
        // not in the ones from e.g. `stdlib.h`
        .allowlist_file(format!(
            "{}.*",
            escape(
                &emscripten_headers_path
                    .join("emscripten/")
                    .to_string_lossy()
            )
        ))
        //.parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect(&format!("Unable to generate bindings for {}", header_name))
        .write_to_file(out_path.join(format!("{}.rs", header_name)))
        .expect(&format!("Unable to write bindings for {}", header_name));
}

fn main() {
    build_binding("emscripten");
    build_binding("html5");
    build_binding("console");
    build_binding("websocket");
}
