use std::path::PathBuf;
use std::{env, fs};

fn main() {
  load_env();
  load_lib("ffmpeg", &["avcodec", "avformat", "avutil", "swscale"]);
  load_lib("libwebp", &["libwebp"]);
}

fn load_lib(name: &str, libs: &[&str]) {
  let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not found"));
  let lib_dir = name.to_uppercase();
  let lib_dir =
    PathBuf::from(env::var(&lib_dir).unwrap_or_else(|e| panic!("{lib_dir} env missing {e}")));
  println!(
    "cargo:rustc-link-search={}",
    lib_dir.join("lib").to_str().unwrap()
  );

  for lib in libs {
    println!("cargo:rustc-link-lib={lib}");
  }

  let headers = format!("{name}.h");
  println!("cargo:rerun-if-changed={headers}");
  println!("cargo:rerun-if-changed=.env");

  let bindings = bindgen::Builder::default()
    .header(headers)
    .clang_arg(format!("-I{}", lib_dir.join("include").to_str().unwrap()))
    // Tell cargo to invalidate the built crate whenever any of the
    // included header files changed.
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  bindings
    .write_to_file(out_dir.join(format!("{name}.rs")))
    .unwrap_or_else(|e| panic!("Unable to write {name} bindings {e}"));
}

fn load_env() {
  if let Ok(contents) = fs::read_to_string(".env") {
    for line in contents.lines() {
      let parts: Vec<_> = line.splitn(2, '=').collect();
      if parts.len() == 2 {
        let key = parts[0].trim();
        let value = parts[1].trim();
        env::set_var(key, value);
      }
    }
  }
}
