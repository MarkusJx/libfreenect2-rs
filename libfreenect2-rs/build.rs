mod build_util;

use crate::build_util::libs::link_os_libs;
use crate::build_util::target_dir::TargetDir;
use crate::build_util::zipped_library::ZippedLibrary;
use std::path::Path;
use std::{env, io};

const USER: &str = "MarkusJx";
const REPO: &str = "libfreenect2-rs";

fn main() -> anyhow::Result<()> {
  // Make docs.rs build pass
  if env::var_os("DOCS_RS").is_some() {
    return Ok(());
  }

  let downloaded_file = ZippedLibrary::new(USER, REPO, "LIBFREENECT2_PATH").download()?;
  if let TargetDir::Path(path) = TargetDir::find() {
    copy_dir(&downloaded_file.include_path, path.join("include"))?;
  }

  println!("cargo:rerun-if-changed=src/ffi.rs");
  build(
    &[
      "libfreenect2",
      "frame",
      "freenect2_device",
      "config",
      "registration",
      "logger",
    ],
    &downloaded_file.include_path,
  );

  downloaded_file.link_libs()?;
  link_os_libs()?;

  Ok(())
}

fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
  std::fs::create_dir_all(&dst)?;
  for entry in std::fs::read_dir(src)? {
    let entry = entry?;
    let ty = entry.file_type()?;
    if ty.is_dir() {
      copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
    } else {
      std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
    }
  }
  Ok(())
}

fn build<P: AsRef<Path>>(files: &[&str], include_path: P) {
  let mut build = cxx_build::bridge("src/ffi.rs");
  build
    .include("../ffi/include")
    .include(include_path)
    .std("c++17");

  for file in files {
    build.file(format!("../ffi/src/{file}.cpp"));
    println!("cargo:rerun-if-changed=ffi/src/{file}.cpp");
    println!("cargo:rerun-if-changed=ffi/include/{file}.hpp");
  }

  if cfg!(feature = "opencl") {
    build.define("LIBFREENECT2_RS_WITH_OPENCL", None);
  }
  if cfg!(feature = "opengl") {
    build.define("LIBFREENECT2_RS_WITH_OPENGL", None);
  }

  build.compile("libfreenect2_ffi");
}
