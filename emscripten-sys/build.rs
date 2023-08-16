use regex::escape;
use std::{env, path::PathBuf};

fn build_binding(header_name: &str, emscripten_path: PathBuf) {
    let emscripten_headers_path = emscripten_path.join("cache/sysroot/include");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

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
            "^({}.*)$",
            escape(
                &emscripten_headers_path
                    .join("emscripten/")
                    .to_string_lossy()
            )
        ))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect(&format!("Unable to generate bindings for {}", header_name))
        .write_to_file(out_path.join(format!("{}.rs", header_name)))
        .expect(&format!("Unable to write bindings for {}", header_name));
}

fn main() {
    let emscripten_path = PathBuf::from(if cfg!(feature = "system-headers") {
        env::var("EMSCRIPTEN_ROOT").expect(
            "`EMSCRIPTEN_ROOT` should be set if compiling using the `system-headers` feature",
        )
    } else {
        "emscripten".to_string()
    });

    build_binding("emscripten", emscripten_path)
}
