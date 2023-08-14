use std::path::PathBuf;
use std::{env, fs};

fn main() {
  load_env();

  println!(
    "cargo:rustc-link-search={}",
    env::var("FFMPEG_LIB").expect("FFMPEG_LIB env missing")
  );
  println!("cargo:rustc-link-lib=avformat");
  println!("cargo:rustc-link-lib=avutil");
  println!("cargo:rerun-if-changed=wrapper.h");
  println!("cargo:rerun-if-changed=.env");

  let bindings = bindgen::Builder::default()
    .header("wrapper.h")
    .clang_arg(format!(
      "-I{}",
      env::var("FFMPEG_INC").expect("FFMPEG_INC env missing")
    ))
    // Tell cargo to invalidate the built crate whenever any of the
    // included header files changed.
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
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

  // Now you can access the environment variables as before
  if let Ok(value) = env::var("MY_VARIABLE") {
    println!("Value of MY_VARIABLE: {}", value);
  } else {
    println!("MY_VARIABLE is not set.");
  }
}
