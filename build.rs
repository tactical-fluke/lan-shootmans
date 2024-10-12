use std::env;
use std::path::{Path, PathBuf};
use fs_extra::dir::CopyOptions;

fn main() {
    println!("cargo:rerun-if-changed=assets");
    let out_dir = get_output_path();
    let options = CopyOptions::new().overwrite(true);
    fs_extra::copy_items(&["./assets"], out_dir, &options).expect("couldn't copy assets");
}

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join(build_type);
    PathBuf::from(path)
}
